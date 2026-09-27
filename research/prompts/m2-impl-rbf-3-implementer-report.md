# 实三交回：管理员回退改成挂着时的一次向前发布，删回退见证、回退行与截断

实现员（implementation-writer），2026-09-26 JST 08:22 开工，JST 11:09 交回。改动都在主工作区，没有提交、没有任何 git 写操作。
开工时把 `crates/`、`litmus/`、`Cargo.*` 拷成快照 `/tmp/claude-1000/impl-rbf-3/pristine/`（含实一、实二没提交的改动），下文「基线」都指它。

## 一、结论先说

- 七件都做了：挂载时回退删掉；`mount::roll_back_by_a_forward_publish` 在挂着的时候发一次普通发布；回退见证、回退行（flags bit0 退役）、重放遇回退行截断、前缀第五条删掉；「4 个不同状态」按 (inode 树根, extent 树根) 去重，入口、checker、理想模型三处；实二交回两件（树表 0 条的一版上卸载不写、照常做成；F_生效根那一半照 D16 原文）；checker 删 I-7.10、I-7.11（45 条）；C558 判了可达性。
- 被抛弃的根今天只由崩溃恢复造出。旧的影子账用例原先靠挂载时回退造被抛弃的根；改成 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：它按第一轮判决 H6 的做法，让 C 的根槽与数据单元暂时读不出、恢复落到前一条根、再写回。影子账相关的钉死值跟着变（40 → 14 等，见第七节）。
- C558：在这两条路上走不到。两块盘，回退目标取环里最旧的 B (1, 4)，回退那次发布 D（txg 28）的根槽写就是 B 那一槽，根槽写三种失败都试了：
  - 进程不崩：D 冻结；同一条回退在任何读写之前拒；原样重发之后读回 B。
  - 进程在重发之前崩：D 的记录已持久，恢复由记录重建 D、读回 B。
  用例留作证据。它顺带量到：崩在重发之前、B 的根槽被写坏的那两格，恢复之前的镜像上 checker 判 I-3.1 红；普通覆盖写在同一格上逐字相同，与回退无关（第十节设计问题 2）。
- 回退那次发布的记录事务号取 0、不推进实例内计数。条款没写，交主 agent 定（第十节设计问题 1）。
- 验证：
  - 动到的测试二进制跑完：基线就红的 fault_injection 两条、first_transaction_on_device 四条、e158 两条除外，全绿。回退语义的新红是 e156 两条、e158 一条，归实四。
  - fmt 过；clippy 只剩派发点名的 e156 那一处，行号从 4059 变成 4080：插了 shim，又被 fmt 挪过。
  - `build --all-targets` 过。
  - 可单跑门禁 27、33、53、74、80、92、93、94 全绿。naming-lint 全仓红，基线就红；crates 里从基线的 120 处降到 108 处，没有新增。
  - 层 0 没跑。

## 二、写过的文件（自己列；与开工快照逐文件比出来的，43 个改、3 个删）

core：`crates/singlefs-core/src/{mount.rs, recovery.rs, transaction.rs, allocator.rs, instance_table.rs, system_configuration.rs, make_filesystem.rs, mounted_read.rs, lib.rs}`；删 `crates/singlefs-core/src/rollback_witness.rs`
format：`crates/singlefs-format/src/lib.rs`
checker：`crates/singlefs-checker/src/{walk.rs, image.rs, lib.rs}`
harness 库：`crates/singlefs-harness/src/{model.rs, model_comparison.rs, history.rs, fault_injection.rs, crash.rs}`
harness 二进制：`crates/singlefs-harness/src/bin/{e156_allocation_basis_counts.rs, e158_root_choice_repair.rs, first_transaction_on_device.rs}`
harness 用例：`crates/singlefs-harness/tests/` 下
`common/mod.rs`、`second_transaction_step_four_rollback.rs`（整个重写）、`checker_known_bad_images.rs`、`second_transaction_supplement_three_random_history.rs`、
`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`、`second_transaction_step_three_formatted_pool.rs`、
`system_configuration_mutability_classes.rs`、`second_transaction_supplement_three_crash_injection.rs`、`second_transaction_supplement_three_fault_injection.rs`、
`second_transaction_step_three_second_instance.rs`、`second_transaction_supplement_two_instance_table_chain.rs`、
`second_transaction_supplement_two_c533_row_publish_record_without_its_root.rs`、`second_transaction_supplement_two_unreadable_abandoned_root_slot.rs`、
`second_transaction_supplement_two_tree_identifier_watermark_crash_orphans.rs`、`second_transaction_supplement_two_presumed_clause_checks.rs`、
`second_transaction_supplement_two_multi_record_transaction_zero_publishes.rs`、`second_transaction_supplement_two_instance_table_page_full.rs`、
`second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs`、`second_transaction_supplement_two_admission_formula.rs`、
`second_transaction_step_five_reuse.rs`、`second_transaction_step_zero_layer0.rs`（只改到编得过）；
删 `second_transaction_supplement_two_rollback_witness.rs`、`second_transaction_supplement_two_rollback_witness_layer0.rs`
变异表：`crates/mutations.tsv`（删 36 行、换 41 行、末尾追加 13 行，见第六节）
`litmus/` 没动。

`crates/mutations.tsv` 末尾追加的 13 行（第 734–746 行），变异名：
1. 实三（回退改成挂着时的向前发布，D23 已定项 14「复活」）：回退那次不把 R_old 引用、cur 的账记已释放的用户可见单元改回已分配
2. 实三（D23 已定项 14「释放」）：回退那次不释放 cur 引用、新根不引用的用户可见单元
3. 实三（D23 已定项 14「在任何写之前拒」第三样）：复活集的单元不逐盘验读不读得出、校验和对不对
4. 实三（D23 已定项 14「水位」）：回退那次的 inode 号水位只取环里读得出的根，不与内存里现行那一版的取 max
5. 实三（D23 已定项 14「这一版的失败处置」、C558）：回退入口不先判冻结着的发布
6. 实三（候选集带文件，D23 已定项 14）：树表 0 条的回退目标报成「不在环里」
7. 实三（候选集 txg ≥ F_生效，D16 已定项 1）：低于 F_生效的回退目标报成「树表 0 条」
8. 实三（影子账，理由改指崩溃恢复）：账读不出的被抛弃根不计数
9. 实三（D16 已定项 1 根环容量边界）：入口的「4 个不同状态」不按 (inode 树根, extent 树根) 去重
10. 实三（D16 已定项 1 根环容量边界）：checker 的「4 个不同状态」不去重（I-7.9 的上限算宽）
11. 实三（D16 已定项 1 根环容量边界，理想模型）：模型的抬 F 上限不去重
12. 实三（D18 已定项 11：flags bit0 退役不回收）：实例表行读者收下 flags 非 0 的行
13. 实三（复活，分配器）：复活 defer 里那一条时 defer 槽数不减
（同样 13 行另存 `/tmp/claude-1000/impl-rbf-3/mutations-append.tsv`）

## 三、`git diff --stat -- crates litmus` 原样（含实一、实二没提交的改动，分不出谁改的；上一节是我自己的清单）

 crates/mutations.tsv                               |  177 +-
 crates/singlefs-checker/src/image.rs               |   77 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/walk.rs                |  441 ++--
 crates/singlefs-core/src/allocator.rs              |  118 +
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/lib.rs                    |    1 -
 crates/singlefs-core/src/make_filesystem.rs        |   11 +-
 crates/singlefs-core/src/mount.rs                  | 1502 +++++++++----
 crates/singlefs-core/src/mounted_read.rs           |    5 +-
 crates/singlefs-core/src/recovery.rs               |  217 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 ---
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  224 +-
 crates/singlefs-core/src/transaction.rs            |  371 +++-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        |   82 +-
 .../src/bin/e158_root_choice_repair.rs             |   33 +-
 .../src/bin/first_transaction_on_device.rs         |  129 +-
 crates/singlefs-harness/src/crash.rs               |  113 +-
 crates/singlefs-harness/src/fault_injection.rs     |   12 +-
 crates/singlefs-harness/src/history.rs             |  279 ++-
 crates/singlefs-harness/src/lib.rs                 |   41 +
 crates/singlefs-harness/src/model.rs               |  294 ++-
 crates/singlefs-harness/src/model_comparison.rs    |   71 +-
 .../tests/checker_known_bad_images.rs              |  753 ++++---
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |    3 +-
 .../tests/second_transaction_step_five_reuse.rs    |  406 ++--
 .../tests/second_transaction_step_four_rollback.rs | 2244 ++++++++++----------
 ...second_transaction_step_three_formatted_pool.rs |  322 ++-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   97 +-
 ...transaction_supplement_three_crash_injection.rs |   34 +-
 ...transaction_supplement_three_fault_injection.rs |    4 +-
 ..._transaction_supplement_three_random_history.rs |  199 +-
 ...transaction_supplement_two_admission_formula.rs |  117 +-
 ...two_c533_row_publish_record_without_its_root.rs |    7 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
 ...rop_and_devices_without_the_selected_version.rs |  139 +-
 ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
 ...tion_supplement_two_instance_table_page_full.rs |   77 +-
 ..._two_multi_record_transaction_zero_publishes.rs |    1 -
 ...action_supplement_two_presumed_clause_checks.rs |  105 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 ----------
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 ----
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  310 +--
 .../system_configuration_mutability_classes.rs     |   35 +-
 51 files changed, 5688 insertions(+), 6050 deletions(-)

