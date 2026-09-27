# 实审 B2 报告（checker 查窄的不变量与 B1 留下的几件；中途交接）

写于 2026-09-26 UTC（JST 2026-09-27）。实现员，主工作区。主 agent 按上下文线（72 万）叫停，这是交接报告：做完的、做到一半的、没开的分开写。

## 一、结论

- 代码停在编得过、两个测试二进制全绿的点：`checker_narrow_invariants_and_abandoned_roots`（新建）19 条全过；`checker_known_bad_images` 39 条全过（主工作区，改后）。
- 规格六件与后来追加的三件，**checker 代码都已改完**（第二节逐件）；每件都有先在改之前的 checker 上跑红的用例（第三节）。
- **没做完的**（第五节）：变异行一行没写（`mutations-append.tsv` 不存在）、证红一条没跑；`crates/mutations.tsv` 我弄腐化的 11 行锚点没改（含主 agent 点名的 865/874/877/878）；fmt / clippy / build 全量与登记给我的门禁阶段没跑；B1 测试文件与 formatted_pool 测试要跟着改的补丁没出。
- **会让别的测试红的**（第六节）：`checker_cross_links_malformed_nodes_and_mapping_entries.rs` 的 4 条（3 条映射 key 断言名改了、1 条计数器 0 从「成立」改「违例」）与 `second_transaction_step_three_formatted_pool.rs` 的不适用名单——两份都不在我的文件单里，我没改、也没跑。

推翻条件：改后的两个二进制在同一份源码上有一条不过；或第三节「改之前红」那几条在改之前的 checker 上其实是绿的。

## 二、这一轮写过的文件

- `crates/singlefs-checker/src/walk.rs`：全部判定改动（下面逐件）。
- `crates/singlefs-checker/src/image.rs`：`Judgements::part_of_the_range_not_judged`（射程有一部分判不了时整条报不适用、不报成立；有违例照报违例）、`first_violation_of`；清单加占位名 `I-MAPPING-KEY`（常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，清单 45 → 46 条，rustfmt 把数组排成一行一条）；`judge_location_entries_order`（I-2.5 不带全零豁免那一半）；`verified_system_configuration_slots` 槽 0 无效时借池里第一块槽 0 有效的盘记的槽距（A2a 第 37 条对齐）。
- `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`：新建，19 条。
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：I-1.2 那份坏镜像连映射里 extent 根那条 key 的出生序号一起 +1（新函数 `raise_the_mapping_key_tail_of_the_node_at`）；`known_bad_images()` 加一份 `I-MAPPING-KEY` 坏镜像（映射里数据单元那条 key 的出生 txg 3 → 4，用现成的 `rewrite_the_data_unit_mapping_key_birth_txg`）；新加 3 条用例与 `break_the_back_chain_of_journal_record`（第三节）。
- `crates/singlefs-checker/src/lib.rs`、`position_addressed.rs`：没改（`cargo fmt -p singlefs-checker` 跑过，`cmp` 与改前重建的那份相同）。
- `crates/mutations.tsv`：没改。
- 草稿：`/tmp/claude-1000/impl-rev-b2/` 下 `orig/`（改之前的 checker 源码：`walk.rs` 取自调查员 16:30:40Z 的快照、`image.rs.rebuilt` = HEAD + 调查员当时存的 diff；`cmp` 核过 `walk.rs.orig` 与 HEAD + diff 逐字节相同）、`copy/`（改之前的副本，下一节）、各次运行日志、`progress.md`。

### 各件改在哪（`crates/singlefs-checker/src/walk.rs`，函数名定位；行号会漂）

