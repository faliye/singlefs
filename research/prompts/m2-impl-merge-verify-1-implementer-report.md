# 合入后验证一 实现员报告（未做完，上下文到线交回）

写于 2026-09-27。规格 `/tmp/claude-1000/impl-merge-verify-1/spec.md`。底座 = 主工作区 crates/（与 `refs/sop/m2-closeout-code-r2-snapshot` 逐字相同，开工时 `git diff --stat refs/sop/m2-closeout-code-r2-snapshot -- crates litmus` 空）。副本 `work/`（改动与复跑）、`edit/`（静态改与 clippy、探针）。主 agent 叫停（上下文过 600k），停在编得过的点：`edit/` 上 fmt 与 clippy（带 CODE_DISCIPLINE_LINTS 七条）退出码 0，改动已同步回 `work/`，补丁从 `work/` 出。

## 一、结论（一句话）

第一节三处 src 做完、编得过；统一跑一遍跑完（105 行，红 21 个目标）；钉值改了一大半，但**改完之后只复跑到 2 个目标就被叫停**，大部分改动「编得过、没跑过」；A3b 小盘撞墙那几份、见证类那几份、first_transaction_on_device 两条、双故障两条没改。逐条见第五、六节。

推翻条件：`patch/crates.patch` 打到主工作区之后编不过；第三节点名的任一条用例在打完补丁的树上红；`patch/mutations-append.tsv` 任一行的原文在对应文件里不是恰好命中一次。

## 二、补丁与文件

- `patch/crates.patch`：15 个文件（主工作区里还在原路径的），对主工作区 `git apply --check` 退出码 0。
- `patch/crates-moved.patch`：12 个文件，旧路径出，没做 apply --check（主工作区已挪走）；清单在 `patch/moved-files.txt`。主工作区里它们挪到的是 `crates/singlefs-checker-tier/…`（`git status` 现查），不是消息里说的 `singlefs-checker/tests/`；两个 bin（e158、first_transaction_on_device）与 3 份非层 0 测试（findings_log、sharded、crash_segments）也在被挪之列。
- `patch/mutations-replacements.tsv`（7 行）、`patch/mutations-append.tsv`（9 行），第七节。
- sha256：crates.patch `24ce451b…3d24b`，crates-moved.patch `8f6a159e…73eb4`，mutations-append.tsv `5d849c15…17`，mutations-replacements.tsv `3afb5d7b…a7`。

`git diff --stat -- crates litmus`（`work/` 上，两份补丁合起来，原样）：
```
 crates/singlefs-core/src/mount.rs                  | 127 ++++++-
 crates/singlefs-core/src/recovery.rs               | 358 +++++++++++++++++---
 crates/singlefs-core/src/transaction.rs            |  13 +-
 .../src/bin/e158_root_choice_repair.rs             |  21 +-
 .../src/bin/first_transaction_on_device.rs         |   8 +-
 crates/singlefs-harness/src/history.rs             |  40 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  10 +-
 ...n_unreadable_and_the_remaining_writer_panics.rs | 118 ++++++-
 ...disk_content_is_refused_instead_of_panicking.rs |  12 +-
 ...log_dedupes_by_signature_and_survives_resume.rs |   6 +-
 .../crash_enumeration_sharded_across_processes.rs  |  22 +-
 ...mage_journal_hint_matches_the_full_ring_scan.rs |  11 +-
 ...ents_per_device_and_torn_in_place_overwrites.rs |  27 +-
 .../tests/first_transaction_step_five_publish.rs   |  16 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  47 +--
 .../tests/in_place_overwrite_torn_state_count.rs   |  13 +-
 .../tests/publish_order_matches_litmus.rs          |  26 +-
 ..._checker_judges_absence_by_the_persisted_set.rs |  34 +-
 ...ystem_configuration_first_and_normal_unmount.rs |  44 ++-
 .../second_transaction_parallel_line_one_layer0.rs |  51 +--
 .../tests/second_transaction_step_one_overwrite.rs |   5 +-
 ...action_step_three_acquisition_barrier_layer0.rs |  17 +-
 ...transaction_step_three_formatted_pool_layer0.rs |  28 +-
 .../tests/second_transaction_step_zero_layer0.rs   | 143 +++++---
 ...transaction_supplement_three_crash_injection.rs | 311 +++++------------
 ...transaction_supplement_three_fault_injection.rs |  22 +-
 ...ing_and_unreadable_reads_are_not_passed_over.rs | 370 ++++++++++++++++++---
 27 files changed, 1302 insertions(+), 598 deletions(-)
```
（27 个 = crates.patch 15 + crates-moved.patch 12；`crates/mutations.tsv` 不在补丁里，走第七节两份 tsv。）

