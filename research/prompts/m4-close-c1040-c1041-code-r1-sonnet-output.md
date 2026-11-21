# 正推腿报告：m4-close-c1040-c1041-code-r1

日期：2026-11-18。立场：正推（从已定条款与 `crates/` 的实现推「应该是什么」，再与被判对象逐格比）。

## 开头：基线、模型与一句话结论

- 基线：`sha256sum -c research/prompts/m4-close-c1040-c1041-code-r1-snapshot/crates-sha256.txt` 按 OK / 非 OK 数得 `OK 17`，没有 FAILED：被判的 17 份文件此刻与开工快照逐字节相同，下面的行号都是现取的（命令 `grep -nF`，行号取命中的那一行）。
- 模型：`research/prompts/m4-close-c1040-c1041-code-r1-sonnet-model/`，纯 Python 标准库，只核证明那一块（第三节）。复跑命令在末尾「要主 agent 跑的命令」。sha256（量的，`sha256sum`）：
  - `intersection_length_model.py` 5820839ea7a978238189a9ebab62d70a84f5cd37950636f51034c281cb6d3cff
  - `tools/rerun.sh` 067b74fba3676a61a63f9cfa52da27cb8ddde13b39fd6b645788897229399e93（从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`）
  - `tools/rerun-from-a-clean-work-directory.sh` e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b
  - `out/model.out` 3cad80048e6f9e8bf3d453dec9ea6505f5572940bee9b1835d785602493a1829（第一次、也是仅有的一次直接跑的原样输出；跑完之后项目代码闸拒了再直接起 `python3`，所以没有第二次，见「没做什么」）
- 探针的内存与时间：没量。模型是单进程纯 Python，最大的一格是 13 的 5 次方（约 37 万）个输入组合，没有外部进程（推的）。
- 一句话结论：
  1. C1041 四格：偏移、值、判法与次序在 core、池级 checker、验证包两处与字段表、I-7.13 一致，**成立**；有两处条文没写的差异（S 越界与四格同在一槽时两边点名的成员不同；GPU 判器没同判，条文已写欠账），结局都是整池拒，不改判定。
  2. C1040：可写挂载三个入口都走到 `mount.rs:7616` 那一比，**成立**；mkfs 的判在任何写之前，**成立**；只读挂载没判，与 D12 第 69、75、103 行「可写与只读都拒」的说法**不成立**（条款写的实现边界与代码对不上）。
  3. 证明：规格是精确交集长度，两条引理的前提在「另一端任意」的意义上就是最弱的（只多出倒置区间那一种退化），后置条件不空真；8 行新必红与 3 行 `proof_errors` 改 2 每行的落点与登记相同（任务 20208 日志转述），**成立**；一处缺口：新加的两处 None 支 `== 0` 没有专属必红行。
  4. 八格：六格成立，「走不到的理由」部分成立（字面没有新 `assert!`，但新增 4 处会 panic 的调用点，输入打不到），「带 --item」一格有两处推不出（待主 agent 列项）。
  5. 已知红测试：从代码推该判「测试判法要换」；探测由 `caller_inputs_agreeing_with_the_disk` 在每次用户改动入口之前做，是「照可写挂载同一套核」的设计结果。条款层面「每次入口都要重探」没有一句话明说（D21 已定项 16 写「挂载时求值一次」），那一半推不出，交主 agent。

## 关 1：C1041 四格（偏移 305、309、313、321）

### 1.1 偏移与值：一致

- 字段表偏移：「单元大小 305 / 落点粒度 309 / 位置条目宽度 313 / mkfs 时的 physical_block_size 317 / 扩展点声明值 N 321」（`.claude/kb/layout/01-first-txn.md:160`）。
- 字段表的值：「| 几何 | 单元大小 | 4 | 32768 |」（`.claude/kb/layout/01-first-txn.md:128`）、「| 几何 | 落点粒度 | 4 | 16384 |」（`.claude/kb/layout/01-first-txn.md:129`）、「| 几何 | 位置条目宽度 | 4 | 14 |」（`.claude/kb/layout/01-first-txn.md:130`）、「| 几何 | 扩展点声明值 N | 4 | 0 |」（`.claude/kb/layout/01-first-txn.md:132`）。
- D22 已定项 9 点名这两项在几何段：「几何段有 mkfs 时的 `physical_block_size` 4 与扩展点声明值 N 4（宽度都按字节计）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:232`）；N 的值：「第一版 N = 0」（`.claude/kb/decisions/21-权威态与派生态的分界.md:173`）。
- 格式常量：「pub const SLOT_BYTES: u64 = 16384;」（`crates/singlefs-format/src/lib.rs:14`）、「pub const DATA_UNIT_BYTES: u64 = 32768;」（`crates/singlefs-format/src/lib.rs:20`）、「pub const LOC_ENTRY: u64 = 14;」（`crates/singlefs-format/src/lib.rs:77`）。
- core 与 checker 各写一份偏移，都是 305、309、313、321：「const UNIT_BYTES_OFFSET: u64 = 305;」（`crates/singlefs-core/src/system_configuration.rs:52`）、「const SLOT_GRANULARITY_BYTES_OFFSET: u64 = 309;」（`crates/singlefs-core/src/system_configuration.rs:53`）、「const LOCATION_ENTRY_BYTES_OFFSET: u64 = 313;」（`crates/singlefs-core/src/system_configuration.rs:54`）、「const EXTENSION_POINT_DECLARED_BYTES_OFFSET: u64 = 321;」（`crates/singlefs-core/src/system_configuration.rs:55`）；checker：「const SYSTEM_CONFIGURATION_UNIT_BYTES_OFFSET: usize = 305;」（`crates/singlefs-checker/src/image.rs:1039`）、「const SYSTEM_CONFIGURATION_SLOT_GRANULARITY_BYTES_OFFSET: usize = 309;」（`crates/singlefs-checker/src/image.rs:1040`）、「const SYSTEM_CONFIGURATION_LOCATION_ENTRY_BYTES_OFFSET: usize = 313;」（`crates/singlefs-checker/src/image.rs:1041`）、「const SYSTEM_CONFIGURATION_EXTENSION_POINT_DECLARED_BYTES_OFFSET: usize = 321;」（`crates/singlefs-checker/src/image.rs:1042`）。
- 写侧另有位置断言，输入打不到：「writer.assert_position(UNIT_BYTES_OFFSET, "单元大小");」（`crates/singlefs-core/src/system_configuration.rs:677`）到「writer.assert_position(EXTENSION_POINT_DECLARED_BYTES_OFFSET, "扩展点声明值 N");」（`crates/singlefs-core/src/system_configuration.rs:684`）：写到的位置等于字段表偏移，写错字段表当场 panic（见第四节第一格）。

### 1.2 判法（每格的比较）：一致

