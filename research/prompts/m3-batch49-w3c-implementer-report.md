# 第 49 批 W3 再补实现员报告（模型罩记账树分裂、恢复判法的两条用例）

日期：2026-10-09。上一位报告 `~/.local/share/singlefs/work/m3-batch49/w3b/report.md`（sha256 1f1f8a87…），再前一位 `~/.local/share/singlefs/work/m3-batch49/w3/report.md`。
补丁目录：`~/.local/share/singlefs/work/m3-batch49/w3c/patch/`（合起来的一份，打在主工作区现在这一版上，不是增量）。

## 一、结论

- 上一位的补丁对主工作区现在这一版（合进根环线第十六到十九块之后）打得上：`apply-writer-patch.py --root <副本>` 退 0，code-source-discipline 的 mutation-tables 格绿（第五节）。在它上面接着改。
- 第 1 条（模型罩记账树分裂）落了，取「照中央映射树那样、根之下的节点不对应角色、占槽按节点数上界带」（理由第二节）：
  - `model.rs`：新函数 `accounting_tree_nodes_upper_bound`（pub），`ModelRoot` 多一段 `accounting_tree_nodes_upper_bound`；写记账树的发布按这次的行数从上一版的上界往下带，占槽按「节点数 × 1 槽」进占槽上界、按节点数进映射条目；去掉「记账树一个节点装不下就 panic」那一判与 `accounting_rows_exceed_one_node`；加 pub `current_version_in_the_session`。
  - `model_comparison.rs`：`AccountingTreeNodeBelowTheRoot(_) => return None`（同 `MappingTreeNodeBelowTheRoot`），注释写为什么。
  - 上界在「从一个节点长到 478 行」那一步恰是 3，与实现长成的根加两片叶相同。
  - `model_compares_*` 三个文件 5 条用例绿（第五节）。
- 第 2 条落了：`recovery.rs` 的新判法不动。
  - `core_review_…::a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking`：种的根的记账行代段跟着换成新 txg（重建记账叶、换树表里那条指针的校验和、重建树表），造得出来，没改成接受回落，名字不动；绿。
  - 新用例 `a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step_is_judged_damaged_by_the_rebuild_and_refused_by_the_writable_mount`：txg 4 那一版自己那一行水位标签改成 13、只剩台阶 (12, 3)。`rebuild_version` 判 `InodeNumberWatermarkRowMissingFromTheAccountingTree`；可写挂载在任何写之前拒、报同一个失败，两块盘逐字节不变。**没有「回落」**，原因第二节最后一行。
- 新测试 4 条（集成 1 条重写、单测 2 条、core_review 1 条），各证过红；新变异 6 行全证过；上一位留给门禁的「建 inode 落在最左那片之外时不认水位变了」那一行连同另两行模型行，在新的判法下也重证过（第四节）。
- 整份补丁：fmt、`cargo clippy --all-targets --all-features`（加 `CODE_DISCIPLINE_LINTS` 七条）、`cargo build --all-targets`、E171 装置单编都过；四个包（core、checker、format、harness）183 个测试二进制全跑一遍（非 ignored），183 个都退 0：1829 过、0 败、67 ignored（第五节）。
- 推翻条件（任一条成立，这份结论就不对）：
  - 主工作区打上补丁后跑第七节第 2 条，有红；
  - 三方造出一段合法历史，接在一版后面写记账树的那次发布实际写的记账树节点数，多于 `accounting_tree_nodes_upper_bound(上一版的上界, 这次的行数)`；
  - 第二节「根切 r ≤ min(s + t, 1 + t)」「切出来的叶 s ≥ 1 时至少 s + 1 片」两条推理有反例。

## 二、我取的（条款没写到、派发让我挑的地方）