## 四、diff 摘要（文件:行号，行号是今天的）

core
- `mount.rs:3004` `roll_back_by_a_forward_publish`：挂着的时候回退，一次普通发布。次序：冻结着就拒（:3011）→ 择系统配置 → 读 cur 的实例表整条链 → 候选集（`rollback_candidate` :2685：不在环里 / txg < F_生效 / 按 cur 的实例表被抛弃 / 树表 0 条，依次判）→ 从盘上重建 R_old 那一版（有节点读不出就拒）→ cur 那一版的账从盘上读一遍（读不出就拒）→ 两版树号不同就拒（条款没写，第一版不支持）→ 复活集（`resurrection_against_the_current_account` :2735：cur 账里没有记录、仍分配而分配代不同就拒）→ 复活集每个单元逐盘验读得出、校验和对（:2833）→ 释放集（cur 引用、R_old 不引用的用户可见单元；位置项要指两块池里的盘、每块盘上仍分配，D19 已定项 5 的释放前核照走，对不上的那一份隔离）→ 水位（树 ID、inode 号各取 max(内存里 cur 的, 环里读得出的根)，:2866、:3118）→ 拼一版「previous」（R_old 的用户可见一半 + cur 的固定点一半，`version_to_publish_the_rollback_after` :2889）→ 分配器上先复活、再释放（释放代 = 新根 txg）→ `publish_version`（file: None、事务号 0、实例表照带 cur 的、F 照带 cur 根上的）。发布在落盘之前被拒：分配器整个换回回退之前那一份；落盘途中失败：冻结在分配器上。
- `mount.rs:2544` `RollbackError`（12 个成员）、`:2615` `RolledBack`（释放集、隔离掉的份、复活集）、`:2629` `is_a_user_visible_unit`、`:275` `RollbackCandidateExclusion`（四成员：NotInRing / BelowEffectiveFloor / OnAbandonedTimeline / VersionWithoutFile）。
- 删：`mount_rollback`、`mount_rollback_with_space_admission`、见证写读与删条补齐那几个函数、`MountError::RollbackTargetNotACandidate`、`MountError::UnmountOfAVersionWithoutFileWhoseEmptyPublishesAreUndecided`、`MountOutput::rollback_witness_written`、`InstanceStart::rows_of_the_chosen_roots_instance_table`。
- `mount.rs:1133` `Unmounted`：`FloorRaisedToTheCurrentVersion(UnmountRaisedTheFloor)` 与 `NothingWrittenOnAVersionWithoutFile { current }`；`:1235` `unmount` 在树表 0 条的一版上一个字节都不写、交回后者。
- `mount.rs:1087` `newest_first_distinct_states`、`:1105` `ceiling_from_newest_and_non_empty_roots`（去重之后不足 4 个取最旧的有效根）。
- `mount.rs:593` `abandoned_by_table` 转到 `recovery::root_is_abandoned_by_the_instance_table`；`:625` 影子账（输入不再有见证）；`:2450` `mount_writable_with_test_only_switches`（影子账开关给用例用；`mount_writable_with_space_admission` 转它、恒开）。
- `recovery.rs:660` `choose_root` 退回按 (txg, 实例) 取最大，不跳过被见证抛弃的根；`:1815` `replay_journal` 少一个参数（回退行高水位），不再在见证与高水位上停，前缀口径注释六条改五条；`:781` `root_is_abandoned_by_the_instance_table`（新，公用）；`:803` `effective_rollback_floor` 照 D16 原文：各块盘上最新的有效根（按最新根指着的实例表判不被抛弃；表读不出就不筛）所带 F 的最大值，再与系统配置里读得出的 F 取大。删 `rollback_witness_of_the_pool`、`rollback_high_water_of_root`、`every_root_ring_slot_holds_a_root`、`RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided`。
- `instance_table.rs:21` `InstanceRow` 去掉 `is_rollback`；`:30` flags 第一版恒 0，写者写 0，读者见非 0 拒收（bit0 在内）。
- `allocator.rs:323` `DeviceFreeMap::mark_resurrected_out_of_the_defer_queue`、`:1070` `PoolAllocator::resurrect_released_record`（已回收的：重新占位、移出回收表；还在 defer 里的：defer 减跨度）。
- `transaction.rs:2755` `refuse_locations_that_do_not_name_two_pool_devices`、`:2821` `copies_of_a_released_unit_failing_the_release_checksum_check`（从发布路径抽出来，回退那次释放也走它）、`:2913` `placement_to_release_after_checking_every_device` 改 `pub(crate)`；删系统配置写里带见证那一段、`write_rollback_witness_from_now_on`。
- `system_configuration.rs:214` `SystemConfiguration` 去掉 `rollback_witness`：字段表之后到槽末是补齐，写 0。`make_filesystem.rs`、`mounted_read.rs` 跟着改。`lib.rs` 去掉 `mod rollback_witness`。

format：`lib.rs` 删 `ROLLBACK_WITNESS_*` 常量与那条布局用例。

checker
- `image.rs:37` `IMPLEMENTED_INVARIANTS` 47 → 45（去 I-7.10、I-7.11）；`lib.rs` 与 `image.rs` 删见证视图。
- `walk.rs` 删 `judge_rollback_witness_tables`、`judge_rollback_witness_against_the_chosen_roots_table`；最新根退回取最大；`:4641` 候选集按最新根的实例表判抛弃（去掉见证）；`:4669`–`:4677` F_生效 = 各块盘最新有效根所带 F 的最大值，再与系统配置里的取大；`:2939` `newest_first_distinct_state_txgs`，I-7.9 上限两沿都用它；I-7.8 排除不再看回退行；I-9.14 的说明改成「被崩溃恢复切掉的旧线」。

harness 库
- `model.rs:1093` `answer_rollback_while_mounted`：理想模型挂着时回退，接着现行那一版发一条根，文件与用户可见的那几个单元取目标那一版的；`:689` F_生效照 D16 原文；`:725` 上限按不同状态去重；`:281` 拒的理由多一个 `RollbackTargetWithoutFile`；操作种类改名 `RollbackWhileMounted`。
- `model_comparison.rs:270` `refusal_reason_of_rollback_error`；排除的映射四个成员各报各的。
- `history.rs:329` `HistoryOperation::RollBackWhileMounted`（原 `CloseAndMountRollback`）、`:2619` `apply_rollback_while_mounted`、`:2128` `rollback_error_member`；比重表里「关着时回退」并进可写挂载（BROAD 92/8，REUSE、ROLLBACK_AFTER、UNIT_AREA 各 95/5，ALLOCATION_WALL 关着时只剩可写挂载）。
- `fault_injection.rs`、`crash.rs` 跟着改名与注释。

harness 二进制
- `first_transaction_on_device.rs` 删两个 MountError 成员的分支。
- `e156_allocation_basis_counts.rs:37`、`e158_root_choice_repair.rs:38` 各加一个本地 shim `mount_rollback`（先 `mount_writable` 再挂着回退），只改到编得过；e158 的 `replay_journal` 调用少一个参数。语义归实四。

用例、模型与变异表的改法在第五至八节。

## 五、证红记录（草稿副本 `/tmp/claude-1000/impl-rbf-3/proof/`、它自己的 target-proof；每次改坏一处，跑那条用例所在的整个二进制，还原时从主工作区拷回并 touch）

基线：副本上 12 个二进制不改动先跑一遍，都绿，基线红集为空。这 12 个是 core lib、harness lib、step_four、step_five、checker_known_bad_images、rollback_floor…normal_unmount、instance_table_chain、admission_formula、unreadable_abandoned_root_slot、crash_injection、formatted_pool、presumed_clause_checks。random_history 用主工作区 final-r1 那一次当基线，21 条过。
这一节里没有一格是被测代码的 `debug_assert` 先红，都是用例自己的断言或它调的入口报错。

