# 实现员报告：I-9.16（树表条目按树 ID 严格升序且合发号次序），冷走读与池级 checker 同步判

日期：2026-09-27。底座：主工作区 + A3b 补丁（副本里 `apply-writer-patch.py` 打的，33 号 0）。补丁相对「主工作区 + A3b 补丁」出；交回前对主工作区（已含 A3b、C577、A3b 善后一之后）`git apply --check` 退 0。

## 一、结论

- core：`TREE_TABLE_ENTRIES_ORDERING_CONTRACT` 改成 `"I-9.16"`（`crates/singlefs-core/src/recovery.rs:1865`）。两道并成一个入口 `tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order`（1886 行）：先判 ①（1897 行，原函数不动），再判 ②（1920 行，改成拿树表条目算，不再拿 `FileVersionTreeIdentifiers`）。`rebuild_version` 在 2019 行调它（原来 ① 那一处；原来排在后面、拿 `FileVersionTreeIdentifiers` 判的 ② 那一处删掉了，② 挪到读任何一棵树之前）；`walk_to_file` 在 3644 行调同一个函数，位置是 `tree_table_entries_each_kind_at_most_once` 之后、读映射树和任何一棵树之前。
- ② 的判法：发号次序表 `TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`（1869 行，extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist）里，树表里有的种类按这个次序取树 ID，相邻两棵要严格升序。缺哪一种、有没登记的种类，这里不判，照旧由后面读那一棵的那一步报。detail 两句与 A3b 的原文逐字相同。
- checker：`IMPLEMENTED_INVARIANTS` 从 47 条变成 48 条，加了 `"I-9.16"`（`crates/singlefs-checker/src/image.rs:57`、105 行）。`walk.rs` 另写了一份判定 `judge_tree_table_entries_ordering`（2611 行），调用处在 942 行，就在 `walk_tree_table_and_central_mapping_root` 读完树表单元之后、逐条往下走之前。每读一版树表判一次，0 条的树表算成立。违例写明是哪一版根（根记录偏移 24 的实例、偏移 28 的 txg，常量在 5463 行；由记录施加出来的那一版写「由记录施加出来的那一版（实例、txg）」），以及是哪两条（下标从 0 数、树 ID、种类）。条目宽不到种类那 2 字节时只判 ①，② 记成「射程里判不了的那一部分」。种类常量 6/7/8 是 checker 自己写的（`walk.rs:47-49`，发号次序表在 52 行），没有从 core 引。94 号绿。
- 坏镜像：在 `known_bad_images` 里加了一份 `"I-9.16"`（`checker_known_bad_images.rs:929`），用的是探针甲：livelist 与稀疏旁表两条**互换种类**，只违反 ②。另加一条「只红 I-9.16」的测试（2704 行），写法照 I-9.15 那条：先判干净镜像上成立，再判违例集合恰为 `["I-9.16"]`、detail 逐字相等。
- 只选探针甲、没有在 `known_bad_images` 里再加一份只违反 ① 的，理由是：树表的 key 就是树 ID，checker 在树表上判 I-1.1 的「key 严格递增」和 ① 是同一个谓词（`crates/singlefs-checker/src/lib.rs:559-562`）。所以只违反 ① 的镜像（两条整条互换位置）必然 I-1.1 与 I-9.16 一起红，「只红 I-9.16」做不到。这一份放进了新测试文件，断言违例集合恰为 `["I-1.1", "I-9.16"]`。
- 新测试 `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 共 6 条：干净镜像上冷走读读得出文件、checker 判 I-9.16 成立；冷走读对探针甲、探针乙各一条，都断言拒成 I-9.16，而且拒之前读过的只有这条根的实例表与树表那几份（包了一层记录读位置的 `PoolReader`）；重建上一版对探针甲、探针乙各一条；探针乙在 checker 上只红 I-1.1 与 I-9.16，detail 逐字相等。

推翻条件：
- 冷走读在拒之前读了实例表、树表之外的单元，那两条冷走读测试会红。
- checker 在探针甲上多红了一条，或者 I-9.16 不红，known_bad 那条「只红」测试会红。
- 干净的 A3b 镜像上 I-9.16 判红，干净镜像那条测试会红。
- 主工作区在打这个补丁之前又改了这 5 份文件，`git apply` 会拒。

## 二、要主 agent 处理的（卡在清单外的文件）

**`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 打上本补丁之后红 2 条。** 这份文件不在我的单里，我没动。它在第 636 行把不变量名写死成字面量 `"D8 已定项 8 排序契约"`（第 624 行的文档注释里也有）。常量换成 `"I-9.16"` 之后，这两条就对不上了：
- `a_tree_table_whose_two_entries_swapped_their_kinds_is_refused_by_the_writable_mount_before_any_write`
- `a_tree_table_whose_two_entries_carry_the_same_tree_identifier_is_refused_by_the_writable_mount_before_any_write`

