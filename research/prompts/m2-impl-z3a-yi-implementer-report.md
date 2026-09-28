# 实 Z3-A 乙：挂着之后的每个入口都逐盘核，含会话发布——实现员报告

写于 2026-09-27。规格 `/tmp/claude-1000/impl-z3a-yi/spec.md`；交补丁（在副本里改，主工作区没碰）。
补丁目录 `/tmp/claude-1000/impl-z3a-yi/patch/`：`crates.patch`、`mutations-append.tsv`（8 行）、`report.md`（本报告的拷贝）。

## 一、结论

- 挂着之后收盘表的五个入口（正常卸载、管理员回退、抬 F、准入抬 F、一次准入里再推一串）共用的 `caller_inputs_agreeing_with_the_disk` 在参数与盘表核过之后，
  逐盘核「可见」：两个系统配置槽里一份本池自证过的（整槽校验和过、fsid 与本池相同）都没有就拒，报在已有的
  `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration` / `RollbackError::CallerInputsDisagreeWithTheDisk(ParametersOrDeviceTable…)`
  的 `disagreeing_device_table` 里，新成员 `DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn { identity_handed_in }`。
- 会话每次 `publish_user_change` 在身份比对与「有没有带文件的版本」之后、任何写之前读每块盘两槽核一遍，不可见就拒成新成员
  `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { devices }`。
- 「同一判」：可写挂载取号之前的逐盘核第一支（`devices_without_the_selected_version`）、挂着之后的入口、会话三处都经同一个读法
  `self_verified_system_configuration_slots_of_the_pool`（补丁后 `crates/singlefs-core/src/mount.rs:2927`），可写挂载那一处只是改成调它，行为不变，报的仍是 D18 已定项 11 点名的 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`。
- 验收（规格表第 1–4 行）：三形（卸载、回退到现行那一版、会话发布）都拒、两块盘逐字节不变、录制流一步都没有；抬 F 三个入口同样拒；
  别的池的盘换进来，挂着之后的入口照旧被择系统配置拒（`Recovery(SystemConfigurationsDisagree)`，对照），会话不择系统配置、由这一判拒；
  一块盘两槽坏一槽仍可见（边界）。改前改后 60 次会话发布每块盘各多读 120 次（每次发布每盘 2 次），见第六节。每处改法各有「改回去它就红」的变异，8 行 prove-red 全抓到。
- **补丁单独打上去编不过**：新会话成员让两处穷举 match 缺分支——`crates/singlefs-harness/src/history.rs`（B3c-3 在改，2 处）与
  `crates/singlefs-harness/tests/common_admission/mod.rs`（不在这一件的文件清单里，1 处）。要加的分支原文在第七节，已做成两份 diff；
  我的副本里加了这两处才编得过、跑得了，补丁里没有。
- 推翻条件：主工作区打上补丁与第七节两处分支之后，新测试二进制
  `entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration` 有一条红；或把第五节任一行变异施加上去、点名的测试不红；
  或盘 1 换成空盘时 `unmount` / `roll_back_by_a_forward_publish` / `publish_user_change` 任一返回 `Ok`、或录制流里有一步。

## 二、这一轮写过的文件

补丁里（从仓根起）：

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-core/src/mount.rs` | `DeviceTableDisagreement` 加第三个成员（补丁后第 506 行）并改它与 `CallerParametersDisagreeWithTheSelectedSystemConfiguration` 的文档；新函数 `self_verified_system_configuration_slots_of_the_pool`（第 2927 行，私有）与 `devices_without_a_self_verified_system_configuration`（第 2946 行，`pub(crate)`，会话也调）；`devices_without_the_selected_version` 第一支改调前者（第 2873、2876 行，行为不变）；`caller_inputs_agreeing_with_the_disk`（第 3179 行）在参数与盘表核过之后加这一判（第 3195–3210 行）并改文档 |
| `crates/singlefs-core/src/mounted_session.rs` | `UserChangeRefused` 加成员（补丁后第 84 行）；`publish_user_change` 在第 242–250 行加这一判；模块、枚举、方法文档跟着改（「五种」→「六种」） |
| `crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs` | 新测试文件，8 条（第四节） |
| `crates/mutations.tsv` | 只在末尾追加 8 行（`patch/mutations-append.tsv`），变异名见第五节 |

`crates/singlefs-core/src/lib.rs` 不用动：新成员都在已导出的枚举里，新函数是 `pub(crate)`。
补丁里没有、只在我的副本 `/tmp/claude-1000/impl-z3a-yi/work/` 里为了编译加的：`crates/singlefs-harness/src/history.rs` 两处、`crates/singlefs-harness/tests/common_admission/mod.rs` 一处（第七节）。