| 改坏哪一处 | 红的新用例 → 红在哪 |
|---|---|
| P1 mount.rs 拿掉复活（`for copy in &resurrected` → `.filter(\|_\| false)`） | step_four 七条：rolling_back_to_the_first_version_while_mounted_… :410「D 的账：A 的单元仍分配、分配代 3」；after_rolling_back_to_the_first_version_the_next_overwrite_… :160 checker I-3.9、I-3.11 红；after_rolling_back_to_the_first_version_the_third_versions_root_… :490；the_rollback_takes_the_watermarks_… :824；rolling_back_to_a_missing_root_… :160；the_failed_root_slot_write_…（C558）:160；every_crash_point_of_the_rollback_publish_… :160。step_five 九条都红在 :57 回退之后第一次覆盖写报 ReleaseTargetAlreadyReleased（the_state_repeated_by_rollbacks_…、raising_the_floor_to_the_first_release_generation_…、floor_carried_by_only_one_device_root_… 在内）。checker_known_bad_images 四条：birth_txg_that_the_line_after_a_rollback_… :2812，raising_the_floor_above_its_ceiling_…、every_root_carrying_… 在 :3280。crash_injection：crash_states_of_the_history_… :414。random_history 十一条：rolling_back_while_mounted_to_the_oldest_ring_root_… :721、the_history_that_raised_the_floor_… :1030、…at_the_effective_floor… :905（都是 NewFinding：模型对不上「单元的分配代」） |
| P2 mount.rs 候选集不按实例表判抛弃（`if abandoned_by_table(…)` → `if false && …`） | step_four rolling_back_to_a_missing_root_… :591（(2, 8) 报成 UserVisibleUnitWithoutItsRecordInTheCurrentAccount，不是 OnAbandonedTimeline） |
| P3 mount.rs 不逐盘验复活单元 | step_four each_refusal_before_any_write_… :699（DataUnitOfTheFirstVersionCorruptOnDeviceOne 那一格回退做成了） |
| P4 mount.rs 影子账一个槽都不隔离（表里原有的第 30 行） | step_four without_the_shadow_ledger_… :944、the_plain_remount_after_a_recovery_… :1000；unreadable_abandoned 两条 :245 等；admission after_a_recovery_abandoned_a_root_… 第九项得 0 |
| P5 mount.rs 账读不出的被抛弃根不计数（`unreadable += 1` → `+= 0`） | step_four torn_tree_table_… :1044、an_abandoned_roots_allocation_record_… :1153；step_five raising_the_floor_counts_abandoned_roots_… :860 |
| P6 allocator.rs 挂载内回收时不补隔离 | unreadable_abandoned a_slot_reclaimed_while_… 「50304 回收了，但 C 还在环里引用它」、the_isolation_bits_only_… 「盖掉之前 14 + 2」得 14 |
| P7 mount.rs 回退入口不判冻结 | step_four the_failed_root_slot_write_… :1431（冻结着再回退没先拒） |
| P8 mount.rs 入口不去重 | core lib mount::non_empty_root_tests::the_rollback_root_repeating_an_older_state_… mount.rs:3222；step_five the_state_repeated_by_rollbacks_… :422 |
| P9 walk.rs checker 不去重 | step_five the_state_repeated_by_rollbacks_… :464（checker 半，得 []） |
| P10 instance_table.rs 读者收下 flags 非 0 | core lib instance_table::chain_record_tests::the_row_writer_writes_zero_flags_… :295 |
| P11 allocator.rs 复活时 defer 不减 | core lib allocator::tests::resurrecting_a_released_record_… :1771 |
| P12 mount.rs 卸载报的现行版本不对（表第 756 行换锚点后） | rollback_floor…unmount_of_a_version_without_file_writes_nothing_and_succeeds :574 |
| P13 recovery.rs F_生效不读系统配置 | rollback_floor…rolling_back_below_a_floor_only_the_system_configuration_… :390（另两条旧用例同红）；step_five 不红（盘 0 的 txg 15 带 11） |
| P14 表第 391 行（实例表读者读到第 0 片就停） | instance_table_chain abandonment_by_the_instance_table_reads_the_row_… :461（另四条旧用例同红） |
| P15 表第 450 行（T = 0 的行也排除）/ 第 283 行（I-9.14 比环里每条根） | checker_known_bad_images published_nodes_behind_an_intermediate_row_… :4431 / birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_… :2859 |
| P16 表第 328 行（第一个文件版本照 mkfs 水位发号） | formatted_pool the_first_file_version_after_a_recovery_dropped_… :1418（得 (18, 19, 15)） |
| P17 表第 364 行（暖机恒推 R 次） | presumed c498_… :200（红在没改的第一次挂载那一段；改写的回退那几步没有单独的变异，靠同一个断言函数判） |
| P18 model.rs F_生效 max→min / 上限不去重 / `<`→`<=`（表第 139 行）；model_comparison 映射（表第 149 行） | harness lib：raised_floor_carried_by_the_newest_root_… model.rs:2186；after_a_forward_rollback_the_ceiling_… :2171；the_forward_rollback_to_the_effective_floor_… :2118；model_comparison::…each_rollback_candidate_exclusion_… :521 |
| N6 / N13 / N14 / N16（新追加的第 739、737、735、740 行） | step_four rolling_back_to_a_missing_root_… :589（得 NotInRing）；the_rollback_takes_the_watermarks_… :802（得 2）；rolling_back_to_the_first_version_while_mounted_… :418 等七条；rollback_floor…rolling_back_below_… :390 |

证红时的用例名是改名之前的：为过 naming-lint，把单字母版本名（a、b、c、d、e）写全了，见第十节末条。改名只动名字，同一批二进制改名之后复跑全绿。

## 六、变异行（`crates/mutations.tsv`：删 36、换 41、追加 13；门禁 33 号绿，741 条原文各命中一次）

删掉的（原行号，逐行原因）：
- 原第 27 行「步 4：回退行不带回退位…」：回退行删了（D18 已定项 11：flags bit0 退役、写者恒写 0），被测的写法不在了；读者拒收 bit0 由新行（实三：实例表行读者收下 flags 非 0）护
- 原第 29 行「步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1，盖掉被抛弃发布的记录槽）…」：挂载时回退删了（D23 已定项 14）：向前回退的 jsn 接现行那一版（`counter: current.record.counter + 1`），不再有「接 R_old 那条之后」这一步
- 原第 32 行「步 4：前缀第五条不判（回退行的 W 不封顶）…」：前缀第五条（回退行的 W 封顶）删了（D23 已定项 14 前缀口径改五条），被测代码不在了
- 原第 58 行「步 5：「非空」的前一条取任意可读根（不排除被抛弃的根与 F 之下的根）…」：「非空」不再跟前一条比，改按 (inode 树根, extent 树根) 去重数「4 个不同状态」（D16 已定项 1 根环容量边界）；被测代码不在了，去重由新行（实三：入口 / checker 的「4 个不同状态」不去重）护
- 原第 90 行「步 0：恢复把没有回退行的所选根也按 W = 0 封顶（前缀第五条误判，种进基镜像的残留记录不施加）…」：重放的回退行高水位（`rollback_high_water_of_root`）删了（前缀第五条删掉），被测代码不在了
- 原第 273 行「步 4：影子账不认树表 0 条的被抛弃根（它那片实例表既不隔离也不在任何账里，回退之后当空闲槽发出去）…」：被测的形（树表 0 条的被抛弃根、读得出且单元没被盖）在向前回退下造不出：回退不抛弃根；崩溃恢复抛弃树表 0 条那一版时新实例写行那次与被抛弃的写行那次落点逐槽相同（实三草稿量过）。代码留着，没有会红的用例（交主 agent）
- 原第 394 行「实例表第二片：影子账只隔离根记录指着的第 0 片…」：被测的形（两片实例表链的被抛弃根，树表 0 条）在向前回退下造不出，同第 273 行的理由；用例 rollback_isolates_every_page_… 删了。代码留着，没有会红的用例（交主 agent）
- 原第 432 行「C493（回退候选集条文与实现说反话）：把「目标那一版树表 0 条」那条排除加回回退候选集（回退到树表 0 条的…」：C493 的改法（回退候选集含树表 0 条的根）被回退改形态推翻：候选要带文件（D23 已定项 14），这条变异的写法就是今天的行为；反方向的变异见新行（实三：树表 0 条的回退目标报成「不在环里」）
- 原第 449 行「改法 D：回退行也排除（被抛弃时间线发布过的号不再算「出现过」，水位压低的坏镜像不红）…」：回退行删了，被测的写法（`!row.is_rollback`）不在了
- 原第 474 行「增补 2 收口表第 38 行顺带发现 O1：影子账不认树表 0 条那一版自己那棵分配记录树的节点（被抛弃根的那几…」：同第 273 行的理由：树表 0 条的被抛弃根在向前回退下造不出，用例 rolling_back_before_any_file_version_isolates_… 删了（交主 agent）
- 原第 475 行「增补 2 收口表第 38 行顺带发现 O1（同一处）：两片实例表链的被抛弃根，隔离里少那棵分配记录树的节点…」：同第 394 行的理由（交主 agent）
- 原第 483 行「增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：崩溃状态上不算「F 落在回退留下的空档里」，…」：已知红清单第 43 行那一形在崩溃注入上没有复现了（向前回退不抛弃根，改写后的用例 crash_states_of_the_history_… 不再撞到它）；这条变异没有会红的用例（交主 agent：清单那一条留不留）
- 原第 485 行「增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：checker 在 I-3.1 的说明文字里…」：它靠已知红第 43 行那一形的复现判出，复现随回退改形态没了（交主 agent，同第 483 行）
- 原第 682 行「实二五 checker 坏镜像语料：I-7.11 恒成立（见证条目所选根的实例表罩不住也不红）…」：I-7.11 停用、checker 删判定，坏镜像用例删了
- 原第 718 行「实二七（fsync 失败掉盘的核查，调度记录 2026-09-24 第三节；D2 已定项 13 挂载准入、D18…」：挂载时回退删了：那条路（回退把 R_old 那条记录交给判落后）不在了；向前回退挂在一次已经做成的可写挂载上
- 原第 725 行「实二八（D2 已定项 13 挂载准入「可写设备数 ≥ w 的下限」、已定项 6「w ≥ 2 是硬下界」；调度记录…」：挂载时回退删了：回退不再是挂载，设备数下界在它之前那次可写挂载判过；用例 mount_rollback_with_only_device_zero_is_refused 删了
- 原第 731 行「实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：回退见…」：回退见证表常量删了（字段表之后是补齐 0），被测代码与用例都不在了
- 原第 562 行「实例表第二片写路径：写行那次只给第 0 片取落点（回退要写 370 行时失败）…」：用例 rollback_counts_the_rows_… 删了；换成可写挂载那条之后与表里前一行（同一改法、同一条用例 mount_after_crashes_right_after_acquisition_…）逐字相同，门禁 33 判重复，删掉
- 原第 637, 638, 639, 640, 641, 642, 643, 644, 645, 646, 647, 648, 649, 650, 677, 678, 679, 680 行（回退见证那一族，18 行）：回退见证删了（D23 已定项 14；D22 已定项 9 见证表删掉、系统配置字段表之后是补齐 0；I-7.10、I-7.11 停用），被测代码与用例都不在了

