# 实审 A1b 实现员报告（A1 交回的 Q1–Q5；Q6、Q7 照 A1 的做法认）

时刻：交回于 2026-09-26 UTC（JST 2026-09-27）。规格 `/tmp/claude-1000/impl-rev-a1b/spec.md`。

## 结论一览

| Q | 用例先红 | 改了什么 | 状态 |
|---|---|---|---|
| Q1 | 红（会话报 `FloorRaiseFailedWhilePushingForSpace`；挂载返回 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite{PlacementRefused}`） | `mount.rs` `FloorRaiseStop` 加成员 `FloorRaiseRefusedForSpace { refusal: Box<MountError> }`（mount.rs:1649）；`floor_raise_refused_for_space`（mount.rs:1655，穷举 `MountError`）判抬 F 的错是不是预演被拒 / 真发时落盘之前被拒、且 `cause` 是 `refusal_is_short_of_space` 那两种；`push_one_floor_raise_within_the_admission_budget` 把这一类换成 `Stopped`，别的错照旧 `Err` | 做完。会话报 `NoSpaceAfterRaisingTheFloor`，挂载走 C565 那一格（`StillShortAfterTheFloorRaises`）；random_history 那条见「交回前的验证」 |
| Q2、Q3 | 红（5 条：会话多交一块盘照发、抬 F 换 io_min 照抬、卸载重复身份照写、准入抬 F 多一块盘交回「F 已在上限」、推一串换 fsid 交回「预算用完」） | 会话：`MountOutput` 新字段 `parameters_and_device_table: ParametersAndDeviceTableOfTheMount`（mount.rs:400 类型、:506 字段；字段私有，只由可写挂载与 `of_a_pool_on_disk` 读盘造）；`MountedSession` 持着它（mounted_session.rs:39），`publish_user_change` 不再收参数（mounted_session.rs:200），交进来的盘表与它逐项比、不同就在任何读写之前拒成新成员 `UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`（mounted_session.rs:68、212）。自由函数（`raise_rollback_floor`、`raise_rollback_floor_to_the_admission_ceiling`、`push_one_floor_raise_within_the_admission_budget`、`unmount`）签名不动（e156、e158、`on_device_modes.rs` 在调），照可写挂载同一套核：`caller_inputs_agreeing_with_the_disk`（mount.rs:2833：重复身份 → 择系统配置 → 参数逐项比 + 盘表比 devs 与本盘设备号），写入口只按盘上那一份建 | 做完，**回退那一处（`roll_back_by_a_forward_publish`）没做**：卡在 `model_comparison.rs`（见「卡住的与停下交主 agent 的」） |
| Q4 | 红（盘表多一块盘 1 的拷贝标成盘 2：改前报的是写行预演 `ReleaseTargetNotAllocated`；盘 1 换成盘 0 的拷贝：改前挂载做成、取号 2） | `CallerParametersDisagreeWithTheSelectedSystemConfiguration` 加字段 `disagreeing_device_table: Vec<DeviceTableDisagreement>`（mount.rs:250）；新枚举 `DeviceTableDisagreement`（mount.rs:278，两成员：盘数与 devs 不同、本盘设备号与盘表身份不同）；`device_table_disagreeing_with`（mount.rs:2682）读每块盘两个自证过的系统配置槽 | 做完。没加新 `MountError` 成员（加了要动 `model_comparison.rs` 与 `first_transaction_on_device.rs` 的穷举 match），两处都是 `{ .. }` 模式，加字段不碰它们 |
| Q5 | 没写 | — | **没做**：要新 `MountError` 成员，卡在 `model_comparison.rs`（B3b 在改） |
| Q6、Q7 | — | 照 A1 的做法认，没动 | — |

推翻条件：59 号复跑这 11 行有一行不红；或主 agent 集成之后 `second_transaction_supplement_three_random_history` 的 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device` 在主工作区仍红（我在副本与主工作区的结果见「交回前的验证」）。

## 写过的文件（这一轮我改的，别的会话同时在改 `crates/`，`git diff --stat` 分不出谁改的）

新建（3）：
- `crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs`（Q1，2 条）
- `crates/singlefs-harness/tests/writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration.rs`（Q4，2 条）
- `crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write.rs`（Q2、Q3，6 条）

