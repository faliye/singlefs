# 实现员报告：m2 增补 2 收口，代码三方第二轮判决第二节第 1–4 行（Z1-d、I-5.4、Z1-a、两处注释）

时刻：开工 2026-09-18 03:39 UTC（12:39 JST），交回约 04:25 UTC（13:25 JST）。

## 结论

四条都做了，`check.sh` 全绿；这一轮新加的 8 条测试（`allocator.rs` 单测 3 条、两个新测试文件共 4 条、坏镜像用例 1 条）各自有一处改坏会让它红（第三节），`crates/mutations.tsv` 追加 8 行，照门禁 59 号的判法在副本上逐行复跑 8/8 红在点名的测试上。
- Z1-d：攻方那条历史（挂载 → 覆盖写 2 次，× 7；第 8 次挂载 → 覆盖写 1 次）走完之后，第 9、10 次可写挂载都成功；每次挂载、每次覆盖写之后 I-5.4 都成立。把 `allocator.rs` 换回改之前那一版：I-5.4 在 txg 38 那次覆盖写之后判红；再把用例里的断言关掉，第 9 次挂载 panic 在 `allocator.rs:375`（第四节）。
- I-5.4：checker 按候选集里每条有效根的分配记录树逐盘判，`IMPLEMENTED_INVARIANTS` 从 28 条变成 29 条，加了两份只让 I-5.4 红的坏镜像。
- Z1-a：可写挂载一直挂到被拒为止。写满一片的那一次放行，下一次在取号之前返回 `InstanceTableRowsExceedOnePageSecondPageUnsupported`，盘上逐字节不变、两盘超级块的实例代号不变、录制流 0 步。改之前那一次 panic 在 `bytes.rs:18`（变异 m5/m6/m8）。
- 两处注释（`transaction.rs`、`walk.rs`）都改了。

推翻条件：门禁 59 号在 `gate-triage` 那一轮复跑，追加的 8 行有一行没红；或者门禁 54 号在新代码上全量跑出 I-5.4 违例，或 `first_transaction_step_seven_layer0` 里 I-5.4 的「评估过的状态数」不等于 4（这个数我只在 22 个状态的快用例上验过，见第八节第 2 条）。出现任一种，上面的结论不成立。

## 一、写过的文件

改的（都在主工作区）：
- `crates/singlefs-core/src/allocator.rs`：Z1-d 的改法，外加 3 条单测
- `crates/singlefs-core/src/mount.rs`：Z1-a 的改法
- `crates/singlefs-core/src/transaction.rs`：第 4 行的注释（加上两处同义的字段、方法说明）
- `crates/singlefs-checker/src/walk.rs`：I-5.4 的判定、`AllocationRecordView` 加跨度字段、第 4 行的注释、另两处 I-3.9 旧简称、文件头 28 改 29
- `crates/singlefs-checker/src/image.rs`：`IMPLEMENTED_INVARIANTS` 28 改 29
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：I-5.4 的两份坏镜像与一条用例，登记进覆盖集；文件头 28 改 29、一处旧简称
- `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`：I-5.4 并进「只在新根下面评估」那一组
- `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`：I-5.4 并进「至少在一个状态上评估过」那一组
- `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs`：同上
- `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`：「没有文件时报不适用」从 10 条改 11 条（加 I-5.4），注释 28 改 29
- `crates/mutations.tsv`：只在末尾追加 8 行（第 121–128 行），变异名如下
  - `Z1-d：复用时新记录罩住的已回收记录不删（I-5.4 要红）`
  - `Z1-d：新落点罩住没回收的记录不断言`
  - `I-5.4：checker 判不出分配记录重叠`
  - `I-5.4：只判最新根那棵账`
  - `Z1-a：取号之前不算实例表`
  - `Z1-a：实例表准入漏算链指针`
  - `Z1-a：实例表准入把正好写满一片也拒掉`
  - `Z1-a：实例表准入按每次挂载只写一行算`

新建的（未跟踪；`[[test]]` 靠 cargo 自动发现，不用改 `Cargo.toml`）：
- `crates/singlefs-harness/tests/second_transaction_supplement_two_reused_record_overlap.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_two_instance_table_page_full.rs`

