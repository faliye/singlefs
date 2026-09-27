# 实审 B3a-3 报告（implementation-writer）：按设备记屏障与第三态带红的清单外测试改期望，`crash.rs` 两处文档注释改成现状

时刻都是 UTC（本机时钟），JST = UTC + 9。开工 2026-09-26T22:08Z。

## 一、结论

- 规格点名的 4 份测试文件都把期望改到了新形状，被测代码的行为没动。`crash.rs` 只改了两处文档注释（`check_records_against` 与 `instance_table_of_the_effective_root` 上面那两段）。
- 开工快照副本上的基线：`rollback_floor_…` `8 passed; 3 failed`，`step_one_overwrite` `11 passed; 1 failed`（第 97 行的种类串），`sharded` `5 passed; 5 failed`，floor-raise `0 passed; 1 ignored`。这些与 B3a-2 报告第八节说的一致。
- 改完之后（副本 = 开工快照 + 我这 5 份）：`rollback_floor_…` `11 passed`，`step_one_overwrite` `12 passed`，`sharded` `10 passed`，floor-raise 照旧 `1 ignored`（标 ignore 的全量没跑）。原文见第六节。
- floor-raise 那条崩溃枚举用例改了两处。一是状态数改按层 0 的口径算（原地覆写取三态）：探针在同一条历史上算出，两态闭式 36880、三态 77850，仍在 10⁶ 以内。二是挂载那一遍改为照枚举用的写表叠镜像，原来 zip 录制写表会把撕裂镜像截掉。整条用例没跑，交提交时的崩溃验证员。
- 分片那份的 golden 钉值按今天的代码重取了。取之前拿同一棵树、把 B3a-2 改过的 4 份源码退回它开工时的样子，打一遍对照，两边逐行比过：各种行的词项一样，不同的只有随状态数变的计数与计划哈希。这组钉值原来的意思是「与加分片之前逐字相同」，从这一版起变成「不分片那条路不悄悄变」。要主 agent 认这个口径（第七节第 1 条）。
- 变异：新写 9 行，放在 `mutations-append.tsv`，都证过红；表里已有、原来因为基线红证不了的 14 行（第 18、19、713、714、715、716、718、719、721、852、855、859、860、864 行），这次在改好的副本上复证，全红。prove-red 末行 `✓ 点名 23 条：跑了 23 条，跳过 0 条，跑的都抓到了`；改名的那 1 行另跑了一次，也红。
- 什么现象会推翻这些结论：
  - 主树上这 4 个测试二进制有一个红，或 `--lib` 红；
  - `mutations-append.tsv` 进表后，59 号跑这 9 行有一行不红；
  - floor-raise 那条崩溃枚举在提交时跑出来的 `tally.states` 不是 77850，或挂载那一遍报挂载失败、checker 红；
  - 把 B3a-2 改的 4 份源码退回之后，打出的行与重取前的 golden（`6deeb48f…`）差的不只是计数与计划哈希。

## 二、写过的文件

