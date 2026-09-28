# 实四乙交回：收尾杂项（12 件）

实现员（implementation-writer），2026-09-26 开工；开工快照 `/tmp/claude-1000/impl-rbf-4b/pristine/`（crates、litmus、Cargo.*，含实一至实三与别的会话没提交的改动），下文「基线」都指它。改动都在主工作区，没有提交、没有 git 写操作。
实四甲同时在改 `e156_allocation_basis_counts.rs`、`e158_root_choice_repair.rs`，并往 `crates/mutations.tsv` 末尾追加了两行（第 747、748 行），这三处我没碰。

## 一、逐件结论

1. **`first_transaction_on_device` 开工就红的 4 条**：照实际改钉值（见第六节钉死值表），整个 bin 16 条全过；实二那条第 616 行（今天是第 602 行：抬 F 失败时不用抬 F 交回的已落盘账）在副本里证红，红在 `failed_raise_of_the_rollback_floor_…`（`first_transaction_on_device.rs:2221`「同一段里失败之前已经落盘的发布各一行」，left `[]`、right `[(raise_rollback_floor, 1, 21)]`）。同 bin 的 92、93、467、468、469、471、481、482、601、606、607 行也逐条证红。
2. **故障注入基线红的 2 条**：
   - C378 那条：可写挂载写行那次发布从 15 次写变成 23 次（分配记录树按位置寻址，写行一次重写 5 个节点、两盘 10 次写），原先摆在第 18 次的注入落进写行那次、记成「新号一条根都没有」。改成第 26 次（写行 23 次之后暖机第一次的第一个写；草稿里逐次注入量过：第 22、23 次报错（23 是写行那次的根槽写）记成「新号一条根都没有」，第 24、25 次（写行之后的系统配置轮换）与 26、27 次记成「新号已有写行那次的根」；这一步一共 46 次写，`logs/probe-mount-writes-2.log`）。
   - 说谎设备 I-3.1 那条：实三把比重表里关着时回退并进可写挂载，同一个种子生成的历史变了，原种子 …5306 那段里不再有只红 I-3.1 的一格。在今天的代码上照大档规模（种子基起 512 段、每段 30 步、注入 6 次，release）扫了一遍：14 段里有 `["I-3.1"]` 这一组；换成种子 7463871032432355210（被吞掉的是 step 25 覆盖写的第 31 次写、整池第 620 次，即根槽写）。整个二进制 9 条全过；第 188、301 行证红。
3. **区域表**：`first_transaction_regions.rs` 照 `layout/01-first-txn.md` 零那一节今天的写清单改成 29 行（12 个单元 × 2 + 根槽 + 记录 × 2 + 系统配置 × 2；分配记录树五个节点 50245–50249，记账 / 映射 / 树表顺延到 50250–50252；区域名照 E142 装置 `descriptive_tag()`）。测试的期望表双份抄同一张写清单；bin `first_transaction_region_bytes` 退 0（`write_calls=29 … matches=true`，基线退 1）。测试改名 `…twenty_one…` → `…twenty_nine…`，第 103 行点名跟着换。
4. **实二八剩的三项**：
   - 第 723 行那一类（今天第 689 行，分配记录树那一判 panic）：改点名新用例 `reading_the_allocation_record_tree_with_the_reader_of_only_device_zero_reports_the_child_key_violation_instead_of_panicking`，从公开读者 `recovery::allocation_records_under_root` 直接喂只有盘 0 的读者，断言交回 I-1.1、不 panic；第 689 行证红（`.expect` 在 `allocation_record_tree.rs:766` panic、用例的 `catch_unwind` 断言红）。
   - 数不同的设备身份：`mount.rs:2128` 新 `distinct_device_identities_handed_in`，可写挂载准入（`:2473`）数盘表里不同的设备身份；新用例 `mount_writable_with_device_zero_handed_in_twice_counts_one_device_and_is_refused_before_any_read`（盘 0 交两次 ⇒ `DeviceCount(1)`、两项都没被读、逐字节不变）。第 690、692 行锚点跟着改。
   - 单盘只读挂载：新用例 `read_only_mount_with_only_device_zero_handed_in_opens_the_newest_version_on_it_and_reads_it_back`（读者只列得出盘 0，只读挂载沿 (1, 5) 打开、读回第三版）。这一条是钉现状的用例，代码没改；它护的是择根时一个根槽读不到就跳过那一行（`recovery.rs` `visit_valid_roots_with_ring_slots` 的 `continue`）。
5. **C519 那条 `#[ignore]`**：删了。它要的结局（丢一整块盘之后确认的 txg 6 还读得回）与用户 2026-09-25 定案（两份都验过才施加）相反。那件事由 `c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version` 护着（`checks-owed.md` 第 501 行 C519 已还清那一行点名的就是它），`crates/mutations.tsv` 第 355 行把 `all` 改成 `any` 它就红（副本里证过：红在 `…c519_whole_device_loss_after_warm_up.rs:242`）。文件头说明跟着改。
6. **C542 普通挂载那一支**：判为走不到，用例留作证据（见第四节）。
7. **收口表第 43 行那一形**：走得到，造出来了（见第四节）。修法没动，只钉今天的结局。
8. **I-3.1 在恢复前镜像上红（实三 Q2）**：查清了机理，写成用例钉住；判法改不改交主 agent（见第四节与第九节）。
9. **影子账树表 0 条的被抛弃根那一支（实三 Q3）**：造得出，补了会红的用例；实三删掉的第 273、474 行那两种改法重新作为变异追加（见第四节）。那一支没删。
10. **旧镜像被拒的错误成员**：`SystemConfigurationSlotRefusal::IncompatBitsNotRecognized { incompat_bitmap }`（`system_configuration.rs:512`）与 `RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized { first_device_carrying_them, incompat_bitmap }`（`recovery.rs:168`）。magic 或整槽校验和不过仍是「自证不过」；两关都过、incompat 位图不认识改报新成员。池里一槽可择的都没有时，有一槽是布局不认识就报新成员，否则才报 `NoValidSystemConfiguration`（`recovery.rs:668` 起）。`history.rs:1996` 穷举臂补一臂。
11. **两处 481**：第 163 行（派发说的第 168 行，实三删行后下移）变异名「四档的和仍是 481」→ 489。`checker_known_bad_images.rs:2280` 的 `.expect("481")` 在开工时已经不在（实三重写那份文件时删了；HEAD 上还在），没有可改的。
12. **命名**：crates 里 naming-lint 报的从 108 处降到 9 处，改了 99 个名字（见第五节）。