副本里实测，`cargo test -p singlefs-harness --test unit_area_start_…` 的结果是 `test result: FAILED. 12 passed; 2 failed`。panic 信息是 `Some(Recovery(InvariantViolated { invariant: "I-9.16", detail: "…" }))`，detail 与原来逐字相同，差的只是名字。

改法是两行，建议补丁放在 `/tmp/claude-1000/impl-tree-table-ordering/unit-area-start-invariant-name.patch`（相对主工作区出的，只改第 624、636 行）。`crates/mutations.tsv` 第 1330、1331 行（A3b 探针一、二）点名的就是这两条测试：这两行的锚点我换到了新函数上（见第四节），但在这份文件改名之前，这两条测试在基线上就是红的，59 号复跑这两行时算不上证据。

## 三、不变量原句（交 kb 第八批，我没写 kb）

> | I-9.16 | 树表条目按树 ID 严格升序且合发号次序：同一张树表里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist；中央映射树不进树表）排，相邻两棵的树 ID 严格升序。两道任一道不成立即违例。依据 D8（核心索引结构） 已定项 8「排序契约」与已定项 8 ②「八棵树的号从水位起连号发，次序照格式常量 11..18 那一组」。 | 判定处：重建上一版 `rebuild_version`、冷走读 `walk_to_file`（都在读任何一棵树之前拒）；池级 checker 每读一版树表判一次。 |

立号来由：实审 A3b Q2（`research/prompts/m2-rev-a3b-implementer-report.md` 第 90–93 行），用户 2026-09-27 弹窗定「立不变量并同步」。

书记员要知道：① 与 checker 在树表上判的 I-1.1（key 严格递增）是同一个谓词。这一条的判别力落在 ② 上：只违反 ① 的镜像，checker 上 I-1.1 与 I-9.16 两条一起红。

## 四、变异

`crates/mutations.tsv` 里追加 9 行（`patch/mutations-append.tsv`），整行替换 2 行（`patch/mutations-replacements.tsv`）。

替换的两行是 A3b 的「实审 A3b Q-A 探针一：重建不判七棵树按发号次序升序（写者装树表的断言 panic）」「实审 A3b Q-A 探针二：重建不判树表条目按盘上次序严格升序」。它们原来的锚点行 `tree_identifiers_ascend_strictly_in_the_issuing_order(&tree_identifiers)?;` 和 `tree_table_entries_ascend_strictly_by_tree_identifier(&tree_table_entries)?;` 已经不在了，换到新入口函数里 ② 与 ① 的那两行，点名的测试不变。**这两行没有证红**：点名的两条测试在打上本补丁、而第二节那两行还没改之前，基线上就是红的。

追加的 9 行逐条用 `research/scripts/prove-red.sh --copy …/work --memory 8G singlefs-harness <9 个名字>` 证过，基线 `test result: ok. 6 passed`。每行都是「改回去它就红」，脚本末行原样：`✓ 点名 9 条：跑了 9 条，跳过 0 条，跑的都抓到了`，退 0。改坏哪一行、哪条断言红、同时红了哪几条，见下表（测试名省掉公共前缀）：