- `crates/singlefs-harness/src/crash.rs`：只动文档注释，两段（第三节）。
- `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`：种类串一处，外加一条断言消息。
- `crates/singlefs-harness/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`：三条用例的录制流期望，外加它们的文档注释。
- `crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs`：状态数、片数、golden 钉值，外加文档注释。
- `crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`：状态数的算法、挂载那一遍用的写表，外加文档注释。
- `crates/mutations.tsv` 没碰。要追加的 9 行在 `/tmp/claude-1000/impl-rev-b3a3/mutations-append.tsv`（六段、名字在今天的主表里都没有，逐个核过命中 0 次），变异名：
  1. 实审 B3a-3：录制器把连着的屏障不分设备并成一道（抬 F 那一串头六步退回「一道屏障、两次系统配置槽写、一道屏障」）
  2. 实审 B3a-3：录制器把连着的屏障不分设备并成一道（先写系统配置失败时录制流退回一道屏障加盘 0 那一次写）
  3. 实审 B3a-3：录制器把连着的屏障不分设备并成一道（正常卸载那一串头六步退回一道屏障）
  4. 实审 B3a-3：录制器把连着的屏障不分设备并成一道（覆盖写只记盘 0 的屏障，盘 1 的写放行不了，段序列并成 27+2）
  5. 实审 B3a-3：原地覆写的写也只取两态（分片那条小流由环境变量驱动那一趟钉的 54 个状态退回 29）
  6. 实审 B3a-3：原地覆写的写也只取两态（不分片与分 2、3 片再 merge 的状态数退回 29）
  7. 实审 B3a-3：原地覆写的写也只取两态（切法不同的账本报出的片数退回 15 与 29）
  8. 实审 B3a-3：原地覆写的写也只取两态（golden 那一趟留下的进度文件退回 30 行）
  9. 实审 B3a-3：原地覆写的写也只取两态（不分片打印的行与进度文件与补第三态之后重取的钉值不同）
  第 1–4 行的文件、原文、替换文照抄表里已有的「实审 B3a-2 第 1 条：…（mkfs 的操作数退回 21）」，即 `lib.rs` 的 `            .any(|previous| previous.device == operation.device);` → 比 `kind`；第 5–9 行照抄「实审 B3a-2 第 4 条：…（第一条流全量退回 67108885）」，即 `crash.rs` 的 `            Self::NotPersistedTornOrPersisted => 3,` → `2`。这两处原文在今天的主树里各命中 1 次（`grep -c` 都是 1）。追加时请用 `insert-row.py`，每行都带 `--absent` 同名。

我改的行数（对开工快照 `originals/`，`git diff --no-index --numstat`，收尾时量）：crash.rs +13/−6，step_one_overwrite +2/−1，sharded +47/−24，floor-raise +34/−7，rollback_floor +18/−12。

## 三、每条改钉值的用例：改前钉什么、改后钉什么、为什么

原因都一样：录制器按设备记屏障（B3a-2 第 1 条，`SharedStream::push` 只并掉同一块盘在一串连着的屏障里已经记过的那一道），所以每道池屏障在录制流里是两块盘各一步。原地覆写（B3a-2 第 4 条：系统配置槽写，长于一个扇区，罩住的地方原来就有内容）多一态「新旧都读不出」，撕裂镜像接在枚举用的写表后面。