| 处 | 我取的 | 为什么 |
|---|---|---|
| 两种处置挑哪种 | 根之下的记账树节点不对应角色，占槽与映射条目按节点数上界带（同中央映射树） | ① 实现哪几片这次重写由 key 在哪片叶定：分裂之后台阶行排在统计量 12 那一段（标签 12 是最大的，`records.rs` 第 21–29 行），右边那片每建一个 inode 涨一行；到第 702 个 inode 前后右片再切一次，中间那片只装旧台阶、之后不再重写。模型要逐片比分配代，就得照抄 key 的排法与规划器的切法，D13 已定项 5 不许、`model.rs` 文件头「不记 key」也这么写着。② `model_compares_*` 里最长的历史建到 703 个 inode，正好跨第二次分裂，逐片记的那条路在这里最容易和实现分叉。③ 根之下节点的分配代、位置与记录本来就归池级 checker 判（I-1.1、I-3.10），与分配记录树、映射树根之下的节点同一处置 |
| 上界怎么算 | 新写 `accounting_tree_nodes_upper_bound(nodes_before, rows)`：N ≤ N_前 + s + t + r。s ≤ ⌊R ÷ 239⌋ − 1（s ≥ 1 时切出来的叶至少 s + 1 片、各至少 239 行）；t ≤ ⌊(N − 1) ÷ 75⌋ − 1，且 s = 0 时 t = 0；r ≤ min(s + t, 1 + t)（根兼叶切开那次是叶切，之后的根切都算在 t 里）。从 2 (N_前 + 2s + 1) 往下迭代到不动点 | 中央映射树那一份（`central_mapping_tree_nodes_upper_bound`）每切一次按 +2 算、只数右半：从 1 个节点长到 478 行给 5，与实现的 3 不等，「占几个槽相同」那条验收就落空。新式子只用 D8 已定项 11 的规则（先删后插、从中间切、只摘空节点），不看 key；478 行给 3、717 行给 4、151 × 239 行给 154，都与按规则手推的形状相同（单测钉了）。中央映射树那一份没动（钉值的用例在、不在这次的范围） |
| 上界的松紧 | 长层那一步恰好；之后每次发布至多 +2（不知道叶不满），节点数过 150 之后内部节点切那一项让它按约 2.7% 一次地涨 | 只用来带占槽与映射条目，占槽只决定单元区墙「允许拒绝的区间」，放宽不会让模型错判实现；见第八节第 1 条 |
| 上一版的上界取哪一版 | 写记账树的发布取 `previous`（回退那次是现行那一版，实现回退时从现行那一版的形状规划） | `next_root_with_the_user_visible_trees` 的 `previous` 已经是这么取的；不写记账树的发布照抄上一版的上界 |
| 模型接口 | 加 pub `IdealModel::current_version_in_the_session() -> Option<&ModelRoot>`，`ModelRoot` 多一段 pub 字段 | 对拍用例要按步看模型那一版的行数与节点数；`ModelRoot` 只在 `model.rs` 里构造（E171 不构造），已有的 pub 名字与签名一个没改 |
| 长层那一步「占几个槽」怎么比 | 实现：交回的记账树单元（根与根之下）里这次重写的那几个，按单元字节数 ÷ 16 KiB 数；模型：那一版节点数上界 × `ModelUnitRole::AccountingTree.span_in_slots()` | 两边各数各的；模型那个数就是 `next_root` 加进占槽上界的那一项，另有一条单测钉「它进了占槽上界」 |
| 第 2 条第一条用例 | 造得出来：记账叶原地重建（key 区间取首行、末行的 key，先断言原叶头就是这样），树表里记账树那一条指针的两条校验和跟着换，树表照原 helper 重建写回；种的根除 txg 外照抄 | 原地改写会让 txg 3 那条根的两条校验和对不上；择根择中种的那条，用不到它，文档注释写明了 |
| 新用例「回落」 | 造不出回落，改成钉「重建判坏 + 可写挂载在任何写之前拒、盘上不变」，名字写明 | 冷读（`recover` → `walk_to_file_in_the_unit_area_starting_at`）不读水位，照读 txg 4 的内容（试跑看到的）；那一判只在 `rebuild_version_in_the_unit_area_starting_at`（`recovery.rs:2902`），可写挂载与回退建上一版走它。可写挂载建不出所选那一版时退不退看 F2 乙：这个池的系统配置 journal tail 是 mkfs 写的 0（`make_filesystem.rs:645`；取号不改它是我读代码推的，没逐行核），「c_见证 = 0 不退」（`recovery.rs` 里 F2 那个函数），所以拒。要造回落得先让系统配置见证一个非 0 计数器（抬 F 之类），那是 F2 的用例，不是这一判的 |

## 三、写过的文件