## 二、推翻条件

- 第 1、3 条：`first_transaction_on_device` 或 `first_transaction_region_bytes` 在当前代码上红；或 `layout/01-first-txn.md` 零那一节写清单与区域表的 29 行对不上。
- 第 2 条：故障注入二进制在当前代码上红；或同一组规模下种子 …5210 那段里没有只红 I-3.1 的一格。
- 第 4 条：同一个设备身份交两次时可写挂载读了盘或没拒；只交盘 0 的只读挂载没读回 (1, 5)。
- 第 7 条：崩溃恢复抛弃 C 之后抬 F 到 5，checker 判红的不只 I-3.1，或归类不到已知红第 1 条。
- 第 9 条：崩溃恢复抛弃写过行的树表 0 条根之后，卸载与下一次挂载隔离的不是每盘 2 槽。
- 第 10 条：只带退役位 0 的镜像，挂载报的不是新成员；或两盘四槽都自证不过时报的不是 `NoValidSystemConfiguration`。
- 第 6 条：出现一格「所选根是环里最新的根、挂载那一串把它挤出环、回收了这一串自己换下的单元」。
- 第 8 条：同一格普通覆盖写的恢复前镜像上，「记账 − 遍历」不等于只被被盖掉那条根引用的单元字节数。

## 三、写过的文件（自己列；与开工快照逐文件比出来的）

改动性质分三类：功能改动、新用例或改用例、只因命名改名。

- **功能改动**（连测试）：
  - `crates/singlefs-core/src/system_configuration.rs`：新成员、`IncompatBitmap`、单测两条跟着改、改一个测试名。
  - `crates/singlefs-core/src/recovery.rs`：新成员、`SystemConfigurationSlotReading`、择系统配置分流、改名 4 处。
  - `crates/singlefs-core/src/mount.rs`：`distinct_device_identities_handed_in`、准入调用、两处文档。
  - `crates/singlefs-harness/src/history.rs`：穷举臂一臂、改名 2 处。
  - `crates/singlefs-harness/src/first_transaction_regions.rs`：区域表 21 → 29。
- **改钉值 / 改用例 / 新用例**：
  - `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`
  - `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`（只改文档里的 21）
  - `crates/singlefs-harness/tests/first_transaction_region_bytes.rs`
  - `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs`（两条改值，另改名）
  - `crates/singlefs-harness/tests/second_transaction_supplement_two_c519_whole_device_loss_after_warm_up.rs`（删 ignore 那条）
  - `crates/singlefs-harness/tests/second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs`（新 3 条）
  - `crates/singlefs-harness/tests/system_configuration_rollback_floor_and_layout_identity.rs`（改 1 条、新 1 条；实一的新文件，未跟踪）
  - `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`（新 1 条与两个辅助）
  - `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`（新 1 条与一个辅助）
  - `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`（新 2 条，另改名）
- **只因命名改名**：`crates/singlefs-checker/src/position_addressed.rs`；core 的 `admission.rs`、`allocation_record_tree.rs`、`allocator.rs`、`code_two_tree.rs`、`extent_tree.rs`、`journal.rs`、`mounted_read.rs`、`write_request_split.rs`；harness 的 `src/device_log.rs`、`src/fault_injection.rs`、`src/lib.rs`、`src/model.rs`、`src/read_tally.rs`、`src/segments.rs`；tests 下 `checker_known_bad_images.rs`、`parallel_line_one_sequential_write.rs`、`second_transaction_parallel_line_one_last_record_flag.rs`、`…_one_multi_unit_file.rs`、`…_one_sequential_write.rs`、`…_three_many_inodes.rs`、`…_two_mounted_read.rs`、`second_transaction_step_zero_test_only_switches.rs`、`second_transaction_supplement_three_bad_disk_input.rs`、`…_three_random_history.rs`、`…_two_admission_formula.rs`、`…_two_instance_table_second_page_write.rs`、`…_two_multi_record_transaction_zero_publishes.rs`、`…_two_publish_failure_resent_unchanged.rs`、`…_two_release_checksum_quarantine.rs`、`…_two_root_ring_turn_in_one_mount.rs`、`…_two_row_publish_checks_before_acquisition.rs`、`…_two_tree_nodes_and_the_central_mapping.rs`、`…_two_tree_split.rs`、`…_two_unreadable_abandoned_root_slot.rs`。
- `crates/mutations.tsv`：改了既有行 100 行（点名换名 94 行、锚点跟改 5 行、变异名改 1 行，第八节列）；末尾追加 11 行（第八节列变异名）。实四甲的第 476、747、748 行不是我改的。
- `litmus/` 没动。`tests/common/mod.rs` 没动。

## 四、第 6–9 件的判法与证据

### 第 6 件 C542 普通挂载那一支：走不到（用例留作证据）