| 件 | 改法 |
|---|---|
| 第 7 条 I-7.4 被抛弃根那一半 | 新函数 `judge_blocks_referenced_by_abandoned_roots`，在 `check_pool_image` 走完候选根与「由记录施加出来的版本」之后调：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）用一份自己的 `Walk` 走（判定丢掉、引用不进 I-3.1 并集），走出 I-2.1 违例或走读失败就按这条根判 I-7.4 红，说明带第一处对不上的与走读失败。根环有读不出的槽（`reader.read` 交 None）时 `part_of_the_range_not_judged("I-7.4", …)`：没违例整条报不适用 |
| B1 缺口（共享子树） | `Walk` 加 `walk_parent_unit` 与 `placements_referenced_below_a_walked_unit`（走过的单元 → 它自己的指针引用的落点）；`note_references_of_a_pointer_followed_by_the_walk` 把落点记在当前父单元下；各走子节点的循环前后 `start_walking_below` / `stop_walking_below`（树表条目、多层码 2、分配记录树、extent 上 / 下段、inode 内部节点、实例表链逐片）；`visited_units` 命中处（`read_index_node`、inode 叶与内部节点、实例表链、数据单元）调 `merge_placements_below_a_unit_walked_by_an_earlier_version`，按表把子树落点并进这一版再判 I-5.1，不读盘；I-5.1 那一判抽成 `judge_a_placement_referenced_in_this_version` |
| 第 9 条 I-2.5 点名项 | `ScannedJournalRecord.named_entry_locations`；`judge_location_order_of_journal_named_entries` 逐盘逐条判；不带全零豁免（条款的豁免只写给 86 字节全零指针） |
| 第 9 条 I-9.2 类型段 0 | `walk_inode_root` 拆成 `walk_inode_internal_node`（递归）+ `walk_inode_leaf_container` + `walk_inode_internal_child`；类型段枚举 `InodeInternalEntryType`；类型段 0 判子单元码 2 且三段为 0，再按码 2 节点判头 / 出生身份 / I-1.3 / I-1.1（与 inode 根同一读法）/ 层级 = 父 − 1 / I-1.10 120 后往下走；层级 0 的码 2 子节点记走读失败不往下走（设计问题 1）；认不得的类型段只判 I-9.2 红、不往下走。`collect_tree_references` 的 inode 那一支改成 `note_every_inode_child_below` 跟着类型段 0 往下数 |
| 第 9 条 I-9.4 三句 | `InodeLeafContainerInLeafOrder` 叶序（含别的版本走过的叶，缓存在 `inode_leaf_order_below_a_walked_unit`）；`judge_container_numbers_along_the_leaf_order` 逐对判「容器号严格递增」「左最大 key < 右容器号」；I-9.12 抽成 `judge_separators_against_the_inode_children`，类型段 0 孩子的区间取它下面各叶记录的最小 / 最大 |
| 第 9 条 I-3.9 / I-5.4（C512） | `references_of_root` 把根记录持有那棵分配记录树的记录并进 `allocation_records`；`judge_allocation_records_disjoint` 每条候选根先判根记录持有的那棵（`ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT`）再判树表里种类 3 的 |
| 第 9 条 I-7.7 ① ② 分开 | `instance_carriers` 交 `InstanceCarriersOnDisk { carriers, some_slot_is_unreadable }`，读不出的槽跳过照读；读不全时 ① 走 `part_of_the_range_not_judged`，② 只在各盘号不等时有对象、在读得出的载体上找到就红；各盘相等时 ② 不再拿高于它的载体红（那是 ① 的事） |
| 新不变量 `I-MAPPING-KEY` | `walk_central_mapping_entries` 里 B1 记在 I-1.6 / I-1.2 下的三处（认不得的类标签、类标签不符、出生身份不符）改记这一条；加补零两字节（码 2 / 码 3 尾段 [25, 27) 恒 0）与出生树（码 1 / 码 3 比单元头偏移 43，码 2 随 C289 不比：`BirthTreeInTheHeader::FieldStillOpenUnderC289`） |
| 计数器 0 | `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`（`expected_back_chain_of`），`judge_journal_back_chain` 判 I-8.6 违例，说明「计数器 0：计数器从 1 起……反向链无从定义」。记在 I-8.6 下 |
| 前缀口径照 I-8.6（A2a 第 25 条对齐） | `records_as_the_recovery_reads_them`（每个计数器取第一块盘上那一份，与恢复扫环同序）+ `record_can_enter_a_replay_prefix`（照恢复：只在同实例前一条在盘上时比链，不等不进；计数器 0 不进；别的不判照进）；`versions_applied_only_by_records` 与 `tree_table_pointer_of_the_version_the_next_mount_applies_first` 都用它。I-8.6 那一判仍逐盘判、对计数器 1 与「前一条是别的实例」照条款判链恒 0（A2a 报告那一节第 2 条那一格没收，设计问题 3） |
| 追加：由记录施加出来那一版走它施加在其上那一版的实例表 | `VersionAppliedOnlyByRecords.root_record_of_the_version_it_was_applied_on`（环里同实例、txg 比它小的根里最大的一条，不管在不在候选集）；`walk_version_applied_only_by_records` 先按那条根记录走实例表链与根记录持有的分配记录树，再走新根段。环里一条都没有时照旧只走新根段（设计问题 2）。那段「这一格条款没写」注释改成现状 |
| 追加：A2a 第 37 条对齐 | `image.rs` 的 `verified_system_configuration_slots` 先读遍各盘槽 0，槽 0 无效的盘借第一块槽 0 有效的盘记的槽距找槽 1，全池都无效才用 4096 |

