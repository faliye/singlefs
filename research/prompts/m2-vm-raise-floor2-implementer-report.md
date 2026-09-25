# 实现员报告：impl-m2-vm-raise-floor2（实二四续：抬 F 模式让 F 真的抬过 0；增补 2 收口表第 58 行）

时刻一律 UTC（东京 = UTC+9）。仓副本 `repo/`（17:03:37 从主工作区 rsync，HEAD `e980a21`；那三个文件与 `crates/mutations.tsv` 的原件快照在 `base/`，不改动的对照副本 `pristine/`）。
主工作区一个字没动：改动只在补丁里，变异表的改动只在两份 tsv 里。17:23:28 核过：主工作区那三个文件与 `crates/mutations.tsv` 仍与 `base/` 逐字节相同，补丁 `git apply --check` 退出 0。

## 一、交付物

| 样 | 路径 | sha256 |
|---|---|---|
| 补丁（只含 3 个文件） | `/tmp/claude-1000/impl-m2-vm-raise-floor2/impl-m2-vm-raise-floor2.patch` | `6156395a8f4886fed5fdf7f894d7a012ee26b4c437854db879eee8bfc8bf4ba1` |
| 追加变异行（6 行整行） | `/tmp/claude-1000/impl-m2-vm-raise-floor2/mutations-append.tsv` | `fcc8a3a3491938913ae29582fcf636e45eac743534f3967ac205e44ed2842c4c` |
| 接替行（2 行整行，按变异名替换原行） | `/tmp/claude-1000/impl-m2-vm-raise-floor2/mutations-replacement-rows.tsv` | `611c0ebe79f0a0c22707e31bbc071a6b96e243071830b6b834aba03fa06f54da` |
| 报告 | `/tmp/claude-1000/impl-m2-vm-raise-floor2/report.md` | 交回里给 |

补丁在主工作区上 `git apply --stat`，原样：

```
 crates/singlefs-harness/src/on_device_modes.rs     |  111 ++++-
 .../src/bin/first_transaction_on_device.rs         |  473 +++++++++++++++-----
 .../src/bin/first_transaction_device_log_check.rs  |   90 ++--
 3 files changed, 492 insertions(+), 182 deletions(-)
```

这一轮写过的文件（都在草稿目录 `/tmp/claude-1000/impl-m2-vm-raise-floor2/`）：`repo/` 下上面那 3 个文件与 `repo/crates/mutations.tsv`（只在副本里，照第六节删 1、换 2、追加 6，供门禁 33 号在副本上数锚点）；补丁、两份 tsv；`new-raise-tests.rs`（拼进用例模块的草稿）；`verify.sh`、`stages.sh`、`proof/prove.py`、`proof/rows.tsv`、`proof/row-*.log`、`proof/summary.txt`；`lines-probe/`（加了打印行的探针副本，取第二节那些结果行用）；各 `*.log`；本报告。`crates/mutations.tsv` 在主工作区一行没动。

## 二、改了什么、F 抬到多少（行号是「主工作区 + 补丁」的）

**历史**：`raise-rollback-floor` = `second-instance` 那条路（A txg 3、B txg 4、冷重开可写挂载：写行 txg 5、暖机 6、7、发布 C txg 8），之后同一次挂载里**发布 D（txg 9，第四版）**，再**把 F 抬到上限**，推两次空发布 txg 10、11。

