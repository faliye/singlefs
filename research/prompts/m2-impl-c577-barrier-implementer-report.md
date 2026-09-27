# 实 C577 报告：发布返回之前，系统配置轮换之后再加一道屏障

写于 2026-09-27 JST 17:5x。主 agent 在 JST 17:0x 到 17:3x 之间两次改派：第一次，8 份钉值文件归 A3b 善后，我不碰；第二次，第 1 条盘点停下，第 4 条不做，补丁只留 transaction.rs、新测试和变异行。这份报告照第二次改派写。

## 结论

- `crates/singlefs-core/src/transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration` 现在的顺序是：`WriteRootRecordForceUnitAccess` → `RotateSystemConfigurationSlots` → `CommitStep::Barrier`，屏障做完才返回。三条发布路径、原样重发、抬 F 那一串、卸载那一串都经过这一处，所以全都带上了这道屏障。
- 新测试文件里 3 条用例，改后绿。去掉那道屏障之后，3 条都红（`prove-red.sh` 3/3 抓到，改完 clippy 那处之后又复证了一遍）。
- 屏障报错怎么处置：D16 已定项 1 里没有「发布调用报错」这一格（`grep -rn '发布调用报错' .claude/kb` 零命中）。我照的是今天根槽 FUA 之后出错（轮换报错）的那一格，也就是 D23 已定项 14「这一版的失败处置」（`.claude/kb/decisions/23-journal的角色与格式.md` 第 397 行）：发布不接受失败，冻结之后原样重发。代码不用另写分支，屏障的错走 `persist_publish_writes` 的 `?`，和轮换报错进同一条路径。经分配器的发布会冻结（`persist_the_publish_or_freeze_it`）。零单元发布不冻结，整个挂载返回错误（`publish_without_units`）。第 3 条用例钉住了经分配器的那一支。零单元那一支没有专门的用例。
- 什么现象会推翻这条结论：同一份池上，某次发布返回之后，录制流里存在一个可达的崩溃状态，它两盘两槽自证过的系统配置 tail 的最大值小于这次发布末条记录的计数器。新测试逐个崩溃点判的就是这件事。

## 这一轮写过的文件（都在副本里，主工作区一个字没动）

- `crates/singlefs-core/src/transaction.rs`：加屏障，并改了 6 处注释里的持久顺序文字（模块头两行、`PublishWrites`、零单元发布的文档与行内注释、写行那一版、带文件那一版），末尾都加上「→ 屏障」。另在 `write_the_raised_floor_into_every_system_configuration` 第一道屏障之前加了一行注释，说明同一个写入口紧跟在发布之后时，这道屏障前面没有写，写入口不发它。取号那里我也加过一行同样意思的注释，但它挡住了 `crates/mutations.tsv` 第 1301 行（实审 A3c Q1）的锚点，门禁 33 号判红，已经拿掉。第 880 行（实审 A1 第 19 条）的锚点也被挡过一次，我把那行注释挪到了锚点之前。
- 新建 `crates/singlefs-harness/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs`。
- `crates/mutations.tsv`：只在末尾追加 3 行，内容在 `patch/mutations-append.tsv`，变异名是：
  - `C577：发布在系统配置轮换之后不过屏障就返回（每条发布路径返回之后见证值可以落后）`
  - `C577：发布在系统配置轮换之后不过屏障就返回（树表 0 条的一版上返回之后见证值可以落后）`
  - `C577：发布在系统配置轮换之后不过屏障就返回（轮换之后没有屏障可报错，发布不冻结）`
- 补丁对主工作区 `git apply --stat` 的原样输出（我没动主工作区，所以没有 `git diff --stat -- crates litmus`）：

```
 crates/singlefs-core/src/transaction.rs            |   31 +
 ...the_system_configuration_rotation_is_durable.rs |  523 ++++++++++++++++++++
 2 files changed, 543 insertions(+), 11 deletions(-)
```

- 对主工作区跑 `git apply --check /tmp/claude-1000/impl-c577-barrier/patch/crates.patch`，退出码 0。主工作区的 `transaction.rs` 最后一次改动是 06:56 UTC，在我建副本之前，底座就是它。
- 补丁目录 `/tmp/claude-1000/impl-c577-barrier/patch/` 下有 `crates.patch`、`mutations-append.tsv` 和 `report.md`（这份报告的拷贝）。

