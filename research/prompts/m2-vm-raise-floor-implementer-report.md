# 实现员报告：impl-m2-vm-raise-floor（实二四：真设备二进制加抬 F 模式，增补 2 收口表第 58 行「二进制那一侧判相等」）

时刻一律 UTC（东京 = UTC+9）。仓副本 `repo/`（16:34:48 从主工作区 rsync，HEAD `e980a21`；我那三个文件的原件快照在 `base/`，不改动的对照副本 `pristine/`）。
主工作区一个字没动：改动只在补丁里，变异行只在追加文件里（锚点要补丁打上之后才命中）。
**16:50 之后主工作区换了底**（别的会话打进一批 core / checker / harness 改动：`mount.rs`、`transaction.rs`、`recovery.rs`、新文件 `code_two_tree.rs` 等 20 来个文件，变异表 591 → 614 行），我那三个文件没被动过。16:54:19 按新底重拷 `repo2/`、打补丁，第七节的测试、clippy、build 与第四节的全部证红在 `repo2/` 上重跑了一遍，结果与第一轮逐条相同（第七节末）；16:58 核过：主工作区自 16:54 起只有 `e158_root_choice_repair.rs` 又变了，补丁仍 `git apply --check` 退出 0。

## 一、交付物

| 样 | 路径 | 说明 |
|---|---|---|
| 补丁 | `/tmp/claude-1000/impl-m2-vm-raise-floor/impl-m2-vm-raise-floor.patch` | 只含 3 个文件，不含 `crates/mutations.tsv`；16:50、16:58 两次在主工作区 `git apply --check` 退出 0 |
| 追加变异行 | `/tmp/claude-1000/impl-m2-vm-raise-floor/mutations-append.tsv` | 6 行整行，追加在 `crates/mutations.tsv` 末尾；锚点在「主工作区 + 补丁」上各恰好命中一次 |
| 报告 | `/tmp/claude-1000/impl-m2-vm-raise-floor/report.md` | 本文件 |

补丁在主工作区上 `git apply --stat`，原样：

```
 crates/singlefs-harness/src/on_device_modes.rs     |   62 ++
 .../src/bin/first_transaction_on_device.rs         |  577 +++++++++++++++++---
 .../src/bin/first_transaction_device_log_check.rs  |  233 +++++++-
 3 files changed, 733 insertions(+), 139 deletions(-)
```

这一轮写过的文件（都在草稿目录）：`repo/` 下上面那 3 个文件；`mutations-append.tsv`；`impl-m2-vm-raise-floor.patch`；`new-raise-tests.rs`（拼进用例模块的草稿）；`proof/prove.sh`（证红脚本）与 `proof/*.log`、`proof/*.txt`；各 `*.log`；本报告。`crates/mutations.tsv` 在主工作区一行没加（见第五节）。

## 二、三条验收各落在哪（行号是 `repo/` 的；补丁只动这三个文件，`repo2/` 上同一行号）

**① `OnDeviceRunMode` 多一个抬 F 的模式。** `on_device_modes.rs:28` `OnDeviceRunMode::RaiseRollbackFloor`，命令行写法 `raise-rollback-floor`（`:62`），`ALL` 六个；`:40` `PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenRaiseOfTheRollbackFloor`。历史 = `second-instance` 那条路（mkfs → 取号 → 暖机 → A txg 3 → B txg 4 → 冷重开可写挂载：写行 txg 5、暖机 txg 6、7 → 发布 C txg 8），然后同一次挂载里抬 F：`on_device_modes.rs:220` `raise_the_rollback_floor_after_the_third_version` 调 `raise_rollback_floor`（`ShadowLedger::On`），推两次带 F 的空发布 txg 9、10，两块盘各落一条（D16（发布语义） 已定项 1「生效」）。冷重开择 (2, 10)，读回第三版（抬 F 的空发布不碰 inode 树）。虚机二进制与宿主检查调的是这同一个函数，两边不各抄一份。

