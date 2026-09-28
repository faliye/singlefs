# C331 在今天 crates/ 上的复查（调查员，2026-09-28）

## 0. 现场

- 源：主工作区 HEAD e5253e8a 加工作区未提交改动，`rsync -a --exclude target --exclude .git` 拷到 `/tmp/claude-1000/closeout-recheck-2026-09-28/c331/repo`。
  拷时主工作区 `git diff HEAD -- crates | sha256sum` = `18c16f0c0754ea4c7e9340f2a37ec69b2c136d46212973a0ed18de0f95632a66`。
- 编译目录 `/tmp/claude-1000/closeout-recheck-2026-09-28/c331/target`；每条 cargo 经 `capped.sh 10` 与 `run-with-memory-cap.sh 12G`。
- 开跑前 `ps` 看负载：没有 qemu / vm-bench / e152 / fio，也没有别的 cargo；只有一条 `ask-local.sh`（本地模型提问）。

### 基线：两份已有用例在副本上原样跑

命令：
`CARGO_TARGET_DIR=…/c331/target nice -n 19 bash research/scripts/capped.sh 10 bash research/scripts/run-with-memory-cap.sh 12G cargo test -p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable --test unreadable_abandoned_root_slot`

原样输出（`grep -E '^test |test result|exit='`）：
```
test the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread ... ok
test a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread ... ok
test a_witnessed_newest_root_whose_reads_fail_only_once_is_chosen_by_the_immediate_reread_of_the_product_path ... ok
test a_witnessed_newest_root_that_reads_back_zeros_until_just_before_the_one_reread_is_chosen_because_zeros_stay_out_of_the_read_cache ... ok
test a_witnessed_newest_root_still_unreadable_after_the_one_reread_refuses_the_writable_mount_before_any_write_while_the_read_only_mount_reads_the_version_before_it ... ok
test a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_is_chosen_and_nothing_is_abandoned ... ok
test without_the_last_record_of_the_selected_version_the_record_at_the_witnessed_counter_decides_and_without_both_the_mount_counts_it_as_witnessed ... ok
test a_newest_instance_table_unreadable_when_the_shadow_ledger_reads_it_is_reread_once_then_refuses_writable_instead_of_turning_the_isolation_off ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.23s
test slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring ... ok
test the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot ... ok
test an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.78s
exit=0
```
这两份用例都是单实例内（或崩溃恢复抛弃根）的形，没有一条走完 C331 那段「新实例发布、写被确认 → 旧根又读得出 → 再重开」的历史，下面自己造。

## 1. 跑前按代码推的预测（写在跑之前，跑后逐条对）

读的代码：`crates/singlefs-core/src/mount.rs` 的 `newer_publish_witness`、`read_stage_settled_at_most_on_the_one_reread`、`first_txg_of_new_instance`，
`crates/singlefs-core/src/transaction.rs` 的 `write_system_configuration_slot`、`write_acquired_instance`，`crates/singlefs-core/src/root_ring.rs` 的 `target_for_publish`。
共同历史：实例 1 暖机 txg 1、2，A txg 3；重开实例 2 写行 txg 4、暖机 txg 5、B txg 6、C txg 7，全部确认返回。

| 场景 | 造法 | 预测 |
|---|---|---|
| S1 | 实例 2 全部根槽（txg 4–7）与记录（jsn 4–7 两份）持续读不出 | 见证 7 > A 的末条 3，重读仍判真，拒可写；只读落到 A；撤故障后可写挂载择 C |
| S2 | 只有根槽读不出，记录读得出 | 重放把 4–7 施加上，所选那一版就是 C，照常挂载 |
| S3 | S1 的故障每段只坏第一次读（产品路径立即重读） | 重读择 C；实例 3 写 E 确认；撤故障重开择 E |
| S4 | C 之后实例 2 再发 D（txg 8），D 的根 FUA 之后系统配置轮换写失败（没确认），崩溃；D 的根与记录读不出 | 见证 7 = C 的末条，判假不重读，从 C 可写挂载；实例 3 写行 txg 8 落在 D 的根槽上盖掉 D；E 活下来 |
| S5 | 两块盘见证 C 的那一槽（世代号最大）持续读不出，C 的根与记录读不出 | 见证退到 6 = B 的末条，判假，从 B 可写挂载：C（已确认）丢；实例 3 写行 txg 7 盖掉 C 的根槽，E 活下来 |
| S6 | 四个系统配置槽只在读阶段判 N-配置 那一次读坏，实例 2 全部根与记录读不出 | 见证读成 0（NothingWitnessed），判假，从 A 可写挂载；实例 3 写行 4、暖机 5、E 6，C（txg 7）的根槽留着；撤故障重开择 C，E 被压过（C331 本身），可写挂载不拒 |
| S7 | 盘 1 两槽系统配置持续读不出，其余同 S1 | 盘 0 仍见证 7，拒可写（或更早在取号那一核拒） |
| S8 | S1 的根与记录第一次读坏、重读读得出、之后每次又坏 | 重读择 C，之后直接读盘的几处（`highest_root_txg`、影子账择根）看不见实例 2 的根；首个 txg 由记录定为 8，E 活下来 |