- `on_device_modes.rs`：`PublishesAfterTheFirstTransaction` 那一成员改名 `SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor`（`:46`，含义换了就换名字）；`FOURTH_VERSION_WRITE_TIME_SECONDS`（`:129`，+180 秒）、`fourth_file_content()`（`:149`，1800 字节）、`publish_the_fourth_version`（`:231`，接在发布 C 那一版上、实例照发布 C 的）；`raise_the_rollback_floor_after_the_third_version` 换成 `raise_the_rollback_floor_to_its_ceiling`（`:261`）：先用 core 的 `rollback_floor_ceiling` 现算上限（入参与 `raise_rollback_floor` 自己算上限时相同：`choose_system_configuration`、现行那一版的根指着的实例表、现行的 F），再 `raise_rollback_floor(…, ceiling, ShadowLedger::On)`。两次算之间没有写；对不上时 core 在任何写之前拒（`RollbackFloorAboveCeiling`）。`last_published_content` 这一档读回第四版。
- `first_transaction_on_device.rs`：分段登记表这一档多 `fourth_transaction`（`:102`，十段）；发布 C 与发布 D 共用一个新函数 `publish_an_overwrite_in_the_mounted_instance_and_describe`（`:915`，发布一行、`name=publish_writes` 一行、窗口行一行、失败时失败账那几行；发布 C 的结果行逐字不变，由 `second_instance_mode_…` 与 `third_version_publish_failing_…` 两条旧用例钉着）；`publish_the_fourth_version_and_describe`（`:990`）；`publish_the_fourth_version_and_raise_the_rollback_floor`（`:1041`，发布 D 再抬 F，`main` 与用例走同一个函数）；抬 F 那一段（`:1066`）改调 `raise_the_rollback_floor_to_its_ceiling`。
- `first_transaction_device_log_check.rs`：`ProgramWindow::FourthTransaction`（`:55`，段名 `fourth_transaction`）；宿主重跑这一档在发布 C 之后 `publish_the_fourth_version`、记这一段终点（`:177`），再 `raise_the_rollback_floor_to_its_ceiling`、记抬 F 那一段终点。

**F 与上限（D16（发布语义） 已定项 1「抬 F 的上限」：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)）**：根环区域 = txg mod 3（`crates/singlefs-core/src/root_ring.rs:114`），区域设备归属 0 / 1 / 0，所以发布 D 之后盘 0 上最新的是 txg 9、盘 1 上是 txg 7，前一项 = 7；按新到旧数非空有效根是 D、C、B、A（txg 9、8、4、3），第 4 新的是 txg 3。上限 = min(7, 3) = **3**，F 抬到 3。实现现算出来的正是这个数：`ceiling=3 requested_floor=3`，末次空发布的根带 F = 3（用例 `:2456` 钉 `CheckpointTxg(3)`）。没有发布 D 时同一条式子给 0（第五节变异 ① 的日志：`requested_floor=0 ceiling=0 … root_txgs=9,10`）。

**冷重开**：择 **(2, 11)**，读回第四版（用例 `:2415` 那条末尾钉 `(InstanceGeneration(2), CheckpointTxg(11))` 与 `fourth_file_content()`）。宿主从盘镜像再读回一次时应是 `name=host_recover outcome=file_read root=2:11 content_matches=true`（没跑，要虚机产的盘镜像）。

**宿主上跑这个模式的结果行**（探针副本 `lines-probe/` 在用例里把 `raised.lines` 逐行打出来，`nanoseconds` 换成 N，`publish_writes` 两类行截到按种类的前几项）：

```
name=fourth_transaction root_txg=9 transaction=2 released=8 nanoseconds=N operations=23 segments=16+2+1+2 closed_form=65543 device_0_writes=11 device_0_written_bytes=172544 device_0_force_unit_access_writes=1 device_0_barriers=2 device_1_writes=10 device_1_written_bytes=172032 device_1_force_unit_access_writes=0 device_1_barriers=2
name=publish_writes publish=fourth_transaction txg=9 write_calls=21 written_bytes=344576 data_unit_write_calls=2 …
name=publish_writes_against_device window=fourth_transaction publishes=1 by_kind_write_calls=21 by_kind_written_bytes=344576 device_write_calls=21 device_written_bytes=344576 matches=true
name=raise_rollback_floor floor_before=0 requested_floor=3 ceiling=3 publishes=2 root_txgs=10,11 reclaimed=1 abandoned_roots_unreadable=0 nanoseconds=N operations=30 segments=8+2+1+10+2+1+2 closed_form=1290 device_0_writes=13 device_0_written_bytes=147968 device_0_force_unit_access_writes=1 device_0_barriers=4 device_1_writes=13 device_1_written_bytes=147968 device_1_force_unit_access_writes=1 device_1_barriers=4
name=publish_writes publish=raise_rollback_floor txg=10 write_calls=13 written_bytes=147968 …
name=publish_writes publish=raise_rollback_floor txg=11 write_calls=13 written_bytes=147968 …
name=publish_writes_against_device window=raise_rollback_floor publishes=2 by_kind_write_calls=26 by_kind_written_bytes=295936 device_write_calls=26 device_written_bytes=295936 matches=true
```

宿主检查（同一个探针副本，`every_mode_reruns_…` 里打的）：