开工时 `git status` 里 `crates/` 全是暂存态（`M ` / `A `），`crates/` 下 mtime 最新的是 `mutations.tsv`，2026-09-18 01:55 UTC，没有别的会话在改。下面的 diff 是相对暂存区的，只含这一轮的改动。

`git diff --stat -- crates litmus` 原样：
```
 crates/mutations.tsv                               |   8 +
 crates/singlefs-checker/src/image.rs               |   6 +-
 crates/singlefs-checker/src/walk.rs                | 136 ++++++++--
 crates/singlefs-core/src/allocator.rs              | 288 +++++++++++++++++++--
 crates/singlefs-core/src/mount.rs                  |  98 +++++--
 crates/singlefs-core/src/transaction.rs            |  10 +-
 .../tests/checker_known_bad_images.rs              | 112 +++++++-
 .../tests/first_transaction_step_seven_layer0.rs   |   5 +-
 ...second_transaction_step_three_formatted_pool.rs |  12 +-
 ...transaction_step_three_formatted_pool_layer0.rs |   2 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   5 +-
 11 files changed, 599 insertions(+), 83 deletions(-)
```
另有两个未跟踪的新文件：一个 260 行，一个 200 行（`wc -l`）。

## 二、改了什么（行号是现在文件里的，`grep -n` 现取）

**Z1-d**（`crates/singlefs-core/src/allocator.rs`）
- 第 669 行 `record`：每块盘先调一次 `make_room_for_record_on_device`（第 508 行），返回 `RecordAtThePlacementSlot`（第 491 行）。返回 `RewrittenFromReclaimed` 就不再追加，返回 `Absent` 才追加一条新记录。
- `make_room_for_record_on_device` 的做法：先找出同一块盘上与 [槽, 槽 + 跨度) 相交的所有记录，逐条处理。
  - 不是已回收的：断言（第 528 行起）。
  - 已回收、起点就是这个槽：改写成这次的分配（改写原来就有，这里只是挪了位置）。
  - 已回收、起点不是这个槽：删掉（第 546 行 `records.retain`），同时从 `reclaimed` 里去掉。
  - 位图和计数都不动：已回收的槽在 `mark_reclaimed` 里已经回到空闲。
- 这两个断言为什么走不到：
  - 相交的记录不是已回收的：调 `record` 的地方只有三处。用户数据落点走 `lowest_user_data_slot` → `is_free`；提交内生块走 bump 与回落，都绕开 `is_blocked_for_commit_generated`；mkfs 那次分配器是空的。这三处发出去的槽在位图上都是空的，而仍分配的记录、已释放但没回收的记录，它们罩住的槽在位图上一直占着（`mark_allocated` 置位，只有 `mark_reclaimed` 才清）。
  - 起点槽在 `reclaimed` 里却没有被改写：`reclaimed` 里的键全是 `reclaim_released_up_to` 从记录里取的，所以起点槽在 `reclaimed` 里就一定有一条起点在这个槽的记录，它必然与新落点相交、必然被改写。
- 文档注释第 23–26 行补了一句 I-5.4。

**I-5.4**（`crates/singlefs-checker/src/walk.rs`、`image.rs`）
- 第 728 行：`AllocationRecordView` 加 `span_slots`（跨度段去掉已释放标志）。
- 第 1165 行 `judge_allocation_records_disjoint`：对候选集里每条根，读树表 → 种类 3 那一条 → 分配记录树根节点。按第一条位置条目的 (设备, 槽, 校验和) 去重，几条根共用同一个节点时只判一次。
- 第 1221 行 `judge_allocation_record_ranges_of_one_node`：同一块盘上的记录按 (槽, 跨度) 排序，相邻两条比较；每个 (节点, 盘) 判一格。
- 以下几种根不判：树表或节点读不出、节点有内部层、条目比 20 字节窄。一个节点都没判到时整条报「不适用」并带理由。
- 第 1410 行：I-3.9/I-9.14 那一遍原来在函数里自建 `IndexNodeCache`，现在改成由 `check_pool_image` 建一份，两遍共用（层 0 每个状态都跑这两遍）。第二个事务的层 0 快用例 debug 耗时：改前 8.72 s，改后 8.43 s。
- `image.rs` 第 36 行：29 条，I-5.4 排在 I-5.2 之后。

