# 实审 B2b 报告（B2 收尾：变异行与证红、11 行腐化锚点、被带红的测试、fmt / clippy / build）

写于 2026-09-27。实现员，主工作区改测试与 checker 的 clippy 四处；`crates/mutations.tsv` 一个字没动，行放在草稿目录。

## 一、结论

- **变异行**：`crates/mutations.tsv` 里 B2 带腐化的 11 行给了新锚点，另有 2 行（875、879）因为我改了它们点名的测试名、第 5 / 6 段跟着换——共 13 行整行替换，写在 `/tmp/claude-1000/impl-rev-b2b/mutations-replacements.tsv`；新写 24 行，在 `/tmp/claude-1000/impl-rev-b2b/mutations-append.tsv`（B2 报告第五节第 1 条点名的全部，外加 3 条阳性对照的「判严」变异、2 条盯我改了期望的 formatted_pool 两处）。
- **证红**：37 行全部在副本上用 `prove-red.sh` 跑过，37 条全抓到（`✓ 点名 37 条：跑了 37 条，跳过 0 条，跑的都抓到了`）。副本的变异表换上这 37 行之后，门禁 33 号在副本上判绿（`crates/mutations.tsv 997 条的原文各命中源码一次`）。
- **测试期望**：B1 那份 4 条照第六节改完（3 条映射 key 的期望换成 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`、函数名跟着改；计数器 0 改判违例）；formatted_pool 三张不适用名单照第六节改完。**另有第六节没列的 1 条**：formatted_pool 里 C554 那一形的历史，B2 补上被抛弃根那一半之后照实红 I-7.4，我把它的期望从「一条违例都没有」改成「只红 I-7.4、红在被抛弃根 (2, 3) 上」（第五节；这一处请主 agent 过目）。
- **checker 源码**：只为 clippy 改了 `walk.rs` 四处（type_complexity、needless_borrow、shadow_unrelated 两个绑定），不动判定。
- **主工作区编不过**：`singlefs-core` 别的会话在改（先是 `admission.rs`，后来是 `allocator.rs`），harness 在主工作区编不出来。测试二进制、harness 那两个目标的 clippy 与 `cargo build --all-targets` 都在副本上跑（副本 = 取副本那一刻的主工作区 + 我的改动，walk.rs 与主工作区逐字节相同），全绿；checker 的 clippy 与 fmt 在主工作区上跑，绿。

推翻条件：主 agent 把两份 tsv 写回主表之后，门禁 33 号在这 37 行上报「不是恰好命中一次」；或者 `core` 编好之后，在主工作区跑我动的两个测试二进制，有一条不过。

## 二、这一轮写过的文件

- `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`（未跟踪文件）：import 加 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`；三条映射 key 用例改期望、改名（旧名 → 新名：`…reddens_only_the_birth_identity_invariant` → `…reddens_only_the_mapping_key_invariant`；`…the_unit_it_names_reddens_only_the_class_tag_invariant` → `…reddens_only_the_mapping_key_invariant`；`…unregistered_class_reddens_only_the_class_tag_invariant_and_is_not_read` → `…unregistered_class_reddens_only_the_mapping_key_invariant_and_is_not_read`），文档注释跟着改；`a_journal_record_whose_counter_is_zero_is_judged_without_a_panic` 改成断言 I-8.6 违例、说明里带「计数器 0」（名字没改：不 panic 仍是它钉的事）。
- `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`：import 加那个常量；`NOT_APPLICABLE_WITHOUT_FILE` 20 → 21、`NOT_APPLICABLE_RIGHT_AFTER_MKFS` 22 → 23（各加映射 key 那一条），`NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE` 18 → 17（加映射 key、去掉 I-3.9 与 I-5.4），三处文档注释跟着改（含主验收用例注释里的成立条数：23 / 25 / 42，按 46 − 名单长度算）；`assert_no_checker_violation` 换成 `assert_only_the_abandoned_root_the_blind_mount_reused_reddens_the_reuse_invariant`（第五节）。A1b 改过的那个常量（`NOT_APPLICABLE_WITH_ONE_FILE_VERSION`，4 条）没动。
- `crates/singlefs-checker/src/walk.rs`（只为 clippy）：第 208 行新加 `type PlacementReferencedBelowAWalkedUnit = ((u32, u64), String);`，第 235 行 `placements_referenced_below_a_walked_unit` 的类型改用它；第 1457 行 `read_u64(&unit, 53)` → `read_u64(unit, 53)`；第 4670 行点名项位置条目那个闭包的绑定 `(device, slot, checksum)` 改成 `(named_device, named_slot, checksum)`，rustfmt 重排成块形。
- `crates/mutations.tsv`：**没改**（主 agent 按名字写回）。要替换的 13 行的名字（今天主表里的行号）：308、309、310、319、494、759、865、870、874、875、877、878、879；要追加的 24 行名字见第四节表的第 14–37 行。
- 草稿目录 `/tmp/claude-1000/impl-rev-b2b/`：`mutations-replacements.tsv`（13 行，sha256 `8a5b38201c85e714240ceb1fac6d1915ffc494ad06590a61f6acf862270ea0fb`）、`mutations-append.tsv`（24 行，sha256 `024c67c42e6959ca05274be361ab849077e8ce26a4f37d2ad39c582c875de18a`），与 `crates/mutations.tsv` 同为六段制表符分隔；`patch/` 里放同样两份与这份报告，可直接交 `research/scripts/apply-writer-patch.py`（没有 `crates.patch`：代码改动已在主工作区）。证红日志 `prove-red-logs/`、`prove-red-run-1.out`，基线日志 `baseline/`，终验日志 `final/`，门禁日志 `gate-*.log`，`progress.md`。

