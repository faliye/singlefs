# 实审 A1 实现员报告（代码审阅第 17、18、19、23 条）

日期：2026-09-27 交回。

## 结论一览

| 条 | 用例先红 | 改了什么 | 状态 |
|---|---|---|---|
| 17 | 红（改前可写挂载照常做成、把 io_min 改写进盘） | `mount.rs` 选完系统配置就逐项比调用方参数，不一致拒成 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields }`（新枚举 `MakeFilesystemParameterField`，两边整个解构）；写入口改用盘上那一份建的参数（`parameters_of_the_pool`，mount.rs:3003、3068） | 做完；射程只到可写挂载，见设计问题 Q2 |
| 18 | 红（改前没有拒：`[(盘0,A),(盘1,B),(盘0,C)]` 在 `admission.rs:288` 的 `assert_eq!`「准入读数里每块盘只一条」panic，不是审阅说的「C 收到系统配置写」） | 设备数那一判之后、读盘之前拒成 `MountError::DeviceIdentitiesHandedInMoreThanOnce { repeated: Vec<RepeatedDeviceIdentity> }`（mount.rs:2444） | 做完 |
| 19 | 红（录制流里同一段有同一块盘的两次系统配置槽写） | `transaction.rs` 取号写之前（:684）、抬 F 先写系统配置之前（:542）、取号回卷写之前（:438）各一道池屏障 | 做完；段序列变了，层 0 钉值照闭式改了、没跑；kb 条款与登记句要补，见下 |
| 23 | 红（改前挂载交回「推满仍不够」做成；会话报 `NoSpaceAfterRaisingTheFloor`） | 删 `FloorRaiseStop::FloorRaiseFailed`；`push_one_floor_raise_within_the_admission_budget` 改成 `Result<FloorRaisePushedWithinTheAdmissionBudget, MountError>`；挂载那一处抬 F 的错原样 `Err`；会话新成员 `UserChangeRefused::FloorRaiseFailedWhilePushingForSpace` | **按规格做完，但与条款对不上一格，停下交主 agent（Q1）**：`second_transaction_supplement_three_random_history` 的 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device` 因此红（门禁 74 号同红） |

推翻条件：Q1 那一格若主 agent 定「抬 F 自己的空发布落点取不到也算推满仍不够」，第 23 条的改法要收窄一格（见 Q1），random_history 回绿；其余三条的用例在它们各自的变异下都红（下文逐条），改法被撤回时 59 号复跑会响。

## 写过的文件

新建（5）：
- `crates/singlefs-harness/tests/writable_mount_refuses_caller_parameters_disagreeing_with_the_system_configuration_on_disk.rs`（第 17 条，2 条用例）
- `crates/singlefs-harness/tests/writable_mount_refuses_a_device_identity_handed_in_twice.rs`（第 18 条，1 条）
- `crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs`（第 19 条，3 条：取号、抬 F、取号回卷）
- `crates/singlefs-harness/tests/a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs`（第 23 条，2 条：挂载那一处、会话那一处）
- 草稿目录里的证红脚本与日志（`/tmp/claude-1000/impl-rev-a1/`，不入库）