- 推理（读 `mount.rs` `dry_run_of_the_publishes_after_acquisition` 与 `allocator.rs` `record_root_written_by_this_process`）：预演与真发只在「这一串换下、核出对不上被隔离的那一份，在这一串里被回收」时分叉。这一串换下的单元释放代 ≥ 这一串第一次发布的 txg；回收门槛是 max(F_生效, 环里最旧有效根)。所选根是最新根时，它在这一串之后还在环里（这一串写 2–3 条根、只盖最旧的几槽，环 24 槽），环里最旧有效根 ≤ 它 < 这一串的 txg ⇒ 这一串自己换下的一个都回收不了。
- 用例 `writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge`（`second_transaction_step_four_rollback.rs:1538`）：满环（txg 4–27）之后改坏盘 1 那一份最新版本的记账树根，普通可写挂载：写行 txg 28 核出对不上、两份一起隔离，挂载做成；txg 27 还在环里；写行换下的槽没被这一串发出去、挂载之后不是空闲槽；checker 红 I-2.1 / I-4.8 / I-7.4（坏盘自身）、I-3.1 / I-3.11 成立。
- 额外搜过、没找到的形（草稿，没入库）：崩溃恢复落到环里最旧的根（更新的根槽清零、之后第一条点名单元的记录验不过）再普通挂载、盘 1 上逐槽改坏：
  - 4 GiB 满环一格：80 个槽，6 格有隔离，全都挂成。
  - 空间准入照判的小盘：单元区 480–1024 槽，每档一格，所有槽逐个改坏，一次都没有「取号之后被落点拒」。
  - 日志 `logs/explore-c542-{3,4,5}.log`、`logs/explore-c542-big.log`。
  - 攻方那 16 格（`m2-final-code-r3-opus-output.md` Z16-a）靠的是挂载时回退，已删；准入照判的普通挂载上我没扫出等价的一格。这不是证明，推翻条件见第二节。

### 第 7 件 收口表第 43 行那一形：走得到

用例 `raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43`（`second_transaction_step_five_reuse.rs:915`）。

- 搭法：A（txg 3）、B（4）、C（5）；崩溃恢复抛弃 C（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`），实例 2 写行 6、暖机 7；再覆盖写四次（8–11）。
- 抬 F 到 5，上限是 8：
  - 抬之前镜像上 `raised_floor_lands_only_on_abandoned_roots(5)` 为真；
  - 之后 checker 只红 I-3.1，记账多于遍历：B 在 F 之下出了候选集，它独占、释放代 6 的单元 F = 5 回收不了；
  - 按 `history::classify_failure` 归到 `KNOWN_RED_FORMS[0]`。
- 对照：同一段抬到 6、7，全绿。
- 这一格只在「F 落在被抛弃根的 txg」时出现。实三随机历史与崩溃注入里不复现，是因为那两处都没有崩溃恢复抛弃根的操作。

### 第 8 件 恢复前镜像上的 I-3.1：查原因

用例 `plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced`（`second_transaction_step_four_rollback.rs:1695`）。

- 搭法：满环 txg 4–27，普通覆盖写 D（txg 28），根槽写成 0 并报错，进程在重发之前崩。恢复前镜像上 checker 只红 I-3.1。
- 机理：
  - 盘 0「记账 − 遍历」= 229 376 字节（14 槽），正好是只被 B（txg 4，被 D 的根槽写盖掉）引用的单元；
  - 这些单元在 C（txg 27）的账里是已释放、释放代 5。B 在环里时回收不了，所以 C 的账照算已分配；
  - B 的根槽没了之后遍历里没有 B；
  - checker 只把「已被某次恢复施加」的记录并进遍历（`walk.rs` `versions_applied_only_by_records` 的 `applied_by_a_recovery`），D 的记录不在其内。
- 恢复（由 D 的记录重建 D）后重开，全绿。
- 判法错在哪、该不该判：见第九节问题 1，我没改 checker。
- 用例钉的是今天的结局。追加的第 759 行变异把 checker 改成「未施加的记录也并进遍历」，这条用例红，红在多出一个 I-4.2（D 的记录并进来之后）。

### 第 9 件 影子账树表 0 条的被抛弃根：造得出

用例 `crash_recovery_abandoning_the_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them`（`second_transaction_step_three_formatted_pool.rs:1491`）。

- 搭法：
  - mkfs → M1（零单元）→ M2（写行 txg 3、暖机 4）；
  - M3 前 txg 3、4 根槽清零、写行那条记录点名的实例表盘 0 那份清零；M3 落到 (1, 2)，写行 txg 5 与 M2 的写行逐槽相同，隔离 0；
  - 写回根槽后，txg 3、4 按实例 3 的表判被抛弃；
  - 同一进程里第一个文件 txg 8（换下 50178、50242–50246，释放代 8），正常卸载把 F 抬到 8。
- 结局：卸载回收 50178 / 50246，但它们仍被被抛弃根 txg 3 引用（树表、分配记录树根），影子账每盘隔离这 2 槽；卸载那一串与 M4 都没写它们；M4 重算每盘仍 2 槽、账读不出 0；checker 无违例。
- 草稿量过：那一支交回 `None` 时，卸载与 M4 都隔离 0、M4 报 2 条账读不出。
- 实三说造不出，是因为他只看了 M3 那一次（M3 看不见被抛弃根，与 M2 逐槽相同、盖掉了它们）。F 抬过它们的释放代之后，那一支才有东西可护。

## 五、第 12 件命名：108 → 9

naming-lint 全仓：开工时「✗ 230 处」（crates 里 108 处），现在「✗ 131 处名字不合命名纪律（查了 261 个 .rs 文件、62720 个声明的名字）」（crates 里 9 处，其余 122 处在 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`，不归我）。