改：
- `crates/singlefs-core/src/mount.rs`（相对开工快照的 diff 全是我的：24 段，+346 −44）
- `crates/singlefs-core/src/mounted_session.rs`（会话持池、新成员、`publish_user_change` 去掉参数、文档）
- `crates/singlefs-harness/src/history.rs`（`WritableSession` 持池；起点那条会话读盘造池，读错交回新的起点步 `StartingPointStep::ReadThePoolForTheSession`；覆盖写不再交参数；两处穷举 match 补新成员）
- `crates/singlefs-harness/tests/common_admission/mod.rs`（mkfs 同一个进程那条会话读盘造池；`publish_user_change` 新签名；match 补成员）
- `crates/singlefs-harness/tests/writable_mount_refuses_caller_parameters_disagreeing_with_the_system_configuration_on_disk.rs`（A1 那两条跟着新字段解构，多钉一句「盘表那张清单是空的」）
- `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`：`DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT` 3 → 4 与两句文档（Q4 在择系统配置之后多读每块盘两槽一次，判定那一读顺延一位）。⚠️ 这份不在派发列的「调用挂着之后入口的用例」之内，是 Q4 多读一次直接带红的；它的 mtime 是 2026-09-26 03:23 UTC，开工时没人在改，我只动了这三行，主 agent 核一下是不是 B3a-2 的「几份测试」之一
- `crates/mutations.tsv`：末尾追加 11 行（第 934–944 行），用 `research/scripts/insert-row.py` 逐行插，名字见证红表
- `crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/lib.rs` 没动（新类型都在 `mount` / `mounted_session` 模块里）

`git diff --stat -- crates litmus` 原样在 `/tmp/claude-1000/impl-rev-a1b/scratch/git-diff-stat.txt`（88 行，末行 `87 files changed, 26419 insertions(+), 11040 deletions(-)`，含别的会话的改动）。

## 证红（改坏哪一行 → 哪条断言红）

两步都做了：

1. **改之前先看新用例红**（开工快照的副本 `work/`，只叠上新测试文件）：九条全红，日志 `scratch/prefix-*.log`。Q1 会话那条红在 `a_floor_raise_refused_for_space_counts_as_short_of_space.rs:55`（实际 `FloorRaiseFailedWhilePushingForSpace(… RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite{refused 2/2, PlacementRefused{AllocationTreeNodeBelowTheRoot…, NoFreeSlotOnAnyDevice}})`）；挂载那条红在 `:91`（挂载返回 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite{1/3, PlacementRefused{TreeTable}}`）。Q4 盘 1 换成盘 0 的拷贝：改前挂载做成、取号 2；多交一块拷贝标成盘 2：改前拒成 `RowPublishAdmissionRefusedBeforeAcquisition{ReleaseTargetNotAllocated{AccountingTree, 盘 2}}`。Q2、Q3：会话多交一块盘 0 → `Ok(UserChangePublished)`；抬 F 换 io_min → 做成、先写系统配置两次；卸载多交一块盘 1 → 改前拒成 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite{MappingEntryLocationsOnTheSameDevice}`（没被重复身份那一判拦下）；准入抬 F 多一块 → `Ok(FloorAlreadyAtTheCeiling)`；推一串换 fsid → `Ok(Stopped(PublishesPerAdmissionWouldBeExceeded{7, 2}))`。
2. **变异证红**：`bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a1b/work --memory 8G singlefs-harness <11 个名>`，先各参数组跑基线（没红），11 条全抓到（`scratch/prove-red-1.txt`，末行 `✓ 点名 11 条：跑了 11 条，跳过 0 条，跑的都抓到了`）。prove-red 按行里的过滤只跑点名那一条，同一二进制里别的用例这一步没跑。每条新测试至少一行，没有留给 59 号只追加不证的行。