改（主工作区，只动我这几处）：
- `crates/singlefs-core/src/mount.rs`：两个新 `MountError` 成员、`RepeatedDeviceIdentity`、`MakeFilesystemParameterField`、`FloorRaisePushedWithinTheAdmissionBudget`、两个判定函数、`FloorRaiseStop` 去掉 `FloorRaiseFailed`、`push_floor_raises_after_the_row_publish` 改返回 `Result`、`mount_writable` 文档
- `crates/singlefs-core/src/transaction.rs`：三道屏障与相关文档注释
- `crates/singlefs-core/src/mounted_session.rs`：新成员与 `FloorRaiseFailedWhilePushingForSpace` 结构
- `crates/singlefs-harness/src/history.rs`、`src/model_comparison.rs`、`src/bin/first_transaction_on_device.rs`：穷举 match 补新成员（history.rs 另改一句文档）
- `crates/singlefs-harness/src/fault_injection.rs`：「第一道屏障那一刻的计数」改成「收到过写之后的第一道屏障」（字段、方法、单测改名 `counts_when_the_first_barrier_after_a_write_arrived*`）——取号写之前那道屏障前面没有写，不改的话真设备二进制的挂载窗口起点错一道；`first_transaction_on_device.rs` 跟着改调用与文档
- 测试钉值：`tests/instance_acquisition.rs`（改 2 条、加 2 条）、`tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`（3 条头几步序列）、`tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`（新返回类型）、`tests/common_admission/mod.rs`（新成员）、`tests/second_transaction_supplement_three_crash_injection.rs`（写死历史的崩溃点下限 100 → 95）、`tests/second_transaction_step_zero_layer0.rs`、`tests/second_transaction_step_three_acquisition_barrier_layer0.rs`（层 0，见第四节）
- `crates/singlefs-core/src/lib.rs` 没动（新类型都在 `mount` / `mounted_session` 模块里，公开路径照旧）
- `crates/mutations.tsv`：末尾追加 9 行（第 880–888 行，名字见第三节）；按主 agent 消息与 33 号改了 6 行的原文（692、714、93、170、301、690），都用 `replace-once.py` 定点改

`git diff --stat -- crates litmus` 原样在 `/tmp/claude-1000/impl-rev-a1/git-diff-stat.txt`（86 行，含别的会话的改动，末行 `85 files changed, 22045 insertions(+), 10581 deletions(-)`）；别的会话同时在改 `crates/`，这份 stat 分不出谁改的，以上面的清单为准。

## 证红（改坏哪一行 → 哪条断言红）

做法：先在草稿目录建副本 `proof/`（`rsync -a --exclude target --exclude .git`、自己的 target），不改动跑一遍基线（七个测试二进制全绿，基线红集为空，记录在 `proof-results.txt` 前 7 行），再逐条改坏、跑点名的测试、从 `pristine/` 拷回并 `touch`。
⚠️ 副本前两次从主工作区整份拷，都编译不过：别的会话在 `crates/singlefs-harness/src/model.rs` / `model_comparison.rs` / `fault_injection.rs` 上改到一半（`has_file`、`judge_file_content` 那一批）。第三次改成「本轮开工时的基线副本 + 我这一轮的文件叠上去」：`model_comparison.rs` 用基线版补我那三处 match，`fault_injection.rs` 用基线版补我那两处改动（改名 + `writes > 0` 条件）——所以第 170 行那条的证红跑的是基线版的单测体（改名后的函数名，期望消息是「收到过屏障」），主工作区里那条单测多了「写之前一道屏障不拍快照」一段。两次编译失败的输出在 `proof-results-attempt{1,2}-compile-errors.txt`。

每条都证了（按测试算，每条新测试至少一条；不留给 59 号的）：

