# 实审 A3c 实现员报告：取号时读不出见证槽就拒；`transaction.rs` 与 `inode_tree.rs` 剩下的 panic 面

写于 2026-09-27。规格 `/tmp/claude-1000/impl-rev-a3c/spec.md`。底座：主工作区现状（开工时 rsync 成 `orig/`，改在 `work/`）。补丁目录 `/tmp/claude-1000/impl-rev-a3c/patch/`。

## 一、结论

- 规格第 1 条（Q1，拒）：做完。`transaction.rs` 新增 `InstanceAcquisitionFailed`（`acquire_instance` 与取号的写那一半改成交它），它和 `ExpectedInstanceAcquisitionFailed` 各加一个成员 `DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness { device }`。`write_acquired_instance` 在第一道屏障**之前**逐盘核一遍，某块盘两槽一份本池自证过的都没有就拒，这时连屏障都没发，录制流一步不多；屏障之后取见证值时再读一遍，这一遍又有盘读不出就不写、报同一个成员（这时录制流只多那一道屏障）。`mount.rs` 把这个成员映射成已有的 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`（那块盘，`NoSelfVerifiedSystemConfiguration`）。见证值改成从这一次读到的结果里现算（`highest_journal_tail_of`）。
- 第 2 条（Q3 两处注释）：照原文改了（`mount.rs:336`、`:540`，副本里的行号）。
- 第 3 条（零单元发布判条数上限）：做完。`publish_without_units` 的错误类型从 `BlockDeviceError` 改成 `PublishError`，在任何写之前调 `refuse_a_publish_of_more_journal_records_than_the_limit`，与带文件的一版、写行那一版用同一个函数。`WarmUpFailed.cause` 跟着改成 `PublishError`。这一格从盘上走不到：可写挂载选系统配置时已经拒在飞上限为 0 的环（A3a 第 38 条），mkfs 也拒。所以用例直接喂一个带一槽环参数的写入口。
- 第 4 条（树表条目按树 ID 升序那条断言）：**从盘上走得到，停下交主 agent**，见第三节 Q-A。
- 第 5 条（重复 key 记账叶那条 assert，今天在 `build_multi_level_tree` 里）：从盘上走不到，靠的是 A3a 在重建时开的 `EveryHeaderAgainstItsReference` 判 key 严格递增。已有用例 `an_accounting_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_before_any_write` 和变异第 1200 行盯着。断言消息和 `# Panics` 文档里写明了它靠的是 `recovery::rebuild_version` 和 `CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference`。
- 第 6 条（C476「缺口」表 `inode_tree.rs:88`）：走不到。实测的坏镜像（inode 叶容器自述 234 条 × 140 字节）让可写挂载在 `unit::parse_packed_unit` 的「记录区越过单元末尾」拒成 `UnitMalformed { what: "inode 叶容器" }`。`free_record_slots` 的 `expect` 消息和 `# Panics` 里写明依赖哪两道读者判定。另加一条单测盯住「一个码 3 单元按 140 字节至多装 233 条」这条算术（普查写的「什么改动会让它炸」就是改记录宽或头宽而不重算）。A3a 那一行末尾列的别的位置都不在 `transaction.rs` 或 `inode_tree.rs`。普查 R11（`transaction.rs` 水位行的 expect）已由 A3a 在重建时报 `InodeNumberWatermarkRowMissingFromTheAccountingTree`（见 `recovery::rebuild_version` 的文档），这次没动。
- A2c 第 5 条（`acquire_instance` 遇到 u32::MAX 仍 panic）：照规格不做，只在 `acquire_instance` 的 `# Panics` 里写清楚什么条件下走得到。
- 新测试 5 条全绿。7 行新变异加 4 行换锚点后的旧行，共 11 行，逐条用 `prove-red.sh` 证红，11 条全抓到（第四节）。