## 新测试怎么判，证红结果

判据：一次发布返回之后的每个崩溃点（录制流前 p 步已发出，p 从返回那一刻数到流尾）上，每个可达的崩溃状态都要满足：两盘两槽全部自证过的系统配置的 journal tail 取最大值，这个值不小于这次发布末条记录的计数器。切段用 `SegmentClosingRule`，和层 0 同一条规则。见证值只由系统配置槽写决定，所以只对还没被放行的那几个系统配置槽写取全部子集。撕开的原地覆写不另外枚举：被撕的是较旧的那一槽，撕坏之后自证不过，对见证值的影响和那一写没持久一样。另外还有一道自检：返回之前、最后一步还没发出的那个崩溃点，判法必须报得出「没见证」，原样重发的那一次除外（枚举 `WitnessJustBeforeTheReturn` 写明了原因：失败那一遍已经发过同一个 tail 的轮换）。没有这道自检的话，返回之后的绿什么都说明不了。

| 用例 | 罩住的路径 | 去掉屏障后红在哪一条断言 |
|---|---|---|
| `every_crash_point_after_a_publish_returns_keeps_a_system_configuration_witness_of_its_last_record` | 暖机零单元发布、第一个文件、3 次覆盖写、抬 F 那一串、重开可写挂载（取号、写行、暖机）、正常卸载 | 文件第 327 行 `assert_eq!`：暖机第二次空发布在崩溃点 43、44、45 等处最小见证值是 1，计数器是 2 |
| `every_crash_point_after_a_publish_on_a_version_without_file_returns_keeps_the_witness` | 只做过 mkfs 的池第一次可写挂载；树表 0 条的一版上重开写行 | 文件第 366 行 `assert_eq!`：崩溃点 43 起见证值 1，计数器 2 |
| `a_barrier_failing_after_the_rotation_freezes_the_publish_and_the_resend_returns_only_after_its_rotation_is_durable` | 盘 1 上轮换之后那道屏障报错一次：发布报 `PublishError::BlockDevice`、冻结 (实例 1, txg 4)、原样重发成功之后才返回，返回之后见证到 | 文件第 470 行 `expect_err`：没有屏障可以报错，发布照常返回成功 |

- 3 条变异改的是同一处：原文 `        journal_instance,\n    })?;\n    writer.perform(CommitStep::Barrier)\n}`（在 transaction.rs 里恰好命中 1 次），替换成 `        journal_instance,\n    })\n}`。每条都跑整个测试二进制，每次三条用例同时红，没有别的测试跟着红。3 行都用 `prove-red.sh` 证过，没有留给 59 号去证的行。
- 基线红集：`prove-red.sh` 在施加变异之前先跑了一次不改源码的基线，是绿的。
- `prove-red.sh` 第二次（最终版）的原样末行：`✓ 点名 3 条：跑了 3 条，跳过 0 条，跑的都抓到了`。

## 第 1 条盘点：已跑完的那部分（主 agent 改派之后停下，没跑完）

做法：在改后的副本里逐个跑 `cargo test --offline -p singlefs-harness --test <名>`，再加 `-p singlefs-core --lib`，每个都经 `run-with-memory-cap.sh 8G`、`capped.sh 4` 起。跑完 core lib 加 57 个 harness 二进制时停下（按 pid 停了 389669 循环脚本和 1363769 测试二进制，其余几层随之退出，ps 复查一个不剩）。跑完的里面 42 个绿（含 core lib 和新测试）、16 个红。**16 个红的都没对基线**：基线副本建好了，但主 agent 改派时还没开跑，已经删掉。下面「和 C577 有没有关系」一栏是我按断言内容读出来的推测，不是基线比出来的。

改后红的 16 个二进制（panic 的位置照日志原样）：