**② 宿主检查逐项比。** `first_transaction_device_log_check.rs:55` `ProgramWindow::RaiseRollbackFloor`（段名 `raise_rollback_floor`）；`:195` `rerun_the_second_instance_on_the_host`（原来内联在重跑函数里的冷重开、挂载、发布 C 挪出来，两个模式共用），`raise-rollback-floor` 接着抬 F、`:175` 记这一段的终点。判法没动：同一个 `compare_one_device`，程序的事件是设备侧日志的逐项前缀、之后至多一个关机 FLUSH。宿主重跑这个模式各段步数 `21+3+13+23+23+50+23+30`（最后 30 是抬 F 那一段；数取自第四节 row 6 变异日志里那一行 `name=host_rerun`，那一行的段名被变异改了，步数没变）。

**③ 结果行带抬 F 那一串的账、与设备一层逐项比。** `first_transaction_on_device.rs:937` `raise_the_rollback_floor_and_describe`：窗口从抬 F 之前那一刻到它返回；成功时打 `name=raise_rollback_floor`（`floor_before`、`requested_floor`、`ceiling`、`publishes`、`root_txgs`、`reclaimed`、`abandoned_roots_unreadable`、挂钟、段序列、逐盘计数）、每次空发布一行 `name=publish_writes publish=raise_rollback_floor`、一行 `name=publish_writes_against_device window=raise_rollback_floor`（`:1046`，与别的窗口同一个 `publish_writes_against_device`），`matches=false` 并进整程序退出码 1；失败时（`:970`）`RaiseFloorSequencePublishFailed` 带的已落盘账与失败账照可写挂载失败那一段的判法（`describe_failed_window`）打、与设备一层比，再以退出码 5 退出；别的 `MountError` 成员都在第一次空发布之前，只打停下原因。`main` 里 `:1350` 那一支走它；分段登记表 `:93` 多一段 `raise_rollback_floor`。宿主上跑出来的四行（`nanoseconds` 删掉，其余原样，`publish_writes` 两行截到 700 字）：

```
name=raise_rollback_floor floor_before=0 requested_floor=0 ceiling=0 publishes=2 root_txgs=9,10 reclaimed=0 abandoned_roots_unreadable=0 operations=30 segments=8+2+1+10+2+1+2 closed_form=1290 device_0_writes=13 device_0_written_bytes=147968 device_0_force_unit_access_writes=1 device_0_barriers=4 device_1_writes=13 device_1_written_bytes=147968 device_1_force_unit_access_writes=1 device_1_barriers=4
name=publish_writes publish=raise_rollback_floor txg=9 write_calls=13 written_bytes=147968 …
name=publish_writes publish=raise_rollback_floor txg=10 write_calls=13 written_bytes=147968 …
name=publish_writes_against_device window=raise_rollback_floor publishes=2 by_kind_write_calls=26 by_kind_written_bytes=295936 device_write_calls=26 device_written_bytes=295936 matches=true
```

顺带改的：`SecondInstanceRun`（`:612`）多带分配器、现行那一版、注入计划，抬 F 接在它后面；`main` 里把重开闭包与「挑出分段行」抽成 `reopen_by_path`（`:1318`）与 `emit_the_lines_after_the_second_version`（`:435`），两支共用，前五个模式打的行一行不变。

推翻条件：虚机里真跑 `raise-rollback-floor`，宿主检查在抬 F 那一段判红（设备收到的与录制流投出的不同），或 `window=raise_rollback_floor` 那一行 `matches=false`，或冷重开不是 `2:10`。

## 三、停下交主 agent 的问题

