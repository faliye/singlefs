# 实二（回退改形态：SysPre 与 B1）实现员报告

交回时刻：2026-09-26 JST（本机 UTC + 9）。在主工作区改；编译一律用自己的 `CARGO_TARGET_DIR`，没碰项目 `target/`。

## 结论

- 生效值：`recovery::effective_rollback_floor` 改成 max(环里读得出的根上的 F, 池里每块盘两槽自证过的系统配置 F)（`recovery.rs:860`）。
- SysPre：抬 F 那一串（准入与卸载共用 `mount::raise_the_floor_through`，`mount.rs:1251`）预演过了、第一次发布之前逐盘写一次系统配置带新 F、过一道屏障（`transaction::write_the_raised_floor_into_every_system_configuration`，`transaction.rs:546`；调用在 `mount.rs:1401`）。先写那一步任一块盘失败：一条根不发，报 `MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`（`mount.rs:97`），已写进的留着，分配器换回抬 F 之前那一份。
- Q1：取号、取号失败的回卷、发布末尾的轮换，这三种系统配置写都带整池 F 生效值。每次写之前现算：`transaction.rs:385` 起，枚举 `RollbackFloorOfASystemConfigurationWrite` 在 `transaction.rs:518`。
- 正常卸载：新增 `mount::unmount`（`mount.rs:1209`）。F 抬到现行那一版的 txg，不判上限。空发布推到每块盘上都有一条带新 F 的根，第一版几何上 2 或 3 次，用例两种余数都钉了。这一串的根都带卸载记号，系统配置照 SysPre 先写。
- Q2：卸载记号进 `RootRecord`，字段是 `unmount_marker: UnmountMarker`（`root_record.rs:23`、`:55`），两个成员 `WrittenByTheUnmountSequence` 与 `NotWrittenByTheUnmountSequence`。读者照实读出，写者照字段写（`:84`、`:128`）。只有卸载那一串经 `transaction::publish_version_with_unmount_marker`（`transaction.rs:4588`）写 1。checker 读成 `UnmountMarkerView`（`checker/src/lib.rs:213`），层 0 读的是 `crash::unmount_marker_written_by`（`crash.rs:1171`）。
- `ROLLBACK_FLOOR_OFFSET` 改成 `SYSTEM_CONFIGURATION_BYTES - ROLLBACK_FLOOR_FIELD_BYTES`（`system_configuration.rs:52`、`:55`）。checker 那边另从自己的 tail 偏移推：`lib.rs:231` = tail + 8 + 4。
- checker：
  - C556：候选集下界取 max(最新根 F, 各盘自证过的系统配置 F)（`walk.rs:4747`）。
  - I-7.12 立成判定（`walk.rs:3157`，调用在 `:4634`），`IMPLEMENTED_INVARIANTS` 46 → 47（`image.rs:37`）。
  - I-7.9 按记号分两支（`walk.rs:3093`）。
  - 新增一处 I-3.1「不适用」（`walk.rs:5009`）：最新根带的 F 低于生效值时报不适用，见设计问题 D5。
- 层 0 那一侧：
  - C557：录制流记入口段，`SharedStream::record_entry` / `entry_spans`（`harness/src/lib.rs:117`、`:160`、`:177`）。判定是 `crash::unmount_markers_outside_the_unmount_entry`（`crash.rs:1198`），接在层 0 切写表的入口 `writes_and_segments_with_stream_indexes[_and_entries]`（`crash.rs:459`）：没记入口的流里只要有一条带记号的根槽写就断言停。
  - C556 的层 0 那一半：记录核对器回收谓词的 F 界，把系统配置槽写带的 F 也算进去（`crash.rs:807`）。
- 新测试 23 条：新文件 `rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs` 11 条、`checker_known_bad_images.rs` 3 条、`root_record.rs` 单测 1 条，另有既有用例里改写的几条（见证红一节）。
- 变异：追加 25 行（745–769），改锚点或点名的旧行 10 行（39、42、51、485、616、737、738、740、742、743）。最终代码上逐行证红，34 行抓到；第 616 行点名的测试在基线就红，证不了（见下）。

推翻条件：
- 主工作区现状下，这一批动到的测试二进制里有一条红，而它不在第 616 行那份基线红集里。
- 59 号复跑时，745–769 与改过的 10 行里有一行（616 除外）没红。
- 层 0 全量跑出 I-7.12 判违例，或 `first_transaction_step_seven_layer0.rs` 的「每状态都评估」对 I-7.12 不成立。

## 这一轮写过的文件

- `crates/singlefs-core/src/`：`root_record.rs`、`recovery.rs`、`transaction.rs`、`mount.rs`、`make_filesystem.rs`、`system_configuration.rs`
- `crates/singlefs-checker/src/`：`lib.rs`、`image.rs`、`walk.rs`
- `crates/singlefs-harness/src/`：`lib.rs`、`crash.rs`、`history.rs`、`model_comparison.rs`、`bin/first_transaction_on_device.rs`
- `crates/singlefs-harness/tests/`：新建 `rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`；改了 `checker_known_bad_images.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`、`second_transaction_supplement_two_instance_table_chain.rs`、`second_transaction_parallel_line_two_mounted_read.rs`
- `crates/mutations.tsv`：追加第 745–769 行；改了第 39、42、51、485、616、737、738、740、742、743 行（下面「变异行」一节有变异名）
- 中途改过又改回、净改动为 0 的两份：`crates/singlefs-harness/src/on_device_modes.rs`（加过一个 `#[allow]`，改成装箱之后删了）；`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`（改过钉死的段序列，发现基线就不对之后逐字改回，见「交主 agent」第 4 条）。

我这一批的补丁只含上面 21 份文件：`/tmp/claude-1000/impl-rbf-2/impl-rbf-2.patch`，sha256 `628f61de752fa204a457c23eba3c45e69cb3dc64951507d79de826e58d407d47`。两道核对都过了：对开工时拍下的 crates 快照正向 `git apply --check`，对主工作区现状反向 `git apply --check -R`。

