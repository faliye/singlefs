# impl-m2-lastflag 报告（实二十：末条再跨记录、记录标志位 0「本次发布末条」、读法乙锚点、I-8.9、层 0 流 L8）

时刻一律 UTC（东京 = UTC+9）。

## 一、交付

| 东西 | 路径 | 说明 |
|---|---|---|
| 补丁（只含 `crates/`，不含 `crates/mutations.tsv`） | `/tmp/claude-1000/impl-m2-lastflag/impl-m2-lastflag.patch` | 对着 13:51:46Z 拍的主工作区现状（`base2/`）生成；主工作区上 `git apply --check` 退出码 0（14:4x 现跑，第五节）。21 个文件，新增 2 个测试文件 |
| 变异追加行 | `/tmp/claude-1000/impl-m2-lastflag/mutations-append.tsv` | 28 行，六段制表符分隔；第四节逐行说明证过哪几行、哪几行留给门禁 59 号 |
| 要删的主表行 | 主表 `crates/mutations.tsv` 第 438、440、441 行（主工作区 13:51Z 现状的行号，三行名字以「并行线一：P6 / C365：链首锚点取…」「并行线一：末条要点名的项多于 67 也照写…」「并行线一：点名项上限按整次发布的角色数判…」打头） | 打上补丁后原文命中 0 次；替代行是追加表第 12、8、7 行 |
| 草稿 | `repo/`（11:37Z 拍的底 + 改动，测试与变异都在它和它的两份副本上跑）、`rebased/`（`base2/` + 补丁，冲突手合过）、`mutant/`、`mutant2/`、`mutation-logs/`、`repo-tests-1/`、`tools/`、`progress.md` | 都在 `/tmp/claude-1000/impl-m2-lastflag/` 下 |

**补丁是两段底合出来的**：实现与测试写在 `repo/`（底 = 11:37:41Z 的主工作区，`base/`）。13:5x 发现主工作区又打进了别人的改动（实十四等：`image.rs` 多了 I-7.9、I-9.15，`formatted_pool.rs` 的四张不适用表，`walk.rs`、`recovery.rs`、`e158_root_choice_repair.rs` 等），原补丁在 `image.rs` 与 `second_transaction_step_three_formatted_pool.rs` 两处冲突。于是另拍 `base2/`，把原补丁打到它的副本 `rebased/` 上：两处冲突手合（`IMPLEMENTED_INVARIANTS` 43 → 44，I-8.9 排在 I-8.8 之后；`NOT_APPLICABLE_RIGHT_AFTER_MKFS` 21 → 22 加 I-8.9、注释「其余 23 条」→「其余 24 条」），`e158_root_choice_repair.rs` 里新出现的一处 `replay_journal` 调用加 `.expect`。交的补丁 = `base2/` 对 `rebased/` 的差。**测试与变异证红都是在 `repo/`（旧底）上跑的**，`rebased/` 上只跑了 fmt、clippy、`cargo build --all-targets` 与 `cargo check`；新底上的测试留给最后统一跑（主 agent 定）。

## 二、做了什么（按条款）

