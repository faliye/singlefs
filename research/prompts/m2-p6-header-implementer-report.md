# impl-m2-p6-header 报告（P6 后一半：journal 记录头加「本次发布内序号」，JOURNAL_HEADER_BYTES 307 → 311）

时刻都是 UTC（东京 = UTC + 9）。主工作区一个字没动（只跑了 `git apply --check` / `--stat`）；改动在副本 `repo/` 里做、交补丁。
**基准**：补丁对 `base-crates/`（04:54:26–04:54:28 拷主工作区 `crates/`）做。05:33:28 对主工作区 `git apply --check -p1` 退出 0；
那时主工作区与基准差 `mutations.tsv`、`walk.rs`、`checker_known_bad_images.rs`、`second_transaction_step_four_rollback.rs`、
`second_transaction_step_three_formatted_pool.rs`、`e156`、`e158` 与一份新用例（别的会话在改），其中 `checker_known_bad_images.rs`
新加的代码按 307 写死了两个偏移，补丁打上之后要跟着改（第五节 ②）。下文行号除注明外都是 `repo/`（基准 + 补丁）里的。

## 一、交付

| 件 | 路径 | sha256 |
|---|---|---|
| 补丁（9 个文件，只含 `crates/`，不含 `crates/mutations.tsv`；`litmus/` 没动） | `/tmp/claude-1000/impl-m2-p6-header/impl-m2-p6-header.patch` | `26008cbb2b29a386f3b4241706f91a3b6e402ef809161f2f0089de7ed86cb585` |
| 新变异行（13 行，补丁打上之后原文各恰好命中一次） | `/tmp/claude-1000/impl-m2-p6-header/mutations-append.tsv` | `20896ff8a1ed8da231085de287c8a88db94132ed9b8abc7f1bcadf594e997f00` |
| 主表第 44 行的替代行（整行） | `/tmp/claude-1000/impl-m2-p6-header/mutations-replace-row-44.tsv` | `004e6cd1553a852b35e62a31cf53d55ca6f7ae13f05be8b2f4cc93086954314d` |
| 报告 | `/tmp/claude-1000/impl-m2-p6-header/report.md` | 交回里给 |

**打上补丁之前主 agent 要换掉主表第 44 行**（第五节 ①）：补丁把 `recovery.rs` 那一判拆成两行，第 44 行的原文（带行尾 ` {`）零命中。

## 二、做了什么

记录头按 D23 已定项 4 加宽，不找空位：「本次发布内序号」无符号 32 位、小端，**紧跟提交标记**，它之后的字段整体后挪 4 字节。

| 字段 | 307 时的偏移 | 311 的偏移 | 宽 |
|---|---|---|---|
| 十个字段（magic … 头校验和） | 0 | 0 | 78 |
| 事务号 | 78 | 78 | 8 |
| 提交标记 | 86 | 86 | 1 |
| **本次发布内序号（新）** | — | **87** | 4 |
| 反向链 | 87 | 91 | 4 |
| 载荷校验和 | 91 | 95 | 4 |
| 新根段 | 95 | 99 | 188 |
| fsid | 283 | 287 | 8 |
| MAC | 291 | 295 | 16 |
| 头末 / 点名项起点 | 307 | 311 | — |

一条 4096 记录的点名项上限不变：⌊(4096 − 311) ÷ 56⌋ = ⌊3785 ÷ 56⌋ = 67（`JOURNAL_NAMED_ENTRIES_PER_RECORD` 的单测照旧钉 67）。

| 文件 | 改了什么（行号） |
|---|---|
| `crates/singlefs-format/src/lib.rs` | `JOURNAL_HEADER_BYTES` 307 → 311（165 行，文档注释加「本次发布内序号 4」与出处）；单测的字段和多加一项 `+ 4`（305–314 行）、字面量断言 311（323 行） |
| `crates/singlefs-core/src/journal.rs` | 新 newtype `JournalRecordOrdinalWithinPublish(pub u32)`（30 行），`FIRST = 1`（34 行），`of_record_at_offset(k) = k + 1`（42–47 行，`expect` 写明依赖「落盘之前按 extent 叶容量截过」）；偏移常量 `JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET = 46 + 32 + 8 + 1`（24 行）；`JournalRecord` 加字段 `ordinal_within_publish`（113 行）；写者在提交标记之后 `assert_position` 再写它（165–169 行）；读者在同一处读它（227 行）；模块注释与 `back_chain_of` 注释、三处 `expect("307")` 改成 311 |
| `crates/singlefs-core/src/transaction.rs` | 三处记录构造：零单元 / 空发布写 `FIRST`（598 行）、树表 0 条那一版上写行那次发布写 `FIRST`（926 行）、带文件的发布第 k 条写 `of_record_at_offset(k)`（3805 行，一次发布 N 条依次 1..N，N = 1 时是 1） |
| `crates/singlefs-core/src/recovery.rs` | `replay_journal` 没有锚点时，链首除了 `checkpoint_txg = 根 + 1` 还要本次发布内序号 = 1（1527–1528 行）；1502–1510 行的注释改写成现状（原来那句「这一半没做」删掉） |
| `crates/singlefs-checker/src/lib.rs` | 固定偏移换成 311 的布局：新增 `JOURNAL_COMMIT_MARKER_OFFSET = 86`、`JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET = 87`、`JOURNAL_BACK_CHAIN_OFFSET = 91`、`JOURNAL_PAYLOAD_CHECKSUM_OFFSET = 95`，`JOURNAL_NEW_ROOT_SEGMENT_OFFSET` 95 → 99、`JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET` 283 → 287（503–510 行）；原来写成 `JOURNAL_TRANSACTION_OFFSET + 8 + 1 + 4` / `+ 9` / `+ 8` 的三处改用这几个名字；`JournalRecordView` 加 `ordinal_within_publish: u32`（548 行，读在 632 行），不据它判任何不变量（第四节 ②） |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | I-8.6 那份坏镜像改反向链的切片 `[87..91]` → 命名常量 `JOURNAL_RECORD_BACK_CHAIN_OFFSET = 91`（810 行、1038 行）；`JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET` 283 → 287（1035 行）；注释「w1 的 307 字节头」→ 311 |
| `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs` | 钉住的产物值 `E142_BACK_CHAIN_OF_FIRST_TRANSACTION` 628216162 → 1057457588（67 行，值的来历见第三节 T5）；断言消息「307 字节头」→ 311；三条记录都断言 `ordinal_within_publish == 1`（493 行，经 checker 的独立解析读） |
| `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs` | 写行那次发布（树表 0 条那一版）的记录断言序号 1（167 行） |
| `crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs` | 新用例 `without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish`（509 行）与文件头第 6 条 |