这一轮我改的（在上一位的补丁上接着改）：

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-harness/src/model.rs` | 新 `accounting_tree_leaf_entry_capacity`、`accounting_tree_internal_child_capacity`、pub `accounting_tree_nodes_upper_bound`；`ModelRoot.accounting_tree_nodes_upper_bound`；`next_root_with_the_user_visible_trees` 里记账行、台阶、节点上界挪到算占槽之前，记账树按节点数占槽、按节点数进映射条目；`central_mapping_entries_other_than_the_allocation_record_tree` 多一个参数；删 `accounting_rows_exceed_one_node` 与 `answer_for_publishes` 里的 panic；pub `current_version_in_the_session`；两条新单测 |
| `crates/singlefs-harness/src/model_comparison.rs` | `AccountingTreeNodeBelowTheRoot(_) => return None` 与注释；发布错误映射那段注释里「两棵树恒只有一个节点」改成如实的说法 |
| `crates/singlefs-harness/tests/accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf.rs` | 文件头一句；第二条用例 `the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step` 重写（名字不变）：实现与模型逐步记行数、节点数、占槽，比第 462、463 个 inode 那两版，历史建到 465 个跑完不停；新 helper `accounting_tree_slots_written_by`、结构 `AccountingTreeOfAVersion` |
| `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs` | 文件头一句；`pool_after_the_first_file` 拆出 `pool_after_the_first_file_and_the_overwrites`；新 helper `root_whose_accounting_rows_are_rewritten`；`image_whose_chosen_root_has_the_checkpoint_txg` 让记账行代段跟着换；第 36 条那条用例的文档；新用例一条 |
| `crates/mutations.tsv`（只经补丁目录追加） | 追加 6 行（第四节），没改别人的行 |

`crates/singlefs-core/src/recovery.rs` 这一轮一个字没动（上一位的判法原样留着）；它仍在补丁里，是上一位的改动。
`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 一个字节没动；`inode_tree.rs`、I-9.4、I-9.12 没碰；kb 一个字没写。

合起来的补丁 33 份文件 = 上一位的 31 份 + `model_comparison.rs` + `core_review_tree_table_duplicates_and_slot_one_search.rs`。`git apply --stat patch/crates.patch`（在主工作区跑）原样：

```
 crates/singlefs-checker/src/image.rs               |   12 +
 crates/singlefs-checker/src/walk.rs                |  253 ++++++++++++++++
 crates/singlefs-checker-tier/src/bad_disk_input.rs |    3 
 .../src/crash_judge_checker.wgsl                   |   16 +
 .../src/crash_judge_checker_common.wgsl            |   21 +
 .../src/crash_judge_checker_scan.wgsl              |  244 +++++++++++++++-
 .../src/crash_judge_common.wgsl                    |   16 +
 .../src/crash_judge_dispatch.rs                    |   14 +
 .../singlefs-checker-tier/src/crash_judge_gpu.rs   |   11 -
 ...sh_enumeration_new_pool_file_creation_stream.rs |    5 
 ...gpu_agrees_with_the_cpu_judge_on_every_state.rs |   47 +++
 crates/singlefs-core/src/admission.rs              |  115 ++++++-
 crates/singlefs-core/src/lineage_tree.rs           |   34 +-
 crates/singlefs-core/src/mount.rs                  |   69 ++--
 crates/singlefs-core/src/records.rs                |   16 +
 crates/singlefs-core/src/recovery.rs               |   36 ++
 crates/singlefs-core/src/transaction.rs            |   98 ++++++
 crates/singlefs-format/src/lib.rs                  |   18 +
 crates/singlefs-harness/src/model.rs               |  271 ++++++++++++++++-
 crates/singlefs-harness/src/model_comparison.rs    |   12 -
 ..._inode_number_watermark_steps_cross_one_leaf.rs |  310 ++++++++++++++++++++
 .../tests/checker_known_bad_images.rs              |  288 ++++++++++++++++++-
 ...hecker_narrow_invariants_and_abandoned_roots.rs |    2 
 crates/singlefs-harness/tests/common/mod.rs        |  180 +++++++++++-
 ...ner_trees_grow_a_level_when_the_root_is_full.rs |    2 
 ...ew_tree_table_duplicates_and_slot_one_search.rs |  265 ++++++++++++++++-
 ...ber_watermark_pressed_back_reissues_a_number.rs |   60 +++-
 .../tests/overwrite_in_one_instance.rs             |    9 -
 ...s_the_deletion_state_of_every_walked_version.rs |    5 
 .../tests/rollback_by_a_forward_publish.rs         |  189 ++++++++++++
 ..._lineage_record_and_its_checkpoint_cost_term.rs |   81 +++++
 ...criteria_and_rollback_to_the_emptied_version.rs |    4 
 .../tests/writable_mount_of_a_formatted_pool.rs    |   18 +
 33 files changed, 2482 insertions(+), 242 deletions(-)
```

