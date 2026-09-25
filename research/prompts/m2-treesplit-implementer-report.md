# 实十九（收口表第 28 行：记账树与中央映射树的分裂）实现员报告

时刻一律 UTC。草稿目录 `/tmp/claude-1000/impl-m2-treesplit/`。交付：`impl-m2-treesplit.patch`（只含 `crates/`，21 个文件）、`mutations-append.tsv`（48 行）、本报告。

## 一、结论

1. 记账树与中央映射树照 D8（核心索引结构） 已定项 11 的五条长成多层码 2 树，两棵树共用一份结构规则（已定项 14）：新模块 `crates/singlefs-core/src/code_two_tree.rs`
   （规划 `plan_the_tree_after_this_publish`，补丁后的主工作区第 633 行；读树 `read_code_two_tree` 第 866 行；拒绝 `CodeTwoTreeRefusal` 第 317 行）。
   ① 分裂出来的节点与别的单元同一段、整条路径照常 COW（只标重写，写序照旧，没加步骤、没加屏障）；② 叶 / 内部节点装不下时从中间切，左半 ⌈n ÷ 2⌉；
   ③ 只摘空节点，根只剩一个孩子时孩子当根（孩子不重写）；④ 插到最左分隔 key 之下时把它压低，切出来的右半在父节点里的分隔 key 取它自己的最小 key；
   ⑤ 树高从根节点码 2 头里现读层级 + 1（`TransactionOutput::height_read_from_the_root_node_header`，transaction.rs 第 1775 行）。
   内部条目 = 本树 key + 子指针 86：记账 108、映射 113（新常量 `ACCOUNTING_INTERNAL_ENTRY_BYTES`、`CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES`，format 单测钉住等于 key + 86）。
   内部节点头里的 key 区间写子树覆盖区间（D18（块里携带什么信息） 已定项 2）。
2. 写路径：`PublishPlan::resolve` 在任何落盘动作之前把两棵树这次之后的形状算完（记账 key 按行现算；中央映射 key 按这次要发的出生身份现算、照抄的取上一版），
   角色清单里两棵树的节点按「树内先叶后根、同层按 key 升序」排在分配记录树之后、树表之前（D3（空间分配） 已定项 10 ⑤），出生序号同一个次序发；
   装单元时核装出来的映射 key 与预先算的逐项相等（断言）。根之下的节点是新角色 `AccountingTreeNodeBelowTheRoot(位置)` / `MappingTreeNodeBelowTheRoot(位置)`，
   根兼叶那一档仍是 `AccountingTree` / `MappingTree`，第一个事务的字节一个不变（改之前的 E142 逐字比对、层 0 与步 1 各用例都绿）。
   换下的节点：记账树节点进映射、经映射释放并读盘核校验和；映射树节点豁免映射，按父条目里的指针释放（D19（块指针的结构与宽度预算） 已定项 8），不核。
   没碰到的节点照抄（字节、落点、指针不动，角色换成这一版的位置）。
3. 读路径：冷走读、挂载态、从盘上重建、读分配记录都整棵读两棵树（D19 已定项 5「中央映射长成多层时，挂载态打开池时把整棵映射树读进挂载态」）；
   挂载态多了观测点 `CentralMappingTreeReadsAtOpen`（节点数、树高）。「提示过期的一次解引用发 3 次设备读」在两层映射上重量仍是 3（D19 已定项 5 末句要的重量）。
   走读与挂载态按父条目逐节点核：树 ID、key 宽、出生身份、fsid、层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间 = 子树覆盖区间、同一节点不被两条父条目引用；
   重建那一路只核拼得成树的几样（层级、节点不空、内部条目宽、不重复引用），与它读别的树同一个口径；分隔 key 坏了由下一次规划交回拒绝（见第五节 B5）。
4. 准入：两棵树装不下一个节点不再报错（`AccountingEntriesExceedOneNode`、`MappingEntriesExceedOneNode` 删掉），换成 `MultiLevelCodeTwoTreeRefused { tree, refusal }`
   （长到 257 层、或上一版的树按分隔 key 走不到它自己叶里的 key），在规划那一步、任何落盘之前交回。分配记录树那一条准入按「这次重写的角色数」算，两棵树分裂多出来的节点各算一个。
   取号之前那一串（写行 + 暖机）按两棵树的形状逐次推每次重写哪些节点（`multi_level_tree_nodes_of_a_publish_sequence_without_content`，transaction.rs 第 3786 行，
   与发布路径同一个规划），准入与实十八新加的「取号之前在分配器拷贝上逐次取落点」（mount.rs 第 1276 行）都读它——推出来的与真发的逐项相同（用例 + mount 自己的断言）。
5. checker：两棵树整棵走，逐节点核「它管的区间就是父条目给的那一段」、分隔 key 那两条不等式、层级逐层减一、内部节点区间 = 子树覆盖区间（判在 I-1.1 名下），
   记账树内部节点条目宽判 I-1.10（108）；引用集合（I-3.1 等）数到两棵树的每个节点（walk.rs `walk_code_two_subtree` 第 615 行、`note_every_node_below` 第 1717 行）。
   新判定是否另立不变量编号交主 agent，条文草稿在第六节。
6. 只供测试的开关：写入口上的 `CodeTwoTreeNodeCapacities::CappedForTests`（transaction.rs 第 1348 行，`PoolWriter::set_code_two_tree_node_capacities` 第 160 行）把两棵树的节点容量压小；
   产品路径按格式算（记账叶 477 / 内部 150，映射叶 294 / 内部 143）。开关只住写入口内存，挂载时新建的写入口按产品路径起步。产品容量下记账树 80 块盘才分裂（用例钉住：483 行切成 239 + 244）。

**推翻条件**：层 0 七条流（第四节）在提交时的全量里有一个状态 oracle 或 checker 违例；或门禁 59 号跑 `mutations-append.tsv` 那 48 行里有一行不红；或某一版里从盘上重建出来的两棵树与发布写出的逐节点不同。