`crates.patch` 对主工作区 `git apply --check` 退 0（2026-09-27 与交回前各核一次，交回前那次的输出在第八节）。补丁的 diffstat（主工作区没打，`git diff --stat -- crates litmus` 看不到它，这里贴补丁自己的）在第八节。

## 三、条款与代码对照（规格表第 4 行）

D18 已定项 11 第 314 行（`.claude/kb/decisions/18-块里携带什么信息.md` 现取；开工时读的那一版第 314 行还没有这一句，是这一轮当中第三批书记员写进去的）末尾那一句，原文整句：

> 挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前同样逐盘核，一块盘一份本池 fsid 的自证系统配置槽都没有就拒；实现随「Z3-A 乙」那一件（还没派，写「实现待派」）。

逐项对：

| kb 那一句 | 代码 | 一致吗 |
|---|---|---|
| 入口：正常卸载、管理员回退、会话每次发布、抬 F | `unmount`、`roll_back_by_a_forward_publish`、`raise_rollback_floor`、`raise_rollback_floor_to_the_admission_ceiling`、`push_one_floor_raise_within_the_admission_budget`（都经 `caller_inputs_agreeing_with_the_disk`）；会话 `MountedSession::publish_user_change` | 一致；kb 的「抬 F」代码里是三个入口（直接给新 F、准入抬 F、再推一串），三个都核了，测试 `raising_the_floor_…_on_every_raise_entry` 逐个钉 |
| 在任何写之前 | 卸载、回退、抬 F：`raise_the_floor_through` 与回退在冻结检查之后第一件事调 `caller_inputs_agreeing_with_the_disk`；会话在进发布循环之前 | 一致；测试钉「录制流 0 步、两块盘逐字节不变」 |
| 「同样逐盘核」 | **只做了可写挂载逐盘核的第一支（不可见）**；第二支（系统配置落后**并且**缺所选那一版的单元）挂着之后不核 | **措辞有歧义**：「同样逐盘核」可以读成两支都核。规格表第 1 行只要第一支，代码照规格。建议 kb 写成「同样核『可见』那一支（不核落后那一支）」 |
| 一块盘一份本池 fsid 的自证系统配置槽都没有就拒 | `self_verified_system_configuration_slots_of_the_pool`：`recovery::verified_system_configuration_slots` 读两槽、整槽校验和过、fsid 等于盘上择到的那份系统配置里的 fsid，结果为空就拒 | 一致；fsid 取的是盘上择到的（挂着之后的入口）或挂载时择到的（会话 `parameters_on_disk`），不是调用方交进来的 |
| （D22 已定项 16）每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距；择槽取校验和过的 | 读偏移 0 与 1 × 本池 `fixed_structure_slot_spacing`（盘上择到的那份系统配置里的），各 4096 字节，`SystemConfiguration::parse_slot` 过的才算 | 一致；两槽只要一槽过就算可见（边界测试钉着） |
| 会话**每次**发布 | 每次 `publish_user_change` 调用核一次，在循环之前；循环里为空间不够推的每一串抬 F 自己经 `caller_inputs_agreeing_with_the_disk` 再核；抬 F 之后重发的那一次不另核 | 按「每次用户改动」读是一致的；按「每次发布动作（含循环里的重发）」读不一致。一次调用里盘表被 `&mut` 借着、换不了盘，重发之前紧挨着的是那一串抬 F 自己的核，所以每个写之前都核过——这是推的，没有用例在循环中途把盘换掉（换不了） |
| 拒成什么 | 卸载、抬 F：`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn { identity_handed_in }] }`；回退：`RollbackError::CallerInputsDisagreeWithTheDisk(ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {…})`；会话：`UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { devices }` | kb 那一句没写错误成员；可写挂载那一句写了 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`。建议 kb 把这三个成员名补进去、把「实现待派」换成这一件 |
| 判的次序 | 挂着之后的入口：重复身份 → 择系统配置 → 参数与盘表 → 可见；会话：身份 → 有没有带文件的版本 → 可见 | kb 没写次序。后果：别的池的盘在挂着之后的入口被择系统配置先拒（报 `Recovery(SystemConfigurationsDisagree)`，不报「不可见」），在会话里报「不可见」；参数也不对时只报参数那一项、不连带报不可见的盘 |

## 四、新测试与「先红后改」

测试文件 `crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs`，起点照攻方：两块 4 GiB 稀疏盘 mkfs、取号、暖机、第一个文件（txg 3），会话里覆盖写两次（txg 4、5）；盘表 [(盘 0, 盘 0 此刻的拷贝), (盘 1, 换上去的盘)]。

| 行号 | 测试 | 钉什么 | 搬的哪条攻方用例 |
|---|---|---|---|
| 249 | `unmount_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_and_the_real_disk_still_unmounts` | 卸载拒、只点名盘 1、录制流 0 步、两块盘逐字节不变、现行版本不动；换回真盘 1 同一个会话照常卸载 | `unmount_with_device_one_replaced_by_a_blank_disk` |
| 295 | `rolling_back_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_for_the_current_and_an_older_target` | 回退到 (1, 5) 与 (1, 3) 都由这一判拒（(1, 3) 改前是复活集逐盘验顺手拒的） | `raising_the_floor_and_rolling_back_…` 的回退两半 |
| 356 | `raising_the_floor_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_on_every_raise_entry` | 抬 F 三个入口都由这一判拒（改前直接给新 F 那一个是读根环那一判侧面拒） | 同上抬 F 那一半，另补准入抬 F、再推一串 |
| 423 | `the_session_refuses_to_publish_with_device_one_replaced_by_a_blank_disk_before_any_write` | 会话拒、点名盘 1、空盘仍一个扇区都没有、现行版本不动；换回真盘 1 照常发布 | `the_session_publishes_with_device_one_replaced_by_a_blank_disk` |
| 468 | `the_session_refuses_to_publish_with_device_one_replaced_by_device_one_of_another_pool` | 别的池的盘 1（两槽自证得过、fsid 不同）：会话由这一判拒 | 新增（攻方只试了卸载那一形） |
| 502 | `unmount_with_device_one_replaced_by_device_one_of_another_pool_is_still_refused_when_choosing_the_system_configuration` | 对照：挂着之后的入口照旧被择系统配置拒，0 步、逐字节不变 | `unmount_with_device_one_replaced_by_device_one_of_another_pool` |
| 531 | `a_device_with_one_of_its_two_system_configuration_slots_corrupted_is_still_visible_to_the_session_and_to_unmount` | 边界：盘 1 槽 0 改坏一字节，仍可见，会话照常发布、之后照常卸载 | 新增 |
| 575 | `the_session_refusal_reads_the_two_system_configuration_slots_of_each_device_and_writes_nothing` | 会话这一拒的代价：每块盘读 2 次、写 0、屏障 0（块层计数，`SharedFaultPlan::counts_of_device`） | 新增 |

**先红后改**：基线副本 `/tmp/claude-1000/impl-z3a-yi/baseline/`＝开工时主工作区的源码，只在 `mount.rs`、`mounted_session.rs` 里各加一个成员声明（不加任何判定，行为与今天相同）好让同一份测试编得过，另加第七节两处分支。同一个测试二进制在基线上 6 红 2 绿（日志 `/tmp/claude-1000/impl-z3a-yi/run-baseline-2.log`，原样摘录）：
    test the_session_refusal_reads_the_two_system_configuration_slots_of_each_device_and_writes_nothing ... FAILED
    test raising_the_floor_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_on_every_raise_entry ... FAILED
    test a_device_with_one_of_its_two_system_configuration_slots_corrupted_is_still_visible_to_the_session_and_to_unmount ... ok
    test rolling_back_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_for_the_current_and_an_older_target ... FAILED
    test the_session_refuses_to_publish_with_device_one_replaced_by_a_blank_disk_before_any_write ... FAILED
    test the_session_refuses_to_publish_with_device_one_replaced_by_device_one_of_another_pool ... FAILED
    test unmount_with_device_one_replaced_by_device_one_of_another_pool_is_still_refused_when_choosing_the_system_configuration ... ok
    test unmount_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_and_the_real_disk_still_unmounts ... FAILED
    test result: FAILED. 2 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

红的那 6 条各自的断言（原样，线程号删掉）：

    thread 'the_session_refusal_reads_the_two_system_configuration_slots_of_each_device_and_writes_nothing' panicked at crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs:599:5:
    盘 1 换成空盘，会话要拒，实际 Ok(UserChangePublished { floor_raises: [], refusals_that_pushed_the_floor_raises: [] })
    thread 'raising_the_floor_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_on_every_raise_entry' panicked at crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs:159:23:
    抬 F 到 3（直接给新 F）：盘 1 不可见，要拒成 CallerParametersDisagreeWithTheSelectedSystemConfiguration，实际 RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot: RootRingSlot { region: 1, slot: 0 }, first_reading: NotSelfVerified, reread: NotSelfVerified }
    thread 'rolling_back_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_for_the_current_and_an_older_target' panicked at crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs:335:32:
    回退到 (1, 5)：盘 1 不可见，要在任何写之前拒，实际做成了（攻方那一轮的样子）：RolledBack { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(5) }, released_user_visible_units: [], quarantined_user_visible_copies: [], resurrected_user_visible_copies: [] }
    thread 'the_session_refuses_to_publish_with_device_one_replaced_by_a_blank_disk_before_any_write' panicked at crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs:440:9:
    盘 1 换成空盘，会话要拒成 DevicesWithoutASelfVerifiedSystemConfiguration，实际 Ok(UserChangePublished { floor_raises: [], refusals_that_pushed_the_floor_raises: [] })
    thread 'the_session_refuses_to_publish_with_device_one_replaced_by_device_one_of_another_pool' panicked at crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs:487:9:
    盘 1 是别的池的，会话要拒成 DevicesWithoutASelfVerifiedSystemConfiguration，实际 Ok(UserChangePublished { floor_raises: [], refusals_that_pushed_the_floor_raises: [] })
    thread 'unmount_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_and_the_real_disk_still_unmounts' panicked at crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs:162:18:
    正常卸载：盘 1 不可见，要在任何写之前拒，实际做成了（攻方那一轮的样子）

## 五、证红（`research/scripts/prove-red.sh`）

命令（副本 `/tmp/claude-1000/impl-z3a-yi/work/`，它的 `crates/mutations.tsv` 末尾已追加这 8 行；参数不带测试名过滤，每条都跑整个测试二进制，好看同时红了哪几条）：

    PROVE_RED_LOG_DIRECTORY=/tmp/claude-1000/impl-z3a-yi/prove-red-logs nice -n 19 bash research/scripts/capped.sh 4 bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-z3a-yi/work singlefs-harness <下表 8 个变异名>

基线（不改源码）那一次：`test result: ok. 8 passed; 0 failed`（`prove-red-logs/baseline.log`），基线红集为空。原样输出：

    Z3-A 乙：挂着之后收盘表的入口不核盘「可见」（正常卸载收到空盘）	抓到	unmount_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_and_the_real_disk_still_unmounts 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/001.log）
    Z3-A 乙：挂着之后收盘表的入口不核盘「可见」（管理员回退收到空盘）	抓到	rolling_back_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_for_the_current_and_an_older_target 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/002.log）
    Z3-A 乙：挂着之后收盘表的入口不核盘「可见」（抬 F 三个入口收到空盘）	抓到	raising_the_floor_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_on_every_raise_entry 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/003.log）
    Z3-A 乙：会话发布之前不核盘「可见」（空盘）	抓到	the_session_refuses_to_publish_with_device_one_replaced_by_a_blank_disk_before_any_write 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/004.log）
    Z3-A 乙：会话发布之前不核盘「可见」（别的池的盘）	抓到	the_session_refuses_to_publish_with_device_one_replaced_by_device_one_of_another_pool 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/005.log）
    Z3-A 乙：会话发布之前不核盘「可见」（拒之前每块盘只读两槽）	抓到	the_session_refusal_reads_the_two_system_configuration_slots_of_each_device_and_writes_nothing 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/006.log）
    Z3-A 乙：「可见」判成两槽都要自证过（只坏一槽的盘被拒）	抓到	a_device_with_one_of_its_two_system_configuration_slots_corrupted_is_still_visible_to_the_session_and_to_unmount 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/007.log）
    Z3-A 乙：「可见」挪到择系统配置之前、拿调用方的参数核（别的池的盘不再由择系统配置拒）	抓到	unmount_with_device_one_replaced_by_device_one_of_another_pool_is_still_refused_when_choosing_the_system_configuration 红了（日志 /tmp/claude-1000/impl-z3a-yi/prove-red-logs/008.log）
    ✓ 点名 8 条：跑了 8 条，跳过 0 条，跑的都抓到了
    prove_red_exit=0

逐条「改坏哪一行 → 哪条断言红」与同时红了哪些（行号是补丁后的；断言原文取自各条日志）：

| # | 变异名 | 改坏哪一行 | 点名的测试红在哪条断言 | 同时红的 |
|---|---|---|---|---|
| 1 | Z3-A 乙：挂着之后收盘表的入口不核盘「可见」（正常卸载收到空盘） | `mount.rs:3203` `if !devices_not_visible.is_empty() {` → `if false {` | 测试第 162 行 `device_table_refusal_of` 的 `Ok(_)` 臂：「正常卸载：盘 1 不可见，要在任何写之前拒，实际做成了（攻方那一轮的样子）」 | 回退、抬 F 两条 |
| 2 | …（管理员回退收到空盘） | 同 1 | 测试第 335 行：「回退到 (1, 5)：盘 1 不可见，要在任何写之前拒，实际做成了…：RolledBack {…}」 | 卸载、抬 F 两条 |
| 3 | …（抬 F 三个入口收到空盘） | 同 1 | 测试第 159 行：「抬 F 到 3（直接给新 F）：…实际 RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {…}」 | 卸载、回退两条 |
| 4 | Z3-A 乙：会话发布之前不核盘「可见」（空盘） | `mounted_session.rs:243` `if !devices_not_visible.is_empty() {` → `if false {` | 测试第 440 行：「盘 1 换成空盘，会话要拒成 DevicesWithoutASelfVerifiedSystemConfiguration，实际 Ok(UserChangePublished {…})」 | 别的池的盘、读次数两条 |
| 5 | …（别的池的盘） | 同 4 | 测试第 487 行：「盘 1 是别的池的，会话要拒成 …，实际 Ok(…)」 | 空盘、读次数两条 |
| 6 | …（拒之前每块盘只读两槽） | 同 4 | 测试第 599 行：「盘 1 换成空盘，会话要拒，实际 Ok(…)」 | 空盘、别的池的盘两条 |
| 7 | Z3-A 乙：「可见」判成两槽都要自证过（只坏一槽的盘被拒） | `mount.rs:2959` `.is_empty()` → `.len() < 2` | 测试第 554 行 `expect`：「盘 1 只坏了一槽，另一槽自证得过：仍可见，会话照常发布: DevicesWithoutASelfVerifiedSystemConfiguration { devices: [DeviceIdentity(1)] }」 | 无 |
| 8 | Z3-A 乙：「可见」挪到择系统配置之前、拿调用方的参数核（别的池的盘不再由择系统配置拒） | `mount.rs:3185` 前面插一段：拿调用方参数先核可见、不可见就报一张空清单 | 测试第 519 行：「别的池的盘：要被择系统配置那一步拒（SystemConfigurationsDisagree），实际 CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [] }」 | 卸载、回退、抬 F 三条（替换文报的清单是空的） |

8 行都证过、没有留给 59 号只复跑的行；测试二进制名不带 layer0，没有跳过的。行 1–3 是同一处改法各点一条测试、行 4–6 同理（`crates/mutations.tsv` 一行只点一条必须红的测试）。

## 六、读次数：60 次会话发布，改前改后每块盘

草稿用例 `/tmp/claude-1000/impl-z3a-yi/measure_session_reads_over_sixty_publishes.rs`（不进补丁）：两块 4 GiB 稀疏盘 mkfs、取号、暖机、第一个文件，之后经会话覆盖写 60 次。读数两路：装置已有的块层计数（`FaultInjectingBlockDevice` 包一层、计划不上膛，`SharedFaultPlan::counts_of_device(..).since(..)`），与草稿里一个只按落点分的包装（偏移落在两个系统配置槽里的读）。改前跑在基线副本、改后跑在补丁副本，同一份源码。原样输出：

改前（`/tmp/claude-1000/impl-z3a-yi/measure-before.log`）：

    MEASURE harness_block_layer device=0 publishes=60 reads=3092 writes=974
    MEASURE harness_block_layer device=1 publishes=60 reads=2012 writes=954
    MEASURE slot_breakdown device=0 publishes=60 reads=3092 reads_of_the_system_configuration_slots=360
    MEASURE slot_breakdown device=1 publishes=60 reads=2012 reads_of_the_system_configuration_slots=360

改后（`/tmp/claude-1000/impl-z3a-yi/measure-after.log`）：

    MEASURE harness_block_layer device=0 publishes=60 reads=3212 writes=974
    MEASURE harness_block_layer device=1 publishes=60 reads=2132 writes=954
    MEASURE slot_breakdown device=0 publishes=60 reads=3212 reads_of_the_system_configuration_slots=480
    MEASURE slot_breakdown device=1 publishes=60 reads=2132 reads_of_the_system_configuration_slots=480

每块盘多读 120 次 = 60 次发布 × 2，全落在两个系统配置槽上（槽读 360 → 480，即每次发布每盘 6 次 → 8 次）；写次数不变（974 / 954）。规格里「每次发布每块盘多读两槽（推的）」量下来就是这个数。改前发布路径每次发布每盘已经读这两槽 6 次（是哪几处读的没逐个查；`transaction.rs` 里 `verified_system_configuration_slots` 有两处调用），这一判没有复用那些读数——复用要动 `transaction.rs`（B3c-3 在改），没做。

## 七、补丁之外要加的分支（不在这一件的文件清单里，我没改主工作区）

新成员 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration` 让下面三处穷举 match 编不过（`MountError` 没有加成员，`history.rs` 的 `MountError` 穷举与 `first_transaction_on_device.rs` 不受影响；`DeviceTableDisagreement` 加的成员在仓里没有穷举 match，只有测试里按值构造）。
我的副本里照下面原样加了才编得过；两份 diff 放在草稿目录，对开工时的主工作区能直接 `git apply`：
`/tmp/claude-1000/impl-z3a-yi/arms-history.diff`、`/tmp/claude-1000/impl-z3a-yi/arms-common-admission.diff`。

`crates/singlefs-harness/src/history.rs`（B3c-3 在改；开工时第 2750–2751 行与第 2825–2828 行之后）：

```diff
         Err(
             UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
-            | UserChangeRefused::NoFileVersionToChange,
+            | UserChangeRefused::NoFileVersionToChange
+            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. },
         ) => &[],
```

```diff
         UserChangeRefused::NoFileVersionToChange => (
             "UserChangeRefused::NoFileVersionToChange".to_string(),
             ObservedRefusalReason::Unexplained,
         ),
+        // 执行器交的是整池那两块盘、没换过盘（Z3-A 乙那一判）：走不到，走到了就按说不出理由的拒绝判。
+        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => (
+            "UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration".to_string(),
+            ObservedRefusalReason::Unexplained,
+        ),
```

「走不到」是推的：随机历史每步交整池那两块盘；故障注入一次只注一个故障（`FaultSchedule::the_nth_call_across_the_pool`），一次读错或读回改坏只坏一槽的一次读，另一槽还在，这一判放行。

`crates/singlefs-harness/tests/common_admission/mod.rs`（开工时第 255–256 行，`PoolUnderTest::publish` 里）：

```diff
             Err(
                 UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
-                | UserChangeRefused::NoFileVersionToChange,
+                | UserChangeRefused::NoFileVersionToChange
+                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. },
             ) => {}
```

另一种改法（没采用，交主 agent 定）：不加成员，在 `DeviceTableOtherThanTheOneOfTheMount` 里加一个字段装不可见的盘。那样上面三处 `{ .. }` 不用动，但 `entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write.rs` 第 172–175 行把这个成员两个字段逐个解构、没写 `..`，照样编不过；而且那个成员的文档与调用方要做的决定是「身份对不上、换一份盘表」，与「盘换了、停下这块盘上的写」不是一回事（`code-discipline.md`「错误：能恢复的写进类型」：成员按调用方要做的决定分）。

## 八、交回前的验证（末尾原样输出）

跑前 `ps -o pid,args -u "$(id -u)"` 看过：没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，也没有别的 `cargo`、`gate.sh`，没等锁。每条 cargo 都经 `research/scripts/capped.sh 4`，跑编出来的代码的都经 `run-with-memory-cap.sh 8G`（上限取 `replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值；派发提示没给）。全部跑在补丁副本 `/tmp/claude-1000/impl-z3a-yi/work/`（副本里多了第七节那三处分支，补丁里没有）。

`cargo fmt --all -- --check`（先 `cargo fmt --all` 排过版）：退 0、没有输出。

`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `check.sh` 里 `CODE_DISCIPLINE_LINTS` 那 7 条 `-D`（`/tmp/claude-1000/impl-z3a-yi/clippy.log` 末尾）：

        Checking singlefs-harness v0.1.0 (/tmp/claude-1000/impl-z3a-yi/work/crates/singlefs-harness)
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.71s
    clippy_exit=0

`cargo build --offline --all-targets`（`build-all.log` 末尾）：

        Finished `dev` profile [unoptimized + debuginfo] target(s) in 49.37s
    build_exit=0

`cargo test --offline -p singlefs-core --lib`（我改了这个 crate 的库源码；`core-lib.log` 末尾）：

    test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.63s
    core_lib_exit=0

新测试二进制 `cargo test --offline -p singlefs-harness --test entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration`（`run-work-final.log`）：

    test the_session_refusal_reads_the_two_system_configuration_slots_of_each_device_and_writes_nothing ... ok
    test rolling_back_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_for_the_current_and_an_older_target ... ok
    test raising_the_floor_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_on_every_raise_entry ... ok
    test the_session_refuses_to_publish_with_device_one_replaced_by_device_one_of_another_pool ... ok
    test a_device_with_one_of_its_two_system_configuration_slots_corrupted_is_still_visible_to_the_session_and_to_unmount ... ok
    test the_session_refuses_to_publish_with_device_one_replaced_by_a_blank_disk_before_any_write ... ok
    test unmount_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_and_the_real_disk_still_unmounts ... ok
    test unmount_with_device_one_replaced_by_device_one_of_another_pool_is_still_refused_when_choosing_the_system_configuration ... ok
    test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
    new_test_exit=0

登记给实现员的门禁阶段（`stage-owners.tsv` 列出 7 个），在补丁副本里逐个跑，末行与退出码：

- `33-mutation-tables.sh`：退 0

      ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1161 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 7 张：research/mutations/e125_zoned_wp.tsv research/mutations/e129_tear_injector.tsv research/mutations/e129_thin_neighbour.tsv research/mutations/e129_thin_rmw.tsv research/mutations/e158_arms.tsv research/mutations/e158_r3_arm_mutations.tsv research/mutations/e158_r4_arm_mutations.tsv （本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）

- `53-format-const-placeholders.sh`：退 0

      ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））

- `89-closeout-row27-preconditions.sh`：退 77

        「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐

- `92-layout-checker-sync.sh`：退 77

      ! /tmp/claude-1000/impl-z3a-yi/work 不是 git 仓，本阶段跳过

- `93-feature-bits.sh`：退 0

      ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））

- `94-checker-implementation-disjoint.sh`：退 0

        这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑

33 号查的是副本里的变异表（已追加这 8 行）：1161 条原文各命中一次。89 号退 77（本次未跑，它说收口表第 27 行那几笔的前置一个都没进来，这一件不碰那几笔）。92 号退 77，因为副本不是 git 仓，**没判**；这一件没动 `layouts.tsv` 里任何格式定义或 checker 路径（补丁只动 `mount.rs`、`mounted_session.rs` 与一份测试），按它的判据推的应当是 77 或绿，没跑出来。

- `74-model-differential.sh`（`SINGLEFS_GATE_FULL=1`，副本里跑）：**退 1，红 3 条**，末尾：

    test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 79.23s
    exit=1

    红的 3 条：crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43、random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation、rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline。
    **不是这个补丁引起的**：同一个测试二进制（`cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture`）在基线副本（今天的行为）上红的是同样这 3 条，
    两边红的都是 `MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`，快档都是「新发现 46」、这个成员都是 46 次；
    补丁副本的日志里本补丁的两个新成员出现 0 次。两份日志 `/tmp/claude-1000/impl-z3a-yi/random-history-work.log`、`random-history-baseline.log`，原样摘录：

    [random-history-work.log]
      Err 成员 MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)：46 次
    test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 82.49s
    历史 96 段：跑完 50、以已知红收尾 {}、新发现 46；根环转过一圈的 43 段；最高 txg 44；一版里最多 746 条分配记录
    [random-history-baseline.log]
      Err 成员 MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)：46 次
    test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 94.35s
    历史 96 段：跑完 50、以已知红收尾 {}、新发现 46；根环转过一圈的 43 段；最高 txg 44；一版里最多 746 条分配记录

    看样子是 C554 乙（`mount.rs` 刚打进来的读缓存与重读）让「崩溃恢复抛弃最新那条根」那一步的可写挂载被判据 N-配置 拒掉，模型说该做成——这是推的，没往下查，归 C554 乙那一件或 74 号的负责人；这一件没修它。

`git apply --check`（主工作区，交回前现跑）退 0；补丁自己的 `git apply --stat`：

     crates/singlefs-core/src/mount.rs                  |  103 +++
     crates/singlefs-core/src/mounted_session.rs        |   30 +
     ...without_a_self_verified_system_configuration.rs |  620 ++++++++++++++++++++
     3 files changed, 732 insertions(+), 21 deletions(-)

主工作区 `git diff --stat -- crates litmus`（原样；补丁没打进主工作区，这里全是别的会话的改动，这一件的一行都不在里面）：

     crates/mutations.tsv                               |    70 +-
     crates/singlefs-core/src/admission.rs              |   405 +-
     crates/singlefs-core/src/allocator.rs              |   258 +-
     crates/singlefs-core/src/mount.rs                  |   440 +-
     crates/singlefs-core/src/transaction.rs            |   125 +-
     .../src/bin/e156_allocation_basis_counts.rs        |    10 +-
     .../src/bin/e158_root_choice_repair.rs             | 11401 ++++++++++++++++++-
     .../src/bin/first_transaction_on_device.rs         |   101 +-
     crates/singlefs-harness/src/history.rs             |    14 +
     crates/singlefs-harness/src/model_comparison.rs    |    12 +-
     .../admission_checkpoint_cost_per_device_paths.rs  |   539 +-
     ...hecker_narrow_invariants_and_abandoned_roots.rs |     8 +-
     crates/singlefs-harness/tests/common/mod.rs        |   311 +
     .../tests/publish_order_matches_litmus.rs          |    12 +-
     ...n_admission_raises_the_floor_before_refusing.rs |   276 +-
     ...inside_the_floor_raise_pushed_by_the_session.rs |    14 +-
     .../tests/second_transaction_step_five_reuse.rs    |     2 +-
     .../tests/second_transaction_step_one_overwrite.rs |    29 +-
     ...second_transaction_step_three_formatted_pool.rs |   504 +-
     ...d_transaction_supplement_two_unequal_devices.rs |    56 +-
     ...upplement_two_unreadable_abandoned_root_slot.rs |    44 +-
     21 files changed, 13922 insertions(+), 709 deletions(-)

开工后主工作区 `crates/mutations.tsv` 被别的会话改过（开工时的 sha256 对不上了），我的 8 行名字与它现在的表不重名（现核）；`mount.rs`、`mounted_session.rs`、`lib.rs`、`history.rs`、`common_admission/mod.rs` 与开工时逐字节相同（`sha256sum -c` 现核 OK）。

## 九、停下交主 agent 的设计问题

没有停在半路的条款：规格与用户定案把要做的都写了。下面几处是我在规格没写的地方选了一种，逐条列出，主 agent 定要不要改：

1. **会话那一拒用新成员**（第七节）：代价是 `history.rs` 两处、`common_admission/mod.rs` 一处要补分支，补上之前 `singlefs-harness` 整个编不过。
2. **挂着之后的入口报在已有成员里**：`DeviceTableDisagreement` 加第三个成员，装进 `CallerParametersDisagreeWithTheSelectedSystemConfiguration` / `CallerInputsDisagreeWithTheDisk` 的盘表清单，`MountError`、`RollbackError` 都没加成员（所以 `history.rs` 的 `MountError` 穷举、`model_comparison.rs`、`first_transaction_on_device.rs` 不受影响）。可写挂载照旧报 D18 已定项 11 点名的那个成员，同一判在两类入口上报成两个成员。
3. **判的次序**：可见在参数、盘数、本盘设备号之后判；前面有一项不对就只报前面那项。别的池的盘在挂着之后的入口被择系统配置先拒（对照测试钉着这一点），在会话里报不可见。
4. **会话每次调用核一次，不是循环里每次重发之前各核一次**（第三节「会话每次发布」那一行）。
5. **`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`（mkfs 同一个进程那条会话的构造）没加这一判**：它不写盘，之后每次发布由会话这一判拦；规格的入口清单里也没有它。
6. **会话只核可见，不核本盘设备号**：把盘 0 的一份拷贝当盘 1 交给会话，身份对得上、两槽都是本池自证过的，这一判放行（读代码推的，没跑）。挂着之后的五个入口有本盘设备号那一项（A1b Q4）；会话没有，规格也没要。这是 Z3-A 同一族的另一形，要不要补由主 agent 定。
7. **不经会话、直接调 `transaction::publish_overwrite` 这一族的地方**（用例、装置）不核：这一判只挂在会话与 `mount.rs` 那几个入口上。产品路径的用户改动都经会话（`mounted_session.rs` 模块文档），所以按规格没往发布路径里加。

## 十、没做什么

- 没走三方对抗；层 0 全量、QEMU、herd7 与 crates 变异整表（59 号）归 `crash-verifier`，层 0 快档在整轮门禁里；没提交，主工作区一个字节没动（补丁没打）。
- 没跑别的测试二进制（只跑了自己新加的那一个、`singlefs-core --lib`，和 74 号阶段自己跑的随机历史那一个）。按读代码推的、提交时最该看的几个：`entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write`（A1b，同一个函数）、`second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version`（可写挂载逐盘核第一支改成调共用读法，行为应当不变）、`second_transaction_supplement_three_fault_injection`（挂着之后的入口每次多读 4 槽、会话每次发布多读 2 槽，按「整池第 n 次读」摆的注入点会落到别的读上，结局分布可能挪动）、`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount`（有一条把盘 1 两槽都改坏再覆盖写，它走 `publish_overwrite_in_process`、不经会话，推的不受影响）。
- 实验装置 `e156_allocation_basis_counts`、`e158_root_choice_repair` 调卸载与回退，它们的产物会不会变没跑（归 87 号）；这一件不改装置。
- 74 号那 3 条红没查到底（第八节：基线上同样红，推的是 C554 乙那一格）。
- 92 号在副本里判不了（不是 git 仓），没有在主工作区跑（主工作区没打补丁，跑了判的也不是这一件）。
- 没改 `history.rs`、`common_admission/mod.rs`（第七节给了原文与 diff）；没改 kb（第三节写了 kb 那一句与代码的出入）。
- 读次数只量了 4 GiB 那一档、只量了覆盖写这一种会话发布；挂着之后五个入口每次多读几槽没单独量（按代码是每块盘 2 槽，`device_table_disagreeing_with` 已经读过的 2 槽没复用）。
- 第九节第 6 条（会话不核本盘设备号）只读了代码，没造用例。

## 十一、草稿目录清理

删了（交回前）：`/tmp/claude-1000/impl-z3a-yi/work`（补丁副本，17G）、`/tmp/claude-1000/impl-z3a-yi/baseline`（基线副本，15G）、`/tmp/claude-1000/impl-z3a-yi/base`（开工时 crates 与 litmus 的快照，8.1M）。
留着：`patch/`（交付物）、`report.md`、各份日志（`run-baseline-2.log`、`prove-red.out`、`prove-red-logs/`、`measure-before.log`、`measure-after.log`、`random-history-work.log`、`random-history-baseline.log`、`gate-*.log`、`clippy.log`、`build-all.log`、`core-lib.log`、`run-work-final.log`）、`arms-history.diff`、`arms-common-admission.diff`、量读次数的草稿用例 `measure_session_reads_over_sixty_publishes.rs`、`make-patch.sh`、`progress.md`：都是主 agent 要核的复跑材料或交付物，一共几 MB。这些都没进 `research/results/`：我的写范围不含 `research/`，入不入库由主 agent 定。