## 三、新测试与「证明会红」

**今天的代码上错接（派发要的那一格）**：`today/` 是基准的原样副本（今天的 307 字节头、没有序号那一判），只往 `second_transaction_parallel_line_one_multi_unit_file.rs` 末尾加了同一条用例、去掉从盘上读序号那一段（今天的头里没有这个字段）。05:04:44 跑整个二进制，红在第一格（原样，`today-k4.log`）：

```
thread 'without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish' (281941) panicked at crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs:562:9:
assertion `left == right` failed: A 那条与 B 的第一条都读不出：B 的第二条序号是 2，不许当链首：施加之后走的根（JournalScanReport { valid_records: 4, above_water: 2, prefix_applied: 2, verification_passed: 2, verification_failed: 0, maximum_applied_transaction: 4 }）
  left: Some((InstanceGeneration(1), CheckpointTxg(4)))
 right: Some((InstanceGeneration(1), CheckpointTxg(3)))
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.37s
```

即今天的代码从 B 的第二条接上、施加了 2 条、走到 B 的根 (1, 4)：缺了第一条的 B 被整体施加。改后同一格一条不施加、走 A 的根 (1, 3)、读回 A 的内容；只撕 A 那条时 B 的第一条（序号 1、txg = A + 1）接上、3 条整次施加、读回 B。

**用例历史**：`build_pool`（mkfs → 暖机两次 → A，一条记录）→ 顺序写 3 个数据单元的 B（3 条记录，jsn 4–6，序号 1、2、3，txg 4）→ 取崩在 B 的根槽 FUA 之前那一刻的镜像（B 的记录两盘都落了）→ 两块盘上各翻 A 那条（jsn 3）记录的第 300 字节，第一格再翻 B 的第一条（jsn 4）。

**变异证红**：`mutant/`（repo 的副本，自己的 target）上逐条改一处、跑那条测试所在的**整个**测试二进制（不带过滤）、从 `repo/` 拷回并 `touch`、`diff -q` 为空。基线：同一份未改的 `mutant/` 上 6 个二进制全绿（05:12:19–05:13:46，`mutations-run.log` 前 8 行）。全部 debug；这几处被测代码里没有 `debug_assert`，红的都是测试断言或测试里的 `expect`。第 1、2 行在 rustfmt 之后又重跑了一次（`mutations-rerun-after-fmt.log`），行号是 `repo/` 的。

| 行（`mutations-append.tsv`） | 改坏哪一行 | 跑的二进制 | 红的断言（点名的那条） | 同时红的测试 |
|---|---|---|---|---|
| 1 | `recovery.rs` 1528 行 `|| record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST` → `|| false` | `second_transaction_parallel_line_one_multi_unit_file` | `without_an_anchor_…`：590 行「A 那条与 B 的第一条都读不出：B 的第二条序号是 2，不许当链首：施加之后走的根」，`prefix_applied: 2` | 无 |
| 2 | `transaction.rs` 3805–3807 行多条记录的序号 → 一律 `FIRST` | 同上 | 同一条：553 行「B 三条记录的本次发布内序号」（盘上经 checker 读出 [1, 1, 1]） | 无 |
| 3 | `journal.rs` 44 行 `record_offset_in_this_publish + 1` → 去掉 `+ 1` | `-p singlefs-core --lib` | `journal::tests::the_ordinal_of_the_record_at_offset_k_of_a_publish_is_k_plus_one`：343 行 `of_record_at_offset(0) == FIRST` | 无 |
| 4 | `journal.rs` 169 行写者 `put_u32(self.ordinal_within_publish.0)` → `put_u32(1)` | `-p singlefs-core --lib` | `journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips`：308 行「本次发布内序号紧跟提交标记，在 [87, 91)」 | 无 |
| 5 | `journal.rs` 227 行读者 → 跳过 4 字节、一律当 `FIRST` | `-p singlefs-core --lib` | 同一条：337 行 `parse(&bytes, 7) == Some(record)` | 无 |
| 6 | `transaction.rs` 598 行空发布记录写 `FIRST` → `(2)` | `first_transaction_step_five_publish` | `root_slots_system_configurations_and_journal_ring_hold_the_published_state`：492–494 行「三次发布各只有一条记录：本次发布内序号都是 1」 | 无 |
| 7 | `transaction.rs` 926 行树表 0 条那一版写行的记录 `FIRST` → `(2)` | `second_transaction_step_three_formatted_pool` | `writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance`：167 行「写行那次发布只有这一条记录：本次发布内序号 1」 | 无 |
| 8 | `checker/src/lib.rs` 506 行序号偏移 87 → 91 | `first_transaction_step_five_publish` | `root_slots_…`：492–494 行同一条（读到的是 w1 的反向链 0） | 无 |
| 9 | 同文件 507 行反向链偏移 91 → 87 | 同上 | `root_slots_…`：488–490 行「反向链 = 本实例内逻辑前一条记录的 311 字节头的 CRC-32C」 | 无 |
| 10 | 同文件 508 行载荷校验和偏移 95 → 91 | 同上 | `root_slots_…`：`expect("记录自证过")` 得 `ChecksumMismatch` | `central_mapping_holds_six_entries_rebuilt_from_the_named_entries_and_excludes_itself`（`ChecksumMismatch`） |
| 11 | 同文件 509 行新根段偏移 99 → 95 | 同上 | `root_slots_…`：498–501 行空记录的（水位, F）= (11, 0) 那一条 | 无 |
| 12 | 同文件 510 行 fsid 偏移 287 → 283 | 同上 | `root_slots_…`：`expect("记录自证过")` 得 `FilesystemIdentifierMismatch` | `central_mapping_holds_six_…`（`FilesystemIdentifierMismatch`） |
| 13 | `format/src/lib.rs` 165 行 311 → 312 | `-p singlefs-format --lib` | `tests::widths_match_the_first_transaction_byte_table`：「journal 记录头 = 十个字段 + … + MAC」那条和式断言 | 无 |
| 第 44 行替代行 | `recovery.rs` 1527 行 `} else if record.checkpoint_txg != …` → `} else if false && record.checkpoint_txg != …`（`false && a \|\| b` 只剩序号那一判） | `second_transaction_step_three_second_instance` | `torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg`：「撕掉 [4, 5]（所选根 (1, 4)）」，jsn 6（txg 6、序号 1）被接上 | 无 |