| mutations.tsv 行 | 改坏 | 红的测试 → 断言 |
|---|---|---|
| 880 第17条：不逐项比 | `mount.rs` `.filter(\|(_, is_equal_on_both_sides)\| !is_equal_on_both_sides)` → `false && …` | `mount_with_a_different_minimum_input_output_size_is_refused_before_any_write_and_names_that_field` 在 `…on_disk.rs:26` 的 `expect_err`（挂载做成了、取号 2） |
| 881 第17条：漏比 fsid | `caller_filesystem_identifier == disk_filesystem_identifier,` → `true,` | `mount_with_another_filesystem_identifier_and_region_devices_…` 在 `:81` 的字段清单 `assert_eq!` |
| 882 第18条：重复身份不拒 | `if repeated.is_empty() {` → `if true {` | `mount_with_device_zero_handed_in_again_as_a_third_device_is_refused_before_any_write`：红在被测代码 `crates/singlefs-core/src/admission.rs:288` 的 `assert_eq!`（「准入读数里每块盘只一条」 left 2 right 3，是 `assert_eq!` 不是 `debug_assert`，release 下同样先红在这里），用例自己的断言没走到 |
| 883 第19条：取号写前无屏障 | `transaction.rs` 取号那道 `pool.perform(CommitStep::Barrier)` → `Ok::<(), BlockDeviceError>(())` | `acquisition_after_a_publish_of_the_previous_process_overwrites_the_older_slot_only_behind_a_barrier` 在 `…only_after_a_barrier.rs:97` |
| 884 第19条：抬 F 先写前无屏障 | 抬 F 那道同样换成 `Ok(())` | `raising_the_floor_overwrites_the_older_system_configuration_slot_only_behind_a_barrier` 在 `:125` |
| 885 第19条：回卷写前无屏障 | `self.perform(CommitStep::Barrier)` → `Ok(())` | `rolling_back_a_failed_acquisition_overwrites_the_older_slot_only_behind_a_barrier` 在 `:173` |
| 886 第19条：回卷前屏障报错照写 | `return AcquisitionFailed { … RollbackFailed(barrier_error) }` → `let _ = barrier_error;` | `barrier_failing_again_before_the_rollback_writes_leaves_the_new_number_and_it_is_never_handed_out_again` 在 `instance_acquisition.rs:216`（交回 `RolledBack`） |
| 887 第23条：挂载处当推满 | `)? {` → `.unwrap_or(Stopped(FloorAlreadyAtTheCeiling{0,0}))` | `a_block_device_error_in_the_floor_raise_pushed_by_the_mount_fails_the_mount_instead_of_reporting_still_short` 在 `…handed_up_as_is.rs:92` 的 `expect_err`（挂载做成了） |
| 888 第23条：会话当空间不够 | 会话 `Err(cause)` 臂换回 `NoSpaceAfterRaisingTheFloor` | `a_block_device_error_in_the_floor_raise_pushed_by_the_session_is_not_reported_as_no_space` 在 `:125` |
| 692（修锚）实二八 | 原文改成含 `refuse_device_identities_handed_in_more_than_once` 的三行 | `mount_writable_with_only_device_zero_is_refused` 在 `fsync_drop….rs:1310`（盘 0 被读了 2 次） |
| 714（修锚）实二 | 原文前面加上 `devices_carrying_the_raised_floor.push(…);\n    }\n` 两行定位到写之后那道 | `raising_the_floor_writes_the_new_floor_into_every_system_configuration_behind_a_barrier_before_the_first_root` 在 `rollback_floor….rs:242`（第 4 步是单元写不是屏障） |
| 93（修锚）增补 2 第 30 行 | 原文换成新字段名、多一层缩进 | `second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches`（`--bin first_transaction_on_device`）在 `first_transaction_on_device.rs:1848`（挂载窗口 `matches=false`） |
| 170（修锚）增补 3 第 4 件 | 同上；参数与测试名跟着单测改名 | `the_counts_when_the_first_barrier_after_a_write_arrived_are_frozen_at_that_moment` 在 `fault_injection.rs:3057`（副本里基线版单测体，见上） |
| 301（修锚）C378 | 原文与替换文里 `establish_instance(\n        parameters,` → `&parameters_of_the_pool,` | `write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it` 在 `…fault_injection.rs:792` |
| 690（修锚）实二八 | 原文换成 `admit…?;\n    refuse_device_identities_handed_in_more_than_once(devices)?;` | `mount_writable_with_only_device_zero_is_refused` 在 `fsync_drop….rs:1294`（报 I-1.1） |

每条同时红的只有点名的那一条（每次跑都用点名过滤）；不改的基线四行在 `proof-results2-baseline.txt`（四条全过）。结果原文：`/tmp/claude-1000/impl-rev-a1/proof-results.txt`、`proof-results2.txt`。