`git diff --stat -- crates litmus` 原样（比 HEAD，含实一的改动与三份 rustfmt 排版过的 bin；新文件未跟踪，不在里面）：

```
 crates/mutations.tsv                               |  54 +++-
 crates/singlefs-checker/src/image.rs               |   6 +-
 crates/singlefs-checker/src/lib.rs                 |  43 +++-
 crates/singlefs-checker/src/walk.rs                | 148 +++++++++--
 crates/singlefs-core/src/make_filesystem.rs        |  11 +-
 crates/singlefs-core/src/mount.rs                  | 279 ++++++++++++++++++---
 crates/singlefs-core/src/recovery.rs               |  55 ++--
 crates/singlefs-core/src/rollback_witness.rs       |   2 +-
 crates/singlefs-core/src/root_record.rs            | 124 ++++++++-
 crates/singlefs-core/src/system_configuration.rs   | 184 +++++++++++---
 crates/singlefs-core/src/transaction.rs            | 169 ++++++++++++-
 crates/singlefs-format/src/lib.rs                  |  26 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |  73 ++++--
 ...e142_first_transaction_write_dump_one_device.rs | 121 ++++++---
 .../src/bin/e156_allocation_basis_counts.rs        |  55 ++--
 .../src/bin/first_transaction_on_device.rs         | 129 ++++++++--
 crates/singlefs-harness/src/crash.rs               | 111 +++++++-
 crates/singlefs-harness/src/history.rs             |  13 +
 crates/singlefs-harness/src/lib.rs                 |  41 +++
 crates/singlefs-harness/src/model_comparison.rs    |  24 +-
 .../tests/checker_known_bad_images.rs              | 246 +++++++++++++++++-
 ...d_transaction_parallel_line_two_mounted_read.rs |   3 +-
 .../tests/second_transaction_step_five_reuse.rs    |  49 ++--
 ...ion_supplement_two_commit_generated_fallback.rs |  26 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   1 +
 ..._transaction_supplement_two_rollback_witness.rs |   6 +-
 ...ction_supplement_two_rollback_witness_layer0.rs |   8 +-
 .../system_configuration_mutability_classes.rs     |  34 +--
 28 files changed, 1732 insertions(+), 309 deletions(-)
```

新文件：`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`，未跟踪、1024 行。

## diff 摘要（行号是改后文件的现行行号）

- `singlefs-core/src/root_record.rs`：
  - `:23` 枚举 `UnmountMarker`，含 `flags_bits` 与只读认记号那一位的解析。
  - `:55` 新字段 `unmount_marker`；`:84` 写者照字段写 flags；`:125`–`:128` 读者先拒记号以外的位，再读出记号。
  - 单测：改 `root_carrying_the_unmount_marker_…` 的期望；新 `:242` `the_writer_puts_the_unmount_marker_into_flag_bit_zero_…`。
- `singlefs-core/src/recovery.rs`：
  - `:860` `effective_rollback_floor` 取两处最大值。
  - `:2013` 由记录施加出来的那一版一律写 `NotWrittenByTheUnmountSequence`：新根段里没有 flags。
- `singlefs-core/src/transaction.rs`：
  - `:385` 系统配置写先现算整池生效值。
  - `:518` 枚举 `RollbackFloorOfASystemConfigurationWrite`，两个成员 `PoolEffectiveFloor` 与 `RaisedFloor(新 F)`；后者写 max(新 F, 现算生效值)。
  - `:529` `RaisedFloorSystemConfigurationWriteFailed`；`:546` SysPre 那一步本身：逐盘写、再一道屏障、交回写账。
  - `:1091` 零单元发布、`:1607` 写行那次发布：都写不带记号。
  - `:4588` `publish_version_with_unmount_marker`；记号一路穿到 `prepare_the_version_publish` 与 `publish_admitted`（`:4618`、`:4681`、`:5605`）。
- `singlefs-core/src/mount.rs`：
  - `:73`、`:82`：`Publish` 与 `RaiseFloorSequencePublishFailed` 改成装 `Box<PublishSequenceFailed>`。`PublishSequenceFailed` 多了 `writes_before_the_first_publish`（`:268`）之后，`MountError` 过了 clippy `result_large_err` 的 128 字节。
  - 新成员：`:97` `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(Box<…>)`、`:104` `RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided`、`:112` `UnmountOfAVersionWithoutFileWhoseEmptyPublishesAreUndecided`。
  - `RaisedFloor` 多一项 `system_configuration_writes_before_the_first_publish`；`:1116` `Unmounted`；`:1131` `RaiseFloorEntry`，两个成员是准入与卸载。
  - `:1172` `raise_rollback_floor` 与 `:1209` `unmount` 都走 `:1251` `raise_the_floor_through`。
  - 流程：`:1293` 往下抬在任何写之前拒；上限只在准入入口判；`:1401` SysPre；`:1420` 发布带记号。预演也带同一个记号。
  - dry-run 两处给 `prepare_the_version_publish` 传不带记号。
- `singlefs-core/src/system_configuration.rs:52`、`:55`：F 的偏移从格式常量推。
- `singlefs-core/src/make_filesystem.rs`：第 0 代根不带记号。
- `singlefs-checker/src/lib.rs`：
  - `:207` `SystemConfigurationView::rollback_floor`；`:213` `UnmountMarkerView`；`:231` F 偏移。
  - `:316`–`:317` `RootView` 多 `unmount_marker`；`:344` 起读记号。