| 用例（文件:行） | 改前钉什么 | 改后钉什么 | 为什么 / 算式 |
|---|---|---|---|
| `second_transaction_step_one_overwrite.rs:99` `overwrite_publishes_the_second_version_through_the_same_commit_shape` | 种类串 `[unit_write×24,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[system_configuration_slot×2]` | `[unit_write×24,barrier×2]\|[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2]`，另加断言消息 | 两道池屏障各多记一步，每道 1 → 2。段序列 `24+2+1+2` 只数写，不变（第 92 行照旧） |
| `rollback_floor_…:243`、`:259`、`:264` `raising_the_floor_writes_the_new_floor_…` | 头 4 步：`(Barrier,0),(SysConf,0),(SysConf,1),(Barrier,0)`；第一条根在第 3 步之后；系统配置槽写取第 1、2 步 | 头 6 步：`(Barrier,0),(Barrier,1),(SysConf,0),(SysConf,1),(Barrier,0),(Barrier,1)`；第一条根在第 5 步之后；系统配置槽写取第 2、3 步 | 两道池屏障各 1 → 2 步。取系统配置槽写那个循环的下标也跟着挪：基线上它排在头 4 步那条断言后面、根本没跑到；不挪的话会去读屏障步的内容，在 `expect("录制流留着内容")` 上 panic |
| `rollback_floor_…:496` `normal_unmount_raises_the_floor_…`（两种覆盖写次数都走这一句） | 头 4 步同上 | 头 6 步同上 | 同上：卸载那一串照抬 F 那一串 |
| `rollback_floor_…:768` `failing_to_write_the_new_floor_into_the_second_system_configuration_…` | 录制流只多 `(Barrier,0),(Write,0)` | 多 `(Barrier,0),(Barrier,1),(Write,0)` | 之前那道池屏障两块盘各一步；盘 1 那次写失败，注入在录制器外面，没录进去；故障计数只数写调用（`FaultSchedule::the_nth_call_across_the_pool`，`fault_injection.rs:482`），所以「第二次写失败」还是盘 1 的系统配置槽写 |
| `sharded…:34` `QUICK_TIER_STATES`（用例 `two_and_three_shards_…`、`the_first_stream_quick_tier_sharded_…` 等都用它） | 29 | 54 | 第一条流 10 段，写数 `2,2,1,2,2,1,26,2,1,2`（副本打印的 `LAYER0_PROGRESS` 按段数出来的状态数 8/3/1/8/3/1/17/3/1/8/1 与之对得上）。四段里各有 2 次系统配置槽写，取三态。甲二：只含两次系统配置槽写的三段各 3² − 1 = 8；A 那一段是 2 次系统配置槽写加 24 次单元写，3² · 2 − 1 = 17；三段 journal 各 2² − 1 = 3；三段根槽写各 1；再加全部持久那一个 1。合计 3·8 + 17 + 3·3 + 3·1 + 1 = 54。两态时是 3·3 + 7 + 9 + 3 + 1 = 29。与 `first_transaction_step_seven_layer0.rs:422` 的 54 相同 |
| `sharded…:350` 「甲二的状态数由展开方式另算一遍」 | `layer0_state_count(writes, segments, quick_tier_expansion)`（两态口径）= 29 | `layer0_state_count_with_torn_in_place_overwrites(base, writes, segments, quick_tier_expansion)` = 54 | 两态那个函数的文档自己写着，枚举出来的状态数是三态那个；不改的话就是拿两态口径去比三态的枚举 |
| `sharded…:527` `merge_stops_when_a_ledger_was_sliced_another_way` | 消息里 `slices 账本写的 "15"、这一趟是 "29"` | `"{⌈54/2⌉ = 27}"`、`"54"`，由 `QUICK_TIER_STATES` 算出来 | 每片 2 个状态切 27 片，每片 1 个切 54 片 |
| `sharded…:805` `print_the_unsharded_enumerations_for_the_golden_comparison` | 进度文件 30 行 | `QUICK_TIER_STATES + 1` = 55 行 | 文件头 1 行，加每片 1 个状态的片行 54 行 |
| `sharded…:824–830` golden 四个常量（`unsharded_enumeration_prints_and_writes_byte_for_byte_…` 用） | 38 行 / `6deeb48f…` / 进度文件名带 `a1d502a7…` / `798bef3c…` | 65 行 / `e4d8027a6b45e3a5557788d41b49b1a40d249ba62411bc0491bb984ab4a05800` / 带 `5026593803c5654d562ab1858215ddd4a5be477116564e882947660016a1eb5e` / `ff0be4e3b94c2619b8e345dd5de6303be9150278b1bed62cdec9447270b08fac` | 行数算式：第一趟片长按线程数定、每片 16 个状态，START 1 行 + PROGRESS ⌈54/16⌉ = 4 行 + FINISHED 1 行 + TALLY 1 行 = 7 行；第二趟 RESUME 1 行 + START 1 行 + PROGRESS 54 行 + FINISHED 1 行 + TALLY 1 行 = 58 行；合计 65 行。改前是 7 + (4 + 29) = 38 行，两趟的 PROGRESS 分别 2 行、29 行。两个 SHA-256 与计划哈希没有算式：取自副本上跑出来的输出。我另用一段自己写的 Python 按用例的遮法遮过、重算了 SHA，两边相同（行数 65，SHA 就是上面那个）。对照与口径的变化见下一段 |
| floor-raise `:115` | `closed_form_state_count(segments)`（两态闭式） | `layer0_state_count_with_torn_in_place_overwrites(base, writes, segments, full_expansion)` | `tally.states` 是三态的计划数。探针在同一条历史上算出（第五节）：11 段，只含两次系统配置槽写的两段（抬 F 先写系统配置那一段、末尾轮换那一段）各 3² − 1 = 8；两段各是 2 次系统配置槽写加 12 次单元写，全量 3² · 2¹² − 1 = 36863；第一次空发布那一段 12 次单元写 2¹² − 1 = 4095；三段 journal 各 3，三段根槽写各 1；再加 1。合计 2·8 + 2·36863 + 4095 + 3·3 + 3·1 + 1 = 77850（两态：2·3 + 2·16383 + 4095 + 9 + 3 + 1 = 36880）。仍在 10⁶ 以内 |
| floor-raise `:139`、`:177`、`:190` | 观察者只收 `persisted`；挂载那一遍拿 `writes_of_the_raise.iter().zip(persisted)` 叠镜像 | 观察者头一次把 `image.writes`（枚举用的写表）收下来，之后每个状态核一次长度相同；挂载那一遍拿这张表 zip `persisted`；另外断言枚举用的写表比录制流长 | `CrashImage` 的文档（`crash.rs:437–439`）写明，观察者拿到的 `writes` 是枚举用的写表，`persisted` 与它逐条对应，比录制流长。原来的 zip 在录制流末尾截断，撕裂那一态叠出来的镜像等于「这次写没持久」，第三态根本没交给挂载 |