换掉的（原行号 → 改了什么；「证过」指第五节或下面单跑过，「留 59」指没单跑、整表复跑归门禁 59 号）：
- 锚点跟着改（被测代码挪了或改写了）：
  - 第 31 行 → `rollback_candidate` 那一判（证过 P2）
  - 第 41 行 → `mount_writable_with_space_admission` 转发那一处（证过，红 the_plain_remount_… 等三条）
  - 第 138 行 → 模型 `distinct_newest_first.get(3)`（留 59）
  - 第 140 行：名字、锚点、用例都换，max → min（证过 P18）
  - 第 148 行 → `rollback_candidate` 里 OnAbandonedTimeline 那一处（留 59；同一处的 P2 证过这条用例会红）
  - 第 281 行 → walk.rs `let abandoned = abandoned_by_the_newest_roots_table(root);`（留 59）
  - 第 307 行 → 去掉 `rows_of_the_chosen_roots_instance_table` 之后的 `establish_instance` 调用（留 59；点名的那条用例在基线上就红，见第九节）
  - 第 406、409、546、547、550、551、630、674、675、676 行：释放前核抽成 `copies_of_a_released_unit_failing_the_release_checksum_check`，只是缩进变了；409、550 两处代码形状也变了，手改。406、550 单跑证过，其余留 59
  - 第 756 行：缩进 24 → 20，用例换成 unmount_of_a_version_without_file_writes_nothing_and_succeeds（证过 P12）
  - 第 762 行 → 新写的 F_生效那一段（证过，红 every_root_carrying_the_raised_floor_unreadable_…）
- 用例换成改写后的：
  - 第 28 行 → checker 的 published_nodes_behind_an_intermediate_row_…（证过）
  - 第 30、251 行 → without_the_shadow_ledger_a_publish_after_a_recovery_…（30 证过 P4；251 留 59）
  - 第 33 行 → rolling_back_to_a_missing_root_…（证过）
  - 第 46、453 行 → the_plain_remount_after_a_recovery_…（留 59）
  - 第 139 行 → the_forward_rollback_to_the_effective_floor_…（证过 P18）
  - 第 149 行 → model_comparison 的 each_rollback_candidate_exclusion_maps_to_its_own_reason（证过 P18）
  - 第 228 行 → step_zero_test_only_switches 的 named_root_ring_slots_name_exactly_the_pairs_they_were_given（留 59）
  - 第 283 行 → …cut_off_by_a_recovery…（证过 P15）
  - 第 326、328、329、605 行 → formatted_pool 新用例（全证过）
  - 第 380、382 行 → after_a_recovery_abandoned_a_root_…（留 59；同一条用例 P4 证过会红）
  - 第 391 行 → abandonment_by_the_instance_table_…（证过 P14）
  - 第 450 行 → published_nodes_behind_an_intermediate_row_…（证过 P15）
  - 第 497、498 行 → rolling_back_while_mounted_to_the_oldest_ring_root_…（留 59）
  - 第 695、696 行：种子用例改名（留 59）
- 追加的 13 行（第 734–746 行）：全证过，对应第五节的 P1、N14、P3、N13、P7、N6、N16、P5、P8、P9、P18（模型去重）、P10、P11。

表里基线就有的 84 处我那份核对脚本报的问题没动：大多是过滤串带模块路径、脚本认不出，不是腐化；门禁 33 号绿。

## 七、删掉与改写的旧用例、去向（逐文件）

## second_transaction_supplement_two_rollback_witness.rs（整个文件删，14 条）
护的都是回退见证（见证表写序、删条规则、容量、择根跳过、I-7.10/I-7.11 判定）与回退实例根读不出时不回到被抛弃时间线（C332）。
- with_the_rollback_instances_roots_unreadable_recovery_does_not_return_to_the_abandoned_timeline：C332。向前回退不开新实例、不抛弃任何根；回退那次发布 D 的根读不出时恢复落到 C 或由 D 的记录重建 D，与任何一次普通发布丢根同形 → 由 step_four 的 every_crash_point_of_the_rollback_publish_recovers_to_the_third_version_or_to_the_rollback_publish_and_every_image_is_green 护。
- the_witness_rides_from_the_first_rotation_after_the_row_publish_and_every_later_system_configuration_write：见证写序。见证删了，不再需要；系统配置字段表之后全是补齐 0 由 core system_configuration 的往返用例（「489 之后全是补齐 0」）护。
- an_entry_is_dropped_only_when_every_ring_slot_is_readable_and_no_root_lies_between_the_target_and_the_new_instance：见证删条规则。不再需要。
- a_rollback_target_abandoned_by_the_witness_is_not_a_candidate：候选集排除见证抛弃的根。见证删了；候选集排除崩溃恢复抛弃的根由 step_four 的 rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write 护。
- a_rollback_whose_witness_table_would_exceed_its_capacity_is_refused_before_any_write：见证容量。不再需要。
- a_writable_mount_with_the_rollback_instances_roots_unreadable_builds_on_the_rollback_target：回退实例根读不出时择根跳过被见证抛弃的根。向前回退不开实例，没有「回退实例」；不再需要。
- the_pool_checker_holds_both_witness_invariants_after_a_rollback、contradicting_or_unparseable_witness_tables_redden_the_witness_consistency_invariant、a_witness_entry_the_chosen_roots_instance_table_does_not_cover_reddens_the_witness_table_invariant、a_rollback_to_the_make_filesystem_root_holds_the_witness_table_invariant_without_a_row_for_instance_zero：I-7.10、I-7.11 的判定。两条停用、checker 删判定，不再需要。
- rolling_back_to_the_same_target_twenty_four_times_…never_fills_the_witness_table、rolling_back_to_the_newest_root_every_time_…still_fills_the_witness_table_on_the_twenty_fourth：见证表满。不再需要。
- after_a_rollback_mount_crashed_between_the_row_root_and_its_rotation_each_later_writable_mount_restores_the_missing_witness、the_witness_restored_after_a_rollback_across_instances_names_the_instance_that_rolled_back_not_the_intermediate_one：恢复补齐见证。不再需要。

## second_transaction_supplement_two_rollback_witness_layer0.rs（整个文件删，6 条：三条流各快、全量两条）
- 流 1（回退，写行那次轮换第一次带非空见证表）、流 2（第二次回退，表越过 512）：见证写序的崩溃状态。不再需要。
- 流 3（跨挂载：回退实例的根读不出时一次挂载在读故障下写出整段流，撤故障后回退实例的根重新读得出）：见证部分不再需要；「一次挂载在读故障下写出来的段」这一形在层 0 已有流里没有别的流罩——崩溃恢复抛弃时间线（第一轮 H6 那一形）在层 0 上没有流，交主 agent 判要不要新开（报告设计问题）。

## checker_known_bad_images.rs
- 删 each_rollback_witness_bad_image_reddens_its_own_invariant 与 known_bad_images_of_the_rollback_witness：I-7.10、I-7.11 停用、checker 删判定，不再需要（IMPLEMENTED_INVARIANTS 45 条与坏镜像清单一一对应由 the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target 护）。
- 改名改写 published_nodes_behind_a_rollback_row_or_an_intermediate_row_still_count_against_the_tree_identifier_watermark → published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark：① 回退行那一道随回退行删掉，没有对象；② T = 0 的行那一道改由崩溃恢复造：实例 2 的根槽全清零 → 实例 3 写 (2, 0, 0)，水位压到 11 红在 15（旧用例是压到 19 红在 23）。
- 改名改写 birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_is_not_compared_and_every_invariant_holds → …cut_off_by_a_recovery…：被切掉的线改由崩溃恢复造（common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before）。
- birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant：名字不变，历史改成挂着时回退（D (2, 9)），只改 D 那一片树表（没有回退之后的暖机）。J26 问的「向前发布下 ② 还造不造得出」：造得出，只红 I-9.14。
- pool_before_raising_the_floor：挂载时回退 → 挂着时回退 D (2, 9)，实例 2 覆盖写五次（txg 10–14，旧的是实例 3 四次 11–14），上限仍 11。

