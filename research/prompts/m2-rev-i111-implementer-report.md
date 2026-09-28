# 实审 B2 编号落定：I-MAPPING-KEY → I-1.11

## 写过的文件

- `crates/singlefs-checker/src/image.rs`：常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 的值从 `"I-MAPPING-KEY"` 改成 `"I-1.11"`；上方注释改成现状（不再写「占位」「编号由主 agent 定」）；`IMPLEMENTED_INVARIANTS` 数组里这一项从末尾挪到 `"I-1.10"` 之后、`"I-2.1"` 之前（按编号排序）。sha256（改后）：`1ac23367622bf41cd076d57d9efb289526a1f9686237676e3996ae179f3e7168`。
- `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`：模块头注释（原第 8 行）、`const MAPPING_KEY_INVARIANT`（原第 44 行，值同样从 `"I-MAPPING-KEY"` 改成 `"I-1.11"`）、小节分隔注释（原第 1268 行）三处占位语言改成 `I-1.11（映射 key 与单元头相符）`。sha256（改后）：`8959c896292ddcd42f30c067d03f0240438a25cece429d306996d866cc8baaf0`。
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`const MAPPING_KEY_INVARIANT` 上方注释（原第 502 行）、两处坏镜像用例注释（原第 958、985 行）改成现状；这个文件里 `const MAPPING_KEY_INVARIANT` 本身引用的是 `singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER`（常量名，不是字面串），随常量的值一起变，没有单独改这一行。sha256（改后）：`83e6f446a7fafa89be2333da9be11e452497165b3eaa091bd3110f62366b6801`。
- `crates/mutations.tsv`：没有追加、没有替换、没有删除任何行。检查过点名 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 的两行（第 874、878 行，锚点在 `crates/singlefs-checker/src/walk.rs`），它们的锚文本落在 `walk.rs` 里，这一轮没有改 `walk.rs`（不在「要动的 crates 文件」清单里），锚文本原样还在、恰好命中一次，没有腐化。

## 验收标准逐项核对

1. `grep -rn 'I-MAPPING-KEY' crates/` 零命中：
```
$ grep -rn "I-MAPPING-KEY" crates/ ; echo "exit:$?"
exit:1
```
2. 三份测试二进制全绿：见下一节「动到的测试二进制」。
3. 门禁 36、94、97 号各跑一次：见「门禁」一节。

## 主工作区的状态与验证做法

主工作区里 `crates/` 有大量未提交改动（`git status --short -- crates litmus` 显示 94 个文件、一处删除，属于里程碑二收尾另几批还没提交的工作，含派发提示里点名「别人在改」的那几批），其中 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`（后者是未跟踪的新文件）已经带着与本次任务无关的 clippy 违规（见下）。为了不被这些无关问题挡住、也不去动不该碰的文件，本轮的编译与测试验证在草稿目录的仓副本上做：`rsync -a --exclude target --exclude .git` 拷到 `/tmp/claude-1000/impl-rev-i111/repo-copy/`，副本自己的 `CARGO_TARGET_DIR=/tmp/claude-1000/impl-rev-i111/target`。取副本时记下派发提示点名「别人在改」的 9 个文件当时的 sha256（`/tmp/claude-1000/impl-rev-i111/other-sessions-sha256-at-copy-time.txt`，见附）。验证做完、交回之前已删掉副本与它的 target（`repo-copy` 181M、`target` 13G）。

## 动到的测试二进制（副本里跑，`bash research/scripts/capped.sh 4 bash research/scripts/run-with-memory-cap.sh 8G cargo test …`）

```
$ cargo test -p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.79s
```

```
$ cargo test -p singlefs-harness --test checker_known_bad_images
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 316.69s
```

`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`（`checker_known_bad_images.rs`）与两条按 `MAPPING_KEY_INVARIANT` 常量点名的用例都在这 39 条里，全绿。

另外顺手核了 `singlefs-checker` 自己的 `--lib`（image.rs 是它的源文件，没有自己的 `#[cfg(test)]`，但 `--lib` 5 个用例全绿，确认没引入编译期或运行期回归）：
```
$ cargo test -p singlefs-checker --lib
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 证红

这一轮没有写新测试（纯改常量取值与注释，字段语义、判定逻辑一个字都没动），`show-me-test.md`「每条新测试证明会红」不适用；改动是不是安全，靠上面三条既有测试二进制全绿来验（它们本来就覆盖 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 的正反两面）。没有新增变异行。

## `cargo fmt --check`

```
$ cargo fmt --check
```
输出里只有 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 一处文件的多段格式差异，与本次改动无关（这一轮没碰这个文件；`git status --short` 里它已经是 `M`，属于别的未提交批次）。本次改动的三个文件（`image.rs`、`checker_narrow_invariants_and_abandoned_roots.rs`、`checker_known_bad_images.rs`）在 `cargo fmt --check` 的输出里一次没出现，没有格式问题。

## `cargo clippy`（check.sh 的那一套 lint）

`cargo clippy --all-targets --all-features` 整仓跑不完：`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`（未提交的别的批次）与 `checker_narrow_invariants_and_abandoned_roots.rs` 里我没碰的两行（第 188、618 行，`match … other => panic!(…)`，通配臂）都是这一轮之前就在的违规，不在「要动的 crates 文件」清单内，没有改。改成分开验：

```
$ cargo clippy -p singlefs-checker --lib --all-features -- -D warnings -D clippy::wildcard_enum_match_arm -D clippy::allow_attributes_without_reason -D clippy::cast_possible_truncation -D clippy::cast_sign_loss -D clippy::cast_possible_wrap -D clippy::undocumented_unsafe_blocks -D clippy::shadow_unrelated
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
```
（`singlefs-checker` 的 `--lib` 在此之前那次全量 `cargo clippy --all-targets` 已经先于 `e156` 报错之前跑过、判定是通过，这一次复核仍是通过。）

```
$ cargo clippy -p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots --test checker_known_bad_images --all-features -- -D warnings <同上那一组 lint>
error: wildcard match will also match any future added variants
   --> crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs:188:9