分片那份 golden 的对照：同一棵开工快照树，只把 `crash.rs`、`lib.rs`、`segments.rs`、`device_log.rs` 退回 B3a-2 开工时的快照（`/tmp/claude-1000/impl-rev-b3a2/originals/`），在副本 `copy-old` 上跑同一段打印，结果：
- `unsharded_enumeration_…` 在这份上照样红，38 行、SHA `34ff1968a072768027942cb5470bfdddb52f71e97129a0242e3fef038bcda2f3`，不是钉的 `6deeb48f…`。所以旧钉值在今天这棵树上本来就对不上：B3a-2 那四份之外，工作区里还有别的改动也让打出来的行变了（B3a-2 开工前还是开工后的改动，我分不出）。旧钉值是 15:24Z 取的，当时的打印原文没留下；这份测试文件也没进 git，我查不出差在哪一处。这一格如实说清：B3a-3 开工时，旧 golden 已经不是「今天的代码减去 B3a-2」打出来的。重取的钉值只证「今天这一版打的就是这样」，不证与加分片之前逐字相同。
- 退回版与今天版逐项比：每种行的词项名一样（逐行抽出 `key=` 前缀排序再比，`keys identical`）。不同的只有下面几样：随状态数变的计数（`states`、`slices`、`finished_*`、按不变量的 `评估/违例/不适用`、`states_by_publish`、`observed_states`、`observer_counts`、`root_persisted_states` 4 → 9、`file_read` 7 → 12、`no_file` 22 → 42）、第一趟的切片数 2 → 4、进度文件名里的计划哈希。违例、`failed`、`journal_differing` 3、`verification_ran` 6、记录核对器两项都是 0，全没变。这与 B3a-2 报告第七节推的恢复计数一致：根已持久 4 → 9，读出文件 7 → 12。

## 四、`crash.rs` 两处文档注释（B3c-1 报告第六节第 3 条）

- `check_records_against` 上面那段（今天是 `crash.rs:766` 起）：原文写崩溃注入交的是「到当前段为止的整条前缀」，层 0 两张写表是同一张。改成按调用点分三条写现状：
  - 层 0（`check_records`，`crash.rs:1586` 调）交的是枚举用的写表，读盘的口子与被核的记录流是同一张；
  - 崩溃注入第一截（`crash_injection.rs:883`）交整条录制流，持久集合里更早的段整段持久、当前段取子集、更晚的段全没持久；
  - 崩溃注入第三截（`crash_injection.rs:1456` 起，写表是 `:1422` 的 `&mount_writes[..first_write_of_the_segment + writes_in_the_segment]`）只交挂载那一段到当前段为止的前缀。