## 二、这一轮写过的文件

补丁对 16:15:53 UTC 的主工作区快照（已打进实二十 14:50Z、实十六接续 16:15Z 那一批；之后到交回主工作区只动了 `mutations.tsv` 与 `e158_root_choice_repair.rs`，都不在补丁里）生成，`git apply --check` 结果见第七节。21 个文件：

- 新建：`crates/singlefs-core/src/code_two_tree.rs`、`crates/singlefs-harness/tests/common_tree_split/mod.rs`（两份新用例共用的内存盘搭建与容量开关）、
  `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split.rs`、`crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split_layer0.rs`。
- 核心：`crates/singlefs-core/src/{lib.rs,transaction.rs,recovery.rs,mounted_read.rs,mount.rs,write_accounting.rs}`；格式：`crates/singlefs-format/src/lib.rs`（两个内部条目宽常量 + 单测）。
- checker：`crates/singlefs-checker/src/{lib.rs,walk.rs}`（`check_internal_node_separators`；多层走读与引用集合）。
- harness：`src/{history.rs,model.rs,model_comparison.rs}`（删掉的两个错误成员、新角色；模型只罩单节点树，遇到多层断言）。
- 改写的旧用例：`second_transaction_mapping_node_admission.rs`（装不下被拒 → 295 条长成两层 + 取号之前推算与真发逐次相等）、
  `second_transaction_supplement_two_accounting_node_full.rs`（80 块盘被拒 → 分裂成 239 + 244）、`second_transaction_parallel_line_two_mounted_read.rs`（层级 1 的根被拒 → 两层映射读数重量为 3；
  层级 1 的根装 55 字节条目 → 内部条目窄于 113 被拒）、`second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs`（两层映射被拒 → 整棵读进挂载态读回文件；根头区间不覆盖 → 两个读者同一个 I-1.1 拒绝）、
  `first_transaction_step_five_publish.rs`（穷举 match 补新角色）。
- `crates/mutations.tsv` 不在补丁里：追加的 48 行在 `mutations-append.tsv`，要删的 25 行按行名列在第三节。

主工作区 `git diff --stat -- crates litmus`（我没有改主工作区；这是别的会话此刻在主工作区里的未提交改动，原样贴，分不出谁改的）与补丁自己的 `git apply --stat` 见第七节。

与别的实现员同一个函数（补丁已建在两批之上，都合过；合并的次数与手合的地方见第七节）：
- `mount.rs`：实十八新加、实十六接续按实例表多片改过的 `placements_of_the_publishes_after_acquisition_on_a_copy`，两边都保留：写行形状按实例表片数
  （`row_publish_rewriting_instance_table_pages`）、第 0 次先放整条实例表旧链（`instance_table_chain_to_release`）再放经映射查到的、树表 0 条那一版取实例表各片（尾片先）
  加分配记录树节点——这是实十六接续的；每一次重写哪些记账树 / 中央映射树节点、换下哪些、换下的落点取哪一次的，按两棵树的形状逐次推——这是我的
  （多了 `instance_to_acquire`、`code_two_tree_node_capacities` 两个参数，共 8 个，加了 `too_many_arguments` 的 allow 与理由）。
  `refuse_publishes_before_acquisition_that_do_not_pass_admission` 多一个容量参数、`publish_sequence_admission` 多一个容量参数；`publish_empty_after` 调 `resolve` 多两个参数。
- `transaction.rs`：实十六接续新加的角色 `InstanceTablePageAfterTheFirst` 进了我的 `multi_level_tree_of_role`（交 `None`），我的两个树节点角色进了它的
  `is_a_page_of_the_instance_table`（交 `false`）；`placements_to_release_via_mapping` 与 `copies_failing_the_release_checksum_check` 只加了两个新角色的臂；
  `publish_version` 里释放清单是「整条实例表旧链 + 经映射查到的这次换下的上一版角色」（后者是 `ResolvedPublish::previous_roles_replaced_by_this_publish`，按 bump 次序、树表最末）。
- `recovery.rs`（实十四 / 实二十）：`rebuild_version` 读记账树、映射树那一段换成整棵读；重建用的映射读者 `CentralMappingRootWithBytesReadOnFirstUse` 换成
  `CentralMappingTreeWithBytesReadOnFirstUse`（整棵读），`allocation_records_under_root` 跟着用它；`walk_to_file` 的映射读者与两处映射条目数 / 查找改读整棵树的叶。
- `walk.rs`：`references_of_root` 里实十六接续新加的「实例表链第 1 片起」与我新加的「中央映射树根之下的节点」两段都在（前者在映射根之前，后者在树表那一圈之后）。
- `model.rs`：两边各删了一个拒绝原因（`InstanceTableChainLongerThanOnePageUndecided`、`AccountingNodeWall`），两处穷举臂都删。

## 三、每条新测试的证红与变异表

**做法**：副本 `mutation/work3`（合并实十六接续 16:15Z 那一批之前的那棵树 rebase2 rsync 出来，自带 target，debug 构建；合并时我的代码只动了 mount.rs 拷贝上那一串与两处穷举臂，#43 在合并后的树上另证一次，见表后），先把涉及的 7 个测试二进制各跑一遍不改动的基线，**基线红集全空**
（核心库单测 104、格式库单测 5、mapping_node_admission 3、parallel_line_two 12、accounting_node_full 2、tree_nodes_and_central_mapping 9、tree_split 5，全绿）。
每行改坏一处 → 跑那条测试所在的整个二进制 → 从原件拷回并 touch。结果 `mutation/logs6/results.tsv`（16:06:16 跑完），逐行日志 `row-0.log`…`row-19.log`、
`format-baseline.log`、`format-row-a.log`、`format-row-b.log`。下表的「#」是 `mutations-append.tsv` 的行号。22 行证过，22 条点名测试全红。