## second_transaction_supplement_three_random_history.rs
- 删 rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned：护的是「被抛弃」与「低于 F」两个排除理由不互报（R1）。向前回退不抛弃根、随机历史里没有崩溃恢复抛弃根的操作，这一格在随机历史上造不出；排除理由逐条报对由 step_four 的 rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write（三种排除各钉成员）与 rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor（F 之下）护，模型那一侧的理由映射由 model_comparison 的 refusal 映射用例护。mutations.tsv 148、149 两行跟着换（见变异行一节）。
- 改名改写 rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition → rolling_back_while_mounted_to_the_oldest_ring_root_on_a_small_device_and_overwriting_again_all_succeed：旧的护「取号之前预演连写行换下的落点一起释放」（挂载时回退那次挂载）；向前回退不取号、不预演，那一半没有对象；改成向前回退到环里最旧的根在 384 槽小盘上做成、之后覆盖写做成（C558 在执行器上的对照）。钉死值 reuse 8 / 2 那一格没了。
- 改名改写 raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_known_red_form_of_closeout_row_43 → the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback：同一段收缩历史换成向前回退之后跑完、每步 checker 绿。已知红清单第 43 行那一形在随机历史上再也造不出（交主 agent：清单那一条留不留）。
- rolling_back_below_the_effective_floor_…、rolling_back_to_the_root_at_the_effective_floor_…、overwrites_raising_the_floor_and_rolling_back_past_812_…：关着时回退换成挂着时回退，名字不变；拒的成员串从 MountError::RollbackTargetNotACandidate(…) 换成 RollbackError::TargetNotACandidate(…)；做成的结局从 Mounted 换成 RolledBack。
- seed_4000000045 / seed_4000000204 两条改名（去掉 into_a_rollback_gap）：比重表变了，同一种子生成的历史跟着变；45 那一步推的次数 2 → 3。
- assert_every_path_was_exercised：拒的成员串换名；「被抛弃或低于 F」那一条只数低于 F。

## rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs
- 改名改写 unmount_of_a_version_without_file_is_refused_before_any_write → unmount_of_a_version_without_file_writes_nothing_and_succeeds（实二交回第一件：不写、卸载做成；钉录制流一步不多、分配器与现行版本不动）。
- rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write：名字不变，挂载时回退换成挂着时回退（进程里那一版上直接调），加钉分配器与现行版本不动。

## second_transaction_step_three_formatted_pool.rs
- 删 rolling_back_before_any_file_version_isolates_the_allocation_record_node_only_the_abandoned_roots_reference 与 rolling_back_to_a_warm_up_root_before_any_file_version_rewrites_only_the_instance_table：两条都是「环里没有带文件的根时回退到树表 0 条的根照常做」，向前回退的候选要带文件，这一格拒成 VersionWithoutFile（step_four 候选集用例钉）。它们顺带护的「影子账认树表 0 条的被抛弃根（实例表指针、分配记录树根指针）」在向前回退下没有造法：改由崩溃恢复抛弃树表 0 条那一版时，新实例写行那次与被抛弃的写行那次同一份账、同一个形状，落点逐槽相同（实三草稿量过：实例 2 写行那次点名的 6 个单元全被实例 3 盖掉，之后重开隔离 0）。mutations.tsv 273、474 两行没有会红的用例（交主 agent，见设计问题）。

## second_transaction_step_five_reuse.rs（历史改成挂着时回退，钉死值见钉死值表）
- build_through_rollback：挂载时回退（实例 3、回退行 txg 9、暖机 10）→ 挂着时回退 D (2, 9)；四次覆盖写（11–14，实例 3）→ 五次（10–14，实例 2），上限仍 11、抬 F 仍推 15、16、E 仍是 17。
- 改名改写 the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it → the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker：旧的护「D 与前一条有效根比、不与被抛弃的 C 比」；向前回退下没有被抛弃的根，「非空」按 D16 已定项 1 的「4 个不同状态」去重。新用例同时钉入口（抬到 9 拒、上限 8）与 checker（骗过入口抬到 9，只红 I-7.9）。
- raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable：被抛弃的 C 改由崩溃恢复造（common 那个 helper），实例 3 覆盖写四次后抬 F。
- floor_carried_by_only_one_device_root_…：挂载时回退 → 挂着时回退，目标 (3, 9) → (2, 9)，加钉分配器与现行版本不动。
- 其余几条（torn_journal_record_…、raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable、reclaiming_without_raising_the_floor_…、the_transaction_number_…、floor_carried_by_the_system_configuration_…）只改钉死值与实例号。

## second_transaction_supplement_two_admission_formula.rs
- 改名改写 after_the_rollback_the_admission_reading_… → after_a_recovery_abandoned_a_root_the_admission_reading_…：被抛弃的根改由崩溃恢复造（抛弃 C），影子账开 / 关两臂在之后那次重开（实例 4）上比；被隔离的 54 → 14 槽，两臂多写的节点 2 → 0。mutations.tsv 380、382 两行的测试名跟着换。

## second_transaction_supplement_three_crash_injection.rs
- 改名改写 crash_states_after_raising_the_floor_into_the_gap_match_the_known_red_form_of_closeout_row_43 → crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_clean_under_the_forward_rollback：同一段历史换成挂着时回退之后没有空档，崩溃状态上一条清单外的失败都不许有；「清单第 43 行那一形在崩溃状态上认得出来」（K6 假阳那一半）不再有复现，交主 agent。

## second_transaction_supplement_three_fault_injection.rs
- 每类会写盘的操作上都注入过：CloseAndMountRollback → RollBackWhileMounted。

## second_transaction_supplement_two_instance_table_chain.rs
- 改名改写 rollback_candidate_set_reads_the_row_that_lives_on_the_second_page → abandonment_by_the_instance_table_reads_the_row_that_lives_on_the_second_page：树表 0 条的池上调不到向前回退入口（它要一版带文件的现行版本）；改成直接钉「按实例表判抛弃读整条链」（recovery::instance_table_of_root + root_is_abandoned_by_the_instance_table），拆片前不抛弃、拆片后抛弃。mutations.tsv 391 那一行测试名跟着换（读者停在第 0 片时照样红）。
- 删 rollback_isolates_every_page_of_the_instance_table_chain_that_only_abandoned_roots_reference：被抛弃的是树表 0 条那几条根，造法同 formatted_pool 那两条，向前回退下造不出（崩溃恢复抛弃它们时新实例写行那次落点与被抛弃的写行那次逐槽相同）。mutations.tsv 394、475 两行没有会红的用例（交主 agent）。

## system_configuration_mutability_classes.rs
- 两份样本删掉 rollback_witness 字段；两个 sha256 字面量不变（空见证表本来就是全 0，与补齐逐字节相同）——这条用例同时钉住「删见证不改系统配置槽一个字节」。

## second_transaction_supplement_two_instance_table_page_full.rs
- 删 rollback_counts_the_rows_of_the_table_it_rolls_back_to：护的是挂载时回退按 R_old 那一版表算要写的行数（369 行写满一片、370 行写两片）。向前回退不取号、不写行、实例表照 cur 的，没有对象。写两片的写路径由同文件可写挂载那几条（370 行写两片）与 second_page_write 那个文件护；mutations.tsv 562 那一行换成可写挂载那条（见变异行一节）。

## second_transaction_supplement_two_presumed_clause_checks.rs
- c497_…：删 ③（挂载时回退那次写回退行的发布）：向前回退照带现行那一版的实例表、不重写。只剩 ① ②。
- c498_…：回退那一步从「关着时回退挂载（写行 12、暖机 1）」换成「挂着时回退（txg 12，一次向前发布）」；之后几次挂载的 txg 跟着变（见钉死值表）。

## second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs
- 删 mount_rollback_with_only_device_zero_is_refused 与 rollback_onto_the_blank_or_stale_device_one_is_refused_the_same_way_before_any_write：护的是「回退也是可写挂载，同一道设备数下界与逐盘核」。向前回退不再是挂载，它挂在一次已经做成的可写挂载上，设备准入在那次挂载判过——由同文件 mount_writable_with_only_device_zero_is_refused 与可写挂载那几条逐盘核的用例护。

## second_transaction_supplement_two_tree_identifier_watermark_crash_orphans.rs
- 删流 ② every_crash_inside_the_first_file_version_after_rolling_back_to_the_warm_up_root_…（连同造它的 first_file_publish_after_rolling_back_to_the_warm_up_root）：回退到树表 0 条的暖机根在向前回退下拒成 VersionWithoutFile，「回退之后从带过来的水位 19 发第一个文件版本」没有对象；孤儿不进水位由流 ① 护。

## second_transaction_supplement_two_unreadable_abandoned_root_slot.rs（三条都保留名字，历史改写）
- 被抛弃的根从「挂载时回退抛弃 B 与实例 2」换成「崩溃恢复抛弃 C（txg 8）」，之后再重开一次（实例 4），暴露面点名 C 的根槽。
- an_abandoned_root_whose_own_root_slot_is_unreadable_is_neither_isolated_nor_counted：隔离 54 / 46 → 14 / 0；掉出来的槽从 B 的八个一槽单元换成 C 独占的 14 槽。
- the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot（C503）：盖掉被抛弃根的那次从 txg 28 换成 32；隔离 56 → 46 换成 16 → 0。
- a_slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring（C518）：补隔离的单元从 mkfs 实例表（50176）换成实例 2 那片实例表（50304），回收在 txg 31、回到空闲在 txg 32、再被用在 txg 34（旧的是 27、28、29）。

