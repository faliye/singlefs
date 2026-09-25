# 实二一 交回：分配记录树与 extent 树按 key 空间定形状（D8 已定项 14，K1–K4）

实现员（implementation-writer），2026-09-24，时刻 UTC。改动直接落在主工作区 `crates/` 里（没有另开副本，所以没有单独的补丁可 `git apply --check`：
主工作区现状就是结果；这一轮 `crates/` 没有别的实现员在改，E158 执行员的 `e158_root_choice_repair.rs` 我没碰）。

## 四条验收各到哪

1. **K1 分配记录树**：做了。按绝对槽号按位置寻址，叶 k 罩一块盘上 `[k·812, (k+1)·812)`，缺席即全空闲（没有记录的一段不写节点），
   根按盘分流、根层由池几何定（4 GiB × 2 根在第 2 层、树高 3）。`PublishError::AllocationRecordsExceedOneNode` 与它那一整套准入
   （`publish_admission` / `publish_sequence_admission` 等）删掉；多于一片叶照写（随机历史墙取样点：一版里最多 1324 条分配记录，0 新发现）。
   Z4-1 / C544：树表 0 条的那一版写行时实例表 66 片、池十二次挂载都挂得上可写（`second_transaction_supplement_two_row_publish_checks_before_acquisition.rs`）。
2. **K2 extent 树**：做了。上段按 inode 号的位置（叶罩 143 个号、扇出 147），下段一个文件一棵按单元号的位置（叶罩 144 个单元、扇出 147）；
   单单元文件内联在上段叶（标签 2），两个及以上建下段（标签 1），标签 0 只有读者认。145 个单元的文件下段长到两层、再缩回一层有进程内用例。
3. **K4 挂载怎么读**：做了。挂载（`recovery` / `mount`）整棵读分配记录树；extent 树按需读：`MountedPoolForRead::open_file` 多一个
   `reader: &dyn PoolReader` 参数，打开文件时按位置走下去（`OpenFileForRead::extent_tree_reads_at_open()` 报读了几个节点）。
4. **崩溃点进层 0**：新文件 `crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs`，五条流：
   `ExtentInlineToLowerSegment`（内联 → 下段一层）、`ExtentLowerSegmentBackToInline`（下段 → 内联）、`ExtentLowerSegmentGrowsToTwoLevels`
   （144 → 145 个单元，下段长到两层）、`AllocationRecordTreeTwoLeavesPerDevice`（一次发布每盘重写叶 61 与叶 62）、
   `VersionWithoutFileRowPublishWithItsOwnTree`（树表 0 条的一版写 370 行、按位置建这一版自己的分配记录树，再暖机）。只编过、一条没跑。
   比已有流多罩的崩溃状态与多跑的步骤写在那个文件的模块注释里（第 3 条的单元写段太大，全量枚举不出来，那条用例的注里写了）。

另：D28 已定项 4 的树高从根节点头现读（`TransactionOutput::position_addressed_tree_heights_read_from_the_root_node_headers`）。C538 没做。

## 这一轮连带改了没动过的测试二进制（格式改动的连锁）

第一个事务从 8 个单元变 12 个（分配记录树在 4 GiB 两块盘上是五个节点），单元区落点整体后移。按规矩只跑动到的二进制；可第一轮跑下来
钉着旧布局的二进制一定红，我把 38 个没动过的非层 0、非战役二进制跑了一遍（`/tmp/claude-1000/impl-m2-keyspace/runs-baseline-*/summary.txt`），
红了 16 个：15 个逐个按新布局改（改的是钉死的槽号、单元数、释放 / 隔离槽数、写调用与字节数，每个数都写了从哪来；
换跨度复用那条与 240 槽挂载被拒那条的历史参数在草稿副本上重扫过），`first_transaction_region_bytes` 等 kb（见下）；
`first_transaction_step_six_recovery`（树表槽 50248 → 50252）是跑之前就先改了的。**超出「只跑动到的」那一条，列在这里**。

本轮之前就红、不是这一轮造成的（照原样留着）：
- `second_transaction_step_three_formatted_pool.rs::transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write`：
  回退见证（`recovery::replay_journal` 里 `choose_system_configuration`，别的会话 18:55 UTC）多读一次系统配置槽 0，「第 2 次读」落到了别处。
- `checker_known_bad_images.rs::the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`：checker 清单多了 I-7.10 / I-7.11（回退见证），
  坏镜像语料没跟上。
- `second_transaction_step_four_rollback.rs` 两条回落到 C 的用例：回退见证那一格，`choose_root` 交回 (1, 3) 而 (2, 8) 读得出。

等书记员写 kb 之后另派的：`first_transaction_region_bytes.rs`（1 条红）与 `src/first_transaction_regions.rs` 的 21 行区域表照规矩从
`layout/01-first-txn.md` 抄，我没改；E142 产物、门禁 55 / 69 跟着要重跑。

## 这一轮写过的文件（我自己列；`crates/` 里别的会话此前留下的未提交改动 `git diff --stat` 分不开）

新建（未跟踪）：
- `crates/singlefs-core/src/allocation_record_tree.rs`（分配记录树几何、按位置的节点、规划、读者）
- `crates/singlefs-core/src/extent_tree.rs`（extent 树两段的几何、上段叶条目 113 与标签、下段节点）
- `crates/singlefs-checker/src/position_addressed.rs`（checker 自己那份两棵树的几何，只共享格式常量）
- `crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs`（新层 0 流，只编过）

改过（已跟踪，或别的会话新建、我接着改的）：
- `crates/mutations.tsv`（见下一节）
- `crates/singlefs-format/src/lib.rs`（八个新常量与字节表单测）
- `crates/singlefs-core/src/`：`lib.rs`、`transaction.rs`、`mount.rs`、`allocator.rs`、`recovery.rs`、`mounted_read.rs`、`write_accounting.rs`
- `crates/singlefs-checker/src/`：`lib.rs`、`walk.rs`
- `crates/singlefs-harness/src/`：`model.rs`、`model_comparison.rs`、`history.rs`、`bad_disk_input.rs`、`bin/e156_allocation_basis_counts.rs`（只删撞墙那一支与两处注释）
- `crates/singlefs-harness/tests/`（36 个）：`checker_known_bad_images.rs`、`first_transaction_step_five_publish.rs`、`first_transaction_step_six_recovery.rs`、
  `parallel_line_one_sequential_write.rs`、`second_transaction_mapping_node_admission.rs`、`second_transaction_parallel_line_one_multi_unit_file.rs`、
  `second_transaction_parallel_line_one_sequential_write.rs`、`second_transaction_parallel_line_two_mounted_read.rs`、`second_transaction_parallel_line_three_many_inodes.rs`、
  `second_transaction_step_one_overwrite.rs`、`second_transaction_step_three_formatted_pool.rs`、`second_transaction_step_three_second_instance.rs`、
  `second_transaction_step_four_rollback.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_supplement_one_write_accounting.rs`、
  `second_transaction_supplement_three_bad_disk_input.rs`、`second_transaction_supplement_three_random_history.rs`、
  `second_transaction_supplement_two_accounting_node_full.rs`、`second_transaction_supplement_two_admission_formula.rs`、
  `second_transaction_supplement_two_c533_row_publish_record_without_its_root.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`、
  `second_transaction_supplement_two_instance_table_chain.rs`、`second_transaction_supplement_two_instance_table_second_page_write.rs`、
  `second_transaction_supplement_two_multi_record_transaction_zero_publishes.rs`、`second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit.rs`、
  `second_transaction_supplement_two_reused_record_overlap.rs`、`second_transaction_supplement_two_row_publish_admission.rs`、
  `second_transaction_supplement_two_row_publish_checks_before_acquisition.rs`、`second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs`、
  `second_transaction_supplement_two_tree_split.rs`、`second_transaction_supplement_two_unreadable_abandoned_root_slot.rs`

