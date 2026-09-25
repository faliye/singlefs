# impl-m2-checker2 报告（实二：收口表第 26 行 I-7.9、第 ② 行岔路 7 I-3.11、第 8 行 C493 判别力变异）

时刻一律 UTC（东京 = UTC+9）。前一个实现员（a2346578aff8a874c）02:38 撞会话限额中断，本份由接手的实现员 03:09 起核现场、补做、写成。主工作区一个字没动。

## 一、结论

1. **I-3.11（已分配减 defer 等于最新根走读）做成套，进补丁**：池级 checker 判定（`walk::check_pool_image`，逐盘判「defer 待释放 + 最新根走读 == 已分配」，写成加法、不回绕）、`IMPLEMENTED_INVARIANTS` 40 → 41、两份只红它的坏镜像（盘 0 的 defer 行 +1 槽 / −1 槽，I-3.1 与 I-5.2 照旧真被评估过且成立）、造出来的基底（β_syn：两盘 defer 挪回 0）上的判别力自证、层 0 两条流 + 第一个事务第 7 步层 0 流 + `second_transaction_step_three_formatted_pool.rs` 三组清单 + 随机历史「至少判过一次成立」清单都加上 I-3.11、5 条变异。5 条在主工作区现状 + 补丁上逐条证过会红（第五节）。
2. **C493（回退候选集条文与实现说反话）**：1 条变异（`mount.rs` 里把「目标那一版树表 0 条」那条排除加回回退候选集），在主工作区现状 + 补丁上证过：点名的随机历史快档由绿转红（第六节）。
3. **I-7.9（回退下界 F 不高于抬 F 的上限）停下交回，不进补丁**：照主 agent 01:00 UTC 追加的那条读法（上限里的三处「有效根」不带 txg ≥ F，「非空」里的「前一条有效根」带）写了判定、坏镜像与测试；坏镜像只红 I-7.9，但**合法历史上判红**：仓里现成的固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 在主工作区现状 + 草稿上 03:17 复跑仍红，随机历史快档与偏回退档各有种子红，都是「合法抬 F 之后回退到候选集里的一条根」。照那条指示不改读法去躲；历史、读数、为什么是条款问题、措辞建议在第七、八节。草稿补丁在草稿目录，不交付。
4. **什么现象会推翻第 1 条**：主工作区现状 + 补丁上有一份合法镜像 I-3.11 判红（`cargo test --all` 02:59 那一轮与随机历史、崩溃注入各档里 I-3.11 零违例，见第九节；层 0 全量没跑）；或者第五节任何一条变异改回去之后点名的测试不红。
   **推翻第 3 条（「是条款问题、不是实现错」）**：拿出一种不改 D16（发布语义） 已定项 1 条文的读法，能同时让第七节那份抬过上限的镜像红、让那几段合法回退历史绿。

## 二、交付物

| 东西 | 路径 | sha256 |
|---|---|---|
| 补丁（只含 `crates/`，不含 `crates/mutations.tsv`，8 个文件） | `/tmp/claude-1000/impl-m2-checker2/impl-m2-checker2.patch` | `c392400becfc173804a32f4d2d48d3883b41cef99e352ba15cb620d30938af43` |
| 新变异行（6 行，整行，六段） | `/tmp/claude-1000/impl-m2-checker2/mutations-append.tsv` | `46c90717bec4600ec217ed57c0813b83ee3962c9736ba4d2a56f3bb41aa6c71d` |
| 本报告 | `/tmp/claude-1000/impl-m2-checker2/report.md` | 交回里给 |
| I-7.9 草稿补丁（打在上面那份补丁之上，**不交付**） | `/tmp/claude-1000/impl-m2-checker2/drafts/i79-draft-on-top-of-main.patch` | `aa84c8a306366f86d7e9783fdcdf497142ebf46305e4b9f76e3e2e2f72571292` |
| I-7.9 证据用例（草稿，只在证据副本里跑，靠关掉入口上限拒绝的那一处改动才造得出抬过上限的镜像） | `/tmp/claude-1000/impl-m2-checker2/drafts/zz_i79_evidence.rs`、`drafts/i79-evidence-mount.diff` | `7d478bf4264a9d2ffe25330eb3376f214c4c6d2915c464e0fc5a12875ecb93da`、`8c73c82a853dfdc370b1a44f303bb5288e6017c26080aa73bb720d14176d0113` |
| 证据用例的输出（前一个实现员 01:48 跑，主工作区五份补丁打进之前的代码） | `/tmp/claude-1000/impl-m2-checker2/drafts/i79-evidence-output.txt` | `34a700e1b9eaf4910dc2e08b855e79a0544e8cf9091d1e0636723500cf87e2ea` |