| 新测试（所在二进制） | # 改坏哪一行 | 哪条断言红 | 同时红的（基线之外） |
|---|---|---|---|
| `inserting_nine_keys_into_a_leaf_of_eight_splits_it_in_the_middle_and_grows_a_root`（核心库） | #1 code_two_tree.rs `keys.split_off(keys.len().div_ceil(2))` → `split_off(keys.len() - 1)` | code_two_tree.rs:1228 叶的 key 分布 `assert_eq`，left `[[1..8], [9]]` | `a_key_below_the_leftmost_separator…`、`deleting_every_key…` |
| `a_key_below_the_leftmost_separator_lowers_it_and_leaves_the_right_leaf_carried` | #2 删掉 `children[0].separator_key = key.clone();` | :1269「最左分隔 key 压低到新插的 key」 | 无 |
| `deleting_every_key_of_a_leaf_drops_it_and_a_root_left_with_one_child_hands_the_root_to_it` | #3 `if children.len() == 1` → `if false && …` | :1313「根只剩一个孩子：降高一层」left 2 | 无 |
| `a_leaf_split_that_overflows_a_full_root_splits_the_root_too_and_the_tree_grows_to_three_levels` | #4 `if children.len() > capacity.internal_entries` → `if false && …` | :1340「叶切开让根装不下，根也切开」left 2 | `the_level_byte_bounds_the_height`（:1364 left `Ok(())`） |
| `the_level_byte_bounds_the_height` | #5 `.checked_add(1)` → `.checked_add(0)` | **红在规划自己的 expect**：code_two_tree.rs:513「刚装满、切开过的节点左半不空」，不是用例的断言 | `a_key_below…`、`a_leaf_split_that_overflows…`、`inserting_nine_keys…` |
| `internal_entries_are_the_key_plus_an_eighty_six_byte_child_pointer` | #6 `… .expect("86")` → `… .expect("86") + 1` | :1204 left 109 | 无 |
| `pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for`（格式库；旧用例，新加两条断言） | #44 `ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 108` → 109；#45 `CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113` → 114 | format lib.rs:378「记账树内部条目 = key + 指向码 2 的指针」left 109；:383 left 114 | 无 |
| `under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts`（tree_split） | #7 子树覆盖区间 `(smallest…, largest…)` → `(smallest…, smallest…)` | tree_split.rs:129「Accounting (0,0)：头里的 key 区间是子树覆盖区间（D18 已定项 2）」 | 同二进制另三条（checker 那条红在「干净的那一份全绿」，I-1.1） |
| `an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes` | #8 `carried_from: Some(position)` → `None` | tree_split.rs:265「中央映射树有叶没被这次空发布碰到」 | 无 |
| `a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on` | #10 recovery.rs 记账树节点记映射 key 的循环加 `.rev().take(1)`；#43 mount.rs 拷贝上按形状推 → `shape.rewritten_roles()`（固定四角色） | #10：**红在发布路径的 expect** transaction.rs:2589（合并后第 2867 行）「照抄进来的角色在上一版里进映射、有一把 key」；#43：**红在挂载自己的 expect** mount.rs:1432（合并后第 1440 行）「那一次重写的节点在那一次取到了落点」 | #10：`under_small_node_capacities…`；#43：无 |
| `a_rebuilt_central_mapping_root_whose_separator_hides_a_key_is_refused_before_the_instance_generation_is_acquired` | #41 `return Err(PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt)` → `return Ok(())` | tree_split.rs:433「取号之前就拒」，实际 `PublishSequenceFailed { cause: ReleaseNotInMapping { unit: AllocationTree } }` | 无 |
| `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root` | #11 walk.rs `judge("I-1.1", view.level == level, …)` → `judge("I-1.1", true, …)` | tree_split.rs:762「Accounting LevelOneAboveItsChildren：I-1.1 要红，实际红的是 [I-4.8, I-7.2 …]」 | 无 |
| `the_central_mapping_node_holds_two_hundred_ninety_four_entries_and_an_internal_node_one_hundred_forty_three`（mapping_node_admission） | #16 同 #6 那一处 `+ 1` | mapping_node_admission.rs:44 left `(294, 142)` | 无 |
| `two_hundred_ninety_five_mapping_keys_grow_the_tree_to_two_levels_instead_of_being_refused` | #17 `div_ceil(2)` → `/ 2` | :106「295 条从中间切：左 ⌈295 ÷ 2⌉ = 148」left `[147, 148]` | 无 |
| `the_rewritten_role_counts_inferred_before_acquisition_equal_the_ones_each_empty_publish_really_rewrites` | #18 transaction.rs 删掉推算里「记账树节点旧映射 key 先删」三行 | :141「取号之前推出来的每次重写角色数与真发布逐次相等」left `[23, 24, 26]` | 无 |
| `eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing`（accounting_node_full） | #19 叶容量 `index_node_entry_capacity(…)` → `… + 1` | accounting_node_full.rs:131 left `[240, 243]` | 无 |
| `a_stale_hint_under_a_two_level_central_mapping_still_costs_three_device_reads_because_the_whole_tree_is_in_the_mount_state`（parallel_line_two） | #20 删掉 `leaf_entries_in_key_order.extend(…)` | parallel_line_two.rs:1132 expect「提示指错，经映射仍读得到」：`CentralMappingMiss` | 同二进制另 5 条（`a_central_mapping_root_whose_entry_width_is_narrower…`、`a_data_unit_resealed…`、`a_location_hint_pointing_at_another_slot…`、`corrupting_one_byte…`、`the_same_image_read_cold_and_read_through_the_mount_state…`） |
| `a_central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries` | #21 `if header.entry_width < internal_width` → `if false && …` | **红在读树自己的 expect**：code_two_tree.rs:998「条目宽上面判过不窄于 key 宽 + 86」，不是用例的断言 | 无 |
| `central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back`（tree_nodes_and_central_mapping） | #22 `Some(header.level - 1)` → `Some(header.level)` | tree_nodes.rs:684 expect「两层映射照常打开」：`Walk(InvariantViolated { I-1.1, "孩子的层级不是父层级减一" })` | `a_two_level_central_mapping_whose_root_header…` |
| `a_two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers` | #23 覆盖区间那一判 → `if false {` | :742「挂载态」left `None` | 无 |