没碰：`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（`cargo fmt --all --check` 只剩它五处，E158 执行员的）、
`src/first_transaction_regions.rs` 与 `tests/first_transaction_region_bytes.rs`（等 kb 布局）、所有名字含 layer0 的已有文件、`litmus/`。

手工编辑之外：五个测试文件与格式化相关的改动是对**我自己改过的文件**逐个跑 `rustfmt --edition 2021`；另有两处小改动（`checker_known_bad_images.rs`
的单元表常量块、两行注释）是用 Bash 里的 python / sed 落的（本该走 Edit），内容与 Edit 等价，列在这里。

## `transaction.rs` / `mount.rs` / `allocator.rs` 里动过的函数（给实二五；按我这一轮每次编辑落点所在的函数整理，可能多列相邻的一两个）

- `transaction.rs`
  - 新增：`settle_the_allocation_record_tree`、`settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`、`build_allocation_record_tree`、
    `build_extent_tree`、`plan_the_extent_tree_after_this_publish`、`prepare_the_version_publish`、`prepare_the_row_publish_on_a_version_without_file`、
    `prepare_the_publish_without_units`、`allocation_record_tree_node_of_role`、`role_of_allocation_record_tree_node`、`role_of_extent_upper_node`、
    `extent_tree_roles_in_bump_order`、`position_addressed_tree_heights_read_from_the_root_node_headers`；类型 `ExtentTreeNodeOrigin`、`ExtentTreePlan`、
    `PositionAddressedTreeHeights`、`MappedPositionAddressedTrees`、`ReleaseChecksumCheck`、`SettledPublish`、`SettledRowPublishOnAVersionWithoutFile`、
    `BuiltAllocationRecordTree`、`BuiltExtentTree`；常量 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`；`TransactionUnit` 三个新成员
    （`AllocationTreeNodeBelowTheRoot`、`ExtentLowerNode`、`ExtentUpperNodeBelowTheRoot`）与它们在 `tag` / `unit_class` / `placement` / `span_slots` 里的分支。
  - 删掉：`publish_admission`、`publish_sequence_admission`、`rewritten_role_counts_of_a_publish_sequence`、`rewritten_roles_of_a_publish_without_content`、
    `multi_level_tree_nodes_of_a_publish_sequence_without_content`、`admission_of_one_publish`、`after_a_publish_without_content`、`build_allocation_record_node`、
    `placements_to_release_on_a_version_without_file`、`refuse_when_the_allocation_records_do_not_fit_one_node`、
    `release_check_and_admission_of_a_row_publish_on_a_version_without_file`、`version_without_file_row_publish_admission`、
    `publish_instance_table_after_the_release_check`；类型 `MultiLevelTreeNodesOfAPublishWithoutContent`、`MultiLevelTreesBeforeAPublishWithoutContent`、
    `TreeNodeWrittenBy`、`PublishSequenceRefusal`、`PlacementsReleasedByTheRowPublish`；`PublishError::AllocationRecordsExceedOneNode` 与 extent 树装不下那个成员；
    新成员 `PublishError::AllocationRecordTreeRewriteSetDidNotSettle`。
  - 改过：`publish_version_of_trees`、`publish_version_of_trees_holding_one_data_unit`、`publish_admitted`、`publish_without_units`、
    `publish_instance_table_on_version_without_file`、`build_file_version_units`、`build_inode_tree_units`、`carried_file_version_units`、`carried_unit`、
    `inode_record_writes`、`placements_released_by_this_publish`、`placements_to_release_via_mapping`、`previous_roles_replaced_by_this_publish`、
    `has_a_placement_in_the_previous_version`、`is_a_page_of_the_instance_table`、`mapping_key_carried_from`、`mapping_locations_for_key`、
    `multi_level_tree`、`plan_the_multi_level_trees`、`PublishPlan::resolve`、`PublishShape::rewritten_roles`（注释）、`rewritten_roles`、`roles`、`highest`、
    `with_writes`、`format_time_tree_table_to_release`、`upper_root_level`、`warm_up_after_journal_counter`、`instance_table_chain_to_release`（注释），
    与几条单测（`every_transaction_unit_names_its_class_tree_and_placement_rule`、`second_file_object_in_the_same_checkpoint_continues_birth_sequences_instead_of_restarting_at_zero`）。