**T5 钉住的产物值怎么来的**：`E142_BACK_CHAIN_OF_FIRST_TRANSACTION` 是「第一个事务那条记录的反向链 = CRC32C(暖机第二条记录的头)」，头一变它就变。新值不照实现的输出抄：`today/` 里导出 307 布局下的 jsn 1–3 三条记录（草稿用例，只在 `today/`），`tools/back_chain_311.py` 自己实现 CRC-32C（先过公开校验值 `123456789 → 0xE3069283`）、先复现 307 布局下的链值当阳性对照，再在提交标记之后插 4 字节序号 1、重算 jsn 2 的反向链、算 jsn 2 的 311 字节头。原样输出：

```
old_layout=307 back_chain_of_jsn_3=628216162 matches_e142_line_37=True
new_layout=311 back_chain_of_jsn_2=823937809 back_chain_of_jsn_3=1057457588
```

把常量留在旧值时，新代码上那条断言红成 `left: 1057457588 right: 628216162`（05:07 那次跑）——实现写出来的与独立换算一致，这条断言也证明了它对头的变化敏感。E142 产物第 37 行要等装置按 311 重跑才会是这个数（第五节 ④）。

没另证红的：`journal::tests::record_is_4096_with_a_311_byte_header_and_round_trips` 只是改名改数（307 → 311）；`checker_known_bad_images.rs` 三处是跟着改偏移（I-8.6 那份坏镜像照旧只红 I-8.6，全量跑里那个二进制全绿）。

## 四、停下交主 agent 的设计问题（条款没写 / 两种读法）

① **所选根那次发布有多条记录、末条（锚点）读不出而前面几条读得出时，锚点按哪一条认**（三方第一轮 K4 的 `anchor-torn-other-P0-readable` 那一支，判决里写「196 格四条臂同中」、不分辨候选）。D23 已定项 14 注 1：「所选根覆盖的最后一条 = 与所选根同实例、同 `checkpoint_txg` 的记录里 jsn 最大的那条……那条读不出时，链首接水位之上第一条可读记录，前提是它的「本次发布内序号」为 1」。两种读法：
  - 甲（今天的实现，补丁没动）：锚点 = **读得出的**同 txg 记录里 jsn 最大那条（`recovery.rs` `root_own_record_counter`，1510–1518 行）。它不是那次发布的末条，链从它 + 1 接，下一条正是读不出的末条 ⇒ 断号即止，下一次发布一条都不施加（少施加，不多接）。
  - 乙：锚点按「那次发布的末条」认；读得出的那几条都不是末条（`record_ends_its_publish` 为假）就算「那条读不出」，走序号 1 那一支 ⇒ 下一次发布若从序号 1 起读得出，整次施加。
  两种读法在同一个坏盘历史上施加的集合不同，我没选。这一支只有坏盘走得到，`recover` 本身不写盘；没加用例钉它。推翻「条款没定」的现象：注 1 写明锚点是在读得出的记录里取，还是按发布末条取。

② **池级 checker 不据序号判任何不变量**。`JournalRecordView` 只把它读出来（548、632 行）；没有一条不变量写「一次发布的 N 条记录序号依次 1..N、与 jsn 同步」。要不要立（I-8.x，坏镜像与 checker 判定一起做）交主 agent。

③ **读者不拒序号 0，也不核一次发布之内序号连不连续**。条款只在「锚点读不出」那一支用序号；锚点读得出的那一支照旧只靠 jsn 连号与同 txg。盘上序号 0 或跳号时今天的行为：锚点读得出 ⇒ 不看序号；锚点读不出 ⇒ 只有序号正好是 1 的那条能当链首。没加分支（条款没写的分支不加）。

④ 条款没写、所以没加的非分支项：`JournalRecordOrdinalWithinPublish` 用 `u32` 不用 `NonZeroU32`（读者要原样读回盘上的 0，拒不拒是 ③ 的事）；没给它 `Display` / `From`；没加「本次发布有几条」字段（条款只定了第几条）。