1. **记录标志 1**（D23 已定项 4 第一条、已定项 17）：`crates/singlefs-core/src/journal.rs` 第 24 行起 `JOURNAL_RECORD_FLAGS_OFFSET = 7`、位 0 常量、`JournalRecordPlaceInPublish`（第 32 行，`LastRecordOfThePublish` / `MoreRecordsOfThePublishFollow`，行号都是 `rebased/` 的），`JournalRecord` 多一个字段 `place_in_publish`。写者第 197 行写 `record_flags_byte()`（原来的 `put_u8(0); // 对齐填充` 去掉）；读者第 266 行位 0 之外有位为 1 ⇒ `None`、第 279 行序号 0 ⇒ `None`（「当损坏」取成与校验和不过同一个结局：这条记录不算在，按 jsn 断号即止）。第 21 行注释「填充 1」→「记录标志 1」，`checker_known_bad_images.rs` 那一处同改。
2. **每次只有一条记录的发布那一条也写 1**：`transaction.rs` 空发布（`publish_without_units`）与树表 0 条那一版的写行记录都写 `LastRecordOfThePublish`。w1、w4、t9 的这一字节因此变成 1。
3. **末条再跨记录**（已定项 17）：去掉 `PublishError::MoreNamedUnitsThanOneJournalRecordHolds` 与它在 `publish_version_of_trees` 里的拒绝（`harness` 的 `history.rs`、`model_comparison.rs` 两处匹配臂跟着删）。`roles_named_by_each_record_of_the_publish`（`transaction.rs` 第 2835 行）在原切法之后把最后一个事务那一项按 bump 次序切成每条至多 67 项（装满一条再开下一条；一个角色都不点名时照旧一条空记录）；新函数 `transaction_offset_of_each_record_of_the_publish`（第 2879 行）给出每条属于第几个事务：前 N − 1 条各是自己的事务，其余全属最后一个事务。写记录时（第 3849–3859 行一带）：事务号 = 首个事务号 + 所属事务序号，**提交标记只在一个事务的最后一条**（已定项 7；I-8.8 ③），**末条标志只在整次发布最后一条**。写序、`highest_transaction_number_in_this_instance`、tail（末条 jsn）照原口径。
4. **恢复按标志认发布边界、读法乙锚点、读者规则**（`recovery.rs`，`rebased/` 行号）：
   - `record_ends_its_publish`（第 1654 行）改成只看记录标志位 0（原来按「点名了码 1 之外的单元」启发式认）。
   - 锚点（新函数 `counter_of_the_last_record_the_root_covers`，第 1668 行）：所选根那次发布读得出的记录里带末条标志的那一条；一条都没有 ⇒ 当锚点读不出、走「序号 1 的第一条可读记录当链首」那一支（读法乙）；**多于一条 ⇒ 新错误成员 `RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`（第 198 行），条款没写，第六节第 1 条**。
   - `replay_journal`（第 1706 行）返回 `Result`；开着的一次发布里：换 txg 即断（原有）、**序号不是上一条 + 1 即断**（第 1775 行，一次发布之内跳号）、**上一条没带提交标记而这一条换了事务号即断**（第 1784 行，那个事务的提交标记没出现）；**带末条标志而不带提交标记的即断**（第 1793 行）；原来的「不带提交标记就停」去掉（跨两条记录的事务第一条不带提交标记，停在那里会把整次发布丢掉）。
   - 调用点：`recover` 把 `Err` 交成 `RecoveryOutcome::Failed`、`effective_root` 为 `None`；`mount.rs` 第 1702 行 `?`（`mount_writable`，实十八在改的文件，只动这一处）；`mounted_read.rs` 第 590 行 `map_err`；三处测试 / 装置里的直接调用加 `.expect`。
5. **checker 判 I-8.9**：`singlefs-checker/src/lib.rs` 的 `JournalRecordView` 多一个 `record_flags_byte`（偏移 7 自己写一份，第 503 行）；`walk.rs` 新函数 `judge_publish_ordinals_and_last_record_flags`（第 3137 行，入口第 3458 行调）逐盘按 (实例, checkpoint_txg) 成组判七条判据：标志其余位、序号 0、跳号（组里序号之差 ≠ 计数器之差）、不从 1 起（组里最小计数器的前一个计数器上坐着别组的记录而序号不是 1）、多于一条末条、末条之后还有记录、没有末条（组里最大计数器的下一个计数器上坐着**同一实例**的别组记录而组里没有末条标志）。只判读得出的记录；末条可能还没落盘的那几格（下一个计数器读不出或是别的实例）不判「没有末条」。环里一条自证过的记录都没有 ⇒ 不适用。`image.rs` 清单加 I-8.9。
6. **层 0 流 L8**：新测试文件 `second_transaction_parallel_line_three_spill_over_layer0.rs`，见第五节。

## 三、新测试与「证明会红」

变异在 `repo/` 的两份副本 `mutant/`、`mutant2/` 上跑（各自的 target，没有共用 `CARGO_TARGET_DIR`；每条命令 `capped.sh 4`，同时至多两份）。照主 agent 14:4x 前收窄的做法：**每条新测试挑一行能让它红的变异证一次；同一个测试二进制里互不重叠的几行一次改坏、跑一次整个二进制**（不带过滤，`--no-fail-fast`，debug）；跑完从 `repo/` 拷回原件并 `touch`，最后 `diff -rq repo/crates mutant*/crates` 为空。组、行、退出码、FAILED 的测试、起止时刻在 `mutation-logs/summary.tsv`，原样日志 `mutation-logs/<组>.log`。**基线红集**：同一份代码在 `repo/` 上先整二进制跑过（`repo-tests-1/summary.txt`，12:20–13:37Z，十个二进制全绿；core / checker / format 单测 13:3xZ 全绿），基线红集为空；`mutant*/` 与 `repo/` 逐字节相同（`diff -rq` 为空）。其中前五个二进制跑在两处纯改名之前（`transaction.rs` 单测里两个局部变量、`…many_inodes.rs` 里两个 `report` 局部变量，为过 clippy `shadow_unrelated`，行为不变），后五个与单测跑在改名之后。被改坏的几个文件里没有 `debug_assert`（`grep -n debug_assert` 零命中），红的都是测试断言。