## 八、钉死值表（改前 / 改后 / 原因）

| 文件:用例 | 量 | 改前 | 改后 | 原因 |
|---|---|---|---|---|
| step_four：without_the_shadow_ledger_… / a_plain_remount_… | 影子账隔离槽数（每盘） | 40 | 14 | 被抛弃的根改由崩溃恢复造（H6 那一形，只抛弃 C 一条根，14 = C 那次发布写的槽）；旧的 40 是挂载时回退抛弃实例 2 四条根 |
| step_four：torn_tree_table_… / an_abandoned_roots_allocation_record_… | 影子账隔离槽数 | 26 | 0 | 同上：C 读不出之后没有别的被抛弃根 |
| step_four：a_plain_remount_… | 重开实例号 | 4 | 4（第一次）、5（第二次） | 抛弃那次挂载（实例 3）看不见 C，隔离从下一次挂载起；加一次覆盖写后再重开一次钉「每次挂载都重算」 |
| model.rs：raised_floor_carried_by_the_newest_root_of_one_device_takes_effect | F_生效 | 0 | 6 | F_生效照 D16 原文取各幸存盘最新持久有效根所带 F 的最大值（实二交回第二件） |
| model.rs：after_a_forward_rollback_the_ceiling_counts_the_repeated_state_once | 抬 F 上限 | （新用例） | 6 | 去重按 (inode 树根, extent 树根) |
| checker_known_bad_images：pool_before_raising_the_floor | 回退形态 / 覆盖写次数 | 挂载时回退（实例 3，txg 9 回退行、10 暖机）+ 实例 3 覆盖写 4 次 | 挂着时回退 D (2, 9) + 实例 2 覆盖写 5 次（10–14） | 上限仍 11：非空不同状态新到旧 14、13、12、11 |
| checker_known_bad_images：raising_the_floor_above_its_ceiling_… | I-7.9 违例说明里的非空有效根 | [3, 11, 12, 13, 14] | [3, 4, 8, 9, 10, 11, 12, 13, 14] | 向前回退不抛弃实例 2 那一段，4、8、9 仍是有效根 |
| checker_known_bad_images：同上与 every_root_carrying_the_raised_floor_unreadable_… | 抬到 11 之后几次发布让数据单元落回第 0 版树表那一槽 | 1 次（E，txg 17） | 2 次（E0 落 mkfs 实例表 50176、E 落 50178，txg 17、18） | 向前回退之后 mkfs 实例表那两槽也在抬 F 到 11 时回收 |
| checker_known_bad_images：every_root_carrying_… | 只读根上 F 时红的不变量 | I-2.1、I-4.8、I-7.4 | I-2.1、I-3.10、I-4.8、I-7.4 | E0 落在 mkfs 实例表那两槽：txg 0 那几条根的账记它分配代 0、单元头诞生代号 17 |
| checker_known_bad_images：every_root_carrying_… | 改坏的带 F 11 的根 | txg 15、16、17 | txg 15–18 | 多一次发布 |
| checker_known_bad_images：published_nodes_behind_… | 水位压到 / 红在 | ①11/15、②19/23 | 11/15（只剩 T = 0 那一道，历史改由崩溃恢复造） | 回退行删了 |
| random_history：seed_4000000045_… | 抬 F 那一串推的次数 | 2 | 3 | 比重表里关着时回退并进可写挂载，同一种子生成的历史变了 |
| random_history：rolling_back_…_oldest_ring_root_… | 挂载那一步的复用改写数 | rewritten_from_released 8、changed_span 2 | （不再断言） | 向前回退不取号、不写行 |
| step_three_formatted_pool：DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT | 判定那一读是第几次读系统配置槽 0 | 7 | 3 | 回退见证的四次读删了（择根、重放、影子账、写见证表之前）；草稿里 1..8 挨个试过，只有 3 分叉成「判定 1、重算 2」 |
| step_four：C558 用例 | 崩在重发之前、恢复之前镜像上红的不变量 | （新用例） | 撕裂 / 写 0 两格 I-3.1 | 与普通覆盖写同一格逐字相同（草稿量过） |
| step_five：raising_the_floor_to_the_first_release_generation_… | 五次覆盖写之后 defer 槽数（每盘） | 83（四次覆盖写之后） | 143 | 向前回退不重建分配器：F = 0、最旧有效根 txg 0，释放代 3–14 全在 defer（1+8+10+8+8+14+14+16×5）；旧的回退挂载从 A 的账重建、B/C 那段靠影子账隔离 |
| 同上 | 抬 F 到 11 回收的落点数 | 32 | 86 | 释放代 ≤ 11：1+8+9+8+8+12+12+14+14 |
| 同上 | 抬 F 之后 defer 槽数 | 64 | 64 | 143 − 95 + 16 |
| 同上 | E 的数据落点 | 50178（mkfs 树表） | 50176（mkfs 实例表 2 槽） | 旧形态下 50176 被 B（被抛弃）引用、影子账隔离；向前回退下 A、B 在 F 之下，50176 回收可用 |
| 同上 | E 之后 defer 槽数 | 80 | 80 | 64 + 16 |
| step_five：floor_carried_by_the_system_configuration_… | 重开之后 defer 槽数 | 90 | 92 | 释放代 12–14 各 16、15/16 各 8、写行 12、暖机两次各 8 |
| step_five：reclaiming_without_raising_the_floor_… | 回收数 / E 落点 | 32 / 50176（A 的数据） | 86 / 50176（mkfs 实例表） | 同上；I-2.1 仍红（A 的根指着 mkfs 实例表） |
| step_five：the_transaction_number_… | 实例的事务号序列 | 实例 3：[0,0,1,2,3,4,0,0,5] | 实例 2：[0,0,0,1,0,2,3,4,5,6,0,0,7] | 回退那次发布 D 写 0、不推进计数（条款没写，交主 agent） |
| step_five：torn_journal_record_… | 撕掉的记录 | (3, 14) | (2, 14) | 实例号 |
| step_five：raising_the_floor_is_refused_when_… | 点名的根 | (3, 12) | (2, 12) | 实例号 |
| admission_formula：after_a_recovery_abandoned_a_root_… | 被抛弃根独占槽 / 两臂多写的节点 | 54 / 2 | 14 / 0 | 被抛弃的只剩崩溃恢复抛弃的 C（14）；两臂写行与暖机落点不跨叶 |
| presumed_clause_checks：c498_… | 回退之后关闭再挂载 | 写行 14、暖机 2 | 写行 13、暖机 1 | 回退从一次挂载（写行 12 + 暖机 13）变成一次发布（12） |
| 同上 | 崩在中途那次挂载 | 写行 17、暖机 18 崩 | 写行 15、暖机 16 崩 | 同上 |
| 同上 | 崩了再挂 | 所选根 17、施加到 18、写行 19、暖机 1 | 所选根 15、施加到 16、写行 17、暖机 2（txg 18 白推） | 同上 |
| unreadable_abandoned_root_slot：an_abandoned_root_whose_own_root_slot_is_unreadable_… | 两臂隔离槽数 | 54 / 46 | 14 / 0 | 被抛弃的只剩 C；C 的根槽读不出时它独占的 14 槽全掉出 |
| 同上 | 掉出来的槽 | B 的八个一槽单元 50257–50264 | C 的 14 槽（50184–50185、50330、50332–50342） | 同上 |
| unreadable_abandoned_root_slot：the_isolation_bits_… | 盖掉被抛弃根的 txg / 隔离变化 | 28 / 56 → 46 | 32 / 16 → 0 | C 是 txg 8 |
| unreadable_abandoned_root_slot：a_slot_reclaimed_while_… | 补隔离的单元 / 三个 txg | mkfs 实例表 50176 / 27、28、29 | 实例 2 那片实例表 50304 / 31、32、34 | 同上 |
| step_zero_layer0（层 0，没跑）：Script::RollbackToFirstVersion | 根槽写条数 | 10 | 9 | 回退从一次挂载（回退行那次发布 + 暖机）变成一次发布；只这一处照翻译改了，别的钉死值见第九节层 0 那一段 |
| step_zero_layer0（层 0，没跑）：ReuseAfterRaisingFloor 的 E 数据落点 / ReuseOfTheFirst… 的 txg 18 数据落点 | 数据槽 | 50178 / 50180 | 50176 / 50178 | 同 step_five；层 0 之外照同一段脚本量的 |

## 九、C558 结论

用例：`second_transaction_step_four_rollback.rs` 的 `the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash`。

