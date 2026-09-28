# 实 C554 乙-模型（模型跟上「重读一次仍读不出就拒可写」）+ 实审 B3c-4（零单元发布按指针往下带着比）——实现员报告

规格 `/tmp/claude-1000/impl-c554-yi-model/spec.md`。交补丁：在副本 `work/` 里改，主工作区一个字没动；补丁在 `patch/`。
副本取于 2026-09-27Z（`rsync -a --exclude target --exclude .git`，主工作区 HEAD `faf255e2`，B3c-3 补丁已在）；取副本那一刻 `crates/` 原件的逐文件 sha256 在 `base-sha.txt`。
这份报告分两段写成：前一个实现员写到撞会话限额停下；接手的实现员随即核现场、把补丁按主工作区现状重新生成、在现状加补丁的副本上重跑全部交回前验证、量第二件第 2 条那几种历史各几版，补齐第六节，改了第二、三、四、七、八节里跟底座有关的句子。

## 一、结论

1. **第一件做完**（C554 乙的模型跟进，走 Q2 第 ① 条）：
   - `model.rs` 新理由 `ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread`；`answer_mount_writable_with_the_newest_root_unreadable` 答「拒可写、一次发布都不做、取号之前判完」，根环只有一条根那一支的 `NotModeled` 照留。
   - `model_comparison.rs` 的 `refusal_reason_of_mount_error` 把 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)` 映射成这条理由；另一样（`InstanceTableOfTheNewestRootForTheShadowLedger`）照旧 `Unexplained`，两样分开。
   - 执行器 `history.rs` 那一步不用改代码：拒了走的是可写挂载共用的 `settle_mount` 的 `Err` 臂，也就是 `apply_mount_writable` 被拒时那一条。它按成员映射出的理由、录制流有没有多出写，同模型比；会话在挂载之前就关了，拒之后照旧没有会话，后面的步照「没有可写会话」走。只改了两处文档注释。
   - 随机历史 `:1082`、`:1135` 两条改用 `tests/common` 的帮手造被抛弃的根，都改了名。前一条照规格红在 I-7.4（抬 F 之前，乙罩不到的「系统配置没见证到」那一形），后一条绿。快档那段「每条路径都跑到了」的断言跟着改了，见第七节 Q2。
   - `checker_known_bad_images.rs` 那条中间实例行的坏镜像不再经可写挂载，改走事务层入口直接写，最后的断言一字没改，绿。见第七节 Q3。
   - 门禁 74 号在副本上改前 3 红、改后 1 红：快档 46 段「模型说该成、实现拒了」归零（96 段跑完、新发现 0）；剩下那一条是 `:1082` 新名，按规格就该红。`fault_injection` 快档从红变绿（12 过 1 红 → 13 过 0 红，新发现 30 → 0，测量跑提前停 13 段 → 0 段）；`--lib` 从 89 过 1 红变成 91 过 0 红。
2. **第二件做完**（B3c-4，走乙）：
   - `model_comparison.rs` 新加三样：`ContentsCarriedToZeroUnitPublishes`、`observed_root_of_version_without_file_carrying`、`observed_mount_carrying_to_zero_unit_publishes`。零单元发布的根里实例表指针、分配记录树根指针与同一次挂载里前一版逐字段相同时，拿前一版交回的 `units` 与分配记录，照写行那一版的判法比整张实例表与每个角色的分配代；有一条不同就判红，实例表那一条报「这一版的实例表」，分配记录树那一条走 `model.rs` 新臂 `AllocationRecordTreeRootChangedByAZeroUnitPublish`、报「单元的分配代」，都不当比不了。
   - `history.rs` 的 `WritableSession` 多一项 `carried_to_zero_unit_publishes`：挂载做成时存下那一份，`PublishWithoutUnits` 从它带；第一个文件版本发出之后清成 None。
   - 新测试文件 7 条。先在桩（不往下带）上跑：6 红 1 绿，红的 6 条都红在「比不了」的计数或 `expect_err`；实做之后 7 条全绿。
   - 一次挂载里第一版就是零单元发布的，照旧走比不了的两臂，只有 mkfs 之后第一次可写挂载会这样，次数见第四节。
   - 那几种历史各几版（第四节表）：比不了的零单元版只出在 mkfs 之后第一次可写挂载（快档 53 次、复用 6 次、回退 6 次，每次 2 版）与接在它后面、第一个文件发出之前实例 1 会话里的 `PublishWithoutUnits`（10、2、1 版），逐段加起来与报告行的数相等；三个墙取样段 0 版。
   - 在主工作区现状加补丁上重跑了全部交回前验证（第六节）：结局与前面相同，只多红 `checker_known_bad_images` 第 4527 行那条，不打补丁同样红，是 A3a 带来的（第七节 Q8）。
3. **变异**：追加 19 行、删 3 行（第 27、800、801 行的名字，原因见第五节），证红结果见第五节。B3c-3 那 13 行点名的用例没改名，全部复证。
4. **推翻条件**：
   - 主工作区打上补丁之后，门禁 74 号在快档报出 `ModelDisagreement` 的任何签名，结论 1 的「快档归零」就不成立。
   - 新测试文件在补丁之后有一条红，结论 2 就不成立。
   - 打上补丁后，随机历史里零单元发布报「这一版的实例表 / 单元的分配代」对不上而实现没错，说明往下带的判法与实现的口径不一致。

## 二、写过的文件（都在副本 `work/` 里；补丁按这些生成）

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-harness/src/model.rs` | 新理由 `ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread`（第 314 行起），`name` 与两处穷举 match 补臂；`answer_mount_writable_with_the_newest_root_unreadable`（第 1255 行）改答「拒可写」，文档注释改写；`ObservedUnitAllocationRecords` 新臂 `AllocationRecordTreeRootChangedByAZeroUnitPublish { what }`（第 483 行）与 `judge_allocation_generations` 里它那一臂（判「单元的分配代」）；`RewrittenRolesOnly` 的文档补一句；新单测 `crash_recovery_that_cannot_read_the_witnessed_newest_root_is_refused_before_any_write_and_leaves_the_model_unchanged`（第 2973 行）与帮手 `refusal_of_the_one_reread` |
| `crates/singlefs-harness/src/model_comparison.rs` | `refusal_reason_of_mount_error` 里 `NewerStateStillUnreadableAfterOneReread` 按两样分开映射（第 742 行），导入 `StillUnreadableAfterOneReread`；B3c-4：`ContentsCarriedToZeroUnitPublishes`（第 418 行）、`observed_root_of_version_without_file_carrying`（第 435 行）、`observed_zero_unit_publish_carried_from_the_version_before`（第 466 行）、`observed_root_of_pool_version_carrying`、`observed_mount_carrying_to_zero_unit_publishes`（第 536 行）；`observed_mount` 文档写明它不往下带；新单测 `only_the_witnessed_newer_publish_still_unreadable_after_one_reread_maps_to_the_models_reason`（第 1040 行） |
| `crates/singlefs-harness/src/history.rs` | `HistoryOperation::CrashRecoveryAbandoningTheNewestRoot` 与 `apply_crash_recovery_abandoning_the_newest_root`（第 3320 行）的文档注释按乙改写，代码不动；`WritableSession` 新项 `carried_to_zero_unit_publishes`（第 756 行）与三处建 / 拆会话；`settle_mount`（第 3059 行）改调 `observed_mount_carrying_to_zero_unit_publishes`、把带到最后的那一份交给会话；`apply_publish_without_units`（第 2904 行）从会话带、再存回；`settle_file_publish` 发第一个文件之后清成 None；报告多打一行「模型比每一版的实例表：比过 N 张、输出不带整条链比不了 M 张；零单元发布只比重写的角色 K 次；比过文件内容 J 次」（第 2061 行，另起一行，不动门禁 74 号认的那一行） |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` | 加 `mod common;`；`assert_every_path_was_exercised`（第 126 行）：崩溃恢复抛弃根那一类改要「至少被拒过一次」，成员清单加乙那一拒，删「回退目标落到过崩溃恢复抛弃的根」一条（改成注释，写明由哪条写死的用例钉）；`:1082` 改名 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`（第 1188 行），`:1135` 改名 `rolling_back_to_a_root_crash_recovery_abandoned_without_a_system_configuration_witness_is_refused_on_the_abandoned_timeline`（第 1256 行），两条照 `second_transaction_step_five_reuse.rs` 的同形用例改用 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`；新帮手 `pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version` 等四个；上一条用例文档里指向它的那句改成点名 |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（第 4676 行）：实例 3 不再经 `mount_writable`，改成 `acquire_instance` + `publish_instance_table_on_version_without_file`（行由用例拼：`(1, 实例 1 最新根的 txg, 0)`、`(2, 0, 0)`，txg / jsn / 水位照可写挂载的取法）+ 零单元暖机到两块盘都有实例 3 的根；账照可写挂载在实例 1 那一版上重建的样子新建（`PoolAllocator::new` + `mark_format_time_units`）；从盘上读回实例 3 最新根的实例表，钉它就是那两行（第 4846 行）；最后的断言不动 |
| `crates/singlefs-harness/tests/zero_unit_publishes_are_compared_with_what_the_row_publish_of_the_same_mount_handed_in.rs` | 新测试二进制，7 条（第四节） |
| `crates/mutations.tsv`（经 `patch/mutations-append.tsv`、`patch/mutations-delete.txt`） | 追加 19 行，删 3 行（第五节） |

`tests/common/mod.rs` 没改：帮手照原样用得上。`transaction.rs` 没动（规格约束）。

`patch/crates.patch` 接手后按主工作区当时的样子重新生成过（这 6 份文件的主工作区原件对 `rebased/` 里改好的那份逐个 `diff -u`），各段没有偏移；内容与前一版补丁打出来的结果逐字节相同（`rebased/` 就是用前一版补丁打出来的）。之后再核：主工作区这 6 份原件打上补丁，与 `rebased/` 里验过的那份逐个 `cmp` 相同。
补丁的统计（`git apply --stat patch/crates.patch`，在主工作区跑，原样）：

```text
 crates/singlefs-harness/src/history.rs             |   62 ++-
 crates/singlefs-harness/src/model.rs               |  155 ++++++-
 crates/singlefs-harness/src/model_comparison.rs    |  225 ++++++++++-
 .../tests/checker_known_bad_images.rs              |  128 +++++-
 ..._transaction_supplement_three_random_history.rs |  321 +++++++++++----
 ..._the_row_publish_of_the_same_mount_handed_in.rs |  425 ++++++++++++++++++++
 6 files changed, 1164 insertions(+), 152 deletions(-)
