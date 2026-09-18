# 实现员报告：「非空」按树表认 + 只做过 mkfs 的池可写挂载（2026-09-17）

## 一、结论一览

| 项 | 结果 |
|---|---|
| 改动一（非空从盘上按树表认） | 做完。`rollback_floor_ceiling` 不再读 journal，`records` 形参删掉，`raise_rollback_floor` 里那次 `scan_journal` 一起删掉 |
| 改动二（只做过 mkfs 的池可写挂载 → 第一个文件版本） | 做完，录制流与 mkfs 同一个进程里跑第一个事务逐字节相同 |
| 改动二（回退到树表 0 条的根，如暖机根 (1, 2)） | **没做成，停在 `todo!`**：条款没写回退行重写实例表时这一版的落点记在哪（第八节①） |
| `NoPublishedVersion` | 删掉；树表 0 条不再报错。另立 `FileVersionWithoutAnyJournalRecord`，只在「所选根有文件而环里零条记录」时报（旧行为保留，第八节⑤） |
| 新测试 | 6 条跑的 + 1 条 `#[ignore]` 全量；6 条各自在副本里改坏一处看着红，全量那条没跑 |
| 变异表 | 追加 7 行（第六节），按门禁 59 号的判法在副本里逐条复跑，7 条都红；我改过附近代码的 7 条旧行也复跑，都红 |
| `check.sh` | 四段全绿（第七节原样）；登记给我的阶段 53 号退出码 0 |
| 基线红集 | 空：改动之前的仓副本 `cargo test --all --no-fail-fast` 23 个 `test result: ok`、0 失败 |

什么现象会推翻「改动一做对了」：固定脚本上换一条会让「inode 树没变而 extent 树变了」或「被抛弃根夹在两条有效根之间」的历史，上限与手算不符；今天钉住它的是第五节 T1–T3。
什么现象会推翻「改动二的形态是照条款推的而不是我定的」：kb 里有一句说只做过 mkfs 的池重开之后写行那次发布要重写实例表或建记账树。我搜到的相反：layout/01-first-txn.md:422「第一次可写挂载要写的区间是空的，第一个事务的字节不变」。

## 二、改过的文件（自列；别的会话的改动在同一批文件里，`git diff --stat` 分不出）

| 文件 | 这一轮改了什么 |
|---|---|
| `crates/singlefs-core/src/mount.rs`（git 看是未跟踪文件，不进下面的 stat） | `rollback_floor_ceiling` 按树表认非空并去掉 `records`；新增 `user_visible_trees_changed`、`PreviousVersion`、`rebuild_previous_version`、`format_time_allocator`、`RowPublishIdentity`、`publish_rows_on_file_version`、`publish_empty_after`；`establish_instance` / `mount_writable` / `mount_rollback` 按上一版有没有文件分两条；`Mounted.current`、`MountOutput.row_publish`、`warm_up_publishes` 换成 `PoolVersion`；删 `NoPublishedVersion`、加 `FileVersionWithoutAnyJournalRecord`；4 处 `todo!`；单测模块 `non_empty_root_tests` |
| `crates/singlefs-core/src/recovery.rs` | `rebuild_version` 改签名：记录变 `Option`，返回 `RebuiltVersion { WithoutFile, WithFile }`，错误 `RebuildVersionFailure`；新增 `UserVisibleTreeRootPointers`、`user_visible_tree_root_pointers`、`tree_table_has_no_entries` |
| `crates/singlefs-core/src/transaction.rs` | 新增 `ZeroUnitPublishPlan`、`ZeroUnitPublishOutput`、`publish_without_units`、`PoolVersion`（带 `root` / `record` / `record_bytes` / `file_version` / `into_file_version`）；`warm_up` 改成循环调 `publish_without_units`（字节不变，第一个事务的全部用例照绿） |
| `crates/singlefs-core/src/records.rs` | `TreeTableEntry::root_pointer_bytes`（取条目里根指针那 86 字节盘上原样）与私有常量 `TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES`，`to_bytes` 里加一句位置断言 |
| `crates/singlefs-harness/tests/common/mod.rs` | 抽出 `memory_pool_of`、`reopen_recorded_images`、`reopen_cold_images`、`formatted_recorded_images`；新增 `FormattedPool` 与 `format_pool`；`build_pool` 改走共用的建镜像 + mkfs |
| `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs`、`…_step_four_rollback.rs`、`…_step_five_reuse.rs`、`…_step_zero_layer0.rs` | 跟 `PoolVersion` 改调用点（`.into_file_version().expect(…)`、`.root()`、`.record()`），断言一句没改；`step_five_reuse.rs` 另加 T1、T2 |
| 新文件 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs` | T4 |
| 新文件 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs` | T5、T6、T7（全量，ignored） |
| `crates/mutations.tsv` | 末尾追加 7 行（第六节），没动别人的行 |

`git diff --stat -- crates litmus` 原样（含别的会话此前的未提交改动；mount.rs、instance_table.rs 与 step three / four / five 的测试文件、两个新文件是未跟踪文件，不在里面）：

```
 crates/mutations.tsv                               |   50 +-
 crates/singlefs-checker/src/image.rs               |   14 +-
 crates/singlefs-checker/src/walk.rs                |   99 +-
 crates/singlefs-core/src/allocator.rs              |  341 ++++++-
 crates/singlefs-core/src/lib.rs                    |    2 +
 crates/singlefs-core/src/records.rs                |   18 +-
 crates/singlefs-core/src/recovery.rs               |  486 +++++++++-
 crates/singlefs-core/src/transaction.rs            | 1023 +++++++++++++-------
 .../src/bin/first_transaction_on_device.rs         |  164 +++-
 crates/singlefs-harness/src/scenario.rs            |    4 +-
 .../tests/checker_known_bad_images.rs              |   66 ++
 crates/singlefs-harness/tests/common/mod.rs        |  188 +++-
 .../tests/first_transaction_step_five_publish.rs   |   29 +-
 .../tests/first_transaction_step_seven_layer0.rs   |    6 +-
 .../tests/first_transaction_step_six_recovery.rs   |    5 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 +-
 .../tests/second_transaction_step_one_overwrite.rs |   40 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  292 +++++-
 18 files changed, 2337 insertions(+), 494 deletions(-)
```

## 三、改动一：「非空」从盘上按树表认

压着的条款（整行抄，`.claude/kb/decisions/16-发布语义.md` 自己的行号）：