| 二进制 | 红的用例、位置 | 改后值 / 期望值 | 和 C577 有没有关系（推的） |
|---|---|---|---|
| first_transaction_step_five_publish | recorded_paths_match_the_registered_segment_sequences，:347 | 暖机那条路径 `(20, "2+1+2+2+1+2", 15)` / `(18, …)` | 有：每次发布多一道池屏障，录制流按设备记，每次多 2 步 |
| publish_order_matches_litmus | litmus_writer_threads_follow_the_recorded_publish_order，:51 | 第一个事务 35 步 / 33 | 有：同上，+2 |
| in_place_overwrite_torn_state_count | the_first_transaction_stream_counts_with_the_torn_third_state_as_measured，:317 | 闭式 16777240 / 67108885 | 有：见下面的算法 |
| crash_image_journal_hint_matches_the_full_ring_scan | on_the_first_transaction_stream_…，:95 | 段数 11 / 10 | 有 |
| crash_segments_per_device_and_torn_in_place_overwrites | :415、:618 两条 | `[2,2,1,2,2,1,2,24,2,1,2]` / `[2,2,1,2,2,1,26,2,1,2]` | 有 |
| second_transaction_step_one_overwrite | overwrite_publishes_the_second_version_through_the_same_commit_shape，:98 | 末段 `[system_configuration_slot×2,barrier×2]` / `[system_configuration_slot×2]` | 有 |
| record_checker_judges_absence_by_the_persisted_set | 5 条都红在 :314 | σ `(16, 16)` / `(18, 16)` | 有：上一次轮换那 2 个原地写不再和 σ 的 16 个单元写同段 |
| crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume | :509、:747 | 快档状态数 46 / 54 | 有：第一条流快档的状态数跟着段变 |
| crash_enumeration_sharded_across_processes | :348、:523、:706、:800、:869，另有 src/crash.rs:3960（账本头的 plan 哈希、slices 23→46 对不上） | 46 / 54；片行 47 / 55 | 有：同上；另有金样文件逐字节比对，也要重录 |
| rollback_floor_written_into_the_system_configuration_first_and_normal_unmount | :242、:495、:764 | 抬 F、卸载那一串的头几步不再以一对屏障开头（改后是 `[(SystemConfigurationSlot,0),(SystemConfigurationSlot,1),(Barrier,0),(Barrier,1),…]`，期望开头是 `(Barrier,0),(Barrier,1)`）；:764 是写 F 报错那一格，改后是 `[(Write,0)]`，期望 `[(Barrier,0),(Barrier,1),(Write,0)]` | 有：发布末尾那道屏障已经放行，同一个写入口上抬 F 前那道屏障前面没有写，`PoolWriter` 不发它。这几条用例截流从哪里起算没细查 |
| second_transaction_step_three_second_instance | torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg，:661 | 可写挂载被拒：`NewerStateStillUnreadableAfterOneReread`（见证计数器 6，所选那一版 txg 4） | 有：轮换现在返回前就持久，见证值到了 6，乙判出有更新的状态；用例钉的场景要重看 |
| second_transaction_supplement_three_crash_injection | :600、:881（3 条） | 落到的实例 1 / 期望 2；种子 7463871032432355115 第 28 段的崩溃状态 | 有：崩溃状态集合跟着段变 |
| second_transaction_step_five_reuse | raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43，:898 | 抬 F 之前 I-7.4 已红 | 和 `:1082` 同形，看下一节；规格说随机历史那一条改前也红，这一条多半改前也红 |
| checker_known_bad_images | an_allocation_generation_past_its_unit_birth_…，:4572 | 可写挂载报 `Recovery(InvariantViolated { "E142 走读同款" … })` | 看不出关系，没对基线 |
| core_review_geometry_back_chain_and_empty_inode | a_journal_ring_shorter_than_one_record_is_refused_before_any_write，:231 | 报成 `JournalRingHoldsFewerRecordsThanTheSafetyFactor` | 看不出关系（像是 A3c、A3b 那边的拒绝次序）；这份文件已改派给 A3b 善后 |
| acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics | a_device_whose_slots_turn_unreadable_after_the_acquisition_barrier_refuses_before_any_write，:274 | 录制流一步没多，期望多一道屏障 | 看不出关系，没对基线 |

钉值怎么变（给统一盘点用的算法）：