被测代码里没有 `debug_assert`，四处「红在 expect」的在 release 下同样红在那里（expect 不随构建档变），没有另跑 `--release`。

**合并后重证 #43**（`mutation/work4` = 合并后的树 rebase3，自带 target，debug；结果 `mutation/logs7/results.tsv`，16:46:55）：基线 tree_split 5/5 绿、基线红集空；
改坏 mount.rs 拷贝上那一串的 `rewritten_roles_of_a_publish_without_content(*shape, &nodes.rewritten_roles)` → `{ let _ = nodes; shape.rewritten_roles() }`，
点名的 `a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on` 红在 mount.rs:1440 expect「那一次重写的节点在那一次取到了落点」，同二进制别的都不红。

**没跑、交门禁 59 号的**（`mutations-append.tsv` 24 行）：
- 点名的测试在 `second_transaction_supplement_two_tree_split_layer0` 二进制里、子 agent 不跑的 2 行：#24「树分裂 层 0：冷走读核映射条目数时记账树按一个节点算（多层记账树的镜像走读失败）」、
  #25「树分裂 树高从根节点头读成层级（不加一，D8 已定项 11 ⑤ / D28 已定项 4）」。
- 同一条测试已由别的行证过、多加的 5 行：#9、#12、#13、#14、#15。
- 旧行换锚点（原行的原文在补丁后的树上命中 0 次，删掉、以新名字重加）19 行：#26–#40、#42、#46–#48（#46–#48 是实十六接续 16:15Z 新加、锚在
  `placements_of_the_publishes_after_acquisition_on_a_copy` 里的三行，那一段合并时按两棵树重排过；#30、#42 按 16:15Z 之后的主工作区又换了一次锚）。

**`crates/mutations.tsv` 要删的 25 行**（按行名；16:17 的主工作区里每个名字恰好一行；它们的原文在补丁后的树上都命中 0 次。第 7、17、22 行实十六接续 16:15Z 按名字换过锚，换过的原文在补丁后的树上同样命中 0 次，照删）：
1. 释放退回按提示 —— 换锚点重加为 #26
2. 步 1 变异：不释放旧落点 —— #27
3. 步 1 变异：defer 行写 0 —— #28
4. 步 2 变异：已分配行不随分配更新（I-3.1 要红） —— #29
5. 增补 2 第 28 行：记账树装不下不判（80 块盘走到装节点的断言 panic） —— 不重加（装不下不再拒，改成分裂；由 #19 顶替）
6. 增补 2 第 28 行：记账树正好装满 477 行也判装不下（79 块盘发布被拒） —— 不重加（同上）
7. 增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉） —— #30
8. 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出 —— #31
9. 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b 同一处）：分配记录过 600 条之后记账「已分配」少记一槽；直接钉已分配统计的那条用例判出 —— #32
10. 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出 —— #33
11. 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出 —— #34
12. 并行线二：中央映射的根是内部节点也照收（把内部条目当 55 字节映射条目解，「整片映射都在挂载态里」这条前提没人守，读数 3 也就没了依据） —— 不重加（根是内部节点现在照收、整棵读；由 #20、#21 顶替）
13. 普查 R2：checker 走读中央映射条目之前不判条目宽（walk.rs 又切 entry[27..55]，checker 自己倒下） —— #35
14. X1（2026-09-23 代码轮攻方腿）：中央映射树的写侧容量准入整条去掉，装不下时走到 build_index_node 的断言 —— 不重加（那一条准入删了，装不下改成分裂；由 #16、#17 顶替）
15. X1（2026-09-23 代码轮攻方腿）：中央映射树容量准入的门槛写成大于等于，正好装满一个节点的那一次发布被误拒 —— 不重加（同上）
16. C511 第 3 步：checker 对中央映射树根的 I-1.3 退回写死树 ID 15，不按根记录里那条根指针的出生树判（回退之后重新发号的映射树判不了） —— #36
17. 增补 2 第 9 行 P1（C497）：带文件的一版上重写实例表的发布把实例表挪到树表单元之后取落点 —— #37
18. C483 ②：冷走读读 inode 叶容器只按位置提示读（不经映射回退） —— #38
19. C483 ①：挂载态的中央映射长成两层也照收（内部条目被当 55 字节映射条目解，D19 已定项 5「挂载态怎么读映射」的第一版限制没人守） —— 不重加（两层现在照收、整棵读；由 #22、#23 顶替）
20. 并行线一：映射条目数不算数据单元（多单元文件的映射节点条目数与准入算的对不上） —— #39（名字改成「…映射 key 与 resolve 预先算的对不上…」）
21. 代码三方 m2-wave3-code-r1 第四节第 4 条：只读挂载认中央映射树的根退回写死树 ID 15，不按根记录里映射根指针的出生树（回退之后再发的第一个文件版本映射树是 23，打不开） —— #40
22. 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点（拷贝上取的与真发的不同，挂载自己的断言判出） —— #42
23. 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出） —— #46
24. 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出） —— #47
25. 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行不释放被换下的那条实例表旧链（经映射那一路从这一轮起跳过实例表；回退到环里最旧的根时写行当场回收的那一片拷贝上看不见） —— #48

全文行名在 `/tmp/claude-1000/impl-m2-treesplit/rows-to-delete.txt`（一行一个，25 行）。按 16:47 的主工作区 `crates/mutations.tsv`（586 行，E158 那一路 16:36 又加了 2 行）删 25、加 48 之后整表 609 行，
在补丁后的树上（`e158_root_choice_repair.rs` 取 16:47 的主工作区那一份）逐行核「原文恰好命中一次」全过；变异名不重复；「文件 + 原文 + 替换文 + 点名的测试」四项不重复
（门禁 33 号 16:25 在同一棵树、当时 607 行的表上跑过，见第七节）。
（16:09 时主工作区自己的「E158 root_choice_repair 候选 (b) …交回之前清空集合」一行原文命中 0 次，16:17 主工作区已改好，与我的补丁无关。）