改法：
- 99 个名字：开头的冠词 `a_` 去掉，名字中间的 `_a_` 换成 `_the_`。所有调用点、文档里点名的反引号、`crates/mutations.tsv` 里的过滤串与必须红的测试名一起换。
- 5 个另起的：
  - `…_at_offset_k_of_a_publish_is_k_plus_one` → `…_at_each_offset_of_the_publish_is_that_offset_plus_one`
  - `a_swallowed_write_after_which_the_checker_flags_only_i_3_1_…` → `swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_invariant_…`
  - `root_c` → `abandoned_third_version_root`
  - `walked_under_c` → `walked_under_the_abandoned_third_version`
  - `walk_to_the_file_under_the_abandoned_root_c` → `walk_to_the_file_under_the_abandoned_third_version_root`
- 改完之后：
  - 每一行点名换过的变异行，都按它自己的 cargo 参数加 `--list` 跑过，点名的测试在过滤出来的清单里：100 行里 99 行在，第 173 行不在。第 173 行点名的测试名在上一次提交 346f5e6 改名时就没跟上（基线就不存在），我把它换成改名后的那条，并证红，见第七节。
  - 门禁 33 号绿。

剩的 9 处，逐个写原因：

| 位置 | 名字 | 为什么没改 |
|---|---|---|
| `bin/e156_allocation_basis_counts.rs:2915` | `k1_1_matches_…` | 实四甲在改这份文件，派发点名留给它 |
| `tests/first_transaction_step_six_recovery.rs:240` | `a_stale_tail_does_not_hide_the_record_above_it_that_recovery_must_apply` | kb 点名（`checks-owed.md:37`、`milestone/02-second-txn.md:223`），kb 不在我写范围；改了 kb 就指向不存在的测试（门禁 78 号判已还清表） |
| `tests/second_transaction_step_three_formatted_pool.rs:736` | `a_third_writable_mount_on_a_version_whose_rows_were_written_…` | kb 点名（`checks-owed.md:512`） |
| 同文件 `:1120` | `a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_…` | kb 点名（`checks-owed.md:429`、`decisions/22-单元原子性怎么合成.md:347`） |
| `tests/second_transaction_step_zero_layer0.rs:1297` | `a_unit_whose_only_later_write_…` | kb 点名（`checks-owed.md:514`）；又是层 0 文件，派发说那一组不改 |
| `tests/second_transaction_supplement_two_commit_generated_fallback.rs:359`、`:685`、`:701` | `a_raise_whose_…` 三条 | kb 点名（`checks-owed.md:470`） |
| `tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs:1079` | `a_leaf_of_a_two_level_lower_extent_segment_…` | kb 点名（`decisions/08-核心索引结构.md:409`） |

kb 点名的 8 条要改，得 kb 与代码同一轮一起换：书记员改 kb 那几处引文，实现员改名。
另外，`research/scripts/memory-peaks.tsv` 按命令记峰值，里面有 17 个我改过的旧测试名。改名之后那几行查不到，第一次按上限排队；我没改那张表（不在写范围）。

## 六、钉死值：改前 / 改后 / 原因

| 文件:位置 | 量 | 改前 | 改后 | 原因 |
|---|---|---|---|---|
| `first_transaction_on_device.rs:1860`（second-instance）、`:2611`（raise-rollback-floor） | 发布 C / D 的段形状 | `16+2+1+2` | `28+2+1+2` | 分配记录树按位置寻址（D8 已定项 14），一次覆盖写重写 7 个节点、两盘 14 次写；单元段 = 7 个别的单元 × 2 + 14。实际打出的行里 `allocation_record_tree_node_write_calls=14` |
| 同文件 `:2460`（failed_writable_mount） | 注入点 / 已落盘两次 / 失败账 / 窗口 | 33 / 15、13 / 2 / 30 | 53 / 27、21 / 2 / 50 | 写行 27 = 实例表 2 + 分配记录树 14 + 记账 / 映射 / 树表 6 + 记录 2 + 根槽 1 + 系统配置 2；暖机 21 = 分配记录树 10 + 6 + 2 + 1 + 2 |
| 同文件 `:2886`、`:2909`（failed_raise） | 注入点 / 已落盘一次 / 失败账 / 窗口 | 16 / 13 / 2 / 15 | 26 / 21 / 2 / 25 | SysPre 先写系统配置两盘 2 次（不属于任何一次发布）；空发布 21 次；窗口 2 + 21 + 2 |
| `second_transaction_supplement_three_fault_injection.rs:749` | 暖机第一次的第一个写 | 18 | 26 | 写行那次 15 → 23 次写（分配记录树 2 → 10） |
| 同文件 `:284` | 说谎设备只红 I-3.1 的种子 | 7463871032432355306 | 7463871032432355210 | 实三的比重表改动让同一种子生成的历史变了；大档 512 段在今天的代码上重扫 |
| 同文件那条的文档注释 | 大档里 `["I-3.1"]` 那一组的格数 | 13 | 14 | 同上重扫 |
| `first_transaction_regions.rs:33` 与测试期望表 | 区域行数 | 21 | 29 | 写清单 2026-09-25 改成按位置寻址之后 29 条写 |
| 同上 | 分配记录树 / 记账 / 映射 / 树表落点 | 50245 / 50246 / 50247 / 50248 | 叶 50245、50246，层级 1 50247、50248，根 50249；记账 50250、映射 50251、树表 50252 | 同上（layout/01 零那一节 t5–t12） |
| `system_configuration_rollback_floor_and_layout_identity.rs` 旧镜像那条 | 挂载报的成员 | `NoValidSystemConfiguration { 盘 0 }` | `SystemConfigurationIncompatBitsNotRecognized { 盘 0, 位图 }` | 第 10 件 |
| `system_configuration.rs` 单测两条 | 读者报的成员 | `NotSelfDescribing` | `IncompatBitsNotRecognized { 位图 }`（坏槽那一格仍是 `NotSelfDescribing`） | 第 10 件 |
| `second_transaction_step_three_formatted_pool.rs` | 各项 checker 判定 | — | 新用例只判「一条违例都没有」（I-8.7 / I-8.8 在那段历史里报不适用） | 新用例 |