- `singlefs-checker/src/image.rs:37`：`IMPLEMENTED_INVARIANTS` 加 `"I-7.12"`，46 → 47。
- `singlefs-checker/src/walk.rs`：
  - `:3093` I-7.9 带记号那一支：F ≤ 根环里 txg 比它小的最新那条根的 txg。
  - `:3157` I-7.12：每块有自证过的系统配置槽的盘判一格；两槽都自证不过的盘不判；一块都判不了整条报不适用。
  - `:4618` 起：读各盘自证过、fsid 相同的槽，调 I-7.12（`:4634`，放在 `roots.is_empty()` 早退之前）。
  - `:4747` 生效 F 取代 `newest_rollback_floor`；它同样用在候选集、由记录施加的版本和机理串里。
  - `:5009` 与 `:5066`：I-3.1 的不适用分支。
- `singlefs-harness/src/lib.rs:117`、`:124`、`:160`、`:177`：`RecordedPublishEntry`（只有 `Unmount` 一个成员）、`RecordedEntrySpan`、`record_entry`、`entry_spans`。
- `singlefs-harness/src/crash.rs`：
  - `:459` `writes_and_segments_with_stream_indexes_and_entries`；原来的 `writes_and_segments_with_stream_indexes` 改成给它传空入口段。
  - `:807` 记录核对器的 F 界把系统配置槽写带的 F 算进去。
  - `:1171` `unmount_marker_written_by`；`:1182` `UnmountMarkerOutsideTheUnmountEntry`；`:1198` 判定本身。
- `singlefs-harness/src/{history.rs,model_comparison.rs}`：三个新错误成员进了穷举 `match`。`model_comparison` 里 `Box` 化的两个成员合成一臂。
- `singlefs-harness/src/bin/first_transaction_on_device.rs`：
  - 新增 `publish_writes_and_writes_outside_any_publish_against_device` 与 `describe_failed_window_after_writes_outside_any_publish`。
  - 抬 F 窗口多打一行 `name=writes_outside_any_publish`；比对那一行在 `publishes=` 之后多一段 `writes_outside_any_publish_write_calls=`，先写系统配置的写并进合计。
  - 失败窗口同样处理；新错误成员进了两处穷举 `match`。

## 每条测试的证红记录

做法：每轮用一份仓副本（`rsync -a --exclude target --exclude .git` 拷 `crates`、`litmus`、`Cargo.toml`、`Cargo.lock`），各带各的 target。先跑不改动的基线，再逐行套 `crates/mutations.tsv` 那一行的替换，跑点名测试所在的整个二进制（不按名字挑），记下红了哪些。每行跑完从原件拷回并 `touch`。脚本 `/tmp/claude-1000/impl-rbf-2/mut/prove2.sh` 与 `prove4.sh`，判定在 `mut/rows-final.txt` 与 `mut/rows-renamed-fresh.txt`，每行的完整日志在 `mut/logs-final/row-<行>.log` 与 `mut/logs-renamed-fresh/`。「抓到」按 59 号的认法：`^test (\S+::)?<名字> ... FAILED$`。debug 构建；被测代码里是 `assert!` / `assert_eq!`，没有 `debug_assert`。

基线（不改的副本，整个二进制，最终代码）原样：
```
baseline row=745 target=singlefs-core/lib/- exit=0 red=[] test result: ok. 123 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
baseline row=747 target=singlefs-harness/test/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount exit=0 red=[] test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 86.
baseline row=39 target=singlefs-harness/test/second_transaction_step_five_reuse exit=0 red=[] test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
baseline row=740 target=singlefs-harness/test/system_configuration_rollback_floor_and_layout_identity exit=0 red=[] test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
baseline row=759 target=singlefs-harness/test/checker_known_bad_images exit=0 red=[] test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
baseline row=758 target=singlefs-harness/test/second_transaction_supplement_two_commit_generated_fallback exit=0 red=[] test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
baseline row=485 target=singlefs-harness/test/second_transaction_supplement_three_random_history exit=0 red=[] test result: ok. 22 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; 
baseline row=616 target=singlefs-harness/bin/first_transaction_on_device exit=101 red=[failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count failed_writable_mount_reports_every_pub
baseline row=39 target=singlefs-harness/test/second_transaction_step_five_reuse exit=0 red=[] test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
baseline row=748 target=singlefs-harness/test/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount exit=0 red=[] test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.
baseline row=759 target=singlefs-harness/test/checker_known_bad_images exit=0 red=[] test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
```
基线红集只有 `first_transaction_on_device` 这个 bin 的 4 条：
- `second_instance_mode_…`、`raise_rollback_floor_mode_…` 两条断言 `"16+2+1+2"`，实际是 `"28+2+1+2"`。
- `failed_raise_of_the_rollback_floor_…`、`failed_writable_mount_…` 两条断言「失败之前已经落盘的发布各一行」，这一行的实际输出对不上。

这 4 条在开工时的快照上一样红，断言行相同（`/tmp/claude-1000/impl-rbf-2/baseline-copy-on-device.log`），不是这一批引入的，钉的值归实四。第 616 行点名的正是其中的 `failed_raise_…`，所以第 616 行证不了（见「交主 agent」第 3 条）。

`rows-renamed.txt` 那一轮作废，已由 `rows-renamed-fresh.txt` 取代。作废原因：我用 `rsync -a` 刷新改坏过的副本，mtime 回到旧值，cargo 用了旧产物，基线假红了一条（C557 那条）。之后删掉副本整份重拷、重跑，就是 `rows-renamed-fresh.txt`。

表里「行」是 `crates/mutations.tsv` 的行号。第 39、748、752、753、759、760、761、764 行点名的测试在最后一轮改了名（去掉开头的 `a_` 单字母段）；这 8 行在改名之后的最终代码上又证了一遍，下表写的是新名字。