- 第 374 行：`| 抬 F 的上限 | min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根) |`
- 第 383 行：`⚠️ **「非空」从盘上怎么认（2026-09-17 用户定案）**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同。用户选的是「比较树表与上一条根」；比的是树表里这两棵树的条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元，按落点比会把空发布全算成非空（主 agent 核代码时收窄，`crates/singlefs-core/src/transaction.rs` 里空发布也重写树表）。`

实现（`crates/singlefs-core/src/mount.rs`）：

- 第 394 行 `user_visible_trees_changed`：两棵树的根指针字节任一不同就算非空（纯函数，单测钉住）。
- 第 410 行 `rollback_floor_ceiling(devices, superblock, current_floor, table)`：有效 = 可读根槽 ∧ 不被传进来的实例表抛弃 ∧ txg ≥ current_floor；第 447 行 `previous_candidates = &valid`：前一条 = 有效根里 (txg, 实例) 比它小的最大那条；没有前一条的与 `UserVisibleTreeRootPointers::ABSENT`（树表里没有这两棵树的条目）比。根指针字节取树表条目里那 86 字节盘上原样（`recovery.rs` 第 702 行 `user_visible_tree_root_pointers`，`records.rs` 第 295 行），不经 `NodePointer` 解析（解析会丢掉第一版恒 0 的 MAC / nonce 段）。
- 「按 txg 排」我写成按 (txg, 实例) 排：同 txg 不同实例的两条根（设备失而复得那一格）在条款里没有先后，择根序（D22（单元原子性怎么合成） 已定项 7）是 txg 为主、实例破平局，照它排。同 (txg, 实例) 的重复根（mkfs 刚种下时三个区域各一份第 0 代根）互相不算前一条。这一句是我按择根序推的，第八节⑥列出。
- 手算与用例对得上的两格：到 E 的脚本上限 11（非空 14、13、12、11；D 与 A 的根指针相同 ⇒ 空；0、1、2、10、15、16 空；`raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused` 与 T1 钉住）；回退之后只覆盖写三次时上限 3（非空 13、12、11、3；每块盘最新 12 / 13；T2 钉住）。只手算没钉的一格：步 5 扣住那条用例（A、B、重开、六次覆盖写）上限 10，那条用例只抬到 8，照旧绿。

## 四、改动二：只做过 mkfs 的池可写挂载

形态（`mount.rs` 第 682 行 `establish_instance`、第 805 行 `mount_writable`）：恢复择到第 0 代根、零条记录 → `rebuild_version` 报 `RebuiltVersion::WithoutFile` → 分配器 = 空图 + mkfs 的实例表 2 槽与树表 1 槽（第 365 行 `format_time_allocator`，落点取根记录里那两条指针）→ 取号 1 → 要写的行为空 → 写行那次发布 = 零单元发布 txg 1 / jsn 1 / 事务号 0 / 反向链 0 / 落盘 1（`transaction.rs` 第 378 行 `publish_without_units`）→ 暖机一次零单元发布 txg 2 / jsn 2 / 落盘 0 → `current` = `PoolVersion::WithoutFile`；测试里接着 `publish_first_file` 发 txg 3。

逐条写「照哪一句推的」：

| 判断 | 照哪一句 |
|---|---|
| 上一个实例是 0 时不写行、写行那次发布不重写实例表、零单元 | layout/01-first-txn.md 第 422 行「每次可写挂载都写行（实例 0 不写，第一次可写挂载要写的区间是空的，第一个事务的字节不变）」；D16（发布语义） 第 192 行「第一次可写挂载的暖机时树表 0 条 ⇒ 零单元」；D18（块里携带什么信息） 已定项 11 第 882 行「每次可写挂载都写行（实例 0 不写」。代码形态：`transaction.rs` 原 `warm_up` 那条零单元发布，现在 `warm_up` 与这里共用 `publish_without_units` |
| 零单元发布的记录新根段、根记录照上一版的根，只换 txg、实例、F | 原 `warm_up`（「新根段照 mkfs 的根……根记录只改 checkpoint_txg 与实例代号」）；F 写恢复后生效的 F，与带文件那一支写行发布同一个来源 |
| jsn 从 1 起、txg = max(根环, 记录) + 1 = 1 | D23（journal 的角色与格式） 第 1240 行第 3 条「新实例从前缀末 + 1 接着写」与 checkpoint_txg 那句；代码里 `next_counter`、`first_txg_of_new_instance` 原样，零条记录时本来就得 1 |
| 暖机次数按「本实例的根覆盖每块盘」现算，这里 1 次 | 原暖机循环（行号 26 那条变异钉的那句条件）原样；txg 1 落盘 1、txg 2 落盘 0 |
| 树表 0 条的一版的账 = mkfs 那两个单元 | `tests/common/mod.rs` 的 `build_pool`（`PoolAllocator::new` + `mark_format_time_units`）与字节表五「mkfs 两个单元分配代 0」；D23（journal 的角色与格式） 已定项 14「defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入」在没有记账树的根上就是 mkfs 那份 |
| 第一个文件版本走 `publish_first_file`、换下 mkfs 那片第 0 版树表 | 同第 422 行「第一个事务的字节不变」：`publish_first_file` 写出的就是第一个事务的字节；`publish_overwrite` 要带文件的上一版，类型上接不上 `PoolVersion::WithoutFile`。释放 mkfs 树表是 `format_time_tree_table_to_release` 原有的路。T4 断言录制流与 `build_pool` 那条逐字节相同、`TransactionOutput` 相等 |

⚠️ `publish_first_file` 里 txg 与 jsn 写死 3（`FIRST_TRANSACTION_TXG`）。只做过 mkfs 的池走到「要写的行为空」这一支时必然是 txg 2、jsn 2 之后（取号写进超级块在任何记录之前，超级块或根环里出现过实例 1 就会取到 2、要写中间实例行、落进 `todo!`），所以今天这条路上它恰好对；它不是一个通用的「在任意无文件版本上发第一个文件」的函数，我没加这样的函数（第八节⑨）。

## 五、新测试，每条怎么证明会红

副本：`/tmp/claude-1000/impl-nonempty-emptypool-0917/mutation-copy`（`rsync -a --exclude target --exclude .git`，自己的 `target`）。先跑不改动的副本：`singlefs-core --lib` 39 过、`second_transaction_step_five_reuse` 10 过、`…formatted_pool` 1 过、`…formatted_pool_layer0` 2 过 1 忽略，基线红集为空。仓里 `debug_assert` 零处，红的都是测试自己的断言或 `todo!`。每条变异跑「那条测试所在的整个测试二进制」；脚本与逐条日志在 `/tmp/claude-1000/impl-nonempty-emptypool-0917/mutation_runner.py`、`mutation_plan.json`、`logs/`。