## 七、证红（副本 `proof/`，自己的 target；每次改坏一处、跑那条用例所在的整个测试二进制、从主工作区拷回并 touch）

基线：每个要证的二进制先在不改动的副本上整跑一遍，都绿，基线红集为空。这些二进制是：
- `first_transaction_on_device` 16 过、`first_transaction_region_bytes` 4 过；
- fault_injection 9 过；fsync_drop 17 过、layout_identity 6 过、c519 1 过、core lib 121 过；
- formatted_pool 13 过、step_five 14 过、step_four 15 过、last_record_flag 8 过、harness lib 68 过。

被测代码里没有 `debug_assert` 先红的格（都是用例自己的断言或它调的入口）。「行」指今天 `crates/mutations.tsv` 的行号。

| 行 | 改坏哪一处 | 点名用例红在哪条断言 | 同一次还红了 |
|---|---|---|---|
| 602（实二的 616） | 抬 F 失败时二进制不用交回的已落盘账 | `failed_raise_…` `first_transaction_on_device.rs:2221`「同一段里失败之前已经落盘的发布各一行」left `[]` right `[(raise_rollback_floor,1,21)]` | 无 |
| 92、93、467、468、469、471、481、482、601、606、607 | 各自原样 | 点名那条都红（`proofs/on-device-rows.txt`） | 见日志 |
| 188 | 白名单去掉 `["I-3.1"]` 那一组 | `swallowed_write_after_which_…:293`「说谎的设备留下的 I-3.1 要被白名单豁免，不算新发现」 | 无 |
| 301 | C378 回卷 | `write_error_after_the_acquisition_…:790`「取号之后第 3 次写报错：…不回卷成 1」 | 无 |
| 749（新） | 区域表分配记录树根退回 50245 | `the_region_table_matches_…:184`「表里每一行都要有一条写落在它上面」left `["allocation_leaf_of_device_0" ×2]` | `…twenty_nine…` 那条 |
| 689 | 分配记录树那一判 `.ok_or` → `.expect` | 新用例 `:1424`「只有盘 0 的读者读分配记录树不许 panic」（先在 `allocation_record_tree.rs:766` panic） | 无 |
| 690 / 692（锚点跟改） | 准入结果丢掉 / 挪到择系统配置之后 | `mount_writable_with_only_device_zero_is_refused` :1294 / :1310（「盘 0 一次都没被读」left 2） | 盘 0 交两次那条 |
| 750（新） | 按盘表项数数 | `mount_writable_with_device_zero_handed_in_twice_…:1377`「盘 0 交两次：该报 …LowerBound，实际 …I-1.1」 | 无 |
| 751（新） | 择根遇读不到的根槽就 `return` | `read_only_mount_with_only_device_zero_…:1458` left (1, 3) right (1, 5) | 无 |
| 355 | C519 `all` → `any` | `c519_known_loss_…:242` 读回内容是 txg 6 的 | 无 |
| 752（新） / 700 / 708 | 布局不认识报成自证不过 / 读者收下位 0 | `mounting_a_pool_whose_…:270`（「实际 …NoValidSystemConfiguration」）；708 红在 core 单测 `system_configuration.rs:670` | 752、700 同时红 `unknown_layout_on_one_device_…` |
| 753（新） | 看过坏盘之后不再记布局不认识 | `unknown_layout_on_one_device_…:336`「盘 0 坏、盘 1 布局不认识：该报 incompat 位图不认识」 | 无 |
| 754（新，原 273 的改法） | 影子账树表 0 条那一臂交回 `None` | `crash_recovery_abandoning_…:1645`「被抛弃的两条根都认得出它们引用的落点」left 2 | 无 |
| 755（新，原 474 的改法） | 那一臂漏分配记录树节点 | 同一条 `:1662`「盘 0：槽 50246 仍被环里的被抛弃根引用，回收之后隔离着」 | 无 |
| 756（新） | checker 说明里少回退下界那一项 | `raising_the_floor_into_the_txg_…:965`「归到已知红清单第 1 条」实际 NewFinding | 无 |
| 757（新） | 已知红第 1 条空档判反 | 同上 | 无 |
| 758（新） | 挂载内回收按新根自己的 txg 当最旧有效根 | `writable_mount_on_the_newest_root_…:1647`「盘 0：写行那次换下的槽 50368 挂载之后还不是空闲槽」 | 同二进制另 11 条（回退、C558 等） |
| 759（新） | checker 把没施加的记录也并进遍历 | `plain_overwrite_whose_root_slot_…:1784`「恢复之前的镜像：只有 I-3.1 红」left `["I-3.1","I-4.2"]` | C558 那条、回退崩溃点那条 |
| 173（点名换成存在的那条） | 系统配置轮换挪到根槽 FUA 之前 | `publish_that_fails_on_the_system_configuration_slot_…:653`「C381 的前提：…txg 4 那条根却已经 FUA 落盘」 | 188 那条 |
| 183、189、615（锚点里的名字跟改） | 各自原样 | 点名那条都红（`proofs/row2-*.log`） | — |