- `instance_table_of_the_effective_root` 上面那段（今天是 `crash.rs:852` 起）：原文括号里断言「根槽没落盘、由记录重建的那一版也在记录流里」。改成：那次根槽写在不在记录流里，要看调用方交的是哪张写表。层 0 与第一截交整条流，在；第三截交的是前缀，那次根槽写落在更晚的段里时就不在，那时返回 None（「None」那一句补上了这种情形）。
- 我只写现状，没改第三截的交法。B3c-1 第六节第 2 条说第三截有一个缺口：前缀里没有那次根槽写，写行那次发布整个不核。这一条还是交主 agent 定。

## 五、证红与探针

**副本**：`/tmp/claude-1000/impl-rev-b3a3/copy`，22:13:24Z 用 `rsync -a --exclude target --exclude .git` 取的开工快照（那一刻 `crash.rs` 的 sha256 是 `f689915b…`，与 B3c-1 记的同一个），用它自己的 target。我的 5 份改完都拷了进去并 `touch`。

**基线红集**（副本上还没放我的改动，逐个跑整个二进制，`run-with-memory-cap.sh 8G` + `capped.sh 4`，日志 `logs/baseline-*.log`）原样：
```text
second_transaction_step_one_overwrite: test result: FAILED. 11 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.12s
crash_enumeration_sharded_across_processes: test result: FAILED. 5 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.35s
second_transaction_crash_inside_the_floor_raise_pushed_by_the_session: test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
rollback_floor_written_into_the_system_configuration_first_and_normal_unmount: test result: FAILED. 8 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.45s
```
红的正好是这次要改的那 9 条：step_one 1 条、sharded 5 条、rollback 3 条。没有这一轮之外的红。

**prove-red**：变异行先追加进副本的 `crates/mutations.tsv`。命令：`PROVE_RED_LOG_DIRECTORY=…/prove-red-logs nice -n 19 bash research/scripts/capped.sh 4 bash research/scripts/prove-red.sh --copy …/copy --memory 8G singlefs-harness <23 个名字>`，每组参数的基线都先跑过、都绿。末行原样：
```text
✓ 点名 23 条：跑了 23 条，跳过 0 条，跑的都抓到了
```
新 9 行，改坏哪一行、红在哪条断言（原样摘自 `prove-red-logs/001–009.log`）：

| 变异 | 改坏哪一行 | 红在哪条断言 |
|---|---|---|
| 1 抬 F 头六步 | `lib.rs` `.any(\|previous\| previous.device == operation.device);` → `previous.kind == RecordedOperationKind::Barrier` | `rollback_floor_…rs:242` 「抬 F 那一串的头六步…」 `left: [(Barrier, DeviceIdentity(0)), (SystemConfigurationSlot, DeviceIdentity(0)), (SystemConfigurationSlot, DeviceIdentity(1)), (Barrier, DeviceIdentity(0)), (UnitWrite, DeviceIdentity(0)), (UnitWrite, DeviceIdentity(1))]` |
| 2 先写系统配置失败 | 同上 | 「录制流只多写之前那道池屏障…」 `left: [(Barrier, DeviceIdentity(0)), (Write, DeviceIdentity(0))]` `right: [(Barrier, DeviceIdentity(0)), (Barrier, DeviceIdentity(1)), (Write, DeviceIdentity(0))]` |
| 3 正常卸载头六步 | 同上 | 「卸载那一串照抬 F 那一串…」，left 与第 1 行相同 |
| 4 覆盖写段序列 | 同上 | `second_transaction_step_one_overwrite.rs:92` 「覆盖写的段序列与第一个事务同型…」 `left: "27+2"` `right: "24+2+1+2"`（盘 1 的屏障没记，盘 1 的写放行不了，段并在一起）。第 99 行种类串那条断言排在它后面，这个变异走不到它；种类串里 `barrier×2` 只由录制器决定，录制器一变，段序列先红，所以没有单独打它的变异。这一行最早叫「…（覆盖写的种类串退回每段一个 barrier）」，看到红在哪条之后改成现在的名字，第 2–6 段不变，改名之后单独又跑了一次：`✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了`（`prove-red-logs-2/`） |
| 5 selftest 54 | `crash.rs` `Self::NotPersistedTornOrPersisted => 3,` → `2` | `sharded…rs:706` `left: 29` `right: 54` |
| 6 merge 与不分片 | 同上 | `sharded…rs:348` `left: 29` `right: 54` |
| 7 切法不同的账本 | 同上 | merge 在 `crash.rs:3186` 照常 panic，报出文件头不同；用例红在 `sharded…rs:523` 那条 `assert!`（消息里片数不是 27 / 54） |
| 8 golden 进度文件 | 同上 | `sharded…rs:800` 「文件头 + 每片一个状态的 54 行片行」 `left: 30` `right: 55` |
| 9 golden 打印行 | 同上 | `sharded…rs:866` 「子进程失败：ExitStatus(unix_wait_status(25856))」（子进程里第 8 行那条先红） |