- 每次发布在轮换之后多一道池屏障，录制流按设备记，一次多 2 步（两块盘各一条 Barrier）。
- 轮换的 2 个系统配置槽写原来和下一件的写同段，现在自己关成一段。下一件指：下一次发布的单元写；或者取号、抬 F、卸载先写系统配置的那 2 写，今天这三处前面各有一道屏障。所以原来 2+n 写的一段拆成 2 和 n 两段，闭式里 (2^(n+2) − 1) 变成 3 + (2^n − 1)。流尾轮换那一段本来就是 2 写，只是段里多 2 条 barrier，段序列数字不变。
- 下一次发布是零单元发布时：它开头那道屏障前面没有写，写入口不发，录制流里少那 2 步。段序列不变（原来那道屏障就在关轮换那一段），只是那道屏障从「下一次发布的第一步」挪成「这一次发布的最后一步」。抬 F、卸载先写系统配置之前那道屏障、取号写之前那道屏障，同一个写入口上也照这样不发（rollback_floor 那三条红就是这个）。
- 第一条流：`2+2+1+2+2+1+26+2+1+2`（67108885）变成 `2+2+1+2+2+1+2+24+2+1+2`。闭式 = 1 + (3+3+1+3+3+1+3+3+1+3) + (2^24 − 1) = 16777240，in_place_overwrite 那条改后打出的正是 16777240。
- 隔离看的路径：第一个事务 `24+2+1+2` 段序列不变，末段种类多 `barrier×2`，步数 33 变 35。暖机路径步数 18 变 20，段序列 `2+1+2+2+1+2` 不变。
- σ（record_checker）：`(18, 16)` 变 `(16, 16)`，σ 段闭式 2^18 变 2^16。登记在 crash-case 的 `c561-sigma-full` 那一行的「闭式 2^18」也要跟着变。

没跑到的 39 个 harness 二进制（第一个是停下时正在跑的那个，结果不算）：

second_transaction_supplement_three_fault_injection second_transaction_supplement_three_random_history second_transaction_supplement_two_accounting_node_full second_transaction_supplement_two_admission_formula second_transaction_supplement_two_c495_first_file_version_previous_record second_transaction_supplement_two_c519_whole_device_loss_after_warm_up second_transaction_supplement_two_c533_row_publish_record_without_its_root second_transaction_supplement_two_commit_generated_fallback second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version second_transaction_supplement_two_instance_table_chain second_transaction_supplement_two_instance_table_page_full second_transaction_supplement_two_instance_table_second_page_write second_transaction_supplement_two_multi_record_transaction_zero_publishes second_transaction_supplement_two_presumed_clause_checks second_transaction_supplement_two_publish_failure_resent_unchanged second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit second_transaction_supplement_two_record_checker_reuse_legality second_transaction_supplement_two_release_checksum_quarantine second_transaction_supplement_two_reused_record_overlap second_transaction_supplement_two_root_ring_turn_in_one_mount second_transaction_supplement_two_row_publish_admission second_transaction_supplement_two_row_publish_checks_before_acquisition second_transaction_supplement_two_tree_identifier_watermark_crash_orphans second_transaction_supplement_two_tree_nodes_and_the_central_mapping second_transaction_supplement_two_tree_split second_transaction_supplement_two_unequal_devices second_transaction_supplement_two_unreadable_abandoned_root_slot second_transaction_supplement_two_warm_up_counter sparse_devices_refuse_requests_outside_the_device_like_the_file_backend sparse_device_zero_fill system_configuration_mutability_classes system_configuration_per_device_redundancy system_configuration_rollback_floor_and_layout_identity system_configuration_slot_is_overwritten_only_after_a_barrier system_configuration_slots_per_region writable_mount_refuses_a_device_identity_handed_in_twice writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration writable_mount_refuses_caller_parameters_disagreeing_with_the_system_configuration_on_disk zero_unit_publishes_are_compared_with_what_the_row_publish_of_the_same_mount_handed_in

另外：`second_transaction_supplement_three_random_history` 在门禁 74 号里跑过（下面有），红的只有 `:1082` 那一条。

改派给 A3b 善后的 8 份：core_review_geometry_back_chain_and_empty_inode 红（见表）。a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable、core_review_unit_area_start_and_publish_limits、core_review_tree_table_duplicates_and_slot_one_search、second_transaction_admission_raises_the_floor_before_refusing、admission_checkpoint_cost_per_device_paths、second_transaction_allocation_record_tree_geometry_of_writer_and_reader 改后绿。second_transaction_supplement_two_root_ring_turn_in_one_mount 没跑到。8 份我都没改过。

各轮日志：`/tmp/claude-1000/impl-c577-barrier/run-modified/<二进制>.log`，退出码表在同目录的 `exit-codes.tsv`。

## 我不改、交主 agent 另派的（从盘点和代码现查列出，按文件）