## 三、第一节三处 src

### 1. 善后一 P1 / D16 第 37 行（`recovery.rs`、`mount.rs`）
- `readable_roots_rereading_unreadable_root_ring_slots_once` 换名 `readable_roots_rereading_ring_slots_known_to_hold_a_root_once`（另有带槽的一份 `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once`），加参数 `ring_slots_known_to_hold_a_root`；抬 F 那一处传 `ring_slots_known_to_hold_a_root_by(allocator)`。不在集合里的槽读不出当没有根、不重读；在集合里的读不出重读一次，仍读不出报错。
- **与 D16 第 37 行的一处差**：那一行写「读不出或自证不过」都重读，规格表里只写「读不出」。我先照 D16 全句做（自证不过也重读、仍坏拒），统一跑一遍里红了两条：`second_transaction_step_four_rollback.rs:807`（回退到 A，C 的根被改坏、自证不过）与 `second_transaction_step_five_reuse.rs:641`（带 F 的根被改坏、F 由系统配置撑着），两条都被新判拒成 `RootRingSlotKnownToHoldARoot{first_reading: NotSelfVerified, reread: NotSelfVerified}`。于是收窄到只管「读不出」，自证不过照旧当没有根（`recovery.rs` 那一份的文档注释写明）。**这一处要主 agent 定**：D16 那一行的「或自证不过」接不接进抬 F 的影子账那一遍与管理员回退（接的话上面两条用例要改造法）。`mount::rollback_floor_ceiling` 那一份原样（它本来就两类都重读）。
- 收窄之后这两条用例**没复跑**（叫停）。

