# 实现员报告：C554 乙-配置续（取号那一写的 tail 写见证值）

写于 2026-09-27（JST）。规格 `/tmp/claude-1000/impl-c554-yi-carry/spec.md`。补丁目录 `/tmp/claude-1000/impl-c554-yi-carry/patch/`（`crates.patch`、`mutations-append.tsv`、`mutations-replacements.tsv`、`report.md` = 这份的拷贝）。

## 一、结论

- `crates/singlefs-core/src/transaction.rs`：取号那一写（`write_acquired_instance`）不再写 tail 0，写取号那一刻按 N-配置 同一取法读到的见证值；取号失败的回卷写（`roll_back_acquisition`）带同一个值。新函数 `highest_system_configuration_journal_tail`（每块盘两槽里全部自证过、fsid 与本池相同的系统配置槽 `journal_tail` 取最大，一份都没有时 0，命名常量 `JOURNAL_TAIL_WITNESSED_BY_NO_SYSTEM_CONFIGURATION`），在第一道屏障之后、第一个取号写之前调一次，逐盘取号写与回卷写都用这一次读到的。`recovery.rs`、`mount.rs` 不改。
- mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：第一个事务的字节不变（新测试钉住）。
- 新测试文件 6 条，全绿；6 行变异逐条用 `prove-red.sh` 证红（在打 A3a 之前的副本上证过一遍，又在「主工作区现状（含 A3a）+ 本补丁」的副本上重证，结果见第四节）。
- 盘点：我跑的 50 个非层 0 测试二进制里，改动之后没有一条测试因这一件变红；改动副本上红的 10 个二进制，在不带这一件的基线上红法逐条相同（第六节）。只换钉值的文件：没有，清单外一份都没改。
- 停下交主 agent 的：取号那一刻某块盘两槽都读不出自证过的系统配置时要不要拒——走得到（瞬时读错），拒要改 `mount.rs` 与 `history.rs`，不在我能动的文件里（第三节 Q1）。

什么现象会推翻「续的行为对」：H1g a = 2 那一形（连着两次取号之后崩、藏掉最新根）可写挂载做成而不是拒；或 mkfs 之后第一次取号写出的系统配置槽 tail 不是 0；或回卷写的 tail 是 0。这三样各有一条测试与变异盯着。

## 二、写过的文件

| 文件 | 改动 |
|---|---|
| `crates/singlefs-core/src/transaction.rs` | 改：`roll_back_acquisition` 加参数 `witnessed_journal_tail`、回卷写带它；新 `JOURNAL_TAIL_WITNESSED_BY_NO_SYSTEM_CONFIGURATION`、`highest_system_configuration_journal_tail`；`write_acquired_instance` 读一次见证值、取号写与两处回卷调用带它；`acquire_instance` 与 `write_acquired_instance` 的文档注释 |
| `crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs` | 新建，6 条测试 |
| `crates/mutations.tsv`（补丁目录里按名字合并，不进 crates.patch） | 追加 6 行（名字见第四节表）；替换 1 行「步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）」——它的原文是 `write_acquired_instance` 末尾那段，`roll_back_acquisition` 多了一个参数、rustfmt 换成多行，锚点跟着改到今天的写法（替换文、参数、必须红的测试不变；它点名的是 layer0 二进制，没证红，留给提交时的 59 号） |

`git apply --stat` 原样（副本不是 git 仓，`git diff --stat -- crates litmus` 在主工作区上会混进别的会话的改动，贴这份补丁自己的）：

```
 crates/singlefs-core/src/transaction.rs            |   63 ++
 ...cquisitions_still_witness_the_newest_publish.rs |  512 ++++++++++++++++++++
 2 files changed, 568 insertions(+), 7 deletions(-)
```

`git apply --check` 对主工作区现状（含 04:1x UTC 打上的 A3a）：过（第七节贴原样）。

## 三、停下交主 agent 的设计问题

### Q1 取号那一刻某块盘两槽都读不出自证过的系统配置：走得到，拒不拒条款没定

规格第 1 条要我确认「取号之前的逐盘核已经拒了这一形」。现查（主工作区现状，含 A3a）：