| # | 测试（文件:行） | 改坏哪一处 | 红在哪条断言 | 同一个二进制里同时红的 |
|---|---|---|---|---|
| T1 | `torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling`（step_five_reuse.rs:321） | mount.rs:462 `user_visible_trees_changed(&pointers_of(root), &previous_pointers)` 换回按记录判（环里有它自己那条记录且事务号非 0） | 第 351 行 `matches!(…ceiling: CheckpointTxg(11)…)`，实得 `RollbackFloorAboveCeiling { requested: 12, ceiling: 3 }` | 无（其余 9 过） |
| T2 | `the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it`（step_five_reuse.rs:368） | mount.rs:447 `previous_candidates = &valid` → `&readable` | 第 380 行 `matches!(…ceiling: CheckpointTxg(3)…)`，抬到 4 成功了（`err()` 为 None） | `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable`：变异后去读被改坏树表的被抛弃根 C，撞上 mount.rs:441 的 `todo!` |
| T3 | `root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root`（mount.rs:1018，单测） | mount.rs:398–399 函数体只留 `inode_tree !=` 那半 | 第一条断言「只有 extent 树的根指针变了」（今天 mount.rs:1020；跑变异时测试名还带 `a_` 前缀、那条在 1019，改名后按门禁判法复跑仍红） | 无（lib 其余 38 过）；step_five_reuse 整个二进制 10 条全过——固定脚本上两棵树总是一起换，这一格只有单测抓得到 |
| T4 | `writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold`（formatted_pool.rs:60） | ① mount.rs:384 `format_time_allocator` 不调 `mark_format_time_units`；② mount.rs:672 `publish_empty_after` 无文件那一支 `back_chain` 写 0；③ mount.rs:385 mkfs 实例表按 `TreeTable`（1 槽）记 | ①③ 第 142 行 `(allocated, deferred) == (3, 0)`；② 第 122 行暖机那组元组（反向链） | 这个二进制只有这一条 |
| T5 | `the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence`（formatted_pool_layer0.rs:120） | transaction.rs:403 `publish_without_units` 删掉记录与根之间那道屏障 | `prepare` 第 54 行段序列 `[2, 2, 1, 2, 2, 1, 18, 2, 1, 2]` | T6（同在 `prepare` 里红）。⚠️ 这条变异 T4 不红：`build_pool` 的 `warm_up` 共用同一个函数，参照流跟着变 |
| T6 | `every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims`（formatted_pool_layer0.rs:128） | ③（mkfs 实例表按 1 槽记） | 第 94 行 I-3.1 违例状态数必须是 0，首处「盘 0：记账的已分配 Some(196608)，遍历全部有效根得到 212992」 | 无（T5 过）；①也让它红（第 152 行 oracle「走读失败：UnitUnreadable { slot: 50176 }」），②不让它红 |
| T7 | `full_enumeration_of_the_formatted_pool_mount_stream_is_exhaustive_and_clean`（formatted_pool_layer0.rs:170，`#[ignore]`） | 没跑 | — | — |

## 六、`crates/mutations.tsv` 追加的 7 行（第 62–68 行）

| 行 | 变异名 | 点名必红 |
|---|---|---|
| 62 | 步 5：「非空」退回按环里记录的事务号认（txg 14 的记录两份都坏了它就算空根，上限掉到 3） | T1 |
| 63 | 步 5：「非空」的前一条取任意可读根（不排除被抛弃的根与 F 之下的根） | T2 |
| 64 | 步 5：「非空」只比 inode 树的根指针、不比 extent 树的 | T3 |
| 65 | 步 3：只做过 mkfs 的池重开后分配器不认 mkfs 写在单元区里的两个单元 | T4 |
| 66 | 步 3：只做过 mkfs 的池上零单元暖机的反向链写 0 | T4 |
| 67 | 步 3：零单元发布在记录与根之间少一道屏障 | T5 |
| 68 | 步 3：只做过 mkfs 的池重开后把 mkfs 实例表按 1 槽记 | T6 |

按门禁 59 号的判法（原文恰好一次、改坏、`cargo test --offline <参数>`、`^test (\S+::)?名字 ... FAILED$`）在副本里复跑这 7 行：7 行都是 `RED`（改名之后 62、64 两行又复跑一次，仍 `RED`）。全表 63 行原文命中数逐行核过，0 行腐化。我挪动过附近代码的 7 条旧行（25 不写行、26 暖机只推一次、31 回退行不带回退位、32 不写中间实例行、33 jsn 接 R_old 之后、39 上限不看第 4 新、45 普通重开不隔离）同样复跑，7 条都 `RED`。没跑门禁 59 号全量（不归我）。

## 七、门禁

`nice -n 19 bash .claude/scripts/check.sh`（主工作区，改完、改名、fmt 之后跑的；日志 `/tmp/claude-1000/impl-nonempty-emptypool-0917/check.log`），四段汇总行原样：

```
  ✓ 格式通过
  ✓ clippy 通过
  ✓ 构建通过
  ✓ 单测通过
```

末尾 25 行原样（`exit=0` 是我追加的退出码）：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_core

running 1 test
test crates/singlefs-core/src/address.rs - address (line 3) - compile fail ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests singlefs_format

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit=0
```

`test result: ok` 共 25 行（基线 23 行 + 两个新测试二进制）。

登记给我的阶段（`stage-owners.tsv` 里 implementation-writer 只有 53 号）：`nice -n 19 bash .claude/gate.d/53-format-const-placeholders.sh` 末行原样与退出码：

```
  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））