什么现象会推翻「Q1 做对了」：可写挂载时盘 1 两槽从取号那一刻起读坏，挂载做成、或者报了别的成员；或者被拒之后 `DiskSnapshot`（系统配置槽、根环、录制流步数）变了。第一节那三条用例各盯一种。

**打补丁要注意**：`crates.patch`（清单内 4 份文件）单独打上去编不过。`history.rs` 和清单外 3 份测试要跟着改：`publish_without_units` 与 `acquire_instance` 的错误类型变了。这 3 份测试不在「别的会话正在改」清单里，改动做成了 `patch/outside-the-list.patch`（对主工作区 `apply --check` 过）；`history.rs` 在别的会话清单里，我没进补丁，要加的原文在第五节。

## 二、写过的文件

补丁内（`patch/crates.patch`，`git apply --stat` 原样）：

```
 crates/singlefs-core/src/transaction.rs            |  163 ++++-
 crates/singlefs-core/src/mount.rs                  |   23 +
 crates/singlefs-core/src/inode_tree.rs             |   33 +
 ...n_unreadable_and_the_remaining_writer_panics.rs |  677 ++++++++++++++++++++
 4 files changed, 851 insertions(+), 45 deletions(-)
```

- `transaction.rs`：`self_verified_system_configurations_of_every_device`（631 行）、`highest_journal_tail_of`（662 行，替掉原来的 `highest_system_configuration_journal_tail`）、`InstanceAcquisitionFailed`（744 行）以及到它的 `From<AcquisitionFailed>`、`ExpectedInstanceAcquisitionFailed` 的新成员、`write_acquired_instance`（813 行）的两次核、`roll_back_acquisition` 的返回类型、`acquire_instance` 的文档（`# Errors`、`# Panics`）、`WarmUpFailed.cause` 与两处 `# Errors`、`publish_without_units`（1212 行）的条数判定与错误类型、`build_multi_level_tree` 的 `# Panics` 与断言消息（6895 行）。行号都是副本 `work/` 里的。
- `mount.rs`：映射那一臂（3428 行），去掉两处变得多余的 `.map_err(PublishError::from)`（clippy 报 `useless_conversion`），Q3 两处注释。
- `inode_tree.rs`：`free_record_slots` 的 `# Panics` 与 `expect` 消息（96 行），新单测 `a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit`（376 行）。
- 新测试 `crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs`，5 条。
- `lib.rs` 没动：新类型都在 `transaction` 模块里，模块本来就是 `pub`。

补丁外，但为编译和测试在副本里改过、交主 agent 定的：
- `patch/outside-the-list.patch`：`tests/instance_acquisition.rs`、`tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs`、`tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs`。三份都只是在 `acquire_instance` 的错上先解开 `InstanceAcquisitionFailed::Acquisition`，新成员那一臂 panic，并补 `use`。
- `crates/singlefs-harness/src/history.rs`：原文见第五节，不在任何补丁里。

变异表（不进 crates.patch）：
- `patch/mutations-append.tsv` 追加 7 行，名字见第四节表。
- `patch/mutations-replacements.tsv` 换 2 行：「C554 乙-配置续：见证值只读盘 0（整池最大的 tail 只在盘 1 上时带错）」的原文改到 `highest_journal_tail_of` 今天的写法（`.iter()\n        .flatten()` 中间插 `.take(1)`）；「实审A1 第19条：取号回卷写之前那道屏障报错照样写回卷」的原文补上 `.into();`（`roll_back_acquisition` 改成交 `InstanceAcquisitionFailed`）。别的锚点在副本上由 33 号核过，这次没动断。

`git diff --stat -- crates litmus`：没贴。我在副本里改，主工作区一个字没动，主工作区的 diff 全是别的会话的。上面贴的是补丁自己的 `git apply --stat`。

## 三、停下交主 agent 的

### Q-A 第 4 条：树表条目按树 ID 升序那条断言从盘上走得到