error: wildcard match will also match any future added variants
   --> crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs:618:9
error: could not compile `singlefs-harness` (test "checker_narrow_invariants_and_abandoned_roots") due to 2 previous errors
```
第 188、618 行是这个文件里既有的两个 `match … other => panic!(…)`，都不在「要动的 crates 文件：只列 3 份，且是特定的占位名/注释」这项任务要改的范围内（任务给的三处是第 8、44、1268 行），也不是这一轮引入的：它们和 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 或 `I-1.11` 无关，是这个未跟踪新文件本身带来的既有违规。逐条去核，`checker_known_bad_images.rs`（我改过注释的那个文件）单独跑不报任何 clippy 违规：
```
$ cargo clippy -p singlefs-harness --test checker_known_bad_images --all-features -- -D warnings <同上那一组 lint>
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.0Xs（无 error 行）
```

## `cargo build --offline --all-targets`

在副本上跑（`--offline`，未联网重新拉取依赖）：
```
$ cargo build --offline --all-targets
   Compiling singlefs-format v0.1.0 (…)
   Compiling singlefs-checker v0.1.0 (…)
   Compiling singlefs-core v0.1.0 (…)
   Compiling singlefs-harness v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 48.02s
```
全绿，没有 warning。

## 门禁 36、94、97 号（主工作区跑，各原样末行）

```
$ bash .claude/gate.d/36-invariant-count-cross-file.sh "$(pwd)"
  ! 本次无对象可判：扫了 204 份 kb 文件（invariants.md 与变更史之外），没有一处写「N 条在用」的声明
exit:77
```

```
$ bash .claude/gate.d/94-checker-implementation-disjoint.sh "$(pwd)"
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 285 行里没有分支与循环（`#[cfg(test)]` 标着的项 271 行不扫）
    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑
exit:0
```

```
$ bash .claude/gate.d/97-invariant-field-anchors.sh "$(pwd)"
  ✓ 这次改动新写或改写的 12 行不变量都点名了已定分项（基准 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321）；存量 79 行里 34 行还点不出，那笔账记在 C69（已定不变量没有字段可判）