exit=0
```

另跑了命名检查（不归我，顺手）：`naming-lint.sh` 第一次报两个测试名以单字母 `a` 开头，改名之后「✓ 命名纪律通过：查了 175 个 .rs 文件、30837 个声明的名字」。开跑前 `ps` 看到门禁 54 号的 release 层 0（pid 2231969、2232041）与另一个会话的 87 号在跑，没有性能测量；我的构建全在 debug 与副本自己的 target 里，没碰 `target/release`。

## 八、停下交主 agent 的设计判断

四处 `todo!`（`crates/singlefs-core/src/mount.rs`）与另外几件没替条款定的事：

| # | 在哪 | 卡在哪条条款、为什么推不出 |
|---|---|---|
| ① | mount.rs:930 `mount_rollback`：目标根的树表 0 条（取号之前就停） | 验收里「第一个事务之后回退到暖机根 (1, 2) ⇒ 成功、冷启动没有文件、checker 全成立」**没做、没写测试**。回退那次发布要在 R_old 那一版实例表上写回退行（D23（journal 的角色与格式） 已定项 14 回退段），也就要重写实例表；而这一版没有分配记录树与记账树，D16（发布语义） 第 192 行只说「第一次可写挂载的暖机时树表 0 条 ⇒ 零单元」。没定的：这次重写实例表的落点记在哪（建不建记账树、建的话树表里有哪几条、映射几条）；换下的 mkfs 实例表 50176 谁护着——候选根 (0,0) (1,1) (1,2) 都还引用它，影子账按 G5 豁免候选根引用的槽，没有记账树就没有 defer 记录挡着，下一版文件的数据单元按「最低偶数空槽」正好落 50176（我推的，没量） |
| ② | mount.rs:736 `establish_instance`：所选根的树表 0 条而要写的行不为空（取号之后才停，超级块已经写了新号） | 与①同一个缺口。**不靠坏盘也走得到**：mkfs 同一个进程里做第一次可写挂载，暖机两次落盘之后、第一个文件版本落盘之前崩溃，重开时所选根 (1, 2)、新实例 2、要写 (1, 2, 0)；取号那两写只落了一块盘时新实例 2、要写 (1, 0, 0)。改动之前这两格报 `NoPublishedVersion`，今天 panic |
| ③ | mount.rs:369 `format_time_allocator`：树表 0 条、而实例表或树表指针的诞生 txg 不是 0 | 今天实现的路径走不到（只有①②实做之后才可能出现这样的根）；这样的一版的账从哪来，没有条款 |
| ④ | mount.rs:441 `rollback_floor_ceiling`：一条有效根的树表读不出或解不开 | D16（发布语义） 第 383 行只说比树表里两棵树的根指针；读不出时它算不算非空、它后面那条跟谁比，没有条款（主 agent 给的「有效 = 可读 ∧ …」里的「可读」我读成根槽自证过；读成「树表也读得出」的话它不算有效，上限跟着变）。抬 F 时坏盘走得到 |
| ⑤ | `MountError::FileVersionWithoutAnyJournalRecord`（mount.rs:36） | 所选根有文件而环里一条自证过的记录都没有时仍拒绝可写挂载（改动之前也拒，报的是 `NoPublishedVersion`）。重建出来的上一版里那条记录只占位（代码注释原话「上一版的记录只有 jsn 会被用到，而 jsn 下面另算」），拒绝可能不必要；但要放行就得凭空造一条记录填进 `TransactionOutput`，我没造 |
| ⑥ | mount.rs:447 附近 | 「前一条有效根（按 txg 排）」写成按 (txg, 实例) 排、同 (txg, 实例) 的重复根不互为前一条：照择根序推的，条款没写同 txg 不同实例怎么排 |
| ⑦ | `MountOutput` 的报法 | 零单元那一支把 txg 1 报成 `row_publish`、txg 2 报成 `warm_up_publishes[0]`（与带文件那一支同形：新实例的第一次发布算写行发布）；字节与 `warm_up` 两次空发布相同，`WARM_UP_EMPTY_PUBLISHES = 2` 这个常量没动也没用上 |
| ⑧ | `raise_rollback_floor` | 仍只收 `&mut TransactionOutput`：现行版本没有文件时类型上调不了。还没发布过文件的池上抬 F 条款没说，我没加这一支 |
| ⑨ | `publish_first_file` | 没改；它写死 txg / jsn 3，只在第四节说的那一格上恰好对。「在任意无文件版本上发第一个文件」的通用发布（txg、对象出生代、改动计数、事务号怎么取）条款没写，①②实做时会用到 |
| ⑩ | 为 `PoolVersion` 加的非分支项 | `root`、`record`、`record_bytes`、`file_version`、`into_file_version` 五个访问器与 `#[derive(Clone, Debug, PartialEq, Eq)]`；`ZeroUnitPublishPlan` / `ZeroUnitPublishOutput` / `UserVisibleTreeRootPointers` / `RebuiltVersion` / `RebuildVersionFailure` 的 derive；三处 `#[allow(clippy::large_enum_variant, reason = …)]`。都是类型搬运，不承载条款没写的行为 |

## 九、层 0：给门禁 54 号接的东西

- 全量测试：`cargo test --release -p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- --include-ignored --nocapture`，用例 `full_enumeration_of_the_formatted_pool_mount_stream_is_exhaustive_and_clean`，打印一行以 `LAYER0F` 开头（字段与 `LAYER0B` 同形），认 `exhaustive=true`。
- 状态数：闭式 262165（十段 `[2, 2, 1, 2, 2, 1, 18, 2, 1, 2]`：1 + 六个 2 写段各 3 + 三个 1 写段各 1 + 18 写段 262143），与 mkfs 同一个进程里跑第一个事务那条流段序列相同，release 下耗时估计与 `first_transaction_step_seven_layer0` 的全量那条同量级（没量）。
- 快的那条 `every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims`：22 个状态、0 违例，平时 `cargo test` 里跑。段序列那条 `the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence` 钉段序列与闭式。
- 全量那条我没跑（主 agent 交代别跑 release 全量；debug 全量更重）。

## 十、kb 里跟着过时的句子（我不写 kb，列给主 agent / kb-scribe）

- `.claude/kb/milestone/02-second-txn.md` 第 135 行步 3 现状「树表空就报 `NoPublishedVersion`，第一版只支持至少发过一版文件的池」。
- 同文件第 189 行步 5 现状「非空 = 环里有它自己那条记录且事务号非 0，预想」与末尾决策点「checker 怎么认『非空持久有效根』（今天按环里记录的事务号）」：实现今天按树表认；checker 仍没有判上限的检查。
- `layout/01-first-txn.md` 八的段序列登记表没有「只做过 mkfs 的池重开之后的可写挂载」一行；它的段序列与第一次可写挂载 + 暖机 + 第一个事务相同（T5 钉住）。

## 十一、没做什么

- 回退到树表 0 条的根（①）、树表 0 条而要写行（②）没实现、没写测试；验收里「第一个事务之后回退到暖机根 (1, 2)」那条用例没有。
- 全量层 0（T7）没跑，没证明它会红；门禁 54 号脚本没改（不归我）。
- 没跑门禁 59 号全量、没跑 `gate.sh`；没走三方对抗（56 号要的判决文件归主 agent 那一轮）；层 0 全量、QEMU、herd7 与 crates 变异表的正式复跑归 `crash-verifier`。
- 没写 kb，没提交，没做 git 写操作。

## 续做：四处 todo 换成拒绝