这几份 I-7.9 草稿没入库：I-7.9 停在条款问题上，要不要留、落在哪由主 agent 定；证据用例靠证据副本里关掉 `raise_rollback_floor` 上限拒绝的一处改动，不能进 `crates/`。

主工作区现状（03:10 核：`/tmp/claude-1000/impl-m2-checker2/base-crates-now/` 与主工作区 `crates/` 逐文件相同，那是 02:36 拷的、五份补丁已在里面）上打得上：

```text
$ diff -rq --exclude target base-crates-now /home/fy5090/code/singlefs/crates; echo "diff-exit=$?"
diff-exit=0
$ git -C /home/fy5090/code/singlefs apply --check /tmp/claude-1000/impl-m2-checker2/impl-m2-checker2.patch; echo "apply-check-exit=$?"
apply-check-exit=0
```

`sync/`（跑 `cargo test --all`、`check.sh` 的那份）的 `crates/` 与「主工作区现状 + 补丁」逐文件相同（新拷一份 `base-crates-now` 打上补丁再 `diff -rq`，退出 0）。交回前最后一次核在第十二节。

## 三、这一轮写过的文件

补丁里的 8 个（副本 `repo/` 里写、`sync/` 里是同一份打在主工作区现状上）：

- `crates/singlefs-checker/src/image.rs`：`IMPLEMENTED_INVARIANTS` 加 `I-3.11`（40 → 41）。
- `crates/singlefs-checker/src/walk.rs`：常量 `STATISTIC_DEFER_QUEUE_BYTES = 5`；函数 `slots_referenced_per_device`（I-3.1 与 I-3.11 共用的逐盘加法）；最新根走读完那一刻取一份 `slots_referenced_by_the_newest_root`；I-3.1 的「遍历得到的和」改由同一个函数从全部候选版本那一份算（与原来按 `per_device` 求和同值，`per_device` 仍给 I-5.1 用）；I-3.11 判定与「最新根下面没有记账树」时的不适用。
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：新用例两条（第五节）；`known_bad_images_of_the_deferred_statistic` 接进 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的那条链；纯泄漏两份、候选 b 那条「再泄漏一槽」、条目宽那条的期望跟着 I-3.11 改（第五节末尾）。
- `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`、`second_transaction_step_zero_layer0.rs`、`second_transaction_step_three_formatted_pool_layer0.rs`：「必须评估到」清单加 I-3.11。
- `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`：三组「不适用」清单加 I-3.11（17 → 18、18 → 19、16 → 17）。
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`：「checker 至少判过一次成立」清单加 I-3.11。

`crates/mutations.tsv` 要追加的 6 行（`mutations-append.tsv`，变异名第一段）：

1. `岔路 7（G27）立的 I-3.11（已分配减 defer 等于最新根走读）：判定恒真（defer 账对不上也判绿）`
2. `I-3.11 不减第 5 项（判别力自证：造出来的 defer 为 0 的基底上再记一槽 defer，判定由红转绿）`
3. `I-3.11 的等号放宽成 ≥（defer 多记一槽那份坏镜像判绿）`
4. `I-3.11 的等号放宽成 ≤（defer 少记一槽那份坏镜像判绿）`
5. `I-3.11 拿走完全部候选版本之后那一份走读（I-3.1 的并集）去比、不只取最新根：发布 B 之后的干净镜像上判红`
6. `C493（回退候选集条文与实现说反话）：把「目标那一版树表 0 条」那条排除加回回退候选集（回退到树表 0 条的根被拒）`

草稿目录里的（都在 `/tmp/claude-1000/impl-m2-checker2/` 下，只归这一轮）：副本 `repo/`、`sync/`、`sync2/`、`sync3/`、`mutant/`、`evidence-i79/`、`i79-check/`、`verify-apply/`，主工作区 `crates/` 快照 `base-crates/`（00:18 版）、`base-crates-now/`（02:36 版）、`base-crates-now2/`（03:27 版）、`base-crates-now3/`（04:03 版），`drafts/`、`logs/`。

主工作区一个字没动，所以 `git diff --stat -- crates litmus` 在主工作区里量到的全是别的会话的改动、不是这一份的；这一份的改动量用补丁自己报（原样）：

```text
$ git apply --stat /tmp/claude-1000/impl-m2-checker2/impl-m2-checker2.patch
 crates/singlefs-checker/src/image.rs               |    9 -
 crates/singlefs-checker/src/walk.rs                |   46 ++++
 .../tests/checker_known_bad_images.rs              |  240 +++++++++++++++++++-
 .../tests/first_transaction_step_seven_layer0.rs   |   11 +
 ...second_transaction_step_three_formatted_pool.rs |   22 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 
 .../tests/second_transaction_step_zero_layer0.rs   |    5 
 ..._transaction_supplement_three_random_history.rs |    3 
 8 files changed, 301 insertions(+), 39 deletions(-)