| 行 | 改坏哪一处 | 红的点名测试 | 同一二进制里同时红的 |
|---|---|---|---|
| 745 | `root_record.rs:84` 写者 flags 写 0 | `the_writer_puts_the_unmount_marker_into_flag_bit_zero_…`（断言「卸载那一串写的根：flags 只有位 0」） | 无（1 条） |
| 746 | `root_record.rs:128` 读者一律读成不带记号 | `root_carrying_the_unmount_marker_is_accepted_…` | `the_writer_puts_…`（2 条） |
| 747 | `recovery.rs` 生效值只取根上的 F | `the_floor_takes_effect_from_the_system_configuration_…`（「F 生效值取系统配置里的 3」left 0） | `failing_to_write_the_new_floor_…`、`rolling_back_below_a_floor_only_the_system_configuration_…`（3 条） |
| 748 | `transaction.rs:385` 非抬 F 写退回「本盘两槽取大、都读不出写 0」 | `system_configuration_write_after_both_slots_…`（「盘 1 上那次轮换写出的一槽带整池的 F 生效值 3」） | 无（1 条） |
| 749 | SysPre 带旧生效值不带新 F | `raising_the_floor_writes_the_new_floor_…`（先写那一槽的 F） | `failing_to_write_…`（2 条） |
| 750 | SysPre 之后不过屏障 | `raising_the_floor_writes_the_new_floor_…`（头三步第三步不是屏障） | `normal_unmount_…`（2 条） |
| 751 | SysPre 一块盘都不写 | `raising_the_floor_writes_the_new_floor_…` | `failing_to_write_…`、`normal_unmount_…`（3 条） |
| 752 | SysPre 失败时分配器不换回 | `failing_to_write_the_new_floor_into_the_second_…`（「分配器与抬 F 之前逐项相同」） | 无（1 条） |
| 753 | 往下抬不拒 | `raising_below_the_effective_floor_…` | `failing_to_write_…`（2 条） |
| 754 | 卸载那一串的根不带记号 | `normal_unmount_…` | `unmount_markers_are_judged_…`、`cutting_a_stream_…`（3 条） |
| 755 | 卸载把 F 抬到现行根带的 F 而不是它的 txg | `normal_unmount_…`（「F 抬到现行那一版的 txg」） | 无（1 条） |
| 756 | 树表 0 条那一格拒时报的现行版本写成 txg 0 | `unmount_of_a_version_without_file_…` | 无（1 条） |
| 757 | 抬 F 做成时先写那一步的写账丢掉 | `raising_the_floor_writes_the_new_floor_…`（写账 2 次 / 8192 字节） | 无（1 条） |
| 758 | 抬 F 发布失败时先写那一步的写账不随错交回 | `a_raise_whose_second_empty_publish_fails_on_a_write_…`（commit_generated_fallback） | 无（1 条） |
| 759 | checker 读系统配置 F 读成 0 | `one_device_whose_system_configuration_floor_is_below_…` | `every_root_carrying_the_raised_floor_unreadable_…`、`marked_root_…`、`raising_the_floor_above_its_ceiling_…`、`a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_…`（5 条） |
| 760 | checker 把每条根读成不带记号 | `marked_root_raising_the_floor_above_the_previous_root_…`（带记号那一支判不到，走准入上限） | 无（1 条） |
| 761 | 带记号那一支 `<=` 改 `<` | `marked_root_raising_the_floor_…`（干净镜像 6 ≤ 6 判红） | 无（1 条） |
| 762 | checker 候选集下界只读最新根 F | `every_root_carrying_the_raised_floor_unreadable_…`（F 11 之下的根回到候选集，I-7.4 等红） | 无（1 条） |
| 763 | I-3.1「不适用」条件 `==` 改 `<=` | `every_root_carrying_…`（I-3.1 该报不适用） | `marked_root_…`（2 条） |
| 764 | I-7.12 恒成立 | `one_device_whose_system_configuration_floor_is_below_…`（该只红 I-7.12） | 无（1 条） |
| 765 | C557 判定拿掉 | `unmount_markers_are_judged_against_the_recorded_unmount_entry`（③ 准入根被打记号那一格） | `cutting_a_stream_…`（2 条） |
| 766 | 切写表不核 C557 | `cutting_a_stream_with_an_unmount_marker_outside_…`（「带卸载记号的根槽写不在任何卸载入口段里，切写表必须当场停」） | 无（1 条） |
| 767 | 记录核对器界不算系统配置 F | `the_record_checker_bounds_the_effective_floor_…`（「界抬到 5 … 开脱」left 1） | 无（1 条） |
| 768 | 入口段区间写成空 | `unmount_markers_are_judged_…`（①） | 无（1 条） |
| 769 | F 偏移减 1 | `the_rollback_floor_is_written_little_endian_at_481_…`（core lib） | 另 6 条：凡是写槽的单测都在 `assert_position` 上停（共 7 条） |
| 39 | 生效值退回「各盘根上 F 最大值的最小值」 | `floor_carried_by_the_system_configuration_takes_effect_…`（step_five_reuse） | `floor_carried_by_only_one_device_root_…`（2 条） |
| 42 | 回退候选集用最新根自己的 F | `rolling_back_below_a_floor_only_the_system_configuration_carries_…` | 无（1 条） |
| 51 | 抬 F 生效之后不放开扣住的槽（锚点换成 `FloorRaisedThroughTheSequence`） | `raising_the_floor_to_the_first_release_generation_…` | 无（1 条） |
| 485 | 机理串少一项（锚点换成 `effective_rollback_floor`） | `raising_the_floor_into_the_gap_left_by_a_rollback_…`（random_history） | `random_histories_fast_tier_…`（2 条） |
| 616 | 失败账不用已落盘账（锚点跟改后的调用） | **基线就红，不算证过** | 4 条都是基线红集 |
| 737 / 738 | 根记录读者 flags 判法（锚点从 `reader.get_u32()` 换成 `flags`） | `root_carrying_the_unmount_marker_…` | 737 另红 `the_writer_puts_…`（2 条）；738 1 条 |
| 740 | 非抬 F 写的 F 写 0（锚点换成 `PoolEffectiveFloor` 那一臂） | `system_configuration_write_after_make_filesystem_carries_…` | 无（1 条） |
| 742 / 743 | checker flags 判法（锚点换成 `flags`） | `the_checker_accepts_the_unmount_marker_…` | 无（各 1 条） |