**Z1-a**（`crates/singlefs-core/src/mount.rs`）
- 第 109 行：新成员 `InstanceTableRowsExceedOnePageSecondPageUnsupported { instance_to_acquire, rows_in_version, rows_to_write, records_per_page }`。
- 第 938 行：新函数 `instance_rows_to_write`，把原来取号之后才列行的那段循环原样挪了出来，按要取的号先列。
- 第 983 行：`establish_instance` 在取号之前就列出这次要写的行，准入与写行用的是同一份。
- 第 857 行 `refuse_publishes_before_acquisition_that_do_not_pass_admission` 多一个参数 `rows_to_write`。在 `WithFile` 那一格、原有两条准入之后，第 888 行判 `这一版的行数 + 要写的行数 + 1 > INSTANCE_TABLE_PAGE_RECORDS`，成立就返回新成员。
  - 这个数是精确的，不是上界：`publish_rows_on_file_version` 就是在那一版表后面 `extend` 这几行。
  - 要取的号在写之前会重算，对不上就不写（`acquire_expected_instance`）。第 1008 行加了 `assert_eq!(instance, instance_to_acquire)`，把这一点钉住。
  - 回退走同一段代码，行数按 R_old 那一版表和 [max(r_old, 1), 新实例) 自己算。
- `mount_writable` 与 `mount_rollback` 的 `# Errors` 各补了一句。

**第 4 行**
- `transaction.rs` 第 230–233 行（`count_failed_publish` 在第 234 行）：说明改成只数落盘阶段失败的发布；落盘之前的准入、释放判定、分配、装单元失败不记。第 99 行与第 242 行两处同义的说明跟着加了「落盘阶段」。
- `walk.rs` 第 966–975 行：`judge_release_generations` 的说明改成与 `.claude/kb/invariants.md` I-3.9 行一致（区间 (L, T]、全部候选根都还引用时不许带已释放标志、已回收的跳过、全是这一类时报不适用）；去掉了「与 I-3.9 的字面不同，等用户定案」与旧简称。同文件另两处旧简称（「一条有效根自己引用了哪些落点」那段、「这一遍要走多深」那段），以及 `checker_known_bad_images.rs` 里的一处，都换成了新简称。

## 三、新测试的「改坏哪一行 → 哪条断言红」

做法：每份变异一个仓副本（`rsync -a --exclude target --exclude .git`，放在 `/tmp/claude-1000/impl-m2-wave2-fixes/copies/<名>`），每份用自己的 target（`targets/<名>`），跑 `cargo test --offline --all --no-fail-fast`。先跑了两份基线：
- 改前的仓副本 `baseline/`：退出码 0，没有红的测试。
- 改后不做变异的副本 `fixed-baseline`：`{"exit": 0, "failed": []}`。

所以基线红集是空的。被测代码里没有 `debug_assert`（`grep -rn debug_assert crates` 零命中），不用再跑一遍 release。下表是最后一轮的结果（改名之后），日志在 `/tmp/claude-1000/impl-m2-wave2-fixes/logs/<名>.log`，每份另有 `.summary.json`。

