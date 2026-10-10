# 第 49 批 W3 实现员报告（Y1 Opt2、Y2 C2aL 台阶行、谱系记录 24 字节、I-9.30）

日期：2026-10-09。副本：`~/.local/share/singlefs/work/m3-batch49/w3/repo`（从主工作区 HEAD 31e47115 加工作区改动拷，交回前删掉，见末节）。补丁目录：`~/.local/share/singlefs/work/m3-batch49/w3/patch/`。

## 一、结论

- 五条都落了：Opt2 回退取水位、台阶行、谱系记录 24 字节带 H_cur、池级 checker 新不变量 I-9.30（①②④，③ 没判，见第五节）、I-3.13 跳过台阶行；GPU 三个内核同改。
- harness 档：动到的测试二进制与四个包的库单测全绿（第四节原样）；新测试 8 条，都证过红（9 行变异抓到，第三节）。
- 补丁对主工作区此刻 `git apply --check` 过（主工作区 `crates/singlefs-checker/src/image.rs` 自拷副本之后改过第 212 行，不重叠）；变异表的追加名在主表里没有、替换名在主表里恰好一行（现核）。
- checker 档一条没跑：GPU 对拍、改了的 checker 档用例、`--all-features` 的 clippy（rocksdb 现编超 240 秒前台上限）交主 agent，命令在第七节。
- 推翻条件：主 agent 跑第七节的命令里有一条红，或三方攻方造出一段合法历史让 I-9.30 判红 / 一段重发让它不红而 I-9.29、I-9.6 也不红。

## 二、我取的（条款没写到的地方）

| 处 | 我取的 | 依据 |
|---|---|---|
| 台阶行的 key | 与每次发布写的那一行同统计量 12、树段 = inode 树的树 ID、设备段 = 0xFFFF_FFFF、代段 = 那次发布的 txg；值 = 那次之后的水位 | 「照同一个统计量现有行的写法」；`.claude/kb/decisions/08-核心索引结构.md` 已定项 6「读现行值 = 取该维度元组下代号最大的那一行」；三方第三轮代价探针 `research/prompts/m3-w3-r3-sonnet-model/probe/code_two_tree_probe.rs` 就是这样摆的（`key(12, INODE_TREE, NO_DEVICE, generation)`，「这一代的水位行：这一代水位变了就是台阶行」），D5 已定项 2 射程那组 462 / 463 / 36 074 的数出自它 |
| 台阶怎么认（不另存标记） | 代段早于这一版 txg 的行都是台阶；代段等于这一版 txg 的那一行值与代段最大的台阶不同（或没有更早的台阶）才是台阶。写路径（`TransactionOutput::inode_number_watermark_steps`）与 checker（`walk.rs` 的 `inode_number_watermark_steps_of_a_version`，按字段表另写一份）同一条认法 | 同上；水位只在「这次的值 ≠ 上一版的值」时留行，上一版的值恒等于它最后一级台阶的值（归纳） |
| 第一个文件那次算不算一级 | 算：(那次的 txg, 2)，就是它写的那一行，行数不多 | 第二轮攻方原型 `research/prompts/m3-w3-r2-opus-output.md` 第 134 行 trace 的 `rows_create=[(3, 2), …]`；不算的话 ④ 在「建过 inode 之前就回退」时 max 取空集 |
| 现行水位怎么读 | 那一统计量代段最大的那一行（`TransactionOutput::inode_number_watermark`） | D8 已定项 6 那一句 |
| 谱系记录三段次序 | T_rb 8 \| T_old 8 \| H_cur 8（新段接在已有两段之后，旧两段偏移不动） | 判决没定（D18 已定项 24 第 5 条「三段的逐字节次序判决没定」），**是我取的** |
| 回退那次的水位怎么交进发布路径 | `LineageStepOfThePublish::AdministratorRollback` 多带 `inode_number_watermark_after_the_rollback`；`version_to_publish_the_rollback_after` 不再改记账行，记账行（连台阶）原样取 cur 的 | 原写法把 cur 每一行水位都换成 max，有台阶行之后会把台阶也改掉；只换现行那一行又会在 max > H_cur 时造出假台阶。这样台阶从 cur 带、水位变了时回退那次自己留一级，两件都对 |
| H_cur 从哪读 | `current.inode_number_watermark()`（回退入口拿到的 cur），不从交给发布路径的「上一版」读 | D18 已定项 24 第 5 条原文 |
| ③ | 不判（第五节） | 同 key 维度之下单张镜像上恒成立 |
| ④ 的「H_cur 为 0 也判红」 | 不另写一个合取：台阶的水位至少是 1（产品路径从 2 起，压回开关下限 1），等式在 H_cur = 0 时不成立；单写一个 `!= 0` 改掉不红、做不成变异 | 第三节 ④ 那两行变异都抓到 H_cur = 0 那张坏镜像 |
| ④ 谱系读不出、接不成链 | 报射程里判不了（`part_of_the_range_not_judged`，理由沿用 `DeletionScanGap`） | 同删除态扫描「在谱系上」的前提；GPU 那边 `d_read_the_lineage_of_a_version` 接不成链也交回假，两边一样 |
| 新不变量读不到台阶 | 最新根下面没有走全了的记账树：报不适用，理由串 `walk::NO_INODE_NUMBER_WATERMARK_ROW_UNDER_THE_NEWEST_ROOT`；别的读不全报射程里判不了 | 照 I-9.6 那两句的报法 |
| 只判最新根 | 是 | 草案「最新根的时间线」；I-9.29 管跨版本 |
| 恢复（`recovery.rs`）| 记账行数 3 + 6 × 盘数不算台阶行；「水位那一行在」改判代段 = 根 txg 的那一行在 | 前者不改就重建不出有台阶的版本；后者不改时坏盘把现行那一行改坏、只剩台阶也照挂、水位取成一级台阶 |
| GPU 判不了的 | 新表 `D_INODE_NUMBER_WATERMARK_ROWS` 256 行，越过置判不了（原因码 63、64，没用过的号）；读不全不置位（CPU 报判不了） | 照现有「表越过上限判不了」的写法 |
| 克隆头 | 没做，`transaction.rs` 与 `walk.rs` 各一句 TODO 指 D5 已定项 2 射程 / D18 已定项 24 第 5 条 | 派发提示 |