收到时间：交回第一份报告之后，协调者来信要求把四处 `todo!` 都换成「在任何落盘动作之前返回一个 `MountError` 成员」，设计判断仍不做。**本节取代第八节 ①–④ 里的「停在 `todo!`」、第一节「回退到暖机根 (1, 2) 没做成，停在 `todo!`」与第五节 T2 那格里「撞上 mount.rs:441 的 `todo!`」三处说法**；现在 `grep -rn "todo!(" crates --include=*.rs` 零命中。

### 续·一　四处改成什么（行号是 `crates/singlefs-core/src/mount.rs` 今天的）

| 原 `todo!` | 新成员（mount.rs） | 在哪里返回、为什么在写之前 |
|---|---|---|
| ② 树表 0 条而要写行（原第 736 行，取号之后） | 第 58 行 `InstanceRowsOnVersionWithoutFileUnsupported { chosen_root, first_row_instance, instance_to_acquire }` | 第 744 行 `refuse_instance_rows_on_version_without_file`，第 777–778 行在 `acquire_instance` 之前调：先用 `transaction::instance_generation_to_acquire`（transaction.rs:287，只读盘，取号的 max(超级块, 根环) + 1 抽出来的那一段，`acquire_instance` 自己也调它）算出要取的号，行区间 [max(所选根的实例, 1), 要取的号) 不为空就返回。之前 `mount_writable` 只做读（恢复、重建上一版、重建分配器），所以一个写、一道屏障都没发。原来那一处改成第 820–823 行的 `assert!(rows_written.is_empty(), …)`：取号用同一个函数、两次之间没有写，走不到（前提是同一时刻只有一个可写挂载，D18（块里携带什么信息） 已定项 11 的过半独占打开） |
| ① 回退到树表 0 条的根（原第 930 行） | 第 65 行 `RollbackToVersionWithoutFileUnsupported(RollbackTarget)` | 第 1016–1017 行，原位置：目标根在环里、在候选集之后，重建上一版与取号之前；之前只读盘 |
| ④ 算抬 F 上限时有效根的树表读不出（原第 441 行） | 第 75 行 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { root, failure }` | `rollback_floor_ceiling`（第 449 行）改返回 `Result<CheckpointTxg, MountError>`：第 483 行 `pointers_of` 读不出就报这个成员，循环里 `?` 传出；`raise_rollback_floor` 第 569 行 `?` 直接返回——它在重算影子账、回收、扣住、发布之前。没有有效根照旧报 `Recovery(NoValidRoot)`。为了不动别人那条变异（第 39 行，锚点 `    Some(newest_on_every_device.min(fourth_newest))`），「排序、取第 4 新、取 min」挪进第 522 行 `ceiling_from_newest_and_non_empty_roots`，那一句原样留着 |
| ③ 树表 0 条而实例表或树表诞生 txg 不是 0（原第 369 行） | 第 68 行 `VersionWithoutFileNotWrittenByMakeFilesystem { root, instance_table_birth_txg, tree_table_birth_txg }` | `format_time_allocator`（第 393 行）改返回 `Result`，第 400 行返回；`rebuilt_allocator`（第 320 行）改返回 `Result`，`mount_writable` 第 934 行 `rebuilt?`、`mount_rollback` 第 1063 行 `)?`。可写挂载那一处先绑 `let rebuilt = …;` 再解构，为的是不动别人那条变异（第 45 行，锚点 `&|_| false,\n        ShadowLedger::On,\n    );`）。两处都在 `establish_instance` 之前，没有写 |

③ 为什么今天实现的路径走不到（没造用例）：这个实现写出的根里，树表 0 条的只有三种——mkfs 的第 0 代根（两个指针的诞生 txg 都是 0）、`warm_up` 与 `publish_without_units` 的零单元发布（指针照抄上一版的根）；重写实例表或树表的只有 `publish_version`，它写出的树表恒有 7 条；而树表 0 条时要写行、要回退都已在写之前拒绝（②①）。所以树表 0 条的根指向诞生 txg 非 0 的实例表或树表，只能来自坏盘或别的写者：根槽自证校验和、单元整单元 CRC 都对得上的伪造字节。

每个新成员：`MountError` 只有 `#[derive(Debug)]`，全仓没有对它的穷举 `match`（测试里只用 `matches!`）；我自己加的 `match`（`refuse_instance_rows_on_version_without_file` 里对 `PreviousVersion` 两个成员）没写通配臂。