| 变异 | 改坏哪一行 | 红的测试（整套里全部红的） | 红在哪条断言 |
|---|---|---|---|
| m1 | `allocator.rs:546`：`records.retain(...)` 换成注释（被罩住的已回收记录不删） | `allocator::tests::reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_rewrites_the_first_and_deletes_the_second`；`allocator::tests::one_slot_placement_inside_reclaimed_two_slot_record_deletes_that_record`；`reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds` | 前两条红在记录清单的 `assert_eq`（清单里多出 `(50179, 1, 4, true)` 或 `(50180, 2, 3, true)`）；第三条红在 `reused_record_overlap.rs:67`，I-5.4 报 `Violated("txg 38 的根那棵分配记录树、盘 0：槽 50320 跨 2 的记录与槽 50321 跨 1 的记录罩住同一个槽")` |
| m2 | `allocator.rs:530`：`was_reclaimed,` 换成 `was_reclaimed \|\| true,` | `allocator::tests::placement_over_record_that_was_not_reclaimed_is_asserted_on_the_records` | 先在位图那一条断言（`allocator.rs:376`「跨度里有已分配的槽」）上 panic，被用例接住；消息里没有「不是已回收的」，于是用例自己的断言（`allocator.rs:1354`）红 |
| m3 | `walk.rs:1238`：`.find(... > pair[1].0)` 换成 `... && false`（判不出重叠） | `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant` | `checker_known_bad_images.rs:783`：`left: []  right: ["I-5.4"]` |
| m4 | `walk.rs:1419` 那次调用：`&candidate_indexes` 换成 `&[newest_index]`（只判最新根那棵账） | 同上一条 | 同上，红在第二份坏镜像（改在 A 那棵账里，只有更早的候选根指着它） |
| m5 | `mount.rs:888`：条件前加 `false &&`（取号之前不算实例表） | `writable_mounts_fill_the_instance_table_page_and_the_next_one_is_refused_before_acquisition`；`mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it`；`rollback_counts_the_rows_of_the_table_it_rolls_back_to` | 三条都 panic 在 `crates/singlefs-core/src/bytes.rs:18:19`：取号之后装实例表单元越界，这就是判决里「今天那一次」的样子 |
| m6 | `mount.rs:888`：去掉 `+ 1`（漏算链指针） | 同 m5 的三条 | 同 m5，`bytes.rs:18:19` |
| m7 | `mount.rs:888`：`>` 换成 `>=`（正好写满一片也拒） | 同 m5 的三条 | 放行的那一格被误拒，例如 `page_full.rs:196`「369 行正好写满一片，要放行：InstanceTableRowsExceedOnePageSecondPageUnsupported { instance_to_acquire: InstanceGeneration(370), rows_in_version: 0, rows_to_write: 369, records_per_page: 370 }」；另两条红在 `:156`、`:235` |
| m8 | `mount.rs:990`：传给准入的 `rows_written.len()` 换成 `1`（按每次挂载只写一行算） | `mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it`；`rollback_counts_the_rows_of_the_table_it_rolls_back_to` | 两条都 panic 在 `bytes.rs:18:19`。一路挂载那条不红，因为它每次挂载本来就只写一行，所以另写了「取号之后崩溃 369 次」那一格 |

`crates/mutations.tsv` 第 121–128 行就是这 8 条（原文在文件里各命中 1 次，脚本数过）。我照 `59-crates-mutation-replay.sh` 的判法写了一个只跑这 8 行的脚本（`/tmp/claude-1000/impl-m2-wave2-fixes/gate59_my_rows.py`），在副本上跑，末行原样是：
```
rows=8 failures=0
exit=0
```
（逐行 8 个 ✓，全文在 `gate59-my-rows.log`。）整道门禁 59 号（120 + 8 条）没跑，归 `gate-triage`。

改名与 `should_panic`：
- 命名 lint 把我新起的四个 `a_` 开头的测试名报成单字母，都去掉了前缀。改完之后全仓 lint 的 9 处与改前那份副本的 9 处相同。
- m2 那条原来写成 `#[should_panic(expected = ...)]`，但门禁 59 号的正则认的是「test 名 ... FAILED」，`should_panic` 的输出行多一段「- should panic」，匹配不上。所以改成用例自己 `catch_unwind` 再核 panic 消息。

## 四、I-5.4 先证明会红：改之前那一版上跑第 1 条的历史

副本 `copies/prefix-allocator`：只把 `allocator.rs` 换回改之前那份（从开工时拷的 `baseline/` 取），checker 与用例都是新的。
- 原样跑 `second_transaction_supplement_two_reused_record_overlap`（日志 `prefix-allocator-z1d.log`）：
  ```
  thread 'reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds' (1156754) panicked at crates/singlefs-harness/tests/second_transaction_supplement_two_reused_record_overlap.rs:67:5:
    left: Violated("txg 38 的根那棵分配记录树、盘 0：槽 50320 跨 2 的记录与槽 50321 跨 1 的记录罩住同一个槽")
   right: Holds
  ```
  txg 38 之前每次都判成立，txg 38 那次覆盖写之后第一次判红，与攻方 `pristine-run.log` 里第一次出现重叠的那一次相同。
- 把用例里的 I-5.4 断言和「50321 那条删掉」的断言都关掉再跑（日志 `prefix-allocator-z1d-mount-panic.log`）：
  ```
  thread 'reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds' (1171608) panicked at crates/singlefs-core/src/allocator.rs:375:9:
  跨度里有已分配的槽
  ```
  第 9 次挂载 panic 在 `allocator.rs:375`，与判决里说的一致。
- 改后的代码上：每次挂载、每次覆盖写之后 I-5.4 都判成立，第 9、10 次挂载都成功。