## 四、层 0：七条新流（交提交时由崩溃验证员跑，这一轮最终的树上没跑）

文件 `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split_layer0.rs`，枚举 `TreeSplitStream::ALL`。写法照三方 `m2-treesplit-r1` T4：
**起点镜像不枚举**（mkfs、取号、暖机、第一个文件版本、铺垫都施加成起点镜像），**只录分裂那一次发布**；段序列恒是四段
`[2 × 单元数, 2, 1, 2]`（单元写段、记录段、根槽 FUA、两盘系统配置槽轮换），每条流用例里断言它。每个状态两遍恢复 + 多版本 oracle（前一版与这一版）+ 池级 checker + 记录核对器。
两个用例：快档 `every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes`（单元写段不展开，每条流 8 个状态）；
全量 `full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean`（`#[ignore]`，每条流断言状态数 = 闭式）。另一条非层 0 的 `the_accounting_streams_read_the_tree_height_from_the_root_node_header` 在同一个二进制里。

| 流名 | 被录的发布 | 那一次前后这棵树（L = 叶、I = 内部节点，数字 = 条目数） | 写出的单元（每个两盘各一次写） | 全量状态数（闭式） |
|---|---|---|---|---|
| `CentralMappingRootSplit` | 空发布，映射叶容量 5 | 映射 `L6` → `L3 L3 I2`（根分裂，树高 1 → 2） | 分配记录树、记账树、映射两片叶 + 新根、树表 = 6 | 4103 |
| `CentralMappingLeafSplit` | 空发布，映射叶容量 5 → 2 | 映射 `L3 L3 I2` → `L3 L2 L1 I3`（右叶切开，树高不变，左叶照抄） | 分配记录树、记账树、映射两片叶 + 根、树表 = 6 | 4103 |
| `CentralMappingTwoLevelsSplitInARow` | 空发布，映射叶 5 → 2、内部 2 | 映射 `L3 L3 I2` → `L3 L2 L1 I2 I1 I2`（叶切开让根装不下、根也切开，树高 2 → 3） | 分配记录树、记账树、映射两片叶 + 两个层级 1 + 新根、树表 = 8 | 65543 |
| `CentralMappingEmptyLeafDropped` | 空发布，记账叶 14，映射叶 2 → 8 | 映射 `L2 L2 L2 L2 I4` → `L2 L2 L4 I3`（只装记账树节点 key 的那片叶删空摘掉；记账树 `L8 L7 I2` 整批换代） | 分配记录树、记账树三个节点、映射两片叶 + 根、树表 = 8 | 65543 |
| `CentralMappingRootLowered` | 覆盖写，映射容量回到产品值 | 映射 `L3 L3 I2` → `L6`（删的时候左叶删空、根只剩一个孩子降高，树高 2 → 1） | 数据单元、extent 根、inode 叶与根、分配记录树、记账树、映射一个节点、树表 = 8 | 65543 |
| `AccountingRootSplit` | 空发布，记账叶容量 14 | 记账 `L15` → `L8 L7 I2`（根分裂，树高 1 → 2；映射多两条） | 分配记录树、记账两片叶 + 新根、映射一个节点、树表 = 6 | 4103 |
| `AccountingLeafSplit` | 空发布，记账叶容量 14 → 8 | 记账 `L8 L7 I2` → `L5 L5 L5 I3`（叶多一片，树高不变） | 分配记录树、记账三片叶 + 根、映射一个节点、树表 = 7 | 16391 |

**各流罩住的写序**（七条同一个形状，只差单元数 N）：单元写段 2 × N 次写（N 见上表第四列：`CentralMappingRootSplit` 6、`CentralMappingLeafSplit` 6、`CentralMappingTwoLevelsSplitInARow` 8、`CentralMappingEmptyLeafDropped` 8、`CentralMappingRootLowered` 8、`AccountingRootSplit` 6、`AccountingLeafSplit` 7；每个单元两盘各一次，段内不排序，全量枚举段内任意子集）→ 记录段 2 次 → 根槽 FUA 1 次 → 两盘系统配置槽 2 次；全量状态数 = 1 + Σ(2^|段| − 1)。新层 0 流只保证编得过、没跑（主 agent 15:1x 的指示）；快档、全量两个用例名见上。

合计 225329 个状态。**比已有的流多罩了什么**：已有五条层 0 流里两棵树恒是根兼叶，一次发布写一个记账节点、一个映射节点；这七条里一次发布写出 2–5 个映射树节点或 3–4 个记账树节点，
它们与别的单元同一段（已定项 11 ①），单元写段里第一次有「新叶落了、新根没落」「新根落了、某片新叶没落」「照抄的叶之外全落了」这一类子集；
根降高那一条里记录的新根段指着一个节点的映射树、上一版是三个节点；摘空节点那一条里上一版被摘掉的叶与新叶同时在盘上。多跑的一步只有恢复本身（没有重开、挂载），
与已有流的基线镜像、写表、段序列都不同，各自全量。
**跑过的（只作参考，不是这一轮最终的树）**：13:3x UTC 在对齐主工作区之前的副本上 release 全量跑过一次，七条流 225329 个状态零违例（oracle、不看 journal 那一遍、走读失败、
记录核对器、checker 各条全 0；`LAYER0_TREE_SPLIT` 行在 `/tmp/claude-1000/impl-m2-treesplit/logs/layer0-full-1.log`）。之后读树改了一处（叶条目宽不在读树时判）、三次对齐主工作区，
按主 agent 15:1x 的指示最终的树上不跑层 0：这个二进制只保证编得过（clippy --all-targets 过）。

## 五、实做时定下的、交主 agent 的设计问题