- 可写挂载的次序：`crates/singlefs-core/src/mount.rs:3395` 调 `devices_without_the_selected_version`（定义在 `mount.rs:2904`，一块盘一份自证过的槽都没有就记 `NoSelfVerifiedSystemConfiguration`、在取号之前拒）；`mount.rs:3413` 才调 `acquire_expected_instance`。
- 取号这一边又读了三遍系统配置槽：`transaction.rs` 的 `next_instance_generation_to_acquire`（`acquire_expected_instance` 写之前重算号）、`write_acquired_instance` 开头的 `highest_system_configuration_instance`、第一道屏障之后的 `highest_system_configuration_journal_tail`（本补丁加的那一遍，行号在副本里是 `transaction.rs:761`）。
- 所以没有新的读错时走不到；**两次读之间一次瞬时读错（或盘在两次读之间坏掉）就走得到**：逐盘核那一遍读得出、见证值那一遍读不出。按实现员定义第 6 条这算「走得到」。
- 另外 `acquire_instance` 本身（测试、`e156` bin、`scenario.rs:151` 在用）前面没有逐盘核。

现在的写法：这块盘不出数，见证值取别的盘里的最大值（与 N-配置 自己对读不出的盘的处置同一取法：登记第 347 行「池里每块盘两槽里全部自证过的……最大值」）；所有盘都读不出时见证值是 0（= 今天的行为）。这不比改动之前差（改动之前恒写 0），但它就是规格要我确认走不到的那一形，我没拒。

为什么没按第 6 条在写之前返回错误成员：`write_acquired_instance` 的错误类型 `AcquisitionFailed` 的 `cause` 是 `BlockDeviceError`（`history.rs:2245` 读它），`ExpectedInstanceAcquisitionFailed` 在 `mount.rs:3413–3431` 被穷举映射——加成员就得改 `mount.rs`（A3a 的文件）或 `history.rs`（模型跟上乙 的文件），两份都在「你都不碰」里。

要拒的话，改法原文（不改，交主 agent 定、另派）：

1. `transaction.rs`：`ExpectedInstanceAcquisitionFailed` 加一个成员 `DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness { device: DeviceIdentity }`；`write_acquired_instance` 在读见证值那一遍逐盘读，某块盘一份自证过的槽都没有就在第一个取号写之前返回它（第一道屏障已发、一个字节没写；要让录制流也不多一步，就把这一核挪到第一道屏障之前、屏障之后再重算一次，对不上不写）。`acquire_instance` 的返回类型要跟着分流（它今天只交 `AcquisitionFailed`）。
2. `mount.rs:3413` 的映射加一臂：映射成已有的 `MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`，`devices` 为那一块、`lacking: SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration`（与 Z3-A 乙、D18（块里携带什么信息） 已定项 11「可见」同一判），`history.rs` 就不用改。
3. 用例钉「返回这个成员、`DiskSnapshot` 不变」：`common::SharedUnreadableRanges` 用 `FailingReadsOfARange::FromTheNthOnward(n)` 让某块盘两槽从见证值那一遍起读坏。

### Q2 回卷写也带见证值：比登记里 N-配置续 的定义多动了一处