prove-red 的参数都带 `-- <过滤>`（与表里已有的行同形），每次只跑过滤到的那几条。每次都是点名的那 1 条红，同一二进制里其余 9–11 条被过滤掉，没跑。红的都是测试自己的断言，不是被测代码里的 `debug_assert`，所以没有在 `--release` 下再跑。

表里已有的 14 行在改好的副本上复证，全红（`prove-red-logs/010–023.log`，末行同上面那一行）：第 18、19 行（step_one 事务号 / 改动计数）；第 713、714、715、716、718、719、721 行（实二那几条 rollback）；第 852、855、859、860、864 行（实分一那几条 sharded）。其中第 859 行就是 B3a-2 第五节说「基线红、证不了，等那份文件改钉值之后复证」的那一行，这次证了。

**floor-raise 探针**（只放进副本，不在仓里）：把那条用例在 `let states =` 之前的前缀抄成 `tests/probe_floor_raise_state_count.rs`，不枚举，只打段、原地覆写数、两态与三态的状态数。源码在 `draft/probe_floor_raise_state_count_2.rs`，驱动脚本在 `draft/probe-loop.sh`，日志在 `logs/probe-floor-raise-2.log`。跑完已经从副本里删掉。探针原样：
```text
PROBE writes=53 segments=11 two_state=36880 three_state=77850 root_writes=3 last_root_write=50
```
逐段：第 0、10 段各 2 次系统配置槽写、原地覆写 2 次；第 4、7 段是 2 次系统配置槽写加 12 次单元写，原地覆写 2 次；第 1 段 12 次单元写；另有三段 journal ×2、三段根槽写 ×1。

探针里另外看到一件事：**用例头上写死的「第 51 次覆盖写落点被拒」在今天的副本上不成立**。探针第一版照抄用例的 50 次覆盖写，第 51 次直接发布照样做成了（`第 51 次：准入放行、落点被拒：Ok(CheckpointTxg(63))`，`logs/probe-floor-raise.log`）。我让探针从 50 起逐次往上试，到 64 次覆盖写之后第 65 次才被拒（`PROBE overwrites=64 direct_publish=PlacementRefused`），上面的数都取自这一格。换句话说，这条标了 ignore 的用例在今天的树上会先红在第 51 次那条 `assert!`，走不到枚举。这不是这一轮带出来的：那一段历史只走核心层的发布与分配器，我这 5 份一份都不在里面；原件的这一段与我改后的逐字相同（`diff` 只差 `let states =` 那一行）。第 51 次这个数要不要跟着分配器改，归主 agent 定（第七节第 2 条）。

## 六、交回前的验证（第 4 步那几样，末尾原样）

开跑前 `ps` 看到：别的会话在跑 `gate.sh`（临时目录里的门禁自测）、`cargo test`（`second_transaction_admis…`、`probe_unit_are…`），没有 qemu、vm-bench、e152、fio。都跑在 `nice -n 19`、`capped.sh 4` 下，跑编出来的代码时经 `run-with-memory-cap.sh 8G`，一次都没撞到包装的 250–254。等锁没单独计时。