每条新测试至少有一行证过：
- `rolling_back_below_a_floor_only_the_system_configuration_carries_…`：第 42、747 行。
- `the_floor_takes_effect_…`：第 747 行。
- `unmount_of_a_version_without_file_…`：第 756 行。
- `raising_below_…`：第 753 行。
- `failing_to_write_…`：第 752 行。
- C557 两条：第 765、766、768 行。
- 记录核对器那条：第 767 行。
- `root_record` 写者那条：第 745 行。
- checker 三条：第 759–764 行。

层 0：这一批没有新测试落在名字含 layer0 的二进制里，没有留给 59 号的只追加行。

## 变异行

追加的 25 行，`crates/mutations.tsv` 第 745–769 行。变异名都带前缀「实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：」，下面只列冒号之后那一段：
- 745 根记录写者不照 unmount_marker 写 flags、卸载那一串的根也写 0
- 746 根记录读者收了卸载记号却把它丢掉、一律读成不带
- 747 F_生效 不读系统配置里的 F、只取根上带的
- 748 非抬 F 的系统配置写退回「本盘两槽取大、都读不出写 0」（主 agent 2026-09-26 Q1 改掉的那一种）
- 749 抬 F 先写系统配置那一步不带新 F、照带盘上旧的生效值
- 750 抬 F 先写系统配置之后不过屏障就发第一条根
- 751 抬 F 那一串不先写系统配置（一块盘都不写）
- 752 先写系统配置那一步失败时分配器不换回抬 F 之前那一份（扣住的槽不放、回收的不回 defer）
- 753 新 F 低于盘上的 F 生效值也照抬（往下「抬」不拒）
- 754 卸载那一串的根不带卸载记号
- 755 卸载把 F 抬到现行那一版带的 F（原地不动）而不是它的 txg
- 756 树表 0 条的一版上卸载，拒的时候报的现行版本不是那一版
- 757 抬 F 做成时先写系统配置那一步的写账丢掉
- 758 抬 F 那一串发布失败时先写系统配置那一步的写账不随错交回
- 759 checker 不读系统配置里的 F（读成 0）
- 760 checker 把每条根都读成不带卸载记号
- 761 I-7.9 带记号那一支把「不高于前一条根的 txg」判成「低于」
- 762 checker 回退候选集的下界只读最新根带的 F、不读系统配置里的（C556 之前的读法）
- 763 最新根带的 F 低于 F 生效值时 I-3.1 照判（按它自己的 F 记的账对按生效值取的候选集并集）
- 764 I-7.12 不比系统配置里的 F（每块盘恒成立）
- 765 C557 那一道拿掉：带卸载记号的根槽写落在卸载入口之外也不列
- 766 层 0 切写表不核 C557（有带记号的根槽写也照切）
- 767 记录核对器的回收谓词界不算系统配置槽写带的 F（C556 层 0 那一半之前的读法）
- 768 录制流记入口那一段时区间是空的（入口发的操作一条都不算进去）
- 769 系统配置里 F 的偏移不从字段表合计推（差一个字节）

改了的旧行（锚点变了的换锚点，点名的测试改了名或换了的跟着换）：
- 39：变异名改成「步 5（2026-09-26 取 SysPre 之后改写这一行）：F_生效 退回「各盘根上 F 最大值的最小值」、不读系统配置」；原文换成新式子，替换文换成改前的算法；点名测试换成 `floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it`。
- 42：原文与替换文不变；点名测试换成新文件里的 `rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write`（原来点名的那条改成了「拒」，见「钉死值」第 2 行）。
- 51：锚点 `Ok(RaisedFloor {` → `Ok(FloorRaisedThroughTheSequence {`。
- 485：锚点 `{newest_rollback_floor}` → `{effective_rollback_floor}`。
- 616：锚点加一行 `Some(writes_before_the_first_publish),`，因为原来的两行原文在文件里命中 2 次。
- 737、738：锚点 `reader.get_u32()` → `flags`。
- 740：锚点换成 `RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor => pool_effective_floor,`，替换成 `CheckpointTxg(0)`。
- 742、743：锚点 `read_u32(slot, ROOT_FLAGS_OFFSET)` → `flags`。

门禁 33 号原样：`✓ 148 个实验二进制都有成形的变异表，1672 条变异的原文各命中源码一次；crates/mutations.tsv 764 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）`

## 钉死值的改前、改后