什么会推翻「C331 在今天被 乙-配置续 挡住」：S1 可写挂载做成，或 S3 / S4 / S5 最后一次挂载择的不是 E。
什么会推翻 S6 的预测：S6 的读阶段见证不是 0（我数错了读的次数），或最后一次择到的是 E。

## 2. 结论

### 问题 1：C331 那段历史今天的结局

照题面造（较新实例的全部根在重开时读不出），今天的结局取决于读不出持续多久。两种情况下新实例已确认的写都没有被压过：
- **读不出一直持续**：拒可写（`NewerStateStillUnreadableAfterOneReread`，见证 7 大于所选 A 的末条 3，重读一次仍然判真）。新实例一个字节都没写，没有已确认的写可以被压。撤掉故障之后可写挂载择 C。只把根藏起来、记录读得出，结局也是拒：重放不跨实例，所选那一版仍是 A。
- **只坏一次，或者只有重读那一次读得出**：重读择到 C。新实例写 E、确认返回，撤故障重开后择 E。

所以只要系统配置槽读得出，C331 的原形今天挡得住。

### 问题 2：乙-配置续 罩不住的变体（逐个造过）

| # | 变体 | 结局 | 丢的是谁的已确认写 |
|---|---|---|---|
| V1 | 四个系统配置槽只在第一遍判 N-配置 那一次见证读上坏（第 3 次读），实例 2 的全部根与记录读不出 | 见证读成 0，判 `NothingWitnessed`、判假，不重读，从 A 可写挂载；实例 3 写 E（txg 6）确认返回；撤故障重开时择 C（2, 7），**E 被压过**。只读挂载择 C、读不回文件；可写挂载报 `UnitUnreadable { slot: 50304 }`（实例 3 写行的实例表复用了 C 的实例表那一片） | 新实例（C331 本身） |
| V2 | 同上，但坏的是重读那一遍的见证读（第 4 次）：第一遍读到见证 7、判真 | 重读那一遍所选的仍是 A，只是见证读成 0，判假，按重读那一遍往下走。结局同 V1：**E 被压过** | 新实例（C331 本身） |
| V3 | 两块盘见证最新发布 C 的那一槽持续读不出，C 的根与记录读不出 | 见证退到 6，等于 B 的末条，判假，从 B 可写挂载；实例 3 写行 txg 7 盖掉 C 的根槽；E 活下来 | 较旧实例（C 丢，挂载不拒） |
| V4 | 两次故障不同时发生：第 k 次挂载时两块盘见证 C 的那一槽读不出，取号写落回那一槽、写进 tail 6，写行之前崩溃；第 k + 1 次挂载时 C 的根与记录读不出 | 崩溃后四个槽 tail 全是 6，见证 7 被抹掉；第 k + 1 次从 B 可写挂载，C 丢；E 活下来 | 较旧实例 |
| — | 系统配置没见证过的最新根（D 的根 FUA 之后轮换写失败、没确认，崩溃；D 的根与记录读不出） | 见证 7 等于 C 的末条，判假，从 C 可写挂载；实例 3 写行 txg 8 落在 D 的根槽上把 D 盖掉；E 活下来 | 无（D 没确认过） |
| — | 两块盘只坏一块：盘 1 两槽系统配置持续读不出，实例 2 的全部根与记录读不出 | 盘 0 仍见证 7，拒可写 | 无 |
| — | 根与记录只在重读那一次读得出、之后又坏 | 重读择 C；之后几处直接读盘的地方看不见实例 2 的根，首个 txg 由重读那一遍的记录定为 8；E 活下来 | 无 |