没法写成「改回去它就红」的一处：第 17 条「写入口按盘上那一份建」（`&parameters_of_the_pool`）本身——比对拒掉不一致之后两份逐字段相等，换回调用方的那一份行为不变，是等价变异，不进表。

## 第 19 条改了段序列：钉值怎么算、哪几条层 0 要主 agent 集成时跑

规则：屏障只切段、不添写。取号写之前那道屏障在首次挂载（mkfs 同一个进程、或只做过 mkfs 的池重开）上紧跟 mkfs 末尾那道，录制器并掉（`crates/singlefs-harness/src/lib.rs` 的 `push` 里相邻屏障只记一道），第一条流、只做过 mkfs 的池那条流、E142 那几行都不变。进程重开之后的取号、抬 F、正常卸载三处，把「上一次轮换 2 写 + 这一步 2 写」的 4 写一段拆成 2、2。
闭式 = 1 + Σ(2^|段| − 1)（`crash.rs` 的 `closed_form_state_count`）：每拆一处 15 → 3 + 3，少 9；甲二快档那一段同样 15 → 3 + 3。

| 用例（名字含 layer0，我没跑） | 改前 | 改后 | 算式 |
|---|---|---|---|
| `second_transaction_step_zero_layer0` 全量常量 `FULL_STATES_THROUGH_THE_UNMOUNT`（门禁 54 号 `crash-case:layer0-second-stream`） | 61 段、6649413746 | 64 段、6649413719 | 三处各 −9 |
| 同文件 `FULL_STATES_BY_PUBLISH_THROUGH_THE_UNMOUNT` | txg5=262162、txg15=65554、txg18=65554 | 262153、65545、65545 | 各 −9，合计 6649413719 |
| 同文件甲二 `QUICK_TIER_STATES_THROUGH_THE_UNMOUNT` 与按发布分 | 232；txg5/15/18 各 20 | 205；各 11 | 各 −9 |
| 同文件 `prepare` 五个脚本的段序列数组 | 含 `4` | 每个 `4` 换成 `2, 2` | 写数不变（477 等） |
| 同文件 `the_fixed_script_through_the_third_publish_…` | 26 段、202113075、甲二 93 | 27 段、202113066、84 | −9 |
| 同文件 `the_fixed_script_through_the_rollback_publish_…` | 29 段、206307382、104 | 30 段、206307373、95 | −9 |
| 同文件残留记录那条流：段序列与 `(states, 走到残留记录的)` | `…4, 18…`；(31, 19) | `…2, 2, 18…`；(22, 10) | 展开段里 15 → 3 + 3 |
| `second_transaction_step_three_acquisition_barrier_layer0` | 段序列 `…18…4, 10…`、展开 [4, 10]、1 + 15 + 1023、第 13/14 段 | `[2,2,1,2,2,1,26,2,1,26,2,1,2,2,18,2,1,18,2,1,18,2,1,2]`、展开 [2, 18]、1 + 3 + 262143、第 14/15 段 | ⚠️ 这条的旧钉值在改之前就已经过时：A/B 的单元写段早是 26、写行那段早是 18（第二条流的数组里是这样），它钉的还是 18/10。新值照第二条流那条已钉住的数组的前 23 段推出来，没跑过；展开 18 写那一段是 2^18 个状态，比旧钉值多两个量级（旧钉值本来就跑不过，实际代价早就在） |

不受影响（推的，没跑）：`first_transaction_step_seven_layer0`（mkfs → 取号，并掉）、`second_transaction_step_three_formatted_pool_layer0`（同）、`second_transaction_position_addressed_trees_layer0`（取号在被录的窗口之前，窗口里那次取号的屏障与前一道并掉）、`second_transaction_parallel_line_one_layer0`、`…parallel_line_three_spill_over_layer0`、`…supplement_two_tree_split_layer0`（都没有重开取号与抬 F）。