## 三、写过的文件（29 份，加 crates/mutations.tsv）

清单内：`crates/singlefs-core/src/mount.rs`、`transaction.rs`、`lineage_tree.rs`、`records.rs`、`recovery.rs`，`crates/singlefs-format/src/lib.rs`，`crates/singlefs-checker/src/walk.rs`，`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_dispatch.rs`，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`rollback_lineage_record_and_its_checkpoint_cost_term.rs`、`rollback_by_a_forward_publish.rs`，`crates/mutations.tsv`。清单里 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 没改（钉值不变，第六节）。

清单外（逐个写为什么）：

| 文件 | 为什么 |
|---|---|
| `crates/singlefs-checker/src/image.rs` | 新不变量要登记进 `IMPLEMENTED_INVARIANTS`（84 → 85）与名字常量 |
| `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` | 覆盖表编译期断言要每条都覆盖（补下标 84）；`SCRATCH_WORDS` 跟草稿区末端 |
| `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | GPU 的 I-3.13 与谱系记录登记宽在走读内核里 |
| `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | GPU 恢复内核也判记账条目数 3 + 6 × 盘数，不排除台阶行就与 CPU 恢复分叉 |
| `crates/singlefs-checker-tier/src/bad_disk_input.rs` | 改「水位那一行」标签的坏盘输入取第一条标签 12 的行，有台阶之后取到的是台阶；改成取最后一条（现行那一行），原意不变 |
| `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` | 逐条钉评估状态数，新不变量要登记进「只在新根下面评估」那一类（推的，没跑） |
| `crates/singlefs-harness/src/model.rs` | 只改三处注释 / 消息串里的 2039 → 1359（格式常量派生形态） |
| `crates/singlefs-harness/tests/common/mod.rs` | 新加「藏点名的打包单元」的崩溃恢复抛弃与 W3 那段历史的搭建（原函数拆出一个本体，行为不变） |
| `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` | 三张逐条不适用表加 I-9.30、条数注释 84 → 85、67 → 68 |
| `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` | 三条压回用例 I-9.30 也红（①/②），期望加它；第三条名字里的 `reddens_only_the_one_birth` 不再成立，改名 |
| `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` | 墓碑号 3 → 2 那张坏镜像 I-9.30 ② 也红，期望加它 |
| `crates/singlefs-harness/tests/overwrite_in_one_instance.rs` | 第二版记账行 15 → 16（多 A 的一级台阶）；水位按代段最大那行读 |
| `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` | 统计量现行值按代段最大那行读 |
| `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs` | 手改水位行时改最后那一行（现行值） |
| `crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` | `expect("1..=2039")` → 1359 |