V1、V2 就是 C331 本身：较新实例的全部根读不出，新实例从更旧的根往上数 txg，写被确认，之后那些根又读得出，按 txg 更高被择中。V3、V4 丢的是较旧实例的写，是 C554 那一族。V3 与仓里已有用例 `a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread` 钉的是同一个结局（那条用例是把槽清零，这里是读报错）。

适用范围：以上都只在这一组参数上造过：两块盘，区域归属 [0, 1, 0]，每次覆盖写一条记录，历史是实例 1 → 实例 2 → 实例 3。V1、V2 要求四个槽恰好在见证读那一次同时读不出。这种故障在真盘上有多常见，我没量过，也没有依据估。

## 3. 机理（文件:行号，行号取自主工作区现在的文件）

- `crates/singlefs-core/src/mount.rs:4146` 起的 `witnessed_journal_counter` 那一段：每块盘两槽用 `verified_system_configuration_slots` 读，**读不出或自证不过的槽直接从 max 里消失**，4159 行 `.unwrap_or(0)`。4160–4161 行 c_见证 = 0 时判 `NothingWitnessed`。386 行 `NothingWitnessed => false`。「一个槽都读不出」和「mkfs 之后从没见证过」在这里得到同一个读数，而判不出的时候并不按真处理（同一个函数对记录读不出的那一支是按真的：`Undecidable => true`）。V1、V3 由此成立。
- `crates/singlefs-core/src/mount.rs:4249–4251`：重读那一遍 `read_stage` 重新读系统配置槽，只拿重读那一遍的读数判。重读那一遍的见证比第一遍低，也照样接受。这里不要求重读那一遍所选的版本比第一遍新，也不拿两遍的见证取大。V2 由此成立。
- `crates/singlefs-core/src/transaction.rs:361`（`slot_generation` = 这块盘读得出的最大世代号 + 1）、409 行（落槽 = 世代号 mod 2）、846 行（取号写的 tail = 读得出的槽的最大 tail）：见证最新发布的那一槽读不出时，取号写正好落回那一槽，写进较旧的 tail。V4 由此成立。
- V1 里最后那次重开也不拒，是因为另一处：新实例的 `next_counter`（`crates/singlefs-core/src/mount.rs:4335`，环里读得出的最大计数器 + 1）没有和见证值比。实例 3 的记录从 jsn 4 起写，每次轮换写这次发布的计数器（`crates/singlefs-core/src/transaction.rs:1324`、`1872`）。最后盘上 tail 只有 6、5（`final system configuration (device, generation, tail): [(0, 14, 6), (0, 13, 5), (1, 14, 6), (1, 13, 5)]`），低于 C 的末条 7，所以那一次读阶段判假、放行 C。
- 覆盖面：根环落点只由 txg 定（`crates/singlefs-core/src/root_ring.rs:109` 起的 `target_for_publish`）。新实例首个 txg 取 max(直接读根环, 记录) + 1（`crates/singlefs-core/src/mount.rs:1090` 起的 `first_txg_of_new_instance`）。所以被藏的根只有 txg 高过新实例已发的全部发布，才会在之后压过它。V1、V2 里实例 3 发了 4、5、6 三次，C 在 7，所以留了下来。被藏的根少于等于新实例的发布数时，它们的根槽会被新实例逐个盖掉（V3、「没见证过的最新根」那一行都是这样）。

## 4. 最小复现（V1、V2，C331 本身）

用例文件 `c331_newer_instance_roots_unreadable_then_readable_again.rs` 要连同 `common/mod.rs` 的两行读窗口变体一起放进 `crates/singlefs-harness/tests/`（见第 8 段）。命令在仓根跑：

`CARGO_TARGET_DIR=<编译目录> TMPDIR=<草稿> nice -n 19 bash research/scripts/capped.sh 10 bash research/scripts/run-with-memory-cap.sh 12G cargo test -p singlefs-harness --test c331_newer_instance_roots_unreadable_then_readable_again every_system_configuration_slot_unreadable_on_the_ -- --nocapture --test-threads 1`