| mutations.tsv 行 | 改坏 | 红的测试 → 断言 |
|---|---|---|
| 934 Q1 会话 | `mount.rs` `Err(refusal) if floor_raise_refused_for_space(&refusal) =>` 加 `false &&` | `a_floor_raise_pushed_by_the_session_whose_own_publishes_find_no_slot_reports_no_space` 在 `…counts_as_short_of_space.rs:55` 的 let-else（报成 `FloorRaiseFailedWhilePushingForSpace`） |
| 935 Q1 挂载 | `floor_raise_refused_for_space` 里预演被拒那一臂 → `false && refusal_is_short_of_space(cause)` | `a_mount_whose_floor_raise_after_the_row_publish_finds_no_slot_is_still_made_and_still_short` 在 `:115` 的 `unwrap_or_else`（挂载返回 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite{1/3, TreeTable}`） |
| 936 Q4 devs | `if devices_handed_in != device_count_in_the_system_configuration {` 加 `false &&` | `mount_with_a_third_device_carrying_a_copy_of_device_one_is_refused_before_any_write` 在 `…disagreeing_with_the_system_configuration.rs:94` 的 `assert_eq!`（清单少了盘数那一项） |
| 937 Q4 本盘设备号 | `.filter(… own_device_number_on_disk != identity_handed_in)` 加 `false &&` | `mount_with_device_one_carrying_a_copy_of_device_zero_is_refused_before_any_write` 在 `:48` 的 `expect_err`（挂载做成） |
| 938 Q4 只看参数清单 | `if disagreeing_fields.is_empty() && disagreeing_device_table.is_empty() {` → `if disagreeing_fields.is_empty() {` | 同上一条，`:48` |
| 939 Q3 会话盘表 | `mounted_session.rs` `if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {` 加 `false &&` | `the_session_refuses_a_device_table_other_than_the_one_of_its_mount_before_any_write` 在 `entries_after_….rs:174` 的 let-else（`Ok(UserChangePublished)`） |
| 940 Q2 读盘造池不核 | `of_a_pool_on_disk` 的 `if !disagreeing_device_table.is_empty() {` 加 `false &&` | `the_pool_of_a_session_without_a_mount_is_read_from_the_disk_and_a_disagreeing_device_table_is_refused` 在 `:216` 的 `expect_err` |
| 941 Q2 抬 F 那一串按调用方参数建 | `raise_the_floor_through` 里核的那四行 → `choose_system_configuration` + `parameters.clone()` | `raising_the_floor_with_other_parameters_is_refused_before_any_write` 在 `:249` 的 `expect_err`（做成、先写系统配置两次） |
| 942 Q3 不查重复身份 | `caller_inputs_agreeing_with_the_disk` 里去掉 `refuse_device_identities_handed_in_more_than_once(devices)?;` | `unmounting_with_a_device_identity_handed_in_twice_is_refused_before_any_write` 在 `:295` 的 let-else（报成盘表那一类，不是重复身份） |
| 943 Q2/Q4 准入抬 F 不核 | 准入抬 F 入口的核 → 不核的一份（盘上系统配置 + 调用方参数） | `raising_the_floor_to_the_admission_ceiling_with_a_third_device_is_refused_before_any_write` 在 `:344` 的 let-else（`Ok(FloorAlreadyAtTheCeiling)`） |
| 944 Q2 推一串不核 | 推一串入口的核 → 同上 | `pushing_a_floor_raise_with_another_filesystem_identifier_is_refused_before_the_budget_is_judged` 在 `:388`（`Ok(Stopped(PublishesPerAdmissionWouldBeExceeded{7,2}))`） |

各行的原样日志 `scratch/prove-red-logs/001.log`–`011.log`（草稿目录里，交回前我删 `work/` 副本时这个目录留着）。

没法写成「改回去它就红」、也没有用例走到的一格：`floor_raise_refused_for_space` 的 `RaiseFloorSequencePublishFailed` 那一臂（预演与真发分叉、真发时第 n 次在落盘之前被落点拒）。它只在「真发时读盘核出校验和对不上、那一份被隔离」的时候走得到，今天没有一条用例造得出这一格，把那一臂改成 `false` 这一轮的用例都不红；我照 D16 已定项 1「准入」那一行的字面把它归成空间不够，没有给它加变异行。

## 卡住的与停下交主 agent 的

- **Q5 没做（卡在 `crates/singlefs-harness/src/model_comparison.rs`，B3b 在改）**。形态照收口表第 58 行 `MountError::Publish` 那一格，要一个新 `MountError` 成员，例如 `FloorRaiseFailedAfterTheMountsPublishes(Box<…>)`，带 `cause: MountError`（抬 F 的错原样）、`writes_of_persisted_publishes: Vec<WritesByStructureKind>`（写行一次、暖机每次、之前推成的每串的先写系统配置与各次空发布）与 `floor_raises: Vec<RaisedFloor>`；`establish_instance` 里 `push_floor_raises_after_the_row_publish(...)?` 那一处改成 map 成它。`MountError` 在三处被穷举：`model_comparison.rs` 的 `refusal_reason_of_mount_error`（:444 起）、`root_ring_slot_still_bad_after_one_reread_of_mount_error`、`reported_ceiling_of_mount_error`，另有 `crates/singlefs-harness/src/bin/first_transaction_on_device.rs` 两处（:822、:1231 附近，不在我的清单里）与 `history.rs` 的 `mount_error_member`。B3b 交回之后再派这一件，或者主 agent 定「挂载那一处抬 F 报错就不带写账」。
- **Q2、Q3 里的管理员回退（`roll_back_by_a_forward_publish`）没做**：它的错是 `RollbackError`，在任何写之前拒要加一个成员（重复身份、参数或盘表与盘上系统配置不一致），而 `model_comparison.rs` 的 `refusal_reason_of_rollback_error`（:411）穷举 `RollbackError`，卡在同一份文件上。今天回退仍拿调用方的参数与盘表建写入口（mount.rs `roll_back_by_a_forward_publish` 末尾 `PoolWriter::new(parameters, …)`）：换一份参数回退，回退那次发布末尾的轮换会改写盘上系统不可变配置。签名不必改（e156、e158 在调），做法照抬 F 那一串：函数开头调 `caller_inputs_agreeing_with_the_disk`，拒成新 `RollbackError` 成员，写入口用 `parameters_of_the_pool`。
- **会话只比盘表的设备身份，不读盘**：`publish_user_change` 核的是交进来的身份（按次序）与挂载时核过的那一份逐项相同（mounted_session.rs:212），不在每次发布之前再读系统配置核本盘设备号——同一个身份底下换了一块盘（盘 1 的位置交一块盘 0 的拷贝）这一格会话看不出来。挂载那一刻核过（Q4），抬 F 与卸载每次都读盘核。要不要让会话每次发布前也读两槽核一遍（每次发布每块盘多两次读，故障注入按第几次读摆的用例会跟着挪位），规格没写，我没做。
- **`CallerParametersDisagreeWithTheSelectedSystemConfiguration` 这个名字现在也管盘表**：按调用方要做的决定分，参数错与盘表错是同一个决定（换对的池、参数或盘再来），所以并在一个成员里、加了字段；名字里的「CallerParameters」读作「调用方交进来的参数与盘表」，文档写明了。改名（例如 `CallerInputsDisagreeWith…`）要动 `model_comparison.rs` 与 `first_transaction_on_device.rs` 的 match，没改。
- **Q4 让可写挂载、抬 F、卸载多读每块盘两个系统配置槽**（`device_table_disagreeing_with` 调 `recovery::verified_system_configuration_slots`）：`second_transaction_step_three_formatted_pool.rs` 那条按「每块盘第几次读槽 0」注入的用例判定那一读从第 3 次挪到第 4 次，我改了常量与文档。别的按读的序号摆注入的用例（随机故障注入按整池第 n 次调用）这一轮跑过都绿，但它们抽到的注入点跟着挪了。
- **起点那条会话读盘造池**：`history.rs` 起点（mkfs 同一个进程）那条会话在取号之前多一步 `ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`（每块盘读两个系统配置槽两遍：择系统配置一遍、核本盘设备号一遍，推的，没数过设备一层的读计数），读错交回新的起点步 `StartingPointStep::ReadThePoolForTheSession`（`finished_filesystem_is_on_the_devices` 判 true：mkfs 已做完）。`second_transaction_supplement_three_fault_injection.rs` 逐个列四步的那条用例只注入写，没碰到这一步；它不要求新的一步也出现过。
- 条款：D16（发布语义） 已定项 1「准入」那一行已写「落点取不到也先推抬 F 再判……做满仍不够才报 ENOSPC」，Q1 的做法与字面相符，不要补句；D2（RAID 条带策略） 已定项 13 / C565 那一格的例外只管空间不够——代码现在也只让空间不够那三种 `FloorRaiseStop` 走 C565。Q2–Q4 管的是 D22（单元原子性怎么合成） 已定项 26 第一档（不可变段 mkfs 之后不可改）在挂着之后入口上的落实，条款里「挂着之后的入口拿哪一份参数」「盘表比哪几项」没有逐字的句子，要不要补一句归主 agent 与书记员。

## 受影响的层 0 流与崩溃枚举用例

没改 checker（`crates/singlefs-checker/src/` 一处没动），这一节按定义可以不写；写一句：这一轮的改动没加写、没加屏障（多出来的只有读），录制流的写与段序列不变，层 0 各流的钉值不受影响（推的，没跑）。名字带 layer0 的只有 `second_transaction_step_zero_layer0.rs` 调了抬 F 与卸载，签名没变、编得过（`cargo build --all-targets` 覆盖了它）。

## 交回前的验证（末尾原样）

跑的地方有两处：`/tmp/claude-1000/impl-rev-a1b/work/`（开工时 16:34 UTC 的整仓快照 + 这一轮我改的 9 份文件，自己的 target），与主工作区。主工作区这段时间里别的会话一直在改（checker 的 `walk.rs`、`image.rs` 17:05 UTC 改过；`admission_checkpoint_cost_per_device_paths.rs` 17:17 UTC 新建、未跟踪；`e158_root_choice_repair.rs` 在改），所以两处分开报。收尾时核过：这 9 份文件在副本与主工作区逐字节相同（`cmp`）。

**测试二进制**（副本，`run-with-memory-cap.sh 8G`、`capped.sh 4`；名字都不含 layer0）：

- 新的三个：`a_floor_raise_refused_for_space_counts_as_short_of_space` `test result: ok. 2 passed; 0 failed`；`writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration` `test result: ok. 2 passed; 0 failed`；`entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write` `test result: ok. 6 passed; 0 failed`（主工作区上同样三行 ok，外加 `writable_mount_refuses_caller_parameters_disagreeing_with_the_system_configuration_on_disk` `test result: ok. 2 passed; 0 failed`）。
- 调到改动的其余 19 个（`scratch/affected-summary.txt`、`affected-summary-2.txt`、`run2.log`，逐个一行）：`a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is` 2 passed；`writable_mount_refuses_a_device_identity_handed_in_twice` 1 passed；`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount` 11 passed；`second_transaction_admission_raises_the_floor_before_refusing` 10 passed；`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session` 0 passed 1 ignored；`system_configuration_slot_is_overwritten_only_after_a_barrier` 3 passed；`second_transaction_step_three_formatted_pool` 13 passed（改常量之后；改之前 12 passed 1 failed，见上一节）；`second_transaction_step_four_rollback` 15 passed；`second_transaction_step_five_reuse` 14 passed；`second_transaction_supplement_two_commit_generated_fallback` 7 passed；`…release_checksum_quarantine` 14 passed；`…presumed_clause_checks` 3 passed；`…fsync_drop_and_devices_without_the_selected_version` 17 passed；`record_checker_judges_absence_by_the_persisted_set` 6 passed 1 ignored；`checker_known_bad_images` 36 passed；`instance_acquisition` 6 passed；`second_transaction_supplement_three_fault_injection` 13 passed 1 ignored；`…bad_disk_input` 9 passed 1 ignored；`…admission_formula` 3 passed；`…root_ring_turn_in_one_mount` 3 passed；`crash_injection_writable_mount_after_the_crash` 1 passed；release 下 `second_transaction_supplement_three_random_history` `test result: ok. 24 passed; 0 failed; 2 ignored`——Q1 那条 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device ... ok`。
- 红的一个，不是这一轮带出来的：`second_transaction_supplement_three_crash_injection` `test result: FAILED. 7 passed; 1 failed; 1 ignored`，红的是快档 `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`（种子 …115 第 56 段 `RecordCheck claimed_state_missing_unit`、种子 …136 第 91 段 `CheckerViolations I-3.1`，与 A1 报告里那两粒相同）。开工快照 `base/`（没有我的改动）上同一条单跑：`test result: FAILED. 0 passed; 1 failed; … 8 filtered out`，新发现逐字相同（`scratch/base-crash-injection-fast.log`）。
- 主工作区上 `second_transaction_step_three_formatted_pool` `test result: FAILED. 10 passed; 3 failed`：三条红在 checker 的判定上（`写行那次发布之后：I-3.9 该报不适用：Holds`、`卸载之后：I-7.4 判红`、`mkfs 之后：I-MAPPING-KEY 要真被评估过且成立 left: NotApplicable`），同一份测试在副本（快照的 checker + 我的改动）上 13 passed：推的是别的会话 17:05 改的 checker 带出来的，我没追。

**门禁阶段**（登记给 implementation-writer 的 7 道，主工作区跑）：

- 33 号 `exit=1`：`✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次`，列了 18 行（155、308、309、310、319、494、759、837、838、839、844、846、851、865、870、874、877、878），锚在 `crash_injection.rs`、`singlefs-checker/src/walk.rs`、`e158_root_choice_repair.rs` 上，没有一行是这一轮加的（934–944）或锚在我改的文件上；我另用脚本核过锚在我改的 6 份文件上的 137 行：`checked 137 rows, bad 0`。
- 53 号 `exit=0`：`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
- 92 号 `exit=0`（末行：`第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）：.claude/kb/layout/02-second-txn.md`）
- 94 号 `exit=0`：`✓ checker 与实现只共享常量模块 singlefs-format（…checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 singlefs_core …`
- 93 号 `exit=0`：`✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（…扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处…）`
- 89 号 `exit=77`（本次未跑）：`⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`
- 74 号 `exit=1`：`test result: FAILED. 22 passed; 2 failed`，红的两条是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与快档 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，签名都是 `CheckerViolations { invariants: ["I-7.4"] }`（快档 `新发现 28`）。Q1 那条在主工作区这一次是 ok（没在失败清单里）。同一个二进制在副本上 release 24 passed：I-7.4 这一批跟着主工作区里改了的 checker 走，不是这一轮的改动；74 号要等 B2 的 checker 落定再判。

**fmt / clippy / build**：

- `cargo fmt --check`：副本 `work fmt exit=0`；主工作区 `exit=1`，67 处全在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（E158 执行员那份）。
- clippy（`check.sh` 那一套 lint：`-D warnings` 加七条编码纪律）：副本上 `-p singlefs-core -p singlefs-checker -p singlefs-format --all-targets` `exit=0`；`-p singlefs-harness --lib --test '*'` `exit=0`（`Finished`）；`--all-targets` 整跑 `exit=101`，错只在两个 bin：`e156_allocation_basis_counts.rs:3312:13`、`:3880:54`、`:3882:54`（`shadow_unrelated`）与 `e158_root_choice_repair.rs` 4 处——e156 与开工快照逐字节相同（`cmp`），不是这一轮改的。
- `cargo build --offline --all-targets`：副本 `exit=0`、0 条 warning（`Finished dev profile … in 18.71s`）；主工作区 `exit=101`，错只在别的会话 17:17 UTC 新建、未跟踪的 `crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`（4 处 E0425 / E0061 / E0308）。

## 仓副本与编译目录

- 删了 `/tmp/claude-1000/impl-rev-a1b/work`（14G，其中自己的 `target` 14G）：开工快照 + 我这 9 份文件，编译、证红与受影响二进制都在它上面跑。
- 删了 `/tmp/claude-1000/impl-rev-a1b/base`（1.6G，其中 `target` 1.5G）：开工时的整仓快照，拿来比 diff 与复核 crash_injection 快档的基线。
- 留着 `/tmp/claude-1000/impl-rev-a1b/scratch/`（54M）：日志与我的辅助脚本（`sync.sh`、`run-affected.sh`、`make_rows.py`、`append_rows.sh`、`anchor_check.py`、`rows.tsv`、`prove-red-logs/`），主 agent 要核的复跑材料，不是仓副本、不是编译目录。
- 主工作区里我起过的 cargo 用的是项目自己的 `target`，那是主 agent 的，没动。

## 没做什么

- Q5 与回退那一处（卡在 `model_comparison.rs`，见上）；会话每次发布前读盘核本盘设备号（规格没写）。
- 没走三方对抗；层 0（`second_transaction_step_zero_layer0` 等）、QEMU、herd7、crates 变异整表（59 号）、`check.sh` 与 `gate.sh` 整轮都没跑，归 `crash-verifier` 与提交时那一次。
- 登记给我的 74 号在主工作区红着（I-7.4，别的会话改的 checker 带出来的，推的，没追到是哪一处）；33 号红着（18 行锚点在别人的文件上）；没修。
- 没写 kb（条款要不要补句见「卡住的与停下交主 agent 的」最后一条）；没提交。
- 主工作区上 `cargo build --all-targets`、`cargo fmt --check`、clippy 整跑都因为别的会话的文件红着，我这 9 份文件在副本上三样都过；主工作区集成之后要再跑一遍。