`git diff --no-index --stat`（副本没有 .git：拷副本时留的原样 `base/` 对改后的 `repo/`，`crates/mutations.tsv` 不算；主工作区别的会话也在改 crates，`git diff --stat -- crates litmus` 分不出谁改的，这一份就是这次的全部）：

```
 crates/singlefs-checker/src/image.rs      |  12 +-
 crates/singlefs-checker/src/walk.rs       | 253 +++++++++++++++++-
 .../singlefs-checker-tier/src/bad_disk_input.rs    |   3 +-
 .../src/crash_judge_checker.wgsl                   |  16 +-
 .../src/crash_judge_checker_common.wgsl            |  21 +-
 .../src/crash_judge_checker_scan.wgsl              | 244 ++++++++++++++++-
 .../src/crash_judge_common.wgsl                    |  16 +-
 .../src/crash_judge_dispatch.rs                    |  14 +-
 .../singlefs-checker-tier/src/crash_judge_gpu.rs   |  11 +-
 ...sh_enumeration_new_pool_file_creation_stream.rs |   5 +-
 ...gpu_agrees_with_the_cpu_judge_on_every_state.rs |  47 +++-
 crates/singlefs-core/src/lineage_tree.rs  |  34 ++-
 crates/singlefs-core/src/mount.rs         |  69 ++---
 crates/singlefs-core/src/records.rs       |  16 +-
 crates/singlefs-core/src/recovery.rs      |  36 ++-
 crates/singlefs-core/src/transaction.rs   |  98 ++++++-
 crates/singlefs-format/src/lib.rs         |  18 +-
 crates/singlefs-harness/src/model.rs      |   6 +-
 .../tests/checker_known_bad_images.rs              | 288 ++++++++++++++++++++-
 ...hecker_narrow_invariants_and_abandoned_roots.rs |   2 +
 .../crates/singlefs-harness/tests/common/mod.rs    | 180 ++++++++++++-
 ...ner_trees_grow_a_level_when_the_root_is_full.rs |   2 +-
 ...ber_watermark_pressed_back_reissues_a_number.rs |  60 ++++-
 .../tests/overwrite_in_one_instance.rs             |   9 +-
 ...s_the_deletion_state_of_every_walked_version.rs |   5 +-
 .../tests/rollback_by_a_forward_publish.rs         | 189 +++++++++++++-
 ..._lineage_record_and_its_checkpoint_cost_term.rs |  73 +++++-
 ...criteria_and_rollback_to_the_emptied_version.rs |   4 +-
 .../tests/writable_mount_of_a_formatted_pool.rs    |  18 +-
 29 files changed, 1575 insertions(+), 174 deletions(-)
```

## 四、变异表与证红

`crates/mutations.tsv` 追加 18 行（`patch/mutations-append.tsv`），名字：