原样输出（去掉了 cargo 的 Compiling / Finished / Running 行和空行、每块盘系统配置的打印行，其余一字未改）：
```
running 2 tests
test every_system_configuration_slot_unreadable_on_the_first_witness_read_lets_a_hidden_root_override_the_new_instance_acknowledged_write ... history: A=(1,3,jsn 3) second-instance mount txgs=[CheckpointTxg(4), CheckpointTxg(5)] B=(2,6,jsn 6) C=(2,7,jsn 7)
C data unit locations: [(0, 50184), (1, 50184)]
C root tree table / instance table: [(0, 50348), (1, 50348)] / [(0, 50304), (1, 50304)]
new-instance mount: Ok: instance 3 chosen (1,3) effective (1,3) row txg 4 warm-ups [5] rereads RereadsOfThisMount { read_stage: OnTheFirstRead { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 0, comparison: NothingWitnessed } } }, instance_table_of_the_newest_root: OnTheFirstRead } isolated [(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_unreadable 0
E = (3,6) jsn 6
final system configuration (device, generation, tail): [(0, 14, 6), (0, 13, 5), (1, 14, 6), (1, 13, 5)]
final roots [(0, 0), (1, 1), (1, 2), (1, 3), (3, 4), (3, 5), (3, 6), (2, 7)]; final read-only Some((2, 7, false)); final writable: Err: Recovery(UnitUnreadable { slot: SlotNumber(50304) })
E data unit locations: [(0, 50182), (1, 50182)]
E root tree table / instance table: [(0, 50334), (1, 50334)] / [(0, 50304), (1, 50304)]
ok
test every_system_configuration_slot_unreadable_on_the_reread_witness_read_flips_the_judgement_and_lets_a_hidden_root_override_the_new_instance_write ... history: A=(1,3,jsn 3) second-instance mount txgs=[CheckpointTxg(4), CheckpointTxg(5)] B=(2,6,jsn 6) C=(2,7,jsn 7)
C data unit locations: [(0, 50184), (1, 50184)]
C root tree table / instance table: [(0, 50348), (1, 50348)] / [(0, 50304), (1, 50304)]
new-instance mount: Ok: instance 3 chosen (1,3) effective (1,3) row txg 4 warm-ups [5] rereads RereadsOfThisMount { read_stage: OnTheOneReread { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 7, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 3 } } }, reread: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 0, comparison: NothingWitnessed } } }, instance_table_of_the_newest_root: OnTheFirstRead } isolated [(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_unreadable 0
E = (3,6) jsn 6
final system configuration (device, generation, tail): [(0, 14, 6), (0, 13, 5), (1, 14, 6), (1, 13, 5)]
final roots [(0, 0), (1, 1), (1, 2), (1, 3), (3, 4), (3, 5), (3, 6), (2, 7)]; final read-only Some((2, 7, false)); final writable: Err: Recovery(UnitUnreadable { slot: SlotNumber(50304) })
E data unit locations: [(0, 50182), (1, 50182)]
E root tree table / instance table: [(0, 50334), (1, 50334)] / [(0, 50304), (1, 50304)]
ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 26.65s
```
怎么读这段输出：`new-instance mount` 那一行里，实例 3 从 A (1, 3) 可写挂载，见证读成 `witnessed_journal_counter: 0, comparison: NothingWitnessed`（V1），或者第一遍读到 7、重读那一遍读成 0（V2）。`E = (3,6)` 是确认返回的写。`final roots` 里 (2, 7) 还在、高于 (3, 6)。撤故障重开时只读择 (2, 7)，读不回 E。可写挂载报 `UnitUnreadable { slot: SlotNumber(50304) }`，而 50304 就是 C 的根和 E 的根共同指着的实例表那一片（`C root tree table / instance table` 与 `E root tree table / instance table` 两行）。

## 5. 推翻条件，以及我在副本里造出来的那一次

**定位要是错的，本该看到什么**：假如 V1、V2 的起因不是「见证读漏掉读不出的槽」，而是别的东西（重放、择根、首个 txg、影子账），那么把坏的那一次读挪出读阶段（第 5 次，读阶段已判完），结局应该不变。反过来，假如定位是对的，挪出去之后两遍见证都读到 7，挂载就该拒。

造的那一次：用例 `every_system_configuration_slot_unreadable_only_after_the_read_stage_still_refuses`，故障与 V1 完全相同，只把 `OnlyTheNth(3)` 换成 `OnlyTheNth(5)`。原样输出里的挂载行（`s6-final.log`）：
```
new-instance mount: Err: NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 7, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 3 } } }, reread: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 7, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 3 } } } })
```
结局是拒可写。所以只挪了见证读那一次，结局就翻了。