- 名字带 layer0 的测试文件（钉的段数组、闭式会跟着第一条流那条算法变；都没跑）：`first_transaction_step_seven_layer0.rs`（第 458、763 行两个数组）、`second_transaction_step_zero_layer0.rs`（第 382 行数组；第 581、610 行两个闭式）、`second_transaction_parallel_line_one_layer0.rs`（第 180 行数组；第 365、385 行闭式）、`second_transaction_step_three_formatted_pool_layer0.rs`（第 59 行数组；第 137 行 67108885）、`second_transaction_step_three_acquisition_barrier_layer0.rs`（第 222 行数组）、`second_transaction_supplement_two_tree_split_layer0.rs`（第 302 行 `expected_segment_sizes`；第 414 行闭式式子）、`second_transaction_position_addressed_trees_layer0.rs`、`second_transaction_parallel_line_three_spill_over_layer0.rs`。
- `.claude/gate.d/stage-inputs.tsv` 的 crash-case 行：layer0-first-stream、layer0-second-stream、layer0-parallel-line-one-stream、layer0-tree-split-streams（注释里写着 78905428 这个数）、c561-sigma-full（注释里写着 2^18）、floor-raise-pushed-by-the-session、crash-injection-fast-tier。用例钉的数都跟着变。
- litmus：`litmus/first-txn-*.litmus` 与 `commit-publish*.litmus` 的 P0 只写到根槽，不写系统配置槽和它之后的屏障。`publish_order_matches_litmus.rs` 那条红的只是步数（33 变 35）。要不要给「轮换之后屏障」补一条 litmus，交主 agent 定；litmus 我没改。
- E142 独立模型 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`：暖机（第 2719 行）和第一个事务（第 3083 行）写完系统配置槽之后都没有屏障。产物 `name=segments`、登记表 `.claude/kb/layout/01-first-txn.md` 八那一节（第 402、403 行两条路径，第 410 行整条流 `2+2+1+2+2+1+26+2+1+2`、67108885）和门禁 52 号会跟着对不上。模型、产物、kb 都不归我。
- 55 号装置样本（`first_transaction_on_device`，别的会话 singlefs-39 在重录）：每块盘每次发布多一次 FLUSH，`name=device_calls` 的 barriers= 跟着变；`src/bin/first_transaction_on_device.rs` 第 2019、2025、2034、2039 行的单测也钉着屏障数（harness 的 bin 目标，我没跑）。
- `crates/singlefs-harness/src/crash.rs` 归 A3b，我没碰。

## `:1082` 那条的看法（交主 agent，这里有一处条款没定）

`:1082` 指的是 HEAD 里 `second_transaction_supplement_three_random_history.rs` 第 1082 行那条用例，今天工作区里改名成 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`（第 1188 行）。它的形状由 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 造：C 的根槽和数据单元暂存后清零，再把每块盘世代号最大的那一槽系统配置直接清零，然后可写挂载。

- 改完之后，崩溃这一路造不出这一形了：发布返回时轮换已经持久（新测试钉住）。
- 但这个辅助函数不是走崩溃造的，它是在轮换持久之后直接清零两块盘各一槽，形状照样造得出来。改后它就成了多故障形：每块盘坏一槽系统配置，同时根槽和数据单元一时读不出。D22 已定项 8 容得下每块盘坏一槽，乙在这一形上照旧看不见 C。所以这条用例和 `second_transaction_step_five_reuse.rs:898` 那条同形用例，改后照旧在抬 F 之前红在 I-7.4（门禁 74 号和盘点里都看到了）。
- 我的判断：C577 的屏障罩住的是崩溃那一路，这一形罩不住。它该钉什么，取决于「系统配置每块盘各坏一槽，同时最新根一时读不出」这种组合在不在容错射程里。条款没写（D22 已定项 8 只说容一槽坏，没说跟根槽读不出叠加时怎么办），我不定，交主 agent。两份用例都不在我的文件单里，没改。

## 停下交主 agent 的设计问题