## 三、新测试：改之前红在哪（副本 `copy/`，checker 换回改之前的 `walk.rs` / `image.rs`，只在副本的 `image.rs` 末尾补了那个占位名常量让用例编得过）

新文件 `checker_narrow_invariants_and_abandoned_roots`：改前 `test result: FAILED. 3 passed; 16 failed`（`before-new-test-2.log`），改后 `test result: ok. 19 passed; 0 failed`（`after-new-test-2.log`）。改前的红（原样摘）：

| 用例 | 改之前 |
|---|---|
| `a_device_whose_slot_zero_does_not_verify_has_its_slot_one_looked_for_at_the_spacing_another_device_records` | `left: (0, 2)` `right: (1, 2)`（盘 0 一个自证过的槽都没找到） |
| `a_journal_named_entry_whose_location_entries_are_not_in_ascending_device_order_reddens_only_the_location_order_invariant` | `left: []` `right: ["I-2.5"]` |
| `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` | `I-8.6 该判违例，实际 Holds` |
| `a_mapping_key_whose_birth_tree_is_not_the_one_in_the_unit_header_reddens_only_the_mapping_key_invariant` | `类标签 1：只该红映射 key 那一条：[]` |
| `a_mapping_key_whose_padding_after_the_birth_sequence_is_not_zero_reddens_only_the_mapping_key_invariant` | `left: []` `right: ["I-MAPPING-KEY"]` |
| `a_legal_type_zero_inode_entry_is_walked_down_to_the_leaf_and_every_invariant_holds` | 红 `I-1.3、I-1.6、I-2.4、I-4.8、I-7.2、I-7.4、I-9.2`（把码 2 当码 3 读） |
| `a_type_zero_inode_entry_whose_container_segment_is_not_zero_reddens_only_the_entry_identity_invariant` | `left: ["I-1.3", "I-1.6", "I-2.4", "I-4.8", "I-7.2", "I-7.4", "I-9.2"]` `right: ["I-9.2"]` |
| `a_right_container_number_not_above_the_largest_key_on_its_left_reddens_only_the_container_number_invariant` | `left: []` `right: ["I-9.4"]` |
| `container_numbers_going_backwards_along_the_leaf_order_redden_only_the_container_number_invariant` | `left: []` `right: ["I-9.4"]` |
| `an_older_version_naming_again_a_placement_inside_a_subtree_it_shares_with_a_newer_version_reddens_the_disjoint_ranges_invariant`（B1 缺口） | `left: []` `right: ["I-5.1"]` |
| `a_released_record_in_the_allocation_record_tree_the_root_record_holds_is_judged_by_the_release_generation_invariant` | `left: NotApplicable("最新的根下面没有带已释放标志的分配记录")` `right: Holds` |
| `overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant` | `left: []` `right: ["I-5.4"]` |
| `the_second_sentence_of_the_instance_carrier_invariant_is_judged_on_the_readable_carriers_when_a_slot_is_unreadable` | `I-7.7 该判违例，实际 NotApplicable("根环、journal 环或单元区有读不出的槽：带号的东西数不全，判不了")` |
| `an_unreadable_root_ring_slot_leaves_the_abandoned_half_of_the_reuse_invariant_not_judged` | `被抛弃根那一半判不了，I-7.4 不许报成立：Holds` |
| `an_abandoned_root_whose_units_a_mount_blind_to_it_handed_out_again_reddens_only_the_reuse_invariant`（C554 那一形） | `C 的槽 [50184] 被重写：只该红 I-7.4：[]` |
| `erasing_a_unit_only_an_abandoned_root_references_reddens_only_the_reuse_invariant` | `left: []` `right: ["I-7.4"]` |
| `the_birth_tree_in_a_mapping_key_of_an_index_node_is_not_judged_while_its_field_is_open`、`two_leaf_containers_in_key_order_hold_every_invariant`、`the_first_sentence_of_the_instance_carrier_invariant_is_not_judged_when_a_slot_is_unreadable` | 改前改后都过：前两条是阳性对照（码 2 出生树不判、两片叶按序全绿），第三条钉条款字面（① 读不全报不适用），判别力要靠变异（第五节待证） |