1. 第49批 W3 I-9.30 ④：谱系记录按「被台阶盖住」比（T_rb − 1 以前最大的台阶 ≥ H_cur），放过 H_cur 偏小与写 0 —— **证过**
2. 第49批 W3 I-9.30 ④（H_cur 为 0）：谱系记录按「被台阶盖住」比，H_cur 写 0 也放过 —— 同一处改法，第 1 行那次证红的日志里这条用例同样红（见下），行留给门禁复跑
3. 第49批 W3 I-9.30 ④：比的台阶取到代段 ≤ T_rb（多算回退那一版自己写的那一行），台阶少一级时被它补上 —— **证过**
4. 第49批 W3 I-9.30 ①：水位不小于每一级台阶那一判恒成立 —— **证过**
5. 第49批 W3 I-3.13：inode 号水位的台阶行不跳过、照判代段等于这一版的 txg（D5 已定项 2 射程的例外撤掉） —— **证过**
6. 第49批 W3 I-9.30 ②：记录与谱系上的墓碑晚于台阶出生那一判恒不找到 —— **证过**
7. 第49批 W3 Y1 Opt2：管理员回退取 inode 号水位时环里读得出的根全都算（被崩溃恢复抛弃的也算，回退跳到被抛弃那一版的水位） —— **证过**
8. 第49批 W3 Y2 C2aL：发布不照抄上一版的 inode 号水位台阶行 —— **证过**
9. 第49批 W3 Y2 C2aL：每一版自己那一行都当台阶照抄（水位没变的发布也留一级） —— 留给门禁
10. 第49批 W3 Y2 C2aL：管理员回退那次的记账行（连台阶行）取 R_old 的，不从 cur 带 —— **证过**
11. 第49批 W3 恢复：记账行数 3 + 6 × 盘数把 inode 号水位的台阶行也算进去（有台阶的版本从盘上重建不出） —— 留给门禁
12. 第49批 W3 谱系记录 H_cur：取回退目标 R_old 的水位，不取 cur 的 —— **证过**
13. 第49批 W3 谱系记录 24 字节：H_cur 那一段写成 0（`-p singlefs-core --lib`，用例 `lineage_tree::tests::a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from`） —— 留给门禁（cargo 调用到了 10 次的上限，没再起一趟 `-p singlefs-core` 的证红；第五节）
14. 第49批 W3 格式：谱系容器容量照旧写 2039（…应是 1359）（`-p singlefs-format --lib`） —— 留给门禁
15–18. 第49批 W3 GPU 判器：扫描内核 I-9.30 ④ 不判 / 扫描内核读谱系记录的 H_cur 读成 T_old 那一段 / 走读内核 I-3.13 不跳过台阶行 / 恢复内核的记账条目数把台阶行也算进去 —— checker 档（`--features verdict-store,gpu`，点名新加的对拍 `the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from`），不证，留给提交时的 checker-tier-crates-mutation-replay

`patch/mutations-replacements.tsv` 整行替换 8 行（锚点随这次改动腐化或点名的用例改名，名字不变）：第 5 批 GPU 判器 I-3.13 记账行代段不比；第 5 批 GPU 判器派活草稿区（`SCRATCH_WORDS` 316_912）；回退那次发布不追加谱系记录（两行：2451、2503 那两行，原文多了一行 `inode_number_watermark_after_the_rollback,`，替换文把它 `let _` 掉）；第 10 批 10b GPU 判器谱系登记宽（原文 24u，替换文照旧 17u）；第22批 C I-1.7 谱系记录宽登记成 15（原文 `Some(24)`，必须红的用例跟着改名）；第10q批 扫描内核单调用点（调用多一个参数）；第22批 C I-9.29（必须红的用例改名）。改完整表 3508 行锚点逐行恰好命中一次（`tools/anchor-check.py` 输出 `checked=3508 bad=0`）。

证红（`research/scripts/prove-red.sh --copy <副本> --memory 16G singlefs-harness …`，`CARGO_BUILD_JOBS=16`，各自先跑基线、基线不红）。第一趟 `prove-red-1.txt` 末行与第二趟 `prove-red-2.txt` 末行原样：

```
✓ 点名 4 条：跑了 4 条，跳过 0 条，跑的都抓到了
✓ 点名 5 条：跑了 5 条，跳过 0 条，跑的都抓到了
```