第几次读是谁读的，在副本里给每次读系统配置槽打了调用栈（盘 0 槽 0 的前 5 次，`s6-trace2.log` 原样）：
```
TRACE config read dev 0 off 0 #1: ["singlefs_core::recovery::choose_system_configuration", "singlefs_core::mount::mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread", "singlefs_core::mount::mount_writable_with_test_only_switches"]
TRACE config read dev 0 off 0 #2: ["singlefs_core::mount::own_device_numbers_on_disk_differing_from_the_identities_handed_in", "singlefs_core::mount::device_table_disagreeing_with", "singlefs_core::mount::parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table"]
TRACE config read dev 0 off 0 #3: ["singlefs_core::mount::newer_publish_witness", "singlefs_core::mount::newer_publish_witness", "singlefs_core::mount::read_stage"]
TRACE config read dev 0 off 0 #4: ["singlefs_core::mount::newer_publish_witness", "singlefs_core::mount::newer_publish_witness", "singlefs_core::mount::read_stage"]
TRACE config read dev 0 off 0 #5: ["singlefs_core::recovery::effective_rollback_floor_of_the_roots_read", "singlefs_core::recovery::effective_rollback_floor_of_the_roots_read", "singlefs_core::recovery::effective_rollback_floor_under_the_newest_roots_table"]
```

**第二条独立的路：在副本里改源码**。改的是 `crates/singlefs-core/src/mount.rs` 的 `newer_publish_witness`：四个槽里有一个读不出，比较一支就判 `Undecidable`（即按真），别的不动。整份用例文件在改过的副本上跑（`localize.log` 原样）：
```
test a_newest_root_left_unwitnessed_by_a_failed_rotation_is_overwritten_by_the_new_instance_row_publish ... ok
test every_system_configuration_slot_unreadable_only_after_the_read_stage_still_refuses ... ok
test one_device_with_both_system_configuration_slots_unreadable_still_witnesses_the_newer_instance_and_refuses ... FAILED
test c331_history_with_only_the_roots_unreadable_also_refuses_because_the_replay_stays_on_the_chosen_instance ... ok
test roots_and_records_readable_only_on_the_reread_are_chosen_and_the_new_instance_write_survives ... ok
test every_system_configuration_slot_unreadable_on_the_reread_witness_read_flips_the_judgement_and_lets_a_hidden_root_override_the_new_instance_write ... FAILED
test c331_history_with_a_one_read_fault_rereads_into_the_newest_root_and_the_new_instance_write_survives ... ok
test every_system_configuration_slot_unreadable_on_the_first_witness_read_lets_a_hidden_root_override_the_new_instance_acknowledged_write ... FAILED
test c331_history_refuses_the_writable_mount_while_every_root_and_record_of_the_newer_instance_stays_unreadable ... ok
test the_newest_system_configuration_slot_unreadable_on_both_devices_loses_the_older_instance_acknowledged_publish_but_not_the_new_instance_write ... FAILED
test result: FAILED. 6 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.80s
```
翻面的 4 条都属于「有系统配置槽读不出」这一类：V1、V2、V3 从做成挂载变成拒可写；盘 1 两槽坏那条仍然拒，只是拒因里的比较一支变成了 `comparison: Undecidable`，而断言是按原文比的，所以判红。其余 6 条（与系统配置槽无关的）一条没动。跑完我用 `cp` 把原文件拷回去，`cmp` 与主工作区的 `mount.rs` 逐字节相同，之后又整份跑了两遍，都是 `test result: ok. 10 passed`（`final-rerun1.log`、`final-rerun2.log`）。这次改动只用来定位，不是修法提议。

**校验路子本身会不会红**：会。上面那组用例在改过的副本上变红，说明断言分得出「拒」和「做成」。反过来，定稿用例在原代码上 11 条全绿，其中 V1、V2 断言的是「E 确认返回之后，最后择到的是 (2, 7)、不是 E」。这条断言成立，说明问题确实在今天的代码上出现了，不是用例没走到那条路径：挂载行里的 `OnTheFirstRead` / `OnTheOneReread` 与 `witnessed_journal_counter` 就是那条路径跑过的读数。

## 6. 排除掉的解释