主 agent 集成时要弹窗问用户跑的：上面两个改了钉值的层 0 二进制的快档（`second_transaction_step_zero_layer0` 不带 `--ignored` 的那几条、`second_transaction_step_three_acquisition_barrier_layer0`），与 54 号 `--full` 里的 `crash-case:layer0-second-stream`；`crash-case:floor-raise-pushed-by-the-session` 那条（`#[ignore]`）枚举的段也多了开头一道屏障，状态数不钉、只判 ≤ 10⁶，没算。
新写的用例没有一条调 `enumerate_layer0*`，没有要登记进 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行。

### kb 写回（我写不了，给原文）

`.claude/kb/layout/01-first-txn.md` 第八节（门禁 52 号核；我没跑 52 号，改了测试数组之后它按今天的 kb 会红）：
1. 「第一次可写挂载取号」那一行，在「取号两写之后一道屏障」前面加一句：「取号两写之前也有一道屏障（代码审阅第 19 条：覆写较旧那一槽之前，上一次写进另一槽的先持久）；首次挂载上它紧跟 mkfs 末尾那道、录制流里并掉，登记的段序列不变」。
2. 「第一次之后的可写挂载（写行）」那一行：「整条流里取号的 2 个写与上一次发布的 2 个系统配置槽写合成 4 写一段」改成「整条流里取号的 2 个写自成一段，与上一次发布的 2 个系统配置槽写之间隔着取号写之前那道屏障（2、2）」；括号里「第二条流第 13–17 段 `4+10+2+1+10`」那串已经过时（今天是 `2+2+18+2+1+18`，第 13–18 段），一并改。
3. 第二条流那段 ⚠️：`PROBE … sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 2, 2, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 2] writes=477 roots=19 segments=64 closed_form=6649413719 quick_tier=205`；段序列串同上用 `+` 连；「6649413746 个状态（19 条根槽写、61 段…）」→「6649413719 个状态（19 条根槽写、64 段…）」；「平时快档…只有 232 个状态」→ 205；「到 C 是 `ThirdVersion` 那一行（26 段、202113075），到 D 是 `RollbackToFirstVersion` 那一行（29 段、206307382）」→「（27 段、202113066）…（30 段、206307373）」；「第 13 段是 B 的 2 个系统配置槽写与重开取号的 2 个系统配置槽写合成的 4 写一段（进程退出与重开之间没有屏障…）」→「第 13 段是 B 的 2 个系统配置槽写、第 14 段是重开取号的 2 个（取号写之前那道屏障把它们分开，代码审阅第 19 条）」；后面「第 26 段」「D 之后…」的段号各 +1；「抬 F 先写系统配置、与 txg 14 那次轮换合成 4 写一段」→「txg 14 那次轮换自成一段、抬 F 先写系统配置自成一段（2、2）」；「正常卸载同抬 F 的形状（4、16、18）」→「（2、2、16、18）」。