| 位置 | 钉的是什么 | 改前 | 改后 | 为什么变 |
|---|---|---|---|---|
| `second_transaction_step_five_reuse.rs:265`（原名 `one_device_carrying_the_floor_alone_does_not_take_effect_on_remount`） | 改坏盘 1 那条带 F 11 的根之后，重开时每盘 defer 里剩几槽 | `(true, 125, 15)` | `(true, 90, 15)`；改名 `floor_carried_by_the_system_configuration_takes_effect_…` | 生效值取 SysPre：系统配置里还有 11，F_生效 = 11，回收与根槽都好时一样多 |
| `second_transaction_step_five_reuse.rs:547`（原名 `roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates`） | 改坏那条根之后回退到 txg 9 | 做成，新实例根 F = 0 | 在任何写之前拒成 `BelowEffectiveFloor`，盘面逐项不变；改名 `floor_carried_by_only_one_device_root_and_the_system_configuration_keeps_…` | 同上 |
| `second_transaction_supplement_two_commit_generated_fallback.rs` `a_raise_whose_second_empty_publish_fails_…` | 注入写错的全池序号 | 第一次空发布的写数 + 1 | 2（先写系统配置）+ 第一次空发布的写数 + 1；另钉先写那一步 2 次写、随错交回 | SysPre 在第一次发布之前多两次写 |
| 同上 `sum_of_accounts` | 失败账相加 | 已落盘 + 失败账 | 先写系统配置那一步 + 已落盘 + 失败账 | 同上；不加它，账与录制流对不上 |
| `root_record.rs` 单测 `root_carrying_the_unmount_marker_…` | 带记号的槽读回什么 | `Some(sample())` | `Some(RootRecord { unmount_marker: WrittenByTheUnmountSequence, ..sample() })` | Q2：记号进 `RootRecord`、照实读出 |
| `checker/src/image.rs:37` | `IMPLEMENTED_INVARIANTS` 条数 | 46 | 47（加 `"I-7.12"`） | 立 I-7.12；`checker_known_bad_images.rs` 那条「每条都有坏镜像」按集合比，跟着加了坏镜像 |
| `first_transaction_on_device` 抬 F 窗口结果行 | `name=publish_writes_against_device window=raise_rollback_floor …` | `publishes=2 by_kind_write_calls=42 …` | `publishes=2 writes_outside_any_publish_write_calls=2 by_kind_write_calls=44 by_kind_written_bytes=566272 device_write_calls=44 device_written_bytes=566272 matches=true`；多一行 `name=writes_outside_any_publish window=raise_rollback_floor write_calls=2 written_bytes=8192 …` | SysPre 两次整槽写进窗口合计 |
| 同上 `name=raise_rollback_floor … segments=` | 抬 F 那一段的段序列 | （基线测试在更早一处就红了，没取到） | 宿主上实测 `segments=2+16+2+1+18+2+1+2 closed_form=327693 operations=49` | 先写系统配置那 2 次写加一道屏障，自成最前面一段 |

改后那一格是在自己的副本 `on-device-check` 里量的：先把基线就红的 `"16+2+1+2"` 那一处断言临时改掉，再打出实际结果行。副本已删，日志在 `/tmp/claude-1000/impl-rbf-2/on-device-check-2.log`。

系统配置槽与第一个事务的字节：mkfs、取号、暖机、第一个事务这条流里，每次系统配置写带的 F 都是整池生效值 0，与改前取的「本盘两槽最大值 0」逐字节相同。`system_configuration_mutability_classes.rs` 钉的两个样本槽 sha256 照旧通过（探索那一轮跑的，2 passed；这份文件这一批没改）。

## 第 4 步那几样的末尾原样输出

动到的测试二进制，整个二进制跑，最终代码（`/tmp/claude-1000/impl-rbf-2/touched-final-summary.txt`；最后三行是改名与最后一次 rustfmt 之后重跑的）：
```
lib:singlefs-core exit=0 test result: ok. 123 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
lib:singlefs-checker exit=0 test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
lib:singlefs-format exit=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
lib:singlefs-harness exit=0 test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:rollback_floor_written_into_the_system_configuration_first_and_normal_unmount exit=0 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:checker_known_bad_images exit=0 test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:second_transaction_step_five_reuse exit=0 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:second_transaction_supplement_two_commit_generated_fallback exit=0 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:second_transaction_supplement_two_instance_table_chain exit=0 test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:second_transaction_parallel_line_two_mounted_read exit=0 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:system_configuration_rollback_floor_and_layout_identity exit=0 test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
bin:first_transaction_on_device exit=101 test result: FAILED. 12 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:rollback_floor_written_into_the_system_configuration_first_and_normal_unmount exit=0 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:checker_known_bad_images exit=0 test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
test:second_transaction_step_five_reuse exit=0 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
```
`first_transaction_on_device` 红的 4 条就是基线红集那 4 条（见证红一节），断言行与开工时快照上的逐条相同。

`cargo fmt --all -- --check`：退出码 0，无输出。

`cargo build --offline --all-targets`：退出码 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 11.52s`。

`check.sh` 那一套 lint 的 clippy（`--keep-going --all-targets --all-features -- -D warnings` 加七条编码纪律 lint），退出码 101，全部 error 行：
```
error: this assertion is always `true`
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:4059:9
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 1 previous error
```
这一处 HEAD 上就有，归实四。把它之外的目标分开跑同一套 lint：`-p singlefs-format -p singlefs-core -p singlefs-checker --all-targets` 退出码 0；`-p singlefs-harness --lib --bins --tests` 只报同一处 e156:4059。

探索阶段在中间状态上跑过、这一批没改的受影响二进制（不在交回的验证范围里，照实写）：
- `second_transaction_supplement_three_random_history`：22 passed。
- `second_transaction_supplement_three_crash_injection`：7 passed。
- `second_transaction_supplement_three_fault_injection`：7 passed、2 failed。红的是 `a_write_error_after_the_acquisition_…`（:786）与 `a_swallowed_write_after_which_the_checker_flags_only_i_3_1_…`（:296），开工时快照上同样这两条红、断言相同（`/tmp/claude-1000/impl-rbf-2/baseline-fault-injection.log`），不归我。
- `first_transaction_device_log_check`：5 passed。
- `second_transaction_supplement_two_release_checksum_quarantine`：13 passed。
- `second_transaction_supplement_two_root_ring_turn_in_one_mount`：3 passed。
- `system_configuration_mutability_classes`：2 passed。

## 门禁判定行

都在主工作区最终状态上跑，命令是 `nice -n 19 bash .claude/gate.d/<阶段>.sh`。74 号另外经内存包装与 `capped.sh 12` 跑，用自己的 target。
- 27（退出码 1）：`  ✗ 格式常量在 kb 与实验源码之间对不上：`，唯一一行 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:111  const SYSTEM_CONFIGURATION_BYTES = 481，而 .claude/kb/layout/01-first-txn.md 定的现行值是 489`。E142 装置，不归我，与实一交回时相同。
- 33（退出码 0）：原样见「变异行」一节。
- 53（退出码 0）：`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
- 74（退出码 0）：`  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：`
- 80（退出码 0）：`  ✓ 152 个实验二进制各自至少有一条绝对值断言（research/e7-index-bench/src/bin 下 148 份，别处 src/bin 下以 e<数字>_ 开头的 4 份）`
- 89（退出码 77）：`  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`
- 92（退出码 0）：`  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，110 个格式常量里变了 2 个（checker 在同一次改动里跟了 2 个，按滞后表放行 0 个），都不欠 checker 跟进`
- 93（退出码 0）：`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 55 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:33，常量声明没写在一行里，解不出值））`
- 94（退出码 0）：`  ✓ checker 与实现只共享常量模块 \`singlefs-format\`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 \`singlefs_core\`；共享模块 1 份源码的正文 304 行里没有分支与循环`
- naming-lint `bash .claude/scripts/naming-lint.sh <仓根>`（退出码 1）：`  ✗ 242 处名字不合命名纪律（查了 264 个 .rs 文件、62533 个声明的名字）`。与实一交回时同为 242；这一轮一度升到 249，都是我新起的以 `a_` 开头的测试名，已全部改名，现在 242 处里没有一处是这一轮写的。