| 新测试（或改了断言的） | 改坏哪一行 | 哪条断言红（日志里的 left / right） |
|---|---|---|
| `checker_known_bad_images::a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant` | `walk.rs` `largest_step_before_the_rollback == Some(…)` → `>=` | 只红 I-9.30 那句：`left: []`、`right: ["I-9.30"]`；同一次 `…_is_zero_…` 也红，同样 `left: []` |
| `…an_inode_number_watermark_step_dropped_from_the_accounting_tree_…` | `walk.rs` `*step_txg < record.rollback_txg` → `<=` | `left: []`、`right: ["I-9.30"]` |
| `…an_inode_number_watermark_step_raised_above_the_watermark_…` | `walk.rs` ① 那一判写成 `true` | `left: []`、`right: ["I-9.30"]` |
| `…a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants`（合法镜像） | `walk.rs` 台阶行不跳过（`if false && …`） | 一条违例都没有那句：`left: ["I-3.13"]`、`right: []` |
| `inode_number_watermark_pressed_back_reissues_a_number::pressing_the_watermark_back_…_and_the_watermark_step_invariants` | `walk.rs` ② 的 `.find(… && false)` | `left: ["I-9.29"]`、`right: ["I-9.29", "I-9.30"]` |
| `rollback_by_a_forward_publish::a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root` | `mount.rs` 回退取水位传 `&ring_roots`（不滤被抛弃的） | 回退那次的水位那句：`left: 5`、`right: 3`（rollback_by_a_forward_publish.rs:2177） |
| `rollback_by_a_forward_publish::inode_number_watermark_steps_are_left_by_…_and_by_a_rollback` | `transaction.rs` 不照抄台阶；另一次 `mount.rs` 回退的记账行取 R_old 的 | 行那句（:2253）：前者 `left: [[(3, 2)], [(4, 4)], [(5, 4)], [(6, 5)], [(7, 5)]]`；后者 E 那一版 `[(3, 2), (4, 4), (7, 5)]`（丢了 (6, 5)） |
| `rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target` | `mount.rs` H_cur 取 `target_version` 的 | :268 那句：`inode_number_watermark_of_the_version_rolled_back_from: 4`，期望 5 |

没单证的新测试：`lineage_tree::tests::a_lineage_record_is_twenty_four_bytes_…`（第 13 行）、checker 档的新对拍（第 15–18 行）。cur 内存那一项（max 里不能去）沿用已有的「实三（D23 已定项 14「水位」）」那一行与 `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的用例，没动它的锚点（`.map_or(current.inode_number_watermark(), |ring| { ring.max(current.inode_number_watermark()) })` 原样留着），这次那个二进制跑绿。

## 五、验证（原样）

harness 档测试（经 `run-with-memory-cap.sh 24G` 与 `capped.sh 16`）。第一趟（`test-1.log`）在 `pool_checker_judges…` 那个二进制跑到一半撞 235 秒的 timeout（退 124），之前各二进制的结果行：
```
Running unittests src/lib.rs	test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.72s
Running unittests src/lib.rs	test result: ok. 215 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
Running unittests src/lib.rs	test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs	test result: ok. 209 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 26.92s
Running tests/accounting_tree_node_full.rs	test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.25s
Running tests/checker_known_bad_images.rs	test result: FAILED. 212 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.61s
Running tests/checker_narrow_invariants_and_abandoned_roots.rs	test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.29s
Running tests/container_trees_grow_a_level_when_the_root_is_full.rs	test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.63s
Running tests/inode_number_watermark_pressed_back_reissues_a_number.rs	test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.44s
Running tests/new_pool_file_creation_publish.rs	test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
Running tests/overwrite_in_one_instance.rs	test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.23s
Running tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs	
```

那一条红是 `an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant`：记账树条目宽被改坏、走读停下，I-9.30 跟着转成不适用（`left` 多一个 `"I-9.30"`），合乎新判法，期望加上它。第二趟（`test-2.log`，改完之后，退 0）：

```
Running tests/checker_known_bad_images.rs	test result: ok. 213 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.77s
Running tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs	test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.21s
Running tests/rollback_by_a_forward_publish.rs	test result: ok. 3 passed; 0 failed; 22 ignored; 0 measured; 0 filtered out; finished in 3.41s
Running tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs	test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
Running tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs	test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.72s
Running tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs	test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.42s
Running tests/writable_mount_of_a_formatted_pool.rs	test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.84s
```

交回前那一条（fmt、clippy、build、E171 装置、gpu 特性 clippy 串在一条里，各段退出码各自落文件）：

```
fmt exit=0
clippy exit=0
    Checking singlefs-checker-tier v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-batch49/w3/repo/crates/singlefs-checker-tier)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.62s
build exit=0
   Compiling singlefs-checker-tier v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-batch49/w3/repo/crates/singlefs-checker-tier)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 57.00s