E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md` 第 348 行 N-配置续 的定义是「另把取号那一写的 tail 从 0 改成……；其余写一个字节都不改」。规格第 1 条要回卷写也带见证值（「回卷的意思是『像没取过号』，tail 不许因为回卷退回 0」），我照规格做了。回卷写现在写：世代号照旧（这块盘两槽自证过的最大 + 1）、实例代号照旧（取号之前全部自证过的槽中最大的号）、tail = 取号那一刻读到的见证值（改之前是 0）、F 照旧（整池生效值）。E158 装置里的「-配置续」臂有没有同样改回卷写，我没查（`e158_root_choice_repair.rs` 在「你都不碰」里）；两边不一致时，E158 第三段 H1g 的数只罩取号写那一半。

### Q3 `mount.rs` 里两处文档注释改后说反话（`mount.rs` 不归我，给改法原文）

- `mount.rs:336`：「c_见证 = 0（没有自证过的槽，或读得出的槽都是取号那一写的 0）：判据为假。」→ 建议「c_见证 = 0（没有自证过的槽，或读得出的槽都是 mkfs 与 mkfs 之后第一次取号写的 0）：判据为假。」
- `mount.rs:540`：「或取号失败回卷时写回了 tail 0（`transaction::acquire_instance`），盘都是这个样子。」→ 建议「或取号失败回卷时写回了取号那一刻的见证值（`transaction::acquire_instance`，C554 乙-配置续），盘都是这个样子。」

### 条款没写、我没加的非分支项

没有：没加 trait 实现、derive、访问器。`highest_system_configuration_journal_tail` 是私有的，测试不调它，测试自己从盘上读 `recovery::verified_system_configuration_slots` 算。

## 四、新测试与证红

测试文件 `crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs`（下称 T），6 条。每条都从盘上的字节核（`recovery::verified_system_configuration_slots` 解镜像），不调实现里算见证值的函数。「连着 k 次取号之后崩」用 `acquire_instance` 连取 k 次号造，与 `second_transaction_supplement_two_instance_table_page_full.rs` 文件头同一个做法；可写挂载取号走的也是 `write_acquired_instance`。藏最新根照 `common::unreadable_root_slot_of` 与 `a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` 的「根槽 + 两份数据单元每次读都坏」。

| 测试 | 钉什么 |
|---|---|
| `two_crashed_acquisitions_in_a_row_still_witness_the_newest_publish_so_the_writable_mount_refuses_instead_of_abandoning_its_unreadable_root` | H1g a = 2：A、B、C（C 的轮换见证 jsn 5）→ 连着两次取号之后崩 → C 的根槽与数据单元读不出 → `mount_writable` 拒成 `NewerStateStillUnreadableAfterOneReread`，两遍读数都是「所选 B (1, 4)、c_见证 5、末条 4」，`DiskSnapshot` 不变；之后再核两槽都是取号写、tail 都是 5 |
| `three_crashed_acquisitions_in_a_row_carry_the_witness_from_acquisition_write_to_acquisition_write` | 同上，连着三次（第三次读到的两槽都是取号写，见证只靠取号写一路带过来） |
| `the_first_acquisition_after_mkfs_still_writes_tail_zero_so_the_first_transaction_bytes_do_not_change` | mkfs 两槽 tail 0；第一次取号写出世代 2、实例 1、tail 0 |
| `a_later_acquisition_writes_the_largest_tail_of_the_pool_into_every_device_when_the_first_device_lost_its_newest_slot` | 第一个事务之后盘 0 世代 5 那一槽清零（盘 0 只剩 tail 2，盘 1 有 2、3）：第二次取号写进两块盘的 tail 都是 3（整池最大，不是各盘自己的） |
| `…_when_the_second_device_lost_its_newest_slot` | 同上，清的是盘 1 |
| `a_rolled_back_acquisition_writes_the_witnessed_tail_back_instead_of_zero` | 盘 1 的取号写报错（`FaultInjectingBlockDevice`）：盘 0 取号写世代 6（号 2、tail 3）、回卷写世代 7（号 1、tail 3）；盘 1 两槽不动 |

变异（`patch/mutations-append.tsv`，6 行都指 `crates/singlefs-core/src/transaction.rs`，参数 `-p singlefs-harness --test <T>`）。每行都用 `research/scripts/prove-red.sh --copy … --memory 8G singlefs-harness <名字>` 证过；下表是在「主工作区现状（含 A3a）+ 本补丁」的副本 `/tmp/claude-1000/impl-c554-yi-carry/work2` 上的那一遍（基线：6 过 0 红，没有基线红集）：

| 变异名 | 改坏哪一行 | 点名的测试红在哪条断言、看到什么 | 同时红的（T 里） |
|---|---|---|---|
| C554 乙-配置续：取号写的 tail 退回 0（连着两次取号之后见证丢了，可写挂载抛弃读不出的最新根） | `write_acquired_instance` 里取号写的 `witnessed_journal_tail,` → `0,` | two_crashed：T:229 `assert!(matches!(… NewerStateStillUnreadableAfterOneReread …))`，挂载做成、读数 `witnessed_journal_counter: 0, comparison: NothingWitnessed` | three_crashed、rolled_back、first/second_device_lost（5 条） |
| C554 乙-配置续：取号写的 tail 退回 0（连着三次取号，见证只靠取号写一路带过来） | 同一处 | three_crashed：T:229 同一条断言、同样的读数 | 同上 5 条 |
| C554 乙-配置续：取号回卷写的 tail 退回 0 | `roll_back_acquisition` 里回卷写的 `witnessed_journal_tail,` → `0,` | rolled_back：T:480 `assert_eq!`，盘 0 世代 7 读回 `journal_tail: 0` | 只它 1 条 |
| C554 乙-配置续：见证值取成槽世代号（…第一个事务的字节变了） | `highest_system_configuration_journal_tail` 里 `.quantities.journal_tail` → `.quantities.slot_generation` | the_first_acquisition：T:331 `assert_eq!`，读回 `journal_tail: 1` | 6 条全红 |
| C554 乙-配置续：见证值只读盘 0（整池最大的 tail 只在盘 1 上时带错） | 同函数 `.iter()` 后插 `.take(1)` | first_device_lost：T:414 `assert_eq!`，取号写 tail 2（期望 3） | 只它 1 条 |
| C554 乙-配置续：见证值取最小的 tail | 同函数 `.max()` → `.min()` | second_device_lost：T:414 同一条断言，tail 2 | 5 条（the_first_acquisition 不红：mkfs 两槽都是 0） |

证过两遍：打 A3a 之前的副本 `/tmp/claude-1000/impl-c554-yi-carry/work` 上 6/6 抓到（`prove-red-1.log`）；那一遍两条 M1 行红在「两槽布局」断言上、没红在挂载断言上，我把布局断言挪到挂载断言之后，重证那两行（`prove-red-2.log`），再在 work2 上整 6 行重证（`prove-red-3.log`）。`prove-red.sh` 末行原样：

```
✓ 点名 6 条：跑了 6 条，跳过 0 条，跑的都抓到了
```

另替换了 1 行（`patch/mutations-replacements.tsv`，第二节表里那一行）：点名的是 `second_transaction_step_three_acquisition_barrier_layer0`，名字带 layer0，没证红，留给提交时的 59 号。原文在副本 work2 的 `transaction.rs` 里恰好命中一次（33 号在副本上没点它）。

## 五、条款对照（规格第 4 条；我不写 kb，书记员另派）

现查的是主工作区此刻的 kb（`D23`、`D16` 两份在工作区里是 M 状态，第四批 kb 已经写进去了一部分）。

**D23（journal 的角色与格式） 已定项 14**（`.claude/kb/decisions/23-journal的角色与格式.md`，已定项 14 从第 351 行起），第 387 行里写取号那一写 tail 的句子，原样：

> **「续」**：取号那一写（`crates/singlefs-core/src/transaction.rs` 的 `write_acquired_instance`）不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证（用户 2026-09-27 JST 12:08 定「乙-配置续（推荐）」；实现在做，实现员 2026-09-27 JST 12:1x 派出）。系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到；这一形记欠见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 Q1。

要改成什么：

1. 「实现在做，实现员 2026-09-27 JST 12:1x 派出」→ 写实现的落点：`transaction.rs` 的 `highest_system_configuration_journal_tail`（取法：每块盘两槽里全部自证过、fsid 与本池相同的系统配置槽 `journal_tail` 取最大，一份都没有时 0），在 `write_acquired_instance` 第一道屏障之后、第一个取号写之前读一次；逐盘取号写与回卷写都带这一次读到的值。补丁打上之后再改。
2. 加一句回卷写：「取号失败的回卷写（D18（块里携带什么信息） 已定项 11）带同一个见证值，不退回 0」。这一半是规格定的、登记里 N-配置续 的定义（第 348 行「其余写一个字节都不改」）没有（第三节 Q2）。
3. 加一句「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：第一个事务的字节不变」。
4. 取号那一刻某块盘两槽都读不出时怎么办：条款没写，等主 agent 定了第三节 Q1 再写。
5. 「系统配置没见证到的最新根，『续』落地之后由下一次挂载在自己的 c_见证 里看到」这半句，我按实现核了一下（推的，没写用例）：取号写带的是系统配置槽里读到的最大 tail，不读根环、不读 journal；一条根 FUA 了而它那次轮换没落盘，这条根的计数器不会经取号写进 c_见证。它之后要被 c_见证 罩住，得等写行那次发布的轮换写出比它大的计数器。这半句要不要改，请书记员对着 C554 的 Q1 现查。

**D16（发布语义） 已定项 1**（`.claude/kb/decisions/16-发布语义.md` 第 27–74 行）：没有写取号那一写 tail 的句子。命令与输出：`awk 'NR>=27 && NR<=74' .claude/kb/decisions/16-发布语义.md | grep -c '取号那一写\|tail'` → `0`。D23 已定项 14 借的是它第 37 行「根槽这一次读坏」那一行的「重读一次」，不涉及取号写，这一件不用改它。

**清单外、写着「回卷写 tail 0」、要跟着改的一句**：D18（块里携带什么信息） 已定项 11，`.claude/kb/decisions/18-块里携带什么信息.md` 第 314 行，原样：

> 放行两种：只落后、单元都在（根落盘之后、轮换之前崩；取号失败回卷写回 tail 0），这是正常崩溃状态；系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）。

→「取号失败回卷写回 tail 0」改成「取号失败回卷写回取号那一刻的见证值」。

这一改对逐盘核（`mount.rs` 的 `devices_without_the_selected_version`，比的是世代号最大那一槽的 (实例代号, tail) 与所选那一版末条的 jsn）有一处读法变化，推的、没写用例：回卷写之后这块盘最新那一槽是 (取号之前的最大号, c_见证)。改前 tail 是 0，只要所选那一版的实例代号等于那个旧号，这块盘就判「落后」、去读它的单元；改后 c_见证 ≥ 所选那一版末条计数器时判「不落后」、不读单元。这块盘在这次取号之前刚过了同一道逐盘核（核在取号之前），所以只在「核过之后、下一次挂载之前它的单元又坏了」时两种写法判得不同：改前拒、改后放行，交给读的时候按位置条目改读另一份。要不要为这一格补条款或用例，交主 agent 定。取号写本身（新号、tail）改前改后都判「不落后」（新号大于所选那一版的实例代号），没有变化。

## 六、盘点：哪些既有测试的钉值跟着变（规格第 3 条）

搜法与原样输出：`grep -rn 'journal_tail' crates --include=*.rs` 56 行，全文存在 `/tmp/claude-1000/impl-c554-yi-carry/grep-journal-tail.txt`；读系统配置槽的文件 `grep -rln 'parse_slot\|verified_system_configuration_slots\|choose_system_configuration' crates/singlefs-harness/tests crates/singlefs-harness/src` 40 份，清单存在 `grep-sysconf-readers.txt`。这两份清单里的非层 0 测试二进制，加上调 `acquire_instance` 的测试二进制，我在改动副本上都跑了，一共 50 个：第一批 22 个在 A3a 之前的副本上跑，第二批 28 个在 work2 上跑。红了的都换回原 `transaction.rs` 当基线再跑一遍。

**结论：跑过的 50 个二进制里，没有一条测试因为这一件变红。只换钉值的文件：0 份。清单外的文件一份都没改。**

改动之后红的 10 个二进制，基线同红，失败的测试集合与 panic 消息（取前 300 字）逐条相同，都不是这一件带来的：

| 二进制 | 红的测试（改动前后相同） |
|---|---|
| second_transaction_supplement_two_warm_up_counter | c366_when_the_chosen_root_own_record_is_unreadable_…（`NewerStateStillUnreadableAfterOneReread`：所选根自己那条记录读不出 ⇒ 判不出、按真） |
| second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version | blank_device_refuses_the_writable_mount_by_name_before_any_write、mount_writable_while_every_read_of_device_one_fails_is_refused_naming_device_one（C554 乙 的拒排在逐盘核前面） |
| second_transaction_step_three_second_instance | torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg |
| e158_root_choice_repair（bin） | 9 条（主 agent 调度记录第 256 行说过的那 9 条） |
| second_transaction_supplement_three_crash_injection | 3 条（crash_state_landing_on_a_version_rebuilt_from_records_…、on_the_whole_stream_a_kept_publish_missing_a_unit_…、units_of_a_publish_the_landed_version_abandons_…） |
| second_transaction_supplement_three_random_history | 3 条（crash_recovery_abandoning_the_newest_root_…、random_histories_fast_tier_…、rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_…）；74 号（release）红的就是这 3 条 |
| core_review_geometry_back_chain_and_empty_inode | a_journal_ring_shorter_than_one_record_is_refused_before_any_write（mkfs 报 `JournalRingHoldsFewerRecordsThanTheSafetyFactor`；这条路径不经过取号） |
| second_transaction_step_five_reuse | raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_… |
| second_transaction_supplement_three_fault_injection | fault_injection_fast_tier_returns_errors_instead_of_panicking |
| checker_known_bad_images | an_allocation_generation_past_its_unit_birth_…、published_nodes_behind_an_intermediate_row_…（后一条是 C554 乙：见证 5 > 所选末条 2，拒可写） |

改动之后照样绿的 40 个：instance_acquisition、system_configuration_slot_is_overwritten_only_after_a_barrier、a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable、second_transaction_step_three_formatted_pool、second_transaction_supplement_two_unreadable_abandoned_root_slot、instance_table_page_full、instance_table_second_page_write、checker_narrow_invariants_and_abandoned_roots、second_transaction_step_four_rollback、crash_segments_per_device_and_torn_in_place_overwrites、system_configuration_rollback_floor_and_layout_identity、rollback_floor_written_into_the_system_configuration_first_and_normal_unmount、row_publish_checks_before_acquisition、entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration、publish_order_matches_litmus、first_transaction_on_device（bin），加上第二批 28 个里除上表 4 个以外的 24 个（汇总在 `suspects-more/summary.tsv`）。

这 50 个都不在「你都不碰」里，只有 `checker_known_bad_images.rs` 与 random_history 例外。它们红在这一件之外，我没改。

**层 0（没跑，静态列）**：哪几条流在录的那一段里有第二次起的取号写。这几条流那两份系统配置槽写的字节要跟着变：tail 从 0 变成上一次轮换的 tail，校验和随之变。
- `second_transaction_step_zero_layer0.rs`：第二条流「固定脚本到 E」第 221–222 行重开可写挂载（取号 2），这次取号写带 B 的 jsn。`writes_with_stale_journal_tail`（第 1277 行）把每次系统配置槽写的 tail 都改成 2，取号写也在其内，它那条 `to_slot() == slot` 的自检照样成立。
- `second_transaction_position_addressed_trees_layer0.rs`：第 297–303 行「取号之后崩溃」369 次，这些写在基线镜像里，tail 从 0 变成第一次挂载末次轮换的 tail；被录的那次挂载的取号写同样变。
- `second_transaction_step_three_acquisition_barrier_layer0.rs`：第 59–60 行重开可写挂载，取号写的 tail 变。它点名的判据是实例代号，不是 tail。
- 不变：`first_transaction_step_seven_layer0`（只有 mkfs 之后第一次取号，tail 0，它的 tally sha 不受影响）、`second_transaction_step_three_formatted_pool_layer0`（只做过 mkfs 的池，第一次取号）、`parallel_line_one`、`parallel_line_three_spill_over`、`supplement_two_tree_split`（这三条流里没有重开，第 N 次取号都没有）。
- 推的、没量过：写数、段序列、状态数不变，只有字节变。恢复（`recover`）与 checker 都不读 tail 的语义（checker 只在 `crates/singlefs-checker/src/lib.rs:225` 把 tail 解出来，别处不用），所以判定计数应当不变；要等提交时的层 0 全量核。

**checker**：没有判「取号写 tail = 0」的地方。`grep -rn 'journal_tail' crates/singlefs-checker/src/` 只有 `lib.rs:138`（字段）与 `lib.rs:225`（解析）两行。没改 checker，所以没有「受影响的层 0 流与崩溃枚举用例」那一节要写。

**门禁 55 号 second-instance 档与 E142**：55 号第二个实例那一档里取号那两份系统配置槽写的字节会变，预录样本要重录（调度记录第 256 行已经归了 singlefs-39）。E142 第一个事务的字节不变：T 的 the_first_acquisition 钉住了 mkfs 之后第一次取号写 tail 0。

## 七、交回前的验证（末尾原样）

最后一遍在 work2 上跑（主工作区 13:2x JST 的样子，含 A3a，打上本补丁；按协调者的话不用 `--all-targets`）：

```
rustfmt --edition 2021 --check crates/singlefs-core/src/transaction.rs crates/singlefs-harness/tests/<T>.rs
rustfmt --check exit=0