## 五、要主 agent 处置的（不在写范围，或不是这一轮的文件）

① **`crates/mutations.tsv` 第 44 行锚点断**（主工作区 05:33 的表 486 行；`anchor_check.py` 对「主工作区 + 补丁」输出 `mutations.tsv:44 命中 0 次…` `rows=486 bad=1`，别的行全在）。替代行（整行，六段，已证红，见第三节表末行）在 `mutations-replace-row-44.tsv`：

```
步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-core/src/recovery.rs	        } else if record.checkpoint_txg != chain_start_txg_without_anchor\n	        } else if false && record.checkpoint_txg != chain_start_txg_without_anchor\n	-p singlefs-harness --test second_transaction_step_three_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg
```

② **别的会话 05:22 加进 `checker_known_bad_images.rs` 的代码按 307 写死了两个偏移**（08:21 主工作区的 3192–3197 行，基准里没有这一段，补丁没碰）：`JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 91` 要改成 95，`JOURNAL_RECORD_HEADER_BYTES: usize = 307` 要改成 311，注释「事务段 78 + 事务号 8 + 提交标记 1 + 反向链 4 之后是载荷校验和」要补「本次发布内序号 4」。用到它们的是 `propagate_into_journal_records`（3203 行），被 3293 行的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` 调；「主工作区 + 补丁」上它红不红见第七节。

③ **`crates/singlefs-checker/src/walk.rs` 两行文档注释还写「307 字节头」**（基准 2358、2424 行；主工作区现在 2443、2509 行）：只是注释、代码走 `back_chain_of_record_header`，不影响判定。walk.rs 在别的会话手里，我没碰；改成 311 或写成「整个记录头」。

④ **kb 与 research（我不改）**：
  - D23 已定项 4（`.claude/kb/decisions/23-journal的角色与格式.md` 139–164 行那一节）：「头 307 字节」「登记值就是 307」→ 311；「307 占 512 扇区的 60%，其后还余 205 字节（装得下 3 个点名项）；4096 上余 3789（67 个点名项，⌊(4096 − 307) ÷ 56⌋ = ⌊3789 / 56⌋）」→「311 占 512 的 61%，余 201（⌊201 ÷ 56⌋ = 3 个点名项）；4096 上余 3785（⌊3785 ÷ 56⌋ = 67 个点名项）」；「随并行线一落地：落地之前实现的头仍是 307……」那一句已兑现。`format-const: JOURNAL_HEADER_BYTES = 311`；可登记进 `stale=` 的候选（登记前照 `format-evolution.md` 现扫一遍）：`JOURNAL_HEADER_BYTES: u64 = 307`、`的 307 字节头`、`头 307 字节`、`4096 − 307`、`合计 **307 字节**`——今天 research 源码、walk.rs 注释与 ② 那段代码里还有，登记了 27 号会先红在它们身上。
  - C500（`checks-owed.md` 443 行）要的故障注入用例就是第三节那一条（链首按条款接、施加集合逐字相等、判别力自证 = 第 1 行变异）；还不还、C365 的前置要不要改，归主 agent。
  - 里程碑收口表第 9 行（P6「实现归第 36 行」）与第 36 行的现状句。
  - E142：产物第 37 行 `back_chain=628216162` 与 `offset_in_structure=91 … 载荷校验和` 一类行都是 307 布局下的；装置按 311 重跑之后，第 37 行应是 1057457588（第三节 T5）。
  - `research/e7-index-bench/src/bin` 的命中逐条附在第十节（72 行，原样）。

## 六、层 0

没加层 0 流，也没加崩溃点重放用例。已有的层 0 快档（两条固定流、并行线一的流、第一个事务那条等）在新布局上跑全绿（第七节全量里的各二进制）：记录仍是整条 4096 写，段序列与闭式状态数不随头宽变；变的只有记录里的字节（头多 4 字节、点名项整体后挪 4 字节）。全量枚举（ignored 的那几条）没跑，归 crash-verifier；层 0 的崩溃状态里锚点记录总是读得出（它在根槽之前落盘），走不到「锚点读不出」那一支，那一支只有坏盘走得到，所以用的是故障注入式的单条用例。

## 七、验证：check.sh 与全量单测

开跑前看负载（04:51、05:33 UTC 两次 `ps`）：没有 `qemu-system` / `vm-bench.sh` / `e152` / `fio`；有别的会话的 `cargo test`（`checker_known_bad_images`、`second_transaction_supplement_three_*`），没等锁（各副本各自的 target）。

**`check.sh`（`repo/` = 基准 + 补丁，05:33:10 起）原样末尾**：

```
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 1 previous error
  ✗ clippy 有告警（按 -D warnings 视为错误），或者踩了编码纪律的某一条
     → 怎么办： 上面每条告警都指着文件和行号，逐条改。编码纪律那几条的写法见 rules/code-discipline.md。
                确有必要保留的，在那一处写 #[allow(<lint>, reason = "为什么")]，理由写进 reason——
                不要整仓关掉 -D warnings（rules/command-safety.md：警告是最便宜的信号）。