```

## 四、I-3.11 判定读的是什么

- 条款：`.claude/kb/invariants.md` 第 137 行（I-3.11 那一行）。读镜像里最新根下面记账树的两行：(1, 设备) 已分配、(5, 设备) defer 待释放（D5（快照 / 空间记账机制） 已定项 4 第 1、5 项），不读内存里的分配器。
- 右边「从最新有效根走读到的、这块盘上被引用的槽数」：与 I-3.1 同一个走法（`walk.references` 里记下的 (设备, 起点槽, 跨度)），只取 `walk.walk_root(最新根)` 刚走完那一刻的那一份（`walk.rs` 第 2907 行），在候选集里别的根与由记录施加出来的版本并进来之前。判定在第 3190 行。
- 最新根下面没有记账树（第 0 代树表）时与 I-3.1、I-5.2 一起报不适用；两行任一行缺，判红（`matches!` 要两行都在）。
- 可达状态上它不跟 I-3.1 的已知红一起红：随机历史快档里 I-3.1 有 2387 步是「根环转过一圈」那一形的已知红，I-3.11 同一批 3618 次判定里判绿 3605、不适用 13、红 0（第九节）——它只走最新根，不受环里旧根被挤掉的影响。

## 五、每条新测试的「改坏哪一行 → 哪条断言红」

在 `mutant/`（03:13 用 `rsync -rlp --checksum` 同步成与 `sync/` 逐文件相同，改过的文件拿到新 mtime；`diff -rq` 退出 0）里做：先跑不改动的整个二进制取基线红集，再逐行改坏、跑整个测试二进制、从 `sync/` 原件拷回并 `touch`、`cmp` 相同。脚本 `drafts/run_mutation_proofs_round3.sh`，日志 `logs/round3/`。代码行号是打上补丁之后的文件的行号。

**基线红集**：`checker_known_bad_images` 25 过 0 红（`logs/round3/baseline-known-bad.log`，03:14）；`second_transaction_supplement_three_random_history` 见第六节。基线红集为空，下面列的红全在基线之外。

| 变异行 | 改坏哪一行 | 点名的测试红在哪条断言 | 同一个二进制里同时红的 |
|---|---|---|---|
| 1 判定恒真 | `walk.rs` 第 3190 行前加 `true \|\|` | `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`，`checker_known_bad_images.rs` 第 2570 行「判红的该只有 I-3.11」，left `[]`、right `["I-3.11"]` | 共 4 条红：另有 `on_a_synthetic_base_…`、`a_pure_leak_that_keeps_free_plus_allocated_equal_to_the_unit_area_reddens_only_the_allocated_statistic_invariant`、`a_version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_a_leak_on_top_still_reddens_it`（后两条登记了 I-3.11 跟着红，恒真之后不红了） |
| 2 不减第 5 项 | 第 3190 行 `deferred.checked_add(referenced_by_the_newest_root)` → `Some(referenced_by_the_newest_root)` | `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`，第 2661 行「defer 多记一槽：I-3.11 红（12 − 1 ≠ 12）」，left `["I-3.1"]`、right `["I-3.1", "I-3.11"]`——造出来的基底上由红转绿的就是这一格 | 共 20 条红（可达镜像上 defer ≥ 1，不减它就在每份干净镜像上红） |
| 3 `==` → `>=` | 第 3190 行 | `live_unit_…`，第 2570 行，defer +1 那份判绿：left `[]` | 共 2 条：另有 `on_a_synthetic_base_…` |
| 4 `==` → `<=` | 第 3190 行 | `live_unit_…`，第 2570 行，defer −1 那份判绿：left `[]` | 共 3 条：另有纯泄漏那条与候选 b「再泄漏一槽」那条 |
| 5 拿全部候选版本那一份比 | 第 3183 行 `slots_referenced_by_the_newest_root` → `slots_referenced_by_every_walked_version` | `live_unit_…`，第 2561 行「发布 B 之后的干净镜像上 I-3.11 要真被评估过且成立」，left `Violated("盘 0：记账的已分配 Some(376832) 减 defer 待释放 Some(180224)，不等于从最新根（txg 4）走读到的 376832")` | 共 19 条红 |

每条各跑一遍 `cargo test -p singlefs-harness --test checker_known_bad_images --no-fail-fast`，退出码都是 101，`restored crates/singlefs-checker/src/walk.rs` 都记在日志末尾。被测代码里没有 `debug_assert` 挡在前面：五条的红都落在测试自己的 `assert_eq!` 上（上表列的行）。

已有用例跟着 I-3.11 改的期望（不是新测试，列出来免得被当成放宽）：

- `a_pure_leak_that_keeps_free_plus_allocated_equal_to_the_unit_area_reddens_only_the_allocated_statistic_invariant`：Z3-A 只抬「已分配」、defer 行没动 ⇒ I-3.11 跟着红（它的射程本来就罩这一形），期望改成 `["I-3.1", "I-3.11"]`；Z3-B 的 defer 行跟着抬了同一槽 ⇒ 仍只红 I-3.1。每份的期望登记在常量 `INVARIANTS_EACH_PURE_LEAK_REDDENS`，别的判定照旧逐项比干净镜像。
- `a_version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_a_leak_on_top_still_reddens_it` 的「再泄漏一槽」：同 Z3-A，期望改成 `["I-3.1", "I-3.11"]`。
- `an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant`：走读在 I-1.10 那一步停下、记账条目不解 ⇒ I-3.11 与 I-3.1、I-5.2、I-9.6 一起转不适用（三条 → 四条）。

## 六、C493 那条变异（第 6 行）

条款：`.claude/kb/checks-owed.md` 第 436 行（C493），「还欠：一条变异把那条排除加回来，模型对拍必须由绿转红」。

- 改坏哪一行：`crates/singlefs-core/src/mount.rs` 里 `let target_has_no_file = tree_table_has_no_entries(&*devices, &target_root)?;` 之后加回「`target_has_no_file` 就返回 `RollbackTargetNotACandidate { exclusion: NotInRing }`」。这一行在主工作区 02:36 与 03:27 两版里是第 1449 行，04:03 那一版是第 1454 行；三版里原文都恰好命中一次。
- 点名的测试：`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`（`second_transaction_supplement_three_random_history`）。
- 结果：见本节末尾（主工作区现状 + 补丁上复证，日志 `logs/round3/mutation-row-6.log`）。
- 基线：同一份 `mutant/` 不改动跑 `second_transaction_supplement_three_random_history` 整个二进制，18 过 0 红 2 忽略（`logs/round3/baseline-random-history.log`，03:21，退出 0）。
- 改坏之后（`logs/round3/mutation-row-6.log`，03:25，退出 101）：点名的测试红在 `second_transaction_supplement_three_random_history.rs` 第 246 行 `assert!(report.new_findings.is_empty(), …)`，新发现是理想模型对拍的 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`，原话：`模型答 MountRollback 该成（写出 [(1, 1), (2, 1)]；允许拒的只有容量墙区间与 []）；实现 拒了：MountError::RollbackTargetNotACandidate(NotInRing)`；快档 96 段里新发现 42 段。
- 同一个二进制里同时红的共 5 条：`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`、`rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms`、`reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms`、`allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count`、`unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`。
- 还原：从 `sync/` 拷回 `mount.rs` 并 `touch`，`cmp` 相同；跑完之后 `diff -rq sync/crates mutant/crates` 退出 0（`logs/round3/final-diff.txt`）。