补丁对主工作区此刻 `git apply --check` 过；主工作区的 `crates/` 自我拷副本之后没变（`diff -rq` 现核，`crates/mutations.tsv` 逐字节相同）。

## 四、变异表与证红

`patch/mutations-append.tsv` 29 行 = 上一位的 23 行原样 + 我的 6 行；`patch/mutations-replacements.tsv` 9 行 = 上一位的 9 行原样。追加名在主表里一个都没有、替换名在主表里各恰好一行（对主表此刻那一份现核：主表自我拷副本之后没变）。

我追加的 6 行（全证过）：

1. 第49批 W3 再补 模型：写记账树的发布不按这次的行数带记账树节点数的上界（行数当 0，台阶跨叶那次仍一个节点）
2. 第49批 W3 再补 模型：记账树切叶次数的上界不减一（不到 478 行也算切过一片叶）
3. 第49批 W3 再补 模型：记账树长层时不算根切开之后多长的新根
4. 第49批 W3 再补 模型：写记账树的发布占槽只算根（根之下的节点不进占槽上界）
5. 第49批 W3 再补 胶水：记账树根之下的节点照旧 panic（模型不罩记账树分裂）
6. 第49批 W3 再补 恢复：重建一版判 inode 号水位那一行只看统计量（只剩台阶也照建）

另重证了上一位的三行模型行（它们点名的用例这次重写了，判法变了）：「数记账行不算 inode 号水位台阶行」「数记账行把这次自己那一行也当照抄的台阶」「建 inode 落在最左那片之外时不认水位变了」（最后一行上一位留给门禁，这次证过）。

证红都用 `scripts/mutation/prove-red.sh --copy <副本 b/> --memory 8G singlefs-harness <名…>`（`CARGO_BUILD_JOBS=16`、`capped.sh 16`），两趟各自先跑基线、基线不红。末行原样：

```
prove-red-a.txt：✓ 点名 3 条：跑了 3 条，跳过 0 条，跑的都抓到了
prove-red-b.txt：✓ 点名 6 条：跑了 6 条，跳过 0 条，跑的都抓到了
```

| 用例 | 改坏哪一行 | 哪条断言红（日志里的原样） |
|---|---|---|
| `accounting_tree_grows_…::the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step` | 第 1 行（`model.rs` 带上界时行数当 0），`prove-red-logs-a/001.log` | 测试文件第 282 行那句「模型：第 462、463 个 inode 那两版……都与实现相同」：`left: (… rows: 477, nodes: 1, slots_written: 1 }, … rows: 478, nodes: 1, slots_written: 1 })`，`right` 第二项 `rows: 478, nodes: 3, slots_written: 3` |
| 同上 | 第 5 行（胶水照旧 panic），`prove-red-logs-b/003.log` | 第 246 行「跑完」那句：`left: (NewFinding { signature: Panic { location: "crates/singlefs-harness/src/model_comparison.rs:89" } … position: Operation(463) …`，`right: (Completed, 466, 465)` |
| 同上 | 上一位的「不算台阶行」，`prove-red-logs-b/004.log` | 第 282 行那句：模型两版都是 `rows: 15, nodes: 1` |
| 同上 | 上一位的「多数一级」，`prove-red-logs-b/005.log` | 第 282 行那句：模型 `rows: 478, nodes: 3` / `rows: 479, nodes: 5`，早一步长层 |
| 同上 | 上一位的「最左那片之外不留台阶」，`prove-red-logs-b/006.log` | 第 282 行那句：模型两版都是 `rows: 15, nodes: 1`（建满第 233 个之后不再留台阶，行数退回 15） |
| `model::tests::accounting_tree_nodes_grow_to_three_at_478_rows_and_keep_the_previous_bound` | 第 2 行（切叶上界不减一），`prove-red-logs-b/001.log` | `model.rs:4408` 那句：`left: [1, 3, 4, 5, 78, 155, 5, 6]`、`right: [1, 1, 3, 4, 77, 154, 3, 5]` |
| 同上 | 第 3 行（不算根切），`prove-red-logs-b/002.log` | 同一句：`left: [1, 1, 2, 3, 76, 152, 3, 4]` |
| `model::tests::the_inode_creation_whose_watermark_step_crosses_one_accounting_leaf_counts_the_nodes_below_the_root_in_the_occupied_slots` | 第 4 行（占槽只算根），`prove-red-logs-a/002.log` | `model.rs:4463` 那句：`left: ((477, 1), (478, 3), 57006547)`、`right: ((477, 1), (478, 3), 57006549)`（少了根之下那 2 槽） |
| `core_review_…::a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step_is_judged_damaged_by_the_rebuild_and_refused_by_the_writable_mount` | 第 6 行（`recovery.rs` 退回只看统计量），`prove-red-logs-a/003.log` | 证红那一刻测试文件第 594 行（之后为 clippy 改了同一文件里它前面的几行，行号会挪）「从盘上重建 txg 4 那一版……」：`left: None`、`right: Some(Walk(InodeNumberWatermarkRowMissingFromTheAccountingTree))` |