1. **F 抬到多少，我按「现行那一版的 F」写了，F 在这段历史上不动（0 → 0）。** 条款（D16（发布语义） 已定项 1）只定上限与「一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)」，没定测试档抬到哪。这段历史上非空有效根只有 txg 3、4、8 三条（不足 4 条），上限取环里最旧的有效根 txg 0，结果行里 `ceiling=0` 就是实现算出来的这个数（用例钉着 `requested_floor=0 ceiling=0 reclaimed=0`）。所以这一档跑的是「抬 F 的那串空发布、推到每块盘各一条」这条写路与它的账（第 58 行要的就是这一串的账），**盘上根记录的 F 字段恒 0、回收集合为空**，设备侧比对对「F ≠ 0 的根记录字节」没有判别力。要 F 抬过 0，得在发布 C 之后再覆盖写一次攒出第 4 条非空根（按 `rollback_floor_ceiling` 的式子推算，上限会变成 min(每块盘上最新的, txg 3) = 3；没跑过），那是再加一次发布、一版内容、一段窗口，超出派发的三条验收，我没做；要不要做交你定。
2. **合并时要跟着实十九改的地方**（不是设计问题）：补丁依赖 `mount.rs` 的 `raise_rollback_floor` 签名、`RaisedFloor` 的四个字段、`PublishSequenceFailed` 的三个字段，二进制里对 `MountError` 的穷举 match 从一处变成两处（可写挂载 `:733` 起那一处、抬 F `:970` 起这一处）。16:54 那一版主工作区的 `mount.rs`（已换了底）上补丁编得过、测试全绿；此后谁再加减 `MountError` 成员或改这几个字段，两处都要跟着改（编不过，不会静默漏）。
3. **观测，不归我**：主工作区的门禁 55 号仍按「宿主只重跑第一个事务」判 `second-transaction` / `second-instance`（要求 `check-exit` 为 1、红在程序之后），而宿主检查早已重跑发布 B、挂载、发布 C（收口表第 30 行）；绿样本 `green/second-instance/check.txt` 是 09-21 的旧输出（`expected_writes=23`）。新模式照新的宿主检查应判 0（见第六节），后两档要不要一起改由你定。

条款没写、我没加的非分支项：无（没加 trait 实现、derive、访问器）。这一轮没写 `todo!` / `assert!`；新加的 `expect` 只在用例里。

## 四、每条新测试「改坏哪一行 → 哪条断言红」

基线（`pristine/`，16:35 起跑，debug）：三个测试二进制全绿，基线红集为空——

```
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.36s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
```

证红办法：`proof/prove.sh` 每条变异一份新副本（从 `repo/` 连 target 一起 rsync，改坏一处、对被改文件 `touch`、跑那条测试所在的整个测试二进制，跑完删副本，不拷回）。全部在 debug 下红在测试断言上（被测代码这几处没有 `debug_assert`），没跑 release。行号是 `repo/` 的。六行新变异全部证过红，没有留给 59 号的行。

| 变异行 | 改坏哪一行 | 红的测试 → 断言 | 同时红的 |
|---|---|---|---|
| 1 | `on_device_modes.rs:62` 写法改成 `raise_rollback_floor` | `every_mode_argument_reads_back_as_the_same_mode_and_unknown_text_is_refused` → `on_device_modes.rs:279` `from_argument("raise_rollback_floor") == None`（left `Some(RaiseRollbackFloor)`） | 无 |
| 2 | `on_device_modes.rs:111` 起那一支读回改成 `second_file_content()` | `the_three_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one` → `:310`「抬 F 的空发布不碰 inode 树…」 | 无 |
| 3 | `first_transaction_on_device.rs:1044` `raise_publishes.push(&raise_publish.writes);` → `raise_publishes.clear();` | `raise_rollback_floor_mode_pushes_an_empty_publish_onto_each_device_and_its_window_matches_the_device_layer_count` → `:2347` `publishes == Some("2")`（left `Some("0")`） | 无 |
| 4 | `first_transaction_on_device.rs:978`–`:979` 抬 F 失败那一支把已落盘账换成 `&[]` | `failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count` → `:1965`（`assert_the_failed_window_is_reconciled`）「同一段里失败之前已经落盘的发布各一行」 | 无 |
| 5 | `first_transaction_on_device.rs:101` 分段登记表里删掉 `"raise_rollback_floor",` | `every_mode_registers_its_own_segments_with_no_repeats` → `:1838`「raise-rollback-floor 比 second-instance 多抬 F 那一段…」 | 无 |
| 6 | `first_transaction_device_log_check.rs:175` 抬 F 那一段的终点记成 `ThirdTransaction` | `raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed` → `:847` `.expect("raise-rollback-floor 跑了抬 F 那一段")` | `every_mode_reruns_its_own_windows_and_matches_a_log_that_received_exactly_the_program_events` → `:590` 段名清单（`…,third_transaction,third_transaction` ≠ `…,third_transaction,raise_rollback_floor`） |