C554 那一形：`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 造出的镜像上，那次看不见 C 的挂载只写行与暖机时**没**落在 C 的槽上（`erasing…` 那条钉着「抛弃之后那一刻全绿」）；接着在同一次挂载里覆盖写，第一版就把 C 的数据单元槽 50184 分了出去，I-7.4 照实红。`checker_known_bad_images.rs` 里原有的 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds` 用的是抛弃之后那一刻，改后照旧全绿、没改判。

`checker_known_bad_images`：改后 `test result: ok. 39 passed; 0 failed`（`after-known-bad-1.log`）。新加的三条：
- `version_applied_only_by_its_journal_record_walks_the_instance_table_of_the_root_it_was_applied_on_even_below_the_floor`（追加件：同一段历史发到 txg 10、抬 F 到 5，B (1, 4) 掉到 F 之下、(1, 5) 还在；实例 1 那张实例表释放代 6 > max(5, 1) 仍已分配）；
- `a_journal_record_whose_back_chain_does_not_match_is_not_walked_as_a_version_a_recovery_applied`（前缀口径：改后红 `["I-3.1", "I-8.6"]`，机理标识「并进遍历的由记录施加出来的版本 0 个」）；
- `a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first`（前缀口径：改后只红 I-8.6、I-3.10 成立）。
它们在改之前那份副本上的结果见第四节末行（交接时那一次还在跑）。

## 四、交回前的验证（只跑到这些）

- 新测试二进制（主工作区，经 `run-with-memory-cap.sh 8G` + `capped.sh 4`）：`test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.83s`
- `checker_known_bad_images`（同上）：`test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.28s`
- `cargo build --offline -p singlefs-checker`：exit 0，无警告（`build-checker-1.log`）。两个测试二进制 `--no-run` 编译 exit 0。
- 没跑：`cargo fmt --check` 全仓（只对我的文件跑过 rustfmt / `cargo fmt -p singlefs-checker`；`tests/common/mod.rs`、`tests/common_tree_split/mod.rs` 的 md5 前后相同）、`check.sh` 那套 clippy、`cargo build --all-targets`、登记给我的门禁阶段（33、53、74、89、92、93、94）——33 号在改到一半时跑过一次，见第五节。
- 改之前那份副本上的 `checker_known_bad_images`：交接时还在跑（`before-known-bad-1.log`），结果见本节末行追加。

## 五、做到一半 / 没开的（下一任接着做）