e171 exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
clippy-gpu exit=0
    Checking singlefs-checker-tier v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-batch49/w3/repo/crates/singlefs-checker-tier)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.73s
```

clippy 带的是 `-D warnings` 加 `check.sh` 里 `CODE_DISCIPLINE_LINTS` 那七条（现抄）。`--all-features`（含 `verdict-store` 的 rocksdb 现编）没跑：副本自己的 target 里 rocksdb 是冷的，前台 240 秒放不下；gpu 特性那一趟 `-p singlefs-checker-tier --all-targets --features gpu` 过了，但 GPU 对拍那份测试文件要 `verdict-store,gpu` 两个特性，没编到（第七节）。最后那条验证之后又改了两处串（`recovery.rs` 一句报错串「记录宽 16」→ 24、`checker_known_bad_images.rs` 一句文档注释「16 → 15」→ 24 → 23），之后 `cargo fmt --all -- --check` 退 0，没再编（cargo 调用 10 次到顶：naga 草稿工具 1、clippy 4、cargo test 2、证红 2、交回前那一条 1）。
GPU 三个内核我用草稿工具（`naga =30.0.1`，`naga-probe/src/main.rs`，照 `crash_judge_dispatch.rs::kernel_inlined_size` 与 `crash_judge_gpu.rs` 的 `parse_and_validate` 同一个数法）解析、校验过，旧源码量回旧钉值逐个相等（331792 / 346138 / 352195），新源码：

```
VALID recovery
KERNEL recovery expressions=332638 margin=67362
VALID checker-walk
KERNEL checker-walk expressions=346307 margin=53693
VALID checker-scan
KERNEL checker-scan expressions=366870 margin=33130
```

`crash_judge_dispatch.rs` 的 `each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget` 钉值照这三个数改了。

## 六、受影响的层 0 流与崩溃枚举用例

池级 checker 改了三处判定集合：新立 I-9.30（只在最新根上判）；I-3.13 不判台阶行；谱系记录宽 16 → 24（I-1.7 的登记宽）。

| 流 / 用例 | 推的影响 | 推法 |
|---|---|---|
| `crash_enumeration_fixed_script_stream.rs` 全量与快档（A 到 E 再卸载） | 段数、写数、态数与第 133–134 行一带的钉值（`FULL_STATES_*`、`QUICK_TIER_*`、谱系槽 50352 / 50354）**都不变** | 这条脚本只有 A（第一个文件）留一级台阶 (3, 2)；之后都是覆盖写、回退、抬 F、卸载，水位不变，每次发布的记账行 15 → 16（多 A 那一级），一片叶装 477 行，记账树仍一个节点，写的单元数不变；谱系记录 24 字节仍落一片容器（1359 条一片），单元数不变。变的只是单元内容（记账叶多一行、谱系容器记录宽 24、H_cur = C 的水位 2）。每个状态的 I-9.30：最新根带记账树的成立（D 那一版 T_rb 9 以前最大台阶 2 = H_cur 2），没有的不适用；第 585 行那张「判违例的状态数」照旧全 0（推的，没跑） |
| `crash_enumeration_new_pool_file_creation_stream.rs` | I-9.30 进「只在新根下面评估」那一类，评估过的状态数 = `root_persisted_states`，违例 0 | 改了这一份的登记（推的，没跑） |
| `crash_enumeration_record_spill_over_stream.rs`、`crash_segments_per_device_and_torn_in_place_overwrites.rs`、`crash_enumeration_writable_mount_of_a_formatted_pool.rs` | 「每条不变量违例 0」「评估过 + 不适用 = 状态数」照旧成立 | 这几条流上没有建过第二批 inode 再回退的发布；I-9.30 在合法历史上恒成立；读不全的报判不了（不进违例） |
| GPU 对拍 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 每条流逐位比；新加一条 H_cur 改成 3 的回退流，CPU 与 GPU 都要红 I-9.30；谱系记录宽那一条的断言 16 → 24 | GPU 扫描内核在墓碑与谱系那一遍走到最新根时判 ② ④，收行那一步判 ①；GPU 恢复内核的记账条目数排除台阶行 |
| 崩溃放量 | 判法摘要（`gpu_judge_digest`）跟着内核源码变，旧判定行当历史、新行另起 | `crash_judge_gpu.rs` 文件头 |

登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 不用新加行：这次没写新的崩溃枚举用例。

## 七、主 agent 要跑的命令

工作目录都是打上补丁之后的主工作区（或它的暂存树）；线程照当时的上限，下面写 16。

1. harness 档全量（这次的台阶行改了所有「建过 inode 之后再发布」的版本的记账行数，我只跑了动到的二进制，没跑到的二进制可能还有钉 15 行或找「第一条标签 12」的地方）：`bash research/scripts/run-with-memory-cap.sh 24G bash research/scripts/capped.sh 16 cargo test --offline -p singlefs-harness -p singlefs-core -p singlefs-checker -p singlefs-format --no-fail-fast`，看每个 `test result:` 行；再 `… -p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored`（那份 22 条慢档用例我没跑，其中 `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write` 钉回退水位 5）。红了用 `python3 research/scripts/rerun-failed-tests.py <日志> -p singlefs-harness` 只重跑红的。
2. clippy 全特性：`BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include" bash research/scripts/capped.sh 16 cargo clippy --offline --all-targets --all-features -- -D warnings <CODE_DISCIPLINE_LINTS 七条>`：GPU 对拍那份测试文件（我加了一条、改了一句断言）只有这一趟编得到。
3. checker 档（提交时的六道重阶段里）：GPU 对拍 `SINGLEFS_HEAVY_TESTS=commit … cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state`（经 `research/scripts/gpu-quiesce-run.sh`），重点看 `the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from` 与 `…_lineage_container_of_a_rollback_publish_carries_another_record_width`；`--lib` 里 `crash_judge_gpu::tests::*`（三个内核解析校验、`SCRATCH_WORDS`）与 `crash_judge_dispatch::tests::each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget`、`each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel`（调用点钉值）；快档 `cargo test --release -p singlefs-checker-tier --lib --tests`（`crash_enumeration_new_pool_file_creation_stream`、`crash_enumeration_fixed_script_stream` 的钉值，第六节推的是都不变 / 加一条登记）。
4. 变异整表（checker-tier-crates-mutation-replay）：追加的 18 行里没证的 9 行（第四节标「留给门禁」的）与替换的 8 行。
5. 层 0 快档与全量照第六节对着看。