- **B1 一次发布里先删后插**（条款没写，我取的）：上一版有、这一版没有的 key 先删（按 key 升序），再插这一版新有的（按 key 升序）。先删让删空的节点先摘掉、树不因换号先长后缩；
  反过来先插会让每片叶在一次发布里先涨一倍再缩，切出一批半空的叶。只决定形状，不决定写序（写的是最终态，D8 已定项 13）。推翻：主 agent / 用户定先插后删或原地换 key。
- **B2 记账树每次发布整批换代**：今天的实现每次发布写一套代 = txg 的行、不留旧代（行数恒 3 + 6 × 盘数），而 D8 已定项 1 写「每个统计量 × 每一代各占一条」。这是分裂之前就有的出入，
  我没动它；它决定了记账树的形状只随行数与容量变——两块盘的池上记账树永远 15 行，产品容量下永远一个节点，80 块盘起才分裂，分裂之后每次发布整棵重写。层 0 的两条记账树流因此只能靠压小容量造。
- **B3 多层之后映射树的「容量准入」**（D19 已定项 5）：多层码 2 树没有条目数上限，剩下的结构上限只有层级 1 字节（257 层写不下），在规划那一步按这次的真实条目数判
  （`HeightBeyondTheLevelField`）；分裂多出来的节点落到分配记录树那一条准入与末条点名项上（后者实二十已做末条再跨记录，今天的用例都在 67 项以内）。
  「按现在的条目数与最多新增的条目数」我读成「在任何落盘之前按这次真要写的条目算出形状」，没另立一个条目数上界。推翻：条款要一个独立于形状的条目数上界。
- **B4 ckpt_cost 没接线**：树高从根节点头现读的函数给了（`height_read_from_the_root_node_header`），但 D28 已定项 4 的 ckpt_cost 今天没有调用点（准入式子没接进发布路径，
  `admission.rs` 模块文档；「记录树」指哪几棵仍是 C363 那一格）。没接，停在这里。「一次发布最多分裂几次」没另立上界：取号之前那一串按形状逐次精确推（第一节第 4 条），发布路径按真规划算。
- **B5 从盘上重建的树分隔 key 坏了**（走得到：坏盘）：重建按「拼得成树」读、不核分隔 key，下一次发布按分隔 key 删 key 删不掉 ⇒ 规划交回
  `PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt`，可写挂载在取号之前拒（`MountError::RowPublishAdmissionRefusedBeforeAcquisition`），盘上逐字节不变（用例
  `a_rebuilt_central_mapping_root_whose_separator_hides_a_key_is_refused_before_the_instance_generation_is_acquired` 用 `DiskSnapshot` 钉）。遇到这种盘该怎么办（修复、只读挂载）条款没写，
  第一版只拒、不钉拒之后的行为。另一种读法是重建也按走读那一套全核（那样同一份坏盘在挂载时就报 I-1.1）——交主 agent。
- **B6 读树时叶条目宽不判**：读树只切 key，叶条目窄于字段表由解条目的一方各报各的（冷走读 / 重建 `EntryNarrowerThanItsFieldTable`，挂载态 `RecordMalformed`），
  与树长成多层之前逐字相同（坏盘输入那张期望表一格不用改）。内部条目窄于 key + 86 由读树报 `EntryNarrowerThanItsFieldTable { what: "记账树内部条目" / "中央映射树内部条目" }`。
- **B7 走不到、写成断言的**（为什么走不到写在断言旁）：`plan_the_tree_after_this_publish` 断言这一版 key 不空（记账恒有池级 3 行；映射恒有分配记录树、记账树、inode 根的条目）、
  容量叶 ≥ 1 / 内部 ≥ 2（开关 setter 先判过）；装单元时断言计划里叶的 key 与装出来的条目 key 是同一个集合、映射 key 与 `resolve` 预先算的逐项相等（发布路径自己的不变量）；
  mount.rs 里取号之前在拷贝上推两棵树 `expect`「准入刚按同样输入推过、没拒」。模型（harness `model.rs`）只罩单节点树：写记账行时行数装不下一个节点就断言（今天的随机历史两块盘，走不到）。
- **B8 条款没写、没加的**：`CodeTwoTreeKey` 等只加了用得到的 derive；没给多层树加「低于一半合并」「借条目」（已定项 11 ③ 明写不做）；extent 树、分配记录树、inode 根的内部节点这一轮不碰。
- **B9 依赖实二十**：一次发布分裂多出来的节点增加点名项；实二十（末条再跨记录）已在 14:50Z 打进主工作区，我的补丁建在它之上，发布路径里不再有「点名项多于 67 就拒」那一判。
  我的用例都在 67 项以内（最大的是两棵树都压小的第一个文件版本，24 个角色）。

## 六、不变量条文草稿与要跟着改的 kb（交书记员）

checker 这一轮把新判定放在已有编号下（I-1.1、I-1.10），免得清单条数与 `IMPLEMENTED_INVARIANTS` 在书记员落条文之前不一致。要另立编号的话，草稿（编号留空）：

> **I-1.__ 多层码 2 树的节点按父条目自证**：记账树与中央映射树（D8（核心索引结构） 已定项 11）的任一节点，头里的层级、key 区间与引用它的父条目一致——
> ① 孩子头里的层级 = 父节点头里的层级 − 1，层级 0 是叶；② 父节点的条目按分隔 key 严格递增、不空；第 i 条的分隔 key ≤ 第 i 个孩子头里的最小 key，且 > 第 i − 1 个孩子头里的最大 key；
> ③ 内部节点头里的 key 区间 = [第一个孩子头里的最小 key, 最后一个孩子头里的最大 key]（D18（块里携带什么信息） 已定项 2 的子树覆盖区间），叶头里的区间 = 首末两条条目的 key；
> ④ 树高 = 根节点头里的层级 + 1。判的全是块头自带的字段，不读任何旁路表。