```
name=host_rerun mode=raise-rollback-floor windows=mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor operations_by_window=21+3+13+23+23+50+23+23+30
RaiseRollbackFloor name=device_log device=0 declared_entries=NA expected_writes=94 expected_flushes=34 observed_writes=94 observed_flushes=35 zero_fills=4 zero_fill_bytes=805404672 trailing_flushes=1 expected_events_by_window=mkfs:14,instance_acquisition:2,warm_up:9,first_transaction:14,second_transaction:12,reopen_and_writable_mount:31,third_transaction:14,fourth_transaction:14,raise_rollback_floor:18 divergence_window=none divergence=none
RaiseRollbackFloor name=device_log device=1 declared_entries=NA expected_writes=90 expected_flushes=30 observed_writes=90 observed_flushes=31 zero_fills=4 zero_fill_bytes=805404672 trailing_flushes=1 expected_events_by_window=mkfs:12,instance_acquisition:2,warm_up:9,first_transaction:12,second_transaction:14,reopen_and_writable_mount:29,third_transaction:12,fourth_transaction:12,raise_rollback_floor:18 divergence_window=none divergence=none
```

（`device_log` 两行里的「设备侧日志」是用例按程序事件合成的，不是虚机录的；行首的 `RaiseRollbackFloor` 是探针加的前缀。）

**三条验收**：
- 验收一：宿主上跑这个模式，`name=raise_rollback_floor` 那一行 `requested_floor=3 ceiling=3`（F 从 0 抬到 3），窗口行 `window=raise_rollback_floor … matches=true`；发布 D 那一段 `window=fourth_transaction … matches=true`。用例 `raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count`（`first_transaction_on_device.rs:2415`）钉着这些数。
- 验收二：宿主检查判这一段相等：`raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed`（`first_transaction_device_log_check.rs`）在发布 D 与抬 F 两段里各改一步（少一个写、少一个 FLUSH、最后一个写的内容变了），两块盘都红、红在被改的那一段；不改的判相等；`second-instance` 的盘拿这一档去比，红在 `fourth_transaction`。
- 验收三：去掉那次覆盖写（变异 ①，第五节），用例红在 `:2446` 的 ceiling 断言上（left `Some("0")`、right `Some("3")`）。

推翻条件：虚机里真跑 `raise-rollback-floor`，结果行不是 `requested_floor=3 ceiling=3 … root_txgs=10,11`，或任一窗口行 `matches=false`，或宿主检查在 `fourth_transaction` / `raise_rollback_floor` 段判红，或冷重开不是 `2:11`、读回的不是第四版。

## 三、门禁 55 号要改的新行（交主 agent 改；我没碰 `.claude/gate.d/`）

主工作区的 55 号今天 `MODES` 仍是五档（17:24 现读 `.claude/gate.d/55-qemu-first-transaction.sh:31`），上一轮报告给的 `raise-rollback-floor` 那几行还没进去。下面是这一版（发布 D + 抬 F 到上限）该进去的，覆盖上一轮报告第六节给的那几条：

- `MODES` 加 `raise-rollback-floor`（六档）。
- `cold_root_of`：`raise-rollback-floor) printf '2:11' ;;`
- `required_lines_of`：发布 B 那四条的 case 改成 `second-transaction|second-instance|raise-rollback-floor)`；`second-instance` 那九条的 case 改成 `second-instance|raise-rollback-floor)`；再加这一档独有的（宿主上跑出来的值，几何 `e142_parameters(512, 512)`，与虚机相同）：
  ```
  name=fourth_transaction root_txg=9 transaction=2 released=
  name=fourth_transaction .*segments=16+2+1+2 closed_form=
  name=publish_writes publish=fourth_transaction txg=9 write_calls=
  name=publish_writes_against_device window=fourth_transaction publishes=1 .*matches=true
  name=raise_rollback_floor floor_before=0 requested_floor=3 ceiling=3 publishes=2 root_txgs=10,11 reclaimed=1 abandoned_roots_unreadable=0 nanoseconds=
  name=raise_rollback_floor .*segments=8+2+1+10+2+1+2 closed_form=
  name=publish_writes publish=raise_rollback_floor txg=10 write_calls=
  name=publish_writes publish=raise_rollback_floor txg=11 write_calls=
  name=publish_writes_against_device window=raise_rollback_floor publishes=2 .*matches=true
  ```