## 八、停下交主 agent 的设计问题与没盖住的

1. **草案 ③ 在这套 key 之下判不出任何东西**：台阶行与每次发布写的那一行同统计量、同树段、同设备段，只差代段（第二节第一行，照三方代价探针的摆法）。单张镜像上「这一版那一行是不是台阶」只能从值认，于是「水位大于初值时有一级台阶 H ≥ 水位、g ≤ 最新根 txg」恒成立，写进去也做不成会红的变异，我没判它。要它有牙，得让台阶行与那一行分开 key（例如设备段另取一个保留值、每级台阶多一行 34 字节——那样 D5 已定项 2 射程「第 462 行」那组代价要重算），这是格式与条款的事，交主 agent / 三方定。
2. **ckpt_cost 的记账树一项在 462 级台阶边上少算**：`admission.rs` 拿上一版的记账树节点数当一次空发布要重写的节点数；上一版那一行若是台阶，下一次发布行数多一行，正好跨过一片叶的容量时节点数会比上一版多。D5 已定项 2 射程写了「第 463 行起 +2」，但 D28 那一项的算法没改。没动 `admission.rs`（不在清单里，也没有条款说该怎么改），交主 agent。
3. `crates/singlefs-harness/src/model.rs` 的 `accounting_rows_exceed_one_node` 只算 3 + 6 × 盘数，不算台阶：台阶到 462 级以上时模型与实现的「记账树长不长一层」会分叉。模型罩的历史够不到这么多，没改，记在这里。
4. `recovery.rs` 改判「代段 = 根 txg 的那一行水位在」那一句没有 harness 用例会红（要一张「现行那一行改坏、只剩台阶」的盘，checker 档 `bad_disk_input.rs` 那一族才造得出，我把那一族改成改最后一行之后它本来就测不到只剩台阶的那一形），没写变异行。
5. 谱系记录三段次序是我取的（第二节），判决没定。
6. 克隆头（台阶按 (统计量, 树) 分、建克隆那次留行、I-9.30 按头）没做，代码里两处 TODO。