cargo clippy --offline -p singlefs-core --lib --tests -p singlefs-harness --test <T> -- -D warnings <CODE_DISCIPLINE_LINTS 七条>
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.59s
clippy exit=0

cargo build --offline -p singlefs-core --lib -p singlefs-harness --test <T>
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.87s
build exit=0

run-with-memory-cap.sh 8G capped.sh 4 cargo test --offline -p singlefs-harness --test <T>
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.04s
```

更早在副本 work 上（A3a 之前）跑过全仓的三样：`cargo clippy --all-targets --all-features -- -D warnings <七条>` 退 0，`cargo build --offline --all-targets` 退 0，`cargo fmt --all -- --check` 退 1。fmt 那一次唯一的 diff 在 `crates/singlefs-harness/src/bin/first_transaction_on_device.rs:3059`，不是我的文件；主工作区 14:3x 已经给它跑过 rustfmt。

登记给实现员的门禁阶段（`stage-owners.tsv`），在副本上跑：
- 33 号：退 1，只点第 659 行（E158 bin 那一行，不是这一件的）；work2 与主工作区此刻红法相同。我追加的 6 行与替换的 1 行都没被点名。末行：`    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。`
- 53 号：退 0，`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
- 93 号：退 0，`✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；…）`
- 94 号：退 0，`✓ checker 与实现只共享常量模块 singlefs-format（…checker 的 4 份源码零处引 singlefs_core…）`
- 92 号：退 77，`! /tmp/claude-1000/impl-c554-yi-carry/work 不是 git 仓，本阶段跳过`（没判）
- 89 号：退 77（本次未跑；末行是它对收口表第 27 行的提示，与这一件无关）
- 74 号（release，在 work 上）：退 1，`✗ 随机历史的测试二进制判红`。红的 3 条就是第六节表里 random_history 那 3 条，debug 基线同红。