| 新测试（文件） | 证它会红的行（追加表行号，组） | 改坏哪一行 → 哪条断言红（原样摘自日志） |
|---|---|---|
| `journal::tests::the_record_flags_byte_at_offset_seven_carries_bit_zero_for_the_last_record_of_the_publish_and_round_trips`（`journal.rs`） | 1（g1） | 写者 `writer.put_u8(self.place_in_publish.record_flags_byte());` → `put_u8(0)`：`journal.rs:413`「本次发布末条：记录标志位 0 = 1，其余位 0」`left: 0 right: 1`。同组同时红的有 journal 里另三条读回比对的用例（写者恒写 0，读回的 `place_in_publish` 对不上） |
| `journal::tests::a_record_whose_flags_byte_sets_any_bit_other_than_bit_zero_is_refused_by_the_parser` | 3（g1） | 读者 `from_record_flags_byte(reader.get_u8())?` → `(reader.get_u8() & 1)`：`journal.rs:441`「记录标志 0b00000010：位 0 之外有位为 1，当损坏」`left: Some(…) right: None` |
| `journal::tests::a_record_whose_ordinal_within_publish_is_zero_is_refused_by_the_parser` | 4（g1） | 读者 `if ordinal_within_publish.0 == 0 {` → `if false {`：`journal.rs:456`「序号 0：当这条记录损坏」`left: Some(…) right: None` |
| `transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction`（`transaction.rs`） | 7（g1） | `record_offset_in_this_publish.min(transactions_named_one_record_each)` → 去掉 `.min(…)`：`transaction.rs:4259`「跨出去的两条属于最后一个事务」`left: [0, 1, 2, 3] right: [0, 1, 1, 1]` |
| `a_publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish`（`…parallel_line_three_many_inodes.rs`，替掉原来那条「装不下就拒」的用例） | 6（g2） | `is_commit: is_the_last_record_of_its_transaction,` → `true`：`…many_inodes.rs:493`「(txg, 事务号, 提交标记, 本次发布内序号, 记录标志, 点名项数)…」`left: [(4, 2, 1, 1, 0, 67), (4, 2, 1, 2, 1, 1)] right: [(4, 2, 0, 1, 0, 67), (4, 2, 1, 2, 1, 1)]` |
| `when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record_is_no_anchor_and_the_next_publish_starts_the_chain`（新文件 `…parallel_line_one_last_record_flag.rs`） | 12（g5a） | 锚点改成读法甲（去掉 `&& record_ends_its_publish(record)`、取 `.max()`）：`…last_record_flag.rs:192`「B 的末条两份都撕了、第一条读得出：…C 整次施加」`left: (Some((1, 4)), 0) right: (Some((1, 5)), 3)` |
| `two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write`（同上） | 13（g5b，单独一组） | `if counters_carrying_the_last_record_flag.len() > 1 {` → `if false {`：`…last_record_flag.rs:255`「恢复在施加任何记录之前停下」`left: (FileRead { root: (1, 4), … }) right: (Failed { … RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { instance: 1, checkpoint_txg: 4, counters: [4, 5] } }, None, 0)` |
| `an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied`（同上） | 14（g5a） | 跳号那一判 → `if false {`：`:192`「C 的序号 1、3、3：一次发布之内跳号，C 整体不施加」`left: (Some((1, 5)), 3) right: (Some((1, 4)), 0)` |
| `a_record_whose_ordinal_is_zero_or_whose_flags_set_another_bit_counts_as_torn_and_its_publish_is_not_applied`（同上） | 3（g5a，与 g1 同一行、另一个二进制） | 读者不拒其余位：`:192`「C 的第二条记录标志位 1 为 1」`left: (Some((1, 5)), 3) right: (Some((1, 4)), 0)` |
| `a_transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied`（同上） | 15（g5a） | 「上一条没带提交标记而换了事务号」那一判 → `if false {`：`:192`「C 的事务 4 没有提交标记、下一条换了事务号」`left: (Some((1, 5)), 3) right: (Some((1, 4)), 0)` |
| `a_last_record_of_a_publish_without_a_commit_marker_keeps_its_publish_unapplied`（同上） | 16（g5a） | `if !record.is_commit && record_ends_its_publish(record) {` → `if false {`：`:192`「C 的末条不带提交标记」`left: (Some((1, 5)), 3) right: (Some((1, 4)), 0)` |
| `every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all`（新文件 `…spill_over_layer0.rs`，L8） | 17（g6） | 上一格那一判改回旧写法 `if !record.is_commit {`：`…spill_over_layer0.rs:202`「两条记录都落了、根没落：由记录整次施加这一版」`left: (Some((1, 3)), 0) right: (Some((1, 4)), 2)` |
| `one_publish_over_two_records_with_the_last_record_flag_on_the_second_holds_the_publish_ordinal_invariant`（`checker_known_bad_images.rs`，I-8.9 阳性对照） | 27（g7x，与 22 同组） | `} else if judged_records > 0 {` → `} else if false {`（有记录也报不适用）：`checker_known_bad_images.rs:1899`「每次发布一条记录：I-8.9 真被评估过且成立」`left: NotApplicable(…) right: Holds`；同组还红了 `the_clean_image_holds_every_invariant_…`（2176 行）与三条「干净镜像上每条都成立」的用例 |
| `each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria`（同上，七份坏镜像） | 26（g7y，单独一组）；22（g7x） | 26：「没有末条」那一判 → `if false {`：`:1962` 整表比对，唯一不同的一份 `left: { "没有末条", violated_invariants: [], reddened_criteria: [] } right: { …["I-8.9"], ["没有末条"] }`。22：跳号判据改成 `ordinal_distance != 0`：同一处，七份里六份多出「跳号」（例 `left: ["跳号", "多于一条末条"] right: ["多于一条末条"]`） |
| `root_slots_system_configurations_and_journal_ring_hold_the_published_state` 新加的一条断言（`first_transaction_step_five_publish.rs`） | 10（g3） | 空发布记录写 `MoreRecordsOfThePublishFollow`：`:499`「三次发布各只有一条记录：那一条就是本次发布末条…」`left: 0 right: 1` |
| `writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance` 新加的一条断言（`…step_three_formatted_pool.rs`） | 11（g4） | 写行记录写 `MoreRecordsOfThePublishFollow`：`:173`「写行那次发布只有这一条记录：它就是本次发布末条…」`left: MoreRecordsOfThePublishFollow right: LastRecordOfThePublish`；同组还红了另三条用例，都是池级 checker 的 I-8.9「没有末条」判出来的（写者与 checker 互相对上：写行记录不带标志、同一实例又往下写了暖机） |