- `mount.rs`
  - 新增：`dry_run_of_the_publishes_after_acquisition`、类型 `DryRunRefusal`。
  - 删掉：`refuse_publishes_before_acquisition_that_do_not_pass_admission`、`placements_of_the_publishes_after_acquisition_on_a_copy`、`take_placement_for_role`、
    类型 `PlacementRefusedOnTheCopy`、`PlacementsOnTheCopy`。
  - 改过：`establish_instance`、`allocator_of_version_without_file`、`rebuilt_allocator`、`placements_referenced_by_root`、`placements_taken_by`、
    `publish_empty_after`、`publish_rows_on_file_version`、`empty_publish_plan_after_file_version`、`empty_publish_plan_after_version_without_file`、
    `row_publish_plan_on_file_version`、`raise_rollback_floor`；`MountError` 的 `RowPublishAdmissionRefusedBeforeAcquisition` /
    `WarmUpAdmissionRefusedBeforeAcquisition` / `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 三个成员的文档。
- `allocator.rs`
  - 改名：`allocation_record_node_of_the_version_without_file` → `allocation_record_tree_of_the_version_without_file`，
    `note_allocation_record_node_of_the_version_without_file` → `note_allocation_record_tree_of_the_version_without_file`；
    新增 `forget_the_allocation_record_tree_of_the_version_without_file`、类型 `AllocationRecordTreeOfTheVersionWithoutFile`。
  - 改过：`PoolAllocator::new`、`make_room_for_record_on_device`、`placements_referenced_by_abandoned_roots`。

## crates/mutations.tsv 改了哪些行（按变异名）

门禁 33：`crates/mutations.tsv 661 条的原文各命中源码一次` ✓（最后一次改表之后复核过）。
表从我开工时的 648 行变成 661 行：删掉 29 行、换成 29 行新写的（守改写之后同一处的同一类错）、表末追加 13 行；另 12 行同名只改了锚点、点名测试或参数。
行号不稳（别的会话会在表末追加），下面按名字列；证红表里的行号是今天的。

### 删掉的 29 行（旧名；它们守的代码随 K1 / K2 / 预演改法删掉或改写了，锚点在今天的源码里不存在）

- 分配记录树装不下不判
- 增补 2 第 20a 行：取号之前只算写行那一次，暖机那几次空发布不算（写行发完、暖机才被拒，实例代号已经烧掉）
- 增补 2 第 20a 行：取号之前只把第一次暖机算进去（写行装得下、暖机第 2 次装不下的池照样放行）
- 增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1）：分配记录墙差一，812 条正好装满也拒（> 写成 >=）；逼近墙的取样点判出
- 增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1 同一处）：分配记录墙差一；812 条那一格的边沿用例判出
- 增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：模型的分配记录墙只看沿来路的上界、不看镜像上的真条数
- 增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：抬 F 做完几次之后被墙拒，基数仍数这一步起点那一版
- 增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录节点先于实例表取落点
- C483 ②：挂载态打开时 extent 树根只按位置提示读（不经映射回退，攻方腿量到的「整池挂不上」）
- 实例表第二片：取号之前不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才在发布路径里撞上，号烧掉）
- 并行线一：extent 树要长内部节点也不拒（内部条目格式没有条款那一格被跳过）
- 并行线一：从盘上重建上一版只读 extent 根兼叶的第一条记录（多单元文件重开之后只剩第一个数据单元）
- 步 1 验收第 4 条（第 331 行那条锚点腐化了，照原意重写锚点）：extent 叶记录的指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）
- 增补 2 收口表第 38 行顺带发现 O1：影子账不认树表 0 条那一版自己那片分配记录树节点（被抛弃根的那一片回退之后是空闲槽）
- 增补 2 收口表第 38 行顺带发现 O1（同一处）：两片实例表链的被抛弃根，隔离里少那片分配记录树节点
- 增补 2 收口表第 39 行那一族：取号之前在分配器的拷贝上取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）
- 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不记根（不转环、不回收），拷贝上被拒的那一次与真发的不同
- 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不释放换下的落点（回退到环里最旧的根时写行当场回收的几片拷贝上看不见，真发得出来的回退被拒）
- 实例表第二片 × 增补 2 收口表第 39 行那一族：发完之后比对时，树表 0 条那一版上写行真取到的落点只认第 0 片（第 1 片起的点名项被错配到别的角色上，挂载自己的断言判出）
- 树分裂 取号之前的推算不删记账树节点的旧映射 key（推出来的重写数与真发布的对不上）
- 增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉；树分裂换锚点：多带节点容量一个参数）
- 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出（树分裂换锚点：准入按重写角色数算）
- 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出（树分裂换锚点：准入按重写角色数算）
- 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点（拷贝上取的与真发的不同，挂载自己的断言判出；树分裂换锚点：每一次的角色表按次序排好）
- 树分裂 取号之前在拷贝上取落点时按固定的四个固定点角色推（不按两棵多层树的形状，多层池挂载时拷贝上取的与真发的对不上，挂载自己的断言判出）
- 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）（树分裂换锚点：拷贝上那一串的形状表第 0 次）
- 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）（树分裂换锚点：每一次的角色表按次序排好）
- 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行不释放被换下的那条实例表旧链（经映射那一路从这一轮起跳过实例表；回退到环里最旧的根时写行当场回收的那一片拷贝上看不见）（树分裂换锚点：第 0 次释放那一段按两棵多层树的形状重排过）
- 实二二三 h（Z4-1）：树表 0 条的一版上写行的释放核与条数准入不在取号之前判（装不下时取号之后才拒，每试一次烧一个号）

### 替代它们的 29 行（新名；大体按上一节的次序一一对应）

- K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出）
- 增补 2 第 20a 行：取号之前的预演只演写行那一次、暖机那几次不演（预演与真发取到的落点对不上，挂载自己的断言判出）
- 增补 2 第 20a 行：取号之前的预演只演第一次暖机（计划推两次暖机的池上预演与真发对不上，挂载自己的断言判出）
- K1：分配记录树根之下的节点头里 key 区间的上端写成起点（按位置规定罩的那一段写错；越过原分配记录墙的取样点照跑 checker 判出）
- K1（同一处）：分配记录树根之下的节点头里 key 区间的上端写成起点；越过 812 条那一段写死的历史每一步跑 checker 判出
- K1 × 模型：分配记录树一次至多重写几个节点的上界少数了每层末端那一个节点
- K1 × 模型：分配记录树一次至多重写几个节点的上界不往下迭代（停在单元区里全部节点，占槽上界放得太宽）
- 增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录树节点先于实例表取落点
- C483 ② × K4：打开文件时 extent 树节点只按位置提示读（经映射回退那一路查不到任何 key，提示过期就打不开）
- 实例表第二片：取号之前的预演不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才撞上，号烧掉；预演走的是发布路径落盘之前那一段，这一处两边共用；树表 0 条的一版上写行那一处）
- K2：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元装满一片叶时白长一层）
- 并行线一：从盘上重建上一版只取第一个文件的第一个数据指针（多单元文件重开之后只剩第一个数据单元）
- 步 1 验收第 4 条 × K2：extent 上段叶条目内联的数据指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）
- 增补 2 收口表第 38 行顺带发现 O1：影子账不认树表 0 条那一版自己那棵分配记录树的节点（被抛弃根的那几片回退之后是空闲槽）
- 增补 2 收口表第 38 行顺带发现 O1（同一处）：两片实例表链的被抛弃根，隔离里少那棵分配记录树的节点
- 增补 2 收口表第 39 行那一族：取号之前的预演取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）
- 增补 2 收口表第 39 行那一族：发布取完落点之后不记根（不转环、不回收；取号之前的预演与真发走同一段，两边一起不回收）
- 增补 2 收口表第 39 行那一族：发布不释放换下的落点（取号之前的预演与真发走同一段；回退到环里最旧的根时写行当场回收的几片哪一边都看不见）
- 实例表第二片：树表 0 条的一版上写行只给第 0 片取落点（第 1 片没有落点，装链时查不到）
- K4：打开文件时按需读 extent 树的节点数多记一倍（实现自报的与块层数到的对不上）
- 增补 2 第 20a 行：取号之前的预演报了写行那次的错也照样取号（取号之后发布才被拒，实例代号已经烧掉）
- K2：extent 上段叶条目内联那一种的标签写成 3（格式登记的是 2）
- K1 × checker：分配记录树根按位置规定罩的那一段上端写成 0xFE…（checker 自己那份几何算错，第一个事务的节点头对不上它）
- K2：extent 下段叶里的记录不判 offset 段是不是净荷容量的整数倍（单元序号写进 offset 段的镜像报的不是位置那一条）
- K1：从盘上读分配记录树时不判记录落在它所在叶按位置罩的那一段里（被抛弃根那棵账里挪到单元区末尾的记录报成别的成员）
- K2 × checker：extent 上段节点按位置规定罩的那一段上端多罩一个 inode（checker 自己那份几何算错，第一个事务的上段根兼叶对不上它）
- K2 × checker：extent 下段节点按位置规定罩的那一段上端少罩最后一个单元的字节（checker 的 I-1.1 在多单元文件上判红）
- C544 / Z4-1：重开时从树表 0 条那一版的分配记录树重建账，却不记下那棵树（下一次写行不知道上一版的节点）
- C544 / Z4-1 × K1：分配记录按叶分装时按槽号除以叶宽取余（记录装进不罩它的叶，下一次重开读不回那棵树）

### 同名、只改锚点 / 点名测试 / 参数的 12 行

- 步 3：不写行（实例表照抄）
- 步 3：只做过 mkfs 的池上零单元暖机的反向链写 0
- 步 3：空池挂载的零单元暖机把回退下界写成 1（流与第一个事务那条不再逐项相同）
- 增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：checker 数一条根下的分配记录只数一半
- 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出
- 增补 2 收口第 26 行：走读数据单元时不判出生身份（I-1.2 / I-4.2 的调用点被摘掉）
- C512（2026-09-23 用户定案）：重开时从这一版的分配记录树重建账，却把带已释放标志的记录丢掉（被换下的那一片当场成了空闲槽）
- 增补 2 第 21 行（C366，挂载层）：带文件的一版上暖机的 jsn 取上一版的 txg + 1
- 实例表第二片写路径：树表 0 条的一版上写行时旧链只释放第 0 片
- 树分裂 树高从根节点头读成层级（不加一，D8 已定项 11 ⑤ / D28 已定项 4）
- 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出（树分裂换锚点）
- 实二二三 h（D18 已定项 11 第五个合取）：取号之前的预演不查写行那次经映射换下的映射条目的位置项（坏映射条目在取号之后才报，号烧掉）

### 表末追加的 13 行

- K1 / K2 × D28 已定项 4：按 key 空间定形状的两棵树的高从根节点头读成层级（不加一）
- K1 单测：分配记录树根的层级取「切出来的格数小于 4」的最低一层（4 GiB 两盘多长一层）
- K1 单测：分配记录树根的层级按八倍扇出判装不装得下（1 TiB 两盘少长一层：第 2 层切出 980 格，装得下 1352 就停在第 2 层）
- K1 单测：分配记录树按槽号找叶时叶宽算成两倍（叶的边界不在 812 的整数倍上）
- K1 单测：记录变了只重写那片叶、不连它的祖先（父节点里指它的那条子指针成了旧的）
- K1 单测：内部条目的 key 不在孩子那一层的格点上也认成孩子
- K2 单测：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元白长一层）
- K2 单测：extent 树上段根的层级取「罩的 inode 数不小于最大 inode 号」的最低一层（inode 143 还算在第 0 片叶里）
- K2 单测：extent 上段叶条目不认识的标签当成「没有单元」
- K1 × checker 单测：checker 那份分配记录树根的层级按四倍扇出判装不装得下
- K1 × checker 单测：checker 判记录落不落在叶里只看起点、不看末槽
- K1 单测：分配记录树根之下的节点的步号把盘与层级写反
- K2 × 格式常量：extent 上段一片叶罩的 inode 数写成 142（与 (16384 − 163) ÷ 113 对不上）

### 基线（不改动的副本，整个测试二进制跑一遍；证红只看基线红集之外）

- `-p singlefs-core --lib`：全绿
- `-p singlefs-checker --lib`：全绿
- `-p singlefs-format --lib`：全绿
- `-p singlefs-harness --test second_transaction_step_three_second_instance`：全绿
- `-p singlefs-harness --test second_transaction_step_three_formatted_pool`：红 transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write
- `-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter`：全绿
- `-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks`：全绿
- `-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain`：全绿
- `-p singlefs-harness --lib`：全绿
- `-p singlefs-harness --test checker_known_bad_images`：红 the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
- `-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission`：全绿
- `-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping`：全绿
- `-p singlefs-harness --test second_transaction_step_one_overwrite`：全绿
- `-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file`：全绿
- `-p singlefs-harness --test second_transaction_step_four_rollback`：红 with_the_shadow_ledger_off_and_every_root_of_the_rollback_instance_unreadable_the_recovery_falls_back_onto_the_abandoned_root_and_reads_a_torn_unit,without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
- `-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write`：全绿
- `-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read`：全绿
- `-p singlefs-harness --test first_transaction_step_five_publish`：全绿
- `-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition`：全绿

### 逐条（行号是今天表里的；「红在」是点名那条测试 panic 的位置与消息开头）

- 第 10 行 K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree` 红，红在 `crates/singlefs-core/src/allocation_record_tree.rs:307:9 | 分配记录（盘 DeviceIdentity(0) 槽 SlotNumber(50176) 跨 2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过`；同时红：allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records, cold_start_reads_the_second_content_and_the_pool_checker_stays_green, content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched, damage_probes_after_the_overwrite_tell_the_new_unit_from_the_released_one, overwrite_publishes_the_second_version_through_the_same_commit_shape, publish_running_out_of_space_midway_leaves_the_allocator_as_it_was, release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released, release_reports_a_mapping_entry_narrower_than_its_field_table_instead_of_slicing_past_it, release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking, release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue, released_placements_are_not_handed_out_again_before_reclaim_exists
- 第 21 行 步 3：不写行（实例表照抄）：改 `crates/singlefs-core/src/mount.rs` → `remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version` 红，红在 `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs:156:5 | assertion `left == right` failed: 写行发布重写实例表 + 四个固定点单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9）+ 分配记录树根之下那四个节点`
- 第 61 行 步 3：只做过 mkfs 的池上零单元暖机的反向链写 0：改 `crates/singlefs-core/src/mount.rs` → `writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold` 红，红在 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:992:5 | assertion `left == right` failed`；同时红：a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold, rolling_back_to_a_warm_up_root_before_any_file_version_rewrites_only_the_instance_table, the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool, transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write
- 第 102 行 增补 2 第 20a 行：取号之前的预演只演写行那一次、暖机那几次不演（预演与真发取到的落点对不上，挂载自己的断言判出）：改 `crates/singlefs-core/src/mount.rs` → `writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds` 红，红在 `crates/singlefs-core/src/mount.rs:1764:9 | assertion `left == right` failed: 取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串`；同时红：writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds
- 第 103 行 增补 2 第 20a 行：取号之前的预演只演第一次暖机（计划推两次暖机的池上预演与真发对不上，挂载自己的断言判出）：改 `crates/singlefs-core/src/mount.rs` → `writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds` 红，红在 `crates/singlefs-core/src/mount.rs:1764:9 | assertion `left == right` failed: 取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串`
- 第 143 行 K1：分配记录树根之下的节点头里 key 区间的上端写成起点（按位置规定罩的那一段写错；越过原分配记录墙的取样点照跑 checker 判出）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:348:5 | 新发现（checker 的判红、模型、执行器的判定与 panic）：`
- 第 144 行 K1（同一处）：分配记录树根之下的节点头里 key 区间的上端写成起点；越过 812 条那一段写死的历史每一步跑 checker 判出：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `overwrites_raising_the_floor_and_rolling_back_past_812_allocation_records_all_succeed` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:708:5 | assertion `left == right` failed: NewFinding { signature: CheckerViolations { invariants: ["I-1.1"] }, observation: FailureObservation { position: StartingPoint`
- 第 145 行 K1 × 模型：分配记录树一次至多重写几个节点的上界少数了每层末端那一个节点：改 `crates/singlefs-harness/src/model.rs` → `allocation_record_tree_nodes_rewritten_by_the_first_file_version_on_4_gib_are_at_most_nine` 红，红在 `crates/singlefs-harness/src/model.rs:1959:9 | assertion `left == right` failed: 单元区里的全部节点：每盘叶 61..=322、第 1 层 0..=1，加根`
- 第 146 行 K1 × 模型：分配记录树一次至多重写几个节点的上界不往下迭代（停在单元区里全部节点，占槽上界放得太宽）：改 `crates/singlefs-harness/src/model.rs` → `allocation_record_tree_nodes_rewritten_by_the_first_file_version_on_4_gib_are_at_most_nine` 红，红在 `crates/singlefs-harness/src/model.rs:1964:9 | assertion `left == right` failed`
- 第 147 行 增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：checker 数一条根下的分配记录只数一半：改 `crates/singlefs-checker/src/walk.rs` → `allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:467:5 | assertion `left == right` failed: 起点（txg 0）、挂载的零单元写行与暖机（txg 1、2）、第一个文件（txg 3）`
- 第 155 行 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出：改 `crates/singlefs-core/src/mount.rs` → `allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:348:5 | 新发现（checker 的判红、模型、执行器的判定与 panic）：`
- 第 267 行 增补 2 收口第 26 行：走读数据单元时不判出生身份（I-1.2 / I-4.2 的调用点被摘掉）：改 `crates/singlefs-checker/src/walk.rs` → `the_birth_identity_bad_images_redden_only_their_own_invariant` 红，红在 `crates/singlefs-harness/tests/checker_known_bad_images.rs:2156:9 | assertion `left == right` failed: 判红的该只有 I-4.2：[("I-1.1", Holds), ("I-1.2", Holds), ("I-1.3", Holds), ("I-1.4", Holds), ("I-1.6", Holds), ("I-1.7", Holds), ("I-`；同时红：the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
- 第 285 行 C512（2026-09-23 用户定案）：重开时从这一版的分配记录树重建账，却把带已释放标志的记录丢掉（被换下的那一片当场成了空闲槽）：改 `crates/singlefs-core/src/mount.rs` → `the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool` 红，红在 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:705:5 | assertion `left == right` failed: 两盘各一条记录罩着被换下的那一片`；同时红：transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write
- 第 359 行 增补 2 第 21 行（C366，挂载层）：带文件的一版上暖机的 jsn 取上一版的 txg + 1：改 `crates/singlefs-core/src/mount.rs` → `c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_warm_up_counter.rs:252:5 | assertion `left == right` failed: jsn 从读得出的最大号 3 接着数（4、5、6），txg 从 max(根环 4, 记录 3) + 1 = 5 起：两个量各走各的`
- 第 363 行 增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录树节点先于实例表取落点：改 `crates/singlefs-core/src/transaction.rs` → `c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_presumed_clause_checks.rs:162:5 | 树表 0 条的一版上写行：实例表（槽 50246）排在分配记录节点（槽 50244）之前 bump`
- 第 376 行 C483 ② × K4：打开文件时 extent 树节点只按位置提示读（经映射回退那一路查不到任何 key，提示过期就打不开）（改过锚点 / 点名测试 / 替换文之后重证）：改 `crates/singlefs-core/src/mounted_read.rs` → `an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs:395:10 | 打开第一个文件: ExtentTreeWalk(MappingMiss { slot: SlotNumber(50240) })`；同时红：an_extent_tree_root_moved_without_updating_the_central_mapping_is_still_unreadable_after_the_hop
- 第 393 行 实例表第二片：取号之前的预演不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才撞上，号烧掉；预演走的是发布路径落盘之前那一段，这一处两边共用；树表 0 条的一版上写行那一处）（改过锚点 / 点名测试 / 替换文之后重证）：改 `crates/singlefs-core/src/transaction.rs` → `writable_mount_follows_the_chain_and_refuses_a_page_missing_from_the_allocation_records_before_acquisition` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_instance_table_chain.rs:403:18 | 第二片不在账里：要在取号之前拒绝：Ok("挂上了")`
- 第 415 行 K2：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元装满一片叶时白长一层）：改 `crates/singlefs-core/src/extent_tree.rs` → `the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline` 红，红在 `crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs:291:9 | assertion `left == right` failed: 144 个单元的下段`；同时红：units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
- 第 417 行 并行线一：从盘上重建上一版只取第一个文件的第一个数据指针（多单元文件重开之后只剩第一个数据单元）：改 `crates/singlefs-core/src/recovery.rs` → `a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping` 红，红在 `crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs:464:5 | assertion `left == right` failed: 写行与暖机照抄三个数据单元的指针`；同时红：a_crash_before_the_second_record_of_a_two_record_publish_keeps_the_old_file, units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte, without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
- 第 423 行 步 1 验收第 4 条 × K2：extent 上段叶条目内联的数据指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）：改 `crates/singlefs-core/src/transaction.rs` → `cold_start_reads_the_second_content_and_the_pool_checker_stays_green` 红，红在 `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs:341:5 | assertion `left == right` failed: 冷启动择 B 的根、读回第二次的内容`；同时红：damage_probes_after_the_overwrite_tell_the_new_unit_from_the_released_one
- 第 474 行 增补 2 收口表第 38 行顺带发现 O1：影子账不认树表 0 条那一版自己那棵分配记录树的节点（被抛弃根的那几片回退之后是空闲槽）：改 `crates/singlefs-core/src/mount.rs` → `rolling_back_before_any_file_version_isolates_the_allocation_record_node_only_the_abandoned_roots_reference` 红，红在 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:1371:5 | assertion `left == right` failed: 实例 2 那片实例表 2 槽加那五个分配记录树节点 5 槽，逐盘`；同时红：transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write
- 第 475 行 增补 2 收口表第 38 行顺带发现 O1（同一处）：两片实例表链的被抛弃根，隔离里少那棵分配记录树的节点：改 `crates/singlefs-core/src/mount.rs` → `rollback_isolates_every_page_of_the_instance_table_chain_that_only_abandoned_roots_reference` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_instance_table_chain.rs:628:5 | assertion `left == right` failed: 实例 4 那张表的第 0 片 2 槽、第二片 2 槽、实例 4 那棵分配记录树七个节点 7 槽，逐盘`
- 第 497 行 增补 2 收口表第 39 行那一族：取号之前的预演取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）：改 `crates/singlefs-core/src/mount.rs` → `a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:528:5 | assertion `left == right` failed: NewFinding { signature: ModelDisagreement { aspect: "拒绝之前写了盘" }, observation: FailureObservation { position: Operation(16), op`
- 第 498 行 增补 2 收口表第 39 行那一族：发布取完落点之后不记根（不转环、不回收；取号之前的预演与真发走同一段，两边一起不回收）（改过锚点 / 点名测试 / 替换文之后重证）：改 `crates/singlefs-core/src/transaction.rs` → `rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:646:5 | assertion `left == right` failed: NewFinding { signature: CheckerViolations { invariants: ["I-3.1"] }, observation: FailureObservation { position: Operation(21)`
- 第 499 行 增补 2 收口表第 39 行那一族：发布不释放换下的落点（取号之前的预演与真发走同一段；回退到环里最旧的根时写行当场回收的几片哪一边都看不见）：改 `crates/singlefs-core/src/transaction.rs` → `rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:646:5 | assertion `left == right` failed: NewFinding { signature: CheckerViolations { invariants: ["I-3.11"] }, observation: FailureObservation { position: StartingPoin`
- 第 558 行 实例表第二片写路径：树表 0 条的一版上写行时旧链只释放第 0 片：改 `crates/singlefs-core/src/transaction.rs` → `a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_instance_table_second_page_write.rs:442:13 | assertion `left == right` failed: 盘 DeviceIdentity(0) 上被换下的那一片改写成已释放`
- 第 564 行 实例表第二片：树表 0 条的一版上写行只给第 0 片取落点（第 1 片没有落点，装链时查不到）：改 `crates/singlefs-core/src/transaction.rs` → `a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both` 红，红在 `crates/singlefs-core/src/transaction.rs:1341:26 | no entry found for key`
- 第 584 行 K4：打开文件时按需读 extent 树的节点数多记一倍（实现自报的与块层数到的对不上）：改 `crates/singlefs-core/src/mounted_read.rs` → `random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring` 红，红在 `crates/singlefs-harness/tests/second_transaction_parallel_line_two_mounted_read.rs:763:5 | assertion `left == right` failed: 打开文件按需读 extent 树：上段根兼叶一个、下段叶一个`；同时红：the_same_image_read_cold_and_read_through_the_mount_state_gives_the_same_bytes
- 第 596 行 增补 2 第 20a 行：取号之前的预演报了写行那次的错也照样取号（取号之后发布才被拒，实例代号已经烧掉）：改 `crates/singlefs-core/src/mount.rs` → `a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_row_publish_checks_before_acquisition.rs:132:5 | 预演里写行那次的释放核验报错 ⇒ 取号之前拒：Err(Publish(PublishSequenceFailed { cause: MappingEntryLocationOnADeviceOutsideThePool { unit: AllocationTree, device: DeviceIdentity(7),`
- 第 597 行 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出（树分裂换锚点）：改 `crates/singlefs-core/src/transaction.rs` → `allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:348:5 | 新发现（checker 的判红、模型、执行器的判定与 panic）：`
- 第 599 行 K2：extent 上段叶条目内联那一种的标签写成 3（格式登记的是 2）：改 `crates/singlefs-core/src/extent_tree.rs` → `inode_and_extent_lookups_from_the_root_read_the_first_file_back` 红，红在 `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:866:5 | assertion `left == right` failed: 标签 2：这个文件唯一那个数据单元的数据指针内联在上段叶条目里`
- 第 600 行 K1 × checker：分配记录树根按位置规定罩的那一段上端写成 0xFE…（checker 自己那份几何算错，第一个事务的节点头对不上它）：改 `crates/singlefs-checker/src/position_addressed.rs` → `every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold` 红，红在 `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:666:17 | assertion `left == right` failed: t5：层级与 key 区间是位置规定的`
- 第 608 行 K2：extent 下段叶里的记录不判 offset 段是不是净荷容量的整数倍（单元序号写进 offset 段的镜像报的不是位置那一条）：改 `crates/singlefs-core/src/extent_tree.rs` → `a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset` 红，红在 `crates/singlefs-harness/tests/second_transaction_parallel_line_two_mounted_read.rs:1151:5 | assertion `left == right` failed`
- 第 609 行 K1：从盘上读分配记录树时不判记录落在它所在叶按位置罩的那一段里（被抛弃根那棵账里挪到单元区末尾的记录报成别的成员）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `an_abandoned_roots_allocation_record_whose_span_runs_past_the_unit_area_is_counted_and_does_not_panic` 红，红在 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:1379:5 | C 那棵账该报「不在它所在叶按位置罩的那一段里」，实际交回的是 Err(InvariantViolated { invariant: "I-1.1", detail: "分配记录树叶里的记录不按 key 严格递增" })（分配器：这条根那棵账里第一条分配记录的槽号 50176 → 262143（单元区末尾是 26214`；同时红：with_the_shadow_ledger_off_and_every_root_of_the_rollback_instance_unreadable_the_recovery_falls_back_onto_the_abandoned_root_and_reads_a_torn_unit, without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
- 第 612 行 K2 × checker：extent 上段节点按位置规定罩的那一段上端多罩一个 inode（checker 自己那份几何算错，第一个事务的上段根兼叶对不上它）：改 `crates/singlefs-checker/src/position_addressed.rs` → `every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold` 红，红在 `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:666:17 | assertion `left == right` failed: t2：层级与 key 区间是位置规定的`
- 第 613 行 K2 × checker：extent 下段节点按位置规定罩的那一段上端少罩最后一个单元的字节（checker 的 I-1.1 在多单元文件上判红）：改 `crates/singlefs-checker/src/position_addressed.rs` → `the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline` 红，红在 `crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs:121:5 | 145 个单元之后：池级 checker 判红 ["I-1.1: extent 树（树 11）inode 1 下段的根：头里的 key 区间不是它的位置规定罩的那一段"]`；同时红：a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping, shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has, units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
- 第 614 行 C544 / Z4-1：重开时从树表 0 条那一版的分配记录树重建账，却不记下那棵树（下一次写行不知道上一版的节点）：改 `crates/singlefs-core/src/mount.rs` → `row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records` 红，红在 `crates/singlefs-core/src/transaction.rs:1129:14 | 根记录那一项不是全零时，分配器记着那一版的分配记录树（同一进程的写行或挂载时读回来的）`
- 第 652 行 实二二三 h（D18 已定项 11 第五个合取）：取号之前的预演不查写行那次经映射换下的映射条目的位置项（坏映射条目在取号之后才报，号烧掉）：改 `crates/singlefs-core/src/transaction.rs` → `a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_row_publish_checks_before_acquisition.rs:132:5 | 预演里写行那次的释放核验报错 ⇒ 取号之前拒：Err(Publish(PublishSequenceFailed { cause: MappingEntryLocationOnADeviceOutsideThePool { unit: AllocationTree, device: DeviceIdentity(7),`
- 第 653 行 C544 / Z4-1 × K1：分配记录按叶分装时按槽号除以叶宽取余（记录装进不罩它的叶，下一次重开读不回那棵树）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records` 红，红在 `crates/singlefs-harness/tests/second_transaction_supplement_two_row_publish_checks_before_acquisition.rs:204:13 | 第 1 次挂载：树表 0 条的一版上写 66 片实例表要做成：Recovery(AllocationRecordOutsideThePoolGeometry { what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽" })`；同时红：a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition
- 第 654 行 K1 / K2 × D28 已定项 4：按 key 空间定形状的两棵树的高从根节点头读成层级（不加一）：改 `crates/singlefs-core/src/transaction.rs` → `the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline` 红，红在 `crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs:313:9 | assertion `left == right` failed: 145 个单元：从根节点头现读的树高`
- 第 655 行 K1 单测：分配记录树根的层级取「切出来的格数小于 4」的最低一层（4 GiB 两盘多长一层）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `two_four_gibibyte_devices_give_a_tree_of_height_three` 红，红在 `crates/singlefs-core/src/allocation_record_tree.rs:897:9 | assertion `left == right` failed`；同时红：a_changed_record_changes_its_leaf_and_every_ancestor_only, an_entry_key_off_the_grid_names_no_child, two_one_tebibyte_devices_give_a_tree_of_height_four
- 第 656 行 K1 单测：分配记录树根的层级按八倍扇出判装不装得下（1 TiB 两盘少长一层：第 2 层切出 980 格，装得下 1352 就停在第 2 层）（改过锚点 / 点名测试 / 替换文之后重证）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `two_one_tebibyte_devices_give_a_tree_of_height_four` 红，红在 `crates/singlefs-core/src/allocation_record_tree.rs:910:9 | assertion `left == right` failed`；同时红：two_four_gibibyte_devices_give_a_tree_of_height_three
- 第 657 行 K1 单测：分配记录树按槽号找叶时叶宽算成两倍（叶的边界不在 812 的整数倍上）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `leaves_split_the_slots_at_multiples_of_the_leaf_width` 红，红在 `crates/singlefs-core/src/allocation_record_tree.rs:918:9 | assertion `left != right` failed`；同时红：a_changed_record_changes_its_leaf_and_every_ancestor_only
- 第 658 行 K1 单测：记录变了只重写那片叶、不连它的祖先（父节点里指它的那条子指针成了旧的）：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `a_changed_record_changes_its_leaf_and_every_ancestor_only` 红，红在 `crates/singlefs-core/src/allocation_record_tree.rs:953:9 | assertion `left == right` failed`
- 第 659 行 K1 单测：内部条目的 key 不在孩子那一层的格点上也认成孩子：改 `crates/singlefs-core/src/allocation_record_tree.rs` → `an_entry_key_off_the_grid_names_no_child` 红，红在 `crates/singlefs-core/src/allocation_record_tree.rs:968:9 | assertion failed: geometry.child_named_by_entry_key(root,`
- 第 660 行 K2 单测：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元白长一层）：改 `crates/singlefs-core/src/extent_tree.rs` → `a_lower_segment_grows_a_level_past_one_hundred_and_forty_four_data_units` 红，红在 `crates/singlefs-core/src/extent_tree.rs:1138:9 | assertion `left == right` failed`
- 第 661 行 K2 单测：extent 树上段根的层级取「罩的 inode 数不小于最大 inode 号」的最低一层（inode 143 还算在第 0 片叶里）：改 `crates/singlefs-core/src/extent_tree.rs` → `the_upper_segment_of_inode_one_is_a_single_leaf` 红，红在 `crates/singlefs-core/src/extent_tree.rs:1164:9 | assertion `left == right` failed`
- 第 662 行 K2 单测：extent 上段叶条目不认识的标签当成「没有单元」：改 `crates/singlefs-core/src/extent_tree.rs` → `upper_leaf_entries_round_trip_and_refuse_an_unknown_tag` 红，红在 `crates/singlefs-core/src/extent_tree.rs:1192:9 | assertion `left == right` failed`
- 第 663 行 K1 × checker 单测：checker 那份分配记录树根的层级按四倍扇出判装不装得下：改 `crates/singlefs-checker/src/position_addressed.rs` → `the_root_level_follows_the_device_sizes` 红，红在 `crates/singlefs-checker/src/position_addressed.rs:305:9 | assertion `left == right` failed`
- 第 664 行 K1 × checker 单测：checker 判记录落不落在叶里只看起点、不看末槽：改 `crates/singlefs-checker/src/position_addressed.rs` → `a_record_crossing_the_last_slot_of_its_leaf_does_not_fit` 红，红在 `crates/singlefs-checker/src/position_addressed.rs:325:9 | assertion failed: !allocation_record_fits_in_the_leaf(leaf, 0, last_slot_of_the_leaf, 2)`
- 第 665 行 K1 单测：分配记录树根之下的节点的步号把盘与层级写反：改 `crates/singlefs-core/src/transaction.rs` → `every_transaction_unit_names_its_class_tree_and_placement_rule` 红，红在 `crates/singlefs-core/src/transaction.rs:6591:9 | assertion `left == right` failed: 分配记录树根之下的节点按位置写步号`
- 第 666 行 K2 × 格式常量：extent 上段一片叶罩的 inode 数写成 142（与 (16384 − 163) ÷ 113 对不上）：改 `crates/singlefs-format/src/lib.rs` → `widths_match_the_first_transaction_byte_table` 红，红在 `crates/singlefs-format/src/lib.rs:357:9 | assertion `left == right` failed: extent 上段叶罩几个 inode 号`

说明：
- 草稿副本 `/tmp/claude-1000/impl-m2-keyspace/mut/w1`、`w2`（`rsync -a --exclude target --exclude .git`，各自的编译目录，各绑 4 核）；每条改一处、跑点名测试所在的**整个**
  测试二进制（随机历史那一个太重，照表里那一行的过滤跑），看点名那条红，再从原件拷回并 `touch`。日志在 `mut/logs/p2-*`、`p3-*`，结果在 `mut/results2.tsv`、`results3.tsv`。
- 有四行红在实现自己的断言上、不在测试断言上（都是 `assert!` / `expect` / 下标取值，不是 `debug_assert`，release 下同样红）：第 10 行
  （`allocation_record_tree.rs` 的 `records_of_each_leaf` 里「记录越过它所在叶的末槽」）、第 102 / 103 行（`mount.rs` 里「预演取的落点与真发逐次相同」，
  这两行本来就写的是「挂载自己的断言判出」）、第 564 行（`transaction.rs` 装链时查不到第 1 片的落点）、第 614 行（写行时分配器没记着上一版那棵树）。
- 第一遍没红的四行（376 编不过、393 锚在带文件那一路而点名测试走树表 0 条那一路、498 点名测试 16 次覆盖写转不过根环、656 四倍扇出不改 1 TiB 的结果）
  改了之后重证都红，上表标了「重证」。
- 留给门禁 59 的两行（点名测试在名字含 layer0 的二进制里，不跑）：第 70 行「步 3：空池挂载的零单元暖机把回退下界写成 1」、
  第 591 行「树分裂 树高从根节点头读成层级」（这一行只修了点名参数里 `--` 后面丢的空格）。

## 不变量条文草稿与格式常量（交书记员写 kb；我不写 kb）

### 格式常量（`crates/singlefs-format/src/lib.rs` 新增，条款把取值交给实现员的那几项在这里写明，等主 agent 定后补进 D8 已定项 14）

| 常量 | 值 | 来历 |
|---|---|---|
| `ALLOCATION_RECORD_TREE_LEAF_SLOTS`（W） | 812 | 取叶条目容量 (16384 − 135) ÷ 20 本身；偶数；**条款只给 W ≤ 812 取偶数，具体取值是我定的，待主 agent 定** |
| `ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES` | 96 | key 10 + 子指针 86（D8 已定项 11 那一行） |
| `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`（F） | 169 | (16384 − 135) ÷ 96 |
| `EXTENT_TREE_INTERNAL_ENTRY_BYTES` | 110 | key 24 + 子指针 86 |
| `EXTENT_TREE_INTERNAL_FANOUT` | 147 | (16384 − 163) ÷ 110 |
| `EXTENT_TREE_LOWER_LEAF_DATA_UNITS` | 144 | (16384 − 163) ÷ 112 |
| `EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES` | 113 | key 24 (0, inode, 0) + 标签 1 + 载荷 88；**上段叶条目字段表是我定的，待主 agent 定** |
| `EXTENT_TREE_UPPER_LEAF_INODES` | 143 | (16384 − 163) ÷ 113 |

上段叶条目载荷：标签 0 全零；标签 1 = 下段根的节点指针 86 + 2 字节零；标签 2 = 那一个数据单元的数据指针 88。
标签 0 只有读者认，写者不写（0 个单元的文件在上段不留条目）。

### 形状（写进 D8 已定项 14 的「实现取值」一段，或 layout）

- 分配记录树：叶 k 罩一块盘上的槽 `[k·W, (k+1)·W)`；层级 L（L ≥ 0）上的节点罩 `W·F^L` 个槽；根罩整个 key 空间 `[00…0, FF…F]`，
  根按盘分流（根的条目直接指每块盘第 R−1 层的节点）。根层 R = 最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ W·F^(R−1)⌉ ≤ F。
  4 GiB × 2：R = 2（树高 3）；1 TiB × 2：R = 3（树高 4）；单元区 240 / 256 / 384 槽的小盘：R = 1（树高 2）。
- 缺席即全空闲：没有一条记录的那一段不写节点（根之下的节点至少一条条目）。
- extent 树上段按 inode 号的位置（叶罩 143 个号、内部扇出 147），下段一个文件一棵按数据单元号的位置（叶罩 144 个单元、扇出 147）；
  下段 key 仍是 (0, inode, 字节偏移)，字节偏移 = 单元号 × 32634。一个单元的文件内联在上段叶（标签 2），两个及以上建下段（标签 1）。
- 第一个事务的写清单因此变了：4 GiB × 2 上分配记录树五个节点（两盘各叶 61、两盘各第 1 层节点 0、根），先叶后根，
  单元区 A 的提交内生块 50240..=50252（原先 50240..=50248），点名项 12 条、单元写 24 次、映射条目 10 条。**layout/01-first-txn.md 的写清单要跟着重写**。

### 不变量草稿

1. **I-1.1 补一句（按位置寻址的两棵树）**：分配记录树与 extent 树（上段、每个文件的下段）的索引节点，头里的层级 = 它的位置规定的层级、
   头里的 key 区间 = 它的位置规定罩的那一段（根罩整个 key 空间）；内部条目的 key 是某个孩子那一段的起点、孩子按位置严格递增；
   分配记录叶里每条记录的 (盘, 起点槽) 落在这片叶罩的那一段里，且末槽 `slot + span − 1` 不越过叶的末槽；
   extent 上段叶条目的 inode 号落在这片叶罩的那一段里；下段叶记录的 key = (0, 这个文件, 单元号 × 32634)，单元号落在这片叶罩的那一段里。
   checker 已判（`walk.rs` 的 `position_and_entry_width_hold`、分配记录树与 extent 树两段的走读）。
2. **extent 上段叶条目标签**：标签 ∈ {0, 1, 2}；标签 0 载荷全零，标签 1 载荷末 2 字节为零；key 的 locality 与 offset 段为零。
   checker 判在 I-1.1 那一格里（解不开就判红）；要不要单列一条归主 agent。
3. **分配记录树根层由池几何定**：根的层级 = 上面那条公式算出的 R；树高从根节点头现读（D28 已定项 4 的 ckpt_cost）。checker 自己按每块盘的槽数算 R 比。
4. **根之下没有空节点**（缺席的一段不写节点）：实现的读者拒（`RecoveryFailure` 的 I-1.1 那一支），**checker 没判这一条**——要不要加进 checker 归主 agent。

## 停下交主 agent 的设计问题与要知道的事

1. **W、上段叶条目字段表、根按盘分流是我定的**（条款交给实现员）：W = 812（叶条目容量本身）；根罩整个 key 空间、根的条目直接指每块盘第 R−1 层的节点；
   上段叶条目 113 = key 24 + 标签 1 + 载荷 88（标签 1 时节点指针 86 + 2 字节零）。要改哪一样都是格式改动，趁还没有盘上的池定下来。
2. **固定点收拢的上限**：发布路径「猜重写集 → 在分配器拷贝上走释放（释放弄脏的叶先并进重写集）→ 取落点 → 算脏节点 → 不在集合里就并进去重来」，
   第 64 轮还没收拢就返回 `PublishError::AllocationRecordTreeRewriteSetDidNotSettle`（在任何落盘之前）。64 这个数没有条款；随机历史墙取样点、快档、
   复用档、回退档都没走到（这些段 0 新发现）。要不要换成证明出来的上界，还是留着这个成员，归主 agent。
   这一轮最后加的那一步（释放弄脏的叶先并进重写集再取落点）只改「落点被拒时报哪个角色」：报的是罩住它们的那一版 bump 次序里第一个取不到的
   （分配记录树的叶排在最前）；收拢出来的那一版不变（草稿副本上步 5 整段逐次发布的落点、释放逐项对过）。
3. **模型里分配记录树节点数的上界偏松**：`model.rs::allocation_record_tree_nodes_rewritten_upper_bound` 按「占用的槽最多伸到单元区起点 + 64 × 已占槽数」
   数可能有记录的节点，取固定点。它是上界、判得对，但比实现真写的多：4 GiB 盘上单元区墙的放行区间从原先第 ~330 次发布之后才开，变成 ~24 次之后就开，
   墙附近的判别力变弱。要紧一点得按实现的落点规则算，那就不独立了。取哪一头归主 agent。
4. **读者的叶位置检查盖住了几何检查**：`recovery` 读分配记录树时先按位置判「记录该在这片叶里、末槽不越过叶末」，原先
   `allocation_records_fit_the_pool_geometry` 那几条（越过单元区末、越过盘末）大多先被位置检查拦下，只剩「最后一片叶越过盘末」那一格还走得到。
   坏盘输入那几条用例的期望成员跟着换了。要不要让几何检查排在前面归主 agent。
5. **checker 没判「根之下没有空节点」**：实现的读者拒收一条条目都没有的根下节点（缺席的一段不写节点），checker 不判这一条。要不要加一条归主 agent。
6. **暖机准入那一格今天没有触发**：`MountError::WarmUpAdmissionRefusedBeforeAcquisition` 原先由「分配记录一个节点装不下」触发；墙拆掉之后预演只剩
   落点拒绝（归 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）、两棵多层码 2 树长过 256 层、固定点收不拢三种。成员留着没删。
7. **公开 API 改了**：`MountedPoolForRead::open_file` 多一个 `reader: &dyn PoolReader` 参数（K4 按需读）；`TransactionUnit` 多三个成员。调用点都改了。
8. **影子账开着时分配记录树多写节点**：回退之后影子账把被抛弃的槽隔离，D 与暖机的落点被推到那几块之后（越过 50344 进叶 62），分配记录树比影子账关着时
   两块盘各多写一片叶 62。准入读数两边之差因此是「被隔离的槽 + 这几个节点」，不再只是被隔离的槽（`second_transaction_supplement_two_admission_formula.rs`
   按这个改了断言）。按位置寻址本来就是落点决定树形，这一点要不要写进 D28 已定项 1 的第九项说明，归主 agent。
9. **层 0 已有流钉着旧布局的数**（槽号、写数、段序列）都会变：我没跑层 0，没改已有层 0 流里钉的数。提交时的层 0 全量会在已有流上红在这些字面量上，
   要 crash-verifier 按新布局重钉（或主 agent 另派）。
10. **E142 区域表、门禁 55 / 69 的产物**：`src/first_transaction_regions.rs` 21 行区域表与 `tests/first_transaction_region_bytes.rs` 的期望表照规矩从
    `layout/01-first-txn.md` 抄（双份记账），kb 没改之前我不改——`the_region_table_matches_the_writes_the_first_transaction_really_issues` 红
    （第一个事务多出四个节点两盘各一份共 8 条写）。E142 产物也要重跑。
11. **E156 的登记常量**：`e156_allocation_basis_counts` 删掉了撞墙那一支；它从登记抄的 `E156_OVERWRITE_EXPECTED_RELEASED_SLOTS = 10`、
    `E156_OVERWRITE_EXPECTED_RECORD_DELTA = 16`、`E156_FIRST_TRANSACTION_EXPECTED_RECORD_COUNT = 20` 在新布局上对不上（一次覆盖写现在释放
    14 槽，改的记录跨两片叶时 16 槽；第一个事务之后每盘 14 条记录），跑它会停在 S1 的断言上。这是实验登记的事，我没改；要不要重跑归主 agent。
12. **本轮之前就红的**（不是这一轮造成、照原样留着）：formatted_pool 瞬时读错那条、checker_known_bad_images 的 I-7.10 / I-7.11 清单、
    step_four_rollback 两条回落到 C（都是回退见证那一格，见第一节）。
13. **`build_pool` 的分配器没有根环表**：`second_transaction_step_one_overwrite.rs` 那条连续覆盖写越过 812 条的用例转过根环之后 I-3.1 红（进程内不回收），
    用例只判 I-1.1 与 I-3.10；装置的形状，不是这一轮的。
14. **145 个单元的下段两层**进层 0 只做了流（`ExtentLowerSegmentGrowsToTwoLevels`），它的单元写段与记录段太大、全量枚举不出来（那条用例的注里写了），
    快档怎么取样归 crash-verifier。

## 验证（末尾原样输出）

`cargo fmt --all --check`（exit 1；diff 只在 `e158_root_choice_repair.rs` 五处，E158 执行员的文件，我没碰）：
```
Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:1927:
Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:2370:
Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:3628:
Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:5988:
Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:6039:
         assert!(
             error.contains("level=1"),
             "错误信息要点名是哪个 level 拦下的，实际: {error}"
```

`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 check.sh 那七条 `-D clippy::…`（exit 0）：
```
    Checking singlefs-core v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-core)
    Checking singlefs-harness v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.36s
```

`cargo build --offline --all-targets`（exit 0，0 条告警）：
```
   Compiling singlefs-harness v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.31s
```

动到的 lib 单测与测试二进制（整个二进制；最后一遍，`final/summary.txt` 原样）：
```
lib-singlefs-format exit=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
lib-singlefs-core exit=0 test result: ok. 118 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
lib-singlefs-checker exit=0 test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
lib-singlefs-harness exit=0 test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.19s
parallel_line_one_sequential_write exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.96s
first_transaction_step_five_publish exit=0 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.74s
first_transaction_step_six_recovery exit=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.23s
checker_known_bad_images exit=101 test result: FAILED. 31 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.38s
second_transaction_mapping_node_admission exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
second_transaction_parallel_line_one_multi_unit_file exit=0 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.64s
second_transaction_parallel_line_one_sequential_write exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.26s
second_transaction_parallel_line_two_mounted_read exit=0 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
second_transaction_parallel_line_three_many_inodes exit=0 test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.47s
second_transaction_step_one_overwrite exit=0 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.02s
second_transaction_step_three_formatted_pool exit=101 test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.39s
second_transaction_step_three_second_instance exit=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.16s
second_transaction_step_four_rollback exit=101 test result: FAILED. 10 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.37s
second_transaction_step_five_reuse exit=0 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.90s
second_transaction_supplement_one_write_accounting exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
second_transaction_supplement_two_accounting_node_full exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.03s
second_transaction_supplement_two_admission_formula exit=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.93s
second_transaction_supplement_two_c533_row_publish_record_without_its_root exit=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
second_transaction_supplement_two_commit_generated_fallback exit=0 test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.37s
second_transaction_supplement_two_instance_table_chain exit=0 test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s
second_transaction_supplement_two_instance_table_second_page_write exit=0 test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
second_transaction_supplement_two_multi_record_transaction_zero_publishes exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit exit=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.54s
second_transaction_supplement_two_reused_record_overlap exit=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.22s
second_transaction_supplement_two_row_publish_admission exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.28s
second_transaction_supplement_two_row_publish_checks_before_acquisition exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.97s
second_transaction_supplement_two_tree_nodes_and_the_central_mapping exit=0 test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.90s
second_transaction_supplement_two_tree_split exit=0 test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
second_transaction_supplement_two_unreadable_abandoned_root_slot exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.59s
second_transaction_supplement_three_bad_disk_input exit=0 test result: ok. 9 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 60.85s
second_transaction_supplement_three_random_history exit=0 test result: ok. 19 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 883.63s
DONE
```

上表里红的三个二进制，红的都只是本轮之前就红的那几条（回退见证那一格，见第一节）：`checker_known_bad_images` 的
`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`、`second_transaction_step_three_formatted_pool` 的
`transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write`、`second_transaction_step_four_rollback`
那两条回落到 C 的；别的一条都没红。`first_transaction_region_bytes`（等 kb）不在这张表里：它红一条
（`the_region_table_matches_the_writes_the_first_transaction_really_issues`）。

随机历史五段（最后一遍原样摘）：
```
越过原分配记录墙：历史 32 段：跑完 32、以已知红收尾 {}、新发现 0；根环转过一圈的 32 段；最高 txg 183；一版里最多 1324 条分配记录
快档：历史 96 段：跑完 92、以已知红收尾 {0: 4}、新发现 0；根环转过一圈的 76 段；最高 txg 46；一版里最多 744 条分配记录
复用档：历史 48 段：跑完 48、以已知红收尾 {}、新发现 0；根环转过一圈的 45 段；最高 txg 47；一版里最多 856 条分配记录
回退档：历史 48 段：跑完 48、以已知红收尾 {}、新发现 0；根环转过一圈的 36 段；最高 txg 49；一版里最多 460 条分配记录
小盘单元区墙：历史 32 段：跑完 32、以已知红收尾 {}、新发现 0；根环转过一圈的 9 段；最高 txg 67；一版里最多 460 条分配记录
```

`git diff --stat -- crates litmus`（原样；含别的会话此前留下、未提交的改动，我这一轮的文件见上一节清单）：
```
 crates/mutations.tsv                               |  289 +-
 crates/singlefs-checker/src/image.rs               |   79 +-
 crates/singlefs-checker/src/lib.rs                 |  100 +-
 crates/singlefs-checker/src/walk.rs                | 2170 ++++++++-
 crates/singlefs-core/src/allocator.rs              |  171 +-
 crates/singlefs-core/src/instance_table.rs         |  135 +-
 crates/singlefs-core/src/journal.rs                |  139 +-
 crates/singlefs-core/src/lib.rs                    |    4 +
 crates/singlefs-core/src/make_filesystem.rs        |    2 +
 crates/singlefs-core/src/mount.rs                  |  865 +++-
 crates/singlefs-core/src/mounted_read.rs           |  285 +-
 crates/singlefs-core/src/recovery.rs               | 1190 +++--
 crates/singlefs-core/src/system_configuration.rs   |   76 +-
 crates/singlefs-core/src/transaction.rs            | 4884 +++++++++++++++-----
 crates/singlefs-core/src/write_accounting.rs       |   20 +-
 crates/singlefs-format/src/lib.rs                  |  131 +
 crates/singlefs-harness/src/bad_disk_input.rs      |  335 +-
 .../src/bin/e156_allocation_basis_counts.rs        |   19 +-
 .../src/bin/e158_root_choice_repair.rs             | 2103 ++++++++-
 .../src/bin/first_transaction_device_log_check.rs  |  253 +-
 .../src/bin/first_transaction_on_device.rs         |  847 +++-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    4 +-
 crates/singlefs-harness/src/history.rs             |  173 +-
 crates/singlefs-harness/src/model.rs               |  461 +-
 crates/singlefs-harness/src/model_comparison.rs    |   84 +-
 crates/singlefs-harness/src/on_device_modes.rs     |  131 +-
 crates/singlefs-harness/src/scenario.rs            |    2 -
 .../tests/checker_known_bad_images.rs              | 1002 +++-
 .../tests/first_transaction_step_five_publish.rs   |  313 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 .../tests/first_transaction_step_six_recovery.rs   |    5 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 -
 .../tests/parallel_line_one_sequential_write.rs    |   55 +-
 .../second_transaction_mapping_node_admission.rs   |  230 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |  167 +-
 ...ansaction_parallel_line_one_sequential_write.rs |   52 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  256 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |  343 +-
 .../tests/second_transaction_step_five_reuse.rs    |   29 +-
 .../tests/second_transaction_step_four_rollback.rs |  148 +-
 .../tests/second_transaction_step_one_overwrite.rs |  218 +-
 ...second_transaction_step_three_formatted_pool.rs |  177 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |  103 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ..._transaction_supplement_one_write_accounting.rs |   97 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   35 +-
 ...transaction_supplement_three_fault_injection.rs |  123 +-
 ..._transaction_supplement_three_random_history.rs |  418 +-
 ...nsaction_supplement_two_accounting_node_full.rs |  134 +-
 ...transaction_supplement_two_admission_formula.rs |   69 +-
 ...two_c533_row_publish_record_without_its_root.rs |   87 +-
 ...ion_supplement_two_commit_generated_fallback.rs |  149 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   53 +-
 ...tion_supplement_two_instance_table_page_full.rs |  278 +-
 ...n_supplement_two_release_checksum_quarantine.rs |  783 +++-
 ...saction_supplement_two_reused_record_overlap.rs |   19 +-
 ...saction_supplement_two_row_publish_admission.rs |  358 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  461 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |   44 +-
 .../system_configuration_mutability_classes.rs     |    5 +
 62 files changed, 16327 insertions(+), 4892 deletions(-)
```

## 没做什么

- 没走三方对抗；层 0（新流只编过，已有流一条没跑）、QEMU、herd7、crates 变异表整表复跑（门禁 59）归 crash-verifier；没提交、没做任何 git 写操作。
- 没写 kb、没写 `research/`：不变量草稿、格式常量、写清单的变化在上面，交书记员；E142 / E156 的产物没重跑。
- C538（分配记录树释放链让保留池兜不住）没做。
- 没改：`src/first_transaction_regions.rs` 与 `tests/first_transaction_region_bytes.rs`（等 kb 布局）、E156 从登记抄的常量、已有层 0 流里钉的数、
  `e158_root_choice_repair.rs`。
- 本轮之前就红的三处（回退见证那一格）照原样留着，没修。
- 没跑：全量 `cargo test --all`、`check.sh`、`gate.sh` 全量、名字含 layer0 的二进制、`second_transaction_supplement_three_crash_injection`、
  `second_transaction_supplement_three_fault_injection`、`publish_order_matches_litmus`（herd7）。