另外，I-3.1 从第 6 次挂载起一直红，改前改后都一样（用例把它打成 `OTHER_RED …` 行，不断言）。这与攻方标的「疑似第 ② 行那一族」是同一个现象，我没核。

## 五、攻方的 overlap 扫描在改后副本上的结果

副本 `copies/probe-fixed`。攻方的 `opus_probe_overlap.rs` 用的是插桩版共用模块，我拷过来之后只改了三处（diff 在 `probe-overlap-adaptation.diff`）：
- 共用模块换成不插桩的 `opus_probe_common_pristine.rs`；
- 删掉 5 处 `take_probe_log()`；
- k 取 {2, 3, 5, 6, 10, 12, 16, 20}。

用 release、每个 k 至多 60 次挂载（探针默认值），日志 `round1/logs/probe-overlap-on-fixed.log`（这两份探针日志是第一轮跑出来的，归档时跟着第一轮的 logs 目录挪进了 round1；探针只依赖 Z1-d / Z1-a 的改法，后来的改名与坏镜像改动碰不到它）。改前那一栏抄自攻方 `overlap.log`。

| k（每次挂载后覆盖写几次） | 改前 | 改后 |
|---|---|---|
| 2 | 第 9 次挂载 panic | 60 次挂载都成功，没出现重叠 |
| 3 | 第 8 次挂载 panic | 60 次都成功，没出现重叠 |
| 5 | 第 7 次挂载 panic | 60 次都成功，没出现重叠 |
| 6 | 第 7 次挂载 panic | 60 次都成功，没出现重叠 |
| 10 | 第 5 次挂载 panic | 60 次都成功，没出现重叠 |
| 12 | 第 4 次挂载 panic | 第 8 次挂载在取号之前被拒：`WarmUpAdmissionRefusedBeforeAcquisition { instance_to_acquire: InstanceGeneration(9), warm_up_publish_index: 2, warm_up_publishes_planned: 2, cause: AllocationRecordsExceedOneNode { records: 818, capacity: 812 } }` |
| 16 | 第 4 次挂载 panic | 第 4 次挂载在取号之前被拒：`RowPublishAdmissionRefusedBeforeAcquisition { instance_to_acquire: InstanceGeneration(5), cause: AllocationRecordsExceedOneNode { records: 818, capacity: 812 } }` |
| 20 | 第 4 次挂载 panic（第 3 次挂载的写行那次已经出现重叠） | 第 3 次挂载后的覆盖写被拒（`AllocationRecordsExceedOneNode { records: 824, capacity: 812 }`），第 4 次挂载在取号之前被拒（`RowPublish…`，818） |

k = 12、16、20 停下来是分配记录树一个节点 812 条的容量墙（收口表第 28 行那一族，判决里 Z1-b 按上界误拒的那一格），不是 panic。同一文件里的定点用例 `overlap_pinpoint_per_session_two` 改后打出「mount=9 mounted (no panic)」「mount=10 did not panic」。

攻方的 `opus_probe_pristine.rs` 在改后副本上也跑了（`round1/logs/probe-pristine-on-fixed.log`）：
- Z1-a 那条：「mount=370 refused InstanceTableRowsExceedOnePageSecondPageUnsupported { instance_to_acquire: InstanceGeneration(371), rows_in_version: 369, rows_to_write: 1, records_per_page: 370 }」。探针把「被拒」当 panic 报，所以测试显示 FAILED。
- Z1-d 那条：「20 次挂载都没 panic」，同样是探针自己的 panic 文案。

## 六、Z1-a 的用例怎么搭的

`second_transaction_supplement_two_instance_table_page_full.rs`，三条用例，都在两块 4 GiB 内存盘上从第一个事务起步：
- `writable_mounts_fill_the_instance_table_page_and_the_next_one_is_refused_before_acquisition`：先连着 360 次「取号之后崩溃」（直接调 `acquire_instance`：取号写完、写行那次发布没发出去就掉电，今天就走得到）。之后连挂 9 次，第 k 次取号 361 + k，表里 360 + k 行；第 9 次正好写满一片（369 行 + 链指针 = 370 条）。第 10 次在取号之前拒绝：(371, 369, 1)，`DiskSnapshot` 逐项相等，四个超级块槽都还是 370。
  - 最早写的版本是攻方原样的「从第一个事务起连挂 369 次、第 370 次被拒」。它在 debug 下跑过一次，通过了，但要 297 s（日志 `z1a-natural-369-mounts-debug.log`，里面「3 passed … finished in 297.01s」），平时的 `cargo test` 背不起，才换成先崩 360 次再挂的做法。两种做法最后那一次被拒时的状态相同（这一版 369 行、要写 1 行、要取 371）。
