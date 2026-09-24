# 报告：C511（回退到无文件那一版之后诞生代怎么接） 第 3 步——树 ID 水位取根环 max、`publish_first_file` 改看树表条数、拿掉回退拒绝（接手续做）

时刻一律 UTC（东京 JST = UTC+9）。副本 `/tmp/claude-1000/impl-watermark/repo/`，草稿 `/tmp/claude-1000/impl-watermark/drafts/`（接手之后的都在 `drafts/resume/`）。

交付：
- 补丁 `/tmp/claude-1000/impl-watermark/watermark.patch`（最终版 23:29 写出，15 个文件，只含 `crates/`；基准是主工作区 23:26:40 的 `crates/` 快照 `/tmp/claude-1000/impl-watermark/base-crates-3/`）。23:46:40 对主工作区现状 `git apply --check` 退出 0，那一刻主工作区 `crates/` 与 `base-crates-3/` 逐文件相同（`diff -rq … | wc -l` → `0`）。
- 新变异行 `/tmp/claude-1000/impl-watermark/mutations-append.tsv`（8 行，不在补丁里）。
- 别的会话那一行变异的改写建议 `drafts/resume/c378-row-with-the-new-field.tsv`（第七节第 8 条，**不接上它，补丁打进去之后门禁 33、59 会红**）。

## 一、现场核对与两次同步

1. **17:12 写出的 `watermark.patch` 不是上一位的最后一版。** 它 17:14:09、17:14:20 又改了一次：把「这一版八棵树的号」从 `PublishPlan` 的字段挪成 `publish_version_of_trees` 与 `PublishPlan::resolve` 的参数（主工作区 `e156` 按旧形构造 `PublishPlan`，不挪就编不过），`second_transaction_step_three_second_instance.rs`、`…_two_row_publish_admission.rs`、`…_two_accounting_node_full.rs` 三份测试随之退回基准。17:12 版 18 个文件、sha256 `822cb7c5cf18430cb4c411bdeefbdb62cb8aca126f05f88bf3fe3dead7298f97`，留在 `drafts/resume/watermark.patch.stale-written-1712UTC`；副本对开工快照的最终差是 15 个文件，按它重出的补丁从第 185 行起与 17:12 版不同。它 17:11 起跑的 `check.sh` 开跑早于这次改动，作废。
2. **第一次同步（22:14）。** 主工作区对开工快照差 `mount.rs`（4 处「甲」加一撇的旧名 →「戊」）、`model.rs`（1 处）、`…_two_row_publish_admission.rs`（1 处）、`mutations.tsv`、`e156`、新增 `e158`；重出的旧补丁对现状 `git apply --check` 报 `error: patch failed: crates/mutations.tsv:301`。做法：主工作区 `crates/` 快照成 `base-crates-2/`，`rsync -a --exclude target --exclude .git` 进副本；我改过的 15 份先存 `drafts/resume/saved-copy/`；主工作区没动过的 12 份原样放回，`mount.rs`、`model.rs` 用 `patch` 打到主工作区版本上（无 offset、无 fuzz，合并结果与存档只差那 5 处「戊」），`mutations.tsv` 按新规矩重做（第二节）。23:00 补了用例 E（第四节）。
3. **第二次同步（23:26）。** 23:26:13 再跑 `git apply --check`，报 `error: patch failed: crates/singlefs-checker/src/walk.rs:437`：别的会话 23:23:25 改了 7 份（`walk.rs` 把根记录那一段走读拆成 `walk_tree_table_and_central_mapping_root`，并新走「由记录施加出来的那一版」；`image.rs`、`mutations.tsv`、`checker_known_bad_images.rs`、`first_transaction_step_seven_layer0.rs`、`second_transaction_step_three_formatted_pool.rs`、`second_transaction_step_zero_layer0.rs`），22:56 还改过 6 份并新增 1 份测试。同样的做法：快照成 `base-crates-3/`，副本同步过去，15 份里主工作区没动过的 10 份放回，`checker_known_bad_images.rs`（offset 24）、`…_formatted_pool.rs`、`…_bad_disk_input.rs`（offset 36）用 `patch` 打上、改动与存档逐行相同；`walk.rs` 那一块冲突手工重放：同一处改法（映射树根的「实际引用它的树」取映射根指针的出生树）落到新函数 `walk_tree_table_and_central_mapping_root` 里（`walk.rs:449`，改的那一行 `walk.rs:469`），它现在同时管根环里的根与由记录施加出来的那一版，两处都是 86 字节的码 2 指针，读法同一条。之后门禁、`check.sh`、变异全部在这一版上重跑（第四到六节只列这一版的结果；第一次同步那一版的日志挪进了 `drafts/resume/gates-before-2326-resync/`、`drafts/resume/mutations/before-2326-resync/`）。