## 九、kb 要跟着改的（我不写 kb）

- `.claude/kb/layout/03-third-txn.md`：第 15 行谱系记录「16 字节」、第 18 行谱系容器「记录宽 = 16」「2039 条」、第 24–25 行 `<!-- format-const: LINEAGE_RECORD_BYTES = 16 -->`、`<!-- format-const: LINEAGE_CONTAINER_RECORDS = 2039 -->`（门禁 code-source-discipline 的 format-constants 格会在补丁打上之后红，直到这两行改成 24 / 1359），「三、谱系记录」那一节的字段表加 H_cur 一行（偏移 16、宽 8）。
- `.claude/kb/decisions/18-块里携带什么信息.md` 第 286 行「≤ 2039」「宽 16」。
- 格式常量改完要登记 `stale=`（`.claude/rules/format-evolution.md` 第 2 步）：候选串「记录宽 = 16」「谱系记录 16」——登记前要现扫 kb 正文与 `crates/`、`research/` 下的 `.rs` 命中 0 次；`2039` 是裸数字，不登记。`crates/` 里我改完之后现扫：`grep -rnP '2039|谱系记录 16|记录宽 16|宽 16[，、）)]|登记 16|Some\(16\)|16 字节谱系' crates` 只剩三处「fsid（偏移 102，宽 16）」，与谱系无关。
- `.claude/kb/invariants.md`：加 I-9.30 一行（文字取 `crates/singlefs-checker/src/image.rs` 那个常量的文档注释），I-3.13 一行补「inode 号水位的台阶行不判」，第 12 行条数 84 → 85、第 16 行 GPU 覆盖下标到 84。
- `research/` 与 `.claude/` 里点名旧用例名 `a_lineage_container_whose_record_width_is_not_sixteen_…`、`pressing_the_watermark_back_…_reddens_only_the_one_birth_per_number_invariant` 的地方：`crates/` 里的跟着改了（`crates/mutations.tsv` 两行已替换）；别处现查只在 `research/prompts/` 下的冻结证据里（`_m3-batch22-code-r1-diff.md`、`m3-batch22-code-r1-sonnet-output.md` 与几份模型日志），不回改。

## 十、没做什么

- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、GPU 对拍、QEMU、herd7、crates 变异整表、Verus）归提交时的主 agent；没提交。
- 没跑 harness 全量与慢档用例、没跑 `--all-features` 的 clippy（第七节）；最后两处串改之后没再编（只 fmt 核过）。
- 第四节标「留给门禁」的 9 行变异没单证；`lineage_tree` 那条 24 字节布局单测没证红。
- 没改 kb、没改 `research/`、没改 `admission.rs` 与模型的记账行数（第八节）。
- `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 一个字节没动，`cargo build -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes` 编过（第五节 e171 那一段）；它引的名字与签名没改，`MountError` 等穷举枚举没加成员（`LineageStepOfThePublish::AdministratorRollback` 多一个字段，E171 不引它）。
- `inode_tree.rs`、I-9.4、I-9.12 没碰；`mount.rs` 只动了 W3 那几个函数（`highest_inode_number_watermark_in_the_ring` 的文档、新加 `ring_roots_not_abandoned_by_crash_recovery`、`version_to_publish_the_rollback_after`、`roll_back_by_a_forward_publish` 取水位与谱系记录那几行）、文件头一行 import 与文件末尾单测里两处谱系记录字面量。

## 十一、草稿目录

`~/.local/share/singlefs/work/m3-batch49/w3/` 下留：`patch/`（交补丁的四份文件）、`report.md`、`test-1.log`、`test-2.log`、`prove-red-1.txt`、`prove-red-2.txt`、`prove-red-logs/`、`prove-red-logs-2/`、`final-*.log`、`clippy-*.log`、`naga-*.txt`、`grep-format-constants.txt`（旧值派生形态全仓搜的原样输出，239 行）、`diffstat.txt`、`tools/anchor-check.py`、`naga-probe/` 的源码。仓副本 `repo/`、原样基线 `base/`、`naga-probe/target/` 交回前删掉。