条款（照规格，查过原文没有「覆写前一道屏障」这一句）：
- D22 已定项 16（`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 327 行起的定案）在「择槽取校验和过且世代号最大的」之后补：「**覆写一槽之前，这块盘上另一槽最近那一次写要已过一道完成了的屏障**（两槽轮换靠「覆写旧槽时新槽已持久」；每次发布末尾的轮换前面有发布自己的屏障；取号、抬 F 先写系统配置、取号失败的回卷这三处前面紧挨着上一次系统配置写，各自先发一道池屏障）」。
- D23 已定项 16「取号那一步的屏障」一条补：「取号那两次系统配置写**之前**也要一道完成了的屏障：上一次写进另一槽的系统配置（上一次发布末尾的轮换，含上一个进程退出之前那一次）先持久。这道屏障报错 ⇒ 取号失败、一个字节不写、没有要回卷的。回卷写之前同样一道；它报错 ⇒ 回卷不写，照『回卷不成才只读挂载』（D18 已定项 11）」。
- D16 已定项 1「抬 F 那一串」与已定项 7「抬 F 那一串在它的第一次发布之前多一步」：「先把新 F 写进每块盘的系统配置、过一道屏障」→「先过一道屏障（上一次写进另一槽的先持久），再把新 F 写进每块盘的系统配置、再过一道屏障」；「先写那一步任何一块盘失败」那句包括写之前那道屏障报错（那时一块盘都还没带上新 F）。
- 代价一句（D16 已定项 1「生效」那段写了「每块盘多一次系统配置写、加一道屏障」）：改成「加两道屏障」；取号那一步同样多一道，首次挂载上设备多收一次 FLUSH（录制流里并掉）。

## 停下交主 agent 的设计问题

- **Q1（第 23 条，与条款对不上，没自己改口径）**：规格要「块设备错与别的 `MountError` 原样往上交」，照做之后抬 F 那一串在预演里自己的空发布取不到落点（`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { cause: PlacementRefused(NoFreeSlotOnAnyDevice) }`）也不再算空间不够：会话交回 `FloorRaiseFailedWhilePushingForSpace`，不再是 `NoSpaceAfterRaisingTheFloor`（ENOSPC）。D16 已定项 1「准入」那一行写的是「准入放行而落点取不到时先推空发布抬 F 再判……做满仍不够才报 ENOSPC」，抬 F 自己推不出空间按字面更像「推不动 = 空间不够」。实测：`second_transaction_supplement_three_random_history` 在主工作区红一条 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（`…random_history.rs:418`「覆盖写、第一个文件一次都没被「每块盘上都没有」拒过」；报告里这类拒绝 457 次都改报成 `FloorRaiseFailedWhilePushingForSpace(MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite(…, PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)))`），本轮开工时的基线副本上同一个二进制 24 过 0 红（`baseline-random_history.log`）。门禁 74 号红的就是这一条。两个走法等主 agent 定：(a) 预演被拒且 `cause` 是 `refusal_is_short_of_space` 那两种时，归 `FloorRaiseStop` 的一个新成员（例如「抬 F 自己的空发布也取不到落点」），仍走推满仍不够 / ENOSPC；(b) 照规格字面，把那条用例的计数改成也认新成员。我两样都没做，代码停在规格字面、那条用例红着。
- **Q2（第 17 条的射程）**：只守住了可写挂载。挂着之后的 `MountedSession::publish_user_change`、`raise_rollback_floor*`、`unmount`、`roll_back_by_a_forward_publish` 仍然每次拿调用方交进来的参数建写入口（`mount.rs` 里 `raise_the_floor_through` 与回退那两处 `PoolWriter::new(parameters, …)`、`mounted_session.rs` 的 `publish_the_change_once`），调用方挂载之后换一份参数照样改写盘上不可变段。怎么收（挂载交回盘上那一份参数、会话持着；或每个入口各比一次、回退要新的 `RollbackError` 成员）规格没写，没做。
- **Q3（第 18 条的射程）**：同上，那几个入口也收设备表，没查重复身份。
- **Q4（第 17 条比哪几项）**：只比 `MakeFilesystemParameters` 里有的 fsid、根环逐区域设备身份、几何五项；系统配置里的设备数 `devs`、本盘设备号不在参数里，没比（交进来的盘数与 `devs` 不同、某块盘系统配置里的本盘设备号与盘表给的身份不同，都没拒）。
- **Q5（第 23 条挂载那一处的写账）**：挂载在写行、暖机（与之前推成的几串）都落盘之后因抬 F 报错返回 `Err`，交回的是抬 F 自己的错；写行与暖机那几次的写账随 `Mounted` 一起丢了（`MountError::Publish` 那一格按收口表第 58 行是连已落盘的一起交的）。要不要包一层带上它们，规格没写。
- **Q6（第 19 条回卷写之前那道屏障报错）**：我让它交回 `AcquisitionRollback::RollbackFailed(那道屏障的错)`、一个回卷写都不发（照 D18 已定项 11「回卷不成才只读挂载」；不发是因为那时刚写的取号写未必持久，覆写另一槽会两槽一起撕坏）。改之前「写之后那道屏障每次都报错」的池会回卷成功（`RolledBack`），现在是 `RollbackFailed`；`instance_acquisition.rs` 的原用例改成「写之后那道只报错一次」，另加一条钉「一直报错」这一格。条款要补的原句见上（D23 已定项 16 那条）。
- **Q7（第 19 条，规格没要的）**：审阅原文另提「上一次发布的 `RotateSystemConfigurationSlots` 之后也没有尾随屏障」；规格选的是在三处写之前补，我没加尾随屏障。另外新开的写入口按「发过写」起步，首次挂载上取号前那道在设备一层多一次 FLUSH（录制流里并掉）：`instance_acquisition.rs` 的 `barrier_right_after_the_acquisition_barrier_is_not_sent_to_the_devices` 钉值 1 → 2，真设备二进制（`first_transaction_on_device`）的挂载窗口起点改成「收到过写之后的第一道屏障」；QEMU（55 号）上的计数没跑过。
- **另记（不是本轮改法带出的，没追）**：`second_transaction_supplement_three_crash_injection` 的快档用例 `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed` 在本轮开工时的基线副本上就红（新发现 1：种子 7463871032432355129 第 17 段 `ModelDisagreement 冷启动读回`，落在 `CrashRecoveryAbandoningTheNewestRoot`）；改完之后仍红，新发现换成种子 7463871032432355115 第 56 段 `RecordCheck claimed_state_missing_unit`（同一种操作，2 写段里扣下第 0 个）——段切法变了、抽到的崩溃点跟着变，这一格是不是屏障带出的新问题我没判，交 crash-verifier。同文件另两条（`every_crash_state_of_a_written_out_history…` 与 `one_thread_and_four_threads_render_the_same_report`）：前一条是崩溃点下限 100（改后摆得出 95：写死的历史里第二次可写挂载那一段 15 → 3 + 3），我改成 ≥ 95；后一条两次跑的报告只差「判红的崩溃镜像曾写在…」一行，是上面那条新发现带出来的，没改。
- **审阅第 18 条的原判不全对**：今天的代码上 `[(盘0,A),(盘1,B),(盘0,C)]` 走不到取号写，先在 `admission.rs:288` panic（空间准入判着时）；只供测试的开关关掉准入时走得更远，我没试。
- **crash_injection 二进制交回前重跑（主工作区，`run4-crash_injection.log`）**：6 过 2 红 1 忽略；红的仍是快档与「一线程与四线程报告相同」两条，新发现三条：种子 …115 第 56 段 `RecordCheck claimed_state_missing_unit`（同上）与种子 …136 第 91 段 `CheckerViolations I-3.1`（`after_the_writable_mount_and_one_publish` / `second_crash_inside_warm_up` 两格）。后一格的检查点名字是别的会话这段时间里往 `crates/singlefs-harness/src/crash_injection.rs` 加的（崩溃状态上再可写挂载那一路，那份文件本轮在别人手里改着、`cargo fmt --check` 也红在它上面），与我的改动分不开，没追。`every_crash_state_of_a_written_out_history…`（我改了下限的那条）这次过。

## 交回前的验证（末尾原样）

跑之前看负载：`qemu-system` / `vm-bench.sh` / `e152` / `fio` 一个都没有（`load-before-final.txt` 空）；E158 执行员的 `e158_root_choice_repair` 七条在跑，照常 `nice -n 19` 并行。

动到的测试二进制（主工作区，最后一次跑的结果）：
- `writable_mount_refuses_caller_parameters_…`：`test result: ok. 2 passed; 0 failed`；`writable_mount_refuses_a_device_identity_handed_in_twice`：`ok. 1 passed`；`system_configuration_slot_is_overwritten_only_after_a_barrier`：`ok. 3 passed`；`a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is`：`ok. 2 passed`（run1-*.log）
- `instance_acquisition`：`ok. 6 passed`；`rollback_floor_written_…`：`ok. 11 passed`；`-p singlefs-core --lib`：`ok. 123 passed`；`-p singlefs-harness --lib`：`ok. 81 passed`；`--bin first_transaction_on_device`：`ok. 16 passed`（run2-*.log）
- `second_transaction_supplement_two_fsync_drop_…`：`ok. 17 passed`；`second_transaction_admission_raises_the_floor_before_refusing`：`ok. 10 passed`；`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`：`0 passed; 1 ignored`（run1）
- 可能被屏障牵动、一并跑过的：`publish_order_matches_litmus`、`first_transaction_step_one_mkfs`、`checker_known_bad_images`、`second_transaction_supplement_one_write_accounting`、`…two_warm_up_counter`、`…two_commit_generated_fallback`、`second_transaction_step_three_second_instance`、`…step_three_formatted_pool`、`first_transaction_step_five_publish`、`…three_fault_injection` 全部 exit 0（run3-*.log）
- `second_transaction_supplement_three_random_history`：`test result: FAILED. 23 passed; 1 failed; 2 ignored`（Q1）
- `second_transaction_supplement_three_crash_injection`：`test result: FAILED. 6 passed; 2 failed; 1 ignored`（上一条）
- 名字含 layer0 的两个二进制改了钉值，没跑。

`cargo fmt --check`：退 1，只红在别人的三份（`crash_injection.rs` 6 处、`model_comparison.rs` 2 处、`model.rs` 1 处，都不是我改的行）；我的文件格式化过、不在清单里。
`cargo clippy`（check.sh 那一套 lint，`--all-targets --all-features`）：退 101，红在 `e156_allocation_basis_counts.rs` 与 `e158_root_choice_repair.rs` 的 `shadow_unrelated`（E156 / E158 执行员的文件）；只对我动到的目标（`-p singlefs-core --all-targets`；`-p singlefs-harness --lib --bin first_transaction_on_device` 加上面那十二个测试目标）跑，两次都退 0、末行 `Finished \`dev\` profile [unoptimized + debuginfo] target(s)`。
`cargo build --offline --all-targets`：`build exit 0`。