check.sh exit=1
```

红在实验执行员在改的装置 `e158_root_choice_repair.rs` 的 `clippy::items_after_test_module`（基准里 2598 行）；`pristine/`（基准原样）上同一条 clippy 同样红，与补丁无关。`check.sh` 在 clippy 就停，后面三步照它的参数分开跑：

| 步 | 在哪 | 结果 |
|---|---|---|
| fmt `--check` | `repo/` | 过（`check.sh` 第一步「✓ 格式通过」） |
| clippy（同一组 `-D`）除 e158 之外全部目标 | `repo/` | `-p singlefs-format -p singlefs-core -p singlefs-checker --all-targets` 与 `-p singlefs-harness --lib` 加每个 `--test`、除 e158 的每个 `--bin`（dev 与 test 两种 profile）零告警 |
| `cargo test --all --no-fail-fast` | `repo/`（05:14:19–05:32:53） | 65 个 `test result` 全 ok：488 passed、0 failed、9 ignored（`repo-test.log`） |
| 同上 | `pristine/`（04:54:56–05:12:20） | 485 passed、0 failed、9 ignored（`pristine-test.log`）；多出的 3 条是这一轮的新用例 |

**「主工作区 + 补丁」**（`checkmerge/` = 05:33:28 拷的主工作区打上补丁）：`check.sh` 同样红在 e158 那条 clippy（那时主工作区的 2619 行）；`cargo test --all --no-fail-fast`（到 08:11:05）492 passed、**2 failed**、9 ignored：

| 红的测试 | 归谁 | 依据 |
|---|---|---|
| `checker_known_bad_images::an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` | **补丁打上之后别的会话那段代码要跟着改**（第五节 ②） | 不打补丁（`mainonly/` = `checkmerge/` 反打补丁）这个二进制 27 passed；打了补丁、只把那两个常量改成 95 / 311（草稿里改、跑完拷回）27 passed（`checkmerge-fixed-constants.log`）。红时的原样：`B 那一版只在记录里时，它的分配代写错要在崩溃镜像上就红、只红 I-3.10 … left: [] right: ["I-3.10"]` |
| `first_transaction_step_seven_layer0::layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape` | **不是这一轮的**：不打补丁同样红 | `mainonly/` 上原样：`panicked at crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs:130:9` / `test result: FAILED. 4 passed; 1 failed; 1 ignored`；红的断言是「I-3.10 评估过的状态数 … left: 7 right: 4」，I-3.10 在别的会话手里（实十二） |

登记给我的门禁阶段按派发没跑。

## 八、这一轮写过的文件

补丁里的 9 个（都在 `repo/crates/` 里改，主工作区没动）：`crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/journal.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/first_transaction_step_five_publish.rs`、`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`、`crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs`。`crates/mutations.tsv` 没写（追加的 13 行变异名见第三节表与 `mutations-append.tsv`，第 44 行替代行见第五节 ①）；`litmus/` 没动。别的会话在改的 `walk.rs`、`mount.rs`、`allocator.rs`、e156、e158 一行都没碰。

`git apply --stat`（补丁本身，05:21 对主工作区）：

```
 crates/singlefs-checker/src/lib.rs                 |   28 +++--
 crates/singlefs-core/src/journal.rs                |  115 ++++++++++++++++++--
 crates/singlefs-core/src/recovery.rs               |   14 ++
 crates/singlefs-core/src/transaction.rs            |   13 ++
 crates/singlefs-format/src/lib.rs                  |    9 +-
 .../tests/checker_known_bad_images.rs              |   17 ++-
 .../tests/first_transaction_step_five_publish.rs   |   17 ++-
 ...ransaction_parallel_line_one_multi_unit_file.rs |  115 ++++++++++++++++++++
 ...second_transaction_step_three_formatted_pool.rs |    7 +
 9 files changed, 292 insertions(+), 43 deletions(-)
```

主工作区 `git diff --stat -- crates litmus` 原样（08:22:05，全是别的会话与此前十份补丁的改动，这一轮一行都不在里面；未跟踪的新文件不在 `git diff` 里）：

```
 crates/mutations.tsv                               |  356 ++-
 crates/singlefs-checker/src/image.rs               |   67 +-
 crates/singlefs-checker/src/lib.rs                 |   21 +-
 crates/singlefs-checker/src/walk.rs                | 2015 ++++++++++++-
 crates/singlefs-core/src/address.rs                |   33 +
 crates/singlefs-core/src/allocator.rs              |  577 +++-
 crates/singlefs-core/src/block_device.rs           |  190 +-
 crates/singlefs-core/src/instance_table.rs         |  306 +-
 crates/singlefs-core/src/journal.rs                |   55 +
 crates/singlefs-core/src/lib.rs                    |    4 +
 crates/singlefs-core/src/make_filesystem.rs        |   50 +-
 crates/singlefs-core/src/mount.rs                  |  837 ++++--
 crates/singlefs-core/src/pointer.rs                |   28 +
 crates/singlefs-core/src/records.rs                |  161 +-
 crates/singlefs-core/src/recovery.rs               | 1382 +++++++--
 crates/singlefs-core/src/root_record.rs            |   25 +-
 crates/singlefs-core/src/root_ring.rs              |  212 +-
 crates/singlefs-core/src/system_configuration.rs   |  126 +-
 crates/singlefs-core/src/transaction.rs            | 3004 ++++++++++++++++----
 crates/singlefs-core/src/write_accounting.rs       |   10 +-
 crates/singlefs-format/src/lib.rs                  |  160 +-
 .../src/bin/first_transaction_device_log_check.rs  |  700 ++++-
 .../src/bin/first_transaction_on_device.rs         | 1495 ++++++++--
 .../src/bin/first_transaction_region_bytes.rs      |    2 +-
 crates/singlefs-harness/src/crash.rs               |  410 ++-
 crates/singlefs-harness/src/crash_injection.rs     |   47 +-
 crates/singlefs-harness/src/device_log.rs          |  210 +-
 .../src/first_transaction_regions.rs               |    7 +-
 crates/singlefs-harness/src/history.rs             |  321 ++-
 crates/singlefs-harness/src/lib.rs                 |  141 +-
 crates/singlefs-harness/src/model.rs               |  214 +-
 crates/singlefs-harness/src/model_comparison.rs    |  106 +-
 crates/singlefs-harness/src/scenario.rs            |   99 +-
 crates/singlefs-harness/src/segments.rs            |   73 +-
 .../tests/checker_known_bad_images.rs              | 2672 ++++++++++++++++-
 crates/singlefs-harness/tests/common/mod.rs        |    7 +-
 .../tests/first_transaction_region_bytes.rs        |    9 +-
 .../tests/first_transaction_step_five_publish.rs   |   97 +-
 .../tests/first_transaction_step_one_mkfs.rs       |  344 ++-
 .../tests/first_transaction_step_seven_layer0.rs   |   56 +-
 .../tests/first_transaction_step_six_recovery.rs   |  176 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    2 +
 .../singlefs-harness/tests/instance_acquisition.rs |  118 +-
 .../tests/second_transaction_step_five_reuse.rs    |  193 +-
 .../tests/second_transaction_step_four_rollback.rs |  870 +++++-
 .../tests/second_transaction_step_one_overwrite.rs |  140 +-
 ...second_transaction_step_three_formatted_pool.rs | 1074 ++++++-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |   52 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  234 +-
 ..._transaction_supplement_one_write_accounting.rs |  117 +-
 ..._transaction_supplement_three_random_history.rs |  249 +-
 ...nsaction_supplement_two_accounting_node_full.rs |    3 +
 ...ion_supplement_two_commit_generated_fallback.rs |  105 +-
 ...tion_supplement_two_instance_table_page_full.rs |   18 +-
 ...saction_supplement_two_row_publish_admission.rs |    6 +-
 ...d_transaction_supplement_two_unequal_devices.rs |   14 +-
 ...d_transaction_supplement_two_warm_up_counter.rs |  231 +-
 .../system_configuration_mutability_classes.rs     |    3 +
 59 files changed, 17646 insertions(+), 2592 deletions(-)