| 变异名 | 改坏哪一行 | 点名的测试（红） | 同时红的 |
|---|---|---|---|
| 冷走读不判（改回只在重建上一版里判） | recovery.rs:3644 那一调用改成 `let _ = &entries;` | the_cold_walk_refuses_…swapped_their_kinds…（`assert_eq!` 期望 Err(I-9.16)，实得 Ok） | 冷走读 places 那条 |
| 重建上一版不判 | recovery.rs:2019 那一调用删掉 | rebuilding_…swapped_their_places | rebuilding_…kinds |
| 核心层不判 ① | recovery.rs:1889 ① 那一行删掉 | the_cold_walk_refuses_…places… | rebuilding_…places |
| 核心层不判 ② | recovery.rs:1890 ② 改成 `Ok(())` | rebuilding_…kinds | 冷走读 kinds 那条 |
| 不变量名改回「D8 已定项 8 排序契约」 | recovery.rs:1865 | rebuilding_…kinds | rebuilding_…places、冷走读两条 |
| checker 读树表时不判 | walk.rs:942 那一调用删掉 | the_clean_image_…holds…（I-9.16 报不适用，不是成立） | the_checker_reddens_…places |
| checker 不判 ① | walk.rs:2620 前面加 `false &&` | the_checker_reddens_…places（违例集合只剩 I-1.1） | 无 |
| checker 不判 ② | walk.rs:2657 前面加 `false &&` | a_tree_table_whose_two_entries_swapped_their_kinds_reddens_only_…（known_bad） | the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target |
| 清单里没有 I-9.16 | image.rs:105 改成重复的 `"I-9.15"` | the_clean_image_…holds…（`judge` 断言「不在清单里」panic） | the_checker_reddens_…places |

每条新测试至少被一行证过红：冷走读两条、重建两条、checker 两条（干净镜像那条、places 那条），known_bad 里新加的「只红」那条也在内。known_bad 里原有的 `the_clean_image_holds_every_invariant…` 那条，被「checker 不判 ②」一并证红。

留给 59 号整表复跑的：只有替换的那 2 行，理由见上。

## 五、受影响的层 0 流与崩溃枚举用例

checker 的判定集合多了一条 I-9.16：每读一版的树表判一次，0 条的树表也判，判成立。下面三件事会跟着变：

- **每条层 0 流、崩溃注入、随机历史的 checker 报告里多一格 I-9.16。** 这些报告都按 `IMPLEMENTED_INVARIANTS` 逐条报：`crates/singlefs-harness/src/layer0_progress.rs:721`（`implemented_invariant_named`）、`crash.rs:1682`（`checker_counts_by_invariant`）、`crash_injection.rs:680`、`history.rs:2078`。进度串里多一段 `I-9.16=评估过/判违例/不适用`；哪些计数行会跟着多出这一段，推的，没量过。仓里钉着 `I-9.15=` 那种进度串的，只查到 `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` 这一份实验产物（`grep -rln 'I-9\.15='`），crates 里没有钉这个串的用例。
- **会断言「每条不变量违例 0 次」或「清单外的都要评估到」的层 0 测试二进制**（我一条都没跑，名字含 layer0）：`first_transaction_step_seven_layer0.rs`、`second_transaction_step_zero_layer0.rs`、`second_transaction_step_three_formatted_pool_layer0.rs`、`second_transaction_parallel_line_one_layer0.rs`、`second_transaction_parallel_line_three_spill_over_layer0.rs`、`second_transaction_supplement_two_tree_split_layer0.rs`、`second_transaction_position_addressed_trees_layer0.rs`，还有非层 0 的 `crash_segments_per_device_and_torn_in_place_overwrites.rs:790`（逐条断言违例 0）。推的：写者照发号次序装树表并断言升序（主工作区今天的 `crates/singlefs-core/src/transaction.rs:6513-6518` 那条 `assert!`，副本里在 6504 行起），崩溃态上读得出的树表都出自写者，I-9.16 不该判红。在树表读不出的状态上，它和别的走读判定一样不评估。这条要层 0 快档核。
- 非层 0 里会逐条核 checker 判决、要求「不在不适用清单里的都成立」的 `second_transaction_step_three_formatted_pool.rs`（第 633 行断言条数等于清单长度），在副本里跑过：`test result: ok. 14 passed`。mkfs 刚写完的空树表上 I-9.16 判的是成立，不是不适用，所以那几张不适用清单不用改。
- 钉值：没有一条层 0 流或崩溃枚举用例的钉值直接写着 checker 的不变量条数（`grep -rn 'IMPLEMENTED_INVARIANTS.len()'` 只命中 `second_transaction_step_three_formatted_pool.rs:633`、`:2040`，都是与清单长度比，不是写死的数）。

## 六、交回前的验证（各贴末尾原样）

副本里（主工作区 + A3b 补丁 + 本改动），每条 cargo 都经 `research/scripts/capped.sh 3`，cargo test 另外经 `run-with-memory-cap.sh 8G`：