`failed_raise_…` 是原名改写（原来直接调 `raise_rollback_floor` + `describe_failed_window`，现在走二进制自己的 `raise_the_rollback_floor_and_describe`，历史多了发布 C，数仍是已落盘 13 + 失败 2 = 15）。压着它的两行旧变异在新用例上重证过，都红：
- 第 515 行（新底上是第 494 行）「…抬 F 那一串失败时 core 不交已落盘那几次空发布的账…」→ `failed_raise_…` 红在 `:1965`（同一条断言）；同时红：无。
- 第 517 行（新底上是第 496 行）「…抬 F 那一串失败时 core 不交写入口的失败账（与可写挂载共用那一处）」→ `failed_raise_…` 与 `failed_writable_mount_…` 都红在 `:1970`「失败账一份」。

扩了断言、没新建的：`every_mode_reruns_…`（`OnDeviceRunMode::ALL` 多一个，穷举 match 逼着写新模式的段清单）、`every_mode_registers_…`、`on_device_modes` 那两条。

## 五、变异表

**追加 6 行**（`mutations-append.tsv`，补丁打上之后整行追加在 `crates/mutations.tsv` 末尾；先打补丁再追加，反过来 59 号会报锚点不命中）：

1. 增补 2 收口表第 58 行（真设备抬 F 模式）：模式名 raise-rollback-floor 写成下划线（跑批脚本送来的参数认不回）
2. 增补 2 收口表第 58 行（真设备抬 F 模式）：raise-rollback-floor 冷重开该读回的取成第二版
3. 增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制抬 F 窗口按种类那一侧不算那一串空发布
4. 增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制抬 F 失败时不用抬 F 交回的已落盘账
5. 增补 2 收口表第 58 行（真设备抬 F 模式）：抬 F 那一段没登记进 raise-rollback-floor 的分段表
6. 增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查把抬 F 那一段的终点记成发布 C（抬 F 那一段不单独比、红了也指不到它）

**要删、要替代的行：没有。** 我那三个文件上已有的 14 行变异，锚点在「主工作区 + 补丁」上逐行恰好命中一次（脚本逐行数过，`checked 14 bad 0`；新底上连同追加的 6 行一起数，`checked 20 bad 0`）；点名 `failed_raise_…` 的那两行（开工时第 515、517 行，新底上第 494、496 行）用例名没变、在改写后的用例上重证红（第四节）。

## 六、门禁 55 号要改的（交主 agent 改）

- `MODES` 加 `raise-rollback-floor`（六档）。
- `cold_root_of`：`raise-rollback-floor` → `2:10`。
- `required_lines_of`：发布 B 那四条的 case 加上 `raise-rollback-floor`；`second-instance` 那九条的 case 加上 `raise-rollback-floor`；再加这个模式独有的（宿主上跑出来的值，几何与虚机相同的 `e142_parameters(512, 512)`）：
  ```
  name=raise_rollback_floor floor_before=0 requested_floor=0 ceiling=0 publishes=2 root_txgs=9,10 reclaimed=0 abandoned_roots_unreadable=0 nanoseconds=
  name=raise_rollback_floor .*segments=8+2+1+10+2+1+2 closed_form=
  name=publish_writes publish=raise_rollback_floor txg=9 write_calls=
  name=publish_writes publish=raise_rollback_floor txg=10 write_calls=
  name=publish_writes_against_device window=raise_rollback_floor publishes=2 .*matches=true
  ```