- 断言在 `transaction.rs` `publish_admitted` 里（副本 `work/` 第 6508 行的消息；开工时的主工作区是第 6418 行）：`assert!(tree_table_entries.windows(2).all(|pair| pair[0].tree < pair[1].tree), "树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）…")`。
- 实测（在副本里写了临时探针测试，跑完已删）：在 A3a 的 `pool_after_the_first_file` 上改最新那条根的树表单元，重封树表、重算根记录里树表指针的校验和，再 `mount_writable`（用 `catch_unwind` 包住）：
  - 探针一：livelist 与稀疏旁表两条条目**互换种类**（条目偏移 10 的 2 字节）。两条都没有根节点，A3a 判全也核不到它们的树 ID，水位之下的号 I-7.8 放行。结果：`panicked at crates/singlefs-core/src/transaction.rs:6418:5`（开工副本）/ `:6504:5`（改后副本）。
  - 探针二：livelist 条目的树 ID 改成稀疏旁表的号（两条同号）。结果同一处 panic。
  - 两个探针在 panic 之后两块盘都逐字节不变：panic 发生在取号之前的整串预演（`dry_run_of_the_publishes_after_acquisition` → `prepare_the_version_publish`），走的是同一段代码。
- 为什么没做：规格写「走得到就在重建时拒成带名字的成员」。重建在 `recovery.rs`，不在我的文件单里（`rebuild_version` 读树表那一步，`tree_table_entries_each_kind_at_most_once` 旁边）。退一步改在发布路径：写之前返回 `PublishError` 的一个新成员，但 `PublishError` 被 `mounted_session.rs` 的 `refusal_is_short_of_space`（core，也不在单里）、`history.rs` 的 `publish_error_member`、`model_comparison.rs` 的 `refusal_reason_of_publish_error`（这两份在别的会话单里）穷举 match，加成员这三处都编不过。另外 `.claude/kb/invariants.md` 里没有「树表条目按树 ID 升序」这条不变量（`grep -n '树表.*升序\|升序.*树表' .claude/kb/invariants.md` 只命中 I-2.5 那一行，说的是位置条目），`invariant` 字段填什么没有现成的名字。冷走读与 checker 今天也不判它。
- 两种改法，交主 agent 定、另派：
  - 甲（规格原意）：`recovery.rs` `rebuild_version` 在 `tree_table_entries_each_kind_at_most_once` 之后判树 ID 严格升序（相同也拒），报 `RecoveryFailure::InvariantViolated { invariant: <待定名>, detail: "树表条目不按树 ID 严格升序" }`；要不要在 `walk_to_file` 与 checker 同步判、要不要在 invariants.md 立一条，一并定。用例照上面探针一、二各一条，钉「可写挂载在任何写之前拒、盘上逐字节不变」。
  - 乙：`transaction.rs` 装树表之前判，报 `PublishError` 新成员（预演在取号之前就先报出来）。要同时改上面那三处穷举 match。
- `transaction.rs` 这条断言我没动，消息也没改：它今天就是走得到的，改成「依赖哪道读者判定」的消息是假话。

### Q-B `history.rs` 要跟着改（在别的会话单里，没进补丁）

`publish_without_units` 改交 `PublishError` 之后，`crates/singlefs-harness/src/history.rs` 的 `apply_publish_without_units`（开工副本第 2941–2951 行附近；主工作区之后已被别的会话改过，今天在第 2970、2977 行）编不过。改法原文（副本里就是这样改的，编得过、clippy 过）：

```rust
            let member = format!("publish_without_units({})", publish_error_member(&error));
```
```rust
                    refusal_reason_of_publish_error(&error),
```
另外 `use crate::model_comparison::{…}` 里删掉 `refusal_reason_of_block_device_error`（不删就报 unused import）。`model_comparison.rs` 里的 `refusal_reason_of_block_device_error` 是 `pub fn`，没人调也不报警，留不留由那边定。别的 `acquire_instance` 调用点（`history.rs:858`、`scenario.rs:151`、E158 bin 三处、E156 bin）只 `format!("{error:?}")` 或 `.expect`，不用改，`cargo build --all-targets` 在副本上过了。`first_transaction_on_device.rs` 与 E158 bin 里没有 `ExpectedInstanceAcquisitionFailed` / `InstanceAcquisitionFailed` / `PublishError` 的新穷举 match 要补。