**E142 钉的产物值**（`first_transaction_step_five_publish.rs` 的 `E142_BACK_CHAIN_OF_FIRST_TRANSACTION`，1057457588 → **3937788698**）不照实现的输出抄，照实十三的做法独立换算：在旧底的副本 `dump/` 里加一条草稿用例导出 jsn 1–3 三条记录（311 字节头、偏移 7 写 0，`dump-records/`），`tools/back_chain_with_last_record_flag.py` 自己实现 CRC-32C（先过 `123456789 → 0xE3069283`），先复现旧值当阳性对照，再把 w1、w4 的偏移 7 写 1、重算 w4 的反向链、算新 w4 头的 CRC32C。原样输出（`back-chain-recompute.out`）：

```
old_layout positive_control w4_back_chain_recomputed=823937809 on_disk=823937809 t9_back_chain_recomputed=1057457588 on_disk=1057457588 matches_pinned_1057457588=True
new_layout w4_back_chain=699178990 t9_back_chain=3937788698
```

新代码上钉 3937788698 的那条用例全绿（`repo-tests-1`，12:38–12:48Z）——实现写出来的与独立换算一致。E142 产物第 37 行要等实验执行员按新代码重跑才会是这个数。

## 四、`mutations-append.tsv`（28 行）与主表要删的三行

上表里的行号（`journal.rs:413` 这种）是 `repo/` 那一份的；`rebased/` 里同一段代码的行号见第二节。追加表每行的原文在 `rebased/` 源码里各命中 1 次（`tools/anchor_hits.py rebased mutations-append.tsv` 无输出）。