补丁对主工作区现状（14:3x 打上 A3d 与 rustfmt 之后，`crates/mutations.tsv` 1264 行，`transaction.rs` 与我开副本时逐字节相同）：

```
git apply --check /tmp/claude-1000/impl-c554-yi-carry/patch/crates.patch
apply-check exit=0
python3 research/scripts/apply-writer-patch.py /tmp/claude-1000/impl-c554-yi-carry/patch --dry-run
✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1270 行
```

注意：替换行必须跟补丁一起打。不打它，第 298 行那条变异的原文在改后的 `transaction.rs` 里就一次都命中不了，33 号会红。

## 八、没做什么

- 没走三方对抗；没提交。层 0 全量、QEMU（55 号）、herd7、crates 变异整表（59 号）都归 `crash-verifier`。名字带 layer0 的二进制一个都没跑，第六节只静态列了。
- 替换行点名的 layer0 用例没证红，留给提交时的 59 号。
- 第三节 Q1（取号那一刻某块盘读不出时要不要拒）没做，只给了改法原文。Q3 的两处 `mount.rs` 注释没改。第五节 D18 那一句与 D23 里的「续」没改（kb 不归我）。
- `e158_root_choice_repair.rs` 里「-配置续」臂有没有同样改回卷写，没查。
- 全量 `cargo test` 与 `check.sh` 没跑。第六节那 50 个二进制之外的测试，我没在改动之后跑过。

## 九、草稿目录清理

- 删了 `/tmp/claude-1000/impl-c554-yi-carry/work`（18G，A3a 之前的改动副本；证红日志先挪到 `prove-red-logs-1`）。
- 删了 `/tmp/claude-1000/impl-c554-yi-carry/base-full`（18G，基线副本）。
- 删了 `/tmp/claude-1000/impl-c554-yi-carry/work2`（主工作区现状 + 补丁的副本，带编译目录）与 `/tmp/claude-1000/impl-c554-yi-carry/base`（8.5M，做 diff 用的 crates / litmus 原件），大小见 `progress.md` 末行。
- 留着：各日志、`transaction-patched.rs`（补丁后的 `transaction.rs` 一份拷贝）、`patch/`、`report.md`、`progress.md`。