- 每条新测试都有一行证过；新追加的 11 行（749–759）全部实跑证红。改名之后第 754–759 行又复证了一次。
- 点名换名的 94 行没有逐行跑变异。每行按自己的参数加 `--list` 核过点名的测试在过滤清单里（`logs/list-check.log`，99 行 ok，第 173 行修过、证过）。整表复跑归门禁 59 号。

## 八、`crates/mutations.tsv` 改动

- **末尾追加 11 行（第 749–759 行）**，变异名依次（都以「实四乙（…）：」开头，括号里写依据）：
  1. 区域表：分配记录树根的落点退回按位置寻址之前的 50245
  2. 可写设备数退回按盘表项数数，同一块盘交两次数成 2、放过这一判
  3. 择根时一个根槽读不到就当整环读完，只交盘 0 时择不到区域 2 上的最新根
  4. 系统配置读者把自证得过、incompat 位图不认识的槽报成自证不过
  5. 择系统配置时看过一块两槽都坏的盘之后就不再记布局不认识的槽
  6. 影子账把树表 0 条的被抛弃根当成账读不出（实三删掉的原第 273 行那一改法）
  7. 树表 0 条的被抛弃根漏掉它那棵分配记录树的节点（原第 474 行那一改法）
  8. checker 在 I-3.1 的说明文字里少写回退下界那一项机理标识（与实三删掉的原第 485 行同一处）
  9. 已知红第 1 条把「F 那个 txg 上的根全属于被抛弃的实例」判反
  10. 挂载内每写一条根按这条根自己的 txg 当环里最旧的有效根回收
  11. checker 把没被任何恢复施加过的记录也并进遍历
- **改了的既有行（100 行，实四甲的第 476 行不算）**：
  - 锚点跟着改（5 行）：183、189、615（锚点里有改名的函数）；690、692（准入调用改成数不同的设备身份）。
  - 只改变异名（1 行）：163（481 → 489）。
  - 改点名的测试（94 行）：
    - 689：接替实二八第 1 问，换成新用例；
    - 103：区域表测试改名；
    - 173：原点名的测试名不存在，换成改名后那条；
    - 其余 91 行：第 12 件的改名，过滤串与必须红的测试名一起换。行号：168、169、172、188、192、193、214、215、220、234、235、236、238、241、245、254、301、306、308、309、310、313、314、319、375、383、399、400、402、406、407、410、411、412、413、428、429、433、451、462、463、483、486、504、505、506、507、509、516、517、533–541、544、545、546、549、553、555、561、571、572、574、581、587、590、592、593、612、613、617–623、629、631、635、639、641、642、646、648。
- 门禁 33 号：「crates/mutations.tsv 754 条的原文各命中源码一次」。

## 九、交主 agent 的问题与牵连（条款没写、或不归我定的）

1. **第 8 件 checker 判法**（停下，没改 checker）：恢复前镜像是合法状态（D 的记录已持久、根槽写坏、进程崩），checker 今天判 I-3.1 红。
   - 红的机理：
     - 记账按最新根 C 的账算，里面还算着只被 B 引用、释放代 5 的单元；
     - B 的根槽被 D 那一次写盖掉了；
     - D 的记录没被任何恢复施加，`versions_applied_only_by_records` 不收它。
   - 三条路，都是推的、没量：
     - (a) 认成已知形态；
     - (b) 候选集收「记录已持久、根槽不在环里、还没被施加」的那一版。草稿变异 759 把条件放宽到「未施加的也收」，冒出 I-4.2，所以不能直接放宽；
     - (c) I-3.1 在这一格上报不适用，照 F 那一格的先例。
   - 今天的用例钉的是「只红 I-3.1、差值 = B 独占的字节」。改判法之后要跟着改：用例那条断言、第 759 行。
2. **第 6 件 C542**：所选根是最新根那一支，推理加用例判为走不到。崩溃恢复落到环里最旧根那一形在准入照判的小盘上扫过，没扫出烧号，但没有证明走不到。
   - C542 挪不挪已还清、那一行点名哪条用例，请主 agent 定；kb 不在我写范围。
3. **第 7 件**：收口表第 43 行那一形走得到（崩溃恢复抛弃根 + 抬 F 进被抛弃根的 txg）。
   - 清单那一条留着有对象了；
   - 随机历史与崩溃注入里没有「崩溃恢复抛弃根」的操作，所以它们复现不了；要不要给生成器加这一种操作，交主 agent；
   - `milestone/02-second-txn.md:382` 那一行点名的复现用例名早已不存在（`raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form`），书记员可换成新用例。
4. **第 10 件的两个取法**（条款没写，我取的）：
   - 池里一槽可择的都没有时，只要有一槽是布局不认识就报新成员（先于 `NoValidSystemConfiguration`）；
   - 带的是按盘序第一块、先读到的那一槽的位图。
   - 另外：一块盘可择、另一块盘布局不认识时，照旧择可择的那一份、不报；与今天「不认识的位」的行为相同，没改。
   - 两个新成员要不要写进 kb（D15 已定项 4 / 恢复失败成员的登记处），交书记员。
5. **第 4 件数设备身份**：
   - 同一身份交两次算一块，已做；
   - 池外的身份照样算一块，由取号前的逐盘核拒（实二八交回第 2 问后半，按代码读，没另跑）；
   - 盘表 [0, 0, 1] 这种「两个不同身份、一个重复」会过这一判，之后走到哪没查；
   - 「过半」合取照旧没单独实现。