搭法：
- 两块盘。先在 mkfs 同一个进程里覆盖写到 txg 10，再重开取号 2，接着覆盖写到 txg 27。根环 24 槽里装 txg 4–27，最旧的是 B (1, 4)。
- 转环之前重开那一次，是为了绕开 C551：`build_pool` 的分配器没有根环表，一路在进程里转环，I-3.1 会红。
- 回退到 B 的那次发布是 D（txg 28），它的根槽就是 B 那一槽。
- 只在 D 的根槽那一次写上注入失败，三种：一个字节都不写；拼一整块写下（前一半是 D、后一半是 B 的旧字节）；整块写 0。起初「写下一半」是按半块写的，录制设备不收不满一块的写，等于一个字节没写；已改成拼整块。

两条路：
- 进程不崩：D 冻结在分配器上，再调同一条回退在任何读写之前拒成 `PublishFrozenAfterAWriteFailureIsNotResentYet`；原样重发 D 做成（txg 28），冷启动读回 B 的内容，checker 全绿。
- 进程在重发之前崩：D 的记录在根槽写之前已持久。恢复由记录重建 D (2, 28)、读回 B；再可写挂载接在 D 后面，checker 全绿。

结论：C558 说的「退不回去」在这两条路上走不到。
- 撕裂、写 0 两格里，B 的根槽确实没了，但回退已经生效，要么靠重发、要么靠记录重放，不用再做一次。
- 走得到的只剩多重故障：D 的记录点名的单元在恢复时也读不出。那时恢复落在 27，B 已离环，退不回去。这是推的，没造。
- 用例留作证据。

顺带量到的（交主 agent，设计问题 2）：
- 崩在重发之前、恢复之前的镜像上，撕裂、写 0 两格 checker 判 I-3.1 红（记账多于遍历 229376 字节）。原因：最新根 27 的账还按「B 在环里」把 B 独占、释放代 5 的单元算已分配；B 的根槽没了，遍历里没有它；D 的记录还没被哪次恢复施加，checker 不并它。
- 同一格换成普通覆盖写（txg 28 的根槽写坏），判出的是逐字相同的一条，在草稿里量过，所以与回退无关。
- 恢复重开之后全绿。用例把这两格钉成「恢复之前只红 I-3.1」。

## 十、第 4 步那几样的末尾原样输出与门禁判定行

动到的测试二进制（主工作区、自己的 target-main；整个二进制，不按名字挑；名字含 layer0 的没跑）：
```
core/checker/format --lib：test result: ok. 121 passed（core）；ok. 5 passed（checker）；ok. 5 passed（format）
harness --lib：test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.60s
second_transaction_step_four_rollback：test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 163.17s
checker_known_bad_images：test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.16s
second_transaction_supplement_three_random_history：test result: ok. 21 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 404.72s
rollback_floor_written_into_the_system_configuration_first_and_normal_unmount：test result: ok. 11 passed; … finished in 5.26s
second_transaction_step_three_formatted_pool：test result: ok. 12 passed; 0 failed; … finished in 4.76s
system_configuration_mutability_classes：test result: ok. 2 passed
second_transaction_supplement_three_crash_injection：test result: ok. 7 passed; 0 failed; 1 ignored; … finished in 27.02s
second_transaction_supplement_three_fault_injection：test result: FAILED. 7 passed; 2 failed; 1 ignored（基线同红：a_write_error_after_the_acquisition_…、a_swallowed_write_after_which_the_checker_flags_only_i_3_1_…，快照上跑同样两条红）
second_transaction_step_three_second_instance：ok. 6 passed
second_transaction_supplement_two_instance_table_chain：ok. 9 passed
second_transaction_supplement_two_c533_row_publish_record_without_its_root：ok. 1 passed
second_transaction_supplement_two_unreadable_abandoned_root_slot：ok. 3 passed
second_transaction_supplement_two_tree_identifier_watermark_crash_orphans：ok. 1 passed
second_transaction_supplement_two_presumed_clause_checks：ok. 3 passed
second_transaction_supplement_two_multi_record_transaction_zero_publishes：ok. 2 passed
second_transaction_supplement_two_instance_table_page_full：ok. 2 passed
second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version：ok. 14 passed
second_transaction_supplement_two_admission_formula：ok. 3 passed
second_transaction_step_five_reuse：ok. 13 passed
second_transaction_supplement_two_commit_generated_fallback：ok. 7 passed（没改，碰改过的核心路径，顺跑）
second_transaction_supplement_two_unequal_devices：ok. 3 passed（同上）
second_transaction_supplement_two_release_checksum_quarantine：ok. 13 passed（用到抽出来的释放前核）
system_configuration_rollback_floor_and_layout_identity：ok. 5 passed（同上）
harness --bins：e156_allocation_basis_counts FAILED 9 passed; 2 failed（rollback_isolation_scenario_matches_the_new_layout、hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test——快照上 11 条全过，是回退语义的新红，归实四）
               e158_root_choice_repair FAILED 40 passed; 3 failed（constants_and_anchors_all_pass、local_constants_match_crates_today 快照上同红；mount_writable_trajectory_distinguishes_persistent_from_transient_faults 是新红：它靠挂载时回退造被抛弃的根，归实四）
               first_transaction_on_device FAILED 12 passed; 4 failed（快照上同样四条红：段形状 28 vs 16，实一实二时就不符）
               其余 bin 全过
```
common/mod.rs 加了 helper，所有 `mod common;` 的二进制都编进它（死代码，`allow(dead_code)`）。没用它的那些二进制没逐个跑，只 `build --all-targets` 过；全量 `cargo test` 按规矩没跑。

fmt / clippy / build（末尾原样）：
```
cargo fmt --all -- --check：exit 0，无输出
clippy（check.sh 那一套 lint，--workspace --all-targets）：
error: this assertion is always `true`
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:4080:9
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 1 previous error
（就是派发点名的 4059 那一处 `170 * 812 >= 812 * 169`：插 shim 下移 27 行，fmt 又挪了几行；其余目标 clippy 全过）
cargo build --offline --all-targets：Finished `dev` profile [unoptimized + debuginfo] target(s)
```

门禁（可单跑的，判定行原样）：
```
== 27-format-constants.sh exit=0
  ✓ 格式常量同步（41 个已登记，41 个在源码里被钉住）
== 33-mutation-tables.sh exit=0
  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 741 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
== 53-format-const-placeholders.sh exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
== 80-absolute-assertions.sh exit=0
  ✓ 152 个实验二进制各自至少有一条绝对值断言（research/e7-index-bench/src/bin 下 148 份，别处 src/bin 下以 e<数字>_ 开头的 4 份）
== 92-layout-checker-sync.sh exit=0
  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，105 个格式常量里变了 7 个（checker 在同一次改动里跟了 7 个，按滞后表放行 0 个），都不欠 checker 跟进
== 93-feature-bits.sh exit=0
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 54 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
== 94-checker-implementation-disjoint.sh exit=0
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`；共享模块 1 份源码的正文 283 行里没有分支与循环
== 74-model-differential.sh（SINGLEFS_GATE_FULL=1，release，CARGO_TARGET_DIR 指自己的 target-main） exit=0
  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：
== naming-lint（.claude/scripts/naming-lint.sh） exit=1
  ✗ 230 处名字不合命名纪律（查了 261 个 .rs 文件、62487 个声明的名字）
```
naming-lint 全仓红，基线就红：
- 122 处在 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`，不是我的。
- crates 里 108 处。我在快照上单跑过同一个脚本，crates 有 120 处；我的改动删掉 12 处、新增 0 处。
- 为了不新增，把新写的单字母版本名改了。例如 `oldest_root_b` → `oldest_ring_root`，`content_of_b` → `content_of_the_oldest_ring_root`，枚举成员 `…OfA…` → `…OfTheFirstVersion…`，开头的 `a_` → `the_`。

层 0（没跑，只改到编得过，最直接的翻译）：
- `second_transaction_step_zero_layer0.rs` 的 `Script::RollbackToFirstVersion` 改成同一个会话里挂着时回退，D 是 (2, 9)，没有暖机；`ReuseAfterRaisingFloor` 改成回退之后覆盖写五次（txg 10–14，实例 2）。
- 没核的钉死值，逐个列出：
  - `RollbackToFirstVersion`、`ReuseAfterRaisingFloor`、`ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish` 三条的段序列与写表条数：没改。实二就报过快照上这几条已经不符。
  - 我在层 0 之外照同一段脚本只跑 `prepare`、不枚举，量到的是：
    - Rollback 段序列 `[2,2,1,2,2,1,26,2,1,26,2,1,4,18,2,1,18,2,1,18,2,1,26,2,1,22,2,1,2]`，写 191、根槽写 9；
    - ReuseAfterRaisingFloor 段序列 `[…,26,2,1,22,2,1,30,2,1,30,2,1,30,2,1,30,2,1,30,2,1,4,16,2,1,18,2,1,30,2,1,2]`，写 433、根槽写 17。
  - 根槽写条数：Rollback 那一格 10 → 9，是翻译必然带来的，已改；另两格仍是 17、18。
  - E 的数据落点 50178 → 50176，prepare 里的断言已改。
  - txg 18 的数据落点 50180 → 50178，新常量 `TXG_18_DATA_UNIT_SLOT`。这一变，脚本原意「环里第一条点名块被合法复用的记录在 txg 18」就不成立了：A 的数据槽 50180 要到 txg 19 才被复用。按「50180 = txg 18」写的下游用例（C507 那两条手搭用例、改坏 tail 那几条）照今天会红，没改，归实四。
  - 这些流的状态数、各状态判定都没核。