## 二、这一轮写过的文件

副本里（补丁就是这 15 份对 `base-crates-3/` 的差）：
- `crates/singlefs-core/src/transaction.rs`、`mount.rs`、`recovery.rs`、`mounted_read.rs`
- `crates/singlefs-checker/src/walk.rs`
- `crates/singlefs-harness/src/model.rs`、`model_comparison.rs`、`history.rs`
- `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`second_transaction_step_three_formatted_pool.rs`、`checker_known_bad_images.rs`、`first_transaction_step_five_publish.rs`、`second_transaction_supplement_three_bad_disk_input.rs`、`second_transaction_supplement_three_random_history.rs`
- `crates/mutations.tsv`：**只删不加**。删验收 3 点名的 5 行（主工作区现状里仍是第 70、165、166、167、178 行）；第 304 行原地改原文（`if version_to_build_on.tree_identifier_watermark != TREE_IDENTIFIER_WATERMARK_AT_MKFS {` → `if tree_table_entries != 0 {`，名字尾巴加「；C511 第 3 步起判的是树表条数」），它守的是一处我改了写法的代码，不改它锚点就腐化。

`mutations-append.tsv` 的 8 行，变异名原样：
1. `C511 第 3 步（D8 已定项 8 ②）：回退行那次发布的树 ID 水位沿回退到的那一版带（暖机根的 11），不取根环里的 max；推到 (1, 3) 离开根环之后 I-7.8 判红`
2. `C511 第 3 步（D8 已定项 8 ②）：回退行那次发布的树 ID 水位沿回退到的那一版带，不取根环里的 max；偏向回退的随机历史抽样判出 I-7.8（这一档的必红是抽样断言，随测试周期的种子基重验）`
3. `C511 第 3 步（D8 已定项 8 ②：号永不重发）：回退到树表 0 条的一版之后再发第一个文件版本，八棵树照 mkfs 的水位 11 重发 11..18，不从那一版带过来的水位起发`
4. `C511 第 3 步：checker 对中央映射树根的 I-1.3 退回写死树 ID 15，不按根记录里那条根指针的出生树判（回退之后重新发号的映射树判不了）`
5. `C511 第 3 步：publish_first_file 退回按「水位 = 11 ⇒ 树还没建」判，不看树表条数；回退到暖机根之后那一版水位 19、树表 0 条，第一个文件版本被拒`
6. `C511 第 3 步：从水位起连号发八棵树的号不先判装不装得下（盘上读来的水位离 u64::MAX 不到八个号时越界 panic，不交回错误成员）`
7. `C511 第 3 步：publish_first_file 读不出那一版的树表时当成 0 条接着写，不在写之前交回 TreeTableOfTheVersionToBuildOnUnreadable`
8. `C511 第 3 步（交主 agent 的那一格：根环里读不出的根怎么算）：本实例第一次发布的树 ID 水位只取读得出的根，不并上环里记录新根段带的水位；带文件版本的根那一槽读不出时退回暖机根的 11`

补丁本身，主工作区上 `git apply --stat` 原样：
```
 crates/mutations.tsv                               |    7 
 crates/singlefs-checker/src/walk.rs                |    8 
 crates/singlefs-core/src/mounted_read.rs           |   11 
 crates/singlefs-core/src/mount.rs                  |  103 +++-
 crates/singlefs-core/src/recovery.rs               |   83 +++
 crates/singlefs-core/src/transaction.rs            |  406 ++++++++++++-----
 crates/singlefs-harness/src/history.rs             |   11 
 crates/singlefs-harness/src/model_comparison.rs    |   12 
 crates/singlefs-harness/src/model.rs               |   20 -
 .../tests/checker_known_bad_images.rs              |   66 +++
 .../tests/first_transaction_step_five_publish.rs   |    9 
 .../tests/second_transaction_step_four_rollback.rs |  492 ++++++++++++++++++--
 ...second_transaction_step_three_formatted_pool.rs |  118 +++++
 ..._transaction_supplement_three_bad_disk_input.rs |   11 
 ..._transaction_supplement_three_random_history.rs |    1 
 15 files changed, 1122 insertions(+), 236 deletions(-)
```
副本里 `git diff --stat -- crates litmus` 对的是 HEAD，把别的会话没提交的改动一起算进去了（末行 `54 files changed, 11808 insertions(+), 1709 deletions(-)`），原样 55 行在 `drafts/resume/copy-git-diff-stat.txt`；分不出谁改的，以补丁那份为准。

## 三、验收 1–5 逐条

行号都是副本最终版现取的；补丁打上主工作区之后同号（这 15 份的其余部分与 `base-crates-3/` 相同）。

1. **做到。** 本实例第一次发布（写行那次）的树 ID 水位 = max(根环里全部自证过的根带的该字段, 环里全部自证通过的记录新根段带的该字段, 要接在后面的那一版自己带的)：`recovery.rs:455` `highest_tree_identifier_watermark_in_the_ring`（被抛弃时间线上的根也算，与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 取 max 的范围同）、`mount.rs:324` `tree_identifier_watermark_of_the_ring`。可写挂载在 `mount.rs:1311`、回退在 `mount.rs:1459` 各算一次，都在任何写之前，经 `InstanceStart` 交给写行那次发布（`mount.rs:929`，以及树表 0 条那一版的三条写行路径 `mount.rs:1186`、`1205`、`1227`），写行那次直接用这个数、不重读。之后本会话的发布照抄现行那一版的水位（它取过环里的 max，之后环里新写的根都是本会话自己写的）；`publish_first_file` 取 max(那一版的水位, 本次发出的最高号 + 1)（`transaction.rs:2130`–`2132`）。零单元发布与树表 0 条那一版的写行发布原先直接抄上一版根的水位，改成取计划里给的（`ZeroUnitPublishPlan` 与写行计划各加一个 `tree_identifier_watermark` 字段）。根环里读不出的根怎么算，单列在第七节第 1 条。
2. **做到。** `publish_first_file` 读要建在上面的那一版的树表单元、数条数（`transaction.rs:2071`–`2073`，`recovery.rs:709` `tree_table_entry_count`），非 0 才报 `FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries }`。随之多出两个走得到的错误成员，都在任何写之前返回：树表读不出 `TreeTableOfTheVersionToBuildOnUnreadable`（`transaction.rs:1712`），水位离 `u64::MAX` 不到八个号 `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`（`transaction.rs:1717`）。八棵树不再写死 11..18：第一个文件版本从那一版的水位起连号发（`transaction.rs:1195` `FileVersionTreeIdentifiers`、`1231` `issued_from_watermark`），之后每一版照抄；从盘上重建的版本按树表条目的种类与根记录里中央映射树根指针的出生树读回（`recovery.rs:808` 起，映射树在 `824`）。
3. **做到。** `mount.rs` 那道拒绝与 `MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion` 拿掉；`model.rs` 的拒绝理由与预测、`model_comparison.rs` 的映射、`history.rs` 的成员名、随机历史必见清单那一行、步 4 那条「写之前拒绝」的用例（换成第四节的 A、B）一起删。副本里 `grep -rn 'RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion' crates litmus | wc -l` → `0`。`crates/mutations.tsv` 那 5 行删掉，守新行为的变异在 `mutations-append.tsv`。还写着这个名字的 kb：`.claude/kb/checks-owed.md`（C511 那一行）、`.claude/kb/experiments/158-择根与修复四岔路.md`，我不写 kb。
4. **做到，有两处与字面有出入，单列第七节第 6、7 条。** 用例 A（`second_transaction_step_four_rollback.rs:254`）：写文件版本 (1, 3) → 回退到暖机根 (1, 2)（环里还有 (1, 3)）→ 回退不拒绝，回退行那次的根与记录新根段都带 19（回退之前环里的 max；暖机根自己带 11，`:307`），回退之后 checker 无违例、I-7.8 真被判过（`:311`）→ 再发第一个文件版本：八个号从 19 起、都高于此前最大的 18、互不相同，树表七条与映射树根头部都是这八个号，新水位 = 最大号 + 1（`:197` `assert_fresh_tree_identifiers`）→ 覆盖写一版照抄这八个号（`:339`）→ checker 无违例，I-7.8、I-9.14、I-9.10 都真被判过且成立 → 冷启动读回覆盖写的内容（`:354`）。判别力：把水位改回沿所基于的那一版带（变异 1），A 红在 `:307` 那条断言；I-7.8 红在用例 B（`:368`，回退之后推零单元发布直到 (1, 3) 离开根环）与偏回退的随机历史上（第四节 M1）。
5. **没有红。** 拿掉拒绝之后，I-9.10（对象出生代三处一致） 在合法历史上一处都没红：最终 `check.sh` 全量测试（第五节，394 过 0 失败）日志里 `Violated` 8 处都是坏盘输入那份测试打印的恢复失败原文（`I-1.1` 7 处、`E142 走读同款` 1 处，那份测试预期的）；另有故障注入那一段「说谎的设备丢掉一份内容之后盘面不一致：16 次（设备丢的，不算新发现）」下面的 `CheckerViolations { invariants: ["I-2.1", "I-4.8", "I-7.4"] }：16 次`——上一位在开工快照上跑的基线 `drafts/pristine-test.log` 第 904 行就有同一行，不是这次引入的。没有一处 I-9.10、I-9.14、I-7.8。变异基线四个二进制全绿；门禁 74 号五段模型对拍都没有对不上的；A、B 最后都点名要求 I-9.10 真被判过（不是「不适用」）且成立。没改判据，也没为它改 checker。

## 四、新测试与「改坏哪一行 → 哪条断言红」

新测试五条（副本最终版行号）：
- A `second_transaction_step_four_rollback.rs:254` `rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_carries_the_ring_watermark_and_the_next_first_file_issues_fresh_tree_identifiers`
- B `second_transaction_step_four_rollback.rs:368` `after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring`
- C `second_transaction_step_three_formatted_pool.rs:386` `first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write`（两个新错误成员各一格：返回该成员、录制流一步不多、分配器不动）
- D `checker_known_bad_images.rs:1884` `a_central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant`
- E `second_transaction_step_four_rollback.rs:487` `with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_from_its_journal_record`（接手之后补的，钉第七节第 1 条那个保守读法）

改了期望的旧测试：随机历史快档必见清单删一行；`second_transaction_supplement_three_bad_disk_input.rs` 那段写死的历史（拒绝拿掉之后第 4 步照常回退，盘面跟着变：槽号 50304 → 50368、恢复择到的根 (2, 9) → (3, 13)，钉的错误成员一个没变，`:344` 起的注释写明）；`checker_known_bad_images.rs` 的映射树坏镜像从 99 改成 16（水位之上的号会让 I-7.8 一起红，判别力算不到 I-1.3 一条头上）；`first_transaction_step_five_publish.rs` 跟 `TransactionUnit::tree` 的新签名。

做法：副本 `drafts/mutation-work/`（自己的 target；同步之后全部 `.rs` touch 过），每条改坏一处、跑点名的整个测试二进制，跑完从 `repo/` 拷回原件再 touch（`drafts/resume/run_mutation.py`）。23:28 起在第二次同步之后的版本上整批跑，日志 `drafts/resume/mutations/`。**基线红集为空**：未改动时 `checker_known_bad_images` 23 过、`second_transaction_step_four_rollback` 11 过、`second_transaction_step_three_formatted_pool` 11 过、`second_transaction_supplement_three_random_history` 18 过 2 ignored，`baseline.log` 末行 `exit=0`。被改的几处代码附近没有 `debug_assert`（`transaction.rs`、`mount.rs`、`recovery.rs`、`walk.rs` 里 `grep -n debug_assert` 零命中）。

| 变异 | 改坏哪一行 | 哪条断言红（原样） | 同时红了哪些测试 |
|---|---|---|---|
| M1 | `mount.rs:338` `ring.max(version_to_build_on.tree_identifier_watermark)` → `version_to_build_on.tree_identifier_watermark` | A：`second_transaction_step_four_rollback.rs:298:5` `回退行那次发布的根与记录新根段都带回退之前根环里的水位 max，不带暖机根自己的 11` `left: (11, 11)` `right: (19, 19)`；B：`:128:5` `回退之后推零单元发布到 (1, 3) 离开根环：池级 checker 一条违例都没有：[("I-7.8", Violated("根环水位最大 11，盘上出现过的最大树 ID 15"))]` | 随机历史 `rollback_heavy_…`（新发现 14）、`allocation_record_wall_sampling_…`（新发现 2）、`unit_area_wall_sampling_…`（新发现 1），新发现的签名只有一种：`6 新发现 CheckerViolations { invariants: ["I-7.8"] }` |
| M2 | `transaction.rs:2098`–`2100` `issued_from_watermark(version_to_build_on.tree_identifier_watermark)` → `issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)` | A、B 都红在 `:204:9` `新发的号 TreeIdentifier(11) 不低于回退带过来的水位 19、高于此前最大的 TreeIdentifier(18)` | 只有 A、B |
| M3 | `walk.rs:469` `parse_node_pointer(mapping_root_pointer).birth_tree,` → `15,` | D：`checker_known_bad_images.rs:1927:9` `只改了根记录里映射树根指针的出生树：判红的该只有 I-1.3：…` `left: []` `right: ["I-1.3"]` | A、B 红在 `:128:5` `[("I-1.3", Violated("中央映射树的根：头里的树 ID 23 不是引用它的树 15"))]`：写死 15 的 checker 在合法的回退历史上误红 |
| M4 | `transaction.rs:2073` `if tree_table_entries != 0 {` → `if version_to_build_on.tree_identifier_watermark != TREE_IDENTIFIER_WATERMARK_AT_MKFS {` | A、B 都红在 `:189:6` `回退到树表 0 条的一版之后再发第一个文件版本: FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: 0 }` | 只有 A、B |
| M5 | `transaction.rs:1234`–`1239` `.checked_add(…).ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {…})?` → `.wrapping_add(…)` | C 红，但红在被测代码里：`panicked at crates/singlefs-core/src/transaction.rs:1240:32: attempt to add with overflow`（变异后的行号）。加 `--release` 再跑一次红在同一处——工作区 `Cargo.toml` 的 `[profile.release]` 开着 `overflow-checks = true`。C 要的是「交回错误成员」，这条变异让它 panic，所以红；C 自己的 `matches!` 断言没走到 | 只有 C |
| M6（第 304 行那条旧变异，原文改到今天的写法） | `transaction.rs:2073` `if tree_table_entries != 0 {` → `if false {` | 随机历史快档 `second_transaction_supplement_three_random_history.rs:245:5` `「已知红」清单外的失败：…新发现 42` | `reuse_heavy_…`（`:274`，新发现 7）、`rollback_heavy_…`（`:311`，新发现 3）；签名只有 `6 新发现 ModelDisagreement { aspect: "模型说该拒、实现做成了" }` |
| M7 | `transaction.rs:2072` `.map_err(\|failure\| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;` → `.unwrap_or(0);` | C：`second_transaction_step_three_formatted_pool.rs:468:5` `那一版的树表两份都读不出：None`（发布做成了） | 只有 C |
| M8 | `recovery.rs:462`–`465` 记录那一半的 max → `let mut highest: Option<u64> = None;` | E：`second_transaction_step_four_rollback.rs:537:5` `根读不出、记录还在：水位按记录带的 19 算，不退回读得出的根带的 11` `left: Some(11)` `right: Some(19)` | 只有 E |

A 与 B 的分工：A 那段历史里 (1, 3) 一直在环里，I-7.8 取全环 max（19），M1 下按构造红不了，A 靠 `:307` 那条直接断言；I-7.8 的红由 B（同一个开头，再把 (1, 3) 轮出根环）与偏回退的随机历史给出。B 中途退出重开一次：上一位 16:23:40 在 B 的前身上看到同一个会话转满一圈根环之后 `I-3.1` 红（`盘 0：记账的已分配 Some(262144)，遍历全部有效根得到 229376`，交接摘要第四节原样），它判为收口表第 ② 行「一次挂载转过一整圈根环」那一族、与水位无关；这个判断我没在最终版上复现。

没有新加层 0 流或崩溃点重放用例。

## 五、`check.sh`

副本原样（主工作区 23:26 现状 + 我的改动）跑 `nice -n 19 bash .claude/scripts/check.sh`，23:41 开跑，**红在格式那一步，红的两份都不是我的文件**：别的会话正在写的 `e156_allocation_basis_counts.rs` 83 处、`e158_root_choice_repair.rs` 32 处 `Diff in`，我的 15 份一处没有。末尾原样：
```
     emit_result(&format!(
  ✗ 格式不合规
     → 怎么办： 跑 cargo fmt --all 让它自己改完，再重跑本脚本。
exit=1
Wed Sep 23 11:41:47 PM UTC 2026
```
这两份即使格式化了，clippy 也红在它们身上（22:17 那次变体日志 `drafts/resume/check-sh-variant.log`：e158 三条、e156 一条）。所以全程跑在一份变体上：`drafts/resume/check-variant/`，与副本只差这两份（e156 退回开工快照 `base-crates/` 里那一版、e158 拿掉；`diff -rq` 只列这两处）。23:27:58 开跑、23:41:31 跑完：`✓ 格式通过`、`✓ clippy 通过`、`✓ 构建通过`、`✓ 单测通过`，测试 394 过、0 失败、7 ignored（各 `test result` 行求和）。末尾原样：
```

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit=0
Wed Sep 23 11:41:31 PM UTC 2026
```

## 六、门禁（副本上，`SINGLEFS_GATE_FULL=1`、`nice -n 19`，都在第二次同步之后）

登记给实现员的七道（`stage-owners.tsv`），另加派发点名的 59（只跑相关行）与 54 快档：

| 阶段 | 时刻 | 原样判定行 | 退出码 |
|---|---|---|---|
| 33（副本原样） | 23:28 | `  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）：` / `      crates/mutations.tsv:337 C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）：原文在 crates/singlefs-core/src/mount.rs 里命中 0 次` | 1 |
| 33（另一份拷贝：副本表里那一行换成第七节第 8 条的改写、末尾接上 `mutations-append.tsv` 8 行） | 23:28 | `  ✓ 145 个实验二进制都有成形的变异表，1539 条变异的原文各命中源码一次；crates/mutations.tsv 358 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）` | 0 |
| 53 | 23:28 | `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））` | 0 |
| 74 | 23:29–23:30 | 首行 `  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 5 段）：`，末段 `      随机历史：小盘上逼近单元区墙的取样点：模型对拍 4628 步：该拒而拒 629、区间里拒 327、该成而成 3672；比过根 4262 条、分配记录 75396 条、冷启动内容 59 次、抬 F 上限 165 次；回退到 txg = F_生效 > 0 的根做成 0 次；分配记录墙按镜像上的真条数放行 0 次；单元区墙按区间放行 308 次` | 0 |
| 92 | 23:28 | 首行 `  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；…`，末行 `      第一条纯 SSD 布局线：.claude/kb/layout/02-second-txn.md` | 0 |
| 94 | 23:28 | 首行 `  ✓ checker 与实现只共享常量模块 `singlefs-format`（…）；checker 的 3 份源码零处引 `singlefs_core`；…`，末行 `    这一道判不了的：…门禁 59 号复跑` | 0 |
| 93 | 23:28 | `  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 1 位、登记表分出去 1 位；扫了 46 个 .rs，认出 2 处 feature bit 常量、解出位号 1 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:26，值 `INCOMPAT_FIRST_SSD_LINE_BIT` 不是位掩码字面量））` | 0 |
| 89 | 23:28 | `  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）` | 77（本次未跑，不是通过） |
| 59（另一份拷贝，表里只留 9 行：改过原文的第 304 行那条 + `mutations-append.tsv` 8 行；自己的 `GATE_MUTATION_TARGET_DIR`，4 个工作进程） | 23:31:05–23:32:08 | `  ✓ crates 变异表复跑：9 条变异各自红在点名的测试上（原文都恰好命中一次；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、与进程数无关）` | 0 |
| 54 快档（主工作区现在那一版脚本，副本当项目根） | 23:30:49–23:31:05 | `  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 8 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/impl-watermark/repo/.git/singlefs-layer0-full-green）` | 1 |

54 用主工作区那一版脚本，是因为副本里那一版（开工时拷的）还没分快档。快档用例全过，红只因为副本没跑过 `--full`、没有全绿标记；全量归主 agent 收尾或崩溃验证员。59 各行的判定原样在 `drafts/resume/gate-59-relevant-rows.log`。

补丁对主工作区现状：
```
Wed Sep 23 11:46:40 PM UTC 2026
0
git_apply_check_exit=0
```
（第二行是那一刻主工作区 `crates/` 与 `base-crates-3/` 之间不同的文件数。）

## 七、停下交主 agent 的问题

1. **根环里读不出的根怎么算（验收 1 点名交回的那一格）。** 条款只写「根环里全部根记录该字段的 max」。实现取最保守的读法：全部自证过的根（被抛弃时间线上的也算）∪ 环里全部自证通过的记录新根段带的水位（D23（journal 的角色与格式） 已定项 15）∪ 要接在后面的那一版自己带的（防两次读环之间的瞬时读错把水位拉低）——`recovery.rs:455`、`mount.rs:324`。⚠️ 并上记录这一半正是 C342（树 ID 水位在根读不出时退回去重发） 前置那一列还开着的问题（「是否像 CJ 那样也取记录里的最大值」），我替它先选了「取」。用例 E + 变异 M8 钉住：(1, 3) 那一槽持续读不出、它的记录还在时水位仍是 19。若定为不取：删 E、删 `mutations-append.tsv` 第 8 行、删 `recovery.rs:462`–`465` 那一半。根与它那条记录都读不出的那一格仍盖不住（号会重发），不在这次改动里、没测。
2. **从 mkfs 之外的水位发号时八棵树各拿哪个号，条款没写。** D8（核心索引结构） 已定项 8 ② 只给水位公式。实现照 `crates/singlefs-format/src/lib.rs` 第 123–130 行那组常量的次序（extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist；偏移由常量减 11 现算，`transaction.rs:1195` 起），树表条目按树 ID 升序的排序契约照旧成立（`transaction.rs:3196` 起那条断言）。
3. **中央映射树的号不再恒是 15，checker 的 I-1.3 读法跟着改了——这一条要你看。** 映射树不进树表；实现、只读挂载（`mounted_read.rs:328`）、恢复（`recovery.rs:824`、`1533`）与 checker（`walk.rs:449` 起的 `walk_tree_table_and_central_mapping_root`，改的是 `walk.rs:469`）都改成拿映射根指针头部的出生树当「实际引用它的树」，原先写死 `TREE_IDENTIFIER_CENTRAL_MAPPING`。这是改 checker 的判据来源，不是为了让 I-9.10 变绿；不改的后果见 M3：写死 15 的 checker 在 A、B 的合法历史上误红 `I-1.3`（`头里的树 ID 23 不是引用它的树 15`）。用例 D 钉「按指针判」。I-1.3（块头树 ID 一致） 那一行条文要不要写明这一句，归 kb。第二次同步时这一改落进了别的会话 23:23 刚拆出来的函数，它同时走「由记录施加出来的那一版」，那一支的映射根指针取自记录新根段，读法同一条，那一支我没另写用例。
4. **`publish_version` 没有上一版时的 `assert_eq!`（`transaction.rs:2311`–`2330`）。** 没有上一版就按 mkfs 那条流的水位 11 发号，并断言计划写的新水位是 19。走不到的理由：全仓 9 个调用点里带 `None` 的只有 `crates/singlefs-harness/tests/second_transaction_supplement_two_accounting_node_full.rs:69`，它的计划写的正是 `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH`；其余 8 个（`mount.rs:852`、`913`、`981`，`transaction.rs:2156`、`2271`，`second_transaction_supplement_two_row_publish_admission.rs:76`，`second_transaction_step_three_second_instance.rs:457`，`e156_allocation_basis_counts.rs:402`）都带 `Some`。从别的水位发号的第一个文件版本走 `publish_first_file` → `publish_version_of_trees`，不经过这一判。
5. **inode 号水位那一行记账的树维跟着新发的 inode 树号走**（`transaction.rs:2945`）。已定的「已发布的 (inode 号, 对象出生代) 不复用」由对象出生代换代保住（I-9.10 在 A、B 上判过且成立），我没另做判断，列出来供你核。
6. **验收 4「新建的树 ID 全部高于它」**：按 D8（核心索引结构） 已定项 8 ② 的形态，水位是「本次发出的最高树 ID + 1」即下一个可用号，从水位 19 起发的第一个号就是 19。用例断言的是「≥ 带过来的水位 19、> 此前发过的最大号 18、新水位 = 最大号 + 1」，没有照字面写成「全部 > 19」——那样要么跳号、要么与 ② 的 +1 对不上。
7. **验收 4「这条历史上 I-7.8 必须红」**：A 那段历史里 (1, 3) 一直在环里，I-7.8 取全环 max，M1 下按构造红不了；A 靠直接断言红，I-7.8 的红由 B 与偏回退的随机历史给出。这个分工接不接受，你定。
8. **与别的会话的交叉，不处理会红：主工作区 `crates/mutations.tsv` 第 342 行 `C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）`**（22:14 之后才有）的原文罩住 `mount_writable` 末尾整段 `InstanceStart { … }` 字面；补丁在那段里加了 `tree_identifier_watermark_of_the_ring,`，打上之后这一行原文命中 0 次（第六节 33 号原样），59 号预扫也会整张表退出。那是别人的行，我没改。改写建议 `drafts/resume/c378-row-with-the-new-field.tsv`：只在原文、替换文两段的 `next_counter,` 后面各插这一个字段。核过：接上它与 `mutations-append.tsv` 之后门禁 33 号绿（第六节第二行）；在「主工作区 23:19 现状 + 补丁」的拷贝上（C378 那条变异涉及的 `mount.rs` 与 `second_transaction_supplement_three_fault_injection.rs` 之后没再变过）基线 `second_transaction_supplement_three_fault_injection` 8 过 1 ignored，变异后点名的 `a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it` 红在 `second_transaction_supplement_three_fault_injection.rs:719:9` `取号之后第 3 次写报错：重开的盘上系统配置还是新号 2、不回卷成 1`（`drafts/resume/mutations/before-2326-resync/mutation-c378-row-with-the-new-field.log`）。
9. **文字里还指着旧状态的地方，都不在我的写范围**：`.claude/gate.d/74-model-differential.sh` 第 9 行「`crates/mutations.tsv` 第 146–178 行」（补丁删掉这个区间里的 4 行）；`.claude/kb/checks-owed.md` C511 那一行（还写着拒绝成员与它的用例）；`.claude/kb/experiments/158-择根与修复四岔路.md`；`.claude/kb/milestone/02-second-txn.md` 第 336 行收口表第 ④ 行。

## 八、负载

22:15 开编之前 `ps` 看到的是别的会话的 `cargo test --all`（pid 3419934）、`cargo test --release --bin e142-first-txn-dry-run`（pid 3420497、3420902）、`cargo test --offline -p singlefs-harness --test scratch_c366_damaged_own_record`（pid 3422458），没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`。我的每份拷贝各用自己的 target，没等锁。

## 九、没做什么

- 没走三方对抗；层 0 全量（54 `--full`）、QEMU、herd7 与 `crates/mutations.tsv` 整表复跑归崩溃验证员；门禁 59 只跑了相关的 9 行；没提交；没碰主工作区。
- 没写 kb（C511 那一行、I-1.3 条文、收口表第 ④ 行、C342），见第七节第 9 条。
- 全绿的 `check.sh` 是在变体上跑的（第五节）；副本原样那次红在别的会话两份文件的格式上。
- B 为什么要中途重开（同一会话转满一圈根环之后 I-3.1 红）是上一位在 B 的前身上看到的，我没在最终版上复现。
- 第七节第 1 条「根与记录都读不出」那一格、第 3 条「由记录施加出来的那一版」那一支的映射树判法，都没有专门的用例。
- 草稿都在 `/tmp/claude-1000/impl-watermark/drafts/`（接手之后的在 `resume/`：日志、变体、变异规格与日志、`saved-copy/`、`saved-copy-2/`；基准快照 `base-crates-2/`、`base-crates-3/` 在上一级），没入库：派发只让在副本里做、交补丁；变异日志要不要搬进 `research/results/` 由你定。