1. **变异行与证红（全没做）**。草稿目录没有 `mutations-append.tsv`。要写的行（每行一条新测试至少一条能让它红的变异）：I-7.4 被抛弃根那一半判恒真、根环读不出不报判不了；共享子树不并（`merge_placements_below_a_unit_walked_by_an_earlier_version` 里 `judge_a_placement_referenced_in_this_version` 去掉）；点名项 I-2.5 不判；类型段 0 条件去掉「三段为 0」、类型段 0 不往下走；I-9.4 第二 / 第三句恒真（两条分开）；根记录持有那棵的记录不并进（I-3.9）、不判（I-5.4）；I-7.7 ① 读不全照判、② 读不全不判；映射 key 补零 / 出生树恒真；`0 => ExpectedBackChain::CounterZeroHasNoLogicalPredecessor` 换成 `PredecessorNotOnThisDevice`；`record_can_enter_a_replay_prefix` 恒真（两个调用点各一条）；`root_record_of_the_version_it_was_applied_on` 恒 None；`verified_system_configuration_slots` 不借别的盘的槽距。证红照 `prove-red.sh --copy`，副本的 `crates/mutations.tsv` 末尾先追加这些行。
2. **`crates/mutations.tsv` 我弄腐化的锚点（主 agent 点名 865/874/877/878，33 号实测还有 308、309、310、319、494、759、870，都是 walk.rs，都是我这一轮挪代码带出来的；155、837–851 不是我的）**。新锚点（现查过恰好一处，`\n` 表换行）：
   - 865：`                self.judgements.judge("I-5.1", false, || {`（16 空格）→ 替换文把 `false` 换 `true`，测试不变。
   - 874：`                MappingKeyUnitClass::Unregistered(unregistered_tag) => {\n                    self.judgements.judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, false, || {` → `true`。
   - 877：`                MAPPING_KEY_MATCHES_THE_UNIT_HEADER,\n                header_class_tag == key_class_tag,` → 第二行换 `                true,`。
   - 878：`                .judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, matches_the_key, || {` → `matches_the_key` 换 `true`。
   - 308：`            )\n            && instance_table_rows.iter().any(|row| {` → `&&` 换 `||`；759：`            && instance_table_rows.iter().any(|row| {\n                row.instance == record.instance\n                    && record.checkpoint_txg <= row.published_checkpoint_txg\n            });` → `            && true;`；309 / 310 / 319：原文同一行、缩进由 12 空格改 8 空格。
   - 494：`                    size_in_bytes.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES);`（20 空格）。
   - 870：原文 `        0 => ExpectedBackChain::CounterZeroHasNoLogicalPredecessor,\n        FIRST_JOURNAL_COUNTER => ExpectedBackChain::Value(0),\n        _later_counter => match counter\n            .checked_sub(1)\n            .and_then(|previous_counter| records.get(&previous_counter))\n        {`，替换文 `        FIRST_JOURNAL_COUNTER => ExpectedBackChain::Value(0),\n        _later_counter => match records.get(&(counter - 1)) {`（计数器 0 下溢 panic），点名测试要改指新文件的 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant`（B1 那条如今在基线上就红，见第六节）。
   - 874 / 875 / 877 / 878 / 879 点名的是 B1 测试文件里断言旧编号的三条，基线上就红：要么照第六节改 B1 那份文件，要么在新文件里补三条同形用例（断言 `I-MAPPING-KEY`）再把这五行改指过去。我原打算后者，用例还没加。
   改完在副本里用 `prove-red.sh` 单跑这些行。
3. **门禁阶段**：33（改完锚点再跑）、53、74、89、92、93、94 都没跑。
4. **层 0 快档 / 崩溃注入**：没跑（重型 / 不归我）。

改之前那份副本上的 `checker_known_bad_images`（`before-known-bad-1.log`）：`test result: FAILED. 35 passed; 4 failed`，红的正是新加的三条与清单那一条：
- `version_applied_only_by_its_journal_record_walks_the_instance_table_of_the_root_it_was_applied_on_even_below_the_floor`：`left: ["I-3.1"]`，I-3.1 说明「盘 0：记账的已分配 Some(1703936)，遍历全部有效根得到 1671168」（差 32768，与调查报告那两条同数）；
- `a_journal_record_whose_back_chain_does_not_match_is_not_walked_as_a_version_a_recovery_applied`：`left: ["I-8.6"]` `right: ["I-3.1", "I-8.6"]`；
- `a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first`：`left: ["I-3.10", "I-8.6"]` `right: ["I-8.6"]`；
- `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`：清单与坏镜像的目标集合对不上（改之前清单里没有 `I-MAPPING-KEY`）。

## 六、改了判定、别的文件要跟着改的（都不在我的文件单里，我没改、没跑；推的）

1. `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`（B1）四条会红：`a_mapping_key_whose_birth_identity_is_not_the_one_in_the_unit_header_reddens_only_the_birth_identity_invariant` 断言 `["I-1.2"]`、`a_mapping_key_whose_class_is_not_the_class_of_the_unit_it_names_reddens_only_the_class_tag_invariant` 与 `a_mapping_key_with_an_unregistered_class_reddens_only_the_class_tag_invariant_and_is_not_read` 断言 `["I-1.6"]`——现在都记在 `I-MAPPING-KEY`；`a_journal_record_whose_counter_is_zero_is_judged_without_a_panic` 断言 I-8.6 `Holds`——现在判违例。改法：前三条的期望名换成 `singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER`（函数名里的 birth_identity / class_tag 跟着改，变异表 874–879 点名的测试名同步），第四条改断言违例（或删掉，新文件已有 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant`）。
2. `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`：`assert_checker_verdicts` 逐条核「名单外的全成立」。`I-MAPPING-KEY` 在没有文件的镜像上报不适用（映射根全零）⇒ `NOT_APPLICABLE_RIGHT_AFTER_MKFS`、`NOT_APPLICABLE_WITHOUT_FILE`、`NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE` 要加它；后者还要**去掉** `I-3.9` 与 `I-5.4`（根记录持有那棵现在被读，写行之后那份上两条判得了且成立，新文件那两条 C512 用例在同一段历史上钉了「成立」）。
3. 层 0（名字含 layer0，推的，没跑）：`first_transaction_step_seven_layer0.rs` 按不变量钉评估过的状态数，`I-MAPPING-KEY` 落进 `else` 那一格（全部状态）会对不上——它只在最新根有文件时有对象，应归 `only_under_the_new_root`；`second_transaction_step_zero_layer0.rs` 等只要求「评估过 + 不适用 = 状态数」，新名字自动满足。第二条流等有被抛弃根的流上，I-7.4 被抛弃根那一半若碰到 C554 那一形或「同一次挂载里环转过、共享槽在候选根离开之后被回收再分配」会新红（推的，没量）。
4. 崩溃注入 / 随机历史（门禁 74 号）：同上一条的 I-7.4 新红；I-3.1 那两条（种子 7463871032432355136 第 91 段）按追加件应转绿（没跑）。`history.rs` / `crash_injection.rs` 的已知红清单若按签名认 I-7.4，需要看。
5. 读不出槽的镜像（故障注入的 reader 若交 None）：I-7.4 在有行的镜像上从「成立」变「不适用」，I-7.7 在 ② 找得到时从「不适用」变「违例」。