exit:0
```

36 号退出码 77（本次未跑/无对象可判），是脚本自己定义的「无对象」结局，不是判红；94、97 都是 0（通过）。

## `git diff --stat -- crates litmus`（原样；这一轮与并行在跑的其它会话共用同一份工作区，分不出每一行改动归谁）

```
 crates/mutations.tsv                               |  772 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  136 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1915 +++--
 crates/singlefs-core/src/admission.rs              |  629 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  302 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   48 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  199 +-
 crates/singlefs-core/src/mount.rs                  | 2748 ++++++--
 crates/singlefs-core/src/mounted_read.rs           |   43 +-
 crates/singlefs-core/src/recovery.rs               |  712 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  641 +-
 crates/singlefs-core/src/unit.rs                   |   43 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   80 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6818 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  183 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2505 ++++++-
 crates/singlefs-harness/src/crash_injection.rs     |  821 ++-
 crates/singlefs-harness/src/device_log.rs          |   38 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1141 ++-
 crates/singlefs-harness/src/lib.rs                 |   60 +-
 crates/singlefs-harness/src/model.rs               |  985 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  399 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |  106 +-
 .../tests/checker_known_bad_images.rs              |  986 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_five_publish.rs   |   22 +-
 .../tests/first_transaction_step_one_mkfs.rs       |    8 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  261 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 .../second_transaction_parallel_line_one_layer0.rs |   64 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...action_parallel_line_three_spill_over_layer0.rs |  138 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 ..._transaction_position_addressed_trees_layer0.rs |   49 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 .../tests/second_transaction_step_one_overwrite.rs |    3 +-
 ...action_step_three_acquisition_barrier_layer0.rs |   55 +-
 ...second_transaction_step_three_formatted_pool.rs |  688 +-
 ...transaction_step_three_formatted_pool_layer0.rs |   55 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  658 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  346 +-
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
 ...transaction_supplement_two_tree_split_layer0.rs |   40 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 94 files changed, 30792 insertions(+), 11331 deletions(-)
```

`crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs` 是未跟踪的新文件（`git status --short` 报 `??`），`git diff` 看不到未跟踪文件，上面这份 stat 里不出现它；这一轮对它的三处改动已在「写过的文件」一节逐行写明。

## 第 4a 条：受影响的层 0 流与崩溃枚举用例

这一轮没有改 `crates/singlefs-checker/src/` 任何一条不变量的判定集合（哪几条被判、怎么判、成立/违例/不适用的边界），只改了一个字符串常量的取值、注释与 `IMPLEMENTED_INVARIANTS` 数组里那一项的位置。逐条核过全仓引用 `IMPLEMENTED_INVARIANTS` 的地方（`grep -rn IMPLEMENTED_INVARIANTS crates/`，共 25 处），全部按名字查找（`.contains`、`.find`）、按集合判定（`BTreeSet`）、`.len()` 或原样遍历拼字符串（不按下标取值），没有一处按数组下标取用；这次改的三处坏镜像/查窄测试文件已经全绿跑过（见「动到的测试二进制」）。据此判断：不需要为这一轮排一次层 0 或崩溃枚举的快档回归；这一节按第 4a 条的要求单列，结论是「不适用」。

## 停下交主 agent 的设计问题

无。这一轮是纯粹的编号落定（把已经在 `.claude/kb/invariants.md` 登记好的 I-1.11 抄进代码），没有碰到条款没写、需要设计判断的地方。

## 没做什么

- 没有改 `.claude/kb/invariants.md` 第 34、297 行那两处仍写着「代码里暂用占位名 `I-MAPPING-KEY`……由主 agent 另派人换成 I-1.11」的话——kb 不在这个 agent 的写范围内，这两句现在已经过时（代码已经换完），需要 kb-scribe 或主 agent 另行回扫改成现状。
- 没有碰 `crates/singlefs-checker/src/walk.rs`（不在「要动的 crates 文件」清单内），尽管它的注释（第 849、854 行）与判定代码里仍写着 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`、「编号待主 agent 写回 kb 时定」——常量名没变、值已经是 `I-1.11`，功能不受影响，但那两处注释的字面话也过时了，留给下一轮或另一个不撞文件的实现员。
- 同样没碰 `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`、`second_transaction_step_three_formatted_pool.rs`（名字含 layer0 的按规矩不跑，前者本身也不在「要动的 crates 文件」清单内；后者的注释第 518 行也有一处待更新的占位语言，同样留给后续）。
- 没有跑全量 `cargo test --all`、没有跑任何名字含 layer0 的测试二进制、没有跑门禁全量或 `gate-triage`，这些照共用约束「不做」一节与派发提示「重型测试：不跑」，不归这一轮。
- `crates/mutations.tsv` 没有新增、替换或删除任何行（见「写过的文件」一节的说明）；`草稿目录/patch/` 没有建（这一轮直接改主工作区，不是交补丁模式）。