6. **第 173 行**：原来点名的测试名（`…leaves_a_root_whose_units_the_next_publish_overwrites`）在上次提交 346f5e6 改名时就不在了，59 号复跑会报它没红。我换成了改名后的那条，并证红。
7. **基线就红、不归这一批的一条**：`second_transaction_supplement_three_bad_disk_input` 的 `every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking`。
   - 期望 `ReleaseTargetAlreadyReleased { … slot: 50368 }`，实际 50304，且被包成 `RowPublishAdmissionRefusedBeforeAcquisition`；
   - 开工快照上同一条命令同样红（`logs/base-bad_disk_input.log`）；
   - 我在这份文件里只改了名，没修它。
8. **故障注入大档在今天代码上的新发现**（第 2 件扫描顺带看到的，没修）：种子基起 512 段里 23 段报新发现，共 25 条。签名：
   - `["I-2.1","I-3.10","I-4.8","I-7.4"]` 11 条；
   - `["I-2.1","I-4.8","I-7.4","I-9.1"]` 7 条；
   - `ModelDisagreement 抬 F 的上限` 3 条；
   - `冷启动读回` 2 条；
   - `["I-2.1","I-3.10","I-3.11","I-4.8","I-7.4"]` 1 条；
   - `["I-2.1","I-3.1","I-3.10","I-4.8","I-7.4"]` 1 条。
   - 种子与注入点在 `logs/scan-findings.log`；
   - 大档 `#[ignore]` 那条今天跑会红。
9. **命名剩的 8 个 kb 点名的名字**：要书记员改 kb 引文、实现员同一轮改名（第五节表）。
   - `research/scripts/memory-peaks.tsv` 里有 17 个我改掉的旧测试名，不在写范围，没动。
10. **区域表那个 bin**：E142 第十五次跑起不再用它（`research/scripts/replay.sh:409` 起的注释）。今天只剩 `first_transaction_region_bytes` 测试钉着表与写清单一致。
    - kb `experiments/142-…` 第 191 行说的「21 行区域清单」是那时的事，要不要注一句，交书记员。

## 十、交回前的验证（主工作区，自己的 target `target-main`；末尾原样）

跑之前 `ps` 看过：没有 qemu-system、vm-bench.sh、e152-file-system-benchmark、fio；有别的会话的 cargo（`m2-layer0-scale-r2-opus` 副本里的一个 release 测试），各用各的 target，没等锁。所有 `cargo test` 与编出来的二进制都经 `run-with-memory-cap.sh 8G` 跑，线程经 `capped.sh 10`；没有撞上限（包装退出码都是 0）。

动到的测试二进制（整个二进制；名字含 layer0 的没跑）：

```text
lib:singlefs-core（final）  test result: ok. 121 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
lib:singlefs-checker（final）  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
lib:singlefs-harness（final）  test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.87s
lib:singlefs-format（final）  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
bin:first_transaction_on_device（final）  test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test:checker_known_bad_images（final）  test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.96s
test:first_transaction_region_bytes（final）  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test:parallel_line_one_sequential_write（final）  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.55s
test:second_transaction_parallel_line_one_last_record_flag（final）  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.84s
test:second_transaction_parallel_line_one_multi_unit_file（final）  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.38s
test:second_transaction_parallel_line_one_sequential_write（final）  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s
test:second_transaction_parallel_line_three_many_inodes（final）  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.55s
test:second_transaction_parallel_line_two_mounted_read（final）  test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
test:second_transaction_step_five_reuse（final）  test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.69s
test:second_transaction_step_four_rollback（final2）  test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.25s
test:second_transaction_step_three_formatted_pool（final）  test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.38s
test:second_transaction_step_zero_test_only_switches（final）  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.44s
test:second_transaction_supplement_three_bad_disk_input（final）  test result: FAILED. 8 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 31.48s
test:second_transaction_supplement_three_fault_injection（final）  test result: ok. 9 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 59.56s
test:second_transaction_supplement_three_random_history（final）  test result: ok. 21 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 405.68s
test:second_transaction_supplement_two_admission_formula（final）  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.08s
test:second_transaction_supplement_two_c519_whole_device_loss_after_warm_up（final）  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test:second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version（final）  test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.37s
test:second_transaction_supplement_two_instance_table_second_page_write（final）  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
test:second_transaction_supplement_two_multi_record_transaction_zero_publishes（final）  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
test:second_transaction_supplement_two_publish_failure_resent_unchanged（final）  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
test:second_transaction_supplement_two_release_checksum_quarantine（final）  test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.63s
test:second_transaction_supplement_two_root_ring_turn_in_one_mount（final）  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.20s
test:second_transaction_supplement_two_row_publish_checks_before_acquisition（final）  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.84s
test:second_transaction_supplement_two_tree_nodes_and_the_central_mapping（final）  test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.11s
test:second_transaction_supplement_two_tree_split（final）  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test:second_transaction_supplement_two_unreadable_abandoned_root_slot（final）  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.21s
test:system_configuration_rollback_floor_and_layout_identity（final2）  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.23s
test:system_configuration_per_device_redundancy（final）  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.14s
```

`second_transaction_supplement_three_bad_disk_input` 那 1 条红是基线红（第九节问题 7）。step_four 与 layout_identity 是最后一处改动之后重跑的（final2）。

fmt / clippy / build：

```text
cargo fmt --all -- --check：exit 0，无输出（logs/fmt-final.log 0 行）
cargo clippy --offline --keep-going --all-targets --all-features -- -D warnings 加 check.sh 那七条 -D clippy::…：exit 0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
cargo build --offline --all-targets：exit 0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.60s
```

门禁（判定行原样；33 号在追加 11 行之后重跑，74 号 SINGLEFS_GATE_FULL=1、CARGO_TARGET_DIR 指 target-main）：

