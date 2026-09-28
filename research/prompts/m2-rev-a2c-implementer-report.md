# 实审 A2c 实现员报告：审阅第 15 条后一半、第 26、34、36 条，A1b 的 Q5 与管理员回退入口

日期：交回于 2026-09-27。规格 `/tmp/claude-1000/impl-rev-a2c/spec.md`。派发要求不改 `crates/mutations.tsv`：变异行在草稿目录，见「变异行放在哪」。

## 结论

- **分两段交**：
  - **阶段 A 已在主工作区**（只动单内三份 + 新测试文件）：第 34 条做完；第 15 条做了分配器那一半（单元区起点成了空闲图的一个量、由环长现算的函数只有一处），**挂载一侧没切到按环长现算**（原因见「停下交主 agent 的」第 1 条）。
  - **阶段 B 是补丁，没进主工作区**：第 26、36 条、Q5、回退入口都要给 `PublishError` / `MountError` / `RollbackError` 加成员，这三个枚举在 `crates/singlefs-harness/src/history.rs`（3 处）与 `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`（2 处）里被穷举 match，不补就编不过；Q5 还让单外的 `tests/a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs` 那条用例的期望变形（抬 F 的错改为装在新成员里）。这三份不在派发列的文件单里，加上单内的共 10 份，超过派发闸一件活 8 份的上限，我没把它们写进主工作区，做成补丁 `/tmp/claude-1000/impl-rev-a2c/patch/`，由主 agent 定打不打（改法已在「主工作区现状 + 补丁」的副本上建过、跑过、证红过）。
- 新测试文件 `crates/singlefs-harness/tests/core_review_unit_area_start_and_publish_limits.rs`：主工作区里是阶段 A 的 7 条，补丁打上之后 20 条；另在 `model_comparison.rs` 的单测里加 1 条。
- 变异：阶段 A 追加 12 行、换锚点 2 行（主 agent 点名的 430、436），14 行在「主工作区现状」的副本上逐条证红 14/14；阶段 B 追加 20 行、换锚点 4 行（64、451、887、942，补丁动断了它们），24 行在「主工作区现状 + 补丁」的副本上逐条证红 24/24。
- 推翻条件：主工作区跑 `cargo test -p singlefs-harness --test core_review_unit_area_start_and_publish_limits` 有一条红；或主 agent 按「变异行放在哪」追加 / 替换之后门禁 59 号在这 38 行里有一行没红；或补丁打上之后 `cargo build --all-targets` 编不过。

## 这一轮写过的文件

主工作区（阶段 A）：
- `crates/singlefs-core/src/allocator.rs`（第 15 条：`UnitAreaStart`、`with_unit_area_start`、`unit_area_slots_of_device_starting_at`；`index` 改成方法、三处扫描与 `is_free` 从这张图的起点起；`release` 三条断言的消息改指新的逐盘函数；单测模块补 `use singlefs_format::UNIT_AREA_START_SLOT`）
- `crates/singlefs-core/src/journal.rs`（第 15 条：`slot_after_the_journal_ring`，由环长算单元区起点只有这一处）
- `crates/singlefs-core/src/transaction.rs`（第 34 条：`format_time_tree_table_to_release` 逐盘核、改返回 `Result`；从 `placement_to_release_after_checking_every_device` 拆出逐盘那一半 `placement_registered_unreleased_on_every_device`）
- `crates/singlefs-harness/tests/core_review_unit_area_start_and_publish_limits.rs`（新建，阶段 A 的 7 条）
- 相对开工快照（`/tmp/claude-1000/impl-rev-a2c/orig/`）的改动行数（`diff | grep -c '^[<>]'`）：allocator 172、journal 13、transaction 75；新测试文件 482 行。
- `crates/singlefs-core/src/mount.rs`、`mounted_session.rs`、`crates/singlefs-harness/src/model_comparison.rs`：主工作区里**没动**（改动全在补丁里）。
- `crates/mutations.tsv`：没动。

补丁 `/tmp/claude-1000/impl-rev-a2c/patch/crates.patch`（阶段 B，基准是交回这一刻的主工作区，`git apply --check` 过；1897 行，9 个文件）：
- 单内：`journal.rs`、`mount.rs`、`mounted_session.rs`、`transaction.rs`、`model_comparison.rs`、新测试文件（+13 条）
- **单外**：`crates/singlefs-harness/src/history.rs`（`publish_error_member`、`mount_error_member`、`rollback_error_member` 各补新成员的臂，19 行）、`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`（两处穷举 match 补臂，12 行）、`crates/singlefs-harness/tests/a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs`（挂载那条用例改为先解 `FloorRaiseFailedAfterTheMountsPublishes` 再核里面的错，19 行）。开工时逐份核过：没有一个在跑的实现员的文件单里有这三份（A2b、A4b、B2b、B3a-3 的 spec.md 各自那一行），开工快照与交回时主工作区里这三份逐字节相同。

## 每条怎么改

### 第 34 条（阶段 A，主工作区）

`transaction.rs` `format_time_tree_table_to_release`（此刻第 2474 行）：「这一片还登记着、没释放过」改为逐盘判——池里没有一块盘的账里它在册且未释放 ⇒ 照旧不释放（两块盘的账一致时与改之前只看第一块盘相同）；有一块盘在册且未释放 ⇒ 走 `placement_registered_unreleased_on_every_device`（第 2979 行，映射那一路 `placement_to_release_after_checking_every_device` 逐盘的那一半，拆出来两处共用），别的盘缺记录 / 已释放 / 跨度不对就报 `ReleaseTargetNotAllocated` / `ReleaseTargetAlreadyReleased` / `ReleaseSpanMismatch`，在准入、读盘核与动分配器之前。mkfs 那片树表与树表 0 条那一版分配记录树的节点两路都走它。没加新错误成员。

### 第 15 条后一半（阶段 A 做了分配器那一半）