- 副本（开工快照 + 我的最终 5 份，与主树 `cmp` 逐字节相同），整个二进制：
  - `second_transaction_step_one_overwrite`：`test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 63.76s`
  - `crash_enumeration_sharded_across_processes`：`test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.11s`
  - `second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`：`test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - `rollback_floor_written_into_the_system_configuration_first_and_normal_unmount`：`test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 75.50s`
  - `--lib`（`crash.rs` 改了注释）：`test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.18s`
- 主工作区上同样 4 个二进制，末尾原样：
  - `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.32s`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 51.14s`
  - `test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.01s`
- `cargo fmt --all -- --check`（主树）：退出 1。`Diff in` 点名的 5 份是 `singlefs-core/src/admission.rs`、`singlefs-core/src/allocator.rs`、`bin/e158_root_choice_repair.rs`、`tests/admission_checkpoint_cost_per_device_paths.rs`、`tests/core_review_unit_area_start_and_publish_limits.rs`，都不是我的。我这 5 份逐个 `rustfmt --check --edition 2021`，退出码都是 0。
- clippy（check.sh 那套：`-D warnings` 加 7 条，`--all-targets --all-features --keep-going`，主树）：退出 101，末尾原样 `error: could not compile `singlefs-core` (lib test) due to 4 previous errors`。报的是 `singlefs-core/src/admission.rs` 与 `allocator.rs`（`manually reimplementing div_ceil`、`.is_multiple_of()`、`doc list item without indentation`），都不是我的。core 编不过，harness 这一趟 clippy 查不到，所以我另跑了只查我动到的目标：同一套 lint，`-p singlefs-harness --no-deps --lib` 加 4 个 `--test`，退出 0，末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.14s`。
- `cargo build --offline --all-targets`（主树）：退出 0，末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 25s`。
- 登记给我的门禁阶段（主树），退出码与判定行原样：
  - 33 号：退出 1，`  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）：`，点名 15 行（308、309、310、319、494、759、865、870、874、877、878、974、975、977、978），指的文件是 `singlefs-checker/src/walk.rs` 11 行、`singlefs-core/src/admission.rs` 4 行，都不是我的。我复证的那 14 行与新写的 9 行都不在里面。
  - 53 号：退出 0，`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
  - 92 号：退出 0，`  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，105 个格式常量里变了 7 个（checker 在同一次改动里跟了 7 个，按滞后表放行 0 个），都不欠 checker 跟进`
  - 94 号：退出 0，`  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 283 行里没有分支与循环（`#[cfg(test)]` 标着的项 263 行不扫）`
  - 93 号：退出 0，`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））`
  - 89 号：退出 77，`  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`。这是本次未跑，不是通过。
  - 74 号：退出 1，`test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 37.92s`。红的两条是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，与 B3a-2、B3c-1 报的是同两条。那个二进制是随机历史，不走我这 5 份：我这边只有 `crash.rs` 两段注释。所以不是这一轮带出来的。这一条是推的，没在改前的副本上对照。
- `research/scripts/crash-case-check.py`：退出 1，点名的是 `crash_enumeration_resumes_from_its_progress_file.rs:383`，不是我的文件。floor-raise 那条已经登记成 `crash-case:floor-raise-pushed-by-the-session`（`stage-inputs.tsv` 第 36 行），名字与计数行都没变，不用重登。

## 七、交主 agent 的问题与发现

1. **分片那份 golden 的口径变了，要你认。** 原来钉的是「与加分片之前逐字相同」。补第三态之后，状态数与计划哈希必然变，那个口径守不住了。我按今天的输出重取，文档里写明了来历与口径的变化（`sharded…rs:813` 起）。另一件：旧钉值 `6deeb48f…` 在这棵树上退回 B3a-2 那四份源码之后也对不上（得 `34ff1968…`），说明 B3a-2 那四份之外还有改动也让打印行变了。这份测试文件没进 git，查不出差在哪。只要这条用例在 B3a-2 之前的某个时刻绿过，那一刻到 B3a-2 开工之间一定还有别的改动动了打印行。要不要去追，你定。
2. **floor-raise 那条崩溃枚举（ignore）在今天的树上大概率先红在第 51 次那条断言，走不到枚举。** 这是推的，依据是探针：照抄同一段前缀，第 51 次直接发布照样做成（`Ok(CheckpointTxg(63))`），到 64 次覆盖写之后第 65 次才被拒。这不是这一轮带出来的，那段历史只走核心层与分配器。要不要把 50 改成 64、要不要先查分配器那边为什么多出 14 次，我没动：用例头上的注释引着 `second_transaction_admission_raises_the_floor_before_refusing.rs` 里同一个第 51 次，改一边另一边也要看，那份文件不在我的清单里。提交时崩溃验证员跑这条会先看到这个红。
3. **step_one 的种类串那条断言没有单独打它的变异。** 录制器一变，段序列（第 92 行）先红。种类串里 `barrier×2` 只由录制器决定，找不到一处改法能让段序列不变而种类串变。这一条只是照实写，不要你定。
4. **没有停在「条款没写」的分支上。** 这一轮只改期望与注释，没加错误成员、`todo!`、`assert!`。floor-raise 里加的是测试自己的断言：写表长度逐状态相同；枚举用的写表比录制流长。
5. **kb**：这一轮没有新的要改的句子。这份覆盖写的种类串，kb 里没登记（`grep 'unit_write×24,barrier\]'` 只命中 E142 实验页第 93 行与 `layout/01-first-txn.md:403`，两处都是第一个事务、跟 E142 产物走，B3a-2 第九节已经列过）。

## 八、受影响的层 0 流与崩溃枚举用例

checker 没动，这一节按定义不适用。崩溃枚举用例里被我改了文件的只有 `crash-case:floor-raise-pushed-by-the-session`，钉值从两态闭式换成三态口径：这条流上 36880 → 77850。挂载那一遍也多了撕裂镜像。层 0 那 6 份不归我。

## 九、`git diff --stat -- crates litmus`（主树，原样末行；全份在 `logs/git-diff-stat.txt`）

别的会话同时在改 `crates/`，这份 stat 分不出谁改的，我写过的以第二节为准。`rollback_floor_…`、`floor_raise_…`、`sharded…` 三份没进 git（`??`），不在 stat 里。
```text
 88 files changed, 29839 insertions(+), 11188 deletions(-)