### 续·二　这一段改过的文件

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-core/src/mount.rs` | 四个新 `MountError` 成员；`refuse_instance_rows_on_version_without_file`；`format_time_allocator`、`rebuilt_allocator`、`rollback_floor_ceiling` 改返回 `Result`；`ceiling_from_newest_and_non_empty_roots`；四处 `todo!` 换成返回或断言 |
| `crates/singlefs-core/src/transaction.rs` | `acquire_instance` 开头那段抽成 `highest_superblock_instance`（私有）与 `instance_generation_to_acquire`（pub，只读），`acquire_instance` 调它们，行为不变 |
| `crates/singlefs-harness/tests/common/mod.rs` | `DiskSnapshot` 与 `disk_snapshot`：两盘四个超级块槽的原样字节 + 根环全部自证过的根 + 录制流步数 |
| `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs` | 新用例 U1 |
| `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` | 新用例 U2（代替原来「回退到暖机根成功」那条验收） |
| `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` | 新用例 U3，辅助 `allocator_state` / `AllocatorState` |
| `crates/mutations.tsv` | 追加 3 行（第 69–71 行）；改了**我这一轮自己加的**第 62 行的原文与替换文（它压的那一句被改写了），没动别人的行 |

⚠️ 改第 62 行时 Edit 工具把行里一个紧跟在替换文后面的制表符吞了，那一行一度只有五段；逐行核段数时发现（「rows 66 rotted 0 malformed 1」），补回之后 66 行全是六段、原文全部恰好命中一次。

### 续·三　新用例怎么证明会红

副本 `/tmp/claude-1000/impl-nonempty-emptypool-0917/mutation-copy`（与主工作区 `crates/` 逐文件相同之后再跑）。先跑不改动的副本：`…formatted_pool` 2 过、`…step_four_rollback` 7 过、`…step_five_reuse` 11 过，基线红集为空。每条跑整个测试二进制；计划与日志在 `mutation_plan_followup.json`、`mutation-results-followup.json`、`logs/`。

| # | 用例（文件:行） | 改坏哪一处 | 红在哪条断言 | 同一个二进制里同时红的 |
|---|---|---|---|---|
| U1 | `writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance`（step_three_formatted_pool.rs:62）。造法：mkfs 之后同一个进程里取号 1、暖机两次（txg 1、2 落盘）就停、镜像关掉——与「暖机之后、第一个文件版本之前崩溃」是同一个盘面，没有用截断录制流；重开可写挂载 | ⓐ mount.rs:778 的拒绝挪到 779 `acquire_instance` 之后（变异表第 69 行）；ⓑ 删掉 778 那一行拒绝（等于回到「取号之后才发现」） | ⓐ 第 102 行「两盘超级块槽逐字节不变：没有取号」（拒绝照样返回同一个成员，但超级块已写进新号）；ⓑ 不在测试断言上红，是 mount.rs:820 的 `assert!` 在取号之后 panic | 无（同文件另一条过） |
| U2 | `rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write`（step_four_rollback.rs:87）：第一个事务之后重开回退到 (1, 2) | ⓐ mount.rs:1016 条件前加 `false &&`（变异表第 70 行）；ⓑ 1017 的返回改回 `todo!` | ⓐ 第 97 行 `matches!(… RollbackToVersionWithoutFileUnsupported …)`，实得 `InstanceRowsOnVersionWithoutFileUnsupported { chosen_root: (1, 2), first_row_instance: 1, instance_to_acquire: 2 }`（被 U1 那道拒绝接住，盘上照样没写）；ⓑ panic「not yet implemented: 回退到树表 0 条的根」 | 无（其余 6 过） |
| U3 | `raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable`（step_five_reuse.rs:427）：回退之后覆盖写四次，把有效根 txg 12 的树表两盘各翻一个字节，抬 F 到 11 | mount.rs:483 的 `match` 里读不出那一臂之前加 `Err(_failure) if true => Ok(UserVisibleTreeRootPointers::ABSENT)`，即按「树表里没有这两棵树」猜（变异表第 71 行） | 第 454 行 `matches!(… RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { root: (3, 12), .. })`，实得抬 F 成功（`err()` 为 None） | 无（其余 10 过） |

U1、U2、U3 还各自断言了「盘上 / 分配器没动」：U1、U2 比 `DiskSnapshot`（超级块槽字节、可读根、录制流步数）；U3 比分配记录、开放段、逐盘六个计数、现行那一版与录制流步数。

### 续·四　`crates/mutations.tsv` 追加的行（第 69–71 行）

| 行 | 变异名 | 点名必红（第六列全名） |
|---|---|---|
| 69 | 步 3：树表 0 条的一版上要写行时，拒绝挪到取号之后（超级块已写进新号才返回） | `writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance` |
| 70 | 步 4：回退到树表 0 条的根不在写之前拒绝 | `rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write` |
| 71 | 步 5：算抬 F 上限时有效根的树表读不出，按「树表里没有这两棵树」猜而不是拒绝 | `raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable` |

③ 没有用例，所以没有变异行。

按门禁 59 号的判法在副本里复跑（`gate59_rows.py`，原文恰好一次 → 改坏 → `cargo test --offline <第五列>` → 认 `^test (\S+::)?<第六列> ... FAILED$`）：69、70、71 三行，改写过的 62 行，本轮加的 63、64 行，以及附近代码被我挪过的别人的 25、26、32、39、45 行，一共 11 行都是 `RED`。全表 66 行六段齐、原文都恰好命中一次。门禁 59 号全量没跑。

### 续·五　门禁

`nice -n 19 bash .claude/scripts/check.sh`（续做改完、fmt 之后；日志 `check-followup.log`），汇总行原样：

```
  ✓ 格式通过
  ✓ clippy 通过
  ✓ 构建通过
  ✓ 单测通过
```

末尾 12 行原样（`exit=0` 是我追加的退出码）：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit=0
```

`test result: ok` 共 25 行，没有一行 failed 非 0。登记给我的 53 号：

```
  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））
exit=0
```

命名检查：「✓ 命名纪律通过：查了 175 个 .rs 文件、30916 个声明的名字」。跑 `check.sh` 之前 `ps` 看到另一个会话在跑 `scripts/mutate.sh e154-…`（research 的 cargo test --release，不是性能测量），宿主层 0 已停；我的构建仍只用 debug 与副本自己的 target。

### 续·六　续做这一段没做什么

- 设计判断一个都没做：没有文件版本的一版上写行、回退到这样的根、读不出树表时的上限、③ 那样的根的账从哪来，都只是拒绝。「第一个事务之后回退到暖机根 (1, 2) 成功」那条验收仍没有（按来信，等设计定）。
- ③ 没造用例、没有变异行。
- 拒绝之后调用方怎么办（只读挂载、走修复）没有接：`mount_writable` 与 `raise_rollback_floor` 只把成员交回。
- 门禁 59 号全量、`gate.sh`、层 0 全量都没跑；没写 kb，没提交。

## 续做二：攻方打中的四处

来信依据 `research/prompts/m2-emptypool-nonempty-r1-opus-output.md`（第二节 2.1、第三节 3.1 / 3.2 / 3.4 末行、第四节 4.1）。仍只做拒绝，不做设计判断。**「非空」判法与 `rollback_floor_ceiling` 这一轮一个字没动**：`recovery.rs` 与续做一之后的副本逐字相同，`mount.rs` 相对续做一之后的副本只改了 `MountError` 成员、`raise_rollback_floor` 里那一处改名、`refuse_formatted_pool_mount_not_shaped_like_the_first_transaction` 与 `establish_instance` 取号那一段（`diff` 核过）。

### 续二·一　每一处改成什么（行号是今天的）