- core：「if u64::from(recorded.unit_bytes) != DATA_UNIT_BYTES {」（`crates/singlefs-core/src/recovery.rs:1537`）、「if u64::from(recorded.slot_granularity_bytes) != SLOT_BYTES {」（`crates/singlefs-core/src/recovery.rs:1545`）、「if u64::from(recorded.location_entry_bytes) != LOC_ENTRY {」（`crates/singlefs-core/src/recovery.rs:1553`）、N：「if recorded.extension_point_declared_bytes」（`crates/singlefs-core/src/recovery.rs:1561`）接着 `!= EXTENSION_POINT_DECLARED_BYTES_OF_THE_FIRST_VERSION`（`crates/singlefs-core/src/system_configuration.rs:58` 定为 0）。
- checker：「if u64::from(read_u32(slot, SYSTEM_CONFIGURATION_UNIT_BYTES_OFFSET)) != DATA_UNIT_BYTES {」（`crates/singlefs-checker/src/image.rs:1067`）、`!= SLOT_BYTES`、「) != LOC_ENTRY」（`crates/singlefs-checker/src/image.rs:1080`）、N：「) != EXTENSION_POINT_DECLARED_BYTES_THIS_CHECKER_READS」（`crates/singlefs-checker/src/image.rs:1087`），常量「const EXTENSION_POINT_DECLARED_BYTES_THIS_CHECKER_READS: u32 = 0;」（`crates/singlefs-checker/src/image.rs:1044`）。
- 两边都是「4 字节读成 u32、前三格放宽成 u64 与格式常量比不等、N 与 0 比不等」，同值域同结局；core 与 checker 各自写常量与偏移，没有共用函数。

### 1.3 次序：成立，另有一处条文没写的差异

次序表（行号现取）：

| 步 | core（`system_configuration_values_this_reader_accepts`） | 池级 checker（`read_system_configuration_slot`） |
|---|---|---|
| 解槽那几关 | `parse_slot` 先出 incompat 不认得、节点大小不等、S 越界：「.map_err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange)?;」（`crates/singlefs-core/src/system_configuration.rs:791`），再调 `system_configuration_values_this_reader_accepts`（`crates/singlefs-core/src/recovery.rs:1706`） | `check_system_configuration_slot`：格式版本「!= SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS」（`crates/singlefs-checker/src/lib.rs:489`）、incompat「return Err(Verdict::UnknownIncompatBit);」（`crates/singlefs-checker/src/lib.rs:498`）、节点大小「return Err(Verdict::NodeBytesNotTheFormatConstant);」（`crates/singlefs-checker/src/lib.rs:501`）、加密类型「return Err(Verdict::EncryptionTypeNotOff);」（`crates/singlefs-checker/src/lib.rs:505`） |
| 格式版本 | 「if format_version != crate::system_configuration::FORMAT_VERSION {」（`crates/singlefs-core/src/recovery.rs:1426`） | 同上，在 `check_system_configuration_slot` 里 |
| 加密类型 | 「if encryption_type != SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF {」（`crates/singlefs-core/src/recovery.rs:1434`） | 同上 |
| **四格** | 「unit_geometry_fields_of_the_format(slot)?;」（`crates/singlefs-core/src/recovery.rs:1441`） | 「if let Some(disagreeing_field) = unit_geometry_field_that_disagrees(&bytes) {」（`crates/singlefs-checker/src/image.rs:1258`） |
| 槽距等 | 「let fixed_structure_slot_spacing = u64::from(sizes.fixed_structure_slot_spacing);」（`crates/singlefs-core/src/recovery.rs:1443`） | 「match geometry_of(&bytes, &view) {」（`crates/singlefs-checker/src/image.rs:1264`），S 在它里面：「return Err(crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval);」（`crates/singlefs-checker/src/image.rs:865`） |

- 与 I-7.13 写的次序一致：「core 在 `system_configuration_values_this_reader_accepts` 里加密类型之后、槽距之前按字段表次序判」（`crates/singlefs-checker/invariants/02-根环.md:20`）；checker 一侧同一行写「`check_system_configuration_slot` 之后 `geometry_of` 之前」，与 `image.rs:1254`、`image.rs:1258`、`image.rs:1264` 的先后相同。
- **差异（条文没写）**：一槽同时带「每区槽数 S 越界」与「四格之一不等」时，core 先报 S（`parse_slot`，`system_configuration.rs:791`，经 `recovery.rs:1735` 交 `RootRingSlotsPerRegionOutOfRange`），checker 先报四格（`image.rs:1258` 在 `geometry_of` 之前、S 在 `image.rs:865`）。两边结局同是整池拒、整池不作保，只是点名的成员不同。同类的已记差异：「根环起点写 0 那一格两边点名的成员不同」（`crates/singlefs-checker/invariants/02-根环.md:20`）。新用例每条只改一格（`system_configuration_unit_geometry_fields_are_refused_by_core_and_the_pool_checker.rs` 的 `assert_core_and_the_pool_checker_refuse_the_field_rewritten_at`），没有钉两格同在、也没有钉四格之间谁先；没有变异行换这两种次序（17 行变异里第 1 至 10 行都是把某一格或整个调用关掉，没有一行换次序）。推（没跑）：交换四格之间的次序、或把四格挪到 S 之前，现有 17 行变异与新用例都不红；对整池拒的结局没有影响。
- 推翻它的现象：有一条条款或用例要求两边在两格同在的槽上点名同一个成员。我没找到这样的句子（`grep -n` I-7.13 一行与 D22 已定项 16 第 1 条，只有「按字段表次序」与「先判不收之值、再核偏移」两句）。

### 1.4 读槽的每一条路都过四格：成立（grep 核过）

- core 只有一处调 `system_configuration_values_this_reader_accepts`（`crates/singlefs-core/src/recovery.rs:1706`，在 `mountable_slot_or_refusal`，`recovery.rs:1686` 起）；`mountable_slot_or_refusal` 被「每档找槽 1」「槽 0」「槽 1」三处调（`recovery.rs:1780`、`recovery.rs:1918`、`recovery.rs:1935`）。D22 已定项 16 要求逐档读到的槽也先判不收之值：「逐档读到自证过的槽先判它带不带这一版读者不收之值」（`.claude/kb/decisions/22-单元原子性怎么合成.md:365`），代码满足。
- 命令 `grep -rn 'geometry_of(' crates --include=*.rs`（我跑的，只读）：非测试源码里调 `singlefs_checker::image::geometry_of` 的只有三处：`crates/singlefs-checker/src/image.rs:1264`、`crates/singlefs-verification/src/verification_read.rs:97`、`crates/singlefs-verification/src/memory_pool.rs:760`，三处前面都有四格判（`image.rs:1258`、`verification_read.rs:94`、`memory_pool.rs:758`）。别的命中是测试文件，或另一个同名函数 `fixed_geometry_of`、`geometry_of(&parameters)`（装置自己的函数）。这句只覆盖「调 `geometry_of`」的路径，没有核 GPU 内核。
- 离线修复也走 `choose_system_configuration_of_the_pool`（`crates/singlefs-core/src/system_configuration_repair.rs:246`），经同一个 `mountable_slot_or_refusal`。
- GPU 判器：条文自己写着欠账：「这四格还没同判，欠在 C1041」（`crates/singlefs-checker/invariants/02-根环.md:20`）。四格只会被不是 `to_slot` 写的槽带上（写侧只有两处调 `to_slot`：`crates/singlefs-core/src/make_filesystem.rs:831`、「let slot_bytes = system_configuration.to_slot();」（`crates/singlefs-core/src/transaction.rs:739`），四格都照常量写：`system_configuration.rs:677` 起），崩溃枚举出的盘面里不会有这种槽（推的，没跑）；所以 GPU 与 CPU 判器不会在枚举状态上判得不同，但对手工改过四格的镜像，GPU 与 CPU 判得不同，这一半条文已承认。

### 1.5 验证包两处：成立；不是只为别的池放宽

- `verification_read.rs:94`：「if singlefs_checker::image::unit_geometry_field_that_disagrees(slot).is_some() {」（`crates/singlefs-verification/src/verification_read.rs:94`）紧跟在「let view = singlefs_checker::check_system_configuration_slot(slot).map_err(|_| ())?;」（`crates/singlefs-verification/src/verification_read.rs:93`）之后、「let geometry = singlefs_checker::image::geometry_of(slot, &view).map_err(|_| ())?;」（`crates/singlefs-verification/src/verification_read.rs:97`）之前，次序与 checker 的 `image.rs:1254` 到 `image.rs:1264` 相同；不等交 `Err(())`，与 `geometry_of` 失败、`journal_ring_geometry_field_that_disagrees` 失败（`crates/singlefs-verification/src/verification_read.rs:105`）同一个处置，没有新增一种放行。
- `memory_pool.rs:758`：「singlefs_checker::image::unit_geometry_field_that_disagrees(&slot).is_none()」（`crates/singlefs-verification/src/memory_pool.rs:758`）放在 `.ok()` 之后、`geometry_of` 之前；不等时退到 `SlotZeroNotSelfDescribing`，它的两个方法取「环起点」与「从环起点罩到盘尾」（`crates/singlefs-verification/src/memory_pool.rs:731`，类型 `RecordedGeometryForScanCandidates`）：候选集合只会变大（宁多不少），不会少读一槽、不放行任何槽。所以这一支没有为别的池的槽放宽什么。
- `verification_read.rs:174` 起的 `system_configuration_slot_bytes_count_as_a_slot_of_another_pool` 不看四格：别的池的槽带四格不等照算「别的池的槽」，与它文件头写的「带这一版读者不收的值的别的池的槽同样算」一致；本池的槽四格不等时，两个分类都不计它（与已有的区域数、加密类型越界同一种处置）。


## 关 2：C1040（物理块大小那一半）

### 2.1 可写挂载三个入口都走到 `mount.rs:7616` 那一比：成立（读代码走通；测试只钉了第一个入口）

- 比对本体：「fn physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration<」（`crates/singlefs-core/src/mount.rs:7616`）逐盘「let probed_physical_block_size = device.probe_physical_block_size();」（`crates/singlefs-core/src/mount.rs:7627`），不等的各报一项（`!=`，不是 `<`、`>`：`mount.rs:7628`）；它由 `device_table_disagreeing_with`（`mount.rs:7566`）末尾调（`mount.rs:7603`）。`device_table_disagreeing_with` 只有两个调用点：「device_table_disagreeing_with(devices, &system_configuration);」（`crates/singlefs-core/src/mount.rs:981`）与「let disagreeing_device_table = device_table_disagreeing_with(devices, system_configuration);」（`crates/singlefs-core/src/mount.rs:7710`）。
- 入口一，可写挂载：`mount_writable`（`mount.rs:9105`）的各变体落到 `mount_writable_with_every_test_only_switch`（`mount.rs:9902`；`mount_writable*` 公开函数 12 个，对它有 7 处调用，没有逐个追链，其余几个按名字推经别的变体），它在读任何盘之前调「parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table(」（`crates/singlefs-core/src/mount.rs:9922`，函数本体在 `mount.rs:7643`，其中 `mount.rs:7710` 调盘表那一比），拒绝经 `mount.rs:690` 转成 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`。
- 入口二，挂着之后收盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载、管理员回退）：它们都先调 `caller_inputs_agreeing_with_the_disk`（`mount.rs:7749`，其中「parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table(」在 `mount.rs:7760`，再到 `mount.rs:7710`）。调用点：`mount.rs:4794`、`mount.rs:5008`、`mount.rs:5358`、`mount.rs:7819`、`mount.rs:10929`（`grep -nE 'caller_inputs_agreeing_with_the_disk\(' crates/singlefs-core/src/mount.rs` 的命中，我数了五处调用）。文档：「挂着之后收调用方参数与盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载）与管理员回退照可写挂载同一套核」（`crates/singlefs-core/src/mount.rs:7730`）。**不读盘、不写盘那一格也先核**：「上限不高于 F、一个字节都不写的那一格也先核。」（`crates/singlefs-core/src/mount.rs:4779`）。
- 入口三，`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`（`mount.rs:975`）直接在 `mount.rs:981` 调。
- 不经这三个入口的写系统配置槽的路：离线修复 `rewrite_system_configuration_slots_for_the_asserted_version`（`crates/singlefs-core/src/system_configuration_repair.rs:240`）只选系统配置（`system_configuration_repair.rs:246`）、没有盘表那一比。条款对它的说法：「离线修复没有探测值，带生效表」（`.claude/kb/decisions/22-单元原子性怎么合成.md:226`）；D12 第 4 项写的是「拒绝挂载」，离线修复不是挂载。要不要让它也比探测值：**推不出**（条款没写）。
- 测试覆盖：测试钉的只有入口一。`writable_mount_refuses_a_device_whose_probed_physical_block_size_differs_from_the_system_configuration.rs` 的两条用例都经 `mount_writable`；入口二、三的现有测试文件 `entries_after_mount_refuse_swapped_or_behind_devices.rs` 只在穷举 `match` 里多了一臂（「DeviceTableDisagreement::PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration」，`crates/singlefs-verification/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:246`），这一臂是「只该是本盘设备号不符」的 panic 分支，不是对探测值不符的钉；变异行 11 改的是共用函数（「(false && probed_physical_block_size.0 != …」），只有入口一的两条用例去抓它。D12 的射程要求「每一条都要能被测试强制走进「对不上」那条分支」（`.claude/kb/decisions/12-目标介质.md:90`）：比对本身满足；每个入口各自满足与否，**推不出**——某个入口以后绕开 `device_table_disagreeing_with`，现有 17 行变异与新用例都不红（推的，没跑）。

### 2.2 mkfs 的判在任何写之前：成立

- `make_filesystem`（`crates/singlefs-core/src/make_filesystem.rs:632`）里：「let unit_area_start = check_geometry(parameters, &sizes)?;」（`crates/singlefs-core/src/make_filesystem.rs:647`），紧接着「first_device_probing_a_physical_block_size_other_than_the_parameter(parameters, devices)」（`crates/singlefs-core/src/make_filesystem.rs:649`）出 `Err`；第一次对盘的写是「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」（`crates/singlefs-core/src/make_filesystem.rs:729`）。`grep -n 'devices\b' make_filesystem.rs` 在 632 到 735 之间命中 634、637、643、649、702（注释）、728、732、735：649 与 728 之间没有对 `devices` 的使用。`:608` 的函数只调 `probe_physical_block_size`（`make_filesystem.rs:614`），不读不写。
- 对外入口都经它：「let output = make_filesystem(&parameters, devices)?;」（`crates/singlefs-core/src/make_filesystem.rs:315`），之前只从随机源取 fsid。
- 比较方向与盘数：`!=`，对 `devices` 里每一块按盘表次序取第一块不等的（`make_filesystem.rs:615`）；参数与探测都是 `u32`（`PhysicalBlockSizeInBytes(pub u32)`，`crates/singlefs-core/src/block_device.rs:132`）。
- **条款与实现之间有一处张力，推不出谁管**：D22 已定项 2 写「mkfs 按池内 `max(physical_block_size)` 划（调用方给的 `physical_block_size` 与 mkfs 时探测到的不同就拒绝 mkfs）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:63`）。实现按「与每块盘的探测值都相等」判：两盘探测值不同时，给 max 的参数也会因另一块盘被拒，异构池 mkfs 不出来。D12 已定项 5 写第一版「各盘互比」（「已定项 4 的布局位与 `physical_block_size` 两条比对在第一版退化成各盘互比」，`.claude/kb/decisions/12-目标介质.md:103`），与实现一致，与 D22 那句「按 max 划」的字面不一致。同一行 D22 第 63 行对挂载写「未达到拒绝挂载」，实现对记录值大于探测值也拒（`!=`），D12 第 75 行的「对不上」是 `!=`。第一版两盘同构下三者给出同一个结局；异构盘上不同。

### 2.3 只读挂载没判与 D12 第 103 行那一句：不成立（条款写的是实现边界，代码对不上）

- 命令（我跑的，只读）`grep -n 'probe_physical_block_size' crates/singlefs-core/src/mounted_read.rs crates/singlefs-core/src/recovery.rs`：零命中（`grep` 退出码 1）。原因在读者类型：「pub trait PoolReader {」（`crates/singlefs-core/src/recovery.rs:129`）的方法是 `device_identities`、`device_size_in_bytes`、`read`、`journal_record_offsets_hint`、`reread_after_a_bad_read`，没有探测；只读挂载拿 `&dyn PoolReader`（`crates/singlefs-core/src/mounted_read.rs:651`）。
- 条款三处都写成「可写与只读都拒」：「**实现边界**：设备身份、布局位与探测到的 physical_block_size 已与记录比（可写与只读都拒；mkfs 时参数与探测不同也拒）」（`.claude/kb/decisions/12-目标介质.md:69`）；表里的规则行「| `physical_block_size` | **拒绝挂载**（可写与只读都拒）」（`.claude/kb/decisions/12-目标介质.md:75`）；已定项 5 的「代码已做这一比（可写与只读挂载都拒；mkfs 时参数与探测不同也拒）」（`.claude/kb/decisions/12-目标介质.md:103`）。这三处在工作区里是未提交的新写（`git diff HEAD -- .claude/kb/decisions/12-目标介质.md` 里是 `+` 行）。
- 判：第 75 行是规则，代码欠只读一侧；第 69、103 行是状态，写错了（只读一侧没做）。实现员第二份报告也停在这一格：「只读挂载比探测值：停下，交主 agent」（`~/.local/share/singlefs/work/m4-close-c1040-c1041/result-batch-2.md`，第 11 行）。出路是主 agent 定的：补只读一侧（实现员列了两种加法，各卡一处），或把第 69、103 行改成「只可写挂载比」并把第 75 行的「只读也拒」标成欠。
- 同类的陈旧写法：D21 已定项 16 的「**实现边界**：挂载时那组判定不存在（C1041（系统配置四格几何读者整段跳过））；物理块大小不探测比对」（`.claude/kb/decisions/21-权威态与派生态的分界.md:330`）两半都过期了一半：可写挂载现在比物理块大小；四格已判。`checks-owed.md:642` 写「`physical_block_size` 那一半已实现」，没写只读一侧没做。以上三处是写回的事，不属于这一轮的代码判决，列给主 agent。
- 注释滞后：`caller_inputs_agreeing_with_the_disk` 的文档说「盘表比设备数与每块盘的本盘设备号」（`crates/singlefs-core/src/mount.rs:7731`），没写这一轮加的探测值那一比。

## 关 3：文件区间交集长度证明

模型数都出自 `research/prompts/m4-close-c1040-c1041-code-r1-sonnet-model/out/model.out`（第一次直接跑的原样输出，推的以外的数都在这里；判决要引的数由主 agent 复跑），整数格 `[-4,9)`（含空、倒置、负数）。

### 3.1 规格：成立

- 规格：「pub open spec fn range_intersection_length(」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:157`），体是「最大起点与最小终点之间的正差，否则零」：「if intersection_start < intersection_end { intersection_end - intersection_start } else { 0 }」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:162`）。`int` 上的数学运算，没有 u64 回绕。
- 模型：`name=spec_equals_bytewise_intersection inputs=28561 grid=[-4,9) bad=0`：规格等于「两个半开区间逐字节交集的元素个数」，空区间、倒置区间、不相交都是零。

### 3.2 后置条件有没有空真：没有空真

- `bytes_in_unit`：None 支加了「`range_intersection_length(…) == 0`」（`file_range_proofs.rs:171`），Some 支加了「length == range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int)」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:176`）。`checked_bytes_in_indexed_unit` 同形，端点用 `unit_index * capacity` 与 `unit_index * capacity + capacity`（`file_range_proofs.rs:221`、`file_range_proofs.rs:226`），在外层 Some 的前提「unit_index * capacity + capacity <= u64::MAX」之下（两处函数体相对 HEAD 没变，补丁只动了 `ensures`，`changes.patch` 里 `file_range_proofs.rs` 的 hunk 只有规格、后置条件与三个 proof fn）。
- 模型（执行函数照 Rust 写成 Python，`name=…` 行）：
  - `bytes_in_unit_postcondition inputs=6561 some_branch=876 none_branch=5685 violations=0`：新旧后置条件在全部输入上都满足，两支都有输入，不是「只在一支上有内容」。
  - `bytes_in_unit_new_conjunct_is_not_vacuous plus_one_fails_on_every_some_input=True none_conjunct_equivalent_to_old_forall=True`：把长度那一句改成 `+ 1` 在每个 Some 输入上都不成立（必红行靠得住）；**None 支新加的 `== 0` 与旧的 `forall … !in_both_ranges` 逐输入等价**——不空真，也没带新信息，它的价值只是把规格接到 None 支上、给下游一条没有量词的事实（推的）。
  - `checked_bytes_in_indexed_unit_postcondition outer_none=2816 inner_none=6445 inner_some=1491 violations=0`（上界取 15 的小模型，外层 None 是地址溢出）三支都有输入、没有违反。
- 消费方：「length_in_bytes」取元组第三项（`crates/singlefs-core/src/write_request_split.rs:383` 起的 `bytes_written_into`），返回行为不变；README 写的射程（只涉字节区间与返回长度，不代表缓冲区拷贝、设备 I/O 或持久化顺序）与代码相符：新加的东西都在 `verus!` 的规格里。

### 3.3 两条组合引理的前提：够弱（只多一种倒置的退化）

- 单元分界引理：「requires unit_start <= boundary <= unit_end,」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:348`）；请求分界引理：「requires write_start <= split <= write_end,」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:360`）。
- 模型：
  - 前提内都成立：`unit_split_lemma_under_premise inputs=76895 holds=76895`、`write_split_lemma_under_premise inputs=76895 holds=76895`。
  - 前提不能去：`unit_split_lemma_premise_dropped counterexamples=61360`、`write_split_lemma_premise_dropped counterexamples=61360`。手算一个：单元 [0,4)、分界 6、请求 [0,10)：整体长 4，分界两侧是 6 与 0，4 ≠ 6。所以前提不是「用不到的前提」。
  - 对「另一端任意」成立的最弱前提：`unit_split_weakest_premise triples_registered_premise_misses=442 all_misses_are_inverted_unit_ue_le_b_le_us=True registered_premise_always_sufficient=True`，请求分界引理同形（`…all_misses_are_inverted_write_we_le_s_le_ws=True`）。意思是：登记的前提总够用；比它更弱的只有「单元（或请求）本身倒置、分界落在倒置区间之间」那一种退化，实际调用的单元区间 `end = start + capacity`（`checked_unit_byte_range`，`file_range_proofs.rs:198` 起）不倒置。登记的前提在实用意义上就是最弱的。
- 引理只被见证函数调用（`grep -rn 'intersection_lengths_add_up' crates --include=*.rs -l` 只命中 `file_range_proofs.rs`），没有别的证明用到它们；它们是库里的引理，现在没有下游。

### 3.4 见证函数：成立，调用对十一条等式没有贡献

- 十一条等式（`file_range_proofs.rs:371` 到 `file_range_proofs.rs:381` 一带）：模型 `witness_facts count=11 bad=0`；六处引理调用的前提都满足：`witness_lemma_call_premises unit_calls=3 write_calls=3 all_premises_hold=True`。
- 十一条等式都是闭式求值，删掉六处引理调用它们照样成立（`witness_lemma_calls_needed_by_the_eleven_facts needed=no`，推的）：这六处调用只证引理前提可满足。必红行 `intersection-length-witness-sum` 只改一条等式，没有一行去掉六处调用，所以「见证里的引理调用」没有被必红行盯着（对安全无影响，列出来）。

### 3.5 八行新必红与三行改 2：每行红在点名的那一格

证据是任务 20208 的日志（`task-server/task log 20208 --all`，第二次尝试；转述，日志在任务服务里）里每行一句「绿，复用（2026-11-18 判绿：落点 <函数>；证明错 N 条）」，门禁判的是落点集合与第六段相等（`scripts/gate/verus-proof-check.py` 文件头：「判的是落点，不是条数」），条数只报。原文都恰好命中一次由门禁一并判了。

| 行（`verus-must-red-worlds.tsv` 行号） | 改坏什么 | 日志里的落点 | 条数 | 我的核 |
|---|---|---|---|---|
| `intersection-length-contract-lemma_adjacent_unit_intersection_lengths_add_up`（402） | 在 `ensures` 前塞 `false,` | `lemma_adjacent_unit_intersection_lengths_add_up` | 1 | 与登记相同；`false` 只红它自己，调用方（见证）拿到 `false` 不再多红 |
| `…lemma_split_write_intersection_lengths_add_up`（403） | 同上 | `lemma_split_write_intersection_lengths_add_up` | 1 | 同上 |
| `…witness_intersection_length_partitions`（404） | 同上 | `witness_intersection_length_partitions` | 1 | 同上 |
| `intersection-length-unit-split-needs-valid-boundary`（405） | 删 `requires unit_start <= boundary <= unit_end,` | `lemma_adjacent_unit_intersection_lengths_add_up` | 1 | 红的是前提：模型有 61360 个反例；见证里的六处调用不受影响（前提都满足） |
| `intersection-length-write-split-needs-valid-boundary`（406） | 删请求分界前提 | `lemma_split_write_intersection_lengths_add_up` | 1 | 同上 |
| `write-intersection-exact-length`（407） | `bytes_in_unit` Some 支 `length == …` 改成 `… + 1` | `bytes_in_unit` | 1 | 模型：每个 Some 输入上都不成立；下游 `checked_bytes_in_indexed_unit` 照理会拿到相反的事实，日志里落点集合只有 `bytes_in_unit`，所以 Verus 没在下游多报（我推不出为什么，只引日志） |
| `indexed-intersection-exact-length`（408） | `checked_…` Some 支改成 `+ 1` | `checked_bytes_in_indexed_unit` | 1 | 同上，上游函数没动 |
| `intersection-length-witness-sum`（409） | 见证第一条等式 `== 4` 改 `== 5` | `witness_intersection_length_partitions` | 1 | 闭式等式，红在见证自己 |
| `indexed-intersection-does-not-drop-written-bytes`（324，1 改 2） | 把 `bytes_in_unit` 的结果换成 `None` | `checked_bytes_in_indexed_unit` | 2 | 模型：这一改动在 1491 个输入上同时破坏旧的 `forall` 与新加的 None 支 `== 0`，`of_which_the_new_length_zero_conjunct_breaks_too=1491`，一处两条，与 2 吻合 |
| `overwrite-split-needs-valid-boundary`（399，1 改 2） | 删 `0 <= split <= data.len(),` | `lemma_splitting_an_overwrite_preserves_the_result` | 2 | 这两行在 `partial_write_proofs.rs`，这一轮没改它；更早的运行已经是 2 条：「must-red:overwrite-split-needs-valid-boundary：绿，复用（2026-11-17 判绿：落点 lemma_splitting_an_overwrite_preserves_the_result；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））」（任务 19000 日志，转述）。改 2 是把登记的数对齐到观测，与这一轮的 `file_range_proofs.rs` 改动无关 |
| `overwrite-contained-needs-payload-bounds`（400，1 改 2） | 删 `overwrite_fits(data, offset, replacement),` | `lemma_a_contained_overwrite_folds_into_the_payload` | 2 | 同上 |

- 日志写明这 15 行里 14 行是复用、只有 `write-intersection-rejects-touching` 现判（「按条（crates/singlefs-replay-prefix）：现判 1、复用 14」）；8 行新必红的 Verus 红例是更早的判定（日期 2026-11-18），不是 20208 这一趟现判的。复用键里含改坏的原文、替换文与第六段落点，不含第五段条数（`.claude/gate.d/formal-verification.sh:41`「必红世界每行的键里是改坏的原文、替换文与第六段落点（landing=），第五段条数只报、不进键」），键相同才复用。
- **缺口**：两处新加的 None 支 `== 0`（`file_range_proofs.rs:171`、`file_range_proofs.rs:221`）没有一行必红行专门盯它（只有 `indexed-intersection-does-not-drop-written-bytes` 顺带踩到，且与旧 `forall` 同时红）。删掉这两个 `== 0`，8 行新行与 3 行改 2 的行都不会变绿成红的反面——但按模型，它们与旧 `forall` 等价，删了不丢可证的东西。是否要补一行，是主 agent 定的。
- `.claude/gate.d/verus-pins.tsv` 义务数 211 改 214：「crate	crates/singlefs-replay-prefix	214」（`.claude/gate.d/verus-pins.tsv:50`）；新增的三个 proof fn（两条引理、一个见证）各一项，`range_intersection_length` 是没有 `decreases` 的 `open spec fn`，不成项；211 + 3 = 214 对得上（推的；是否整格对，由 20208 的 `verus-proof` 格「各 crate 各项 verified 之和等于它登记的数」判，日志里该格绿）。


## 关 4：正文八格检查单

### 格一：走不到的理由二次查实：部分成立

- 报告字面「没有新加 `todo!`、`unreachable!`、`assert!`」：对。我在 `changes.patch` 里 `crates/singlefs-core/src/`、`crates/singlefs-checker/src/` 的 `+` 行搜 `assert!`、`assert_eq!`、`unreachable!`、`todo!`、`unimplemented!`、`panic!`、` _ =>`：产品代码里零命中（命中的 `assert`、`expect` 都在 `#[cfg(test)]` 与新测试文件里）。
- 对不上的一半：「写四格前断言位置」是四处新的 `writer.assert_position(…)` 调用（「writer.assert_position(UNIT_BYTES_OFFSET, "单元大小");」`crates/singlefs-core/src/system_configuration.rs:677`，另 `system_configuration.rs:679`、`system_configuration.rs:681`、`system_configuration.rs:684`），函数体里是 `assert!`：「"{what} 的偏移与字段表不符：写到 {cursor}，字段表是 {expected}"」（`crates/singlefs-core/src/bytes.rs:558`）。所以「没有新的会 panic 的点」不成立，「没有新的 `assert!` 字样」成立。触发条件：写到的位置不等于字段表偏移，也就是 `to_slot` 在这些字段之前写的宽度变了：之前都是定宽写（`put_u32` 常量、「writer.skip(80);」`system_configuration.rs:670`、中间的 `put_u32(self.immutable.sizes.physical_block_size)` 4 字节 `system_configuration.rs:683`），没有一个量由输入决定；任何盘上或调用方给的输入都触发不了（读 `to_slot` 推的）。`assert_position` 自己带 `requires`，由 Verus 验过的调用方证；`system_configuration.rs` 不在 `verus!` 里（`grep -c 'verus!'` 为 0），这四处靠运行时的 `assert!`，行为与已有的十几处同类调用一致（`system_configuration.rs:645` 起）。
- 新 `expect`：「let field_offset = |offset: u64| usize::try_from(offset).expect("字段表里的偏移装得进 usize");」（`crates/singlefs-core/src/system_configuration.rs:195`）：依赖的不变量写在了消息里，偏移是 305 到 321 的常量，触发不了。`# Panics` 文档写「槽短于 4096 字节」（`system_configuration.rs:190`），真实触发是短于 325（`ByteReader::at` 加 `get_u32`），文档比需要的严，不算错；checker 一侧写的是「槽短于偏移 325」（`crates/singlefs-checker/src/image.rs:1064`），调用方都先过了判槽宽的关（`check_system_configuration_slot` 的 `TooShort`，`lib.rs:479` 一带）。
- 以 Undecided、Unsupported、OutsideTheFirstVersion 收尾的新成员：没有（四个新成员 `recovery.rs:1257` 起，`MakeFilesystemError`、`DeviceTableDisagreement` 的新成员名见补丁）。池级 checker 与验证包两处的新分支都是「不等就 Err」，没有不可达分支。

### 格二：类型与名字：成立（有一处类型不一致，不是缺陷）

- 四格的值是裸整数：`UnitGeometryFieldsRecordedInTheSlot { u32 × 4 }`（`crates/singlefs-core/src/system_configuration.rs:180`）；仓里没有「单元大小」「落点粒度」「位置条目宽」的 newtype（格式常量 `DATA_UNIT_BYTES`、`SLOT_BYTES`、`LOC_ENTRY` 都是裸 u64，`crates/singlefs-format/src/lib.rs:14`、`:20`、`:77`），所以没有混用已有的 newtype。
- `physical_block_size_in_the_parameters: u32` 与 `probed_physical_block_size: PhysicalBlockSizeInBytes` 不是同一个类型：同一个成员里一个裸 `u32`、一个 newtype（`PhysicalBlockSizeInBytes(pub u32)`，`crates/singlefs-core/src/block_device.rs:132`；`DeviceTableDisagreement` 的新成员同形：`physical_block_size_in_the_system_configuration: u32`）。裸 `u32` 是 `SystemImmutableSizes.physical_block_size` 现有的字段类型（「pub physical_block_size: u32,」`system_configuration.rs:299`），比较时取 `.0`（`make_filesystem.rs:615`、`mount.rs:7628`）。没有违反 `fs-design.md` 的「不同地址空间必须是不同的 Rust 类型」，只是成员内两种写法并列。
- 每处 `expect` 写没写依赖的不变量：新增的产品代码里只有上面那一处，写了；测试里的 `expect("32768")`、`expect("16384")`、`expect("14")` 是常量装得进 `u32`，也是不变量。
- `match` 有没有 `_ =>`：产品代码新增行零命中；改了的穷举 `match`（`entries_after_mount_refuse_swapped_or_behind_devices.rs:246`）是加一臂，不是 `_ =>`。

### 格三：不兼容旧代码与旧格式：成立

- 补丁里没有版本号分支、旧布局号常量、`legacy` 一类开关；只收紧读者。
- `memory_pool.rs:758` 的 `.filter` 退到 `SlotZeroNotSelfDescribing`：候选集合变大（`memory_pool.rs:721` 与 `memory_pool.rs:732` 的两个方法），不放行任何槽，所以没有为别的池放宽（第 1.5 节）。`verification_read.rs:94` 交 `Err(())`：与已有的几何失败同一个处置（`verification_read.rs:97`、`verification_read.rs:105`），别的池的槽照原分类（`verification_read.rs:174` 起，`mount_transparency.rs:2395` 起）。

### 格四：不新建判定库：成立

- 命令（我跑的，只读）：在 `changes.patch` 的 `+` 行搜 `std::fs|std::env|env!|CARGO_MANIFEST_DIR|hostname|std::thread|File::|\.tsv|rocksdb|sqlite|Command::|std::process|tempfile|#\[ignore\]|num_threads|static `，唯一的命中是 `+++ b/crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 这个补丁头，不是代码。新测试文件整份读过（`writable_mount_refuses_a_device_whose_probed_physical_block_size_differs_from_the_system_configuration.rs`）：只用 `SparseBlockDevice`、`SparseDevice`，镜像是内存里的 `clone`；没有文件、TSV、库。证明沿用门禁 formal-verification 的逐项判定（任务 20208 日志）。

### 格五：所有任务多机可用：成立

- 同上一条的搜索：新测试不读仓里的文件、`CARGO_MANIFEST_DIR`、环境变量、主机名。镜像常量 `IMAGE_BYTES = 4 << 30` 是稀疏内存盘（`SparseBlockDevice::new`），不占真盘。证明只依赖 Verus 与登记表（`.claude/gate.d/verus-pins.tsv:50`）。模型也是纯 Python，只读自己目录里的文件。

### 格六：测试优先多线程：成立（切不切片的判：不切；推的，没量）

- 新用例每条一个场景：mkfs 一次或两次、可写挂载一两次、读槽一次；没有写死线程数、没有 `std::thread`；不能并行的理由没写在用例旁，也没有需要写的（用例之间不共享状态，每条自己造盘）。各条的挂钟没量；规模按稀疏内存盘与一次 mkfs 推毫秒级（推的）。该不该切片：按「每条一个场景、互不依赖」判，不该切。

### 格七：带 `--item` 的命令与项：两处推不出（待主 agent 跑）

- 两份实现员报告里点名的测试目标（命令是 `grep -ohE '(--test|--lib) [a-z_0-9]*' result.md result-batch-2.md` 的输出，我跑的）：`--lib verification_read`、`--test core_review_geometry_back_chain_and_empty_inode`、`--test entries_after_mount_refuse_swapped_or_behind_devices`、`--test make_filesystem_refuses_a_physical_block_size_parameter_other_than_the_probed_one_before_any_write`、`--test memory_pool_scan_candidates_do_not_take_the_geometry_of_a_slot_zero_carrying_a_refused_unit_geometry_field`、`--test mkfs_refuses_geometries_whose_root_slots_do_not_align`、`--test system_configuration_unit_geometry_fields_are_refused_by_core_and_the_pool_checker`、`--test writable_mount_refuses_a_device_whose_probed_physical_block_size_differs_from_the_system_configuration`；checker 库单测由变异行 6 到 9 的命令点名（`-p singlefs-checker --lib -- image::tests::…`）。新增与改动的测试目标这些都点到了，没有漏的新目标。
- 推不出一：17 行变异「抓到 17 条、没红 0 条」（任务 19266）与 8 行新必红、3 个 proof 项由 formal-verification 列项（任务 20208 退 0）是主 agent 给的数，我只核了 20208 的日志（第 3.5 节）；`crates-mutation-replay --list-items` 与 `formal-verification --list-items` 列不列得出新项：待主 agent 跑（命令见末尾）。
- 推不出二（缺口）：那条已知红测试所在目标合入后验证没跑到，是因为合入后验证「只跑补丁改到的测试目标」（`.claude/agent-common.md:65`），而补丁的文件清单里没有 `common_images_for_the_given_states_engine/` 下任何一份、也没有 `crash_given_states_of_the_images_harness_scenarios_hand_out.rs`（`changes.patch` 的 `diff --git` 头共 16 份，`grep -c` 数的）。C1040 改的是每个写入口的行为（每次用户改动前多一次探测），被它改变的不是改到的文件，而是用探测次数当观测的测试——这一类按「补丁改到的测试目标」选不出来。任务 19999 只跑了那一条（日志「357 filtered out」），同一个文件里另有五处断言探测次数为零（`raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:4719`、`:4832`、`:4918`、`:5046`、`:5131`），没有跑；这五处走的入口用会话自己的盘表判法（「.is_the_device_table_of_the_mount(devices)」`crates/singlefs-core/src/mounted_session.rs:3966` 一带，不经 `device_table_disagreeing_with`），按代码推它们不探测、不红（推的，没跑）。

### 格八：新崩溃流多罩了什么：成立（写者）、推不出（探测≠参数的盘）

- 「写系统配置槽的只有 `SystemConfiguration::to_slot`」：成立。产品代码里 `to_slot()` 两处调用：`crates/singlefs-core/src/make_filesystem.rs:831` 与「let slot_bytes = system_configuration.to_slot();」（`crates/singlefs-core/src/transaction.rs:739`）；四格都由 `to_slot` 照常量写（`system_configuration.rs:677` 到 `:684`）。验证装置里把槽拷回去写的（`crash_judge_tables.rs`、`e158` 等 bin）也经 `to_slot`，字段不属 `SystemConfiguration` 的四格不会被改。
- 「有没有流或装置在探测值与参数不同的盘上做 mkfs 或可写挂载」：我按字面 grep `PhysicalBlockSizeInBytes(<非 512 字面量>)` 与 `physical_block_size: <非 512 字面量>`（`crates/` 下 `.rs`）：探测非 512 的盘只有 `root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs:955` 一处（参数也是 4096）；参数非 512 的是 `core_review_geometry_back_chain_and_empty_inode.rs`（256、8192、4096；这一轮改成盘探测取参数）、`e197_scan_journal_glue_memory_by_device_count.rs:485`（实现员第二份报告说它只造系统配置、不 mkfs，我没核）与别的报错类用例。运行时才算出参数的调用点没追（实现员报告也这样写）。所以字面量这一层成立，运行时算出的那一层**推不出**。
- 本轮新增的探测只调 `probe_physical_block_size`，不写盘（「只调探测，不读盘、不写盘。」`crates/singlefs-core/src/mount.rs:7613`），崩溃枚举数的是写，所以不改崩溃状态集合（推的，没跑）。


## 关 5：已知红测试 `…:3798`（期望 (0, 0)，读到 (2, 0)）：从代码推，该判「测试判法要换」

- 红在哪（任务 19999 日志，转述）：只跑了那一条（「357 filtered out」），断言消息是「那一次不预演：一块只进内存的盘都没建、一个改盘调用都没有」，`left: (2, 0)`、`right: (0, 0)`。断言在 `crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3798` 起，期望值在 `:3803`，消息在 `:3804`。
- 测试把「探测次数为零」当成「没建只进内存的盘」的代理，前提写在两处：「发布路径别处不探：数探的次数」（`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3770`）与「另数整池的读与探物理块大小（只进内存的盘一建就探一次底下那块盘，发布路径别处不探）。」（`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:4123`）。「发布路径别处不探」是对 C1040 之前代码的描述，不是条款：命令（我跑的，只读）`grep -rnE '别处不探|发布路径别处|写路径.{0,6}不探|不探测' .claude/kb/decisions/*.md`：唯一命中是 `.claude/kb/decisions/21-权威态与派生态的分界.md:330` 那句已过期的实现边界（「物理块大小不探测比对」），不是对写路径的要求。
- 为什么现在探了：被拒的那一次调用也走 `publish_user_change`（`crates/singlefs-core/src/mounted_session.rs:2951`），它的文档写明「三样在进循环之前核一次：一次调用里盘表借着、换不了盘」（`crates/singlefs-core/src/mounted_session.rs:2928`），核用的是 `caller_inputs_agreeing_with_the_disk`（`crates/singlefs-core/src/mount.rs:7749`），其中的盘表那一比逐盘探测一次（`mount.rs:7626` 起的 `filter_map`、`mount.rs:7627`），盘数 2，所以读到 2；不写盘所以改盘调用数仍是 0（「只调探测，不读盘、不写盘。」`crates/singlefs-core/src/mount.rs:7613`）。这个核「照可写挂载同一套核」（`mount.rs:7730`），连「一个字节都不写的那一格也先核」（`crates/singlefs-core/src/mount.rs:4779`）——作者明确让每个入口先核、不论后面写不写。预演用的只进内存的盘是另一处探测：「physical_block_size: disk.probe_physical_block_size(),」（`crates/singlefs-core/src/mounted_session.rs:924`），由 `DeviceKeepingWritesInMemory::over_the_disk` 建（`mounted_session.rs:1862`）。
- 判：这一格是「测试判法要换」，不是「那条路不该探测」。依据三条：
  1. 入口核盘表是已有的设计（D18 已定项 11 的「挂着之后收盘表的每个入口……核三样」，见 `mounted_session.rs:2928` 一带与 `mount.rs:7730`），C1040 把探测值那一比放进同一个函数，这条路因此多探一次；让写路径别探，要么在 `caller_inputs_agreeing_with_the_disk` 之前先报「被宽护卡住」（让「一个字节都不写」的入口不核，与 `mount.rs:4779` 的设计相反），要么缓存挂载时探到的值（那是新设计，条款没写）。
  2. 代理失灵：同一个计数器现在数两种东西（入口核的探测、预演建盘的探测），断言 (0, 0) 想证的是「没建预演盘」，要么换成只数后一种，要么把期望写成差值。
  3. 另一个方向（「那条路不该探测」）的依据我只找到一句可读成支持它的：「单元几何在挂载时求值一次，运行时只查不算」（`.claude/kb/decisions/21-权威态与派生态的分界.md:332`）。它管的是单元几何那一组「挂得上 / 挂不上」的判定（该项射程：「本项管单元几何」），入口核盘表的对象是「这一刻交进来的盘」，每次调用都由调用方重新给；这一句是否也禁止每次入口再探一次，**条款没说，推不出**，交主 agent。
- 怎么改测试（推的，没跑）：`:3798` 期望写成 `(盘数, 0)`（入口核的探测，盘数是 `pool.devices.len()`），`:3816` 起那一处现在期望 `(盘数, 0)` 的累计值会变成 `2 × 盘数`（入口核两次盘数 + 预演一次盘数，前提是 `ways_out_of` 自己不核，它走 `is_the_device_table_of_the_mount`，`mounted_session.rs:3966`），更稳的写法是每次调用前后各取一次计数、断言差值。其余五处数探测的断言（`…:4719`、`:4832`、`:4918`、`:5046`、`:5131`，场景是「不读盘就拒」）走会话自己的盘表判法，按代码推不红。
- 推翻这个判的现象：主 agent 跑那一条得 (2, 0)，而去掉 `caller_inputs_agreeing_with_the_disk` 的盘表探测之后读到 (0, 0)（用变异或副本）——说明 2 确实来自入口核；若去掉之后仍读到 2，说明探测来自别处（比如预演盘提前建），本节的因果就错了。命令在末尾。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 关 1.1 偏移与值 | 成立（一致） | 305、309、313、321 与 32768、16384、14、0，字段表、core、checker 都对得上 |
| 关 1.2 每格判法 | 成立（一致） | 四处都是同值域的 `!=` |
| 关 1.3 次序 | 成立，一处条文没写的差异 | 加密类型之后、槽距之前两边同；S 越界与四格同在时 core 先报 S、checker 先报四格，整池拒的结局相同 |
| 关 1.4 每条读槽路径 | 成立（只覆盖 `geometry_of` 的调用点与 core 的单一入口）；GPU 判器 | 三个非测试调用点都有四格判；GPU 没同判是条文已写的欠账 |
| 关 1.5 验证包两处 | 成立 | 不放宽任何槽；扫描候选只变大 |
| 关 2.1 可写挂载三入口 | 成立（读代码）；每个入口的测试推不出 | 都走到 `mount.rs:7616`；只有入口一有用例 |
| 关 2.1 离线修复 | 推不出 | 条款没写离线修复比不比探测值 |
| 关 2.2 mkfs 在任何写之前 | 成立 | `make_filesystem.rs:649` 在第一次写 `:729` 之前 |
| 关 2.2 D22「按 max」与实现 | 推不出 | 条款自己的「按 max 划」与「不同就拒」在异构盘上给出不同结局 |
| 关 2.3 只读挂载与 D12 第 69、75、103 行 | 不成立 | `PoolReader` 没探测，代码只比可写一侧；条款状态句写错或实现欠一半 |
| 关 3.1 规格 | 成立 | 模型：逐字节交集个数，含空与倒置 |
| 关 3.2 后置条件不空真 | 成立 | 两支都有输入、全满足；`+1` 在每个 Some 输入上不成立；None 支新条件与旧 `forall` 等价 |
| 关 3.3 引理前提 | 成立（够弱） | 前提去掉有反例；比登记前提更弱的只有倒置一种 |
| 关 3.4 见证 | 成立 | 十一条等式对、六处调用前提满足；调用对等式无贡献 |
| 关 3.5 八行新必红、三行改 2 | 成立（日志转述）；一处缺口 | 落点与登记相同；两处新 None 支 `== 0` 没有专属必红行 |
| 关 4 格一 走不到的理由 | 部分成立 | 没有新 `assert!` 字样，有四处新的会 panic 的调用点，输入打不到 |
| 关 4 格二 类型与名字 | 成立 | 一个成员里 newtype 与裸 `u32` 并列（沿用现有字段类型） |
| 关 4 格三 不兼容旧 | 成立 | 无版本分支；两处验证包新分支不放宽 |
| 关 4 格四 不新建判定库 | 成立 | 新增行搜不到文件、库、进程 |
| 关 4 格五 多机可用 | 成立 | 同上；证明只依赖 Verus 与登记表 |
| 关 4 格六 测试优先多线程 | 成立（推的） | 每条一个场景，不切片，没写死线程数 |
| 关 4 格七 带 `--item` | 推不出（两处，待主 agent 跑） | 列项没跑；合入后验证按「改到的目标」选，选不出行为被改的目标 |
| 关 4 格八 新崩溃流 | 成立（写者）、推不出（运行时算出的探测与参数） | `to_slot` 两处调用点；字面量层面无探测≠参数的流 |
| 关 5 已知红测试 | 测试判法要换（代码层）；条款层面「每次入口要不要重探」推不出 | 探测来自入口核，是「同一套核」的设计结果 |

## 要主 agent 跑的命令

每项：命令、在哪个目录起、跑完看哪几行。

1. 命令：`task-server/task submit -- bash .claude/gate.d/formal-verification.sh --list-items`；目录：`/home/fy5090/code/singlefs`；看 `task-server/task log <号>` 里有没有 `must-red:intersection-length-contract-lemma_adjacent_unit_intersection_lengths_add_up`、`must-red:intersection-length-contract-lemma_split_write_intersection_lengths_add_up`、`must-red:intersection-length-contract-witness_intersection_length_partitions`、`must-red:intersection-length-unit-split-needs-valid-boundary`、`must-red:intersection-length-write-split-needs-valid-boundary`、`must-red:write-intersection-exact-length`、`must-red:indexed-intersection-exact-length`、`must-red:intersection-length-witness-sum` 八项与 `proof:singlefs_replay_prefix::file_range_proofs::lemma_adjacent_unit_intersection_lengths_add_up`、`…::lemma_split_write_intersection_lengths_add_up`、`…::witness_intersection_length_partitions` 三项；末行 `TASK_SERVER_RESULT` 的 `exit=0`。
2. 命令：`task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items`；目录：`/home/fy5090/code/singlefs`；看输出里名字以「C1040：」「C1041：」起头的行共 17 行（与 `research/prompts/m4-close-c1040-c1041-code-r1-snapshot/mutations-c1040-c1041-rows.tsv` 的 17 行对名），以及「键对不上」「锚点失效」两行报几条；`exit=0`。
3. 命令（模型复跑，整份）：`task-server/task submit -- bash research/prompts/m4-close-c1040-c1041-code-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/m4-close-c1040-c1041-code-r1-snapshot /home/fy5090/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-forward/model-run /home/fy5090/code/singlefs/research/prompts/m4-close-c1040-c1041-code-r1-sonnet-model`；目录：`/home/fy5090/code/singlefs`；看输出末尾 `name=model_result failures=0` 与 `name=rerun_matches_saved_output ok=yes`，`TASK_SERVER_RESULT` 的 `exit=0`；`task wait` 末行退非 0 的件数为 0 算过。里面只起 `python3 -I` 一个纯 Python 进程；包装脚本只删 `~/.local/share/singlefs/work/` 下名字以 `-run` 收尾的那个运行目录。
4. 命令（模板自证）：`task-server/task submit -- bash research/prompts/m4-close-c1040-c1041-code-r1-sonnet-model/tools/rerun.sh --selftest`；目录：`/home/fy5090/code/singlefs`；看「自证通过」那一行与 `exit=0`。
5. 命令（复现关 5 的 (2, 0)，并核它来自入口核）：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_given_states_of_the_images_harness_scenarios_hand_out 里的 1 条测试; 范围=1 个测试; 为什么=复现 :3798 读到 (2, 0)，并配合副本里去掉入口核探测的变异核它来自 caller_inputs_agreeing_with_the_disk；这个目标名带 crash，范围声明闸若要缩域或全域理由，照闸的拒绝说明补' task-server/task submit -- cargo test -p singlefs-verification --test crash_given_states_of_the_images_harness_scenarios_hand_out -- write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push --exact`；目录：`/home/fy5090/code/singlefs`；看 `left:` 与 `right:` 两行。要核「去掉入口核探测读到 (0, 0)」，在仓副本里（`scripts/hosts/minimal-copy.py` 建）把 `crates/singlefs-core/src/mount.rs:7603` 的 `disagreements.extend(…)` 注释掉再跑同一条；这一步我没建副本、没做。
6. 命令（可选，核「被宽护卡住」之外五处数探测的断言）：同上写法、`--test` 同一个目标，点名五条场景的测试函数名（名字是上面几处 `run_the_scenario_where_…` 去掉前缀，在 `crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 里 `grep -n` 取）；看它们是否都 `ok`。我没列全名，按那个文件现取。

## 交主 agent 的写回项（不属这一轮代码判决）

- `.claude/kb/decisions/12-目标介质.md` 第 69、103 行写「可写与只读都拒」是状态句，只读一侧代码没做；第 75 行是规则。出路：补只读一侧，或改状态句并把第 75 行标欠。
- `.claude/kb/decisions/21-权威态与派生态的分界.md:330` 的实现边界与 `.claude/kb/checks-owed.md:642` 的现状，对可写挂载已探测、四格已判这两件已过期或没写只读一侧。
- `crates/singlefs-core/src/mount.rs:7731` 的文档没写探测值那一比（注释滞后）。
- `.claude/kb/decisions/22-单元原子性怎么合成.md:63` 的「按池内 `max` 划」与「不同就拒绝 mkfs」「未达到拒绝挂载」，和实现的「与每块盘都相等」在异构盘上不一致，第一版两盘同构下不显现。

## 什么现象会推翻这些结论

- 关 1：一条条款或用例要求 S 越界与四格同在一槽时两边点名同一个成员；或一个读系统配置槽的路径不经 `mountable_slot_or_refusal` 也不经 `read_system_configuration_slot`（我只核了 `geometry_of` 的非测试调用点与 core 的单一入口，没核 GPU 内核）。
- 关 2：入口二或三有一条路径不经 `device_table_disagreeing_with`（我读的是调用图；没有逐入口的测试）；`make_filesystem.rs:649` 与 `:729` 之间有我没看到的对盘的写（我用 `grep -n 'devices\b'` 数过）。
- 关 3：Verus 在主 agent 复跑时对某行落点集合与登记不等；模型的小格（`[-4,9)`、`MAX = 15`）漏了大数上的反例（`witness` 里 u64 上界那一条用的是精确大整数，格内没有）。
- 关 4 与关 5：见上各节的「推翻」句。

## 没做什么

- 不替主 agent 采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。
- 我直接跑过一次 `python3 -I intersection_length_model.py`（第一次，输出就是 `out/model.out`），之后项目代码闸拒了再直接起；这一次不是经服务跑的，主 agent 复跑时以服务那次为准。`out/model.out` 是那一次输出的原样。
- 没跑 Verus、没跑 cargo、没建仓副本，没跑 `--list-items`、`--selftest`；所有「推的」都没跑。
- 没核 GPU 判器两个内核；没核 `crates/mutations.tsv` 以外的变异表；没逐行核 17 行变异各自的 `--item` 命令（只读了表）。
- 草稿目录 `~/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-forward/` 里没有副本、编译目录与库拷贝（只有认领时建的 `.owner`、`.writer-lock`）；模型在 `research/prompts/m4-close-c1040-c1041-code-r1-sonnet-model/`，没有可删的东西。