## 七、I-7.9 停下交回：合法历史上判红

**照的读法**（主 agent 01:00 UTC 追加）：上限 min(每块盘上最新的有效根, 第 4 新的非空有效根)、非空不足 4 个取最旧有效根，这三处「有效根」= 自证合法 ∧ 按最新根指着的实例表判仍然有效，**不带** txg ≥ F；「非空」里的「它前一条有效根」照 D16（发布语义） 已定项 1「「非空」从盘上怎么认」原文带 txg ≥ 当前的 F（最新根带的那个）。条款：`.claude/kb/invariants.md` 第 59 行，`.claude/kb/decisions/16-发布语义.md` 第 27 行起的「#### 已定项 1」。

**判红的合法历史**（草稿补丁 `drafts/i79-draft-on-top-of-main.patch` 打在主工作区现状 + 交付补丁之上）：

| 历史 | 在哪一步红 | checker 的原话 |
|---|---|---|
| 固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`（首个文件、重开、覆盖写 4 次、抬 F 到 6（推 txg 10、11）、回退到 F 上那条根 (2, 6)、冷启动） | Operation(6)，回退挂载之后 | `最新根带的回退下界 F 6 高于抬 F 的上限 0：每块盘上最新的有效根 {0: 12, 1: 13}，txg ≥ F 的非空有效根（从新到旧）[6]，最旧有效根 txg 0` |
| 随机历史快档，种子 7463871032432355159（同签名 [7463871032432355159, 7463871032432355168, 7463871032432355197]） | Operation(21)，CloseAndMountRollback 之后 | `最新根带的回退下界 F 1 高于抬 F 的上限 0：每块盘上最新的有效根 {0: 23, 1: 22}，txg ≥ F 的非空有效根（从新到旧）[10, 9, 8]，最旧有效根 txg 0` |
| 偏回退档，种子 7463871032432355127（同签名 [7463871032432355127, 7463871032432355149]） | Operation(22)，CloseAndMountRollback 之后 | `最新根带的回退下界 F 17 高于抬 F 的上限 5：每块盘上最新的有效根 {0: 27, 1: 28}，txg ≥ F 的非空有效根（从新到旧）[17]，最旧有效根 txg 5` |
| 证据用例：步 5 的脚本（回退到 A、覆盖写到 txg 14、抬 F 到 11、再覆盖写 E），再回退到 (3, 12) | 回退之后，最新根 (4, 19)，实例表行 [(1, 3), (2, 0), (3, 12)] | `F 11 高于抬 F 的上限 0：每块盘上最新的有效根 {0: 18, 1: 19}，txg ≥ F 的非空有效根（从新到旧）[12, 11]，最旧有效根 txg 0`；不发 E、直接回退的那一段一样 |

固定用例那一行 03:17 在主工作区现状 + 草稿上复跑过（`logs/round3-i79/fixed-rollback-test.log`，退出 101，原话同上）；随机历史两行与证据用例是前一个实现员 01:3x–01:48 在五份补丁打进之前的代码上跑的（`drafts/i79-targeted-tests.log`、`drafts/i79-evidence-output.txt`），没在现状上复跑全档。同一份草稿的坏镜像（`raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`）03:17 在现状上照旧过（只红 I-7.9）。

**为什么是条款问题，不是实现错**：

- 实现的上限是**抬 F 那一刻**的量：`crates/singlefs-core/src/mount.rs` 的 `raise_rollback_floor` 在第 811 行把抬之前的 F（`current.root.rollback_floor`）传给 `rollback_floor_ceiling`，后者第 689 行按 txg ≥ 这个 F 取有效根，第 814 行 `new_floor > ceiling` 就拒。上面四段历史里抬 F 都在上限之内做成（证据用例：抬之前实现报上限 11，抬到 11）。
- 之后一次合法回退（目标 txg ≥ F_生效，在候选集里）让实例表把目标之后那一段判抛弃，当初撑起 F 的那几条非空根一起出了「有效」；F 不能跟着降（F 之下的单元已回收复用）。于是在回退之后的任何一张镜像上重算上限，都可以低于 F——不论「当前的 F」怎么读：
  - 照 invariants.md 字面（有效带 txg ≥ 最新根的 F）：第 4 新的非空根与最旧有效根两支按构造 ≥ F，检查近乎恒真；证据用例里把 F 越过上限抬到 12 的那份镜像，实现的 `rollback_floor_ceiling` 按 F = 12 算出上限 12，抓不到。
  - 照 01:00 的读法（上限里的有效不带 txg ≥ F）：抓得到那份越界镜像，但上面四段合法历史全红。
- 所以「F 不高于抬 F 的上限」是**对抬 F 那个动作**的约束，不是一张镜像在任何时刻都成立的性质；按单张镜像重算上限，回退之后必然误红或恒真二选一。

**已有的、罩着抬 F 那一刻的检查**（供主 agent 判要不要另立）：随机历史的理想模型每次抬 F 都比实现报的上限（`crates/singlefs-harness/src/history.rs` 第 2697 行把 `raised.ceiling` 交给 `judge_by_model`，拒绝那一支第 2747 行比 `reported_ceiling_of_mount_error`）。它能不能抓到步 5 验收「F 抬过上限」那条变异，没验。

## 八、`invariants.md` 那两行的建议措辞（我不改 kb）

**I-3.11（第 137 行）状态列**，补丁与 6 行变异打进主工作区之后：

> 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7））

变异行号等主 agent 追加之后现查（主工作区 03:10 是 440 行，追加在末尾）。

**I-7.9（第 59 行）状态列**：

> 未实现（2026-09-24 实现员照「上限里的有效根不带 txg ≥ F」写了池级判定：越过上限的坏镜像只红它，但合法的「抬 F 之后回退到候选集里的根」历史上判红——回退让撑起 F 的非空根出了有效集合，F 不降，按单张镜像重算的上限低于 F；照字面「有效带 txg ≥ 当前的 F」又恒真。定义要先改成按抬 F 那一刻判，见实现员报告 `impl-m2-checker2` 第七节；里程碑「第二个事务」收口表第 26 行）

**I-7.9 定义列的一种改法**（设计判断，由主 agent 定；我没照它写代码）：

> 根环里每一条**抬了 F 的根** r（r 带的 F 高于同一实例里按 txg 排在 r 前面、仍有效的那条根 p 带的 F；回退或新实例的第一条根没有同实例的 p，不算抬），都满足 r 带的 F ≤ 上限(r)。上限(r) 照 D16（发布语义） 已定项 1 按 r 写出之前的根算：只看 txg < r 的根；有效 = 自证合法 ∧ 按 r 自己指着的实例表判仍然有效 ∧ txg ≥ p 带的 F（抬之前的 F）；「非空」里的「前一条有效根」用同一个「有效」。p 或上限要用到的根已被根环盖掉 ⇒ 这一条 r 报不适用。

这样读，理由三条：抬之前的 F 正是实现入口用的那个（`mount.rs` 第 811 行）；r 自己的实例表不受之后的回退影响，回退之后的新根不是「抬了 F 的根」，第七节四段合法历史都不判；越界那份镜像上 r 就是那条带 F = 12 的根、p 带 F = 0，上限按实现算是 11，红。没验的：这个读法在层 0 两条流与随机历史各档上零误红、在环转过一圈之后有多少 r 只能报不适用——要主 agent 定了读法再做。另一条路是不在单张镜像上判，把「F 抬过上限」交给抬 F 那一刻的模型对拍（第七节末尾那一处），I-7.9 改成那一处的检查。

## 九、受影响的测试在主工作区现状 + 补丁上的结果

- `sync/`（主工作区 02:36 版 + 交付补丁，与现状逐文件相同）上跑 `cargo test --all --no-fail-fast`（前一个实现员 02:37 起、02:59 完，`logs/sync-test.log`）：退出 0；各二进制的 `test result` 加起来 457 过、0 红、8 忽略（忽略的是层 0 全量与各档大档，本来就不进每次的 `cargo test`）。
- 其中 I-3.11 的计数（checker 每次判定都记）：随机历史快档 3618 次判定里判绿 3605、不适用 13、红 0；偏回退档判绿 1239、不适用 273；偏复用档判绿 880、不适用 40；两档墙取样判绿 229 / 3210；崩溃注入四段判绿 15 / 64 / 78 / 102。层 0 两条流的快档（`second_transaction_step_zero_layer0`、`second_transaction_step_three_formatted_pool_layer0`）与第一个事务第 7 步层 0 快档（`first_transaction_step_seven_layer0`）都过，「必须评估到」清单里加上的 I-3.11 在这几条流上真被评估过。
- 03:14–03:25 在 `mutant/`（与 `sync/` 逐文件相同）上复跑的两个基线：`checker_known_bad_images` 25 过 0 红，`second_transaction_supplement_three_random_history` 18 过 0 红 2 忽略。

## 十、`check.sh`

`sync/`（`crates/` = 主工作区现状 + 补丁，其余文件 03:13 从主工作区同步）上 03:14 跑 `CARGO_BUILD_JOBS=4 nice -n 19 bash .claude/scripts/check.sh`，末尾原样：

```text
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 1 previous error
  ✗ clippy 有告警（按 -D warnings 视为错误），或者踩了编码纪律的某一条
     → 怎么办： 上面每条告警都指着文件和行号，逐条改。编码纪律那几条的写法见 rules/code-discipline.md。
                确有必要保留的，在那一处写 #[allow(<lint>, reason = "为什么")]，理由写进 reason——
                不要整仓关掉 -D warnings（rules/command-safety.md：警告是最便宜的信号）。