腐化锚点里「行为已被改掉、换成盯哪一处」的：
- 308 / 759：原先盯 `record.commit_marker == Present && instance_table_rows.iter().any(…)`；今天中间多了 `record_can_enter_a_replay_prefix(…)`（A2a 前缀口径），308 改成盯 `)\n            && instance_table_rows…` 那个 `&&`（换 `||`），759 改成把行那一段整条换成 `&& true;`，两条行为与原先各自那一句相同。
- 309 / 310 / 319：同一行，缩进从 12 空格变 8 空格（代码挪进了 `versions_applied_only_by_records` 的 `for` 体）。319 那一句全文件有两处（另一处在 `tree_table_pointer_of_the_version_the_next_mount_applies_first`，4 空格），8 空格的只命中一次。
- 870：计数器 0 如今有自己的一臂（`0 => CounterZeroHasNoLogicalPredecessor`），「减一下溢」要连那一臂一起去掉才走得到：原文从 `0 =>` 那一行起到 `{` 止，替换文只留 `FIRST_JOURNAL_COUNTER` 一臂与 `records.get(&(counter - 1))`。点名测试不变（B1 那条如今断言违例，变异让它 panic：`walk.rs:4723` `attempt to subtract with overflow`）。
- 874 / 877 / 878：判定点从 `"I-1.6"` / `"I-1.2"` 搬到 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，877 的比较挪到单独一行（只盯 `header_class_tag == key_class_tag,`）；点名测试换成改名后的三条。
- 875 / 879：锚点没腐化，只因点名测试改名换了第 5 / 6 段。

## 三、证红（`research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-b2b/copy --memory 8G singlefs-harness <37 个名字>`，经 `capped.sh 4`，2026-09-27）

副本 = 取副本那一刻的主工作区 rsync（去 target、.git）+ 我改的两份测试；副本的 `crates/mutations.tsv` 换上 13 行、追加 24 行之后跑。prove-red 每组参数先跑一次不改源码的基线（都绿），再逐条改坏、跑、还原。行的参数照表里惯例带 `-- <测试名>` 过滤，所以每条只看到点名那一条红，「同时红了哪些」看不到；整个二进制的基线红集另跑，在第四节。行号是主工作区今天的（clippy 那四处改完之后，原文在 walk.rs 里仍各命中一次，`count_anchors.py` 数过 37 行都是 1）。第 1–13 行是替换，第 14–37 行是追加（与 `mutations-append.tsv` 同序）。

原样末行：

```
✓ 点名 37 条：跑了 37 条，跳过 0 条，跑的都抓到了
```