| 解释 | 凭哪条观测排除 |
|---|---|
| 新实例其实择到了 C，E 是后来被别的写盖掉的 | V1 挂载行 `chosen (1,3) effective (1,3) row txg 4`；`final roots` 里 (3, 4)、(3, 5)、(3, 6) 都在，E 自己的根还在环里，最后是择新选了 txg 更高的 (2, 7) |
| 重放把实例 2 的记录施加了 | 只藏根、不藏记录的那条用例同样拒可写，所选那一版仍是 (1, 3)，读数 `selected_version_last_record_counter: 3`。重放不跨到实例 2 |
| 首个 txg 取错了（没看记录） | 「只有重读那一次读得出」那条：记录进了读缓存，写行 txg 是 8，不是 4 |
| 我数错了读的次数，V1 实际是别的读坏了 | 调用栈：第 3、4 次读都在 `singlefs_core::mount::newer_publish_witness` 里；挪到第 5 次（`effective_rollback_floor_of_the_roots_read`，读阶段之后）就变回拒 |
| V1 最后可写挂载报错，是因为读故障没撤干净 | 最后那次重开前 `unreadable.lift()` 已撤，而且是冷重开新句柄。报错的槽 50304 是 C 与 E 共用的实例表那一片，是写出来的冲突，不是读故障 |
| 只读挂载报的 (2, 7) 是用例读错了长度 | 只读看 E 是按 E 的长度（3700）读的。(2, 7) 是 `read_only.effective_root` 自己报的，与读多长无关 |

## 7. 跑前预测对照（第 1 段那张表）

| 场景 | 预测 | 实测 | 对上没有 |
|---|---|---|---|
| S1 全部根与记录持续读不出 | 拒可写，撤故障后择 C | 拒可写（见证 7 对 A 的末条 3），撤故障后择 (2, 7) | 对 |
| S2 只有根读不出 | 重放到 C，照常挂载 | **拒可写**：重放不跨实例，所选那一版仍是 A | **错**。我当时以为重放会把实例 2 的记录施加上 |
| S3 只坏第一次读 | 重读择 C，E 活 | 重读择 (2, 7)，E = (3, 11)，最后择 E | 对 |
| S4 没见证过的最新根 D | 写行盖掉 D，E 活 | 写行 txg 8 落在 D 的根槽上，(2, 8) 不在了，(3, 8) 在；最后择 (3, 11) | 对 |
| S5 两块盘最新那一槽读不出 | C 丢，E 活 | 从 (2, 6) 挂载，E = (3, 9) 活 | 对（即 V3） |
| S6 只坏见证读那一次 | E 被压过 | 第 2 次读不是见证读（是核盘表本盘设备号那一读），第一次跑见证仍读到 7、拒可写。改成第 3 次、第 4 次后 E 被压过 | 结局对，**读的次数数错了**，按调用栈改正 |
| S7 一块盘两槽读不出 | 拒 | 拒 | 对 |
| S8 只有重读那次读得出 | E 活，首个 txg 8 | 写行 txg 8，最后择 (3, 11) | 对 |
| （跑后补）V4 取号抹掉见证 | 按代码推：C 丢，E 活 | 崩溃后 tail 全是 6，C 丢，E = (4, 9) 活 | 对 |

## 8. 用例与辅助改动（交主 agent 的草稿，不在仓里）

- 用例：`/tmp/claude-1000/closeout-recheck-2026-09-28/c331/c331_newer_instance_roots_unreadable_then_readable_again.rs`，988 行，sha256 `0db170f1abee8e8ca0510b16480413a85b3075cc3fa6051e8d98211d9027add2`。11 条，一条一个场景：
  - 挡住的形：`c331_history_refuses_the_writable_mount_while_every_root_and_record_of_the_newer_instance_stays_unreadable`、`c331_history_with_only_the_roots_unreadable_also_refuses_because_the_replay_stays_on_the_chosen_instance`、`c331_history_with_a_one_read_fault_rereads_into_the_newest_root_and_the_new_instance_write_survives`、`roots_and_records_readable_only_on_the_reread_are_chosen_and_the_new_instance_write_survives`、`one_device_with_both_system_configuration_slots_unreadable_still_witnesses_the_newer_instance_and_refuses`、`a_newest_root_left_unwitnessed_by_a_failed_rotation_is_overwritten_by_the_new_instance_row_publish`、`every_system_configuration_slot_unreadable_only_after_the_read_stage_still_refuses`（推翻用例）；
  - 罩不住的形（断言钉的是**今天的坏结局**，修好之后应该变红）：V1 `every_system_configuration_slot_unreadable_on_the_first_witness_read_lets_a_hidden_root_override_the_new_instance_acknowledged_write`、V2 `every_system_configuration_slot_unreadable_on_the_reread_witness_read_flips_the_judgement_and_lets_a_hidden_root_override_the_new_instance_write`、V3 `the_newest_system_configuration_slot_unreadable_on_both_devices_loses_the_older_instance_acknowledged_publish_but_not_the_new_instance_write`、V4 `an_acquisition_that_cannot_read_the_newest_system_configuration_slots_overwrites_them_with_an_older_tail_and_a_later_mount_loses_the_acknowledged_publish`。