改过的第 36 条那条用例（`a_chosen_root_whose_checkpoint_txg_is_the_largest_value_…`）没新加变异行：它改的是用例造盘的方法，不是新判法；它现在绿、改之前（上一位补丁原样）红，红绿两边都跑过（上一位第五节与本报告第五节）。
两条新单测的单条用时没单独量：harness 库单测整份（211 条，含这两条）debug 下 29.5 秒；集成那个二进制（两条用例，新那条两段各建 465 个 inode）26 秒（`runs-1`），都不到慢档的 60 秒，没标 ignore。

## 五、验证（原样）

全在副本 `b/`（主工作区此刻那一版整份拷来，`apply-writer-patch.py --root b/` 打上上一位的补丁，退 0）里跑，`capped.sh 16`，跑编出来的代码经 `run-with-memory-cap.sh 8G`。

打上一位补丁那一步（`apply-previous.log`）：

```
✓ 打上了：patch（补丁 有）
code-source-discipline：退出码 0，✓ code-source-discipline.sh：1 格里 1 格绿（mutation-tables），0 格本次未跑（无）
调度表可贴的一行：| patch | 已打（2026-10-09） | code-source-discipline 0 |
```

交回前那几段（各段退出码各写进自己的文件、最后逐段读；在最后一处源码改动之后跑）：

```
fmt exit=0                          （cargo fmt --all -- --check，输出 0 字节）
clippy exit=0                       （cargo clippy --offline --all-targets --all-features -- -D warnings 加 CODE_DISCIPLINE_LINTS 七条，现抄自 check.sh 第 72–80 行）
    Checking singlefs-checker-tier v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-batch49/w3c/b/crates/singlefs-checker-tier)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.26s
build exit=0                        （cargo build --offline --all-targets）
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.50s
e171 exit=0                         （cargo build --offline -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes）
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
```

clippy 第一趟（同一条命令）红在新用例文件的三处（两处 `shadow_unrelated`、一处多余的 `clone`），改了重跑，上面是第二趟。

harness 档四个包：`cargo build --all-targets` 那一趟的 JSON 里挑 singlefs-core、singlefs-checker、singlefs-format、singlefs-harness 的测试产物，183 个二进制，8 个一组并行、每个经 `run-with-memory-cap.sh 8G`、`capped.sh 2`、`timeout 200` 跑（日志 `runs-final/`）。每个二进制一个退出码：

```
    183 0
test result 行 183 行：passed=1829 failed=0 ignored=67
```

派发点名的几个二进制（同一趟）：

```
model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.06s
model_compares_inode_creations_into_the_third_inode_leaf_container: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.44s
model_compares_the_third_and_fourth_inode_leaf_containers: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.21s
core_review_tree_table_duplicates_and_slot_one_search: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.57s
accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.05s
singlefs_harness: test result: ok. 211 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 29.49s
```

测试源码静态判据（`scripts/crash/crash-case-check.py --only placement,file-names,one-scenario <副本 b/>`，退 0）：

```
  ✓ 崩溃枚举用例都住在 singlefs-checker-tier 包里、标了 #[ignore] 的都登记了（扫了 313 份测试源码，认出 101 条；按测试函数直接调 enumerate_layer0 一族、自己逐个造崩溃状态、或经同一文件里的函数传递地这样做认；经 tests/<子目录>/ 里的共用函数间接调的、宏展开出来的认不出）
  ✓ 查了 225 份测试文件的名字（harness 档与 checker 档的 tests/*.rs），没有按里程碑、步号、增补号、并行线号或欠账号起的
  ✓ harness 档 1447 条用例都是一条一个场景
  ✓ crash-case-check 判了 3 样（placement、file-names、one-scenario），0 样无对象可判
```