## 七、停下交主 agent 的设计问题

1. inode 树里类型段 0 的条目指着层级 0 的码 2 节点：条款没有这种形状（叶是码 3，I-9.1 只给根留了「空的根」），我记走读失败、不往下走，不判哪条违例。类型段 0 子节点的 key 区间与 inode 根同一读法（贴紧首末条目），C478 定了之后一起改。
2. 由记录施加出来的那一版，环里一条同实例、txg 更小的根都读不出时，它的实例表与根记录持有的分配记录树走不到（照旧漏数）；「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的。
3. A2a 报告「checker 要跟着改什么」第 2 条（残留记录坐在同计数器上时 I-8.6 把下一条判成本实例第一条）：I-8.6 那一判照条款字面留着第 ③ 种情形没收，只在前缀口径上照恢复不判。要不要收交主 agent。
4. I-2.5 点名项不带全零豁免（条款的豁免只写给 86 字节全零指针），今天写者不写全零点名项。

## 八、交主 agent 写回 kb 的（我写不了 `.claude/kb/invariants.md`）

- **新不变量**（占位名 `I-MAPPING-KEY`，落点 `crates/singlefs-checker/src/image.rs` 常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`、判定在 `walk.rs` 的 `walk_central_mapping_entries`；坏镜像在新文件与 `checker_known_bad_images.rs`）：原句照 B1 报告第六节第 3 条，今天判到的射程：类标签在登记表里且等于被指单元头的；出生 txg / 实例代号 / 尾段等于单元头的；码 2 / 码 3 尾段后 2 字节补零恒 0；码 1 / 码 3 的出生树等于单元头偏移 43；码 2 的出生树随 C289 不判。映射根全零（无文件的一版）时报不适用。
- **计数器 0**：记在 I-8.6 下，原句建议「jsn 计数器从 1 起（D23 已定项 18），计数器 0 的记录没有本实例内逻辑前一条、反向链无从定义，判违例」。
- **I-7.4 状态列**补：被抛弃、还在根环里的根也判（各走一遍、只看读回来对不对、不进 I-3.1 并集）；根环有读不出的槽且最新根实例表有行时整条报不适用；C554 那一形照实红。
- **I-5.1 状态列**补：B1 留下的共享子树缺口已补（别的版本走过的单元，按缓存把它下面的落点并进这一版再比，不读盘）。
- **I-2.5 / I-9.2 / I-9.4 / I-3.9 / I-5.4 / I-7.7 状态列**：各把「查窄了」那一半补上（第二节表）。
- **I-8.6 状态列**补「不等的记录不进重放前缀」在 checker 的两处口径（恢复施加过的版本、下一次挂载先施加的那一版）照恢复对齐。

## 九、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与变异整表归 `crash-verifier`；没提交。
- 第五节列的：变异行、证红、腐化锚点修复、门禁阶段、B1 与 formatted_pool 两份测试的补丁——都没做。
- 没跑任何不在我文件单里的测试二进制（B1 那份、formatted_pool、崩溃注入、随机历史）。
- 负载：开跑前 `ps` 只见别的会话一条 `cargo test … second_transaction_suppl…`，照常 `nice -n 19` 跑；主工作区有一段时间 harness lib 编不过（`history.rs` 对不上别的会话改的 `mounted_session.rs`），我先改 checker，后来再编已通过。
- 草稿：`/tmp/claude-1000/impl-rev-b2/copy/`（改之前的副本连它的 target）交回前删掉；`orig/`（改之前的源码）、各日志、`progress.md` 留着给下一任。

## 附：`git diff --stat -- crates litmus` 原样（含别的会话与此前各轮没提交的改动；我的文件以第二节为准；新测试文件未跟踪、不在表里）

 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
 ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
 ...second_transaction_supplement_two_tree_split.rs |  147 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 87 files changed, 28969 insertions(+), 11040 deletions(-)
 crates/singlefs-checker/src/image.rs               |  216 ++-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1911 +++++++++++++++-----
 .../tests/checker_known_bad_images.rs              |  986 ++++++----
 5 files changed, 2287 insertions(+), 934 deletions(-)

整张 `git diff --stat -- crates litmus`（上面两段是它的末 8 行与只算 checker 四份加 known_bad_images 的那一段）：

```
 crates/mutations.tsv                               |  618 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1911 +++--
 crates/singlefs-core/src/admission.rs              |  272 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  132 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   24 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  179 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   22 +-
 crates/singlefs-core/src/recovery.rs               |  493 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  433 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6813 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2498 ++++++-
 crates/singlefs-harness/src/crash_injection.rs     |  821 ++-
 crates/singlefs-harness/src/device_log.rs          |   38 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1122 ++-
 crates/singlefs-harness/src/lib.rs                 |   60 +-
 crates/singlefs-harness/src/model.rs               |  964 ++-
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
 .../tests/first_transaction_step_seven_layer0.rs   |  233 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
 ...second_transaction_step_three_formatted_pool.rs |  567 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  658 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  342 +-
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
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 87 files changed, 28969 insertions(+), 11040 deletions(-)
```