```
我相关的两行（含开工前别人已有的改动）：`crates/singlefs-harness/src/crash.rs               | 2505 ++++++-`、`.../tests/second_transaction_step_one_overwrite.rs |    3 +-`。

## 十、草稿目录里删了什么、留了什么

- 删了：仓副本 `/tmp/claude-1000/impl-rev-b3a3/copy`（删前 `du -sh` 2.7G，含它自己的 target）；对照副本 `/tmp/claude-1000/impl-rev-b3a3/copy-old`（删前 1.7G）。
- 留着的，主 agent 要核或要打：`report.md`、`mutations-append.tsv`、`logs/`、`prove-red-logs/`、`prove-red-logs-2/`、`golden-old-1/` 与 `golden-new-1/`（两版 golden 的进度文件）、`draft/`（探针源码、遮行脚本、两版遮过的打印行 `golden-old-today-masked.txt` / `golden-new-masked.txt`）、`originals/`（开工快照）、`progress.md`、`sha-at-start.txt`、`copy-snapshot-sha.txt`、`spec.md`。都不是编译目录，也不是仓副本；整个草稿目录现在 1.1M。

## 十一、没做什么

- 没走三方对抗，没提交；层 0、QEMU、herd7、crates 变异表整表都归崩溃验证员。floor-raise 那条标 ignore 的全量没跑，名字带 layer0 的目标没跑，层 0 那 6 份没碰。
- `crates/mutations.tsv`、`stage-inputs.tsv`、kb 都没碰；B3c-1 说的第三截交法没改（第四节）。
- floor-raise 用例里第 51 次那个数没改（第七节第 2 条）。
- 全仓 clippy、fmt 被别人的文件挡住，我只单独查了自己动到的目标。74 号那两条红的来历，只推了，没在改前副本上对照。