### Q-C 条款没写、我没加的非分支项

没有：只加了 `From<AcquisitionFailed> for InstanceAcquisitionFailed`（`?` 与 `.into()` 用），没加 derive 之外的 trait、访问器。

## 四、新测试与证红

新测试文件（下称 N）5 条：

| 测试 | 钉什么 |
|---|---|
| `a_device_whose_slots_turn_unreadable_before_the_acquisition_barrier_refuses_the_writable_mount_without_a_step` | 盘 1 两槽从「取号写之前最后一次读」起读坏。这个次数取另一份逐字节相同的池上照常挂载时、第一道屏障那一刻数到的读数，所以择系统配置、逐盘核、算号那几遍都读得出。钉：可写挂载报 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`（盘 1、`NoSelfVerifiedSystemConfiguration`、所选那一版 (1, 3)），`DiskSnapshot` 不变 |
| `a_device_whose_slots_turn_unreadable_after_the_acquisition_barrier_refuses_before_any_write` | 读坏从再晚一次读起：屏障之前那一核读得出，屏障之后取见证值那一遍读不出。钉：同一个成员，系统配置槽与根环不变，录制流只多屏障、没有写 |
| `acquire_instance_refuses_a_device_whose_slots_are_unreadable_before_any_step` | 不经挂载、直接 `acquire_instance`，盘 1 两槽每次都读坏。钉：`InstanceAcquisitionFailed::DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness { device: 盘 1 }`，`DiskSnapshot` 不变 |
| `a_zero_unit_publish_on_a_ring_with_no_record_in_flight_is_refused_before_any_write` | mkfs 之后的池，写入口参数里的环是一槽（在飞上限 0）。钉：`JournalRecordsOfThePublishExceedTheLimit { 1, 0 }`，两盘逐字节不变，失败账是空的 |
| `an_inode_leaf_container_claiming_more_records_than_a_unit_holds_is_refused_by_the_writable_mount` | 第一个文件那一版唯一那片 inode 叶容器自述 234 条（记录数与声明长度改了，载荷与头重封，inode 根、树表、映射、根记录的校验和都重算）。钉：可写挂载报 `UnitMalformed { what: "inode 叶容器" }`，两盘逐字节不变 |

另在 `inode_tree.rs` 的单测模块里加 1 条：`a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit`。

证红：先把行写进副本 `work/crates/mutations.tsv`，再在副本上跑 `bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a3c/work --memory 8G <crate> <名…>`（`capped.sh 4`）。每组参数的基线都 0 红（日志 `prove-red-logs-*/baseline.log`：`test result: ok`）。

| 变异名 | 改坏哪一行 | 红在哪条断言 |
|---|---|---|
| 实审 A3c Q1：取号屏障之前不核每块盘两槽读不读得出（读不出的盘拒到屏障之后，录制流多一道屏障） | `write_acquired_instance` 里屏障之前那句 `self_verified_system_configurations_of_every_device(pool)?;` 删掉 | N:236 `assert_eq!` 「拒在第一道屏障之前：一个写、一道屏障都没发」（拒成了同一个成员，录制流多一道屏障） |
| 实审 A3c Q1：屏障之后取见证值那一遍读不出的盘不拒、照写取号写 | 屏障之后那一次读的 `?` 换成 `.unwrap_or_default()` | N:261 `assert!`：「得到 挂载做成：取到实例 InstanceGeneration(2)，所选那一版 (1, 3)」 |
| 实审 A3c Q1：取号那一刻两槽都读不出的盘不报、当作没有（acquire_instance 照写取号写） | `if slots_of_this_device.is_empty() {` → `if false && …` | N:300 `assert!`：「得到 Ok(InstanceGeneration(2))」 |
| 实审 A3c Q1：可写挂载把取号那一刻读不出的盘映射错（报成盘 0） | `mount.rs` 映射那一臂 `device,` → `device: DeviceIdentity(u32::from(device.0 == 0)),` | N:230 `assert!`：报了 `WritableMountRefusedByDevicesWithoutTheSelectedVersion` 但盘号是 0 |
| 实审 A3c：零单元发布不判记录条数上限（在飞上限 0 的环上照写） | `publish_without_units` 里的 `refuse_a_publish_of_more_journal_records_than_the_limit(…)?;` → `let _ = writes.records.len();` | N:529 `assert!`：「Ok(CheckpointTxg(1))」 |
| 实审 A3c C476 缺口 inode_tree.rs:88：码 3 解析不判记录区越过单元末尾（自述 234 条的 inode 叶容器） | `unit.rs` `if records_start + declared_length > bytes.len() {` → `if false && …` | 红在 `crates/singlefs-core/src/unit.rs:498:24` 的切片越界 panic（`range end index 32896 out of range for slice of length 32768`），不在 N 的断言上：拿掉这道判定，读者自己先 panic |
| 实审 A3c C476 缺口 inode_tree.rs:88：inode 记录宽改窄而不重算一容器的上限（盘上一片装得下第 234 条） | `singlefs-format/src/lib.rs` `INODE_RECORD_BYTES: u64 = 140` → `120` | `inode_tree.rs:384` `assert!`：「一个码 3 单元装得下 271 条 inode 记录，多于一容器的上限 233」 |
| （换锚点）C554 乙-配置续：见证值只读盘 0 | `highest_journal_tail_of` 里 `.iter()` 后插 `.take(1)` | 原点名测试 T:414 `assert_eq!`，tail 2（期望 3） |
| （换锚点）实审A1 第19条：取号回卷写之前那道屏障报错照样写回卷 | 回卷之前屏障失败那一句 `return … .into();` → `let _ = barrier_error;` | `instance_acquisition.rs:223` 「rollback: RolledBack」 |
| （锚点没改，重证）C554 乙-配置续：见证值取成槽世代号 | `.quantities.journal_tail` → `.slot_generation` | T 的 6 条全红，点名那条读回 `journal_tail: 1` |
| （锚点没改，重证）C554 乙-配置续：见证值取最小的 tail | `.max()` → `.min()` | T:414，tail 2 |

`prove-red.sh` 末行原样：harness 那一趟 `✓ 点名 10 条：跑了 10 条，跳过 0 条，跑的都抓到了`；core 那一趟 `✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了`。点名的都不是 layer0 二进制，没有留给 59 号的行。

## 五、受影响的层 0 流与崩溃枚举用例

没改 checker。取号那一半在第一道屏障之前多了一次读；零单元发布只多一道写之前的判定；录制流里的写、屏障与段序列在正常路径上不变（N 的第二条就钉「读不出时才多那一道屏障」；层 0 各流里没有读坏）。层 0 钉值不受影响——这是推出来的，没跑层 0。

## 六、交回前的验证（都在副本 `work/` 上跑：主工作区现状 + 本补丁 + outside-the-list 的改动 + 第三节 Q-B 的 history.rs 改法；线程上限 4，内存上限 8G）

`cargo fmt --all -- --check`：
```
fmt exit 0
```

`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 CODE_DISCIPLINE_LINTS 那 7 条 -D（照 check.sh 第 72–80 行现抄），末尾：
```
    Checking singlefs-core v0.1.0 (/tmp/claude-1000/impl-rev-a3c/work/crates/singlefs-core)
    Checking singlefs-harness v0.1.0 (/tmp/claude-1000/impl-rev-a3c/work/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.78s
```

`cargo build --offline --all-targets` 末尾：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.70s
build exit 0
```

动到的测试二进制（整个二进制跑，没有挑名字；名字带 layer0 的不跑）：
```
affected-run-2.log:     Running tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs (target/debug/deps/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics-57b7ece09e997ec9)
affected-run-2.log:test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.20s
affected-run-2.log:     Running tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs (target/debug/deps/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish-3e5da39aac590de8)
affected-run-2.log:test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.76s
affected-run-2.log:     Running tests/core_review_unit_area_start_and_publish_limits.rs (target/debug/deps/core_review_unit_area_start_and_publish_limits-0bfd52e767162075)
affected-run-2.log:test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.57s
affected-run-2.log:     Running tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs (target/debug/deps/corrupt_on_disk_content_is_refused_instead_of_panicking-d386b62b61a11d62)
affected-run-2.log:test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.19s
affected-run-2.log:     Running tests/instance_acquisition.rs (target/debug/deps/instance_acquisition-fd1c3e473c0956da)
affected-run-2.log:test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.33s
affected-run-2.log:     Running tests/second_transaction_step_three_formatted_pool.rs (target/debug/deps/second_transaction_step_three_formatted_pool-16308d5197d08beb)
affected-run-2.log:test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.11s
affected-run-2.log:     Running tests/second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs (target/debug/deps/second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version-d31a4e9f9db20114)
affected-run-2.log:test result: FAILED. 15 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 78.46s
affected-run-2.log:     Running tests/second_transaction_supplement_two_multi_record_transaction_zero_publishes.rs (target/debug/deps/second_transaction_supplement_two_multi_record_transaction_zero_publishes-eacb53dba08bf31d)
affected-run-2.log:test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.47s
affected-run-2.log:     Running tests/second_transaction_supplement_two_root_ring_turn_in_one_mount.rs (target/debug/deps/second_transaction_supplement_two_root_ring_turn_in_one_mount-3acd4819867cd47c)
affected-run-2.log:test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.92s
affected-run-2.log:     Running tests/second_transaction_supplement_two_tree_identifier_watermark_crash_orphans.rs (target/debug/deps/second_transaction_supplement_two_tree_identifier_watermark_crash_orphans-eba294e6a5ff5a7a)
affected-run-2.log:test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 585.33s
affected-run-2.log:     Running tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs (target/debug/deps/system_configuration_slot_is_overwritten_only_after_a_barrier-a7fc8ef08de08833)
affected-run-2.log:test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.42s
affected-run-2.log:harness tests exit 101
core-lib-run.log:     Running unittests src/lib.rs (target/debug/deps/singlefs_core-1133d274220e6ae8)
core-lib-run.log:test result: ok. 132 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
core-lib-run.log:core exit 0
affected-run.log:     Running unittests src/lib.rs (target/debug/deps/singlefs_harness-ed0b65334d8d5632)
affected-run.log:test result: FAILED. 94 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 269.85s
affected-run.log:harness exit 101
```

上面红的 3 条，逐条在开工底座 `orig/`（主工作区现状，不带本补丁）上重跑，红法相同，**不是这一件带来的**：
- `singlefs-harness --lib` 的 `model_comparison::tests::every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways`：两边都是「种子 0：NewFinding … Operation(15) CrashRecoveryAbandoningTheNewestRoot … 拒了：MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)」。底座一趟末尾：
```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 94 filtered out; finished in 8.81s
baseline exit 101
```
- `second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version` 里的 `blank_device_refuses_the_writable_mount_by_name_before_any_write`、`mount_writable_while_every_read_of_device_one_fails_is_refused_naming_device_one`：两边都红在第 1556 行，「该报 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 NewerStateStillUnreadableAfterOneReread(…)」。底座一趟末尾：
```
test blank_device_refuses_the_writable_mount_by_name_before_any_write ... FAILED
test mount_writable_while_every_read_of_device_one_fails_is_refused_naming_device_one ... FAILED
test result: FAILED. 15 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 80.56s
exit 101
```
看上去是 C554 乙 / 乙-配置续（已在主工作区）与模型、旧期望还没对上，属于「模型跟上乙」那一件，不是我的；推的，没深查。

登记给我的门禁阶段（在副本上跑，`nice -n 19`）：
```
53-format-const-placeholders.sh：  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1�
92-layout-checker-sync.sh：  ! /tmp/claude-1000/impl-rev-a3c/work 不是 git 仓，本阶段跳过
94-checker-implementation-disjoint.sh：    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/
93-feature-bits.sh：  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 57 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INC
89-closeout-row27-preconditions.sh：    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐
33-mutation-tables.sh：  ✗ 这些实验二进制没有同名变异表：（exit 1，只剩 research/mutations/e163_gpu_multicard_crc32c.tsv 缺表，是 E163 那一边的，不是我的；crates/mutations.tsv 锚点那一项在副本上不红）
```
- 退出码：53 号 0；92 号 77（副本不是 git 仓，本次没跑；这件没动布局与 checker）；94 号 0；93 号 0；89 号 77（收口表第 27 行清单对不上，开工前就这样，与本件无关）；33 号 1（原因同上）。
- 74 号（`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`）在副本上 exit 1，红 3 条：`crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`、`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`、`rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline`。在底座 `orig/` 上同样跑 74 号，红的是同样这 3 条，末行原样：
```
改后：test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 67.95s
底座：test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 69.77s
```
红的是 `second_transaction_supplement_three_random_history`（在别的会话单里），不是这一件带来的。

## 七、补丁与打法

`patch/` 里：`crates.patch`（清单内 4 份）、`outside-the-list.patch`（清单外 3 份测试，`apply-writer-patch.py` 不认这个文件名，要主 agent 另外 `git apply`）、`mutations-append.tsv`（7 行）、`mutations-replacements.tsv`（2 行）、`report.md`（这份的拷贝）。对主工作区现状（之后打上了 checker 那一半，`crates/mutations.tsv` 1300 行）核过：
```
git apply --check crates.patch：过
git apply --check outside-the-list.patch：过
✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1307 行
```
次序：`crates.patch`、`outside-the-list.patch` 与第三节 Q-B 的 `history.rs` 三处要一起落，否则 harness 编不过（`history.rs` 那两处用 `BlockDeviceError` 接 `publish_without_units` 的错；三份测试要读 `AcquisitionFailed.rollback`）。

## 八、没做什么

- 第 4 条没改（Q-A），`recovery.rs`、`mounted_session.rs` 一字没动。
- `history.rs` 没改（别的会话单里），原文在 Q-B。
- 层 0、崩溃注入快档、59 号整表、全量 `cargo test` 都没跑（重型）。受影响的层 0 流是推的，见第五节。
- 92 号在副本上是 77，没判（副本不是 git 仓）。
- 规格第 1 条要求「录制流不多一步」，只在读不出的情形上钉了（N 第一条）；正常路径上的录制流步数没专门写用例钉，靠既有 `instance_acquisition`、`system_configuration_slot_is_overwritten_only_after_a_barrier` 与层 0 流（层 0 我没跑）。
- 没走三方对抗、没提交。

## 九、草稿目录清理

- 删了：`/tmp/claude-1000/impl-rev-a3c/orig`（3.7G，开工底座副本，含 74 号底座那一趟的 target）、`/tmp/claude-1000/impl-rev-a3c/work`（20G，改动副本及其 target）、`/tmp/claude-1000/impl-rev-a3c/probe`（探针草稿）。
- 留着：`patch/`、`report.md`、`progress.md`、`prove-red-1.out` / `prove-red-2.out` 与 `prove-red-logs-1/`、`prove-red-logs-2/`（证红日志，主 agent 要核）、各 `*.log`（第六节贴的输出出处）、`history-change.diff`（Q-B 的原样 diff）。都是文本，没有编译目录。