1. 上一节：多故障形（每块盘各坏一槽系统配置，同时最新根一时读不出）在不在射程里，`:1082` 和 step_five_reuse 那两条该钉什么。
2. 规格要照的「D16 已定项 1『发布调用报错』那一格」不存在。我照的是 D23 已定项 14「这一版的失败处置」（轮换报错走的就是这一格）。主 agent 要是想要的是另一格，这里要改。
3. `second_transaction_step_three_second_instance.rs:661`：改后可写挂载被乙拒（见证值 6 现在返回前就持久了）。那条用例原来钉的「锚点记录撕开、链从下一个 txg 起」要换个造法，还是改钉成被拒，交统一盘点定。
4. 取号、抬 F、卸载先写系统配置之前那道屏障，同一个写入口紧跟发布之后时不再发出（`PoolWriter` 见上次屏障之后没有写就不发）。代码审阅第 19 条要的「覆写旧槽时新槽已持久」由发布末尾那道屏障兑现，语义没变；录制流里少那 2 步，rollback_floor 那 3 条用例钉的次序要改。

## 交回前的验证（都在副本 work/ 里跑，末尾原样）

```
$ cargo fmt --all -- --check
fmt exit=0
$ cargo clippy --offline --all-targets --all-features -- -D warnings + CODE_DISCIPLINE_LINTS 七条 -D
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.03s
clippy exit=0
$ cargo build --offline --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.44s
build exit=0
$ cargo test --offline -p singlefs-harness --test a_publish_returns_only_after_the_system_configuration_rotation_is_durable（经 run-with-memory-cap.sh 8G）
test a_barrier_failing_after_the_rotation_freezes_the_publish_and_the_resend_returns_only_after_its_rotation_is_durable ... ok
test every_crash_point_after_a_publish_returns_keeps_a_system_configuration_witness_of_its_last_record ... ok
test every_crash_point_after_a_publish_on_a_version_without_file_returns_keeps_the_witness ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.67s
test exit=0
test exit=0
$ git apply --check …/patch/crates.patch（在主工作区）
apply-check=0
```

登记给我的门禁阶段（`stage-owners.tsv` 里 implementation-writer 那几行，都在副本 work/ 里跑）：

```
33 exit=0
  ✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.t
53 exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
92 exit=77（本次未跑：副本不是 git 仓，92 号没判）
  ! /tmp/claude-1000/impl-c577-barrier/work 不是 git 仓，本阶段跳过
94 exit=0
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 286 行里没有分支与循环（`#[cfg(test)]` 标着的项 313 行不扫）
93 exit=0
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 57 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
89 exit=77（本次未跑：收口表第 27 行那几笔的前置一个都没进来）
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）
74 exit=1（GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G）
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
    crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43
test result: FAILED. 23 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 71.03s
```

74 号红的只有 `:1082` 那一条，它在抬 F 之前红在 I-7.4。规格写明它改前就红（主 agent 15:0x 现跑）。这一形 C577 罩不住，见上面「`:1082` 那条的看法」。

## 没做什么

- 第 1 条盘点没跑完（主 agent 改派时停下）：39 个 harness 二进制没跑，16 个红的都没对基线，全部名字列在上面。第 4 条（改非层 0 钉值）按改派不做，补丁里一份钉值测试文件都没有。
- 层 0、崩溃枚举用例、55、57、59 号、全量 `cargo test`、`src/bin/first_transaction_on_device.rs` 的单测都没跑。
- 92 号、89 号退 77，本次未跑（原因在上面）。
- 零单元发布路径上「轮换之后屏障报错」那一支（整个挂载返回错误、不冻结）没有专门的用例，只有经分配器那一支有用例钉住。
- 没走三方对抗；没提交；kb、E142 模型、litmus、55 号样本一样没动。

## 删掉的副本与编译目录

- `/tmp/claude-1000/impl-c577-barrier/work`（改后副本，含 target）：20G，已删。
- `/tmp/claude-1000/impl-c577-barrier/work-base`（基线副本，还没编译过）：288M，已删。
- `/tmp/claude-1000/impl-c577-barrier/patch-a`（400K）、`patch-b`（436K），做补丁用的两份文件拷贝：已删。
- 留着的：`patch/`（交付物）、`run-modified/`（盘点日志）、`gate-*.log`、`fmt.log`、`clippy.log`、`build.log`、`new-test-*.log`、`prove-red-*.log`、`progress.md`、`spec.md`、`candidate-binaries.txt`、`not-ran.txt`、`ran.txt`、`run-candidates.sh`、`mutations-append-draft.tsv`，都是报告材料，不是仓副本也不是编译目录。