- `forbidden_lines_of`：前五档都加 `name=raise_rollback_floor `（`second-instance` 今天落在 `*) :`，要新开一支）。
- 设备侧日志判据的 `case` 加 `raise-rollback-floor)` 一支：宿主检查重跑到抬 F 为止，照 `direct` 那样判 `check-exit` = 0、两块盘 `divergence=none`；可再判 `check.txt` 里 `name=host_rerun … windows=…,third_transaction,raise_rollback_floor`。要宿主从盘镜像再读回一次，就把 `disks=(…)` 那一行也给这一档（应读回 `root=2:10 content_matches=true`）。
- 分段登记对账不用改：二进制自己按 `registered_segments_of` 判，这一档九段。
- 预录样本：绿样本 `green/raise-rollback-floor/` 下四件（`out.txt`、`vm-exit`、`check.txt`、`check-exit`，一次真跑的原样输出），`green/expect` 那行「判了 5 档 …」改成 6 档、末尾加 `raise-rollback-floor`。红样本今天那一件（`second-transaction` 盘上写不多于第一个事务）照旧能用；要给新档一个红样本，另四件（例如拿 `second-instance` 的设备日志当这一档的 `check.txt`，应红在 `divergence_window=raise_rollback_floor`）。
- `vm-bench.sh` 不用改（模式参数原样转给二进制，里面没有模式清单，`grep -rln second-instance` 只命中门禁 55、变异表与这三个文件）。

## 七、交回前的验证（都在 `repo/`，线程上限 6，`nice -n 19`）

动到的三个测试二进制（整个二进制，debug），末尾原样：

```
     Running unittests src/lib.rs (target/debug/deps/singlefs_harness-ed0b65334d8d5632)
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.54s
     Running unittests src/bin/first_transaction_device_log_check.rs (target/debug/deps/first_transaction_device_log_check-c087645c5f49f873)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
     Running unittests src/bin/first_transaction_on_device.rs (target/debug/deps/first_transaction_on_device-e8c510422433a824)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
```

`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `check.sh` 那七条 `-D clippy::…`，末尾原样（告警 0 条）：

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.19s
exit=0
```