| 条 | 改法 |
|---|---|
| F1（Z2：判定与取号各读一遍超级块） | `transaction.rs:331` 新 `acquire_expected_instance(pool, expected)`：写之前重算号（第 336 行），不等就返回 `ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite { expected, recomputed }`（第 316 行新枚举），一个字节不写；相等才走第 349 行 `write_acquired_instance`（原 `acquire_instance` 的写那一半，抽出来）。`acquire_instance`（第 307 行）对外不变，改成「算一次号 → `write_acquired_instance`」，第一个事务、`scenario.rs`、真设备二进制照旧调它。`mount.rs:829` 把判定时算出的号传进去，第 834 行映射成新 `MountError::InstanceGenerationChangedBeforeAcquisition { expected, recomputed }`（第 82 行）。第 883 行那条 `assert!` 保留 |
| F2（Z3-B：mkfs 不核区域归属） | `make_filesystem.rs:38` 新常量 `FIRST_VERSION_REGION_DEVICES = [盘 0, 盘 1, 盘 0]`（按设备身份 0 / 1 写死，与超级块那 12 字节写的值同一个意思）；`check_geometry` 第 164 行：两块盘时参数不等于它就返回新成员 `RegionDevicesNotTheFirstVersionLayout { region_devices }`（第 75 行）。`check_geometry` 在 `make_filesystem` 的任何写之前 |
| F3 ①（Z3-A：空池挂载形状） | `mount.rs:782` 新 `refuse_formatted_pool_mount_not_shaped_like_the_first_transaction`，第 828 行在取号之前调：上一版树表 0 条时，只放行「`first_txg` = 1 ∧ `next_counter` = 1 ∧ txg 1 与 txg 2 落在不同的盘上」（落哪块盘按 `parameters.region_devices` 与 `target_for_publish`，与暖机循环同一个算法），其余返回新成员 `FormattedPoolMountNotShapedLikeTheFirstTransaction { chosen_root, first_txg, first_counter }`（第 88 行）。放在「要写的行」那道判定之后：两道都中的盘面（暖机之后崩溃）仍报 `InstanceRowsOnVersionWithoutFileUnsupported`，续做一的 U1 不变 |
| F3 ②（`publish_first_file` 写死 txg 3） | `transaction.rs:1000` 在写之前把 `previous_record_bytes` 按本池 fsid 解成记录，checkpoint_txg + 1 与 jsn + 1 都等于 `FIRST_TRANSACTION_TXG` 才往下走（第 1017 行），否则返回 `PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record: Option<(txg, jsn)> }`（第 868 行；`None` = 解不出本池的记录）。⚠️ 来信写「核上一版的根 txg = 2」：核的是记录不是 `genesis` 参数——mkfs 同一个进程里的第一个事务传的是 mkfs 的第 0 代根（txg 0），它只供照抄实例表指针；记录的 checkpoint_txg 就是上一版那条根的 txg |
| F4（Z4：层 0 第三条流） | `second_transaction_step_three_formatted_pool_layer0.rs` 删掉 `#[ignore]` 的全量枚举（连同 `LAYER0F` 那一行打印）；新用例（第 174 行）逐项比「只做过 mkfs 的池可写挂载 + 第一个文件版本」与 `build_pool`（mkfs 同一个进程里的第一个事务）：mkfs 之后的基线镜像 `==`、写表逐条带内容 `==`、段序列 `==`。快测与段序列用例留着。门禁 54 号脚本里本来就没有这个文件（`grep -n formatted_pool .claude/gate.d/54-layer0-replay.sh` 零命中），那一段不用删 |
| F5（改名） | `RaiseNeedsWritableMountInThisProcess` → `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`（`mount.rs:43`，文档注释写明真实条件：现行版本里没有重写过的实例表单元），行为不变；测试改名为 `raising_the_floor_on_a_current_version_without_a_rewritten_instance_table_unit_is_refused_instead_of_panicking`（step_five_reuse.rs:607） |

### 续二·二　这一段改过的文件

`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-harness/tests/common/mod.rs`（新 `crash_state_devices`、`memory_pool_of_sparse_devices`：从 mkfs 基线 + 持久了的写造两块内存盘，外包录制器录进一条新流）、`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`（`TransientSuperblockReadErrorDevice` 与四条新用例）、`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs`、`crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs`、`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`（改名）、`crates/mutations.tsv`。

### 续二·三　新用例怎么证明会红

副本 `mutation-copy`，逐文件与主工作区 `crates/` 相同。⚠️ 这一段中途撞上一次 `rsync -a` 保留旧修改时间的坑：同一份副本上一次编的是 R69 变异，rsync 把 `mount.rs` 的修改时间拨回主树的旧值，cargo 判「新鲜」，没改动的基线跑出两条红（与 R69 变异的结果逐字相同）。之后 `find crates -name '*.rs' -exec touch {} +` 再跑，基线全绿：formatted_pool 6 过、layer0 3 过、step_one_mkfs 7 过、step_five_reuse 11 过、step_four_rollback 7 过、instance_acquisition 4 过。本段表里的红都是 touch 之后或那次 rsync 之前跑的（那次 rsync 之前的一轮基线同样全绿）。

| # | 用例（文件:行） | 改坏哪一处 | 红在哪 | 同一个二进制里同时红的 |
|---|---|---|---|---|
| V1（F1） | `transient_superblock_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write`（formatted_pool.rs:84）。盘面：mkfs 之后第一次可写挂载只做完取号两写（同一个进程里取号就停、镜像关掉），重开时两块盘第 2 次读超级块槽 0 各报一次瞬时读错 ⇒ 返回 `InstanceGenerationChangedBeforeAcquisition { expected: 1, recomputed: 2 }`，`DiskSnapshot` 不变 | transaction.rs:336 `if recomputed != expected {` → `if false {`（变异表第 72 行） | 取号写进号 2 之后 mount.rs:883 的 `assert!` panic（就是攻方腿 Z2 那一格） | 无 |
| 探针（不进变异表） | 同一条用例把「取号就停」那一步删掉，盘面变成只做过 mkfs 的池 | — | 第 115 行 `matches!`，实得挂载成功（`err()` 为 None）：只做过 mkfs 的池上两盘槽 0 都是号 0，第 2 次读错不让两次算号分叉。来信写「只做过 mkfs 的池」，攻方腿的镜像是「取号两写已持久」，用例照后者 | — |
| V2（F2） | `region_layout_other_than_zero_one_zero_is_refused_before_any_write`（step_one_mkfs.rs:281）：[0, 1, 1] 与 [1, 0, 0] 各一次 ⇒ 返回 `RegionDevicesNotTheFirstVersionLayout`，录制流一步没有，两盘超级块槽 0 与三个区域根槽 0 全零 | make_filesystem.rs:164 条件换成 `false`（第 73 行） | 第 316 行 `matches!`，[0, 1, 1] 那一次 mkfs 成功 | 无 |
| V3（F3 ①） | `formatted_pool_mount_starting_after_a_leftover_record_is_refused_before_acquiring_an_instance`（formatted_pool.rs:181）：攻方腿模型第 213 行那段历史——第一次可写挂载的录制流只留段 0、段 1（取号两写 + txg 1 的记录两写），两盘超级块槽 0 各翻一个字节 ⇒ 返回 `FormattedPoolMountNotShapedLikeTheFirstTransaction { chosen_root: (0, 0), first_txg: 2, first_counter: 2 }`，`DiskSnapshot` 与整份镜像都不变 | mount.rs:794 `if shaped_like_the_first_transaction {` → `if true {`（第 74 行） | 第 224 行 `matches!`，实得挂载成功 | 无 |
| V4（F3 ②） | `first_file_version_after_a_third_zero_unit_publish_is_refused_before_any_write`（formatted_pool.rs:250）：mkfs 之后取号、暖机两次、再一次零单元发布（txg 3、jsn 3），在这一版上调 `publish_first_file` ⇒ `FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record: Some((3, 3)) }`，录制流一步不多、分配记录不变 | transaction.rs:1017 `if !follows_directly {` → `if false {`（第 75 行） | 第 298 行 `matches!`，实得发布成功 | 无 |
| V5（F4） | `formatted_pool_mount_stream_has_the_same_base_writes_and_segments_as_the_first_transaction_stream`（layer0.rs:174） | mount.rs `publish_empty_after` 无文件那一支的 `rollback_floor` 写成 `CheckpointTxg(1)`（第 76 行） | 第 190 行「写表第 7 条（JournalRecord）连同内容相同」 | 无（快测、段序列两条过） |
| V6（续做一 U1 那一行的替身） | `writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again`（formatted_pool.rs:141）：只做完取号两写的盘面、不注入 ⇒ `InstanceRowsOnVersionWithoutFileUnsupported { chosen_root: (0, 0), first_row_instance: 1, instance_to_acquire: 2 }`，`DiskSnapshot` 不变 | 「要写的行」那道判定挪到 `acquire_expected_instance` 之后（变异表第 69 行，改写过） | 第 169 行 `DiskSnapshot` 不等（超级块已写进号 2） | 续做一的 U1（第 353 行那条）也红，但红在第 377 行 `matches!`：形状判定先拒了（txg 3 起），盘上没写 |