- `forbidden_lines_of`：前三档与 `second-transaction` 各加 `name=fourth_transaction ` 与 `name=raise_rollback_floor `；`second-instance` 今天落在 `*) :`，要新开一支 `second-instance) printf '%s\n' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;`。
- 设备侧日志那一段给这一档一支：宿主检查重跑到抬 F 为止，照 `direct` 判 `check-exit` = 0、两块盘 `divergence=none`；可再判 `check.txt` 里 `name=host_rerun mode=raise-rollback-floor windows=mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor operations_by_window=21+3+13+23+23+50+23+23+30`。给盘镜像的话应读回 `name=host_recover outcome=file_read root=2:11 content_matches=true`。
- 分段登记对账不用改：二进制自己按 `registered_segments_of` 判，这一档十段（比 `second-instance` 多 `fourth_transaction`、`raise_rollback_floor`）。
- 预录样本：绿样本 `green/raise-rollback-floor/` 四件要一次真跑的原样输出，我给不出；`green/expect` 的档数随之改成 6 档。
- 文件头注释「五次虚机跑」那一段要加这一档的一句（发布 D 之后 F 抬到 3，冷重开择 (2, 11) 读回第四版）。

## 四、取舍与交主 agent 定的

1. **新 F 取的是「抬 F 的上限」，不是「一次处置的目标」**。D16（发布语义） 已定项 1 那张表两句都有：「抬 F 的上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)」「一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)」。后一句要一个「这次释放」，只在准入不够、产品路径触发抬 F 时有；`raise_rollback_floor` 的文档注释写着今天只有测试入口、没有产品触发（`crates/singlefs-core/src/mount.rs:887` 起那段）。测试档里没有「这次释放」，我按前一句现算、抬到上限。这段历史上两句给的数相同的前提是「那次释放的释放代 ≥ 3」（发布 D 释放 C 的单元，释放代按预想是 9；**这个数我没从实现里读出来核**）。推翻条件：主 agent 要按「一次处置的目标」取，且那个释放代 < 3。
2. **`reclaimed=1` 是实现现算的数，我没有独立推一遍是哪一个落点**。旁证只有一条：同一段历史上抬到 F = 0 时是 `reclaimed=0`（第五节变异 ② 的日志：`requested_floor=0 ceiling=3 … reclaimed=0`），抬到 3 多回收 1 个（两盘同槽按盘 0 报）。门禁 55 号那一行把 `reclaimed=1` 写死了；不想写死它就把那条正则截在 `root_txgs=10,11 `。
3. **`raise_the_rollback_floor_to_its_ceiling` 在 harness 里把 core 抬 F 前的三步（选系统配置、读实例表、算上限）重做了一遍**，因为 core 没有「按现行那一版算上限」的公开入口，而 `mount.rs` 在别人手里。core 以后给这个入口，这里换成调它。补丁依赖 `rollback_floor_ceiling(&Vec<…>, &SystemConfiguration, CheckpointTxg, &InstanceTableRecords)`、`recovery::choose_system_configuration`、`recovery::instance_table_chain_of_root(…).records` 的今天的签名；实二二三改了这几样会编不过，不会静默漏。
4. **发布 D 单独成一段**（`fourth_transaction`：结果行、窗口行、分段计时、宿主检查的一段），不并进抬 F 那一段：并进去的话抬 F 那一行的 `publishes`、`segments` 与那一串空发布的账就混了覆盖写。门禁 55 号因此多四条 `fourth_transaction` 行。
5. 条款没写、要设计判断而停下的：**无**（这一轮只动 harness，没有盘上行为的分支）。没写 `todo!` / `assert!`；新加的 `expect` 只在用例里。没加 trait 实现、derive、访问器。

## 五、每条新测试「改坏哪一行 → 哪条断言红」

基线（`pristine/`，17:04 起跑，debug）：三个测试二进制全绿，**基线红集为空**——

```
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.90s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
```

证红办法：`proof/prove.py` 每条变异一份新副本（从 `repo/` 连 target 一起 rsync，各用各的 target），改坏一处、对被改文件 touch、跑那条测试所在的**整个**测试二进制（`--lib` 或 `--bin <名>`，不按名字挑），记红了哪些，跑完删副本、不拷回。17:14 前后起跑、17:19:10 跑完，14 条全部在 debug 下红在点名那条测试的断言上（没有一条红在 `debug_assert` 或编译错上），没跑 release。证红跑在补丁的前一版上，与最终补丁只差 6 处文档注释（`on_device_modes.rs` 因此多 2 行）；下表行号已换算成「主工作区 + 最终补丁」的。明细 `proof/summary.txt`，日志 `proof/row-NN.log`。