`cargo build --offline --all-targets`，末尾原样：

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.29s
exit=0
```

`cargo fmt --all -- --check` 退出 1，`Diff in` 只落在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（第 1927、2349、2887、3599 行，别的会话没提交的改动，不是我的）；我那三个文件单独 `rustfmt --edition 2021 --check` 退出 0。

`git apply --check`（主工作区，16:50、16:58）退出 0，无输出。

**新底上重跑（`repo2/` = 16:54 的主工作区 + 补丁）**，末尾原样：

```
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.56s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
```

clippy（同上那套 lint）`Finished … in 3.06s` / `exit=0`、告警 0 条；`cargo build --offline --all-targets` `Finished … in 9.16s` / `exit=0`；三个文件 `rustfmt --check` 无差异。证红 8 条（新 6 行 + 那两行旧的）在 `repo2/` 上重跑，逐条红在与第四节相同的测试、相同的行（`:2347`、`:1965`、`:1970`…），同时红的集合也相同（`proof/rebase-rows-debug.txt`）。抬 F 那四行结果行在新底上逐字不变（`segments=8+2+1+10+2+1+2`、26 次 / 295936 字节）。

## 八、主工作区 `git diff --stat -- crates litmus`（16:58 现跑，原样）

那是别的会话没提交的改动（我的改动只在补丁里，不在主工作区），照定义原样附上：

```
 crates/mutations.tsv                               |  175 +-
 crates/singlefs-checker/src/image.rs               |    8 +-
 crates/singlefs-checker/src/lib.rs                 |   32 +-
 crates/singlefs-checker/src/walk.rs                | 1024 +++++++-
 crates/singlefs-core/src/allocator.rs              |   98 +-
 crates/singlefs-core/src/instance_table.rs         |  135 +-
 crates/singlefs-core/src/journal.rs                |  131 +-
 crates/singlefs-core/src/lib.rs                    |    1 +
 crates/singlefs-core/src/mount.rs                  |  602 ++++-
 crates/singlefs-core/src/mounted_read.rs           |   87 +-
 crates/singlefs-core/src/recovery.rs               |  624 +++--
 crates/singlefs-core/src/transaction.rs            | 2574 ++++++++++++++++----
 crates/singlefs-core/src/write_accounting.rs       |   12 +-
 crates/singlefs-format/src/lib.rs                  |   16 +
 .../src/bin/e158_root_choice_repair.rs             | 2104 +++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |   95 +-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    4 +-
 crates/singlefs-harness/src/history.rs             |   55 +-
 crates/singlefs-harness/src/model.rs               |  107 +-
 crates/singlefs-harness/src/model_comparison.rs    |   50 +-
 crates/singlefs-harness/src/scenario.rs            |    2 -
 .../tests/checker_known_bad_images.rs              |  727 +++++-
 .../tests/first_transaction_step_five_publish.rs   |   42 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 -
 .../second_transaction_mapping_node_admission.rs   |  257 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |   45 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  223 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |  225 +-
 .../tests/second_transaction_step_four_rollback.rs |    8 +-
 .../tests/second_transaction_step_one_overwrite.rs |    1 -
 ...second_transaction_step_three_formatted_pool.rs |   45 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |   10 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ...transaction_supplement_three_fault_injection.rs |   35 +
 ..._transaction_supplement_three_random_history.rs |  182 +-
 ...nsaction_supplement_two_accounting_node_full.rs |  131 +-
 ...two_c533_row_publish_record_without_its_root.rs |    3 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   73 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   45 +-
 ...tion_supplement_two_instance_table_page_full.rs |  278 +--
 ...n_supplement_two_release_checksum_quarantine.rs |  551 +++--
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  416 +++-
 45 files changed, 9313 insertions(+), 1976 deletions(-)
```

## 九、负载

开工（16:29）`ps` 看到：另一个实现员的 `cargo test --release … second_transaction_supplement_two_tree_…`（pid 1082482，在 `/tmp/claude-1000/impl-m2-treesplit/` 自己的 target 里）、别的会话的 `cargo test --release --bin e158_root_choice_repair`（pid 2853009）与 `cargo test --release --bin e142-first-txn-dry-run`（pid 2853162）；没有 `qemu-system`、`vm-bench.sh`、E152、`fio`。各用各的 target，没等文件锁。16:54 重跑前再看：只有别的会话的 `cargo test --release --bin e142-first-txn-dry-run`（pid 1778725），同样没有虚机与性能测量。我的每条命令都是 `nice -n 19 bash research/scripts/capped.sh 6 …`。

## 十、没做什么

- 没走三方对抗；没起虚机、没跑门禁 55 号，也没编 musl 静态版（`x86_64-unknown-linux-musl`，门禁 55 号自己编）——层 0、QEMU、herd7 与 crates 变异整表归 crash-verifier；没提交。
- 没跑全量 `cargo test`、`check.sh`、`gate.sh`，没跑任何层 0 流（这一轮没加层 0 流，也没有新测试落在名字含 layer0 的测试二进制里）。
- 补丁没打进主工作区，6 行变异没追加进主工作区的 `crates/mutations.tsv`（锚点要补丁打上之后才有）。
- 门禁 55 号的 `MODES`、判据与预录样本没改（第六节，交主 agent）；预录样本要一次真跑才有，我给不出。
- F ≠ 0 的抬 F 历史没做（第三节第 1 条）。
- kb 里该跟着改的（我不写 kb）：收口表第 58 行「二进制那一侧与增补 1 验收那句待补 … 实现待派」那一格的状态；第 58 行与实十八报告第三节第 4 条说的「五个模式里没有抬 F」已不成立（六个模式，`raise-rollback-floor`）。