- `common/mod.rs` 的改动：`/tmp/claude-1000/closeout-recheck-2026-09-28/c331/common-mod.rs.diff`，sha256 `1d8c3c4df52392659ad633b8414c47bb2a013f1933ccefa1a6125e864d025f66`。只加了 `FailingReadsOfARange::OnlyTheNth`、`AllButTheNth` 两个变体和它们的判法。
- 定稿在副本上整份跑的原样结果（`final-full.log`）：`test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.13s`。10 条时的版本另外重跑过两遍，都是 10 passed。这些用例是确定性的，不含随机源，所以跑几遍只能说明没有隐藏状态，不算统计上的稳定。
- 用例里留着 `eprintln!` 诊断打印；`rustfmt` 过了；`cargo clippy -p singlefs-harness --test …` 报 1 条 warning（`&Vec` 可以写成 `&[_]`），没改。文件名与测试名没过门禁 13、14 号，因为门禁不归我跑。
- 草稿目录里留着的日志：`baseline.log`、`run1.log`（探索版）、`s6-n{1,3,4}.log`、`s6-trace.log`、`s6-trace2.log`、`s6-final.log`、`s6-units.log`、`s6-tails.log`、`localize.log`、`final1.log`、`final2.log`、`final-rerun{1,2}.log`、`final-full.log`、`acq.log`、`minimal.log`、`exploratory-test.rs`（探索版用例）、`mount.rs.orig`（主工作区 `mount.rs` 的原样拷贝，定位时拿来复原）。都没进 `research/results/`，因为写仓不归我，要不要入库由主 agent 定。

## 9. 没做什么

- 没修，也没判该怎么修。第 5 段那处源码改动只用来定位，已经复原。它罩不罩得住 V4、会不会误拒只做过 mkfs 的池，我都没核。
- 重型测试一条没跑：没跑 checker 档（`singlefs-checker-tier`）、层 0、54 / 55 / 57 / 59 / 87 号，也没跑全量 cargo test。只跑了 harness 档的两个目标：已有的两份（基线）与新用例这一份。
- 没用池级 checker（`check_pool_image`）判 V1 结束时的镜像。V1 里 C 的单元被复用，I-7.4 那一类会不会红，我没看。
- 没拿 E158 第 4 次跑的装置对照。「-配置续 各臂丢写为 0」那句说的是装置副本，这里的 V1、V2 是不是在它的故障空间之外，我没查。
- 没扫参数：只用了两块盘、区域 [0, 1, 0]、每次发布一条记录这一组。V1、V2 需要的被藏根个数（要多于新实例挂载与写 E 的发布数）会随写行之后的暖机次数变，我只量了「实例 2 藏 4 条、实例 3 发 3 次」这一格。
- `OnlyTheNth(1)`（第 1 次读就坏，即 `choose_system_configuration` 那一读）那一格结局是 `Recovery(NoValidSystemConfiguration { first_device_with_no_valid_system_configuration_slot: DeviceIdentity(0) })`，挂载失败（`s6-n1.log`）。没为它写成用例。
- 故障只造了「读报块设备错」这一种，没造「读回全 0」（`UnreadableRangeReadBack::Zeros`）。
- 没改主工作区任何文件，副本里的改动没有带回主工作区。
- 删了：仓副本 `/tmp/claude-1000/closeout-recheck-2026-09-28/c331/repo`（344M）、编译目录 `/tmp/claude-1000/closeout-recheck-2026-09-28/c331/target`（11G）、用例镜像临时目录 `/tmp/claude-1000/closeout-recheck-2026-09-28/c331/tmp`（4.0K，空）。草稿目录里其余文件是日志、用例草稿与 diff，留给主 agent 核。