| # | 变异（改坏哪一行） | 红的测试 → 断言 | 同时红的 |
|---|---|---|---|
| ① | 新：`first_transaction_on_device.rs:1048`–`:1054` 发布 D 那一步换成 `let fourth_version_run = third_version_run;`（去掉那次覆盖写） | `raise_rollback_floor_mode_publishes_the_fourth_version_…` → `:2446` ceiling（left `Some("0")` right `Some("3")`；那一行 `requested_floor=0 ceiling=0 … root_txgs=9,10`） | `fourth_version_publish_failing_midway_…` |
| ② | 新：`on_device_modes.rs:279` 调 `raise_rollback_floor` 之前把 `ceiling` 遮成现行的 F | 同上 → `:2451` requested_floor（left `Some("0")`；那一行 `ceiling=3 … reclaimed=0`） | 无 |
| ③ | 新：`first_transaction_on_device.rs:1017` 发布 D 失败那一支删掉 `lines.extend(failed.lines);` | `fourth_version_publish_failing_midway_…` → `:2102`「失败账一份」 | 无 |
| ④ | 新：`on_device_modes.rs:119` 这一档读回改成 `third_file_content()` | `the_four_versions_differ_…` → `:373`「抬 F 的空发布不碰 inode 树，冷重开读回的是发布 D 那一版」 | 无 |
| ⑤ | 新：`first_transaction_device_log_check.rs:170`–`:176` 宿主不重跑发布 D（换成 `second_instance.current_version.clone()`） | `every_mode_reruns_…` → `:608`「RaiseRollbackFloor 盘 0 的 fourth_transaction 一件事都没投」 | `raise_rollback_floor_mode_compares_…` |
| ⑥ | 新：`first_transaction_device_log_check.rs:177` 发布 D 那一段的终点记成 `ThirdTransaction` | `raise_rollback_floor_mode_compares_…` → `:861` `.expect("raise-rollback-floor 跑了发布 D 与抬 F 两段")` | `every_mode_reruns_…` |
| ⑦ | 接替第 355 行（同一处变异）：`on_device_modes.rs:116` `second-instance` 读回改成第二版 | `the_four_versions_differ_…` → `:369`（`SecondInstance` 那一条） | 无 |
| ⑧ | 接替第 617 行（同一处变异）：`first_transaction_on_device.rs:1173` `raise_publishes.push(…)` → `.clear()` | `raise_rollback_floor_mode_publishes_the_fourth_version_…` → `:2512` 两段窗口一览（抬 F 那一段 `publishes=0 by_kind_write_calls=0 … device_write_calls=26`） | 无 |
| ⑨ | 重证第 357 行：`on_device_modes.rs:317` 失败账换成 `Vec::new()`（发布 C 的失败路径改走共用函数） | `third_version_publish_failing_midway_…` → `:2102`「失败账一份」 | `second_version_…`、`fourth_version_…` |
| ⑩ | 重证第 494 行：`mount.rs:1002` 起抬 F 失败时已落盘账换成 `Vec::new()`（用例前面多了发布 D） | `failed_raise_of_the_rollback_floor_…` → `:2097`「同一段里失败之前已经落盘的发布各一行」 | 无 |
| ⑪ | 重证第 496 行：`mount.rs:1578` 失败账换成 `Vec::new()` | `failed_raise_of_the_rollback_floor_…` → `:2102`「失败账一份」 | `failed_writable_mount_…` |
| ⑫ | 重证第 618 行：`first_transaction_on_device.rs:1107` 抬 F 失败时已落盘账换成 `&[]` | `failed_raise_of_the_rollback_floor_…` → `:2097` | 无 |
| ⑬ | 重证第 619 行：`first_transaction_on_device.rs:103` 分段表删掉 `"raise_rollback_floor",` | `every_mode_registers_its_own_segments_with_no_repeats` → `:1968`「…多发布 D 与抬 F 两段…」 | 无 |
| ⑭ | 重证第 620 行：`first_transaction_device_log_check.rs:185` 抬 F 那一段的终点记成 `ThirdTransaction` | `raise_rollback_floor_mode_compares_…` → `:861` | `every_mode_reruns_…` |