跑之前看过负载：没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，也没有别的会话的 `cargo` 或 `gate.sh`，没有等锁。

## 交主 agent 的

### 条款没写、我停在那一处或取了一个的（设计问题）

- **D1 树表 0 条的一版上卸载**（`mount.rs:112`、`:1216` 起）：准入抬 F 的走法（影子账重算、回收扣到生效、整串预演）今天只写了带文件的一版。树表 0 条那一版上，卸载那一串的空发布是零单元发布；要不要回收、预演什么，条款没写。我做成错误成员 `UnmountOfAVersionWithoutFileWhoseEmptyPublishesAreUndecided`，在任何写之前拒，用例 `unmount_of_a_version_without_file_is_refused_before_any_write` 钉住盘面逐项不变。走得到：只做过 mkfs 的池，可写挂载之后立刻卸载。
- **D2 往下「抬」F**（`mount.rs:104`、`:1293`）：新 F 低于盘上现算的生效值时怎么办，条款没写。我做成 `RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided`，在任何写之前拒。走得到：同一个进程里先写系统配置那一步只写进了一部分盘，之后再要抬到比那个值低的 F（用例 `failing_to_write_the_new_floor_…` 的后半段）；或者调用方直接传一个低值。另外，SysPre 写本身带的是 max(新 F, 写之前现算的生效值)（`transaction.rs:393`）：拒的那一判与写之间隔着两次读盘，这样一次瞬时读错也不会把系统配置写回更低的值。
- **D3 生效值里根那一半**（`recovery.rs:860`）：照 Q1 的写法取「环里读得出的根」，包括被抛弃时间线上的根。D16 已定项 1「生效」那一行字面是「各幸存盘最新持久有效根」。两者只在被抛弃时间线上的根带着更高的 F 时不同。D16 自己写着「被抛弃时间线上的根带的 F 算不算进 F_生效」仍开着。
- **D4 checker 的生效 F 根那一半取「最新根带的」**（`walk.rs:4747`），不是 core 的「环里全部读得出的根」。checker 今天就是按最新根取候选集下界，我只并进了系统配置里的 F。合法状态下 F 沿时间线单调，两种读法相同；带更高 F 的被抛弃根读得出时两边可以不同。要不要拉成一种读法，与 D3 一起定。
- **D5 I-3.1 新增一处「不适用」**（`walk.rs:5009`、`:5066`）：最新根带的 F 低于 F 生效值时，I-3.1 这一格报不适用。两种情形会走到这里：先写系统配置之后、带新 F 的根落盘之前；带新 F 的根全读不出。这时最新根的记账是按它自己那个较低的 F 记的，而候选集按生效值取、并集更小，两边不可比，照判会在合法状态上红（C556 那份镜像上就会）。另一种做法是 I-3.1 的并集按最新根自己的 F 取、别的几条按生效值取。条款没写选哪一种。
- **D6 记号的几个取法**：
  - 由记录施加出来的那一版（`recovery.rs:2013`）一律不带记号：新根段里没有 flags。
  - 零单元发布（`transaction.rs:1091`）写死不带：卸载在树表 0 条那一版上不支持，见 D1。
  - SysPre 那次写的 tail 与实例代号，取现行那一版末条记录的计数器与实例代号（`mount.rs:1401` 起），即它末次轮换写的那一对。
- **D7 I-7.12 的「评估过」口径**：每块有自证过的系统配置槽的盘都判一格，这块盘上一条根都没有时那一格照样成立。所以层 0 每个状态都评估得到它，落在 `first_transaction_step_seven_layer0.rs` 等「其余每个状态都评估」那一档。没跑层 0，没核。
- **D8 C557 接进层 0 的方式**：只登记卸载一个入口。没记入口的流里一条带记号的根槽写都不许有，在 `crash.rs:459` 断言停。今天没有一条层 0 流调了卸载，所以这一道在层 0 里只核「没有带记号的根」；带卸载入口的层 0 流没有新开（崩溃点没多罩，层 0 流一条都没加）。要罩「卸载那一串」的崩溃状态，得新开一条流，由你定。
- **D9 `MountError` 两个成员改成装 `Box<PublishSequenceFailed>`**（`mount.rs:73`、`:82`）：`PublishSequenceFailed` 多了一个字段之后，`MountError` 超过 clippy `result_large_err` 的 128 字节，这是公开错误类型的形状变化。
- 这一批按调度记录不做、另有归属的：「4 个不同状态」去重（实三）；删见证、回退行、`mount_rollback*`（实三）。

条款没写、又不是分支的，我没加：`Unmounted` 与 `RaisedFloor` 之间的转换、`UnmountMarker` 的 `Default`、`RecordedPublishEntry` 除卸载之外的入口（准入、发布、挂载）。