| 行 | 变异名（原样） | 状态 |
|---|---|---|
| 1 | 实二十：记录头写者不写末条标志（记录标志恒写 0） | 证过（g1） |
| 2 | 实二十：记录头读者把末条标志位 0 读反（带标志的读成不是末条） | 留给 59 号 |
| 3 | 实二十：记录头读者不拒记录标志其余位非 0（只看位 0） | 证过（g1、g5a） |
| 4 | 实二十：记录头读者不拒本次发布内序号 0 | 证过（g1） |
| 5 | 实二十：多条记录的发布每条都带末条标志 | 留给 59 号 |
| 6 | 实二十：末条跨出去的记录也各带提交标记（一个事务两条都带，I-8.8 ③） | 证过（g2） |
| 7 | 实二十：末条跨出去的记录各算一个事务（事务号各加一，替换第 441 行那条锚点已腐化的变异） | 证过（g1） |
| 8 | 实二十：末条再跨记录时不装满 67 项就开下一条（替换第 440 行那条锚点已腐化的变异） | 留给 59 号 |
| 9 | 实二十：一个角色都不点名的发布一条记录都不切出来 | 留给 59 号 |
| 10 | 实二十：空发布记录不带末条标志 | 证过（g3） |
| 11 | 实二十：树表 0 条那一版上写行那次发布的记录不带末条标志 | 证过（g4） |
| 12 | 实二十：锚点取读法甲——…（替换第 438 行那条锚点已腐化的变异） | 证过（g5a） |
| 13 | 实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链） | 证过（g5b） |
| 14 | 实二十：一次发布之内跳号不断链 | 证过（g5a） |
| 15 | 实二十：一个事务的提交标记没出现、下一条换了事务号也照接 | 证过（g5a） |
| 16 | 实二十：末条不带提交标记也整次施加 | 证过（g5a） |
| 17 | 实二十：层 0 L8：恢复在不带提交标记的记录上就停（…） | 证过（g6） |
| 18 | 实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去） | 留给 59 号 |
| 19 | 实二十：I-8.9 不判记录标志其余位 | 留给 59 号 |
| 20 | 实二十：I-8.9 不判序号 0 | 留给 59 号 |
| 21 | 实二十：I-8.9 不判一次发布之内跳号 | 留给 59 号 |
| 22 | 实二十：I-8.9 把多条记录的发布一律判成跳号（阳性对照那一格） | 在 g7x 里证过它让七份坏镜像多红「跳号」；它点名的阳性对照用例在同组里先被第 27 行红在第一条断言上，**单独跑它点名的那条用例留给 59 号** |
| 23 | 实二十：I-8.9 不判一次发布的第一条不从 1 起 | 留给 59 号 |
| 24 | 实二十：I-8.9 不判多于一条末条 | 留给 59 号 |
| 25 | 实二十：I-8.9 不判末条标志之后同一次发布还有记录 | 留给 59 号 |
| 26 | 实二十：I-8.9 不判没有末条 | 证过（g7y） |
| 27 | 实二十：I-8.9 在环里有记录的镜像上也报不适用 | 证过（g7x） |
| 28 | 实二十：池级 checker 读记录标志的偏移错成算法类型那一格 | 留给 59 号 |

证过 16 行（1、3、4、6、7、10、11、12、13、14、15、16、17、22 的一半、26、27），留给门禁 59 号整表复跑 12 行（2、5、8、9、18、19、20、21、22 点名那条、23、24、25、28）。

**主表要删的三行**（补丁打上之后原文命中 0 次，门禁 33 号在补丁后的树上现红在这三行上，`gatelogs/33-mutation-tables.sh.cwd.log`）：438（锚点 `.map(|record| record.counter)\n        .max();`，旧锚点那种「取 jsn 最大」写法没有了；替代：第 12 行）、440（`if named_units_of_the_last_record > named_unit_capacity {`，拒绝分支删了；替代：第 8 行）、441（`let named_units_of_the_last_record = roles_named_by_each_record_of_the_publish(&rewritten)…`，同一处删了；替代：第 7 行）。在 `gatecheck/`（`rebased/` 的副本、主表删这三行再接上追加表）里跑 33 号，退出码 0，末行原样：`  ✓ 147 个实验二进制都有成形的变异表，1593 条变异的原文各命中源码一次；crates/mutations.tsv 561 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）`。
锚在这几段代码上的其余旧变异，原文我刻意留着没动、仍各命中 1 次（第 28、44、436、437、448、449、487–493 行；主表除 438、440、441 之外逐行现数都是 1 次），其中第 437、449 行（`if !record_ends_its_publish(record) {` → `if false {`）现在测的是「按标志认边界」那一判，第 28 行测的是读法乙锚点之后接的那一格。

## 五、验证

**跑了的**（负载：12:2x 起机器 load 58–80、别的会话同时有多条 cargo；12:54Z 收到线程上限 4 之后每条重命令都经 `research/scripts/capped.sh 4`，之前起的那一串没加）：

- 动到的十个测试二进制，`repo/` 上整二进制各跑一次（`repo-tests-1/summary.txt` 原样，节选末段）：
  `second_transaction_parallel_line_one_last_record_flag … exit=0 test result: ok. 6 passed`、`…three_many_inodes … 4 passed`、`…three_spill_over_layer0 … 1 passed`、`first_transaction_step_five_publish … 8 passed`、`checker_known_bad_images … 29 passed`、`…one_multi_unit_file … 7 passed`、`…step_three_formatted_pool … 13 passed`、`…step_four_rollback … 12 passed`、`…c533_row_publish_record_without_its_root … 1 passed`、`first_transaction_step_seven_layer0 … 5 passed; 0 failed; 1 ignored`。