（`mount.rs` 的行号是主工作区 17:25 现取的，那份文件归实二二三，不在补丁里；⑩⑪ 证红用的是 17:03 的快照。）

新测试与扩了的测试各由哪几行证过：`raise_rollback_floor_mode_publishes_the_fourth_version_…`（改名重写）由 ①②⑧；`fourth_version_publish_failing_midway_…`（新）由 ③；`the_four_versions_differ_…`（改名扩断言）由 ④⑦；宿主检查两条（扩断言）由 ⑤⑥⑭；`failed_raise_…`（前置多了发布 D）由 ⑩⑪⑫；`every_mode_registers_…`（扩断言）由 ⑬。**没有留给 59 号的行**：追加的 6 行、接替的 2 行都证过。

## 六、变异表（主工作区 `crates/mutations.tsv` 要做的三件，先打补丁再改表）

**删 1 行**（锚点随补丁消失）：
- 「增补 2 收口表第 58 行（真设备抬 F 模式）：raise-rollback-floor 冷重开该读回的取成第二版」（今天第 616 行），由追加的 ④ 接替。

**换 2 行**（变异本身不变，只有第六段「必须红的测试名」跟着用例改名；整行在 `mutations-replacement-rows.tsv`，按变异名替换原行）：
- 「增补 2 第 30 行：second-instance 冷重开该读回的取成第二版」（今天第 355 行）：必须红的测试 `the_three_versions_differ_…` → `the_four_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one`。
- 「增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制抬 F 窗口按种类那一侧不算那一串空发布」（今天第 617 行）：`raise_rollback_floor_mode_pushes_an_empty_publish_onto_each_device_and_its_window_matches_the_device_layer_count` → `raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count`（第五段 `-- raise_rollback_floor_mode` 照旧命中新名字）。

**追加 6 行**（`mutations-append.tsv`，整行加在末尾）：
1. 增补 2 收口表第 58 行（真设备抬 F 模式）：raise-rollback-floor 发布 C 之后不再覆盖写一次（第 4 新的非空有效根够不到 A，F 的上限退回 0）
2. 增补 2 收口表第 58 行（真设备抬 F 模式）：抬 F 抬到现行的 F、不抬到 core 现算的上限
3. 增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制发布 D 失败时丢掉失败账那几行
4. 增补 2 收口表第 58 行（真设备抬 F 模式）：raise-rollback-floor 冷重开该读回的取成第三版（没算发布 D；接替原「冷重开该读回的取成第二版」那一行）
5. 增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查不重跑发布 D（抬 F 接在发布 C 上，发布 D 那一段空着）
6. 增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查把发布 D 那一段的终点记成发布 C（发布 D 那一段不单独比、红了也指不到它）

锚点核对：主工作区表里文件落在这三个文件上的行、加上追加与接替的 8 行，在「主工作区 + 补丁」上逐行数（脚本输出 `checked 28 bad 1`，那 1 行就是要删的第 616 行）；三件都做完的整表放在副本 `repo/crates/mutations.tsv`，门禁 33 号从副本根跑，末行原样：

```
  ✓ 147 个实验二进制都有成形的变异表，1614 条变异的原文各命中源码一次；crates/mutations.tsv 621 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
```
退出码 0。第 494、496 行的锚点在 `mount.rs`（实二二三在改），以上核的是 17:03 的快照。

## 七、交回前的验证（都在 `repo/`，线程上限 5，`nice -n 19`，17:22–17:23 跑最终那一版）

动到的三个测试二进制（整个二进制，debug），末尾原样：

```
     Running unittests src/lib.rs (target/debug/deps/singlefs_harness-ed0b65334d8d5632)
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.60s
     Running unittests src/bin/first_transaction_device_log_check.rs (target/debug/deps/first_transaction_device_log_check-c087645c5f49f873)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
     Running unittests src/bin/first_transaction_on_device.rs (target/debug/deps/first_transaction_on_device-e8c510422433a824)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
tests_exit=0
```