### 写范围之外、这一批牵连到的

1. kb（书记员）：
   - `invariants.md` I-7.12 那一行，状态改成已实现：checker `walk::judge_system_configuration_floor_against_the_roots_on_each_device`，坏镜像 `checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_…`，变异 759、764。
   - I-7.9 状态格：带卸载记号那一支已实现，坏镜像 `marked_root_raising_the_floor_above_the_previous_root_…`，变异 760、761。
   - 第 12 行「池级 checker 判 44 条」：`IMPLEMENTED_INVARIANTS` 现在 47 条，含还没删的 I-7.10、I-7.11。
   - `checks-owed.md` C556：checker 那一半与层 0 记录核对器那一半已做，用例 `every_root_carrying_the_raised_floor_unreadable_…`、`the_record_checker_bounds_…`；层 0 oracle 的别处没普查。C557：判定已做，用例 `unmount_markers_are_judged_…`、`cutting_a_stream_…`；没有调卸载的层 0 流（D8）。C419 的实现那一半也跟着做了。
2. `.claude/gate.d/55-qemu-first-transaction.sh` 的 `raise-rollback-floor` 期望行（第 96 行起）：
   - `segments=8+2+1+10+2+1+2` 前面要多一段先写系统配置的 2 写。宿主上现在是 `2+16+2+1+18+2+1+2`；宿主与虚机几何不同，虚机上的数没量。
   - 多了一行 `name=writes_outside_any_publish`。
   - 比对行 `publishes=2 .*matches=true` 仍匹配，中间多一段 `writes_outside_any_publish_write_calls=2`。
3. `crates/mutations.tsv` 第 616 行点名的 `failed_raise_of_the_rollback_floor_…` 在 HEAD 上就红（`first_transaction_on_device` bin 基线红 4 条，钉的 `"16+2+1+2"` 与「已落盘各一行」过时），59 号跑到它会报没红。这 4 条归实四。
4. `second_transaction_step_zero_layer0.rs` 的 `ReuseAfterRaisingFloor` 两条脚本钉死的段序列，在开工时的快照上就与实际不符。我在自己的副本里只跑 `prepare`、不枚举，实际是 `[…, 26, 2, 1, 26, …]`，钉的是 `[…, 18, 2, 1, 18, …]`。SysPre 之后抬 F 那一处又变成 `…, 4, 16, 2, 1, 18, …`（多一个 4 写段）。我改过一版又逐字改回，这份文件没动，归实四。层 0 没跑。
5. E158 装置 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 调 `raise_rollback_floor`：编得过，行为变了（多 SysPre 两写，生效值换式子），它钉的数可能动。没跑，没动。
6. e156 实验 bin（`e156_allocation_basis_counts.rs`，我这一批没改它）调 `effective_rollback_floor` 与 `raise_rollback_floor`：式子与 SysPre 都会改变它的产物，要不要复跑、按 69 / 87 号换产物，由你定。
7. `second_transaction_step_three_formatted_pool.rs` 断言「不在不适用名单里的每条都 Holds」：I-7.12 应当成立，没跑（这一批没改它）。

## 草稿与产物

都在 `/tmp/claude-1000/impl-rbf-2/`，没进仓；写范围不含 `research/results/`，要留的请你拷：
- `impl-rbf-2.patch`：这一批的补丁。
- `mut/`：`make_rows.py`、`mutations-append.tsv`、`prove2.sh`、`prove4.sh`，判定 `rows-final.txt`、`rows-renamed-fresh.txt`，逐行日志在 `logs-final/`、`logs-renamed-fresh/`。
- `touched-final/`：动到的二进制的最终日志。
- `clippy-final*.log`、`build-final.log`、`fmt-check-final.log`、`gate-final-*.log`、`gate-33-final.log`、`gate-74.log`、`naming-lint-final.log`。
- `baseline-copy-on-device.log`、`baseline-fault-injection.log`：基线红的证据。
- `on-device-check-2.log`：抬 F 窗口的实测行。
- `shape-check*.log`：layer0 段序列的核对。
- `progress.md`：中途的进度记录。

## 交回前删掉的编译目录与仓副本

每个先 `du -sh` 记大小再 `rm -rf`：
- `/tmp/claude-1000/impl-rbf-2/target-main`：11G（主工作区用的编译目录）
- `/tmp/claude-1000/impl-rbf-2/target-gate74`：791M
- `/tmp/claude-1000/impl-rbf-2/baseline-copy`：1.4G（开工时快照的副本，连同它的 target）
- `/tmp/claude-1000/impl-rbf-2/shape-check`：1.2G
- `/tmp/claude-1000/impl-rbf-2/on-device-check`：554M
- `/tmp/claude-1000/impl-rbf-2/mut/work`：1.7G（证红副本，连同它的 target；中途整份删过重拷一次，那一次没记大小）
- `/tmp/claude-1000/impl-rbf-2/mut/source`：5.9M
- `/tmp/claude-1000/impl-rbf-2/pristine`：5.8M（开工时 crates 快照，补丁核完之后删）
- `/tmp/claude-1000/impl-rbf-2/patch-work`：12M

现在整个目录只剩日志、判定、补丁、脚本与本报告，共 3.0M。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表（59 号）归 `crash-verifier`；没提交，没做任何 git 写操作。
- 没跑名字带 layer0 的测试二进制；没跑全量 `cargo test`、`check.sh`、`gate.sh` 整轮，也没跑 54、55、57、59、87 号。
- 这一批没改、但受影响的二进制，只在探索阶段跑过一部分（见第 4 步那一节），其余没跑：E158 bin、e156 bin、`second_transaction_step_three_formatted_pool` 等。
- 没改 kb、`.claude/gate.d/`、E142 装置、E158 装置（要改成什么写在上面）。
- 中途收到主 agent 一次例行询问（跑满一小时时），照要求写了 `progress.md`，范围没变。