- `journal.rs` `slot_after_the_journal_ring`（第 186 行）：`JOURNAL_RING_START_SLOT + 环长.div_ceil(16384)`——D3（空间分配） 已定项 10 ④「第一版 = journal 环末尾的下一个槽」，末尾不在槽边界上时取下一个整槽。
- `allocator.rs` `UnitAreaStart`（第 198 行）：只从 `following_the_journal_ring(环长)` 进来，**起点不落在 64 槽聚簇段边界上就拒**（`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`，第一版不支持，理由见「停下交主 agent 的」第 2 条）。`of_the_default_journal_ring()` = 默认环 768 MiB 的起点 50176（用例钉住等于 `UNIT_AREA_START_SLOT`）。
- `DeviceFreeMap` 多一个字段 `unit_area_start`；`with_unit_area_start(设备, 字节数, 起点)` 新构造；`new(设备, 字节数)` 签名不变、取默认环的起点（全仓 50 多处调用、多在别的会话的文件里）；`unit_area_slots_of_device(字节数)` 同样保留、取默认环的起点（`recovery.rs` 在调）。`index`、`is_free`、`lowest_user_data_slot`、`lowest_empty_segment`、`lowest_commit_generated_fallback_slot` 都从这张图的起点起。
- 默认环下与今天逐字节相同：用例在 mkfs 同一个进程里按两种建法（`allocator_after_make_filesystem` 与「按环长算起点 + `with_unit_area_start`」）各写一遍第一个文件，两块盘整盘镜像相等。
- **没做**：`mount.rs` 建空闲图（`rebuilt_allocator`，`format_time_allocator` 收的就是它建的图）仍调 `DeviceFreeMap::new`，没切到「按盘上系统配置里的环长现算」——见「停下交主 agent 的」第 1 条。

### 第 26 条（补丁）