- `cargo test -p singlefs-harness --test tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker`：`test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.21s`
- `cargo test -p singlefs-harness --test checker_known_bad_images`：`test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 129.26s`（基线 40 条）
- `cargo test -p singlefs-core --lib`：`test result: ok. 132 passed; 0 failed; …`（基线 132）
- `cargo test -p singlefs-checker --lib`：`test result: ok. 5 passed; 0 failed; …`（基线 5）
- `cargo test -p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over`：`test result: FAILED. 12 passed; 2 failed; …`（基线 14 条全绿；红的是第二节那两条，只差不变量名）
- `cargo test -p singlefs-harness --test second_transaction_step_three_formatted_pool`：`test result: ok. 14 passed; 0 failed; …`
- `cargo fmt --all -- --check`：退 0，无输出。
- `cargo clippy --all-features -p singlefs-core -p singlefs-checker -p singlefs-format --all-targets -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 那 7 条：`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.00s`，退 0。
- 同一组 lint 跑 `-p singlefs-harness --lib --bins --test checker_known_bad_images --test tree_table_ordering_… --test unit_area_start_…`：`Finished …`，退 0。
- 工作区 `--all-targets` 的 clippy 与 `cargo build --offline --all-targets` 都只红在 `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs`（2 处 `E0599`，`JournalRingPastTheCompiledUnitAreaStartUnsupported`）。这份文件是 A3b 善后一在改的，与派发提示预期的一致。build 末行原样：`error: could not compile \`singlefs-harness\` (test "core_review_geometry_back_chain_and_empty_inode") due to 2 previous errors`
- 门禁（副本里跑）：94 号退 0，`✓ checker 与实现只共享常量模块 \`singlefs-format\`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 \`singlefs_core\`（别名引进来的 0 个）；共享模块 1 份源码的正文 286 行里没有分支与循环（\`#[cfg(test)]\` 标着的项 313 行不扫）`
- 33 号退 0：`✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.tsv 1342 条的原文各命中源码一次；…`
- 53 号退 0（4 个占位）；92 号退 0（`111 个格式常量里变了 0 个`）；93 号退 0。89 号退 77，本次未跑，不算通过，末行：`「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`（与本改动无关）。74 号见第七节。

## 七、74 号与整份补丁在主工作区现状上的核

把主工作区现状（已含 A3b、C577、A3b 善后一）rsync 两份：`verify` 用 `apply-writer-patch.py` 打上本补丁目录，`verify-base` 不打。两份 `crates/` 只差本补丁那 6 个文件（`diff -rq` 核过）。