- `mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it`：崩 368 次后挂一次，一次写 369 行，放行；崩 369 次后挂一次，要写 370 行，取号之前拒绝 (371, 0, 370)。
- `rollback_counts_the_rows_of_the_table_it_rolls_back_to`：
  - 崩 367 次、挂一次（表 368 行），再回退到 A（txg 3，它那一版表 0 行），要写 369 行，放行。若拿最新那张表算，就会误拒。
  - 崩 368 次、挂一次（表 369 行），再回退到 A，要写 370 行，取号之前拒绝。

三条都用 `SharedStream::new()`（只记步数、不留内容），录制流步数是 `DiskSnapshot` 里的一项。最后一次 `check.sh`（debug）里各测试二进制的耗时：这个文件 `3 passed … finished in 5.79s`，`second_transaction_supplement_two_reused_record_overlap` `1 passed … finished in 13.89s`，`checker_known_bad_images` `4 passed … finished in 0.97s`。

## 七、门禁

`nice -n 19 bash .claude/scripts/check.sh`，`CARGO_TARGET_DIR` 指到我自己的 `main-target`（没碰默认 target，门禁 54 号在用它），最后一次在 04:11:33–04:12:28 UTC。四段结果行与末尾原样：
```
══ cargo fmt --check ══
  ✓ 格式通过
══ cargo clippy ══
  ✓ clippy 通过
══ cargo build ══
  ✓ 构建通过
══ cargo test ══
  ✓ 单测通过
```
```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit=0
```
（全文 `check-4.log`。第一次 clippy 报过 `shadow_unrelated`，在新测试里，已改。）

登记给 implementation-writer 的门禁阶段只有一个：`53-format-const-placeholders.sh`，退出码 0，末行原样：
```
  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））
```

负载：开工时 `ps` 看到两样在跑。一是门禁 54 号：pid 1213561（bash）、1215014（`cargo test --release … second_transaction_step_zero_layer0`），跑的是开工前编好的旧代码，我没停它。04:14 UTC 再看还在跑，已经 2 小时 11 分。二是另一个会话的 `cargo test --release --bin e142-first-txn-dry-run`（pid 3966781）。没有 qemu、vm-bench、e152、fio。我全程用自己的 target 目录，没等过锁。

## 八、停下交主 agent 的事（没有自己定）

1. **kb 要跟着改，不归我写**：
   - `.claude/kb/invariants.md` I-5.4 那一行的状态列还写着「未实现」，要改成已实现，并写上 `walk::judge_allocation_records_disjoint`、坏镜像用例 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；另注明层 0 全量还没在新代码上跑过（第 2 条）。
   - 同一文件第 12 行「判 28 条」要改成 29。
   - 里程碑「第二个事务」增补 2 收口表里 Z1-d、Z1-a、Z2-d、Z6 这几格的现状要登记。
2. **门禁 54 号要在新代码上重跑**（归 crash-verifier）。此刻在跑的那一趟是改之前编的二进制。checker 多了一条 I-5.4，两条层 0 流的全量用例都受影响：
   - `first_transaction_step_seven_layer0` 的全量用例要求 I-5.4 恰好在 4 个状态上评估过（我把它放进了「只在新根下面评估」那一组）。这个数我只在 22 个状态的快用例上验过，`assert_checker_counts(&tally, 22, 4)` 通过了。
   - `second_transaction_step_zero_layer0` 的全量用例要求 I-5.4 至少在一个状态上评估过、而且一个违例都没有。快用例（`every_crash_state_outside_the_two_unit_segments…`）和残留记录、旧尾两条流都绿了。
   - 这一轮没新开层 0 流，也没加崩溃点重放用例。