exit=1
```

红在 clippy 那一步，告警全落在两份实验编号的 bin 上：`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 4 条（`shadow_unrelated`：`mounted`、`release_generation` 两处、`before`）、`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 1 条（`items_after_test_module`）。两份在主工作区里都是未跟踪文件（`git status --short` 报 `??`），不在这份补丁里（补丁里两个文件名零命中），不归我修。`check.sh` 在 clippy 那一步就退出，后面几步没走到，我在同一份 `sync/` 上分开补跑：

| 补跑的 | 命令要点 | 结果 |
|---|---|---|
| clippy，除那两个实验 bin 之外的全部目标，同一组 `-D` | `cargo clippy --workspace --lib --test '*' --bin first_transaction_device_log_check --bin first_transaction_on_device --bin first_transaction_region_bytes --all-features`；另 `--workspace --exclude singlefs-harness --all-targets`；另 `-p singlefs-harness --profile test --lib` 加那三个 bin | 三次都退出 0（`logs/round3-check/clippy-*.log`） |
| `cargo build --all-targets` | 同 `check.sh` | 退出 0（`logs/round3-check/build-all-targets.log`，03:26） |
| `cargo test --all` | 同 `check.sh`（02:37–02:59 那一轮，第九节） | 退出 0 |
| `cargo fmt --check` | `check.sh` 第一步 | `✓ 格式通过` |

## 十一、门禁阶段

登记给 `implementation-writer` 的七个（`33-mutation-tables.sh`、`53-format-const-placeholders.sh`、`74-model-differential.sh`、`92-layout-checker-sync.sh`、`94-checker-implementation-disjoint.sh`、`93-feature-bits.sh`、`89-closeout-row27-preconditions.sh`）这一次没跑：按主 agent 要求留到最后统一跑（01:49 追加的第 ③ 条，接手时的派发也这么说）。

## 十二、主工作区在 03:25 与 04:02 又变了两次之后的复核

我接手之后主工作区又打进了两批（03:25 前后：`allocator.rs`、`recovery.rs`、`transaction.rs`、`crash.rs`、`history.rs`、`model_comparison.rs`、`mutations.tsv`，另加两个新测试文件；04:02 前后：`mount.rs`、`admission.rs`、`journal.rs` 等十个源文件、二十多个测试文件与 `mutations.tsv`，另加两个新测试文件）。两次补丁都照原样打得上，没有冲突要重做；`walk.rs` 与 `checker_known_bad_images.rs` 这两次都没被别人动过（打上补丁之后与 `sync/` 那份 `cmp` 相同）。

| 主工作区哪一版 | 副本 | 做了什么 | 结果 |
|---|---|---|---|
| 03:27（`base-crates-now2/`） | `sync2/` | `git apply` 补丁；现有 441 行变异锚点与新 6 行重核 | 现有 441 行都恰好命中一次，新 6 行各命中 1 次、名字不重 |
| 同上 | `sync2/` | 跑 21 个会跑 checker 的测试二进制（`logs/round4/checker-binaries.log`，03:27–03:46） | 退出 0；165 过、0 红、7 忽略；I-3.11 各档计数与第九节逐项相同 |
| 同上 | `mutant/`（与 `sync2/` 逐文件相同） | 第五节、第六节的 6 条变异整轮复证（`drafts/run_mutation_proofs_round5.sh`，`logs/round5/`，03:47–04:02） | 两个基线 25 过 0 红、18 过 0 红；6 条都红，点名的测试、红在哪条断言、同时红的条数都与第五、六节逐条相同（C493 那条红在第 246 行，新发现原话同样是 `实现 拒了：MountError::RollbackTargetNotACandidate(NotInRing)`）；跑完 `diff -rq sync2/crates mutant/crates` 退出 0 |
| 同上 | `sync2/` | `check.sh`（`logs/round5-check/check.log`，03:47） | 这一次红在第一步 `cargo fmt --check`，唯一一处 `Diff in …/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:2799:`，不在补丁里；补跑 clippy（除两个实验 bin，三次）与 `cargo test -p singlefs-checker` 都退出 0（`logs/round5-check/supplement.log`） |
| 04:03（`base-crates-now3/`） | `sync3/` | `git apply` 补丁；现有 457 行与新 6 行锚点重核 | 现有 457 行都恰好命中一次；新 6 行各命中 1 次（C493 那行移到 `mount.rs` 第 1454 行） |
| 同上 | `sync3/` | 补丁动到的 6 个测试二进制 + `singlefs-checker` 单测（`logs/round6/touched-binaries.log`，04:03–04:09） | 都退出 0：25 / 5 / 11 / 3 / 8 / 18 过，0 红；I-3.11 随机历史各档计数同第九节 |

04:03 那一版上没有再跑变异复证与 `check.sh`（主工作区一直在变，按「不因产物变化无限续做」停在这里）。04:09 最后一次核（这之后主工作区 `crates/` 只有 `e156_allocation_basis_counts.rs` 一处又变了，不在补丁里）：

```text
$ git -C /home/fy5090/code/singlefs apply --check /tmp/claude-1000/impl-m2-checker2/impl-m2-checker2.patch; echo "apply-check-exit=$?"
apply-check-exit=0
```

`crates/mutations.tsv` 的现有行数随别人追加一直在涨（435 → 441 → 457）；6 行追加在末尾就行，不改别人的行。

**负载**：03:13 `ps` 看到一个别的会话的 `cargo test -p singlefs-harness --test second_transaction_step_zero_layer0`（pid 3985984），03:26 看到一个 `cargo test --release --bin e159-fsync-wait-group-commit` 与一个 `cargo test … --test m2_newq_attack4 --no-run`；没有性能测量进程。我的副本各用各的 target，没等过锁；机器负载 03:19 到过 53。

## 十三、停下交主 agent 的设计问题

1. **I-7.9 的定义要改成按抬 F 那一刻判，还是交给抬 F 时的模型对拍**（第七、八节）。这一条定之前，I-7.9 不进 `IMPLEMENTED_INVARIANTS`，草稿补丁不交付。

没有别的设计问题：I-3.11 与 C493 那条变异都照条款原文做，没加条款没写的分支。I-3.11 判定里「两行任一行缺就判红」这一处照 `invariants.md` 第 137 行「等于」字面，与 I-3.1、I-5.2 同一个写法。

## 十四、没做什么

- 没走三方对抗；层 0 全量、QEMU、herd7、变异整表（门禁 59 号全表）都没跑，层 0 与 crates 变异表归 `crash-verifier`；没提交。
- 登记给我的七个门禁阶段没跑（第十一节）。
- `check.sh` 两次都没跑到底：03:14 那一次停在 clippy，03:47 那一次停在 fmt，两次都卡在不归我的实验 bin 上（第十、十二节）；后面几步是分开补跑的。04:03 那一版上没跑 `check.sh`，也没做变异复证。
- 完整的 `cargo test --all` 只在 02:36 那一版上跑过（第九节）；03:27 那一版跑了会跑 checker 的 21 个二进制，04:03 那一版只跑了补丁动到的 6 个二进制。
- I-7.9：随机历史两个种子与证据用例的读数是前一个实现员在主工作区五份补丁打进之前的代码上量的，只有固定用例与坏镜像在 03:17 的现状 + 草稿上复跑过；第八节那种改法没实现、没验。
- `invariants.md`、`checks-owed.md`（C493 那一行状态）、里程碑收口表都没改（不写 kb）；C493 还清要写的措辞没起草，只有变异行与第六节的证据。