- `verify` 上 `apply-writer-patch.py` 的输出：`✓ 打上了：patch（补丁 有）`，33 号退 0：`crates/mutations.tsv 1354 条的原文各命中源码一次`。
- `verify` 上 `cargo build --offline --all-targets` 退 0，末行 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 43.50s`。A3b 善后一打进来以后，core_review 那份也编得过了。
- **74 号**（`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`）：
  - `verify` 上退 1，`test result: FAILED. 21 passed; 3 failed; 2 ignored`。
  - `verify-base` 上（`SINGLEFS_GATE_FULL=1`，不强制时它退 77：干净工作树）同样退 1，`test result: FAILED. 21 passed; 3 failed; 2 ignored`，红的是同样 3 条：`crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`、`seed_4000000204_raising_the_floor_on_narrow_devices_is_refused_before_any_write_instead_of_leaving_the_new_floor_on_one_device`、`writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged`（都在 `second_transaction_supplement_three_random_history`）。
  - 所以这 3 条红是主工作区现状本来就有的，不是本补丁带进来的。我没去修它们：不在我的单里，也不在这件活的范围里。
  - 打补丁那一份的各不变量计数里多了一行 `I-9.16：判绿 3520 次、不适用 0 次`。其余各行与基线逐行相同；日志只留了尾部，两份都截在 26 行，所以差出来的是首行 I-5.2 与末行 I-9.16。

## 八、停下交主 agent 的设计问题

- **② 管不管中央映射树的号。** D8 已定项 8 ② 的发号次序里有中央映射树（第 5 个，第一版恒为 15），它不进树表。它的号在核心层是根记录映射根指针的出生树（`root.mapping_root.head.birth_tree`），在 checker 里是映射根指针的 `birth_tree`。A3b 的原判定、主 agent 给的名字（「树表条目……」）和本实现都只比树表里那七棵，不拿映射树的号去比「记账 < 映射 < livelist」。要不要把它纳进 I-9.16，条款没写，我没定。
- **② 只比树表里有的种类。** 缺一种、有没登记的种类，在核心层照旧由后面读那一棵的那一步报（`rebuild_version` 报「树表里没有那棵树」，`walk_to_file` 报「树表里缺一棵有根的树」或「树的种类没登记」）。checker 不另立判定。同一种出现两条时，checker 取先出现的那条来比。核心层在这之前已经把「同一种两条」判成损坏了。
- **多处违例同时存在时，报哪一个错变了。** `rebuild_version` 里原来 ② 排在水位判定、缺树判定之后，现在挪到它们之前。只违反其中一条的镜像，报错不变；同时违反 ② 和「缺树」或「水位」的镜像，现在报的是 I-9.16。这是「在任何往下走之前拒」的直接后果。

## 九、这一轮写过的文件

副本里改的，主工作区一个字没动：
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-checker/src/image.rs`
- `crates/singlefs-checker/src/walk.rs`
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`
- 新建 `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`
- `crates/mutations.tsv`：追加 9 行，名字都以「I-9.16 树表排序：」开头，全名见 `patch/mutations-append.tsv`；整行替换 2 行（「实审 A3b Q-A 探针一……」「实审 A3b Q-A 探针二……」）。

草稿目录里另有 `unit-area-start-invariant-name.patch`：清单外那份测试文件的建议改法，没打进任何地方。它对主工作区 `git apply --check` 退 0。

`git diff --stat -- crates litmus`（副本「主工作区 + A3b 补丁」上出，新文件用 `git add -N` 计入）原样：

```
 crates/mutations.tsv                               |  13 +-
 crates/singlefs-checker/src/image.rs               |   6 +-
 crates/singlefs-checker/src/walk.rs                | 106 +++++-
 crates/singlefs-core/src/recovery.rs               |  82 +++--
 .../tests/checker_known_bad_images.rs              |  80 +++++
 ...g_is_judged_by_the_cold_walk_and_the_checker.rs | 384 +++++++++++++++++++++
 6 files changed, 641 insertions(+), 30 deletions(-)
```

补丁目录 `/tmp/claude-1000/impl-tree-table-ordering/patch/` 里有这几个文件，sha256 如下：
- `crates.patch` `ff12e50629dfbca69c830bf609f7b26248cf83a6abe100bc26d02695014b7545`
- `mutations-append.tsv` `31e245750d5482a79ecd0083eddf24d31f0bbd74635f6ffb92ef973602d217e3`
- `mutations-replacements.tsv` `71ca6e39c0c76609c3b2a4b736685d45932e197a084247310133cf0323f97206`
- `report.md`，就是这份报告。

对主工作区 `git apply --check patch/crates.patch` 退 0（最后一次核在删副本之前）。

## 十、删了的仓副本

- `/tmp/claude-1000/impl-tree-table-ordering/work`，9.4G
- `/tmp/claude-1000/impl-tree-table-ordering/check-base`，2.5G
- `/tmp/claude-1000/impl-tree-table-ordering/verify`，18G
- `/tmp/claude-1000/impl-tree-table-ordering/verify-base`，1.6G

四份都已 `rm -rf`。草稿目录里只剩日志、报告和补丁目录。

## 没做什么

- 没走三方对抗。层 0 全量、QEMU、herd7 与 crates 变异整表归 `crash-verifier`（层 0 快档在整轮门禁里）。没提交。
- 名字含 layer0 的测试二进制一条都没跑，第五节列的那几份要层 0 快档核。`crash_segments_per_device_and_torn_in_place_overwrites` 也没跑。
- 替换的那 2 行变异没有证红，见第四节。
- 清单外的 `unit_area_start_…` 两条红没修，见第二节。
- 74 号那 3 条基线红没修，见第七节。
- 没写 kb。不变量原句在第三节，交 kb 第八批。
- `cargo test --all` 与 `check.sh` 整份没跑，只跑了 `CODE_DISCIPLINE_LINTS` 那几条 clippy。
- 89 号退 77，本次未跑，不算通过。