不另立编号时要改的两行：
- **I-1.1** 的判定说明补一句：「多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间」；
  checker 状态列补 `walk::Walk::walk_code_two_subtree` 与本轮坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`。
- **I-1.10** 的登记宽度补「记账树内部节点 108（key 22 + 子指针 86）」；中央映射树仍不判（C307）。

跟着改的 kb（我没写，交书记员）：
- D19（块指针的结构与宽度预算） 已定项 5 依据第 114 行「多层映射第一版拒绝打开」已不成立（挂载态整棵读进来）；C483 那一行同理；「提示过期一次解引用 3 次读」多层重量结果 = 3（第一节第 3 条）。
- `crates/singlefs-format` 新常量 `ACCOUNTING_INTERNAL_ENTRY_BYTES = 108`、`CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113` 的 doc 注释里写了 `format-const:` 名字，kb（D8 已定项 11 那一句或布局表）要补对应的
  `<!-- format-const: 名字 = 值 -->` 标记，门禁 27 号才绑得上（今天没登记、27 号不管它）。
- 里程碑收口表第 28 行：记账树与中央映射树的分裂落地（这一份），分配记录树与 extent 树仍等 `m2-keyspace-r1`；C478（码 2 头 key 区间取子树覆盖）在这两棵树上已按覆盖区间写与判，inode 树根仍是首末分隔 key。
- 错误成员：`AccountingEntriesExceedOneNode`、`MappingEntriesExceedOneNode` 删，`MultiLevelCodeTwoTreeRefused` 新增；`OpenPoolForReadFailure::CentralMappingWithMoreThanOneLevelIsNotSupportedInTheFirstVersion` 删。

## 七、验证（第 4 步那几样；最终的树 = rebase3 = 16:15:53 的主工作区 + 补丁；各贴末尾原样输出）

**合了几次主工作区**：三次——14:0x（base2，手合 recovery.rs 三处、walk.rs / mount.rs 导入）、14:50Z 实二十之后（base3，零冲突）、16:15Z 实十六接续之后
（base5，`patch --merge` 冲突 8 处全部手合：walk.rs 导入与 `references_of_root`、mount.rs 导入与拷贝上那一串、transaction.rs 两处、model.rs 两处；
另补两处穷举臂）。合并之后逐文件核过：主工作区那一边与我这一边各自增删的行都在（按行的多重集比对，只差手合的那几处）。

**`git apply --check`**（对主工作区现状，只读）：`APPLY-CHECK-OK 16:47:18`；补丁在 base5 快照上 `git apply` 之后与 rebase3 逐文件相同。主工作区 16:15:53 之后只动了
`mutations.tsv` 与 `e158_root_choice_repair.rs`，都不在补丁里。`git apply --stat`：
```
 crates/singlefs-checker/src/lib.rs                 |   22 
 crates/singlefs-checker/src/walk.rs                |  423 ++++-
 crates/singlefs-core/src/code_two_tree.rs          | 1366 ++++++++++++++++
 crates/singlefs-core/src/lib.rs                    |    1 
 crates/singlefs-core/src/mount.rs                  |  258 ++-
 crates/singlefs-core/src/mounted_read.rs           |   84 +
 crates/singlefs-core/src/recovery.rs               |  291 ++-
 crates/singlefs-core/src/transaction.rs            | 1667 ++++++++++++++++----
 crates/singlefs-core/src/write_accounting.rs       |    8 
 crates/singlefs-format/src/lib.rs                  |   16 
 crates/singlefs-harness/src/history.rs             |    3 
 crates/singlefs-harness/src/model.rs               |   16 
 crates/singlefs-harness/src/model_comparison.rs    |   17 
 .../tests/common_tree_split/mod.rs                 |  288 +++
 .../tests/first_transaction_step_five_publish.rs   |    7 
 .../second_transaction_mapping_node_admission.rs   |  257 +--
 ...d_transaction_parallel_line_two_mounted_read.rs |  225 ++-
 ...nsaction_supplement_two_accounting_node_full.rs |  131 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  140 +-
 ...second_transaction_supplement_two_tree_split.rs |  793 ++++++++++
 ...transaction_supplement_two_tree_split_layer0.rs |  385 +++++
 21 files changed, 5560 insertions(+), 838 deletions(-)