```

`git diff --stat -- crates litmus` 在主工作区上报的是别的会话的改动（这一件没碰主工作区），不代表这一件，不贴。

## 三、第一件：先红、改后、门禁 74 号改前改后

跑法照 C554 乙报告第三节（74 号不是重型）：`cargo test --release --offline -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture`，经 `capped.sh 4` 与 `run-with-memory-cap.sh 8G`；`repo/` 是取副本那一刻的原样（改前），`work/` 是改后。74 号本身也各跑一次（`SINGLEFS_GATE_FULL=1 … bash .claude/gate.d/74-model-differential.sh <副本>`）。日志：`logs/base-random-history-release.log`、`logs/final-random-history-release.log`、`logs/gate74-before.log`、`logs/gate-74-final.log`。

**先红（改前原样）**：
```
历史 96 段：跑完 50、以已知红收尾 {}、新发现 46；根环转过一圈的 43 段；最高 txg 44；一版里最多 746 条分配记录
新发现 ModelDisagreement { aspect: "模型说该成、实现拒了" }：第一个种子 7463871032432355114（同签名的种子 [7463871032432355114, 7463871032432355115, 7463871032432355116, 
test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 74.67s
```
`:1135`（旧名 `rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline`）改前红在第 2 步：`NewFinding { signature: ModelDisagreement { aspect: "模型说该成、实现拒了" }, … implementation_answer: "拒了：MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)" …`（`logs/base-random-history-release.log` 第 282 行起）。`:1082` 旧名改前红在 `second_transaction_supplement_three_random_history.rs:1105` 的断言「崩溃恢复抛弃根那一步是一次做成的可写挂载、取号 2：Refused { member: "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)" }」。

**只改模型（第 1、2 条）之后**（`logs/work1-random-history-release.log`）：快档 `历史 96 段：跑完 96、以已知红收尾 {}、新发现 0`，快档用例却红在第 116 行 `CrashRecoveryAbandoningTheNewestRoot 一次 Ok 都没有`——那一步乙之后一律被拒（`Ok 0、Err 53、前提不满足没调 15`），「回退目标落到过崩溃恢复抛弃的根」也不再造得出。这是第七节 Q2 那一处断言改动的由来。

**两件都改完之后**（`logs/final-random-history-release.log`，原样）：
```
历史 96 段：跑完 96、以已知红收尾 {}、新发现 0；根环转过一圈的 61 段；最高 txg 44；一版里最多 746 条分配记录
  操作 CrashRecoveryAbandoningTheNewestRoot：Ok 0、Err 53、前提不满足没调 15
  Err 成员 MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)：53 次
test result: FAILED. 23 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 166.55s
```
红的那一条是 `:1082` 新名，红法原样：
```
thread 'crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43' (3483742) panicked at crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:1201:5:
assertion `left == right` failed: 抬 F 之前一条违例都没有
  left: [("I-7.4", "被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50184 的那一份与位置条目里的校验和对不上；…
```
与 `second_transaction_step_five_reuse.rs` 同形那条的红法相同（`logs/base-step-five-row43.log`：`抬 F 之前一条违例都没有：[… ("I-7.4", Violated("被抛弃的根（实例 1、txg 5）引用的单元已被重新分配…`）。它不红在乙的拒上。

**剩下的签名**：六段取样点改后都是 `新发现 0`，没有要收缩的种子（改前那 46 段的签名就是规格要归零的那一个，最短复现见 C554 乙报告第三节）。

**门禁 74 号**：改前 `test result: FAILED. 21 passed; 3 failed`、`exit=1`；改后（两件都改完）`test result: FAILED. 23 passed; 1 failed`、末行 `  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）`、`exit=1`。在主工作区现状加补丁的副本 `rebased/`（见第六节）上又跑一次，结局相同：`test result: FAILED. 23 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 93.36s`，红的只有 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`，末行 `  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）`、`exit=1`（`logs/rebased-gate-74.log`）。同一副本上单跑这个二进制（`logs/rebased-random-history-release.log`）快档那一段原样：`历史 96 段：跑完 96、以已知红收尾 {}、新发现 0`、`操作 CrashRecoveryAbandoningTheNewestRoot：Ok 0、Err 53、前提不满足没调 15`，`:1082` 新名红在第 1201 行「抬 F 之前一条违例都没有」、`left: [("I-7.4", "被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头…`。
74 号跑整个二进制，`:1082` 按规格红着，所以 74 号在 `:1082` 转绿之前（C554 的「系统配置没见证到」那一格修掉）一直是红的，见第七节 Q4。

**`second_transaction_supplement_three_fault_injection` 快档（debug，不带 `--ignored`）**，跟 C554 乙报告第 214 行比：那一行是「12 passed; 1 failed; 1 ignored（基线同样 12 / 1：快档新发现 今天 38、乙 30）」；这一件的基线（`repo/`）同样是 `12 passed; 1 failed`，新发现 30、测量跑就提前停的 13 段，签名 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`。改后 `test result: ok. 13 passed; 0 failed; 1 ignored`，`历史 24 段：测量跑就提前停的 0 段、摆不出注入点的 0 段；注入 96 次`，`以「已知红」收尾 {}、新发现 0`（`logs/final-fault-injection.log`）。

## 四、第二件：往下带的比法、先红、哪几种历史还落在「比不了」

**比法**（`model_comparison.rs` 第 435 行起）：一版树表 0 条的发布交回的分配记录是 `WrittenIntoTheAllocationRecordTreeByThisPublish`（写行那一次）时，照 B3c-3 的判法比，并把它的 `units`、全部分配记录连同它的根存成 `ContentsCarriedToZeroUnitPublishes`。零单元发布（`SameAsThePreviousVersionNotHandedInByAZeroUnitPublish`）手里有这一份时，分三种情形：
- 根里 `instance_table`、`allocation_record_tree_root` 两条指针与存着的前一版的根逐字段相同：实例表拿带下来的 `units` 解（`observed_instance_table_written_by_a_row_publish`），分配代拿带下来的记录按这一版的根找每个角色（`allocation_records_of_every_role`），两个方向都比，判法与写行那一版相同；之后接着带同一份，存的根换成这一版的。
- 实例表指针不同：交回 `ObservedInstanceTable::Undecodable`，写明两条指针，模型报「这一版的实例表」。
- 分配记录树根指针不同：交回 `ObservedUnitAllocationRecords::AllocationRecordTreeRootChangedByAZeroUnitPublish`，模型报「单元的分配代」。

有指针不同之后就不再往下带（那一步已经判红，历史停在那里）。手里没有这一份的零单元发布照旧交比不了的两臂。
执行器里：`settle_mount` 用 `observed_mount_carrying_to_zero_unit_publishes`，写行 → 暖机在同一次挂载里往下带，带到最后的那一份交给会话；`apply_publish_without_units` 从会话带、再存回。
比的全是实现交出来的东西，不另读盘，也不从挂着的分配器取。

**先红**：新测试文件先在「接口在、零单元发布那一臂不往下带」的桩上跑（`logs/b3c4-red-first-stub-2.log`：`test result: FAILED. 1 passed; 6 failed`）：

| 用例 | 桩上红在哪 |
|---|---|
| `the_warm_up_after_the_row_publish_of_the_same_mount_is_compared_by_the_whole_instance_table_and_every_role` | 第 191 行「暖机那一版不再走「比不了」的两臂」`left: (1, 1)` `right: (0, 0)` |
| `a_wrong_row_in_the_instance_table_carried_to_the_warm_up_is_reported` | `expect_err("带下来的实例表错了一行")` 拿到 `Ok`（计数里 `instance_tables_not_in_the_output: 1`） |
| `a_wrong_allocation_generation_carried_to_the_warm_up_is_reported` | `expect_err("带下来的分配代错了一条")` 拿到 `Ok`（`rewritten_role_sets_compared: 1`） |
| `a_zero_unit_publish_whose_root_names_another_instance_table_than_the_version_before_it_is_reported` | `expect_err("零单元发布改了实例表指针")` 拿到 `Ok`（`instance_tables_not_in_the_output: 1`） |
| `a_zero_unit_publish_whose_root_names_another_allocation_record_tree_than_the_version_before_it_is_reported` | `expect_err("零单元发布改了分配记录树根指针")` 拿到 `Ok`（`rewritten_role_sets_compared: 1`） |
| `the_session_carries_the_row_publish_down_to_the_zero_unit_publishes_after_the_mount` | 「比不了的只剩第一次挂载那两版」`left: (5, 5)` `right: (2, 2)` |
| `a_mount_whose_first_version_is_a_zero_unit_publish_counts_every_version_as_not_in_the_output` | 桩上绿（它钉的就是改前改后都一样的计数，证红见第五节 B7） |

实做之后 `test result: ok. 7 passed; 0 failed`（`logs/final-new-b3c4.log`）。

**哪几种历史还落在「比不了」**：只有「一次挂载里第一版就是零单元发布」这一种，即 mkfs 之后第一次可写挂载（上一个实例是 mkfs 的 0，要写的行为空，取号之后第一次发布接的是从盘上恢复的 mkfs 那一版，没有单元字节），包括：
- 那次挂载的零单元发布与暖机：每次 2 版，新测试钉 `(2, 2, 0)`；
- 同一个会话里后面的 `PublishWithoutUnits`：会话里没有可带的，照旧计数。

之后只要挂载过一次（写行那一次交回了单元），这次挂载里的暖机与之后的零单元发布就都比。起点 `AfterFirstFile` 的历史一版树表 0 条的都没有。
随机历史各段改后的数（`logs/final-random-history-release.log`，新加的那一行，原样）：
```
── 随机历史：偏向抬 F 之后复用的取样点 ──
  模型比每一版的实例表：比过 1452 张、输出不带整条链比不了 208 张；零单元发布只比重写的角色 14 次；比过文件内容 1626 次
── 随机历史：偏向抬 F 之后回退的取样点 ──
  模型比每一版的实例表：比过 922 张、输出不带整条链比不了 344 张；零单元发布只比重写的角色 13 次；比过文件内容 1237 次
── 随机历史快档 ──
  模型比每一版的实例表：比过 1909 张、输出不带整条链比不了 488 张；零单元发布只比重写的角色 116 次；比过文件内容 2210 次
── 随机历史：小盘上逼近单元区墙的取样点 ──
  模型比每一版的实例表：比过 448 张、输出不带整条链比不了 331 张；零单元发布只比重写的角色 0 次；比过文件内容 779 次
── 随机历史：小盘上逼近单元区墙的取样点（空间准入判着） ──
  模型比每一版的实例表：比过 7823 张、输出不带整条链比不了 530 张；零单元发布只比重写的角色 0 次；比过文件内容 8353 次
── 随机历史：越过原分配记录墙的取样点 ──
  模型比每一版的实例表：比过 5012 张、输出不带整条链比不了 176 张；零单元发布只比重写的角色 0 次；比过文件内容 5188 次
```
读法：
- 「零单元发布只比重写的角色」那个数就是上面那种历史里树表 0 条的版数，每一版比不了的实例表也各算一次。
- 「比不了」里另有带文件的那两格：mkfs 之后第一次重写实例表之前的文件版本，从盘上重建、实例表多于一片的那一版。那是 B3c-3 报告第八节那一件，这一件不做。
- 快档、复用、回退三段抽的起点有 mkfs（`history.rs` 第 434、476、510 行），所以非 0；三个墙取样点只从第一个文件起（第 544、566 行），所以是 0。
- 「只比重写的角色」那个数拆成「挂载那两版」与「同一会话里的 `PublishWithoutUnits`」，报告行里分不出，下面另量了。

**各几版（接手之后量的，2026-09-27）**：在 `measure/`（`rebased/` 的另一份副本，只多三行 `eprintln!`，不进补丁）里逐段单跑六个取样用例（`measure.sh`，release、`--exact`、`--test-threads=1`），每次挂载打一行「实例号、写了几行、几版、几版比不了、带没带下去」，每次会话里的 `PublishWithoutUnits` 打一行「比没比」。日志 `logs/measure/<用例名>.log`，汇总 `logs/measure-table.txt`，六段都 `1 passed`。

| 取样段 | 挂载次数 | mkfs 后第一次可写挂载（实例 1、写 0 行） | 有「比不了」的挂载 | 会话里 `PublishWithoutUnits` 比不了 / 带着比 | 挂载 ×2 + 会话 | 报告行「只比重写的角色」 |
|---|---|---|---|---|---|---|
| 快档 | 396 | 53 | 53 | 10 / 3 | 106 + 10 = 116 | 116 |
| 偏向复用 | 267 | 6 | 6 | 2 / 1 | 12 + 2 = 14 | 14 |
| 偏向回退 | 122 | 6 | 6 | 1 / 1 | 12 + 1 = 13 | 13 |
| 越过原分配记录墙 | 605 | 0 | 0 | 0 / 0 | 0 | 0 |
| 小盘逼近单元区墙 | 52 | 0 | 0 | 0 / 0 | 0 | 0 |
| 同上（空间准入判着） | 324 | 0 | 0 | 0 / 0 | 0 | 0 |

读法：
- 有「比不了」的挂载全是 mkfs 之后第一次可写挂载，每次正好 2 版（写 0 行的写行那一版、暖机那一版）；实例号不是 1、或写了行的挂载，一版比不了的都没有（六段合计挂载 1766 次，`uncarried=0` 的 1701 次，其余 65 次全是「实例 1、写 0 行、2 版、2 版比不了」）。
- 会话里比不了的 `PublishWithoutUnits` 全在实例 1 的会话里，也就是接在那种挂载后面、第一个文件发出之前；实例 2 起的会话里 `PublishWithoutUnits` 都带着比了。
- 两类加起来与报告行的数逐段相等，「只比重写的角色」里没有别的来源。

## 五、变异：追加、删除与证红

证红一律 `bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-c554-yi-model/prove --memory 8G singlefs-harness <名字…>`（在主工作区里起那份脚本，`prove/` 是 `work/` 的另一份副本、自己的 target，变异表按补丁合并过；经 `capped.sh 4`，debug）。日志 `logs/prove-red/NNN.log`，汇总 `logs/prove-batch1.out`、`logs/prove-batch2.out`。每组参数先跑基线（不改源码）且绿，这是 prove-red 自己的第 ④ 步。被测代码里没有 `debug_assert`，红的都是测试自己的断言或 `expect`。prove-red 按行里的过滤串跑，同一份日志里「同时红了哪些」只看得到过滤进来的那几条：每条都只有点名的那一条红。表里的行号是变异施加之后那份副本里的行号。

**追加的 19 行**（名字原样；变异位置写在「改坏哪里」）：

| # | 变异名 | 改坏哪里 | 红的测试 → 断言（日志） |
|---|---|---|---|
| A1 | C554 乙 模型跟进：崩溃恢复抛弃根那一步模型答案翻回「该成」（不要求拒可写） | `model.rs` 那一步的 `required_refusals` 从 `{新理由}` 换成空集 | `--lib` `model::tests::crash_recovery_that_cannot_read_the_witnessed_newest_root_is_refused_before_any_write_and_leaves_the_model_unchanged` → `model.rs:2979`「系统配置见证过的最新那条根重读一次仍读不出：拒可写」（001） |
| A1b | C554 乙 模型跟进（随机历史快档）：崩溃恢复抛弃根那一步模型答案翻回「该成」（快档报「模型说该成、实现拒了」） | 同 A1 | 快档（见表后「第二批」） |
| A2 | C554 乙 模型跟进：胶水把「系统配置见证过更新的发布、重读一次仍读不出」映射成模型没有的理由 | `model_comparison.rs` 那一臂改成 `ObservedRefusalReason::Unexplained` | `--lib` `model_comparison::tests::only_the_witnessed_newer_publish_still_unreadable_after_one_reread_maps_to_the_models_reason` → `model_comparison.rs:1068` `assert_eq!`（002） |
| A3 | C554 乙 模型跟进：胶水不分那一拒说的是哪一样（实例表重读仍读不出也映射成「见证过更新的发布」那一条） | 实例表那一臂也映射成新理由 | 同一条 → `model_comparison.rs:1072` `assert_eq!`（003） |
| A6 | C554 乙 模型跟进：生成器的崩溃恢复抛弃根挂载期间一处都不藏（等于一次普通可写挂载，模型要求拒、实现做成了） | `history.rs` `for hidden in &self.hidden_ranges {` → `.iter().take(0)`（原第 800 行同一处） | 快档（第二批） |
| A8 | C554 乙 模型跟进（随机历史那条回退改用 common 帮手造被抛弃的根）：回退候选集不按实例表判（被抛弃时间线的根也能退到） | core `mount.rs` `if abandoned_by_table(&target_root, current_table) {` 前加 `false &&`（与第 29 行同一处） | 随机历史 `rolling_back_to_a_root_crash_recovery_abandoned_without_a_system_configuration_witness_is_refused_on_the_abandoned_timeline` → 第 1277 行「被拒成「在被抛弃的时间线上」：UserVisibleUnitWithoutItsRecordInTheCurrentAccount { … }」（004） |
| A10a | C554 乙 模型跟进（checker 那份中间实例行坏镜像改走事务层）：实例 3 写行漏交中间实例行 (2, 0, 0) | `checker_known_bad_images.rs` 用例拼的行去掉 `(2, 0, 0)` | `published_nodes_behind_…` → 第 4824 行「盘上实例 3 最新那条根的表里是实例 1 那一行与中间实例行 (2, 0, 0)」（005） |
| A10b | C554 乙 模型跟进（同上一份坏镜像）：实例 3 写行那次的树 ID 水位不照环里的记录取（取实例 1 那一版自己带的） | 写行那次的水位改成实例 1 最新根带的 | 同一条 → 第 4630 行「中间实例行后面的实例 2：干净镜像上一条都不许红」（006） |
| A11 | C554 乙 模型跟进：可写挂载只写上一个实例那一行、不写中间实例行（烧掉的号那一形护；崩溃恢复写 (2, 0, 0) 那一形乙之后拒可写、checker 那份改走事务层） | core `mount.rs` `(first_row_instance..instance_to_acquire.0)` → `..first_row_instance + 1`（原第 27 行同一处） | `second_transaction_supplement_two_c533_row_publish_record_without_its_root` 那一条 → 「重建出来的根指着 mkfs 那一片实例表，不是写行那次写的那一片」（007） |
| B1 | 实审 B3c-4（零单元发布从同一次挂载里前一版往下带着比）：零单元发布不从前一版带、照旧交比不了的两臂 | 有可带的那一臂退回不带 | 新文件 `the_warm_up_after_…` → 第 194 行「暖机那一版不再走「比不了」的两臂」（008） |
| B2 | 同前缀：零单元发布带下来的实例表不比（照旧报比不了） | 带下来的实例表换成 `NotInTheOutput` | `a_wrong_row_in_the_instance_table_carried_to_the_warm_up_is_reported` → 第 249 行 `expect_err("带下来的实例表错了一行")`（009） |
| B3 | 同前缀：零单元发布带下来的分配记录不比（照旧只比重写的角色） | 带下来的分配记录换成 `RewrittenRolesOnly(空集)` | `a_wrong_allocation_generation_carried_to_the_warm_up_is_reported` → 第 285 行 `expect_err("带下来的分配代错了一条")`（010） |
| B4 | 同前缀：零单元发布不核实例表指针与前一版相同 | `instance_table_pointer_carried = true` | `a_zero_unit_publish_whose_root_names_another_instance_table_…` → 第 310 行 `expect_err("零单元发布改了实例表指针")`（011） |
| B5 | 同前缀：零单元发布不核分配记录树根指针与前一版相同 | `allocation_record_tree_root_carried = true` | `a_zero_unit_publish_whose_root_names_another_allocation_record_tree_…` → 第 331 行 `expect_err("零单元发布改了分配记录树根指针")`（012） |
| B6 | 同前缀：模型把零单元发布换了分配记录树根指针那一臂放过 | `model.rs` 那一臂前插一个同模式、交回 `Ok(())` 的臂 | 同一条 → 同一处 `expect_err`（013） |
| B7 | 同前缀：一次挂载里第一版就是零单元发布、没有可带的也当比过一张空表（比不了的次数不再照计） | 没有可带的那一臂交 `Rows(空)` | `a_mount_whose_first_version_is_a_zero_unit_publish_counts_every_version_as_not_in_the_output` → 第 364 行「两版都比不了」（014） |
| B8 | 同前缀：挂载之后不把往下带的那一份交给会话（之后的零单元发布照旧比不了） | `history.rs` `settle_mount` 建会话时那一项给 `None` | `the_session_carries_…` → 第 413 行「比不了的只剩第一次挂载那两版」（015） |
| B9 | 同前缀：会话里的零单元发布不拿会话记着的那一份比 | `apply_publish_without_units` 交 `None` | 同一条 → 第 413 行（016） |
| B10 | 同前缀：往下带的那一份带过一版之后丢了单元（之后的零单元发布解不出实例表） | `carried_on` 里 `units: Vec::new()` | 同一条 → 第 390 行 `run.ending` 不是 `Completed`：`NewFinding { signature: ModelDisagreement { aspect: "这一版的实例表" }, … Operation(2), PublishWithoutUnits …`（017） |

**删的 3 行**（`patch/mutations-delete.txt`）：
- 原第 27 行「可写挂载只写上一个实例那一行、不写中间实例行（原为步 4 回退写行那一格；回退改成向前发布之后由崩溃恢复写 (2, 0, 0) 那一形护）」：它点名的 checker 用例不再经可写挂载写中间实例行，这一行留着 59 号必定「没红」；同一处原文与替换文换名字、改点 c533 那条，就是 A11。
- 原第 800 行「实七 生成器：崩溃恢复抛弃根挂载期间一处都不藏（…写死的那一段不以收口表第 43 行收尾）」：它点名的用例改名了，名字里说的「写死的那一段」也不在了；同一处改点快档，就是 A6。
- 原第 801 行「实七 模型：崩溃恢复抛弃根照环里最新那条根择（不去掉它）」：原文 `let chosen = view_without_the_newest.newest_root().clone();` 随这一件删掉了，锚点不在（留着 33 号红），它护的那一处由 A1、A1b 接。

**复证、没改的行**：
- `改法 D：T = 0 的行（恢复写的中间实例行）也排除…`（checker 那条用例改了造法）：抓到，第 4646 行「中间实例行后面的实例 2：水位压到 11 只红 I-7.8」（018）。
- B3c-3 的 13 行（点名的用例一条没改名）：13 条全抓到（019–031），红在哪与 B3c-3 报告第五节同一批断言，逐条见 `logs/prove-batch1.out`。
- `实七 生成器：抽到崩溃恢复抛弃根时生成的是一次可写挂载（快档里这一种操作一次都没有）`（快档那段断言改了）：第二批。

第一批原样末行：`✓ 点名 31 条：跑了 31 条，跳过 0 条，跑的都抓到了`、`batch1 exit=0`。

**第二批**（随机历史快档，debug；基线不改源码跑那条用例 `1 passed … finished in 205.85s`）：

| 变异 | 红的断言（日志） |
|---|---|
| A1b 模型答案翻回「该成」 | 快档第 283 行「「已知红」清单外的失败」：`历史 96 段：跑完 50、以已知红收尾 {}、新发现 46`，`新发现 ModelDisagreement { aspect: "模型说该成、实现拒了" }：第一个种子 7463871032432355114`——正是改前的样子（001） |
| A6 生成器一处都不藏 | 同一行：`新发现 46`，`新发现 ModelDisagreement { aspect: "模型说该拒、实现做成了" }：第一个种子 7463871032432355114`（002） |
| 原第 802 行 生成器把这一步生成成可写挂载 | 第 137 行「CrashRecoveryAbandoningTheNewestRoot 一次都没被拒过」（快档本身 `新发现 0`，红在改过的那段路径断言上）（003） |

第二批原样末行：`✓ 点名 3 条：跑了 3 条，跳过 0 条，跑的都抓到了`、`batch2 exit=0`。

两批一共 34 条（这一件追加的 19 条、复证的 15 条），都抓到；没有「只追加、没证、留给 59 号」的行。
日志说明：两批用的是同一个日志目录，prove-red 每次从 001 起编号，第二批的 001–003 盖掉了第一批的 001–003（A1、A2、A3）；那三条的红法是第二批跑之前从日志里抄进上表的，判定行原样还在 `logs/prove-batch1.out`。
底座说明：两批证红都跑在 `prove/`（`work/` 的副本，底座是较早取的主工作区，没有 A3a 在 core 七份读者上的改动）。接手之后没在现状上重证（派发写「已证红的不重做」）；A8、A11 两行改的是 core `mount.rs`，锚点在现状里各命中一次（门禁 33 号的 crates 表那一半在 `rebased/` 上没报，见第六节），红不红留给提交时的 59 号整表。

## 六、交回前的验证（主工作区现状加补丁）

副本 `rebased/`：接手后把主工作区 `rsync -a --delete --exclude target --exclude .git` 过来，`patch -p1 < patch/crates.patch`，变异表按 `patch/mutations-*.tsv|txt` 合并（`追加 19 行、删 3 行，原文各命中一次`，合并后 1271 行）。这一刻的主工作区与前一个实现员先前取的那份，只差 `e158_root_choice_repair.rs` 与 `crates/mutations.tsv`（`diff -rq` 查的），这 6 份文件一样。跑法 `verify-rebased.sh`（每条 cargo 经 `capped.sh 4`，跑测试的经 `run-with-memory-cap.sh 8G`），日志 `logs/rebased-<名>.log`；先前那一轮的旧日志挪进了 `logs/rebased-0456/`。开跑前 `ps`：别的会话在跑 `cargo test`（harness 的随机历史、bad_disk_input、step_five_reuse，e163 的 GPU bin，e161 的 bin），没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；负载 56（32 核），没等锁。

汇总（`logs/verify-rebased-summary.log`，原样）：
```
fmt: exit=1 
clippy: exit=0 
build: exit=0 
random-history-release: exit=101 test result: FAILED. 23 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 126.71s 
new-b3c4: exit=0 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.38s 
b3c3: exit=0 test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.47s 
checker-known-bad: exit=101 test result: FAILED. 38 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 342.54s 
harness-lib: exit=0 test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 348.54s 
fault-injection: exit=0 test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 447.25s 
```
逐样说：
- `cargo fmt --all -- --check` 退 1，只报 `Diff in /tmp/claude-1000/impl-c554-yi-model/rebased/crates/singlefs-harness/src/bin/first_transaction_on_device.rs:3059:`，不是这一件的文件（55 号那一路在改）。主工作区自己跑 `cargo fmt --all -- --check` 退 0（那份文件已经又改过，与 `rebased/` 里的不同）；这一件的 6 份文件（打上补丁之后）单独 `rustfmt --edition 2021 --check` 退 0。
- clippy（`--all-targets --all-features -- -D warnings` 加 `check.sh` 第 72–80 行那 7 条 `-D`）末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 18.60s`、`exit=0`；`cargo build --offline --all-targets` 末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 56s`、`exit=0`。
- 随机历史（release）：红的一条是 `:1082` 新名，按规格红在 I-7.4（第三节）。
- `checker_known_bad_images` 红的一条是 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount`（第 4527 行起），红在第 4572 行 `坏镜像照样可写挂载: Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })`。这条这一件没碰；主工作区先前取的那份不打补丁（`mainbase/`）单跑它同样红在这一处（`logs/mainbase-checker-two.log`：`test result: FAILED. 0 passed; 2 failed`，另一条是这一件改好的 `published_nodes_behind_…`，不打补丁时红在 C554 乙那一拒）。前一个实现员在开工时的底座上跑这个二进制是 39/0，所以是 A3a 那批 core 改动带来的，不归这一件，照写不修。
- `fault_injection` 快档、`--lib`、新测试文件、B3c-3 那份测试都绿；`--lib` 97 个（开工时的底座上是 91 个，多出来的 6 个是别的会话加的）。

登记给实现员的门禁阶段（在 `rebased/` 上跑，`SINGLEFS_GATE_FULL=1`；末行与退出码原样）：

| 阶段 | 末行 | 退出码 |
|---|---|---|
| 74 | `                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。`（上一行 `  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）`） | 1 |
| 33 | `    改完跑 bash research/scripts/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。` | 1 |
| 53 | `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` | 0 |
| 94 | `    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 \`crates/mutations.tsv\` 里，门禁 59 号复跑`（判定行 `  ✓ checker 与实现只共享常量模块 \`singlefs-format\`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；…`） | 0 |
| 93 | `  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 57 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；…）` | 0 |
| 89 | `「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐` | 77（本次未跑） |
| 92 | `  ! /tmp/claude-1000/impl-c554-yi-model/rebased 不是 git 仓，本阶段跳过` | 77（本次未跑） |

- 74 号红，只红在 `:1082` 新名（第三节、第七节 Q4）。
- 33 号红只有一处：`research/mutations/e163_gpu_multicard_crc32c.tsv 不存在（被测的是 research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs）`，是别的会话的实验，不归这一件。crates 表那一半这回没报（先前那一轮报的第 658 行 E158 锚点，主工作区后来改好了）。
- 89、92 退 77，按没判写。

补丁对主工作区：`git apply --check patch/crates.patch` 过；`python3 research/scripts/apply-writer-patch.py patch --dry-run` 原样 `✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1286 行`（主工作区的表这时已是 1270 行，又长了）。

## 七、停下交主 agent 的

**Q1 `observed_mount` 本身没往下带（B3c-3 那份测试不在我的文件单里）。** 同一次挂载里写行 → 暖机的往下带，放在新入口 `observed_mount_carrying_to_zero_unit_publishes`，执行器用它；`observed_mount` 照旧一次挂载单独看、不往下带。原因是 B3c-3 的 `the_row_publish_on_a_version_without_a_file_is_compared_by_its_whole_instance_table_and_every_role_both_ways` 直接调 `observed_mount`，钉着暖机那一版「比不了 1、只比重写角色 1」。`observed_mount` 一改，那两处钉值就要改成 0、比过的实例表与分配记录跟着变，而那份文件不在「要动的 crates 文件」里。
现在 `observed_mount` 只剩那份测试在用，B3c-3 的 13 行变异改的都是两个入口共用的内层函数，照旧全抓（第五节）。要不要把 `observed_mount` 并成往下带的那一个、改那两处钉值，交主 agent 定；并的话要把那份测试放进文件单。
推翻「共用内层、两个入口判得一样」的现象：同一份挂载输出，两个入口给写行那一版的 `ObservedRoot` 不同（现在都走 `observed_root_of_version_without_file`）。

**Q2 快档「每条路径都跑到了」那段断言跟着乙改了（第 126 行起）。** 乙之后「崩溃恢复抛弃根」那一步一律被拒（快档里 `Ok 0、Err 53`），这里有两处跟着改：
- 这一类从「至少一次 Ok」改成「至少被拒过一次」，成员清单加乙那一拒；原第 802 行变异（生成器把这一步生成成可写挂载）照样让它红，见第五节第二批。
- 删了「回退的目标落到过崩溃恢复抛弃的根、按被抛弃的时间线拒过」那一条。随机历史造不出被抛弃的根了，这一路由写死的 `:1135` 新名那一条（common 帮手的「没见证」那一形）钉，它另有 A8 证红。

这等于随机历史里「被抛弃的时间线」与影子账隔离被抛弃根那一路不再有随机覆盖；规格写「影子账那一路在随机历史里还有管理员回退在走」，管理员回退走的是现行时间线上的影子账（本来就不造被抛弃的根），这一格我没另量。
要不要再给生成器加一步造「没见证」那一形（Q2 的 ② 或 ③），规格明写这一件不加。

**Q3 checker 那份坏镜像没有「直接按字节」写，而是经事务层入口写。** 规格写的是「直接按字节写出中间实例行 (2, 0, 0) 与它后面实例 2 的码 2 节点，不经可写挂载」。现在的写法：
- 实例 2 的码 2 节点照旧由实例 2 真发一次第一个文件写出（「已发布节点」原样成立），它的根槽照旧清零；
- 实例 3 不走 `mount_writable`：用例自己取号，自己拼行 `(1, T, 0)`、`(2, 0, 0)` 交给 `publish_instance_table_on_version_without_file`（txg、jsn、水位照可写挂载的取法），再推零单元暖机；
- 用例从盘上读回实例 3 最新根的实例表，钉它就是这两行；
- 最后「只红 I-7.8、红在 15」的断言一字没改。

没有逐字节手造的理由：要手造就得同时造出实例 3 的根记录、实例表单元、这一版的分配记录树、journal 记录与系统配置轮换，等于在测试里重写一遍写行那次发布。
这条路不经可写挂载，C554 乙管不着它；写行本身仍经 `publish_instance_table_on_version_without_file`，那一段实现改了，这条用例跟着变。要不要改成真正的逐字节造法，交主 agent。
规格举的变异「字节造法漏写码 2 节点」在这个造法里没有对应的一处（节点是真发布写出来的），换成了 A10a（漏交中间实例行）与 A10b（水位不照环里的记录取）。原第 27 行（可写挂载漏写中间实例行）改点 c533 那条（A11）。

**Q4 门禁 74 号在 `:1082` 转绿之前一直红。** 规格要 `:1082` 新名「仍要红在收口表第 43 行那一形（I-7.4），不许红在乙的拒」，照做了。74 号跑整个随机历史二进制，只要这一条红着，74 号就红；规格开头那句「门禁 74 号绿是提交前的门禁」在 C554 的「系统配置没见证到」那一格（C554 乙报告第六节 Q1，乙-配置续在另一个实现员手里）修掉之前做不到。怎么处置（等续、标 `#[ignore]` 登记、或别的）交主 agent。

**Q5 会话里「最近那一份」只从树表 0 条的一版记。** 规格写「带文件的一版、写行那一版都算」。零单元发布只接在树表 0 条的一版后面：
- `apply_publish_without_units` 要现行版本是 `PoolVersion::WithoutFile`（`history.rs` 第 2919 行）；
- 带文件的一版之后再不会有树表 0 条的一版。

所以带文件的一版不记，发第一个文件之后清成 None；记了也用不上，也没有用例走得到。

**Q6 `history.rs` 多打了一行计数**（第 2061 行）：规格没要，是为了报出第四节那些数（五条硬要求第 4 条「分支必须可观测」）。它另起一行，门禁 74 号认的「模型对拍 N 步」那一行不动。不要就删那一段 `writeln!`，没有变异护它。

**Q7 乙-配置续（主 agent 来消息）**：模型不建系统配置的 journal tail，`model.rs` 只记取过的最大实例代号 `highest_acquired_instance`，取号那一写的 tail 模型里没有，今天也没答 0；续落地之后模型这边不用改。
续会动的是 `tests/common` 帮手那一形（「没见证」靠清掉见证槽造）在实现里的结局。`:1082`、`:1135` 两条新名都靠它，续之后要重跑这两条看结局变没变。

**另记（不是问题，是底座变了）**：
- 主工作区的 `history.rs` 在我取副本之后被别的会话改了：`RecoveryFailure` 的穷举 match 多了 `SystemConfigurationValueRefused`、`DeviceEndsBeforeTheUnitAreaStart` 两臂，+6 行，core 那边有对应改动。补丁已按现状重新生成，没有偏移了（第二节）。
- 第三到五节的验证跑在开工时的底座上，不含那批 core 改动；第六节在现状加补丁上重跑了一遍，结局除了 `checker_known_bad_images` 多红一条（A3a 带来的，不打补丁同红）之外都一样。证红没在现状上重跑（第五节末）。

**Q8 `checker_known_bad_images` 第 4527 行那条在主工作区现状红着，不归这一件**（第六节）。它在我的文件单里的那份测试文件里，但红因是 A3a 之后可写挂载在恢复阶段就判出「E142 走读同款」、不再让坏镜像挂上；要改的是那条用例的造法或断言，不是这一件的条款。交主 agent 派给 A3a 那一路。

## 八、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交；主工作区一个字没改（交补丁）。
- 重型测试一条没跑：名字带 layer0 的测试二进制、崩溃注入快档（标了 `#[ignore]` 的）、全量 `cargo test`。
  - 执行器 `history.rs` 改了会话与挂载那一处的比法，崩溃注入（`second_transaction_supplement_three_crash_injection` 等）也经这个执行器、也拿模型比。「崩溃恢复抛弃根」那一步的模型答案变了，零单元发布的比法变了，那几个二进制的结局与计数会跟着变，我没跑，留给提交时。
  - 规格点名的 `fault_injection` 快档跑了（第三节）。
- 补丁不动 `crates/singlefs-checker/src/`，没有「受影响的层 0 流与崩溃枚举用例」那一节的对象；没加层 0 流、没加崩溃点重放用例，`crash-case-check` 那一条无对象。
- 没改 `tests/common/mod.rs`、`transaction.rs`、B3c-3 那份测试文件（第七节 Q1）；没改 kb。
- 没量的：
  - 「比不了」那两个数里挂载那两版与同一会话 `PublishWithoutUnits` 各占多少；
  - 管理员回退在随机历史里走影子账到什么程度（第七节 Q2）；
  - 快档里原本就点名快档的那几行变异（第 111、122、127、128、130、131、267 行等，不是这一件的）。快档改前一直红，那几行在 59 号里一直是「白抓」；这一件之后快档绿了，它们要在 59 号里真红一次才算数。我没替它们证，只证了这一件的三行（第二批）。
- 门禁 92 号在副本上退 77（副本不是 git 仓），没判；89 号退 77（无对象）。
- 草稿副本都删了：`repo/`（改前基线）、`work/`（改的那份）、`prove/`（证红）、`gates/`（跑门禁）、`mainbase/`（不打补丁对照）、`rebased/`（现状加补丁）、`measure/`（量第四节那张表），以及 `base/`（开工那一刻的 `crates/` 源码快照）。补丁已对现状重新生成、验证已落进 `logs/`，再用得上的只有 `patch/`；要复现就按第六节那几行重新拷一份。