**为什么多了 V6**：F3 ① 的形状判定在取号之前也拦得住续做一 U1 那个盘面（暖机之后崩溃，新实例从 txg 3 起），于是第 69 行变异（「要写的行」判定挪到取号之后）在 U1 上只让返回的成员变了、盘上照样不动——那一行守的「挪到取号之后就会写进新号」在 U1 上量不出来。V6 用「只做完取号两写」的盘面：形状判定放行（txg 1、jsn 1）、只有「要写的行」那道能拦，挪到取号之后就写进号 2，`DiskSnapshot` 红。第 69 行的第六列改指 V6。

### 续二·四　`crates/mutations.tsv` 这一段的改动

| 行 | 变异名 | 第六列（测试全名） | 这一段做了什么 |
|---|---|---|---|
| 56（别人的行） | 步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错 | `raising_the_floor_on_a_current_version_without_a_rewritten_instance_table_unit_is_refused_instead_of_panicking` | F5 改名让原文与测试名腐化，按来信「测试名跟着改」只改原文、第五列与第六列，变异名、替换文没动 |
| 69（续做一我加的） | 步 3：树表 0 条的一版上要写行时，拒绝挪到取号之后（超级块已写进新号才返回） | `writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again` | 原文随取号那一段改写；第五、六列改指 V6 |
| 72 | 步 3：取号不核判定时算出的号（瞬时读错让判定与取号算出不同号，写进新号之后才发现） | `transient_superblock_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write` | 追加 |
| 73 | 第一个事务 步 1：mkfs 不核根环区域归属是不是第一版写死的 0 / 1 / 0 | `region_layout_other_than_zero_one_zero_is_refused_before_any_write` | 追加 |
| 74 | 步 3：空池挂载不核形状是不是第一个事务那一种（新实例从 txg 1、jsn 1 起，写行与暖机落两块盘） | `formatted_pool_mount_starting_after_a_leftover_record_is_refused_before_acquiring_an_instance` | 追加 |
| 75 | 步 3：publish_first_file 不核上一版的记录是 txg 2、jsn 2 | `first_file_version_after_a_third_zero_unit_publish_is_refused_before_any_write` | 追加 |
| 76 | 步 3：空池挂载的零单元暖机把回退下界写成 1（流与第一个事务那条不再逐项相同） | `formatted_pool_mount_stream_has_the_same_base_writes_and_segments_as_the_first_transaction_stream` | 追加 |

全表 71 行六段齐、原文都恰好命中一次。按门禁 59 号的判法在 touch 过的副本里复跑 16 行：新加的 72–76、改过的 56 与 69、续做一加的 70 与 71、这一轮挪到附近代码的 25（不写行）、26（暖机只推一次）、32（不写中间实例行）、65–68（空池挂载那几条），**16 行全部 `RED`**。门禁 59 号全量没跑。

### 续二·五　门禁

`nice -n 19 bash .claude/scripts/check.sh`（续做二改完、fmt 之后；日志 `check-followup2.log`），汇总行原样：

```
  ✓ 格式通过
  ✓ clippy 通过
  ✓ 构建通过
  ✓ 单测通过
```

末尾 12 行原样（`exit=0` 是我追加的退出码）：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit=0
```

`test result: ok` 共 25 行，没有 failed 非 0 的行。登记给我的 53 号：

```
  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））
exit=0
```

命名检查这一次红：「✗ 45 处名字不合命名纪律（查了 176 个 .rs 文件、31359 个声明的名字）」——45 处全在 `research/e7-index-bench/src/bin/e154_two_gates_serial_rejudge_and_reclaim_timing.rs`（36）与 `research/results/e153-s1-probe-tests/e153_s1_pairing_probe.rs`（9），不在这一轮的改动里（另一个会话的计数实验），没碰。跑之前 `ps` 看到另一个会话在跑 `target/release/e154-…`（research 计数实验，不是 qemu / fio / e152 那类性能测量）；我只用 debug 与副本自己的 target。

### 续二·六　续做二没做什么

- 设计判断一个都没做：空池挂载只放行与第一个事务同形的那一格，别的形状（环里有残留记录、号被取过而没有根）一律拒；`publish_first_file` 仍写死 txg 3 / jsn 3，只加了核对。攻方腿第五节 V2 射程观测（挂上不写就退出，之后每次可写挂载都被拒）照旧成立，没处理。
- 攻方腿 Z1（F 回落之后再抬 F 被合法复用的树表单元挡住）来信没列，按「不许碰非空判法与 `rollback_floor_ceiling`」没动。
- F2 按设备身份 0 / 1 写死；池里两块盘的身份不是 {0, 1} 时，要么先报 `RegionDeviceMissing`，要么报这个新成员——第一版全仓都用 0 / 1，没另外造用例。
- 层 0 全量、门禁 59 号全量、`gate.sh` 都没跑；没写 kb，没提交。