登记给我的门禁阶段（末行原样）：
- 33 号：先红（第 93、170、301、690 行原文命中 0 次，都是我改名、改调用带出的），修锚、证红之后重跑退 0：`✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 883 条的原文各命中源码一次；…`
- 53 号：退 0，`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
- 74 号：退 1，`签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。`（红在 random_history 那一条，Q1）
- 92 号：退 0；93 号：退 0；94 号：退 0（末行见 `final-gate-*.log`）
- 89 号：退 77（本次未跑）：`「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`

## 仓副本与编译目录

删了：`/tmp/claude-1000/impl-rev-a1/baseline`（2.3G，含 target）、`/tmp/claude-1000/impl-rev-a1/proof`（3.5G，含 target）。
留着：`/tmp/claude-1000/impl-rev-a1/pristine/`（四份源文件原件，几百 KB，证红还原用、主 agent 可对照），其余是日志与脚本。
`/tmp/singlefs-crash-injection-1155558-seedbase-7463871032432355115` 不是我建的（我的跑都是种子基 …113），没动。

## 没做什么

- 没走三方对抗；层 0（两个改了钉值的二进制与 54 号 `--full`）、QEMU（55 号，真设备二进制挂载窗口改了起点）、herd7、crates 变异整表（59 号）归 crash-verifier，都没跑；没提交。
- 门禁 52 号没跑（不登记给我）；按第四节它会红，等 kb 那几句写回。
- 没写 kb（第 19 条要补的条款原句与 layout 登记句都在第四节）。
- Q1–Q7 都没自己定；Q1 那条用例红着交回。