3. **D3 已定项 7 与这次删记录的读法**：已定项 7 写「条目留到该落点被重新分配时覆盖」，I-5.4 那一行写「新记录罩住的槽上别的已回收记录要随这次分配一起删掉」。我按后者实现：只要罩住的槽有一个被这次分配占用，整条已回收记录就删掉。起点不在新落点里的那一截槽之后没有记录，与从没分配过的槽一样（单测 `one_slot_placement_inside_reclaimed_two_slot_record_deletes_that_record` 钉了这一格：已回收的 50180 跨 2，单槽落在 50181，50180 那条删掉，50180 这个槽空着、没有记录）。这一格今天的固定脚本走不走得到我没查。如果已定项 7 的意思是「一条记录只能被整段覆盖」，这里就要改。
4. **I-3.1 在第 1 条的历史上从第 6 次挂载起就红**（改前改后一样），用例只打印、不断言。攻方标的是疑似收口表第 ② 行那一族，主 agent 也还没核。
5. **checker 的 I-5.4 跳过了三类根**：分配记录树有内部节点的（内部条目格式没有条款）、读不出的、条目比 20 字节窄的。一个节点都判不到时整条报不适用。今天第一版的分配记录树只有一层，这三类根一条都不跳；等分配记录树长出第二层，这条不变量要跟着扩。
6. **Z1-a 那条用例用「取号之后崩溃 360 次」代替攻方从头连挂 369 次的原样历史**（理由见第六节）。原样那一版要不要留成 `#[ignore]`、交给 release 门禁跑，由主 agent 定；它的代码就是 `page_full.rs` 里那条用例把前 360 次崩溃删掉、挂载次数改成 1..=369，下一次期望 (371, 369, 1)。
7. **没定的欠账**：实例表第二片、删行、超过 370 次可写挂载的池怎么办，照判决不在这一轮。现在那样的池一试可写挂载就返回新错误成员，只能只读用。

## 九、草稿产物（只在 /tmp，没入库）

我的写范围只有 `crates/`、`litmus/` 与 `/tmp/claude-1000/` 下的报告和草稿目录，写不了 `research/`。下面这些要不要转存进 `research/results/` 或 `research/prompts/`，由主 agent 定。它们都在 `/tmp/claude-1000/impl-m2-wave2-fixes/` 下，会话一重启就没了：
- `logs/<名>.log` 与 `.summary.json`：最后一轮 8 份变异加 `fixed-baseline` 的整套 `cargo test` 输出，第三节的表出自这里。`round1/`、`round2/` 是改名与补坏镜像之前的两轮，结论相同，只是测试名是旧的。
- `gate59-my-rows.log`（照门禁 59 号判法只跑追加的 8 行），脚本 `gate59_my_rows.py`；变异驱动脚本 `mutations/run_mutation.py` 与 `mutations/*.json`。
- `prefix-allocator-z1d.log`、`prefix-allocator-z1d-mount-panic.log`（第四节）。
- `round1/logs/probe-overlap-on-fixed.log`、`round1/logs/probe-pristine-on-fixed.log`、`probe-overlap-adaptation.diff`（第五节，攻方探针在改后副本上的复跑）。
- `z1a-natural-369-mounts-debug.log`（Z1-a 原样历史在 debug 下跑通的那一次）。
- `check-1.log` 到 `check-4.log`、`gate-53.log`、`baseline-test.log`（改前副本的整套测试，退出码 0）。
- 副本 `copies/`、`baseline/` 与各自的 target（几个 GB），用完可以删。

## 十、没做什么

- 没走三方对抗：门禁 56 号要的判决文件由主 agent 那一轮出。这一轮改的 `allocator.rs`、`mount.rs`、`transaction.rs`、`walk.rs`、`image.rs` 都要在那份判决里按路径点名。这一轮的改法被攻过零轮。
- 没跑层 0 全量（门禁 54 号）、QEMU、herd7，也没跑整道 crates 变异表（门禁 59 号，120 + 8 条），这些归 crash-verifier 与 gate-triage。我只在副本上照 59 号的判法跑了追加的 8 行。
- 没写 kb（第八节第 1 条），没写 `research/`，没提交，没做任何 git 写操作。
- 没做实例表第二片、删行（判决写明不在这一轮）。
- 没有停下来的分支：这一轮只加了一个新的错误成员，就是条款点名的「取号之前拒绝」。新加的三个断言（`allocator.rs` 两处、`mount.rs` 第 1008 行）为什么走不到，写在第二节。
- 没核 I-3.1 在第 1 条历史上判红的原因（第八节第 4 条）。