```text
27 exit=0   ✓ 格式常量同步（41 个已登记，41 个在源码里被钉住）
33 exit=0   ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 754 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
53 exit=0   ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
74 exit=0   ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：
80 exit=0   ✓ 152 个实验二进制各自至少有一条绝对值断言（research/e7-index-bench/src/bin 下 148 份，别处 src/bin 下以 e<数字>_ 开头的 4 份）
89 exit=77   ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）
92 exit=0   ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，105 个格式常量里变了 7 个（checker 在同一次改动里跟了 7 个，按滞后表放行 0 个），都不欠 checker 跟进
93 exit=0   ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 54 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
94 exit=0   ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`；共享模块 1 份源码的正文 283 行里没有分支与循环
naming-lint exit=1   ✗ 131 处名字不合命名纪律（查了 261 个 .rs 文件、62725 个声明的名字）
```

89 号 exit 77 是「本次未跑」，不是通过。naming-lint 全仓红是 E142 装置那 122 处加第五节表里 9 处。

## 十一、`git diff --stat -- crates litmus` 原样（含实一至实三、实四甲与别的会话没提交的改动，分不出谁改的；我的在第三节；`system_configuration_rollback_floor_and_layout_identity.rs` 是未跟踪文件，不在这张表里）

```text
 crates/mutations.tsv                               |  374 +--
 crates/singlefs-checker/src/image.rs               |   77 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                |  441 ++--
 crates/singlefs-core/src/admission.rs              |    4 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  122 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   10 +-
 crates/singlefs-core/src/lib.rs                    |    1 -
 crates/singlefs-core/src/make_filesystem.rs        |   11 +-
 crates/singlefs-core/src/mount.rs                  | 1527 ++++++++----
 crates/singlefs-core/src/mounted_read.rs           |    7 +-
 crates/singlefs-core/src/recovery.rs               |  336 +--
 crates/singlefs-core/src/rollback_witness.rs       |  314 ---
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 ++-
 crates/singlefs-core/src/transaction.rs            |  371 ++-
 crates/singlefs-core/src/write_request_split.rs    |    2 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        |  103 +-
 .../src/bin/e158_root_choice_repair.rs             |   60 +-
 .../src/bin/first_transaction_on_device.rs         |  157 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               |  113 +-
 crates/singlefs-harness/src/device_log.rs          |    2 +-
 crates/singlefs-harness/src/fault_injection.rs     |   59 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             |  286 ++-
 crates/singlefs-harness/src/lib.rs                 |   43 +-
 crates/singlefs-harness/src/model.rs               |  296 ++-
 crates/singlefs-harness/src/model_comparison.rs    |   71 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |    2 +-
 .../tests/checker_known_bad_images.rs              |  773 +++---
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 ++--
 .../tests/second_transaction_step_four_rollback.rs | 2626 +++++++++++---------
 ...second_transaction_step_three_formatted_pool.rs |  566 +++--
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   97 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |    8 +-
 ...transaction_supplement_three_crash_injection.rs |   34 +-
 ...transaction_supplement_three_fault_injection.rs |   44 +-
 ..._transaction_supplement_three_random_history.rs |  201 +-
 ...transaction_supplement_two_admission_formula.rs |  119 +-
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
 ...n_supplement_two_release_checksum_quarantine.rs |   23 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 --------
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 ----
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
 ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
 ...second_transaction_supplement_two_tree_split.rs |    4 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +--
 .../system_configuration_mutability_classes.rs     |   35 +-
 79 files changed, 7128 insertions(+), 6419 deletions(-)
```

## 十二、没做什么

- 没走三方对抗；层 0（含 `second_transaction_step_zero_layer0.rs`，派发点名的那几处段形状没改）、QEMU、herd7、crates 变异整表（59 号）都归 `crash-verifier`；没跑全量 `cargo test`、`check.sh`、整轮门禁；没提交，没做任何 git 写操作。
- 改名换点名的 94 行变异没有逐行实跑，只核了过滤串（`--list`）。整表复跑留给 59 号。
- 第 8 件没改 checker 判法（交主 agent）；第 6 件崩溃恢复落到最旧根那一形没证明走不到，只扫过没扫出；第 7 件的修法没动。
- 故障注入大档的 23 段新发现、`bad_disk_input` 那条基线红没修（不归这一批）。
- kb、`research/`、`memory-peaks.tsv` 一处没改（kb 要跟着改的地方列在第九节）；kb 点名的 8 个测试名没改。
- E156 / E158 两个 bin 没碰（实四甲）。
- `first_transaction_on_device` 在真设备 / QEMU 上没跑，只跑了宿主上的单测。

## 十三、草稿与清理

草稿都在 `/tmp/claude-1000/impl-rbf-4b/`。

交回前删掉的（先 `du -sh`）：
- `target-main` 11G、`target-pristine` 8.2G、`target-proof` 2.9G、`target-scan` 1.5G；
- 开工快照 `pristine/` 5.8M、证红副本 `proof/` 5.8M、探索副本 `scan/` 5.9M。

删完目录 4.3M，留下：
- 这份报告、`progress.md`；
- `draft-rows.tsv`（追加的 11 行原样）、`changed-rows.tsv`（改过的既有行）；
- `proofs/`（每次证红的日志）、`logs/`（基线、扫描、主工作区各次运行、门禁）；
- `explore-sources/`（探索与扫描用的草稿测试源码，从 `scan/` 拷出，没进仓）；
- 证红与核对脚本（`prove.py`、`prove-batch.sh`、`list-check.sh`、`run-*.sh`、`batch-*.sh`）；
- 改名映射（`naming-*.json`、`naming-plan.txt`）；`fmt-before-*.rs`（rustfmt 之前的拷贝）。

这些都是验证草稿，不是实验产物，不入 `research/results/`。