- `singlefs-core` / `singlefs-checker` / `singlefs-format` 单测（capped，`repo-tests-2-core.log`）：checker 3 passed、core 98 passed、format 5 passed（另有文档测试 1 passed）。
- 层 0 快档：第一条流（`first_transaction_step_seven_layer0`，上面那条）与 `second_transaction_step_zero_layer0`（13:39–13:48Z，`8 passed; 0 failed; 1 ignored`）绿；其余三条（`parallel_line_one_layer0`、`acquisition_barrier_layer0`、`formatted_pool_layer0`）照主 agent 13:4x 的指示停掉、没跑完，留给最后统一验证。
- **层 0 流 L8（全量）**：`every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all` 绿（12:32–12:38Z）。它自己钉的数：20 个状态 = 记录段 15 + 根槽 1 + 系统配置槽 3 + 全部持久 1；逐状态按两条记录落没落核发布边界，计数 `{ no_record: 1, only_one_of_the_two_records: 6, both_records_without_the_root: 9, root_persisted: 4 }`；每个状态两遍恢复 + 多版本 oracle、池级 checker（全部不变量违例 0，I-8.8、I-8.9 都真被评估过）、记录核对器，全部零违例。`LAYER0_SPILL_OVER` 计数行在 `--nocapture` 下才打，这一轮没带，逐条不变量的评估 / 不适用数没取。
- `rebased/`（交的补丁那棵树）：`cargo fmt --all -- --check` 0、clippy（check.sh 那一串 `-D` 全带上，capped）0、`cargo build --offline --all-targets`（capped）0，`rebased-fmt-clippy-build.log` 末行原样 `BUILD 0 end 2026-09-24T14:41:51Z`；`cargo check --all-targets` 0。
- `git apply --check /tmp/claude-1000/impl-m2-lastflag/impl-m2-lastflag.patch`（主工作区，14:4xZ）退出码 0。
- 登记给 implementation-writer 的七个门禁阶段（`stage-owners.tsv`），对着 `rebased/` 跑（33 号对着 `gatecheck/`）：

| 阶段 | 退出码 | 末行原样 |
|---|---|---|
| 33-mutation-tables | 0（`gatecheck/`，主表删三行、接上追加表）；1（补丁后不删那三行） | 见第四节 |
| 53-format-const-placeholders | 0 | `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））` |
| 93-feature-bits | 0 | 以 `  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致` 打头的那一行（全文在 `gatelogs/93-feature-bits.sh.rebased.log`） |
| 94-checker-implementation-disjoint | 0 | 以 `    这一道判不了的：两边各自手写的那份语义对不对` 打头的那一行（全文在 `gatelogs/94-…rebased.log`） |
| 89-closeout-row27-preconditions | 77（本次未跑） | `      alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像：扣住位只住内存、盘上没有落点，没有可扫的对象`（前置一个都没进来，与这次改动无关） |
| 92-layout-checker-sync | 77（本次未跑） | `  ! /tmp/claude-1000/impl-m2-lastflag/rebased 不是 git 仓，本阶段跳过`（它判 `singlefs-format` 与 kb 标记里的格式常量；我加的常量在 `singlefs-core/src/journal.rs`，不在它的格式定义路径里，checker 路径也碰了） |
| 74-model-differential | 没跑 | 它跑随机历史那五段（重活），照主 agent 收窄的范围留给最后统一验证 |

**没跑的**：`check.sh` 里的 `cargo test --all`（主 agent 定全量测试在全部代码落定之后统一跑一次）；`rebased/` 上的任何测试（测试与变异都在旧底 `repo/` 上）；层 0 其余三条流；门禁 54（L8 不在它的两条流里）、59（整表复跑）、74。

## 六、停下交主 agent 的设计问题

1. **所选根那次发布读得出的记录里带末条标志的多于一条，锚点认哪一条：条款没写**（已定项 14 注 1 读法乙只说「那条按末条标志认」「一条都不带就算读不出」）。走得到，所以照定义在任何落盘动作之前返回错误成员 `RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { instance, checkpoint_txg, counters }`（`rebased/crates/singlefs-core/src/recovery.rs` 第 198 行成员、第 1682 行判），用例 `two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write` 钉住「`recover` 报这个成员、`mount_writable` 交回同一个成员、录制流 0 步、两块盘整份镜像逐字节相等」，不钉之后怎么办。**两条走得到的路**：
   - 一条记录坏了而 CRC-32C 恰好仍对得上，或镜像是改出来的；
   - **同一个实例在一次多条记录的发布失败之后接着重发**：今天失败之后不做实例切换（D23 已定项 7「失败即实例切换」、已定项 14 失败处置的代码挪到后面的里程碑），下一次发布的 txg 与计数器照旧从内存里的上一版接（`transaction.rs` 第 2349、2355 行一带：`previous.root.checkpoint_txg + 1`、`previous.record.counter + 1`）——两条记录的发布 P 在写第二条时一块盘报错、另一块盘上第二条已落，重发的 P′ 若只一条记录，就与 P 的第二条共享 (实例, txg)、各带一个末条标志；P′ 的根落盘之后，下一次挂载停在这个成员上。旧代码（读法甲）在这一格锚在 jsn 大的那条上照常挂。I-8.9 在这一格判「多于一条末条」。要不要在写路径上挡住「失败之后同一实例接着发」、或给这一格定一个锚点，交主 agent。