| # | 表 | 变异名 | 改坏哪一行（主工作区现行行号） | 点名测试 | 红在哪条断言（prove-red 日志里第一处 panic） |
|---|---|---|---|---|---|
| 1 | 替换 | 增补 2 收口第 54 行（候选 b）第 ① 条放宽：不看实例表里有没有这个实例的行，环里带提交标记的记录都算施加过 | `walk.rs:5317` | `journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds` | 抓到：`checker_known_bad_images.rs:4015` assertion `left == right` failed: 挂载之前那一刻一条都不许红：[("I-1.1", Holds), ("I-1.2", Holds), ("I-1 |
| 2 | 替换 | 增补 2 收口第 54 行（候选 b）第 ③ 条去掉：低于回退下界 F 的那一版也并进遍历 | `walk.rs:5325` | `version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it` | 抓到：`checker_known_bad_images.rs:4166` assertion `left == right` failed: F 抬过 (1, 5) 之后一条都不许红：[("I-1.1", Holds), ("I-1.2", Holds) |
| 3 | 替换 | 增补 2 收口第 54 行（候选 b）第 ④ 条去掉：环里最旧的有效根已比那一版新（它换下的单元已可回收）也并进遍历 | `walk.rs:5326` | `version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring_has_turned_past_it_and_the_mount_reclaimed_its_units` | 抓到：`checker_known_bad_images.rs:4346` assertion `left == right` failed: 环转过 (1, 5) 之后那一版不再并进遍历，一条都不许红：[("I-1.1", Holds), ("I-1.2 |
| 4 | 替换 | 增补 2 收口第 54 行（候选 b）第 ② 条去掉：根槽读得出的那一版也从它的记录再并进一次遍历 | `walk.rs:5322` | `version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_the_leak_on_top_still_reddens_it` | 抓到：`checker_known_bad_images.rs:3999` 机理标识要报出那一版并进了遍历：盘 0：记账的已分配 Some(1048576)，遍历全部有效根得到 1032192（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 7、 |
| 5 | 替换 | 收口表第 46 行 C480：I-9.15 按 ⌊size ÷ 512⌋ 比（向下取整，写路径写的 6 在 3000 字节上被判红） | `walk.rs:1476` | `inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant` | 抓到：`checker_known_bad_images.rs:2546` assertion `left == right` failed: 干净镜像上 blocks = 6：I-9.15 真被评估过且成立 |
| 6 | 替换 | 实四乙（实三交回 Q2 查原因：恢复之前的镜像上 checker 只把已被某次恢复施加的记录并进遍历，I-3.1 多出来的正好是被写坏根槽的那条根独占的单元）：checker 把没被任何恢复施加过的记录也并进遍历 | `walk.rs:5318` | `plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced` | 抓到：`second_transaction_step_four_rollback.rs:1806` assertion `left == right` failed: 恢复之前的镜像：只有 I-3.1 红 |
| 7 | 替换 | 实审 B1 第 5 条：checker 同一版里第二次引用同一个落点判成立（两个 inode 的 extent 指同一个数据单元照绿） | `walk.rs:549` | `two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_only_the_disjoint_ranges_invariant` | 抓到：`checker_cross_links_malformed_nodes_and_mapping_entries.rs:219` assertion `left == right` failed: 判红的该只有 I-5.1：[] |
| 8 | 替换 | 实审 B1 第 6 条：checker 反向链拿计数器减一找逻辑前一条（计数器 0 下溢 panic） | `walk.rs:4726` | `a_journal_record_whose_counter_is_zero_is_judged_without_a_panic` | 抓到：`walk.rs:4723` attempt to subtract with overflow |
| 9 | 替换 | 实审 B1 第 8 条：checker 映射 key 的类标签不在登记表里也判成立 | `walk.rs:871` | `a_mapping_key_with_an_unregistered_class_reddens_only_the_mapping_key_invariant_and_is_not_read` | 抓到：`checker_cross_links_malformed_nodes_and_mapping_entries.rs:706` assertion `left == right` failed: 类标签 0：判红的该只有映射 key 与单元头相符那一条（不读、I-2.1 不红）：[] |
| 10 | 替换 | 实审 B1 第 8 条：checker 映射 key 的类标签认不得照样按 32 KiB 硬读 | `walk.rs:877` | `a_mapping_key_with_an_unregistered_class_reddens_only_the_mapping_key_invariant_and_is_not_read` | 抓到：`checker_cross_links_malformed_nodes_and_mapping_entries.rs:706` assertion `left == right` failed: 类标签 0：判红的该只有映射 key 与单元头相符那一条（不读、I-2.1 不红）：[("I-2.1", "映射 |
| 11 | 替换 | 实审 B1 第 8 条：checker 不拿映射 key 的类标签比被指单元头里的 | `walk.rs:898` | `a_mapping_key_whose_class_is_not_the_class_of_the_unit_it_names_reddens_only_the_mapping_key_invariant` | 抓到：`checker_cross_links_malformed_nodes_and_mapping_entries.rs:676` assertion `left == right` failed: 判红的该只有映射 key 与单元头相符那一条：[] |
| 12 | 替换 | 实审 B1 第 8 条：checker 不拿映射 key 的出生身份比被指单元头里的 | `walk.rs:924` | `a_mapping_key_whose_birth_identity_is_not_the_one_in_the_unit_header_reddens_only_the_mapping_key_invariant` | 抓到：`checker_cross_links_malformed_nodes_and_mapping_entries.rs:654` assertion `left == right` failed: 判红的该只有映射 key 与单元头相符那一条：[] |
| 13 | 替换 | 实审 B1 第 8 条：checker 映射 key 说码 2 也按 32 KiB 读、记两槽引用 | `walk.rs:870` | `a_mapping_key_whose_birth_identity_is_not_the_one_in_the_unit_header_reddens_only_the_mapping_key_invariant` | 抓到：`checker_cross_links_malformed_nodes_and_mapping_entries.rs:654` assertion `left == right` failed: 判红的该只有映射 key 与单元头相符那一条：[("I-2.1", "映射条目指的单元 在盘 0 槽 50240 |
| 14 | 追加 | 实审 B2 第 7 条（I-7.4 被抛弃根那一半）：被抛弃的根走出对不上的单元也判成立 | `walk.rs:5410` | `an_abandoned_root_whose_units_a_mount_blind_to_it_handed_out_again_reddens_only_the_reuse_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:549` assertion `left == right` failed: C 的槽 [50184] 被重写：只该红 I-7.4：[] |
| 15 | 追加 | 实审 B2 第 7 条（I-7.4 被抛弃根那一半）：根环里被判抛弃的根一条都不走 | `walk.rs:5391` | `erasing_a_unit_only_an_abandoned_root_references_reddens_only_the_reuse_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:583` assertion `left == right` failed: C 的数据单元只有被抛弃的 C 引用：抹掉它只红 I-7.4：[] |
| 16 | 追加 | 实审 B2 第 7 条（I-7.4 被抛弃根那一半）：根环有读不出的槽时不报判不了（照报成立） | `walk.rs:5380` | `an_unreadable_root_ring_slot_leaves_the_abandoned_half_of_the_reuse_invariant_not_judged` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:618` 被抛弃根那一半判不了，I-7.4 不许报成立：Holds |
| 17 | 追加 | 实审 B2（B1 缺口：共享子树）：别的版本走过的单元下面的落点并进这一版时不判同一版第二次引用 | `walk.rs:581` | `an_older_version_naming_again_a_placement_inside_a_subtree_it_shares_with_a_newer_version_reddens_the_disjoint_ranges_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:676` assertion `left == right` failed: F 这一版里 D 被引用两次，只该红 I-5.1：[] |
| 18 | 追加 | 实审 B2 第 9 条（I-2.5 点名项）：journal 点名项的位置条目不判升序 | `walk.rs:4693` | `a_journal_named_entry_whose_location_entries_are_not_in_ascending_device_order_reddens_only_the_location_order_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:722` assertion `left == right` failed: 只该红 I-2.5：[] |
| 19 | 追加 | 实审 B2 第 9 条（I-9.2 类型段 0）：类型段 0 的条目不判「三段为 0」 | `walk.rs:1386` | `a_type_zero_inode_entry_whose_container_segment_is_not_zero_reddens_only_the_entry_identity_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:832` assertion `left == right` failed: 只该红 I-9.2：[] |
| 20 | 追加 | 实审 B2 第 9 条（I-9.2 类型段 0）：类型段 0 指着的码 2 节点判完头不往下走 | `walk.rs:1593` | `a_legal_type_zero_inode_entry_is_walked_down_to_the_leaf_and_every_invariant_holds` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:817` assertion `left == right` failed: I-9.4 要走到叶、真被评估过且成立 |
| 21 | 追加 | 实审 B2 第 9 条（I-9.4 第二句）：沿叶序容器号严格递增不判 | `walk.rs:1298` | `container_numbers_going_backwards_along_the_leaf_order_redden_only_the_container_number_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:950` 第一处违例是「沿叶序容器号严格递增」那一句：左容器 1 的最大 key 10 不小于右容器号 0 |
| 22 | 追加 | 实审 B2 第 9 条（I-9.4 第三句）：左容器的最大 key 小于右容器号不判 | `walk.rs:1305` | `a_right_container_number_not_above_the_largest_key_on_its_left_reddens_only_the_container_number_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:934` assertion `left == right` failed: 只该红 I-9.4：[] |
| 23 | 追加 | 实审 B2 第 9 条（I-9.4 第三句）：判严一格（左最大 key + 1 < 右容器号），右容器号恰比左最大 key 大一的合法两片误红 | `walk.rs:1305` | `two_leaf_containers_in_key_order_hold_every_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:922` assertion `left == right` failed: 两片叶按序排好，一条都不许红：[("I-9.4", "左容器 1 的最大 key 10 不小于右容器号 11") |
| 24 | 追加 | 实审 B2 第 9 条（I-3.9 / C512）：根记录持有的那棵分配记录树的记录不并进候选根的账 | `walk.rs:2618` | `a_released_record_in_the_allocation_record_tree_the_root_record_holds_is_judged_by_the_release_generation_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:1054` assertion `left == right` failed: 根记录持有的那棵树里的已释放记录要真被评估过且成立 |
| 25 | 追加 | 实审 B2 第 9 条（I-5.4 / C512）：根记录持有的那棵分配记录树不判互不相交 | `walk.rs:3256` | `overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:1123` assertion `left == right` failed: 两条记录罩住同一个槽，只该红 I-5.4：[] |
| 26 | 追加 | 实审 B2 第 9 条（I-7.7 ①）：有读不出的槽时 ① 照判（照报成立） | `walk.rs:2417` | `the_first_sentence_of_the_instance_carrier_invariant_is_not_judged_when_a_slot_is_unreadable` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:1205` 有槽读不出：① 报不适用、② 没有对象，整条不适用 |
| 27 | 追加 | 实审 B2 第 9 条（I-7.7 ②）：有读不出的槽时 ② 不在读得出的载体上判 | `walk.rs:2436` | `the_second_sentence_of_the_instance_carrier_invariant_is_judged_on_the_readable_carriers_when_a_slot_is_unreadable` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:188` I-7.7 该判违例，实际 NotApplicable("根环、journal 环或单元区有读不出的槽：① 那一半按条款报不适用，② 在读得出的载体上没找到较大者，也判不全") |
| 28 | 追加 | 实审 B2（映射 key 与单元头相符）：码 2 / 码 3 的 key 尾段补零 2 字节不判 | `walk.rs:939` | `a_mapping_key_whose_padding_after_the_birth_sequence_is_not_zero_reddens_only_the_mapping_key_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:1326` assertion `left == right` failed: 只该红映射 key 那一条：[] |
| 29 | 追加 | 实审 B2（映射 key 与单元头相符）：码 1 / 码 3 的 key 出生树不比单元头偏移 43 | `walk.rs:955` | `a_mapping_key_whose_birth_tree_is_not_the_one_in_the_unit_header_reddens_only_the_mapping_key_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:1347` assertion `left == right` failed: 类标签 1：只该红映射 key 那一条：[] |
| 30 | 追加 | 实审 B2（映射 key 与单元头相符）：码 2 的出生树也按单元头偏移 43 比（C289 还开着） | `walk.rs:355` | `the_birth_tree_in_a_mapping_key_of_an_index_node_is_not_judged_while_its_field_is_open` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:1370` assertion `left == right` failed: 码 2 的出生树随 C289 定，今天不判 |
| 31 | 追加 | 实审 B2（计数器 0，用户 2026-09-27 定判违例）：计数器 0 当成逻辑前一条不在盘上、跳过不判 | `walk.rs:4726` | `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:188` I-8.6 该判违例，实际 Holds |
| 32 | 追加 | 实审 B2（A2a 第 25 条对齐，前缀口径）：认恢复施加过的版本时反向链对不上的记录也算进了前缀 | `walk.rs:5313` | `a_journal_record_whose_back_chain_does_not_match_is_not_walked_as_a_version_a_recovery_applied` | 抓到：`checker_known_bad_images.rs:4255` assertion `left == right` failed: 只该红 I-3.1 与 I-8.6：[("I-1.1", Holds), ("I-1.2", Holds), ( |
| 33 | 追加 | 实审 B2（A2a 第 25 条对齐，前缀口径）：认下一次挂载先施加的那一版时反向链对不上的记录也算进了前缀 | `walk.rs:3357` | `a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first` | 抓到：`checker_known_bad_images.rs:4285` assertion `left == right` failed: B 那条记录进不了前缀：只该红 I-8.6：[("I-1.1", Holds), ("I-1.2", Holds |
| 34 | 追加 | 实审 B2 追加件：由记录施加出来的那一版找不到它施加在其上那一版的根记录（不走那张实例表） | `walk.rs:5350` | `version_applied_only_by_its_journal_record_walks_the_instance_table_of_the_root_it_was_applied_on_even_below_the_floor` | 抓到：`checker_known_bad_images.rs:4211` assertion `left == right` failed: B 掉到 F 之下、(1, 5) 还在：一条都不许红：[("I-1.1", Holds), ("I-1.2",  |
| 35 | 追加 | 实审 B2 追加件（A2a 第 37 条对齐）：槽 0 无效的盘不借别的盘记的槽距，照 4096 找槽 1 | `image.rs:280` | `a_device_whose_slot_zero_does_not_verify_has_its_slot_one_looked_for_at_the_spacing_another_device_records` | 抓到：`checker_narrow_invariants_and_abandoned_roots.rs:1256` assertion `left == right` failed: 盘 0 的槽 1 在盘 1 记的槽距 8192 处找到；盘 1 两槽都自证过 |
| 36 | 追加 | 实审 B2b（formatted_pool 写行之后那一格 I-5.4 改判成立）：根记录持有的那棵分配记录树不判互不相交 | `walk.rs:3256` | `a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold` | 抓到：`second_transaction_step_three_formatted_pool.rs:633` assertion `left == right` failed: 写行那次发布之后：I-5.4 要真被评估过且成立 |
| 37 | 追加 | 实审 B2b（formatted_pool 那段 C554 历史改判只红 I-7.4）：被抛弃的根走出对不上的单元也判成立 | `walk.rs:5410` | `crash_recovery_abandoning_the_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them` | 抓到：`second_transaction_step_three_formatted_pool.rs:1805` assertion `left == right` failed: 卸载之后：只该红 I-7.4：[] |

## 四、交回前的验证（末行原样）

**整个二进制的基线红集**（副本，不改源码，改 C554 那一处期望之前，`baseline/summary.txt` 原样）：

```
checker_narrow_invariants_and_abandoned_roots exit 0 11s
checker_known_bad_images exit 0 22s
checker_cross_links_malformed_nodes_and_mapping_entries exit 0 1s
second_transaction_step_three_formatted_pool exit 101 6s
second_transaction_step_four_rollback exit 0 25s
```

formatted_pool 那一次唯一的红：`crash_recovery_abandoning_the_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them`，`second_transaction_step_three_formatted_pool.rs:1786`（旧行号）`卸载之后：I-7.4 判红：Violated("被抛弃的根（实例 2、txg 3）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：实例表单元 在盘 0 槽 50240 的那一份与位置条目里的校验和对不上；实例表单元两份都读不到对得上的；树表 0 条那一版的分配记录树（树 0）的根 两份都读不到对得上的")`。改了期望（第五节）之后这份 13 条全过；证红在这些二进制上跑时基线都绿，没有落在基线红集里的点名测试。

**副本终验**（walk.rs 与主工作区逐字节相同，`final/summary.txt` 与各日志末行原样）：

```
test checker_cross_links_malformed_nodes_and_mapping_entries exit 0
test second_transaction_step_three_formatted_pool exit 0
test checker_narrow_invariants_and_abandoned_roots exit 0
test checker_known_bad_images exit 0
test second_transaction_step_four_rollback exit 0
clippy checker exit 0
clippy harness two tests exit 0
build all-targets exit 0
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s    （checker_cross_links…）
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 69.94s   （checker_known_bad_images）
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.31s   （checker_narrow…）
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 81.25s   （step_four_rollback）
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.04s   （formatted_pool）
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.81s                          （build --all-targets，warning 0 行）
```

clippy 的 lint 集照 `check.sh`：`-D warnings` 加 `wildcard_enum_match_arm`、`allow_attributes_without_reason`、`cast_possible_truncation`、`cast_sign_loss`、`cast_possible_wrap`、`undocumented_unsafe_blocks`、`shadow_unrelated`；目标是 `-p singlefs-checker --all-targets` 与 `-p singlefs-harness --test checker_cross_links_malformed_nodes_and_mapping_entries --test second_transaction_step_three_formatted_pool`。

**主工作区**（core 一度编得过的那一刻；门禁 33 等六道在那时跑）：

```
build main exit 0 18s                        （cargo build --offline --all-targets，warning 0 行）
checker_cross_links_malformed_nodes_and_mapping_entries exit 0
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
second_transaction_step_three_formatted_pool exit 0
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.11s
clippy checker (main) exit 0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
clippy harness two tests (main) exit 101
error: could not compile `singlefs-core` (lib) due to 3 previous errors
```

主工作区 harness 那两个目标的 clippy 红在 `singlefs-core`：`admission.rs:665`、`admission.rs:666`（manual div_ceil）、`allocator.rs:216`（manual is_multiple_of）——不在我的改动里，别的会话在改，没碰；同样两个目标在副本上 clippy 绿。更早两次主工作区编不过（`admission.rs` 缺 `CodeTwoTreeNodeCapacities` 等类型、`allocator.rs` 多处），同一原因。

`cargo fmt --check`（主工作区）exit 1，Diff 全在别人的文件：`singlefs-core/src/admission.rs` 5 处、`allocator.rs` 1 处、`singlefs-harness/src/bin/e158_root_choice_repair.rs` 67 处、`tests/admission_checkpoint_cost_per_device_paths.rs` 5 处、`tests/core_review_unit_area_start_and_publish_limits.rs` 2 处；我动的三份（两份测试、walk.rs）在输出里命中 0 次，`rustfmt --edition 2021 --check` 单查 walk.rs exit 0，两份测试在草稿目录里配空桩单查也无 diff。

**登记给我的门禁阶段**（`stage-owners.tsv` 里 implementation-writer 的 7 道，主工作区跑，末行原样）：

| 阶段 | 退出码 | 原样 |
|---|---|---|
| 33 | 1 | `    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。`——点名 15 行：我交的 11 行（308、309、310、319、494、759、865、870、874、877、878）与实审 A4 的 4 行（974、975、977、978，不是我的，A4b 在改）。同一阶段在副本（表换上我的 37 行）上 exit 0：`✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 997 条的原文各命中源码一次；…` |
| 53 | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` |
| 89 | 77 | `⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`（末行另报「第 27 行写 4 条，这里探针与清单合计 3 条，对不上」，不在我的改动里） |
| 92 | 0 | `✓ 布局清单 1 套布局、6 条路径都在，…105 个格式常量里变了 7 个（checker 在同一次改动里跟了 7 个，按滞后表放行 0 个），都不欠 checker 跟进` |
| 93 | 0 | `✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；…）` |
| 94 | 0 | `✓ checker 与实现只共享常量模块 singlefs-format（…内部依赖图 4 个 crate）；checker 的 4 份源码零处引 singlefs_core…` |
| 74 | 1 | 在副本上跑（`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G bash .claude/gate.d/74-model-differential.sh <副本>`；主工作区那时编不过）：`  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）`，`test result: FAILED. 21 passed; 3 failed; 2 ignored; …`——见第六节 |

`apply-writer-patch.py /tmp/claude-1000/impl-rev-b2b/patch --dry-run`：`✓ 核过了（--dry-run，没改）：补丁 没有，变异表合并之后 1002 行`。

## 五、第六节之外改的那一处期望（请主 agent 过目）

`second_transaction_step_three_formatted_pool.rs` 的 `crash_recovery_abandoning_the_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them`：这段历史故意让 M3 看不见被抛弃的两条根（根槽清零），M3 的写行与 M2 的「逐槽相同」（用例自己断言的），也就是把被抛弃根 (2, 3) 引用的实例表 50240 与分配记录树根 50246 在它离开根环之前又发了一遍——正是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形。B2 补上 I-7.4 被抛弃根那一半之后，checker 在「卸载之后」「M4 之后」两处照实红 I-7.4。调度记录 `records/2026-09-24-里程碑二收尾调度.md` 第 215 行已写「C554 那一形改后照实只红 I-7.4」，I-7.4 原文「它们引用的块在离开根环之前同样不许重新分配、不许抹头」，我照这两处把期望从「一条违例都没有」改成「只红 I-7.4，说明里带『被抛弃的根（实例 2、txg 3）』」，helper 改名 `assert_only_the_abandoned_root_the_blind_mount_reused_reddens_the_reuse_invariant`，用例的文档注释跟着改；影子账那一半（隔离 2 槽、M4 不写那两槽）一字没动。追加表第 24 行（`实审 B2b（formatted_pool 那段 C554 历史改判只红 I-7.4）…`）证过它会红（`second_transaction_step_three_formatted_pool.rs:1805`，`卸载之后：只该红 I-7.4：[]`）。

这一处要不要换成别的钉法（例如改造历史让 M3 不复用那几槽），不归我定；要撤回就是把那两次调用换回「一条违例都没有」，那条用例就会照今天的 checker 红。

## 六、门禁 74 号在副本上的三条红（不在我的改动里；照写，没修）

在副本上 `cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history` 单跑一次（`random-history-copy-1.log`），三条红：

1. `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`（`…random_history.rs:1127`）：要以「已知红」第 0 条收尾，实际 `NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, …` 第 3 步 `PublishOverwrite`，说明「被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头…数据单元 在盘 0 槽 50184 的那一份与位置条目里的校验和对不上…」。
2. `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`（`…random_history.rs:256`）：`历史 96 段：跑完 67、以已知红收尾 {0: 1}、新发现 28`，新发现只有一种签名 `CheckerViolations { invariants: ["I-7.4"] }`（第一个种子 7463871032432355115，说明「被抛弃的根（实例 1、txg 4）引用的单元已被重新分配或抹头…盘 0 槽 50182…」）；统计行 `I-7.4：判绿 1682 次、不适用 0 次`（checker 跑了 1710 次）。
3. `unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval`（`…random_history.rs:496`）：`准入判着时式子先拒，落点那一道走不到`，`left: 2` `right: 0`。

前两条都是 B2 新加的 I-7.4 被抛弃根那一半在随机历史上的新红（B2 报告第六节第 3、4 条推过「会新红」），说明都点名「被抛弃的根」，改之前 checker 根本不走被抛弃的根，这一格不可能红——是 checker 判严之后照出来的，还是写者在抛弃之后抬 F / 回收时放掉了被抛弃根的影子账、是真违例，我没查（要读 `mount.rs` 的影子账与随机历史那几段的操作序列，不在这件活里）。第 3 条数的是落点那一道的拒绝次数，与 checker 无关（推的），副本里的 `singlefs-core` 是取副本那一刻的快照，A4 一族正在改准入。三条都交主 agent 分派。

## 七、受影响的层 0 流与崩溃枚举用例

我对 `crates/singlefs-checker/src/` 的改动只有 clippy 那四处（类型别名、去一个多余的借用、闭包绑定改名），任何一条不变量的判定集合都没变，所以我这一轮不让任何层 0 流或崩溃枚举用例的钉值变化。判定集合的变化全来自 B2 已落在主工作区的改动，影响面照 B2 报告第六节第 3–5 条（推的、没跑）：`first_transaction_step_seven_layer0.rs` 里新名字 `I-MAPPING-KEY` 的归类（B3a-3b 的活）；有被抛弃根的层 0 流上 I-7.4 可能新红——第六节的随机历史已经实测到这一形，层 0 上没跑；崩溃注入、故障注入的已知红清单若按签名认 I-7.4 要看。层 0 与崩溃注入我一条没跑。

## 八、交主 agent 的

1. 第五节那一处期望（formatted_pool 里 C554 那段历史改判只红 I-7.4）：照调度记录第 215 行与 I-7.4 原文改的，要换钉法由主 agent 定。
2. 我给 B1 那三条映射 key 用例改了名（旧名里的 class_tag / birth_identity 指的是搬走之前的 I-1.6 / I-1.2），所以替换表多了 875、879 两行（锚点没腐化，只换第 5 / 6 段），不是规格点名的 11 行之内。不想改名的话，把测试名改回、这两行不替换、874 / 877 / 878 三行的第 5 / 6 段换回旧名即可。
3. 追加表第 17 行（`码 2 的出生树也按单元头偏移 43 比（C289 还开着）`）钉的是「码 2 今天不比」；C289（码 2 的出生树取哪个字段没写） 定了之后这一行与它点名的阳性对照要一起改。
4. 追加表第 10 行是阳性对照 `two_leaf_containers_in_key_order_hold_every_invariant` 的判严变异：那条用例的右容器号 11 恰是左最大 key 10 + 1，正好卡在 I-9.4 第三句「<」的边界上，`+ 1 <` 让它误红。
5. 第六节门禁 74 号那三条红（两条 I-7.4 被抛弃根那一半在随机历史上的新发现、一条准入式子），要分派。
6. 规格第 2 步写「用 `replace-once.py` 定点改那 11 行」，派发提示改成「一个字都不改 `crates/mutations.tsv`、写进草稿目录」——我照派发提示办了，没有用 replace-once。

没有碰到条款没写、要我自己做设计判断的分支（这一轮没改判定逻辑）。

## 九、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表（门禁 59 号）归 `crash-verifier`；没提交。
- 没改 `crates/mutations.tsv`（派发提示明令），37 行在草稿目录等主 agent 写回；写回之后主工作区的 33 号才会在这 11 行上转绿（A4 那 4 行不归我）。
- 层 0 那份（`first_transaction_step_seven_layer0.rs` 的新名字归类）不归我，没动；名字带 layer0 的测试一条没跑。
- 门禁 74 号在主工作区没跑成（那时 core 编不过），在副本上跑了；它的三条红没查根因（第六节）。
- 主工作区 harness 那两个测试目标的 clippy 红在别人的 `singlefs-core`，没修；副本上绿。
- 没跑 `cargo test --all`、`check.sh`、`gate.sh` 整轮。
- 负载：开跑前 `ps` 没看到性能测量进程；中途看到别的会话在主工作区跑 `cargo build --offline --all-targets`、`cargo test … crash_enumeration_sharded…`、`… rollback_floor…`、`… core_review_tree_…`，我的编译大多在副本自己的 target 上跑，没等锁；主工作区那几次 build / clippy 都在几秒到 18 秒内返回。
- 清理：删了 `/tmp/claude-1000/impl-rev-b2b/copy`（仓副本连它的 target，删前 `du -sh` 14G）与 `/tmp/claude-1000/impl-rev-b2b/fmt`（rustfmt 单查用的草稿，128K）。草稿目录其余（两份 tsv、`patch/`、日志、`progress.md`、辅助脚本）留着，共 664K 左右，是主 agent 写回与复核要用的。

## 附：`git diff --stat -- crates litmus` 原样（含别的会话与此前各轮没提交的改动；我这一轮写的文件以第二节为准；B1 那份测试是未跟踪文件，不在表里）

```
 crates/mutations.tsv                               |  643 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1915 +++--
 crates/singlefs-core/src/admission.rs              |  612 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  299 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   37 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  199 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   43 +-
 crates/singlefs-core/src/recovery.rs               |  712 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  506 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6818 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2505 ++++++-
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
 .../tests/second_transaction_step_one_overwrite.rs |    3 +-
 ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
 ...second_transaction_step_three_formatted_pool.rs |  688 +-
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
 88 files changed, 29839 insertions(+), 11188 deletions(-)
```