```

**`cargo fmt --all -- --check`**：exit 1，只红在主工作区自己的 `e158_root_choice_repair.rs`（E158 那一路在改，补丁不碰）：
```
Diff in crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:2261:
Diff in crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:2799:
```
补丁里的 21 个文件单独 `rustfmt --check --edition 2021`：`rustfmt-mine-exit=0`。

**`cargo clippy --offline --all-targets --all-features -- -D warnings` + check.sh 那七条编码纪律 lint**：
```
    Checking singlefs-harness v0.1.0 (/tmp/claude-1000/impl-m2-treesplit/rebase3/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.16s
clippy-exit=0
```
**`cargo build --offline --all-targets`**：
```
   Compiling singlefs-harness v0.1.0 (/tmp/claude-1000/impl-m2-treesplit/rebase3/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.66s
build-exit=0
```

**动到的测试二进制**（release，整个二进制，不按名字挑；名字含 layer0 的不跑）：我改过的 15 个 + 主工作区 16:15Z 这一批改过、与补丁同一段代码的 7 个（含新加的
`instance_table_second_page_write`）；`run-chain3.sh`，16:46:55 跑完：
```
     Running tests/checker_known_bad_images.rs
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 360.09s
     Running tests/first_transaction_step_five_publish.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 86.75s
     Running tests/first_transaction_step_two_data_unit.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.36s
     Running tests/second_transaction_mapping_node_admission.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/second_transaction_parallel_line_one_last_record_flag.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 63.31s
     Running tests/second_transaction_parallel_line_one_multi_unit_file.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 79.56s
     Running tests/second_transaction_parallel_line_three_many_inodes.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.78s
     Running tests/second_transaction_parallel_line_two_mounted_read.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.26s
     Running tests/second_transaction_step_four_rollback.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 153.01s
     Running tests/second_transaction_step_one_overwrite.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 151.86s
     Running tests/second_transaction_step_three_second_instance.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 65.63s
     Running tests/second_transaction_supplement_three_bad_disk_input.rs
test result: ok. 9 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.58s
     Running tests/second_transaction_supplement_two_accounting_node_full.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/second_transaction_supplement_two_commit_generated_fallback.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 54.62s
     Running tests/second_transaction_supplement_two_instance_table_chain.rs
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
     Running tests/second_transaction_supplement_two_instance_table_page_full.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/second_transaction_supplement_two_instance_table_second_page_write.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.29s
     Running tests/second_transaction_supplement_two_release_checksum_quarantine.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 108.24s
     Running tests/second_transaction_supplement_two_row_publish_admission.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.46s
     Running tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 171.64s
     Running tests/second_transaction_supplement_two_tree_split.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
exit=0
```
三个库单测（release）：
```
     Running unittests src/lib.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs
test result: ok. 105 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running unittests src/lib.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```
（依次是 singlefs-checker 3、singlefs-core 105、singlefs-format 5。）

**第 3 步证红**：见第三节（22 行在合并之前的树上证过，#43 在合并后的树上重证）。

**门禁**（副本 `gatecheck` = rebase3 + 删 25 加 48 之后的表，16:25:39）：
```
33-mutation-tables.sh
  ✓ 147 个实验二进制都有成形的变异表，1573 条变异的原文各命中源码一次；crates/mutations.tsv 607 条的原文各命中源码一次
53-format-const-placeholders.sh
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成…
93-feature-bits.sh
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 1 位、登记表分出去 1 位；扫了 48 …
94-checker-implementation-disjoint.sh
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖
```
89、92 号在副本上 exit 77（不是 git 仓），74 号要跑 cargo、没跑。

**主工作区 `git diff --stat -- crates litmus`**（16:47，我没动主工作区；这是别的会话在主工作区里的未提交改动，原样贴，分不出谁改的）：
```
 crates/mutations.tsv                               |  114 +-
 crates/singlefs-checker/src/image.rs               |    8 +-
 crates/singlefs-checker/src/lib.rs                 |   10 +-
 crates/singlefs-checker/src/walk.rs                |  609 +++++-
 crates/singlefs-core/src/allocator.rs              |   98 +-
 crates/singlefs-core/src/instance_table.rs         |  135 +-
 crates/singlefs-core/src/journal.rs                |  131 +-
 crates/singlefs-core/src/mount.rs                  |  456 ++++-
 crates/singlefs-core/src/mounted_read.rs           |    3 +-
 crates/singlefs-core/src/recovery.rs               |  405 +++-
 crates/singlefs-core/src/transaction.rs            |  905 ++++++---
 crates/singlefs-core/src/write_accounting.rs       |    4 +-
 .../src/bin/e158_root_choice_repair.rs             | 2021 +++++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |   95 +-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    4 +-
 crates/singlefs-harness/src/history.rs             |   52 +-
 crates/singlefs-harness/src/model.rs               |   91 +-
 crates/singlefs-harness/src/model_comparison.rs    |   33 +-
 crates/singlefs-harness/src/scenario.rs            |    2 -
 .../tests/checker_known_bad_images.rs              |  727 ++++++-
 .../tests/first_transaction_step_five_publish.rs   |   35 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 -
 ...ransaction_parallel_line_one_multi_unit_file.rs |   45 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  223 ++-
 .../tests/second_transaction_step_four_rollback.rs |    8 +-
 .../tests/second_transaction_step_one_overwrite.rs |    1 -
 ...second_transaction_step_three_formatted_pool.rs |   45 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |   10 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ...transaction_supplement_three_fault_injection.rs |   35 +
 ..._transaction_supplement_three_random_history.rs |  182 +-
 ...two_c533_row_publish_record_without_its_root.rs |    3 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   73 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   45 +-
 ...tion_supplement_two_instance_table_page_full.rs |  278 ++-
 ...n_supplement_two_release_checksum_quarantine.rs |  551 ++++--
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  276 ++-
 40 files changed, 6553 insertions(+), 1216 deletions(-)
```

全量 `cargo test --all`、层 0 各流、门禁 59 号整表复跑没跑，留给提交时统一的那一次验证。

## 八、没做什么

- 没跑层 0：七条新流（第四节）在最终的树上一条没跑，`second_transaction_supplement_two_tree_split_layer0` 这个二进制只保证编得过；已有层 0 流的快档、全量都没跑；
  名字含 layer0 的二进制一律没跑（主 agent 15:1x 的指示）。13:3x 那次全量（225329 个状态零违例）是对齐主工作区之前的副本上跑的，只作参考。
- 没跑全量 `cargo test --all`、没跑门禁 59 号（整表复跑）、74 号（要跑 cargo）、89 / 92 号（不是 git 仓，副本上 exit 77），留给提交时统一的那一次验证。
  跑过的门禁：33、53、93、94（第七节）。
- 没走三方对抗；QEMU、herd7 与 crates 变异表整表复跑归崩溃验证员；没提交、没动主工作区（合并只在 `/tmp/claude-1000/impl-m2-treesplit/rebase3` 里做）。
- 没写 kb、没写 `research/`：不变量条文草稿与要跟着改的 kb 列在第六节，交书记员。
- 分配记录树、extent 树的分裂不在这一轮（等 `m2-keyspace-r1`）；inode 树根的 key 区间仍是首末分隔 key。
- D28 已定项 4 的 ckpt_cost 没接线（第五节 B4）；多层树没有「低于一半合并」「借条目」（已定项 11 ③ 明写不做）。
- 没改 `e158_root_choice_repair.rs`（E158 那一路在改）：它的 `allocation_record_tree_reachable_placements_via_central_mapping` 把中央映射树的根当叶解，
  这个补丁落地之后映射条目多于 294 条（长成两层）的镜像上它会报「中央映射条目解不开」、不会静默算错；E158 的历史会不会长到 294 条以上我没核。交主 agent 转那一路。
- 被测代码之外没有再加断言或访问器；条款没写、没加的逐项在第五节 B7、B8。