2. **锚点读得出时，下一次发布的第一条序号不是 1（也不是 0）**：读者规则（已定项 4）只写「序号 0 或一次发布之内跳号当损坏」，这一格不属于两者，读者照接；只有锚点读不出那一支要求序号 1（注 1）。没加分支。checker 的 I-8.9 判它「不从 1 起」。
3. **一条记录的末条标志坏在一次发布中间**（同样要 CRC 恰好对上）：读者照字面按标志认边界，施加到那条为止；同 txg 的下一条开一次新的「发布」，它的序号是上一条 + 1、不查是不是 1。没加分支；checker 判「末条之后还有记录」。
4. **「当损坏」落在解析器里**：序号 0、记录标志其余位非 0 的记录由 `JournalRecord::parse` 交 `None`，于是它在接链、锚点、`scan_journal` 给出的「环里读得出的最大计数器」（新实例从它 + 1 起）、新实例 txg 下界里都算读不出——与校验和不过同一个结局。另一种读法是只在接链时断、别处照算，我取了前一种；推翻它的现象：条款写明「当损坏」只管前缀。
5. **跨出去的记录属于最后一个事务**：事务号相同、提交标记只在它的最后一条——从已定项 7「同一事务的全部记录共享它」与 I-8.8 ③「带提交标记的至多一条且是计数器最大那条」推的，不是新定的；另一种写法（跨出去的每条各算一个事务、各带提交标记）会让事务号与数据单元对不上（C310）。第四节第 7 行变异钉着「各算一个事务」那一种。
6. **checker「没有末条」「不从 1 起」两条判据的射程**：I-8.9 说「只判读得出的记录」。我把「没有末条」收成「组里最大计数器的下一个计数器上坐着**同一实例**的别组记录」才判（崩在一次发布的记录之间、新实例从读得出的最大号 + 1 接着写，那一格是合法的，不判），「不从 1 起」收成「组里最小计数器的前一个计数器上有自证过的别组记录」才判。它们的合法性压在「失败即实例切换」上：第 1 条那种同实例重发一出现，I-8.9 在合法镜像上也会红。

## 七、要主 agent 或别人处置的（不在我的写范围，或不是我的文件）