- 回退见证那个层 0 文件整个删了，三条流。其中流 3「一次挂载在读故障下写出来的段」这一形，层 0 上没有别的流罩。崩溃恢复抛弃时间线（H6 那一形）在层 0 上也没有流。要不要新开，交主 agent。

## 十一、交主 agent 的设计问题

1. 回退那次发布的记录事务号：条款没写。
   - 实现取 0，不推进实例内计数（`mount.rs:3154` `transaction: 0`）。
   - D23 已定项 19 ① 说「事务号 0 留给不承载事务的空发布记录」，而回退那次不是空发布：它写单元、改用户可见的状态。
   - 另一读法：当成一个事务、取下一个号。这会影响 I-8.7 的序列与 W 的 max。
   - 用例 `the_transaction_number_keeps_counting_…` 钉的是今天的取法（实例 2 的序列里 D 是 0）。要改是一行，连带那条用例。
2. 满环时根槽写坏的崩溃镜像，checker 判 I-3.1 红（C558 顺带量到，第九节）。
   - 撕裂、写 0 两格，恢复之前的合法镜像上 I-3.1 红；普通发布同形，逐字相同。
   - 要定的是 checker 在「最旧的根被这次没写成的根槽盖掉、这次的记录还没被施加」时怎么判，或者这一格认成已知红。
   - 今天的用例把它钉成「只红 I-3.1」，修了 checker 那条断言要跟着改。
3. 树表 0 条的被抛弃根，影子账那一支今天没有会红的用例。
   - 被测的形是「读得出、单元没被盖的树表 0 条被抛弃根」。向前回退不抛弃根；崩溃恢复抛弃树表 0 条那一版时，新实例写行那次与被抛弃的写行那次同一份账、同一个形状，落点逐槽相同（草稿量过：6 个点名单元全被实例 3 盖掉，之后重开隔离 0）。
   - 代码（`mount.rs` 里树表 0 条那一支的 `placements_referenced_by_root`）留着。变异表第 273、394、474、475 行删了，用例 `rolling_back_before_any_file_version_isolates_…`、`rolling_back_to_a_warm_up_root_…`、`rollback_isolates_every_page_…` 删了。
   - 留不留那一支、要不要另找造法，交你。
4. 已知红清单第 43 行那一形（F 落进被抛弃实例留下的空档，`history::KNOWN_RED_FORMS`）在随机历史与崩溃注入里都没有复现了：向前回退不抛弃根，随机历史里没有崩溃恢复抛弃根的操作。
   - 清单那一条与它的判读（`raise_after_rollback_leaves_allocated_statistic_above_walked`）都留着。
   - 原先的两条复现改成「同一段历史在向前回退下跑完」。
   - 变异表第 483、485 行删了。
   - 留不留、要不要在崩溃恢复抛弃的形上重造复现，交你。
5. F_生效根上那一半，core 没有会红的用例。
   - 实现照 D16 原文：各块盘最新有效根所带 F 取最大。但抬 F 先写系统配置，系统配置里的 F 在所有走得到的状态上都不低于根上的，根上那一半被它罩住（推的；P13 那一格量到 step_five 不红）。
   - 只有理想模型的用例钉它（P18 证过）。
   - D16 开着的那一问「被抛弃时间线上的根带的 F 算不算」照开着：「有效」按最新根的实例表判不被抛弃，被抛弃的根不算进去，这是字面读法，没另判。
6. 两版树号不同的回退（`TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided`）：条款没写，在任何写之前拒。
   - 走不走得到：同一条线上带文件的各版共用第一个文件版本发的那八棵树的号。树号不同要求目标与现行那一版分属两个「第一个文件版本」，那只在崩溃恢复丢掉一次第一个文件版本之后出现（formatted_pool 新用例那一形）。而丢掉的那一版在被抛弃的线上，不是候选。所以推的是走不到，没造。
   - 用例 `each_refusal_…` 里用手改现行那一版内存里的树号钉「拒、盘上不变」。
7. 回退那次新根带的 F 取 cur 根上带的（`rollback_floor: current.root.rollback_floor`），与普通覆盖写一样，不取盘上的 F_生效。可写挂载写行那次取的是 F_生效（实二）。条款没说回退这次该带哪个，照普通发布取。
8. 回退那次的 jsn 接 cur 那一条（`counter: current.record.counter + 1`）。旧变异表第 29 行删了；这一处没有单独的变异行，D 的崩溃点重放用例钉「恢复到 C 或 D」。
9. 层 0 的覆盖缺口与 step_zero_layer0 下游用例的语义断裂，见第十节末。
10. 旧格式兼容：读者见实例表行 flags 非 0 一律拒收。老代码写过回退行（bit0 = 1）的池会因实例表读不出而挂不上，靠实一换的 incompat 位在更早就拒。没另造老镜像核这一格。

## 十二、写范围之外的牵连（没改，交主 agent）

kb：
- `layout/02` 那两行（见证表偏移、回退行 flags bit0）与 `layout/01`「八、」里步 4 那一行：照向前回退改。门禁 92 号今天把第一条纯 SSD 布局线算作「没有回退见证与回退行」，没报欠。
- `invariants.md`：
  - I-7.10、I-7.11 停用，已实现条数 47 → 45。
  - I-7.8 片段三点名的用例名：`published_nodes_behind_a_rollback_row_or_an_intermediate_row_…` → `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`，现在只剩 T = 0 那一道。
  - I-9.14 两份坏镜像：② 仍叫 `birth_txg_that_the_line_after_a_rollback_…`，历史改成挂着时回退，J26 问的「向前发布下还造不造得出」答案是造得出、只红 I-9.14；③ 改名 `…cut_off_by_a_recovery…`，被切掉的线改由崩溃恢复造。
- milestone/02 步 4、步 5：
  - 步 4 的验收脚本：回退到 A 是 D (2, 9)，不取号、不写行。
  - 步 5 的数：覆盖写 txg 10–14 五次；defer 143 → 64 → 80；回收 86 个落点；E 落 50176，不是 50178。见第八节。
- D3 已定项 10 ⑤「一次发布重写实例表单元时（写行、暖机、回退那几次）」：向前回退不重写实例表，「回退」那一格没对象。
- D16 已定项 8「回退之后」的暖机次数：回退不再暖机，「回退之后关闭再挂载」照普通挂载算。
- checks-owed：
  - C558 写结论（第九节）；
  - C554 仍成立，unreadable_abandoned 那条用例钉着「根槽读不出时隔离 0、计数 0」；
  - C551 的绕法：先挂载一次再转环；
  - 设计问题 2、3、4、5 要不要立账。
- 层 0 登记表（step_zero 那几条流）。

`.claude/gate.d/`：grep 过 `*.tsv` 与 `*.sh`，没有一处指到删掉的两个见证用例文件或回退见证；`59-crates-mutation-replay.sh` 下次整表跑时，第六节「留 59」那几行第一次复跑。

E142 / E158 装置：E158、E156 两个 bin 的本地 shim 只保证编得过，e156 两条、e158 一条单元测试照今天红，实验页与装置语义归实四。E142 执行员的副本我没碰。

CLAUDE.md：没碰。

## 十三、没做什么

- 没走三方对抗；层 0（含 step_zero_layer0 与删掉的见证层 0 流）、QEMU、herd7、crates 变异整表（59 号）归 `crash-verifier`；没跑全量 `cargo test`、整轮门禁、54 / 55 / 57 / 59 / 87 号；没提交，没做任何 git 写操作。
- 第六节「留 59」的那十几行变异没单跑。
- common/mod.rs 改了之后，没用新 helper 的那些测试二进制没逐个跑，只编过。
- 设计问题 3 那一形（树表 0 条的被抛弃根）没另找造法；已知红第 43 行没在崩溃恢复形上重造复现；层 0 没新开流。
- C558 的多重故障那一格（D 的记录点名单元在恢复时也读不出）没造。
- F_生效根上那一半在 core 上没有会红的用例（设计问题 5）。
- kb、`.claude/`、`research/`、CLAUDE.md 一处没改。

## 十四、草稿与清理

草稿都在 `/tmp/claude-1000/impl-rbf-3/`。交回前删掉的（先 `du -sh`）：`target-main` 11G、`target-proof` 3.3G、`target-pristine` 1.8G、证红副本 `proof/` 5.8M、开工快照 `pristine/` 5.9M、naming-lint 基线副本 `naming-baseline/` 5.9M、九个诊断用的 `*.rs` 草稿拷贝（各几十 K）。删完目录 3.3M。留下的有：
- 这份报告；
- `progress.md`；
- `proofs.md`、`deleted-tests.md`、`pinned-values.md`：第五、七、八节的底稿；
- `mutations-before.tsv`：整理之前的表；
- `mutations-deleted.tsv`、`mutations-modified.tsv`、`mutations-append.tsv`；
- `check-mutations.py`、`rewrite-mutations.py`、`proof-*.{py,sh}`、`clippy.sh`；
- `logs/`。