### 2. A3b Q7 两处读不出无声放过
- 管理员回退 `rollback_candidate` 加参数 `ring_slots_known_to_hold_a_root`（`roll_back_by_a_forward_publish` 传分配器的根环表）：「根环里有它」那一遍与 F_生效 那一遍都走已知槽重读；F_生效 另把最新根的实例表读不出重读一次（`recovery::effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）。仍读不出报新成员 `RollbackError::CandidateJudgementStillUnreadableAfterOneReread(Box<StillUnreadableAfterOneReread>)`；`model_comparison.rs` 记 `Unexplained`，`history.rs` 的成员名函数加分支。
- `transaction.rs` `PoolWriter::write_system_configuration_slot`：改调 `recovery::effective_rollback_floor_rereading_unreadable_reads_once`（读不出的根槽、最新根读不出的实例表各重读一次，仍读不出照原退路：当没有根 / 不按表滤）。**不拒的理由**（代码注释里也写了）：这个写入口没有分配器的根环表，分不出哪个槽是这个进程知道住着根的；这一写是发布末尾的轮换、取号或先写 F，错误出口只有块设备错；退路不让 F 回落——生效值取系统配置与根上的大者，要覆写的那一槽在这里先读进来，I-7.12 保证根上的 F 不高于它那块盘系统配置里的，只剩「那块盘两槽系统配置与带最高 F 的根同时读不出」这种叠加故障算得低（推的，没造过）。
- 没做：回退里算水位那一遍 `readable_roots`（`mount.rs` `roll_back_by_a_forward_publish` 里「tree_identifier_watermark / inode_number_watermark」那一处）同样把读不出的槽当没有根，规格没点名，没改。

### 3. A3b Q4 成员归族
- `RecoveryFailure` 删 `InstanceTableOfTheNewestRootStillUnreadableAfterOneReread`、`RootRingSlotStillUnreadableAfterOneReread`；`mount::StillUnreadableAfterOneReread` 加 `InstanceTableOfTheNewestRootForTheEffectiveFloor { newest_root_on_the_reread: Option<RollbackTarget> }`、`RootRingSlotKnownToHoldARoot { ring_slot }`，抬 F 经 `MountError::NewerStateStillUnreadableAfterOneReread` 交出；recovery 一侧的错改成两个小类型 `RootRingSlotKnownToHoldARootStillUnreadableAfterOneReread`、`InstanceTableOfTheNewestRootStillUnreadableAfterOneReread` 与枚举 `EffectiveFloorReadingStillUnreadableAfterOneReread`，mount 里 `From` 转。`model_comparison.rs:742` 那个 match 加两臂记 `Unexplained`；`history.rs` 删那两个 `RecoveryFailure` 分支、抽出 `still_unreadable_after_one_reread_member`。`first_transaction_on_device.rs` 对这个枚举没有穷举 match（只有 `NewerStateStillUnreadableAfterOneReread(_)`），不用加。

### 新测试（都在 `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`）
- 改造：`raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_unreadable_is_refused_before_any_write`（原 :954 那条，换成最新根住的已知槽、第 3、4 次读坏）；`a_root_ring_slot_unreadable_once_…_is_named`（传已知槽集，另加「没写过的最后一槽每次都坏 ⇒ 当没有根」）；:901、:1052 两条改期望成员。
- 新加：`a_root_ring_slot_known_to_hold_a_root_read_on_the_one_reread_lets_the_floor_be_raised`、`raising_the_floor_while_a_root_ring_slot_never_written_stays_unreadable_counts_it_as_no_root`、`rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write`、`a_system_configuration_write_rereads_an_unreadable_root_ring_slot_once_before_taking_the_effective_floor`。
- 跑过的：收窄之前（已含回退、PoolWriter 两条新用例）`work/` 上这份文件 18 passed（`logs/src-unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.log`）；`second_transaction_admission_raises_the_floor_before_refusing` 10 passed（P1 那条转绿，`logs/src-second_transaction_admission_raises_the_floor_before_refusing.log`）。**收窄之后两份都没再跑**；src 改动的证红一条都没做（第七节）。

## 四、统一跑一遍（改钉值之前、第一节 src 改完之后；`run-all.sh`，`run1/exit-codes.tsv` 原样）

没跑的目标：`lib-singlefs-checker`（checker 包的测试是重型，子 agent 不跑）、`second_transaction_supplement_three_crash_injection`（重型闸拒：登记的崩溃枚举用例）、`record_checker_judges_absence_by_the_persisted_set`、`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`（规格去掉）、名字带 layer0 的 8 个。

```
build-all-targets 0
lib-singlefs-format 0
lib-singlefs-core 0
lib-singlefs-harness 0
bin-first_transaction_on_device 101
bin-first_transaction_region_bytes 0
bin-e156_allocation_basis_counts 0
bin-e142_first_transaction_write_dump 0
bin-e142_first_transaction_write_dump_one_device 0
bin-e158_root_choice_repair 0
a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is 0
a_floor_raise_refused_for_space_counts_as_short_of_space 101
a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable 0
a_publish_returns_only_after_the_system_configuration_rotation_is_durable 0
a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side 0
acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics 101
acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish 0
admission_checkpoint_cost_per_device_paths 0
admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments 101
checker_cross_links_malformed_nodes_and_mapping_entries 0
checker_judges_reserved_bytes_and_widths_it_used_to_skip 0
checker_known_bad_images 0
checker_narrow_invariants_and_abandoned_roots 0
core_review_geometry_back_chain_and_empty_inode 0
core_review_tree_table_duplicates_and_slot_one_search 0
core_review_unit_area_start_and_publish_limits 0
corrupt_on_disk_content_is_refused_instead_of_panicking 101
crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume 101
crash_enumeration_resumes_from_its_progress_file 0
crash_enumeration_sharded_across_processes 101
crash_image_journal_hint_matches_the_full_ring_scan 101
crash_injection_record_checker_sees_every_publish_across_a_second_crash 0
crash_injection_writable_mount_after_the_crash 0
crash_segments_per_device_and_torn_in_place_overwrites 101
entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration 0
entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write 0
first_transaction_region_bytes 0
first_transaction_step_five_publish 101
first_transaction_step_one_mkfs 0
first_transaction_step_six_recovery 0
first_transaction_step_two_data_unit 0
in_place_overwrite_torn_state_count 101
index_node_header_width_computed_three_ways_agrees_for_every_key_width 0
instance_acquisition 0
model_comparison_judges_the_instance_table_and_allocation_generations_of_a_version_without_a_file 0
parallel_line_one_sequential_write 0
publish_order_matches_litmus 101
rollback_floor_written_into_the_system_configuration_first_and_normal_unmount 101
second_transaction_admission_raises_the_floor_before_refusing 0
second_transaction_allocation_record_tree_geometry_of_writer_and_reader 0
second_transaction_mapping_node_admission 0
second_transaction_parallel_line_one_last_record_flag 0
second_transaction_parallel_line_one_multi_unit_file 0
second_transaction_parallel_line_one_sequential_write 0
second_transaction_parallel_line_three_many_inodes 0
second_transaction_parallel_line_two_mounted_read 0
second_transaction_step_five_reuse 101
second_transaction_step_four_rollback 101
second_transaction_step_one_overwrite 101
second_transaction_step_three_formatted_pool 0
second_transaction_step_three_second_instance 101
second_transaction_step_zero_test_only_switches 0
second_transaction_supplement_one_write_accounting 0
second_transaction_supplement_three_bad_disk_input 0
second_transaction_supplement_three_fault_injection 0
second_transaction_supplement_three_random_history 101
second_transaction_supplement_two_accounting_node_full 0
second_transaction_supplement_two_admission_formula 101
second_transaction_supplement_two_c495_first_file_version_previous_record 0
second_transaction_supplement_two_c519_whole_device_loss_after_warm_up 0
second_transaction_supplement_two_c533_row_publish_record_without_its_root 0
second_transaction_supplement_two_commit_generated_fallback 0
second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version 101
second_transaction_supplement_two_instance_table_chain 0
second_transaction_supplement_two_instance_table_page_full 0
second_transaction_supplement_two_instance_table_second_page_write 0
second_transaction_supplement_two_multi_record_transaction_zero_publishes 0
second_transaction_supplement_two_presumed_clause_checks 0
second_transaction_supplement_two_publish_failure_resent_unchanged 0
second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit 0
second_transaction_supplement_two_record_checker_reuse_legality 0
second_transaction_supplement_two_release_checksum_quarantine 0
second_transaction_supplement_two_reused_record_overlap 0
second_transaction_supplement_two_root_ring_turn_in_one_mount 0
second_transaction_supplement_two_row_publish_admission 0
second_transaction_supplement_two_row_publish_checks_before_acquisition 0
second_transaction_supplement_two_tree_identifier_watermark_crash_orphans 0
second_transaction_supplement_two_tree_nodes_and_the_central_mapping 0
second_transaction_supplement_two_tree_split 0
second_transaction_supplement_two_unequal_devices 0
second_transaction_supplement_two_unreadable_abandoned_root_slot 0
second_transaction_supplement_two_warm_up_counter 101
sparse_device_zero_fill 0
sparse_devices_refuse_requests_outside_the_device_like_the_file_backend 0
system_configuration_mutability_classes 0
system_configuration_per_device_redundancy 0
system_configuration_rollback_floor_and_layout_identity 0
system_configuration_slot_is_overwritten_only_after_a_barrier 0
system_configuration_slots_per_region 0
unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over 0
writable_mount_refuses_a_device_identity_handed_in_twice 0
writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration 0
writable_mount_refuses_caller_parameters_disagreeing_with_the_system_configuration_on_disk 0
zero_unit_publishes_are_compared_with_what_the_row_publish_of_the_same_mount_handed_in 0
ALL-DONE 2026-09-27
```

## 五、21 个红目标逐个：改了什么、验到哪一步

「改后跑过」只有 run2 跑到的两个（`run2/exit-codes.tsv`：`bin-first_transaction_on_device 101`、`bin-e158_root_choice_repair 101`——后者是我叫停时杀掉的，那一次不算结果）。其余「改了」的都是**编得过（edit/ clippy 0）、没跑过**。

| 目标 | 红因（依据） | 处置 | 验到 |
|---|---|---|---|
| bin first_transaction_on_device | ① 边界那条 [23,2,18,33,53]→[23,2,20,35,57]（C577）；②③ 录制流投的 FLUSH 比设备一层少 1 | ① 改钉；②③ 没改（主 agent 定「照今天盘上的数改钉、注明归第五步 55 号」，**没做**） | 改后 15 passed / 2 failed（②③） |
| a_floor_raise_refused_for_space_counts_as_short_of_space | A3b Q1 小盘几何（256 槽档，19/17 次覆盖写的撞墙步数变） | **没改**（探针起了被叫停） | — |
| acquisition_refuses_…_remaining_writer_panics :274 | C577 末尾屏障与重开后取号那道屏障之间没有写，录制器把后者并掉（`lib.rs` `SharedStream::push` 的 trailing-run 去重），设备照收 | 加「数屏障调用」的包装盘：屏障之前拒那条钉 0 次、之后拒那条钉 2 次（保住变异 1295 的判别力） | 没跑 |
| admission_refuses_before_units_…_clustered_segments :180、:424 | A3b Q1 小盘几何（384 槽档段内外空槽对 0/79 变 20/54 等） | **没改** | — |
| corrupt_on_disk_content… :441 | I-9.16 那一判排在水位之前（树表排序报告第八节第 3 条），livelist 改 19 同时违反 I-9.16 | 被改的条目换 deadlist（发号最后、号最大 18），只违反 I-7.8，保住变异 1193 | 没跑 |
| crash_enumeration_findings_log… | C577：快档 54→46 | 常量改 46 | 没跑 |
| crash_enumeration_sharded_across_processes | C577：54→46，金样哈希、计划哈希跟着变 | 常量与注释改了（46、23 片、12 片）；**金样四个值（行数 67、行哈希、进度文件名哈希、内容哈希）没重录** | 没跑，必红 |
| crash_image_journal_hint… :95 | C577：10→11 段 | 段数 11、边界读到的记录数 [0,0,1,1,1,2,2,2,2,3,3,3] | 没跑 |
| crash_segments_per_device… :415、:618 | C577 | 见第六节 | 没跑 |
| first_transaction_step_five_publish :347 | C577 | 见第六节 | 没跑 |
| in_place_overwrite_torn_state_count :317 | C577 | 见第六节 | 没跑 |
| publish_order_matches_litmus :51 | C577：33→35 | 35；litmus 只到根槽，根槽之后「系统配置×2、屏障×2」另钉，collapsed 只取到根槽（litmus/ 没动） | 没跑 |
| rollback_floor_written… :242、:495、:764 | 同 acquisition 那一格（录制器并掉抬 F / 卸载开头那道屏障） | 次序改成今天的，另钉「抬 F 之前录制流最后两步是上一次发布返回前那道屏障」 | 没跑 |
| second_transaction_step_five_reuse :641、:898 | :641 我第一版 src 的「自证不过也重读」误拒；:898 双故障形 | :641 靠 src 收窄；:898 **没改**（用户定改钉「红在 I-7.4 是已知」，没做） | 没跑 |
| second_transaction_step_four_rollback :807 | 同 :641 | 靠 src 收窄 | 没跑 |
| second_transaction_step_one_overwrite :98 | C577 末段多 barrier×2 | 改钉 | 没跑 |
| second_transaction_step_three_second_instance :661 | 见证值 6 > 所选 txg 4，乙拒 | **没改**（规格：改钉「被乙拒、盘上逐字节不变」） | — |
| supplement_three_random_history :1201、seed_4000000204、writable_mount_whose_own_publishes… | :1201 双故障形；另两条 A3b Q1 小盘撞墙步数 | **都没改** | — |
| supplement_two_admission_formula :499、:638 | A3b Q1 小盘几何（−47 槽 → −39 槽等） | **没改** | — |
| supplement_two_fsync_drop… :1556（两条） | 乙在「盘上没有所选那一版」之前先拒；**去掉 C577 那道屏障这两条仍红**（`logs/noc577-second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.log`），不是 C577 引起 | **没改** | — |
| supplement_two_warm_up_counter c366 :231 | 乙拒（见证 comparison Undecidable）；**去掉 C577 那道屏障仍红**（`logs/noc577-second_transaction_supplement_two_warm_up_counter.log`），不是 C577 引起 | **没改**（看不出是哪一次合入引起的，没对 r1 基线） | — |

另：`second_transaction_supplement_three_fault_injection.rs` 与 `e158_root_choice_repair.rs` 的手拼分配器换成 `allocator_after_make_filesystem`（A3b Q8），都没跑；E158 产物要不要重跑归第 5 步 87 号。

## 六、钉值怎么算（C577 那一族；段模型脚本 `calc/model.py`、`calc/streams.py`、`calc/arrays.py`）

算法：C577 之后轮换那 2 写（系统配置槽，原地覆写、取三态）过了发布末尾那道屏障自成一段；只有「上一次轮换 + 下一次发布的 n 个单元写」那种段会拆成 [2] 与 [n]（取号、抬 F、卸载先写系统配置之前本来就有屏障，零单元发布开头没有写，段不变）。每拆一处：两态闭式 (2^(n+2) − 1) → 3 + (2^n − 1)；三态全量 (9·2^n − 1) → 8 + (2^n − 1)；甲二三态 17 → 8 + 1；甲二两态 7 → 3 + 1；段数 +1；写数不变。录制流步数：每次发布末尾多两步屏障，紧跟其后的零单元发布开头那道屏障不再出现（写入口不发 / 录制器并掉）。模型先拿改之前的钉值逐个核对（第一条流 67108885、150994980、29、54；到 C 202113066、84、454426691、159；到 D 206307373、95、463863878、180；到 E 6649413719、14960689284、205、390 与两串按发布分的数；并行线一 5435818083、12230590578），全部对上之后再算新值。

| 文件（新路径见 moved-files） | 处 | 改前 → 改后 | 算式 |
|---|---|---|---|
| first_transaction_step_five_publish | 暖机 / 事务 / 整条 | (18,…,15)→(20,…,15)；(33,…)→(35,…)；(53,"2+2+1+2+2+1+26+2+1+2",67108885)→(57,"2+2+1+2+2+1+2+24+2+1+2",16777240)，三处种类串末段加 barrier×2 | 57 = 2+20+35；1 + (3+3+1+3+3+1+3+3+1+3) + (2^24 − 1) |
| first_transaction_step_seven_layer0（moved） | 段数组、两个常量、快档、按发布分、:763 | [..,26,..]→[2,2,1,2,2,1,2,24,2,1,2]；FULL 2 态 16777240；FULL 16777260；快档 46；两态快档 26；txg3 按发布 16777227 / 快档 13；:763 [..,2,24,3,2] | 1 + 4·8 + 3·3 + 3·1 + (2^24 − 1)；1 + 4·8 + 1 + 9 + 3；txg3 = 8 + (2^24−1) + 3 + 1 |
| second_transaction_step_three_formatted_pool_layer0（moved） | 段数组、段数、闭式、快的那条 | 11 段、16777240、展开段 25、三态 45 | 1 + 7·3 + 3 + (2^24−1)；1 + 4·8 + 3·3 + 3 |
| second_transaction_step_zero_layer0（moved） | 五个脚本的段数组；到 C / 到 D；到 E 的两态全量、三态全量、快档 163/278、两串按发布分；残留记录流段数组与 (53,20)；陈旧 tail 那条改成先找轮换段再 +1 找 28 写单元段 | C：32 段 50724921 / 69 / 50724971 / 119；D：36 段 51773503 / 77 / 51773558 / 132；E：78 段 1662648449 / 1662648564；按发布：一次带单元的普通发布 2^n + 11（n=24:16777227，16:65547，20:1048587，28:268435467），快档 13；写行 txg5、txg15、txg18 不变 | 十四处拆段（n=24 三处、16 四处、20 一处、28 六处）Δ两态 = −Σ(3·2^n − 3)；三态 − 两态 = 23 段 × 5 |
| second_transaction_parallel_line_one_layer0（moved） | 段数组、快的那条、全量 | 19 段；展开段两态 111、三态 141；全量两态 1358954604、三态 1358954634；ignore 理由里的数 | 1 + 9·3 + 5 + (2^24−1)+(2^28−1)+15+(2^30−1)+63；三态 6 个系统配置槽段各 8 |
| second_transaction_step_three_acquisition_barrier_layer0（moved） | 段数组、展开段号 | 28 段；(13,14)→(15,16)；展开的 [2,18] 与状态数不变 | 与第二条流前 28 段逐段相同 |
| tree_split / position_addressed / parallel_line_three_spill_over layer0 | — | **不变，没改** | 这三份只录一次发布（切点取在发布之前、上一次发布返回之后），段是 [单元][记录][根][轮换+屏障]，C577 那道屏障只并进末段；C577 报告第 91 行说它们会变，我读代码判不变（没跑） |
| record_checker_judges_absence_by_the_persisted_set（moved，没跑） | σ | (18,16)→(16,16)；部分落盘那条 64→16 个状态（σ 里没有原地写了）；σ 全量 262144→65536、ignore 理由改 | σ 只剩 16 个单元写：2^0 × 2^4；2^16 |
| in_place_overwrite_torn_state_count | 第一条流 | 67108885→16777240；(150994980,29,54)→(16777260,26,46) | 同第一条流 |
| crash_segments_per_device… | 两处段数组、闭式、快档 | [..,2,24,2,1,2]、漏屏障那条 [..,2,24,5]；16777240 / 26 / 16777260 / 46 | 同上 |
| crash_image_journal_hint | 段数 | 10→11，边界记录数多一个 2 | — |
| second_transaction_step_one_overwrite | 末段种类 | 加 barrier×2 | — |
| findings_log / sharded | 快档 | 54→46；sharded 的片数 27→23、自测 14→12 片 | ⌈46/2⌉、⌈46/4⌉ |

`.claude/gate.d/stage-inputs.tsv`（不在写范围，交主 agent）：c561-sigma-full 那一行注释「2^18」改 2^16（σ 16 个写，65536 个状态）；layer0-tree-split-streams 那一行的 78905428 **不变**（上表理由）；其余 crash-case 行注释里若写着上表的旧数，照上表改（没逐行查）。

## 七、变异表

两份 tsv 在 `patch/`，行内原文我在 `work/`（打完补丁的样子）上逐行核过恰好命中一次（16 行都 = 1）。

- **整行替换（7 行，锚点被这一轮改掉的）**：711（实二，写系统配置那一处换函数名）、1266（C554 乙 模型跟进，model_comparison 那一臂后面多了注释与两个成员）、1332、1333（改锚到 `effective_rollback_floor_rereading_the_newest_instance_table_once` 的「重读仍读不出」那一臂，替换成不按表滤）、1334（改锚到 `…_once_over` 里「重读读出表」那一臂）、1335（改锚到已知槽重读仍读不出那一臂，点名测试改成 `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_unreadable_is_refused_before_any_write`）、1336（锚点仍在、落到 PoolWriter 那一份读根环里，点名测试改成 `a_system_configuration_write_rereads_…_effective_floor`）。**这 7 行一条都没证红**。
- **追加 9 行**：
  - P1 两行（已知槽判定 `if false &&` / `if true ||`）、Q7 三行（回退两处传空集、`transaction.rs` 换回 `crate::recovery::effective_rollback_floor`）、Q4 一行（抬 F 的错改回 `MountError::Recovery`）——**都没证红**，留给下一件或 59 号。
  - 崩溃注入三行（C554 乙那一判关掉，原文 / 替换文照抄已有第 1136 行），点名 `second_transaction_supplement_three_crash_injection` 的三条改钉用例——**三行都证过**，但证在探针上：那份二进制登记了崩溃枚举用例、重型闸不让跑，我把改后的文件去掉登记的快档那一条、拷成 `edit/…/tests/zz_probe_crash_injection.rs` 跑 `prove-red.sh`：`logs/prove-red-probe2.out` 第 1 行「抓到」、`logs/prove-red-probe3.out`「✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了」。先试过用 C577 那道屏障做变异（主 agent 的写法），三条都没红（`logs/prove-red-probe.out`），见第八节。
  - 这三行的 `-p singlefs-harness --test …` 是旧路径；那份文件已被挪进 `singlefs-checker-tier`，合入时跟那边的行一起改包名。
- 新测试的证红（第一节那几条）一条都没做。

## 八、单列：主 agent 定过的两处

### 崩溃注入三条（:600、:881）
- 做了：三条都改钉「这一形不可达」，保留原来的构造。`units_of_a_publish_…` 钉「整条流落盘后恢复落到 (1, 4)、流里只有实例 0 / 1 的发布、记录核对器不红」，扣单元的两次判别力照留（扣 (1, 3) 与 (1, 4)）。另两条删了 `CrashStateLandingOnAVersionRebuiltFromRecords` 那一套，改钉「这段历史里崩溃恢复抛弃根的每一步都被拒成 `…NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`」与「这批崩溃状态里一个那一形都没有」。
- **与主 agent 定的前提不同**：不可达不是 C577 造成的，是 C554 乙。依据有三条：
  - 第一轮快照副本 `base-r1/`（C554 乙、C577 都没进）上，原来三条都绿（`logs/probe-ci-base-r1.log`：3 passed）；
  - 今天的底座去掉 C577 那道屏障，我的三条断言照样绿（`logs/prove-red-probe.out`：三条「没红」）；
  - 关掉 C554 乙那一判：`units_of…` 红；另两条先红不了，是因为 C577 改了切段、快档抽到的崩溃状态跟着变。另两条加了「崩溃恢复那一步被乙拒」这一断言之后也红了。
- 文档注释照实写成 C554 乙。跑都在探针上跑，正式那份文件没跑过（重型闸拒）。

### first_transaction_on_device 两条（:2788 / :3088）
- 现象：两条都在比录制流投出来的 FLUSH 数与设备一层数到的「屏障 + FUA」，差 1（`run1/bin-first_transaction_on_device.log`：`left: 12 right: 13`、`left: 16 right: 17`，每条只打出第一处不等）。
- 原因（读代码）：C577 之后，每次发布以一道池屏障收尾。下一步（抬 F 先写系统配置、重开之后取号）起的是一个新的 `PoolWriter`，开头再发一道屏障。两道之间没有写，录制器 `SharedStream::push`（`crates/singlefs-harness/src/lib.rs`）遇到「同一块盘已经在末尾一串屏障里」就不记，把后一道并掉了。注入包装那一层照收，所以设备多数到一次 FLUSH。rollback_floor 与 acquisition 两份的红，是同一个机制在录制流上的样子。
- **没做**：主 agent 定的「照今天盘上的数改钉、注明归第五步 55 号六档重录时定」。这两条今天仍红。

## 九、没做什么（下一件接着做的清单）

- **没跑到的目标**：改完钉值之后，除 `bin first_transaction_on_device` 外，第五节表里标「没跑」的目标都没复跑。第三节新测试收窄 src 之后也没复跑。`bin e158_root_choice_repair` 那一次被我叫停时杀掉，没有结果。另外：
  - 规格点名要跑的 `bin e158_root_choice_repair` 单测，改后一次都没跑成；
  - `second_transaction_supplement_three_fault_injection`（手拼分配器换掉之后）没跑；
  - `second_transaction_admission_raises_the_floor_before_refusing`、`unit_area_start_follows_the_ring…` 在收窄之后没跑。
- **没改的钉值**：
  - `a_floor_raise_refused_for_space_counts_as_short_of_space`（两条）、`admission_refuses_before_units_…_clustered_segments`（:180、:424）、`second_transaction_supplement_two_admission_formula`（:499、:638）、`second_transaction_supplement_three_random_history` 的 `seed_4000000204_…` 与 `writable_mount_whose_own_publishes_find_no_placement_…`：都是 A3b Q1 小盘几何，要按新几何量撞墙步数。小盘探针 `zz_probe_small_disks.rs` 写好了，没跑完，已随 `edit/` 删掉。
  - `random_history` :1201 与 `step_five_reuse` :898：双故障，用户定改钉，没做。
  - `step_three_second_instance` :661：改钉被乙拒，没做。
  - `fsync_drop…` 两条与 `warm_up_counter` c366：乙先拒，去掉 C577 屏障仍红，没对 r1 基线定因。
  - `first_transaction_on_device` 两条：主 agent 定的改钉没做。
  - `crash_enumeration_sharded_across_processes`：金样 4 个值没重录，必红。
- **没证红的变异行**：第七节替换 7 行、追加里的 P1 两行、Q7 三行、Q4 一行。
- **没核的**：`apply-writer-patch.py --dry-run` 没跑（crates-moved.patch 的路径已不在主工作区，那个脚本认不认我不知道）。登记给我的门禁阶段（stage-owners.tsv 里 implementation-writer 那几道，含 74 号）一道都没跑。`cargo build --all-targets` 没单独跑（`edit/` 上的 clippy 编过全部目标）。
- 单列 D16 那一处取舍（第三节 1）：「自证不过」要不要接进已知槽重读，交主 agent。
- 规格第二节「其余跑出来的红，看不出关系的先查 r1 基线」只对崩溃注入三条做了。
- 层 0、QEMU、herd7、crates 变异整表归 crash-verifier，没跑；没走三方对抗；没提交。

## 十、删了的副本与编译目录

- `/tmp/claude-1000/impl-merge-verify-1/work`（改后副本，含 target）17G，删了；
- `/tmp/claude-1000/impl-merge-verify-1/edit`（静态改 + 探针副本，含 target）3.4G，删了；
- `/tmp/claude-1000/impl-merge-verify-1/noc577`（去掉 C577 屏障的对照副本）2.4G，删了；
- `/tmp/claude-1000/impl-merge-verify-1/base-r1`（第一轮快照基线副本）1.7G，删了。
- 留着的都不是副本：`patch/`（交付物）、`report.md`、`progress.md`、`spec.md`、`run-all.sh`、`run-2.sh`、`run1/`、`run2/`、`logs/`、`calc/`（段模型脚本）。它们是报告材料。
- 叫停时我停了自己起的 run-2、noc577、小盘探针三条链，从最底层逐个 `proc.py stop`，事后 `ps` 复查一个不剩。prove-red 那条链是自己跑完的。