- **kb**：`invariants.md` I-8.9 那一行（第 86 行）状态「未实现」→ 已实现（checker 判定写在第二节第 5 条；阳性对照与七份坏镜像在 `checker_known_bad_images.rs`；层 0：L8 与第一条流每个状态都判）；第 12 行「判 43 条」随之 44，与 `IMPLEMENTED_INVARIANTS` 长度 44 对齐（`rebased/` 里现数）。D23 已定项 14 第六条正文（第 353 行）还写着「**发布边界怎么认**：一次发布的末条 = 点名了这次发布共享的提交内生块的那一条」，与依据里「末条按记录标志位 0 认」和今天的实现说反话。里程碑收口表第 9、36 行，C491 与 C500 的状态随这次改。
- **`research/`**：E142 装置按新代码重跑（第一个事务那条记录的反向链应为 3937788698，w1、w4、t9 偏移 7 那一字节是 1）；「同步 311」那一批（调度表第一节）照旧由实验执行员做。
- **门禁登记**：L8 不在门禁 54 号跑的两条流里，要不要进 54 号、`stage-inputs.tsv` 与 `layout/01-first-txn.md` 八那张段序列登记表（L8 孤立看 `136+4+1+2`，起点是第一个事务整段做完的镜像），归 crash-verifier / 主 agent。
- **碰到别人的文件**：`crates/singlefs-core/src/mount.rs` 只在 `mount_writable`（实十八在改的文件）里给 `replay_journal(...)` 加了一个 `?`（`rebased/` 第 1702 行）；同一个函数第 1703 行起取「上一版的记录」仍按「同 txg 里 jsn 最大那条」取，没改——那条记录只有 jsn 会被用到、jsn 另算，读法乙不改变这里的结果，但措辞与注 1 不一致。`recovery.rs` 第 1124 行 `rebuild_version`（实十四）的文档注释也还写「jsn 最大的那条」，没改。`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（E158 装置）新出现的那处 `replay_journal` 调用加了 `.expect`，装置归实验执行员。
- **`crates/mutations.tsv`**：删第 438、440、441 行，末尾接 `mutations-append.tsv` 的 28 行（第四节）。

## 八、这一轮写过的文件

补丁里的 21 个（`b/` = `rebased/`；新建 2 个：`crates/singlefs-harness/tests/second_transaction_parallel_line_one_last_record_flag.rs`、`crates/singlefs-harness/tests/second_transaction_parallel_line_three_spill_over_layer0.rs`）。`crates/mutations.tsv` 没改，追加的 28 行在 `mutations-append.tsv`（变异名见第四节表）。我在副本里改、没碰主工作区，主工作区的 `git diff --stat -- crates litmus` 只会列别的会话的改动，所以这里附的是补丁对着主工作区的 `git apply --stat`（14:45:35Z 现跑）原样：

```
 crates/singlefs-checker/src/image.rs               |    6 
 crates/singlefs-checker/src/lib.rs                 |   10 -
 crates/singlefs-checker/src/walk.rs                |  192 ++++++++++
 crates/singlefs-core/src/journal.rs                |  131 +++++++
 crates/singlefs-core/src/mounted_read.rs           |    3 
 crates/singlefs-core/src/mount.rs                  |    2 
 crates/singlefs-core/src/recovery.rs               |  146 ++++++--
 crates/singlefs-core/src/transaction.rs            |  232 ++++++++++---
 .../src/bin/e158_root_choice_repair.rs             |    3 
 crates/singlefs-harness/src/history.rs             |    7 
 crates/singlefs-harness/src/model_comparison.rs    |    3 
 .../tests/checker_known_bad_images.rs              |  333 ++++++++++++++++++
 .../tests/first_transaction_step_five_publish.rs   |   17 +
 .../tests/first_transaction_step_seven_layer0.rs   |    4 
 ...ansaction_parallel_line_one_last_record_flag.rs |  366 ++++++++++++++++++++
 ...ransaction_parallel_line_one_multi_unit_file.rs |    7 
 ..._transaction_parallel_line_three_many_inodes.rs |  223 +++++++++++-
 ...action_parallel_line_three_spill_over_layer0.rs |  293 ++++++++++++++++
 .../tests/second_transaction_step_four_rollback.rs |    3 
 ...second_transaction_step_three_formatted_pool.rs |   22 +
 ...two_c533_row_publish_record_without_its_root.rs |    3 
 21 files changed, 1857 insertions(+), 149 deletions(-)
apply-check exit 0
2026-09-24T14:45:35Z
```

草稿目录里另写的：`base/`、`base2/`、`repo/`、`rebased/`、`mutant/`、`mutant2/`、`dump/`、`gatecheck/`（仓副本）、`a`、`b`（生成补丁用的软链接）、`tools/`（`back_chain_with_last_record_flag.py`、`mutation_rows.py`、`anchor_hits.py`、`run_binaries.sh`、`run_mutations.py`、`run_mutation_group.py`、拼接用的几段 `.rs`）、`dump-records/`、`back-chain-recompute.out`、`repo-tests-1/`、`repo-tests-2/`、`mutation-logs/`、`gatelogs/`、`clippy*.log`、`rebased-*.log`、`progress.md`、`impl-m2-lastflag.patch`、`mutations-append.tsv`、`patch-stat.txt`、本报告。

## 九、没做什么

- 没走三方对抗；层 0（除第一条流与 step_zero 的快档、L8 全量）、QEMU、herd7 与 crates 变异整表归 crash-verifier / 门禁 59；没提交。
- 新底 `rebased/` 上没跑任何测试（只有 fmt、clippy、build、check）；`check.sh` 的 `cargo test --all` 没跑（主 agent 定最后统一跑）；层 0 `parallel_line_one_layer0`、`acquisition_barrier_layer0`、`formatted_pool_layer0` 三条流的快档按指示停掉。
- 追加表 12 行没单独证红（第四节），第 22 行点名的那条用例没单独跑。
- L8 的单元段（136 写）只以整段持久进入后面的状态，2^136 − 1 个子集没枚举（第五节、测试文件头注释）；L8 只录了一个事务跨两条记录的形态，没录「多个数据单元 + 最后一个事务再跨记录」（那一形今天的写法里够不着：顺序写的共享内生块只有七八个，远不到 67；切法由 `transaction::tests::the_last_transaction_spills_over_…` 在纯函数上钉住三条记录跨出去的形态）。
- 门禁 74、54、59 没跑；92、89 报 77（本次未跑，第五节）。
- kb、`research/` 一个字没动（第七节）。
- 第六节六条都没替主 agent 定。