cargo 调用按单条命令数 10 次：编测试 1、跑动到的二进制 3（core_review 那份第一趟红在用例自己的期望、第二趟编不过、第三趟绿）、证红 2、clippy 2、`build --all-targets` 1、E171 单编 1。183 个二进制是直接执行 `build --all-targets` 编出的产物，不另起 cargo。

## 六、受影响的层 0 流与崩溃枚举用例

补丁动了 `crates/singlefs-checker/src/`（`image.rs`、`walk.rs`），都是更早两位的改动，这一轮一个字没动。池级 checker 判定集合的变化与推的影响照上一位报告第六节、再前一位报告第六节，原样有效：新立 I-9.30（只在最新根上判）、I-3.13 不判台阶行、谱系记录宽 16 → 24；`crash_enumeration_fixed_script_stream.rs` 的钉值推的不变，`crash_enumeration_new_pool_file_creation_stream.rs` 加了 I-9.30 的登记，GPU 对拍多一条流。

这一轮的改动不碰 checker，只改理想模型、胶水与两份 harness 用例。下面几样会不会跟着变（推的，都没跑）：

| 流 / 用例 / 装置 | 推的影响 | 推法 |
|---|---|---|
| 层 0 各条流与崩溃枚举用例 | 不变 | 它们不经理想模型判；模型只在记账树长层（两块盘第 463 级台阶）之后才与改之前不同：记账树一个节点的版本上界恒 1，占槽与映射条目与改之前逐项相同 |
| checker 档里用理想模型的（崩溃注入的模型目录、`crash_injection_*`） | 不变 | 同上：那几段历史建不到几百个 inode |
| E171 装置（`e171_three_oracle_collusion_after_the_oracle_fixes`） | 它把胶水 panic 记成「模型接不了这份观测」；记账树根之下的节点不再 panic，那一种来源没了。它的流要建到第 463 级台阶才碰得到，推的碰不到 | 装置一个字节没改，单编过（第五节）；名字与签名没改，它穷举的枚举没加成员 |
| 理想模型对拍（门禁 harness-model-differential-and-scenarios 的模型几格、随机历史） | 建到第 463 个 inode 起照常往下比、不再停 | 第五节 `model_compares_*` 三个二进制绿；随机历史一段建不到这么多 |

登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 不用新加行：这一轮没写崩溃枚举用例（`crash-case-check.py` 的 placement 过，第五节）。

## 七、主 agent 要跑的命令

工作目录：主工作区（`/home/fy5090/code/singlefs`）。线程照当时的上限（下面写 16）；内存上限照 `replay.sh` 的默认 8G 或当时给的。

1. 打补丁：`python3 scripts/gate/apply-writer-patch.py ~/.local/share/singlefs/work/m3-batch49/w3c/patch`，看最后两行（退 0、code-source-discipline 那一格绿）。我在一份主工作区此刻的整份副本上原样跑过一次，结局见第十一节。
2. harness 档四个包全跑：`bash scripts/hosts/run-with-memory-cap.sh 24G bash scripts/hosts/capped.sh 16 cargo test --offline -p singlefs-harness -p singlefs-core -p singlefs-checker -p singlefs-format --no-fail-fast`（我那边是 8 个一组并行跑二进制，最慢的 `checker_known_bad_images` 在 2 线程下 210 秒；整份串着跑放后台），看每个 `test result:` 行，预期全绿。红了用 `python3 scripts/gate/rerun-failed-tests.py <日志> -p <包>` 只重跑红的。
3. 慢档：`… cargo test --offline -p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored`（22 条慢档用例，三位实现员都没跑，其中一条钉回退水位 5）。
4. checker 档（提交时的六道重阶段）：上一位报告第七节第 5 条与再前一位第七节第 3 条原样（GPU 对拍、`--lib` 里 GPU 判器的钉值、快档；快档里另看 `crash_enumeration_tree_split_streams` 与 `crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short` 的钉值）。
5. 变异整表（checker-tier-crates-mutation-replay）：没证的是再前一位第四节标「留给门禁」的那几行（减去上一位证了的「谱系记录 24 字节」）与替换的 9 行里没证的 8 行；我追加的 6 行与上一位的三行模型行都证过。