- `transaction.rs` `journal_record_limit_of_one_publish(环长)` = min(在飞上限 = 环槽数 ÷ 3（D23（journal 的角色与格式） 已定项 18，`singlefs_format::journal_in_flight_record_limit`）, 2³² − 1（本次发布内序号 32 位，已定项 4））；`refuse_a_publish_of_more_journal_records_than_the_limit` 多于上限就报新成员 `PublishError::JournalRecordsOfThePublishExceedTheLimit { records_of_the_publish, record_limit }`。
- 判在两处：带文件的一版（`prepare_the_version_publish`，算定 `settled.resolved.rewritten_roles` 之后、空间准入、读盘核与动分配器之前）；树表 0 条的一版上写行（`prepare_the_row_publish_on_a_version_without_file`，算定重写集合之后、释放与取落点之前）。条数用落盘时切记录的同一个函数 `roles_named_by_each_record_of_the_publish` 数。
- 可写挂载的写行与暖机在取号之前的预演里走这两处，超了在取号之前报成 `RowPublishAdmissionRefusedBeforeAcquisition { cause: JournalRecordsOfThePublishExceedTheLimit }`（推的，没造用例）。
- `mounted_session.rs` `refusal_is_short_of_space` 把它归「不是空间不够」：推抬 F 腾不出记录的位置。
- `journal.rs:84` 那句过时的 expect（「按 extent 叶容量截过」）与 `# Panics` 改成现状：条数在落盘之前按 `journal_record_limit_of_one_publish`（≤ 2³² − 1）拒过。

### 第 36 条 mount / transaction 那几处（补丁）

- 发布路径：`checkpoint_txg_of_the_publish_after`（`checked_add`），`publish_first_file`（挪到函数开头、读树表之前）、`publish_overwrite`、`publish_sequential_write`、`publish_new_inodes` 四处；溢出报新成员 `PublishError::NextCheckpointTxgPastTheTopOfItsRange { version_to_build_on }`，在任何读写之前。
- 实例代号：新 `next_instance_generation_to_acquire` 交 `Result<_, InstanceGenerationPastTheTopOfItsRange>`；可写挂载取号之前调它、`acquire_expected_instance` 写之前重算也调它（`ExpectedInstanceAcquisitionFailed` 加成员 `InstanceGenerationPastTheTopOfItsRange`，只在 `mount.rs` 里被 match）。**原 `instance_generation_to_acquire` 保留签名、内部 `expect`**：它 `pub`、被 `acquire_instance` 与单外测试 `second_transaction_supplement_two_row_publish_checks_before_acquisition.rs` 直接调；`acquire_instance` 的错是 `AcquisitionFailed { cause: BlockDeviceError, .. }`，装不下这一种（mkfs 同一个进程那条流里实例代号从 0 起，走不到 u32::MAX）。
- 挂载一侧：`checkpoint_txg_after_the_version`（`checked_add`）用在新实例第一次发布的 txg（`first_txg_of_new_instance`）、暖机计划（`warm_up_publish_txgs`）、抬 F 那一串的 txg（`txgs_of_the_publishes_carrying_the_floor_to_every_device`，这一串在抬 F 里挪到影子账重算与回收之前算）；溢出报新成员 `MountError::SequenceNumberPastTheTopOfItsRange(SequenceNumberAtTheTopOfItsRange::{InstanceGeneration, CheckpointTxg})`，都在任何写之前（抬 F 在动分配器之前）。
- 计划里的每一次空发布（暖机、抬 F 各次、预演）再取 txg 时走 `checkpoint_txg_of_the_planned_publish_after`（`expect`）：**为什么走不到**——那几次的 txg 与在任何写之前用 `checked_add` 算好的计划逐次相同（`establish_instance` 的 `assert_eq!(…, *planned_txg)`、抬 F 按 `planned_txgs` 逐次推），计划那一步越界就先拒了；调用点只有 `empty_publish_plan_raising_the_floor`、`empty_publish_plan_after_file_version`、`empty_publish_plan_after_version_without_file` 与暖机循环里那条断言。
- 管理员回退：`RollbackError::NextCheckpointTxgPastTheTopOfItsRange { current }`，在核参数之后、读实例表之前。
- 没动：jsn 计数器的 `+ 1`（48 位，不在第 36 条点名的「txg、实例代号」里）；`recovery.rs` 那一处归 A2b。

### A1b 的 Q5（补丁）

- 新成员 `MountError::FloorRaiseFailedAfterTheMountsPublishes(Box<FloorRaiseFailedAfterTheMountsPublishes>)`，结构三个字段：`cause: MountError`（抬 F 的错原样，它那一串自己的账在它里面：`RaiseFloorSequencePublishFailed` 的 `PublishSequenceFailed`、`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot` 的 `writes`）；`writes_of_persisted_publishes: Vec<WritesByStructureKind>`（写行一次、暖机每次，与 `PublishSequenceFailed` 同一口径）；`floor_raises: Vec<RaisedFloor>`（报错之前推成的几串，每串的先写系统配置与各次空发布的写在里面）。
- `establish_instance` 把写行与暖机的写账交给 `push_floor_raises_after_the_row_publish`，后者抬 F 报 `Err` 时装进新成员（原来是 `?` 原样上抛）。空间不够那几种照旧走 `StillShortAfterTheFloorRaises`，不进这里。
- `model_comparison.rs`：`refusal_reason_of_mount_error`、`reported_ceiling_of_mount_error`、`root_ring_slot_still_bad_after_one_reread_of_mount_error` 对新成员都照 `cause` 递归取（与改之前挂载原样交回它时同一个判法），另认 `SequenceNumberPastTheTopOfItsRange`、两个 `PublishError` 新成员、两个 `RollbackError` 新成员（都归 Unexplained）；加 1 条单测。
- `mount.rs` `floor_raise_refused_for_space` 补两臂（都不是空间不够）。

### 管理员回退入口（补丁）

- `roll_back_by_a_forward_publish` 在冻结判之后调 `caller_inputs_agreeing_with_the_disk`（与抬 F、卸载同一套：重复身份不读盘就拒 → 择系统配置 → 参数逐项比、盘表比 devs 与本盘设备号），写入口改用盘上那份参数（`PoolWriter::new(&parameters_of_the_pool, …)`）。
- 为了让回退报自己的成员而不借 `MountError`：新枚举 `CallerInputsDisagreeingWithTheDisk`（两成员：`DeviceIdentitiesHandedInMoreThanOnce { repeated }`、`ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields, disagreeing_device_table }`，`impl From<…> for MountError` 一一对应原来两个 `MountError` 成员）；`refuse_device_identities_handed_in_more_than_once` 与参数比对函数改交它；私有的 `CallerInputsCheckRefused`（交进来的不对 / 系统配置择不出来）+ `From` 让挂载一侧的 `?` 照旧报原来那几个 `MountError` 成员（挂载、抬 F、卸载的外部行为不变，A1b 那几条用例与 942 行变异照绿照红）。
- 回退：`RollbackError::CallerInputsDisagreeWithTheDisk(CallerInputsDisagreeingWithTheDisk)`；系统配置择不出来照旧 `RollbackError::Recovery`。

## 停下交主 agent 的

1. **第 15 条：挂载一侧没切到「按盘上环长现算」**。规格写「mkfs 现算的那个值进 allocator.rs … 与 mount.rs 的 format_time_allocator；盘上哪里记着环长就从哪里现算」。现查：mkfs 今天没有现算的那个值——`crates/singlefs-core/src/make_filesystem.rs:49-50` `INSTANCE_TABLE_SLOT` / `TREE_TABLE_GENESIS_SLOT` 取编译期常量，环长 > 768 MiB 按 A2a 的「第一版不支持」拒、≤ 768 MiB 的环单元区照旧从 50176 起。同一个常量还在：`system_configuration.rs:390`（写进系统配置偏移 417 的单元区起点 8 字节，checker 按它读，`singlefs-checker/src/image.rs:232`）、`recovery.rs:1109`（`allocation_records_fit_the_pool_geometry`：槽号 < 50176 的分配记录判坏镜像）、harness `crash.rs:650`、`model.rs:919/955`、`history.rs:112-118`（小盘按「50176 + N 槽」定盘宽）。若只把 `mount.rs:967`（`rebuilt_allocator` 建空闲图，`format_time_allocator` 收的就是这张图）切成按盘上环长现算：`history.rs` 小盘用 128 MiB 环（`history.rs:103`），挂载之后分配器从槽 9216 起发落点，下一次挂载 `recovery.rs:1109` 把这些记录判坏、池挂不上（推的，没跑），checker 按 417 那 8 字节判单元区也对不上。这几份不在我的单里（`make_filesystem.rs`、`recovery.rs` 归 A2b，另几份没人领），所以停在「分配器能按任意合法起点工作、默认环逐字节不变」这一步：切换是 `mount.rs:967` 与 `make_filesystem.rs:586` 各一行（`DeviceFreeMap::with_unit_area_start(…, UnitAreaStart::following_the_journal_ring(环长)?)`），要与上面那几处在同一次改动里落。起点不在段边界上那一格要在 mkfs 拒（见第 2 条）。
2. **单元区起点不在 64 槽段边界上怎么办，条款没写**（第 15 条）：D3（空间分配） 已定项 10 ①「64 槽对齐」、③「起点 32768 对齐」说的是设备内槽号，分配器按单元区相对位置 2 槽 / 64 槽步进；起点是奇数槽时数据单元失去 32768 对齐，不在 64 的倍数上时聚簇段按槽号对齐还是按单元区对齐、开头那几槽算不算一段（「全空聚簇段数」那一行记账跟着变）都没有条款。我取「第一版不支持」：`UnitAreaStart::following_the_journal_ring` 在任何建图之前拒成 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`（今天没有调用方会走到：挂载与 mkfs 都还取默认环的起点）。环长是 1 MiB 整数倍的都落在边界上（E158 的 3 MiB 环：1024 + 192 = 1216 = 19 × 64）。
3. **第 34 条的一个取法**：池里没有一块盘的账里 mkfs 那片在册且未释放时，照旧不释放（改之前只看第一块盘也是这样，两块盘一致时行为不变）；映射那一路没有「跳过」这一臂，这一路有，是因为 `format_time_tree_table()` 记着的落点在合法历史里可能已不在册（推的，没找到走得到的历史）。要改成「必须每块盘在册，否则报错」只动 `placement_still_to_release` 那一个判断。
4. **第 26 条没罩到的一格**：零单元发布（`publish_without_units`，暖机、mkfs 那一版上可写挂载那一串）恒一条记录，错类型是 `BlockDeviceError`，没加判。在飞上限为 0 的环（短于 3 条记录）上它照写；A2b 在 mkfs 拒这种环。
5. **第 36 条没罩到的一格**：`acquire_instance`（mkfs 同一个进程那条流、几十份用例在调）遇到盘上实例代号 u32::MAX 仍 panic（带消息）；它的错类型 `AcquisitionFailed { cause: BlockDeviceError, .. }` 装不下这一种，改它要动用例。可写挂载走的是不 panic 的那一条。
6. **补丁里单外三份要主 agent 定**：见「结论」第一条。不打补丁时第 26、36 条、Q5、回退入口都没进主工作区。
7. **`first_transaction_on_device.rs` 对 Q5 新成员只报原样的错、不描写窗口账**：那个二进制每次在新建的池上挂，取号之前的准入不会不够、走不到推抬 F（推的，没在真设备上造过）。要它像 `MountError::Publish` 那样把写行、暖机与抬 F 那一串的账都描写出来，得定抬 F 那一串的先写系统配置与失败账怎么并进窗口（收口表第 58 行那一族），我没做。

## 受影响的层 0 流与崩溃枚举用例

没改 checker（`crates/singlefs-checker/src/` 一处没动；草稿副本里为让 clippy 走到 harness 临时加过一行 `#![allow]`，跑完从主工作区拷回、`cmp` 相同）。阶段 A、B 都没加写、没加屏障：默认环下分配器落点逐字节不变（用例钉住），第 34 条只在两块盘账不对称时改判，第 26、36 条与回退入口只加在任何写之前的拒绝，Q5 只改错的形态。录制流与段序列不变，层 0 各流钉值不受影响（推的，没跑层 0）。

## 证红（改坏哪一行 → 哪条测试红）

先看新用例在改之前红：第 34 条 4 条用例放进开工快照的副本（代码没改）跑，3 条红、1 条对照绿（`/tmp/claude-1000/impl-rev-a2c/pre-fix-34.log`）：盘 1 已释放那两条红在 `allocator.rs:1045` 的 `assert!(!record.is_released, "同一个落点释放了两次…")`（panic），盘 0 已释放那条红在用例自己的 `expect_err`（第一个文件版本照发了）；两块盘都已释放的对照改前改后都绿。第 15 条的用例与阶段 B 的用例要新接口 / 新成员，改前编不过，靠下面的变异证它们判得出。

变异一律 `research/scripts/prove-red.sh --copy <副本> --memory 8G <crate> <名…>`：阶段 A 那 14 行在「交回这一刻的主工作区」的副本 `prove-a` 上（`prove-a-final.txt` 末行 `✓ 点名 14 条：跑了 14 条，跳过 0 条，跑的都抓到了`）；阶段 B 那 24 行在「主工作区 + 补丁」的副本 `prove-b` 上（harness 23 行 + core 1 行；其中「回退那次发布的 txg 直接加一」一行 rustfmt 之后锚点断了，换锚点重证一次，`prove-b-final-2.txt` 抓到）。每条新测试至少一行，没有只追加不证的行。同一次整个测试二进制里另外红了的，列在表后。

| 段 | 变异名 | 改坏（原文 → 替换文） | 必须红的测试 |
|---|---|---|---|
| A 追加 | 实审 A2c 第 34 条：第一个文件版本换下 mkfs 那几片时不逐盘核（在册就交给释放） | `placement_registered_unreleased_on_every_device(identity, slot, allocator).map(Some)` → `Ok::<Option<Placement>, PublishError>(Some(Placement { ⏎ slo…` | `the_first_file_version_refuses_a_genesis_tree_table_that_device_one_already_released_before_any_write` |
| A 追加 | 实审 A2c 第 34 条：换不换下 mkfs 那几片退回只看第一块盘 | `let registered_unreleased_on_some_device = allocator.devices.iter().any(|device_map| {` → `let registered_unreleased_on_some_device = allocator.devices…` | `the_first_file_version_refuses_a_genesis_tree_table_that_device_zero_already_released_before_any_write` |
| A 追加 | 实审 A2c 第 34 条：两块盘上都已释放的 mkfs 那片树表照样逐盘核（不再跳过） | `if registered_unreleased_on_some_device {` → `if true || registered_unreleased_on_some_device {` | `the_first_file_version_publishes_without_releasing_a_genesis_tree_table_released_on_every_device` |
| A 追加 | 实审 A2c 第 34 条：第一个文件版本不换下树表 0 条那一版的分配记录树节点 | `if rewritten.contains(&TransactionUnit::AllocationTree) { ⏎ if let Some(tree) = allocator.…` → `if false && rewritten.contains(&TransactionUnit::AllocationT…` | `the_first_file_version_refuses_an_allocation_record_tree_node_of_the_version_without_file_that_device_one_already_released` |
| A 追加 | 实审 A2c 第 15 条：环末尾的下一个槽向下取整（落进环里） | `SlotNumber(JOURNAL_RING_START_SLOT + journal_ring_bytes.div_ceil(SLOT_BYTES))` → `SlotNumber(JOURNAL_RING_START_SLOT + journal_ring_bytes / SL…` | `a_journal_ring_ending_inside_a_slot_rounds_up_and_a_start_off_the_cluster_segment_boundary_is_refused` |
| A 追加 | 实审 A2c 第 15 条：单元区起点不判落不落在聚簇段边界上 | `if !slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {` → `if false && !slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {` | `a_journal_ring_ending_inside_a_slot_rounds_up_and_a_start_off_the_cluster_segment_boundary_is_refused` |
| A 追加 | 实审 A2c 第 15 条：DeviceFreeMap::new 不取默认环的起点 | `Self::with_unit_area_start( ⏎ device, ⏎ device_bytes, ⏎ UnitAreaStart::of_the_default_jour…` → `Self::with_unit_area_start( ⏎ device, ⏎ device_bytes, ⏎ Unit…` | `the_default_journal_ring_puts_the_unit_area_at_the_compiled_slot_and_the_first_file_lands_byte_for_byte_as_before` |
| A 追加 | 实审 A2c 第 15 条：用户数据落点不从这张图的单元区起点起扫 | `let mut candidate = self.unit_area_start.slot().0; ⏎ let end = self.unit_area_end_slot();` → `let mut candidate = self.unit_area_start.slot().0 + CLUSTER_…` | `a_free_map_whose_unit_area_follows_a_128_mebibyte_journal_ring_hands_out_slots_right_after_the_ring` |
| A 追加 | 实审 A2c 第 15 条：开段仍从默认环的起点算段号 | `self.unit_area_start.slot().0 ⏎ + u64::try_from(segment).expect("段号") * CLUSTER_SEGMENT_SL…` → `UnitAreaStart::of_the_default_journal_ring().slot().0 ⏎ + u6…` | `a_free_map_whose_unit_area_follows_a_128_mebibyte_journal_ring_hands_out_slots_right_after_the_ring` |
| A 追加 | 实审 A2c 第 15 条：提交内生块的回落不从这张图的单元区起点起扫 | `let mut candidate = self.unit_area_start.slot().0; ⏎ while candidate + footprint.slots() <…` → `let mut candidate = self.unit_area_start.slot().0 + 2; ⏎ whi…` | `a_free_map_whose_unit_area_follows_a_128_mebibyte_journal_ring_hands_out_slots_right_after_the_ring` |
| A 追加 | 实审 A2c 第 15 条：位图下标仍按默认环的起点减 | `let offset_in_slots = slot.0.checked_sub(self.unit_area_start.slot().0).expect(` → `let offset_in_slots = slot.0.checked_sub(UnitAreaStart::of_t…` | `a_free_map_whose_unit_area_follows_a_128_mebibyte_journal_ring_hands_out_slots_right_after_the_ring` |
| A 追加 | 实审 A2c 第 15 条：is_free 把起点之前一槽也算进单元区 | `slot.0 >= self.unit_area_start.slot().0 ⏎ && slot.0 < self.unit_area_end_slot()` → `slot.0 + 1 >= self.unit_area_start.slot().0 ⏎ && slot.0 < se…` | `a_free_map_whose_unit_area_follows_a_128_mebibyte_journal_ring_hands_out_slots_right_after_the_ring` |
| A 换锚点 | C503（隔离位清零的时机条文与实现说反话）：轮转覆写被抛弃根的根槽时不清隔离位 | `let index = self.index(slot); ⏎ if self.isolated[index] {` → `let index = self.index(slot); ⏎ if false && self.isolated[in…` | `the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot` |
| A 换锚点 | 增补 2 收口第 27 行 ④：隔离位不挡分配（is_free 不看隔离位）；C503 清隔离位之后改名的那条用例判出（盖掉之前那几槽发不出去） | `&& !self.isolated[self.index(slot)]` → `&& true` | `the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot` |
| B 追加 | 实审 A2c 第 26 条：带文件的一版上数记录条数只数一个角色（上限判不出来） | `roles_named_by_each_record_of_the_publish( ⏎ &settled.resolved.rewritten_roles, ⏎ pool.jou…` → `roles_named_by_each_record_of_the_publish( ⏎ &settled.resolv…` | `a_sequential_write_cut_into_more_journal_records_than_the_in_flight_limit_is_refused_before_any_write` |
| B 追加 | 实审 A2c 第 26 条：上限多算一条 | `journal_in_flight_record_limit(journal_ring_bytes).min(u64::from(u32::MAX))` → `journal_in_flight_record_limit(journal_ring_bytes).min(u64::…` | `a_sequential_write_cut_into_more_journal_records_than_the_in_flight_limit_is_refused_before_any_write` |
| B 追加 | 实审 A2c 第 26 条：记录条数等于上限也拒 | `if records_of_the_publish > record_limit {` → `if records_of_the_publish >= record_limit {` | `a_sequential_write_cut_into_as_many_journal_records_as_the_in_flight_limit_is_published` |
| B 追加 | 实审 A2c 第 26 条：树表 0 条的一版上写行数记录条数只数一个角色 | `roles_named_by_each_record_of_the_publish( ⏎ &settled.rewritten_roles, ⏎ pool.journal_reco…` → `roles_named_by_each_record_of_the_publish( ⏎ &settled.rewrit…` | `a_row_publish_on_a_version_without_file_cut_into_more_journal_records_than_the_limit_is_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：覆盖写的 txg 直接加一（溢出 panic） | `let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?; ⏎ publish_ve…` → `let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);…` | `publishes_after_a_version_whose_txg_is_at_the_top_of_its_range_are_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：顺序写的 txg 直接加一（溢出 panic） | `let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?; ⏎ // 改动计数与第一…` → `let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);…` | `publishes_after_a_version_whose_txg_is_at_the_top_of_its_range_are_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：建 inode 的 txg 直接加一（溢出 panic） | `let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?; ⏎ let first_…` → `let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);…` | `publishes_after_a_version_whose_txg_is_at_the_top_of_its_range_are_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：第一个文件版本的 txg 直接加一（溢出 panic） | `let first_file_version_txg = ⏎ checkpoint_txg_of_the_publish_after(version_to_build_on.che…` → `let first_file_version_txg = CheckpointTxg(version_to_build_…` | `publishes_after_a_version_whose_txg_is_at_the_top_of_its_range_are_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：新实例第一次发布的 txg 直接加一（溢出 panic） | `checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg)) ⏎ }` → `Ok(CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 …` | `a_writable_mount_over_a_journal_record_at_the_top_txg_is_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：暖机计划的 txg 直接加一（溢出 panic） | `let next_txg = checkpoint_txg_after_the_version(latest_txg)?;` → `let next_txg = CheckpointTxg(latest_txg.0 + 1);` | `a_writable_mount_whose_warm_up_would_pass_the_top_txg_is_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：要取的实例代号直接加一（溢出 panic） | `highest_on_disk ⏎ .0 ⏎ .checked_add(1) ⏎ .map(InstanceGeneration)` → `Some(highest_on_disk.0 + 1) ⏎ .map(InstanceGeneration)` | `a_writable_mount_over_a_system_configuration_slot_at_the_top_instance_is_refused_before_any_write` |
| B 追加 | 实审 A2c 第 36 条：抬 F 那一串的 txg 直接加一（溢出 panic） | `let txg = checkpoint_txg_after_the_version( ⏎ publishes.last().map_or(current_txg, |previo…` → `let txg = CheckpointTxg(publishes.last().map_or(current_txg,…` | `raising_the_floor_after_a_version_at_the_top_txg_is_refused_before_the_allocator_moves` |
| B 追加 | 实审 A2c 第 36 条：回退那次发布的 txg 直接加一（溢出 panic） | `.0 ⏎ .checked_add(1) ⏎ .map(CheckpointTxg) ⏎ .ok_or(RollbackError::NextCheckpointTxgPastTh…` → `.0 ⏎ .checked_add(0) ⏎ .map(|_| CheckpointTxg(current.root.c…` | `rolling_back_after_a_version_at_the_top_txg_is_refused_before_any_write` |
| B 追加 | 实审 A2c 回退入口：调用方的参数与盘表核不过时不报成自己的成员（报成实例表坏） | `CallerInputsCheckRefused::Disagreeing(disagreeing) => { ⏎ RollbackError::CallerInputsDisag…` → `CallerInputsCheckRefused::Disagreeing(_disagreeing) => { ⏎ R…` | `rolling_back_with_a_device_identity_handed_in_twice_is_refused_before_any_write` |
| B 追加 | 实审 A2c 回退入口：调用方参数与盘上不一致只看盘表那一张 | `if disagreeing_fields.is_empty() && disagreeing_device_table.is_empty() {` → `if disagreeing_device_table.is_empty() {` | `rolling_back_with_other_parameters_is_refused_before_any_write_and_with_the_pool_parameters_is_made` |
| B 追加 | 实审 A2c Q5：抬 F 报错时交回的写行与暖机写账是空的 | `.map(|version| version.writes().clone())` → `.map(|_version| WritesByStructureKind::NOTHING_WRITTEN)` | `a_block_device_error_in_the_first_floor_raise_after_the_row_publish_hands_back_the_writes_of_the_mounts_publishes` |
| B 追加 | 实审 A2c Q5：抬 F 报错时不交回之前推成的那几串 | `writes_of_persisted_publishes, ⏎ floor_raises, ⏎ }),` → `writes_of_persisted_publishes, ⏎ floor_raises: Vec::new(), ⏎…` | `a_block_device_error_in_the_second_floor_raise_after_the_row_publish_hands_back_the_first_raise_too` |
| B 追加 | 实审 A2c Q5：对拍不认装在里面的抬 F 的错（理由报成说不清） | `MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => { ⏎ refusal_reason_of_mount…` → `MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed)…` | `model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside` |
| B 追加 | 实审 A2c Q5：对拍不认装在里面的抬 F 报的上限 | `MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => { ⏎ reported_ceiling_of_mou…` → `MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed)…` | `model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside` |
| B 追加 | 实审 A2c Q5：对拍不认装在里面的抬 F 点名的读坏根环槽 | `MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => { ⏎ root_ring_slot_still_ba…` → `MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed)…` | `model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside` |
| B 换锚点 | 步 3：第一个文件版本的 txg 写死 3，不从它要建在上面的那一版接着算（2026-09-23 用户定案：`FIRST_TRANSACTION_TXG` 只管 mkfs 那条流） | `let first_file_version_txg = ⏎ checkpoint_txg_of_the_publish_after(version_to_build_on.che…` → `let first_file_version_txg = CheckpointTxg(3);` | `a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold` |
| B 换锚点 | P6 后一半：本次发布内序号从 0 起 | `Self(u32::try_from(record_offset_in_this_publish + 1).expect(` → `Self(u32::try_from(record_offset_in_this_publish).expect(` | `journal::tests::the_ordinal_of_the_record_at_each_offset_of_the_publish_is_that_offset_plus_one` |
| B 换锚点 | 实审A1 第23条：可写挂载推抬 F 报的错当成推满仍不够（挂载照样做成） | `) { ⏎ Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => { ⏎ publishes_pushed…` → `) ⏎ .or(Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped…` | `a_block_device_error_in_the_floor_raise_pushed_by_the_mount_fails_the_mount_instead_of_reporting_still_short` |
| B 换锚点 | 实审A1b Q3：挂着之后的入口不查盘表里重复的设备身份 | `refuse_device_identities_handed_in_more_than_once(devices) ⏎ .map_err(CallerInputsCheckRef…` → `let system_configuration = choose_system_configuration(devic…` | `unmounting_with_a_device_identity_handed_in_twice_is_refused_before_any_write` |

表后：同一次还红了别的测试的几行（其余每行都只红它点名的那一条）：
- A「第 34 条：不逐盘核」：`the_first_file_version_refuses_a_genesis_tree_table_that_device_zero_already_released_before_any_write`、`the_first_file_version_refuses_a_genesis_tree_table_that_device_one_already_released_before_any_write`、`the_first_file_version_refuses_an_allocation_record_tree_node_of_the_version_without_file_that_device_one_already_released`
- A「DeviceFreeMap::new 不取默认环的起点」：`the_default_journal_ring_puts_the_unit_area_at_the_compiled_slot_and_the_first_file_lands_byte_for_byte_as_before`、`the_first_file_version_publishes_without_releasing_a_genesis_tree_table_released_on_every_device`、`the_first_file_version_refuses_a_genesis_tree_table_that_device_zero_already_released_before_any_write`、`the_first_file_version_refuses_a_genesis_tree_table_that_device_one_already_released_before_any_write`、`the_first_file_version_refuses_an_allocation_record_tree_node_of_the_version_without_file_that_device_one_already_released`
- B「第 26 条：上限多算一条」：`a_row_publish_on_a_version_without_file_cut_into_more_journal_records_than_the_limit_is_refused_before_any_write`、`a_sequential_write_cut_into_more_journal_records_than_the_in_flight_limit_is_refused_before_any_write`
- B「回退入口：核不过报成实例表坏」：`rolling_back_with_a_device_identity_handed_in_twice_is_refused_before_any_write`、`rolling_back_with_other_parameters_is_refused_before_any_write_and_with_the_pool_parameters_is_made`

## 变异行放在哪（主 agent 追加 / 替换；主表我没动）

- **阶段 A（主工作区现在就要）**：`/tmp/claude-1000/impl-rev-a2c/mutations-append.tsv`（12 行追加，名字以「实审 A2c 第 15 条：」「实审 A2c 第 34 条：」起头，与交回时主表的名字逐个比过、不撞）与 `/tmp/claude-1000/impl-rev-a2c/mutations-replacements.tsv`（2 行整行替换：主表第 430 行「C503（隔离位清零的时机条文与实现说反话）：轮转覆写被抛弃根的根槽时不清隔离位」、第 436 行「增补 2 收口第 27 行 ④：隔离位不挡分配…」，锚点里的 `Self::index(slot)` 改成 `self.index(slot)`，主 agent 途中点名的那两行）。同一份另放在 `/tmp/claude-1000/impl-rev-a2c/patch-phase-a/`（没有 crates.patch），`apply-writer-patch.py` 可以直接打。
- **阶段 B（打补丁时一起）**：`/tmp/claude-1000/impl-rev-a2c/patch/` 里 `crates.patch`、`mutations-append.tsv`（20 行，「实审 A2c 第 26 条 / 第 36 条 / 回退入口 / Q5：」起头）、`mutations-replacements.tsv`（4 行：主表第 64 行「步 3：第一个文件版本的 txg 写死 3…」、第 451 行「P6 后一半：本次发布内序号从 0 起」、第 887 行「实审A1 第23条：可写挂载推抬 F 报的错当成推满仍不够…」、第 942 行「实审A1b Q3：挂着之后的入口不查盘表里重复的设备身份」——补丁改的代码让这四行锚点命中 0 次，不打补丁就不要换）、`report.md`（本报告的拷贝）。
- 行号是交回这一刻主表的；主表别的会话还在写，按第一段的名字找。

## 交回前的验证（末尾原样）

### 「主工作区现状 + 补丁」的副本 `prove-b`（阶段 B）

```text
== build
   Compiling singlefs-format v0.1.0 (/tmp/claude-1000/impl-rev-a2c/prove-b/crates/singlefs-format)
   Compiling singlefs-harness v0.1.0 (/tmp/claude-1000/impl-rev-a2c/prove-b/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 37.17s
build exit=0
== singlefs-harness --test core_review_unit_area_start_and_publish_limits
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s
exit=0
== singlefs-harness --test a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.48s
exit=0
== singlefs-harness --lib
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.28s
exit=0
== singlefs-core --lib
test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
exit=0
wrapper exit=0
```

clippy（check.sh 那一套 lint，副本 `work`，内容与 `prove-b` 里我的文件逐字节相同）：

```text
$ cargo clippy --offline -p singlefs-core --all-targets --all-features -- -D warnings <check.sh 七条>
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.70s
exit=0（这是 is_multiple_of 改完之后重跑的那一次；前一次红在 allocator.rs:216 manual_is_multiple_of，已改）
$ cargo clippy --offline -p singlefs-harness --all-features --lib --test core_review_unit_area_start_and_publish_limits --test a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is --bin first_transaction_on_device -- <同上>
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.19s
exit=0
$ cargo clippy --offline -p singlefs-harness --all-features --lib --profile test -- <同上>
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.29s
exit=0
```

harness 的 clippy 要先编过 checker，而主工作区的 `crates/singlefs-checker/src/walk.rs`（别的会话在改）此刻有 4 条 clippy 错（231、1453、4668 两条），我在草稿副本的 checker `lib.rs` 头上临时加了一行 `#![allow(clippy::type_complexity, clippy::needless_borrow, clippy::shadow_unrelated, …)]` 才走到 harness，跑完从主工作区拷回、`cmp` 相同。harness 整目标（`--all-targets`）上剩下的红在别人的文件：`tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs:21`（unused import）、`src/bin/e158_root_choice_repair.rs:7244/7245/9801`（shadow_unrelated），`work-clippy-harness-3.log`。

### 主工作区（阶段 A）

```text
== cargo fmt --all -- --check
exit=1
     67 Diff in crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
== rustfmt --check 我的四份
exit=0
== cargo clippy --all-targets --all-features（check.sh 那一套）
exit=101
      3     --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs
      2    --> crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 3 previous errors
== cargo clippy -p singlefs-core --all-targets --all-features
exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
== cargo build --offline --all-targets
exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.92s
script exit=0

== cargo test -p singlefs-harness --test core_review_unit_area_start_and_publish_limits
exit=0
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
== cargo test -p singlefs-core --lib
exit=0
test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
== gate 33-mutation-tables.sh
      crates/mutations.tsv:430 C503（隔离位清零的时机条文与实现说反话）：轮转覆写被抛弃根的根槽时不清隔离位：原文在 crates/singlefs-core/src/allocator.rs 里命中 0 次
      crates/mutations.tsv:436 增补 2 收口第 27 行 ④：隔离位不挡分配（is_free 不看隔离位）；C503 清隔离位之后改名的那条用例判出（盖掉之前那几槽发不出去）：原文在 crates/singlefs-core/src/allocator.rs 里命中 0 次
    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。
exit=1
== gate 53-format-const-placeholders.sh
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
exit=0
== gate 74-model-differential.sh
     → 怎么办：单跑看细节（经内存包装，上限同这一道）：
                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。
exit=1
== gate 92-layout-checker-sync.sh
      跟上了：改值 SYSTEM_CONFIGURATION_BYTES：481 → 489（crates/singlefs-format/src/lib.rs）
    没抽到常量的格式定义路径 1 条（第 ④ 条对它们没有对象可判，只受第 ②③ 条管）：
      第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）：.claude/kb/layout/02-second-txn.md
exit=0
== gate 94-checker-implementation-disjoint.sh
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 285 行里没有分支与循环（`#[cfg(test)]` 标着的项 271 行不扫）
    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑
exit=0
== gate 93-feature-bits.sh
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
exit=0
== gate 89-closeout-row27-preconditions.sh
      alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像：扣住位只住内存、盘上没有落点，没有可扫的对象
    收口表第 27 行现算 5 笔；探针与没做成探针的清单两边都没有的 1 笔：被抛弃根的根槽读不出时既不隔离也不计数
    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐
exit=77
script exit=0
```

- fmt：全仓 `cargo fmt --check` 红在别人的 `e158_root_choice_repair.rs`（67 处）；我的四份单独 `rustfmt --check` 绿。
- clippy：全仓那一次红在别人的 `e156_allocation_basis_counts.rs`（3 处）与 `checker_narrow_invariants_and_abandoned_roots.rs`（2 处）；`-p singlefs-core` 整包 0 条。
- 33 号红的正是主 agent 点名的 430、436 两行（我的 allocator 改动带断的），换锚点的两行在 `mutations-replacements.tsv`、已证红；换上之后我按 33 号的判法（每行原文在源码里恰好命中一次）在两份副本上逐行核：`prove-a`（主工作区 + 阶段 A 那 14 行）1047 行 0 行不合、`prove-b`（再加补丁与阶段 B 那 24 行）1067 行 0 行不合（33 号脚本本身对副本退 77「找不到 research/…」，没判）。
- 74 号红的两条（`crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`、`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`）与调查员报告 `research/prompts/m2-investigate-gate74-reds-report.md` 第 54–55 行点名的是同两条，主 agent 已说归 C554、不处理。
- 89 号退 77（本次未跑），按没判写；53、92、93、94 绿（报数在上面原样）。

## `git diff --stat -- crates litmus`（主工作区，原样；别的会话同时在改 crates/，这张表分不出谁改的，我的以「这一轮写过的文件」为准；新测试文件还没被 git 跟踪、不在表里）

```text
 crates/mutations.tsv                               |  734 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  136 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1915 +++--
 crates/singlefs-core/src/admission.rs              |  629 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  302 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   37 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  199 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   43 +-
 crates/singlefs-core/src/recovery.rs               |  712 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  506 +-
 crates/singlefs-core/src/unit.rs                   |   43 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   80 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6818 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2505 ++++++-
 crates/singlefs-harness/src/crash_injection.rs     |  821 ++-
 crates/singlefs-harness/src/device_log.rs          |   38 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1122 ++-
 crates/singlefs-harness/src/lib.rs                 |   60 +-
 crates/singlefs-harness/src/model.rs               |  985 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  338 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |  106 +-
 .../tests/checker_known_bad_images.rs              |  986 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_five_publish.rs   |   22 +-
 .../tests/first_transaction_step_one_mkfs.rs       |    8 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  261 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 .../second_transaction_parallel_line_one_layer0.rs |   64 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...action_parallel_line_three_spill_over_layer0.rs |   36 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 ..._transaction_position_addressed_trees_layer0.rs |   30 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 .../tests/second_transaction_step_one_overwrite.rs |    3 +-
 ...action_step_three_acquisition_barrier_layer0.rs |   55 +-
 ...second_transaction_step_three_formatted_pool.rs |  688 +-
 ...transaction_step_three_formatted_pool_layer0.rs |   36 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  658 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  346 +-
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
 ...transaction_supplement_two_tree_split_layer0.rs |   40 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 94 files changed, 30231 insertions(+), 11289 deletions(-)
```

## 删掉的副本与留下的材料

- 删了：`/tmp/claude-1000/impl-rev-a2c/work`（15G，阶段 B 的开发副本）、`/tmp/claude-1000/impl-rev-a2c/prove-a`（2.0G，阶段 A 证红副本）、`/tmp/claude-1000/impl-rev-a2c/prove-b`（14G，「主工作区 + 补丁」证红与验证副本）、`/tmp/claude-1000/impl-rev-a2c/prefmt`（1.3M，rustfmt 之前的几份拷贝）。
- 留着：`patch/`（补丁、阶段 B 变异行、报告拷贝）、`patch-phase-a/`（阶段 A 变异行、报告拷贝）、`mutations-append.tsv` 与 `mutations-replacements.tsv`（阶段 A，同 `patch-phase-a/`）、`patch-files-after-the-patch/`（补丁打上之后那 9 份的整文件，主工作区漂了补丁打不上时对照用）、`orig/`（开工快照，算「我的改动」用）、各份日志（`prove-*-final*`、`verify-*.log`、`gate-*.log`、`pre-fix-34.log`）、`scratch/`（变异行的 JSON 与生成脚本、两段验证脚本）、`progress.md`。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7、crates 变异整表（门禁 59 号）都没跑，归 `crash-verifier`；没提交。
- 阶段 B（第 26、36 条、Q5、回退入口）没进主工作区，是补丁，理由见「结论」；打不打由主 agent 定。
- 第 15 条挂载一侧没切（「停下交主 agent 的」第 1 条），`mount.rs` 的 `format_time_allocator` / `rebuilt_allocator` 一行没动。
- 零单元发布不判记录条数上限、`acquire_instance` 的实例代号到顶仍 panic（第 4、5 条）。
- `first_transaction_on_device.rs` 对 Q5 新成员不描写窗口账（第 7 条）；这个二进制没跑（要真设备）。
- 全仓 fmt / clippy 红在别人的文件上，没修；`research/scripts/capped.sh` 途中被别的会话插坏过一阵（第 22 行语法错），那一段我改用开工快照里的那一份起命令，没修它（主 agent 后来说已修好）。
- 模型对拍（74 号）里那两条红照旧，没处理（主 agent 说归 C554）。
- 故障注入、崩溃注入的随机档没跑：阶段 A 不改默认环下的落点、阶段 B 只加写之前的拒绝与改错的形态，按注入序号摆点的用例理应不挪位（推的，没跑）。