```

## 九、没做什么

- 没走三方对抗；层 0 全量、QEMU、herd7 与 crates 变异表整表复跑归 crash-verifier；没提交。登记给我的门禁阶段按派发没跑。
- 没改 kb、`research/`、`walk.rs`、别的会话 05:22 加进 `checker_known_bad_images.rs` 的那段（第五节 ②③④）。E142 没重跑，第 37 行的新值只由独立换算得出。
- 第四节 ① 那一格（锚点读不出而同一次发布前面几条读得出）没改、没加用例；② ③ 的检查与分支没加。
- 变异证红全在 debug 下跑；被测代码里没有 `debug_assert`，没另跑 `--release`。
- `check.sh` 没跑绿：红在 e158 的 clippy（不是这一轮的文件），后三步照同一组参数分开跑，结果在第七节。
- 草稿目录里的副本（`pristine/`、`repo/`、`mutant/`、`today/`、`checkmerge/`、`mainonly/`、`mergecheck/`、`patchwork/`）与日志都留着，没入库；`today/` 里那条去掉序号读取的用例变体和导出三条记录的草稿用例只用来取证，不进补丁。

## 十、`research/e7-index-bench/src/bin` 里的命中（主 agent 处理，原样）

其中 e142、e43、e116、e155 四份、e157、e159 的 `JOURNAL_HEADER_BYTES` / `RECORD_HEADER_BYTES = 307` 与由它推出的 3789、205、(4096 − 307) 是真命中；e71 的 2307、e130 的 307 个条目、e56 的扇出 307、e68 的 3072、e132 的 3075、e145 文件头的 C307、e143 的 `JOURNAL_HEADER_BYTES_TODAY = 95` 那一族与这次改动无关（e143 那一族是别的头宽口径，没核它要不要跟）。

```
$ grep -rn 'JOURNAL_HEADER_BYTES\|307' research/e7-index-bench/src/bin
research/e7-index-bench/src/bin/e71_accounting_keys.rs:360:        // 窄读臂在同一格远在预算内：(4×64 + 3 + 2×1024) = 2307 个条目，× 2 × 30 = 138_420
research/e7-index-bench/src/bin/e71_accounting_keys.rs:362:        assert_eq!(entries_per_generation(Arm::PerStatisticNarrow, 1024, 64, true), 2307);
research/e7-index-bench/src/bin/e145_self_describing_node_header.rs:1://! E145：码 2 自描述头与映射树 key 宽的代价——C306（码 2 头不自描述 key 宽）与 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
research/e7-index-bench/src/bin/e132_livelist_carrier_recount.rs:213:        assert_eq!(tree_table_entries(Arm::OneLivelistPerHead, 1024), 3075);
research/e7-index-bench/src/bin/e68_inline_threshold.rs:40:const THRESHOLDS: [u64; 8] = [0, 256, 512, 1024, 2048, 3072, 3584, 4096];
research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs:47:const RECORD_HEADER_BYTES: u64 = 307;
research/e7-index-bench/src/bin/e56_epsilon.rs:1159:        for fanout in [2usize, 3, 17, 34, 68, 119, 170, 204, 238, 256, 273, 290, 307, 324, 339, 341] {
research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs:43:const RECORD_HEADER_BYTES: u64 = 307;
research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:42://! - journal 记录 4096、记录头 307、点名项 56 ⇒ 67 项/条（D23 已定项 12/17）——这两个量本段不判定，
research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:70:const JOURNAL_HEADER_BYTES: u64 = 307;
research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:74:const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES;
research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:310:        (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES,
research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:506:        assert!(67 * JOURNAL_NAMED_ENTRY_BYTES + JOURNAL_HEADER_BYTES <= JOURNAL_RECORD_BYTES);
research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:507:        assert!(JOURNAL_RECORD_BYTES < 68 * JOURNAL_NAMED_ENTRY_BYTES + JOURNAL_HEADER_BYTES);
research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs:271:        // 0.7 继承率下只有 self_born_block_count() 进 livelist：1024 − round(0.7×1024) = 1024 − 717 = 307。
research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs:273:        assert_eq!(naive(&Load::new(1024, 0, 0.7)).entries, 307);
research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs:395:        // 而 naive 在同两格上是 3072 与 33792 —— 两条臂必须分得开。
research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs:396:        assert_eq!(naive(&Load::new(1024, 1, 0.0)).entries, 3072);
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:12:const JOURNAL_HEADER_BYTES_TODAY: u64 = 95;
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:18:const JOURNAL_HEADER_BYTES_MULTI: u64 = JOURNAL_HEADER_BYTES_TODAY - 9 + 16;
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:73:            Arm::OneRecordPerTransaction | Arm::OneTransactionPerFsync => items_fitting(JOURNAL_HEADER_BYTES_TODAY, ITEM_BYTES_TODAY),
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:74:            Arm::ManyTransactionsPerRecord => items_fitting(JOURNAL_HEADER_BYTES_MULTI, ITEM_BYTES_WITH_TRANSACTION),
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:143:    emit(&mut emitter, &format!("name=config record_bytes={JOURNAL_RECORD_BYTES} header_today={JOURNAL_HEADER_BYTES_TODAY} header_multi={JOURNAL_HEADER_BYTES_MULTI} item_today={ITEM_BYTES_TODAY} item_multi={ITEM_BYTES_WITH_TRANSACTION} devices={DEVICES} t_dirty_bytes={DIRTY_THRESHOLD_BYTES} ring_bytes={RING_BYTES} safety_factor={SAFETY_FACTOR} fsync_per_second={FSYNC_PER_SECOND}"));
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:195:        assert_eq!(items_fitting(JOURNAL_HEADER_BYTES_TODAY, 803), 4);
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:196:        assert_eq!(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES_TODAY - 71 * ITEM_BYTES_TODAY, 25);
research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:200:        assert_eq!(JOURNAL_HEADER_BYTES_MULTI, 102);
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:74:/// + 事务号 8 + 提交标记 1 + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16 = 307
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:75:/// （fsid 与 MAC 是 2026-09-14 用户定案加的）。4096 的记录装 (4096 − 307) / 56 = 67 个点名项。
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:76:const JOURNAL_HEADER_BYTES: u64 = 307;
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:1560:/// journal 记录 4096：头 307（字节表六的表序 + D23 已定项 15 的新根段 + 2026-09-14 加的 fsid 与 MAC）+ 点名项数组。
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:1582:/// CRC32C(**本实例内逻辑前一条**记录的 307 字节头，其中 `header_csum` 那 32 字节按零参与)；本实例第一条恒 0、不读盘。
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:1584:    let mut header = previous_record_bytes[..JOURNAL_HEADER_BYTES as usize].to_vec();
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:1618:        writer.assert_position(JOURNAL_HEADER_BYTES, "记录头");
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:1625:        let payload_checksum = castagnoli_crc32(&bytes[JOURNAL_HEADER_BYTES as usize..payload_end]);
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:1661:        let payload_end = JOURNAL_HEADER_BYTES as usize + named_count * JOURNAL_NAMED_ENTRY_BYTES as usize;
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:1665:        if castagnoli_crc32(&bytes[JOURNAL_HEADER_BYTES as usize..payload_end]) != payload_checksum {
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3618:            let relative = offset - JOURNAL_HEADER_BYTES;
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3704:    ("G6", "已收口（2026-09-13 D23已定项19②，2026-09-14改写口径）：previous_hash=CRC32C(**本实例内逻辑前一条**记录的307字节头，header_csum那32字节按零参与)，本实例写出的第一条恒0、不读盘"),
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3786:        ("journal_header", 307, JOURNAL_HEADER_BYTES),
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3790:        ("journal_named_entries_per_record", 67, (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES),
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4237:        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4 + JOURNAL_NEW_ROOT_SEGMENT_BYTES + 8 + 16, JOURNAL_HEADER_BYTES);
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4238:        assert_eq!(JOURNAL_HEADER_BYTES, 307);
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4240:        assert_eq!(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES, 3789, "4096 − 307");
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4241:        assert_eq!((JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES, 67, "4096 的记录装 67 个点名项");
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4644:            (JOURNAL_HEADER_BYTES as usize + 3, "点名项数组"),
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4651:        // 反向：只罩头 [0, 307) 的那个读法对补齐区那一字节说不出话——这一行证明上面第三条不是白抓的。
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4654:        let head_only_before = wide_checksum_with_field_zeroed(&output.record_bytes, JOURNAL_HEADER_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET);
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4655:        let head_only_after = wide_checksum_with_field_zeroed(&padded, JOURNAL_HEADER_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET);
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4705:    /// journal 记录头 307（D23 已定项 15 的新根段 188 + 2026-09-14 加的 fsid 8 与 MAC 16）；
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4718:        assert!(output.record_bytes[291..307].iter().all(|byte| *byte == 0), "记录头末尾 MAC 16 第一版全 0");
research/e7-index-bench/src/bin/e116_pack_settle.rs:25://! - **D23（journal 的角色与格式）**：journal 记录头 307（十个字段 78 + 已定项 7 / 8 / 13 三笔已定增量 17 + 已定项 15 新根段 188 + fsid 8 + MAC 16），登记名 JOURNAL_HEADER_BYTES。
research/e7-index-bench/src/bin/e116_pack_settle.rs:84:const JOURNAL_HEADER_BYTES: u64 = 307; // D23（journal 的角色与格式），登记名 JOURNAL_HEADER_BYTES
research/e7-index-bench/src/bin/e116_pack_settle.rs:156:    let journal_w = if journal && policy != 2 { n * (JOURNAL_HEADER_BYTES + MAP_ENTRY) } else { 0 };
research/e7-index-bench/src/bin/e116_pack_settle.rs:186:         journal_hdr={JOURNAL_HEADER_BYTES} slot_extra={SLOT_EXTRA} n={N} sizes={SIZES:?} batches={BATCHES:?}",
research/e7-index-bench/src/bin/e116_pack_settle.rs:337:        let journal_w = N * (JOURNAL_HEADER_BYTES + MAP_ENTRY);
research/e7-index-bench/src/bin/e116_pack_settle.rs:373:        // 头 277 → 307（已定项 4 加 fsid 8 与 MAC 16、新根段 182 → 188）之后 2.099192。
research/e7-index-bench/src/bin/e116_pack_settle.rs:497:    /// （每搬一个对象一条记录，头 307 + 映射条目 61 = 368 字节 × 100 000）。
research/e7-index-bench/src/bin/e155_fsync_write_volume.rs:26:const RECORD_HEADER_BYTES: u64 = 307;
research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs:44:const RECORD_HEADER_BYTES: u64 = 307;
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:173:const JOURNAL_HEADER_BYTES: u64 = 307; // D23 已定项 4：头 307 字节 = 十个字段 78 + 已定项 7 / 8 / 13 三笔已定增量 17 + 已定项 15 新根段 188 + fsid 8 + MAC 16
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:177:/// D23 已定项 4 逐字：「头是 307 字节（……），占 512 扇区的 60%，其后还余 205 字节（装得下 3 个点名项，
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:203:    if extension_point_bytes > self_witness_room(JOURNAL_HEADER_BYTES, atomic) {
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:343:                "name=self_witness kind=journal_record hdr={JOURNAL_HEADER_BYTES} atomic={atomic} \
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:345:                self_witness_room(JOURNAL_HEADER_BYTES, atomic),
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:378:            self_witness_room(JOURNAL_HEADER_BYTES, 512),
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:380:            self_witness_room(JOURNAL_HEADER_BYTES, 512).min(ROOT_SLOT_CANDIDATE - 1),
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:519:    /// 307 字节头落进 512 扇区之后余 205。
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:522:        assert_eq!(self_witness_room(JOURNAL_HEADER_BYTES, 512), 205);
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:523:        assert_eq!(self_witness_room(JOURNAL_HEADER_BYTES, 4096), 3789);
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:536:    /// 独立算术：min(512 − 307, 256 − 1) = min(205, 255) = 205。
research/e7-index-bench/src/bin/e43_extension_point_budget.rs:540:        let bound = self_witness_room(JOURNAL_HEADER_BYTES, 512).min(ROOT_SLOT_CANDIDATE - 1);
research/e7-index-bench/src/bin/e155_third_run_release_cascade.rs:35:const RECORD_HEADER_BYTES: u64 = 307;
命中行数：72
```

## 十一、字节表 `.claude/kb/layout/01-first-txn.md`「六、journal」要改成什么（我不改 kb）

行号是那份文件 08:2x 的现状。改的只有下面这些行，别的行原样：

- 64 行 w4：「反向链 = CRC32C(w1 的 307 字节头)」→「反向链 = CRC32C(w1 的 311 字节头)」；75 行 t9：「反向链 = CRC32C(w4 的 307 字节头)」→「… 311 字节头」。
- 307 行（记录头那一行）改成：

| 记录头（十个字段，不含事务边界、本次发布内序号、反向链、载荷校验和、新根段、fsid、MAC 七笔增量） | 78 | 见「记录头字段表」，合计 311 | D23（journal 的角色与格式） 已定项 4 | 已定 |

- 308 行（事务边界）之后插一行：

| 本次发布内序号（紧跟提交标记；无符号 32 位，从 1 起） | 4 | w1、w4、t9 都是 1（各自那次发布只有一条记录；空发布也写 1）；一次发布切成 N 条记录时依次 1..N | D23（journal 的角色与格式） 已定项 4 / 已定项 14 注 1 | 已定（2026-09-23 用户定案，被攻过零轮） |

- 309 行（反向链）：「w4 = CRC32C(w1 的 307 字节头，header_csum 按零参与)」→「… 311 字节头 …」。
- 315 行（点名项）：「4096 − 307 = 3789 ⇒ 一条记录最多装 67 项」→「4096 − 311 = 3785 ⇒ 一条记录最多装 67 项」。
- 319–320 行表头说明：「加六笔已定增量」→「加七笔已定增量」；「`format-const: JOURNAL_HEADER_BYTES` 登记 307」→「登记 311」。
- 「记录头字段表」在「提交标记 | 1 | 1 |」那一行之后插一行，反向链那一行改数：

| 本次发布内序号 | 4 | 1（这次发布只有这一条） | D23（journal 的角色与格式） 已定项 4 |
| 反向链 | 4 | t9：CRC32C(w4 的 311 字节头)；w1：0（本实例第一条） | D23（journal 的角色与格式） 已定项 8 / 已定项 10 / 已定项 19 |

- 342 行：「合计 **307 字节**，4096 记录余 3789，装 67 个点名项。」→「合计 **311 字节**，4096 记录余 3785，装 67 个点名项。」
- 各字段在头里的偏移（事务号 78、提交标记 86、**本次发布内序号 87**、反向链 91、载荷校验和 95、新根段 99、fsid 287、MAC 295、头末 311）见第二节那张表；字节表今天不列偏移，要不要加归主 agent。

**交回**：报告全文 `/tmp/claude-1000/impl-m2-p6-header/report.md`（sha256 在交回里现算）；补丁、变异行与第 44 行替代行的路径与 sha256 见第一节。