## 八、停下交主 agent 的设计问题与没盖住的

1. **模型的占槽上界在长历史上早已大过整个单元区，单元区墙那一格在这些历史上判不出东西**（不是这一轮引入的，没改）。
   - 看到的数：第 463 个 inode 那次发布，模型一次的占槽上界是 57 006 549 槽（第四节证红日志 `prove-red-logs-a/002.log` 的 `right`），4 GiB 盘的单元区每盘 211 968 槽。
   - 推的来源：中央映射树那一份上界（`central_mapping_tree_nodes_upper_bound`）每次发布按上一版的上界往下带，内部节点切那一项 2 ⌊(N − 1) ÷ 72⌋ 让它每次约涨 2.8%，几百次发布之后成千万。这一轮新写的记账树那一份同样每次至多 +2，过 150 个节点之后按约 2.7% 涨。
   - 后果：模型对单元区墙只答「允许拒绝的区间」，区间上端撑开之后，实现在这种历史上拒了也判不出错。要收紧得按树另写（例如把「切出来的两半都数」那一条也用到映射树上，或让模型记叶数），超出这一轮的范围，交主 agent 定。
2. **冷读不判「这一版自己那一行水位在」**：`recover` 与只读挂载走 `walk_to_file_in_the_unit_area_starting_at`，那里只判条数（台阶行不算）不判水位那一行；只剩台阶的那一版冷读照读、可写挂载拒（第二节最后一行、新用例）。要不要让冷读也判，条款没写，交主 agent 定。
3. 上一位报告第八节第 3、4、5 条原样（模型认「水位变了」按发布种类认；`accounting_tree_nodes_after_the_publish_built_on` 在规划器拒时取这一版的节点数，没有用例；草案 ③ 与克隆头留给代码轮）。

## 九、kb 要跟着改的（我不写 kb）

- 上一位、再前一位报告第九节原样。这一轮没有新的。

## 十、没做什么

- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、GPU 对拍、QEMU、herd7、crates 变异整表、Verus）归提交时的主 agent；没提交。
- 中央映射树那一份上界没收紧（第八节第 1 条）；冷读要不要判水位那一行没动（第八节第 2 条）。
- `rollback_by_a_forward_publish` 的 22 条慢档用例没跑；变异整表没跑。
- `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 一个字节没动；harness 的 `model`、`model_comparison`、`memory_pool` 已有的公开名字与签名没改（只加了 pub 函数、pub 字段）；`MountError` 与 harness 的那几个穷举枚举没加成员；`inode_tree.rs`、I-9.4、I-9.12 没碰；kb 一个字没写。

## 十一、草稿目录与补丁核对

补丁在一份主工作区此刻的整份副本（`c/`）上原样打过一次（`apply-final-check.log`），退 0：

```
✓ 打上了：patch（补丁 有）
code-source-discipline：退出码 0，✓ code-source-discipline.sh：1 格里 1 格绿（mutation-tables），0 格本次未跑（无）
调度表可贴的一行：| patch | 已打（2026-10-09） | code-source-discipline 0 |
```

打完的 `c/crates` 与我的工作副本 `b/crates` 逐文件相同（`diff -rq` 无输出，`crates/mutations.tsv` 也相同）。

`~/.local/share/singlefs/work/m3-batch49/w3c/` 下留：`patch/`（`crates.patch`、`mutations-append.tsv`、`mutations-replacements.tsv`、`report.md`）、`report.md`、`runs-final/`（183 个二进制的日志与退出码）、`runs-1/`、`run-2.log`–`run-4.log`、`prove-red-a.txt`、`prove-red-b.txt` 与 `prove-red-logs-a/`、`prove-red-logs-b/`、`final-*.log` / `.exit` / `final-build.json`、`build-1.log`、`binaries-final.txt`、`apply-previous.log`、`apply-final-check.log`、`crash-case-check.log`、`patch-stat.txt`、`full.diff`、`model.diff`、`new-rows-append.tsv`、`run-pool.py`、`run-sel.sh`、`sel-1.txt`、`fmt-1.log`。
仓副本 `a/`（拷副本那一刻的 crates 与 litmus，出补丁用的基准）、`b/`（工作副本，连 51G 的编译目录）、`c/`（打补丁核对用）交回前都删了。