`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `check.sh` 那七条 `-D clippy::…`，告警与错误 0 条，末尾原样（最终这一版只重查了 harness；17:13 那一次从零查了四个 crate，同样 0 条、退出 0）：

```
    Checking singlefs-harness v0.1.0 (/tmp/claude-1000/impl-m2-vm-raise-floor2/repo/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.37s
clippy_exit=0
```

`cargo build --offline --all-targets`，末尾原样：

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.35s
build_exit=0
```

`cargo fmt --all -- --check` 退出 1，`Diff in` 只落在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（第 1927、2370、3628、5988、6039 行，E158 执行员没提交的改动，不是我的）；我那三个文件 `rustfmt --edition 2021 --check` 退出 0。

`git apply --check`（主工作区，17:23:28）退出 0，无输出。

登记给实现员的门禁阶段（`stage-owners.tsv`），拿副本 `repo/` 当被判目录跑，末行与退出码原样：
- 33 号：见第六节，退出 0（带参数跑时它按相对路径找 `research/e7-index-bench/src/bin`、退 77，从副本根重跑才判到）。
- 53 号：`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`，退出 0。
- 74 号：退出 0，末三行是三个取样点的对拍数（「小盘上逼近单元区墙的取样点：模型对拍 4626 步：该拒而拒 628、区间里拒 2、该成而成 3996；…」）。这一道与补丁无关，照登记跑了。
- 92 号：`  ! /tmp/claude-1000/impl-m2-vm-raise-floor2/repo 不是 git 仓，本阶段跳过`，退出 77（本次未跑）。
- 94 号：`  ✓ checker 与实现只共享常量模块 `singlefs-format`…`，退出 0。
- 93 号：`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致…`，退出 0。
- 89 号：`  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`，退出 77。

## 八、主工作区 `git diff --stat -- crates litmus`（17:24 现跑，原样）

那是别的会话没提交的改动与上一轮已经打进来的补丁（我的改动只在补丁里，不在主工作区），照定义原样附上：

```
 crates/mutations.tsv                               |  182 +-
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
 .../src/bin/e158_root_choice_repair.rs             | 2103 +++++++++++++++-
 .../src/bin/first_transaction_device_log_check.rs  |  233 +-
 .../src/bin/first_transaction_on_device.rs         |  534 +++-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    4 +-
 crates/singlefs-harness/src/history.rs             |   55 +-
 crates/singlefs-harness/src/model.rs               |  107 +-
 crates/singlefs-harness/src/model_comparison.rs    |   50 +-
 crates/singlefs-harness/src/on_device_modes.rs     |   62 +-
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
 47 files changed, 9984 insertions(+), 2045 deletions(-)
```

## 九、负载

开工（17:03）`ps` 看到别的会话的 `cargo test --release --bin e142-first-txn-dry-run`（pid 2653009）；17:10 再看：`cargo test --release -p singlefs-harness --test opus_r1_attack z6_a2`（pid 2656491）、三条 E158 的 `cargo run --release … e158_root_choice_repair`（在 `/tmp/claude-1000/e158-s9/arms/…` 各自的目录里）、`cargo test --release --bin e142-first-txn-dry-run`（pid 2944188）；17:22 前再看，没有 `qemu-system`、`vm-bench.sh`、E152、`fio`。各用各的 target，没等文件锁。我的每条编译、测试命令都是 `nice -n 19 bash research/scripts/capped.sh 5 …`，证红的 14 份副本一次只跑一份。

## 十、没做什么

- 没走三方对抗；没起虚机、没跑门禁 55 号、没编 musl 静态版；层 0、QEMU、herd7 与 crates 变异整表归 crash-verifier；没提交。
- 没跑全量 `cargo test`、`check.sh`、`gate.sh`，没跑任何层 0 流（这一轮没加层 0 流，没有新测试落在名字含 layer0 的测试二进制里）。
- 补丁没打进主工作区，变异表的删 1、换 2、追加 6 没在主工作区做。
- 门禁 55 号的 `MODES`、判据与预录样本没改（第三节，交主 agent）；预录样本要一次真跑才有。
- `reclaimed=1` 回收的是哪一个落点、发布 D 那次释放的释放代是多少，没从实现里读出来核（第四节第 1、2 条）。
- `recover_from_disk_images`（宿主从盘镜像读回）这一档要读回 `2:11` 第四版，只由 `last_published_content` 的用例与二进制的冷重开用例间接钉着，宿主检查那一支没有用例（要真盘镜像）。
- kb 里该跟着改的（我不写 kb）：收口表第 58 行那一格与上一轮报告第三节第 1 条说的「F 在这一档里没有动（0 → 0）」已不成立（这一版 F 0 → 3，冷重开 `2:11` 读回第四版）。
