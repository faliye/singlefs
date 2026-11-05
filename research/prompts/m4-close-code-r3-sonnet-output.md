# m4-close-code-r3 正推腿报告（Sonnet，2026-11-05）

方案按 `crates/` 今天的实现来谈。代码一律读冻结副本（下称「冻结副本」）：`~/.local/share/singlefs/work/m4-close-code-r3/repo/`，引文行号以它为准（文件名前省去 `crates/singlefs-core/src/` 的写全路径）。kb 条款行号是我在仓里现查的（`grep -nF`），不是背景材料里的行号。没有写模型，没有起任何脚本或测试；需要实测的在「要主 agent 跑的命令」一节。

## 0　先说两处背景材料与仓里对不上的

- 正文说 C946 在 `.claude/kb/checks-owed.md` 第 595 行；现查在第 593 行（`grep -nF '| C946 |'` 命中一行，593）。第 595 行是 C949。D4 已定项 4 定案在第 71 行，对得上。
- 正文把「I-9.4 第四句」当作行内记录升序那一句：`crates/singlefs-core/src/recovery.rs:5991` 的注释写「I-9.4 第四句」，而 `crates/singlefs-checker/invariants/09-inode树结构.md:16` 那一行（I-9.4）今天只剩三个分句（第三句已随 M3 借删），行内记录升序是第三句。只是编号口径，不影响判定。

## T1　四个读者判 inode 叶交叠

**判定：成立**（附两条旁注，不改判定）。

### 依据

判的是同一个函数。`crates/singlefs-core/src/recovery.rs:6010` 起的 `inode_records_of_the_leaf_ascend_after_the_leaves_before` 是唯一实现，四个调用点：

| 读者 | 调用点 | 失败类型 |
|---|---|---|
| 可写挂载重建（`inode_tree_read_and_judged`） | `crates/singlefs-core/src/recovery.rs:3833` | `RecoveryFailure::InvariantViolated`，经 `RebuildVersionFailure::Walk` 与 `map_rebuild_failure` 进 `MountError` |
| 冷走读 | `crates/singlefs-core/src/recovery.rs:7412` | `RecoveryFailure::InvariantViolated` |
| 快照第一个文件冷读 | `crates/singlefs-core/src/recovery.rs:7597` | `RecoveryFailure::InvariantViolated` |
| 只读挂载 `open_pool_for_read` | `crates/singlefs-core/src/mounted_read.rs:575` | `OpenPoolForReadFailure::Walk(RecoveryFailure::…)`（`From` 在 `crates/singlefs-core/src/mounted_read.rs:218`） |

四处喂的都是「叶序上一片及以前的最大记录号」加这一片已解开的记录；里面那个值是同一个常量对：交叠 ⇒ 「I-9.12」加 `INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM`（`crates/singlefs-core/src/recovery.rs:6017-6021`），片内不升序 ⇒ 「I-9.4」加 `INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND`（`crates/singlefs-core/src/recovery.rs:6028`）。四个外壳类型不同，里层的 `RecoveryFailure` 值逐字相同。harness 里 `crates/singlefs-harness/tests/inode_tree_readers_refuse_overlapping_inode_leaves.rs` 四条 `#[test]`（第 243、257、272、290 行起）对同一张坏镜像各断一个读者；我没跑，见「要主 agent 跑的命令」。

与用户定案和 r2 判决对：判决第三节取改法甲「读回照 I-9.12 判叶间严格升序，片内不升序判 I-9.4」；代码判的是 `largest_before >= first.inode` 即叶间严格升序（`crates/singlefs-core/src/recovery.rs:6017`），片内相邻 `>=` 判 I-9.4。对得上。

### 第五个读者有没有

我把 inode 叶的解析入口全仓 grep 了一遍：

- `grep -rn 'inode_internal_entry_parsed_and_judged\|inode_leaf_container_judged_against_its_entry' crates` 在产品代码里只有 `recovery.rs` 三处（第 3809/3816、7383/7400、7573/7585 行）；`mounted_read.rs` 自己的一份（第 507–575 行）没走这两个函数、自己解记录，但接了同一个判叶序函数。
- `grep -rln 'InodeRecord::parse' crates`：产品代码只有 `records.rs`（定义）、`recovery.rs`、`mounted_read.rs`；其余是测试。
- 可写挂载「回退读上一版」「读阶段缓存」：`rebuild_version_in_the_unit_area_starting_at` 是唯一建版入口（`crates/singlefs-core/src/recovery.rs:3326`），内部就是 3409 行调 `inode_tree_read_and_judged`；`ReadStageCache`（`crates/singlefs-core/src/mount.rs:8725`）只是读者外壳，不解 inode 叶。`crates/singlefs-core/src/mount.rs:10277` 的 `inode_leaf_containers.clone()` 是拷已建的版，不是读者。`region_scan.rs` 里的 `PACKED_TYPE_INODE` 全在 `mod tests`（第 849 行起）。
- 池级 checker（`crates/singlefs-checker`）自己判 I-9.12，另一套实现，不在这一轮射程。

结论：核心层读 inode 树的路径四个，都接了，没有第五个。

### 误拒合法镜像

- 空叶：函数开头 `let Some(first) = records_of_the_leaf.first() else { return Ok(largest_of_the_leaves_before) }`，空叶原样放行、不推进最大值，不误拒。
- 单条叶：`windows(2)` 对一个元素不产生窗口，只比下界，不误拒。
- 叶间严格升序是合法镜像的必要条件：I-9.12 要求第 i 条分隔 key 大于第 i−1 个孩子区间上界、不大于第 i 个孩子区间下界（`crates/singlefs-checker/invariants/09-inode树结构.md:24`「第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界」），推出上一片最大记录号 < 这一片最小记录号；`>=` 才拒，等号拒是对的（同一个 inode 号在两片里也是交叠）。
- 叶跨盘副本不一致：这个函数只看读者交来的那份字节解出的记录，副本之间不一致由读单元那一层按位置条目校验和挑；这个函数不引入新的误拒来源。

### 旁注（不改判定，但主 agent 要知道）

1. 空叶的处理四处并不一致，且这是改动之前就有的：可写挂载重建在 `crates/singlefs-core/src/recovery.rs:3826` 拒空叶（`UnitMalformed`「inode 叶容器没有记录」）；冷走读、快照冷读、只读挂载没有这句，空叶放行。补丁没动这件事（`records.is_empty` 在改动全文里零命中：`grep -n 'records.is_empty' research/prompts/_m4-close-code-r3-diff.md` 无输出）。「同一张镜像四个读者一个结局」对「含空叶的镜像」不成立；对「两片叶交叠」成立。
2. 读者里只判了「记录」的叶间升序，没有判「内部条目分隔 key 自己严格递增」（I-9.12 第一半）。叶间记录严格升序加每条记录不小于自己的分隔 key（`crates/singlefs-core/src/recovery.rs:6810` 的 I-9.4 判）并不推出分隔 key 递增（分隔 key 可以比前一片的最大记录小）。这是 r2 判决改法甲没要的，不算补丁没做到，只是「I-9.12」这个名字在读者里管的是它的必要条件。

## T2　只坏一份的计数

**判定：部分成立。** 计数落点、两份都坏不计数、不加盘上字节、重开从 0 起都对；有三处与「同一份坏副本一次操作数一次」「按一次可写挂载累计」的字面不严，其中一处（c）是条款没覆盖、实现者与主 agent 自己选的口径。

### 条款对照

D4 已定项 4 的计数句：「读到只坏一份时计数，并经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离；走同一个事务层，不开第二条写路径」（`.claude/kb/decisions/04-校验和位置.md:71`）。C946 的用例：「读出的都是原内容、计数各加一，之后这个单元落在新槽、两份新写、旧落点隔离」（`.claude/kb/checks-owed.md:593`）。用户 2026-11-04 两条：有活快照时「另发一批搬迁修好」、只读挂载「只读挂载只计数，不修」（`records/2026-10-13-里程碑四预开发.md:1434`）。

### 五处各在哪里计、计什么、累计在哪

| 读路径 | 计在哪（冻结副本） | 计的是 | 累计在哪 |
|---|---|---|---|
| 部分写读回 | `crates/singlefs-core/src/transaction.rs:11645`（`read_back_the_old_payload_counting_a_failing_copy`，被 `publish_partial_write` 在 11914 起的循环调用，每个要读回的单元一次） | 单元读（读回一个单元，该单元前一份没读成后一份对上） | `PoolWriter` 的 `Cell<u64>` 加一份映射 key 表（`crates/singlefs-core/src/transaction.rs:371-375`）→ `crates/singlefs-core/src/mounted_session.rs:2273` 并进 `OldDataUnitReadsBackWithOneCopyFailingInThisChange` → `crates/singlefs-core/src/mounted_session.rs:2536` 加进 `CompactionOfThisMount` 私有字段（`crates/singlefs-core/src/compaction_plan.rs:855`、累加在 999） |
| 截断读回 | 同上，`publish_size_change` 在 `crates/singlefs-core/src/transaction.rs:11473` 起调同一个函数 | 同上 | 同上 |
| 挂载态 `read_at` | `crates/singlefs-core/src/mounted_read.rs:1211`（`read_unit_by_location_entries`，只在读成功那一支且前一条失败时加一，同时报给读者 1212） | 单元读 | `ReadPathObservation.data_unit_reads_with_one_copy_failing`（`crates/singlefs-core/src/mounted_read.rs:86`），一次读成功后并进 `observation_since_open`（`crates/singlefs-core/src/mounted_read.rs:1043`） |
| 冷恢复读 | `crates/singlefs-core/src/recovery.rs:821`（`read_unit_via_locations`，同一模式），只报给读者；默认实现什么都不做（`crates/singlefs-core/src/recovery.rs:191`） | 单元读（各类单元：数据、树节点、映射节点） | 套了 `PoolReaderCountingUnitReadsWithOneCopyFailing` 的读者自己的 `Cell`（`crates/singlefs-core/src/recovery.rs:197-216`），冷读不挂载时调用方自己套 |
| 挂载时建上一版 | 同上（recovery 的单元读）；`crates/singlefs-core/src/mount.rs:9416` 把盘套上计数层，只套在 `rebuild_previous_version` 的两处（9436、9487）；判退的三样读者不经它（注释写明，避免重数） | 单元读（各类单元） | 9502 取出 → `InstanceStart` 字段（`crates/singlefs-core/src/mount.rs:1394`）→ `crates/singlefs-core/src/mount.rs:7661` 记进 `CompactionOfThisMount` 的另一个私有字段（`crates/singlefs-core/src/compaction_plan.rs:858`） |

计的都是「一次单元读」，不是按盘计。「读到只坏一份」在三处实现里是同一个定义：同一组两条位置条目里，前一条没读回字节或整单元校验和对不上，后一条对上（`crates/singlefs-core/src/transaction.rs:11765`、`crates/singlefs-core/src/mounted_read.rs:1211`、`crates/singlefs-core/src/recovery.rs:821`）。位置提示两条都没对上、转去查中央映射那一回不并进来，映射那一组里自己只坏一份的照算。

### 逐问

**同一份坏副本一次操作里会被数两次吗？** 一次单元读之内不会（成功那一支才加，且每次单元读只走一支成功）。一次操作之内会，有两形：

- (a) 用户改动被空间拒、推抬 F 再试：`publish_the_change_raising_the_floor_for_space` 循环里每轮调 `publish_the_change_once`（`crates/singlefs-core/src/mounted_session.rs:3053`），每轮新建 `PoolWriter`，部分写在 `crates/singlefs-core/src/transaction.rs:11914` 起的循环里读回旧单元，发生在准入之前，所以被拒的轮次读回也计数，重试再读再计。实现者在文档里写明了：「被拒的那几次发布读回的也算：读是真发到盘上的」（`crates/singlefs-core/src/mounted_session.rs:259` 这一段注释），并在 `relocations_of_old_data_units_kept_for_the_snapshots` 里对映射 key 去重（文档写「被拒重判的那次发布读回同一个单元，记两把」，`crates/singlefs-core/src/transaction.rs:11662`；计划只留一把）。所以计数是「读次数」，不是「坏副本个数」。C946 用例里的「计数各加一」只在不被拒的操作里成立。我没找到断言被拒重试时计数的用例（`crates/singlefs-harness/tests/data_unit_read_with_one_copy_failing_is_counted.rs` 里被拒的两条用例 871、921 起，拒的是另发那一批，不是用户改动本身）。
- (b) 挂载时：`fall_back_judged_on_the_three_readers` 判出要退时，先在 `crates/singlefs-core/src/mount.rs:9436` 用计数层建退到的那一版，失败就 `.ok()` 吞成 `None`（`crates/singlefs-core/src/mount.rs:9442`），再在 9487 对所选那一版建一遍，用的是同一个计数层、没清零。退的那一版若在计到只坏一份之后才失败，那几次计数留着，所选版再读同一批单元再计。窄，但是真的。

**两份都坏时照旧返错、不计数吗？** 是。三处计数都在「后一条对上」那一支才执行；两份都坏时：`read_back_the_old_payload_counting_a_failing_copy` 里 `?` 在 `match` 之前返回（`crates/singlefs-core/src/transaction.rs:11634`），`read_unit_by_location_entries` 落到函数末尾的两个失败结局（`crates/singlefs-core/src/mounted_read.rs:1218` 起），`read_unit_via_locations` 循环走完无 `return Ok`。

**计数不加任何盘上字节吗？** 是。计数的存放处全是内存量：`Cell<u64>`、`RefCell<Vec<Vec<u8>>>`、`CompactionOfThisMount` 私有 `u64`、`ReadPathObservation`；补丁里没有任何一行新增 `to_bytes`、`write_at` 或格式层改动（`grep -n '^+' research/prompts/_m4-close-code-r3-diff.md | grep -i 'to_bytes\|write_at\|singlefs_format'` 零命中）。

**会话重开、续做、整理之后对不对？**

- 重开：`CompactionOfThisMount::of_a_new_mount()` 把两个计数初始化为 0（`crates/singlefs-core/src/compaction_plan.rs:883-884`），挂载里的构造点只有 `crates/singlefs-core/src/mount.rs:7927` 一处，`MountedSession` 的结构体字面量只有 `crates/singlefs-core/src/mounted_session.rs:2401` 一处（`grep -n 'MountedSession {'` 的构造点），所以新挂载 0 起，会话存续期间不被清零或替换（`grep -n 'mem::take\|mem::replace' mounted_session.rs` 与 compaction 相关的零命中）。用例 610 起已钉重开从 0 起。
- 续做：挂载续做（删除队列、整理意图）发的是删除类与整理发布，不读回旧数据单元，不产生这两份账；挂载时那一份在续做之后、清扫之前并进（`crates/singlefs-core/src/mount.rs:7661`，在 `compaction_intent_continued_at_the_mount` 返回之后），不会被续做那一步里新建的 `CompactionOfThisMount::of_a_new_mount()`（7927）盖掉，因为是加到续做返回值里的那一份上。
- 整理之后：`run_compaction` 对 `compaction` 就地改，没有整体替换；整理的搬迁读不经这两条计数。
- 一条小的丢失：`read_at` 的单次调用跨几个单元时，前面单元计到了只坏一份、后面单元读失败（返错）会让 `observation` 不并进 `observation_since_open`（并进只在成功路径，`crates/singlefs-core/src/mounted_read.rs:1043`；失败用 `?` 早退）。写入口那一侧「被拒的也算」，读者这一侧「失败的不算」，口径不一致。读者通知（`reader.note_that_a_unit_was_read_with_one_copy_failing`）已经发出去了，所以套了计数层的读者仍然数到。这是改动之前 `device_reads_issued` 等全部计数共有的行为，补丁没改它。

### (c) 条款没写到的口径，是实现者与主 agent 定的

C946 用例要求四条读路径读完「这个单元落在新槽、两份新写」。代码只在部分写、截断两路兑现（单元本来就在同一次发布里整个重写）；挂载态 `read_at` 与冷恢复读只计数不搬。前者由用户 2026-11-04 明令（只读挂载只计数）；冷恢复读、挂载时建上一版那一读只计数不搬，记录里是主 agent 认下的实现员提案（`records/2026-10-13-里程碑四预开发.md:1437`「挂载读只计数不搬——认」），不是用户原话。kb 的 D4 已定项 4 与 C946 两处今天还是旧字面（「这两样未实现」），没改写；记录说「写回 kb 随这一批收尾」，所以现在是字面对不上。我按用户明令加主 agent 认下的口径判代码，字面落差记在这里。

## T3　有活快照时另发一批搬迁与被拒推抬 F

**判定：成立**（附四条口径提醒，其中第 1、2 条是条款没写到、实现者选的，交主 agent 判是不是要用户知道）。

### 条款对照

- 用户 2026-11-04 三条：有活快照时「另发一批搬迁修好」；另发那批被拒时「推抬 F 再试一遍」（`records/2026-10-13-里程碑四预开发.md:1436`，选项原话是「推抬 F 再试一遍」，另两项「记下来，下次改动再试」「照现在：交回不再试」）。
- D8 已定项 16 第 6 条：「会话走空时删除类发布（意图那次与分批的每一批）被落点那一判以空间拒，推一串抬 F、回收后再试」「挂载续做走空时删除类发布被落点那一判以空间拒，同样推抬 F 再试，预算口径与会话走空相同」（`.claude/kb/decisions/08-核心索引结构.md:512`），以及「每一次被拒的删除类发布各算一次准入」（同文件同行）。
- D4 已定项 4：「走同一个事务层，不开第二条写路径」（`.claude/kb/decisions/04-校验和位置.md:71`）。
- `.claude/rules/fs-design.md:122`「每条分支必须能被测试强制进入。」

### 逐问

**另发那批走的是同一个事务层、没开第二条写路径吗？** 是。入口 `relocate_the_old_data_units_kept_for_the_snapshots`（`crates/singlefs-core/src/mounted_session.rs:2047`，在 `publish_user_change` 里于 `published?` 之后调，2537/2542 行）→ `publish_the_relocation_batch_raising_the_floor_for_space`（2123 起）→ `publish_the_compaction_step_once` → `publish_compaction_step_with_the_region_check_of_the_intent_deletion` → `publish_compaction_step`，意图一栏是 `CompactionIntentStep::CarryTheIntent`（`crates/singlefs-core/src/transaction.rs:10600`，整理批用的同一个入口 `publish_relocation_batch`），落盘经 `publish_version_of_trees_relocating_units_after_a_check`（`crates/singlefs-core/src/transaction.rs:10889` 调它）末尾的 `persist_the_publish_or_freeze_it`（`crates/singlefs-core/src/transaction.rs:12525`），用户改动的 `publish_version` 末尾也是同一个函数（`crates/singlefs-core/src/transaction.rs:1869`）。被活快照钉着的单元走的是已有的「留存的映射条目」支（`crates/singlefs-core/src/transaction.rs:11040-11075` 的 `RelocatedUnitHolder::KeptForTheSnapshots`），字节从盘上按映射位置读、两份取先核得上位置项校验和的那份，旧落点照释放之前读盘核后两份一起隔离。没有新增 `PoolWriter::new` 以外的写入口，没有直接 `write_at`。`grep -n 'write_at' ` 在补丁新增行里零命中（见 T2 同一条 grep）。

**推抬 F 的预算口径与会话走空相同吗？** 相同。两处都用同一个函数 `push_one_floor_raise_within_the_admission_budget`（`crates/singlefs-core/src/mount.rs:4778`），都传「这次准入已推的空发布数」：走空版本 `publishes_pushed_in_this_admission += publishes_of_this_raise`（`crates/singlefs-core/src/mounted_session.rs:5110`），另发版 `publishes_pushed_in_this_admission += raised.publishes.len()`（`crates/singlefs-core/src/mounted_session.rs:2195`）；上限都是 `PUBLISHES_PER_ADMISSION_AT_MOST`。「这一批算一次准入、推的数从 0 起」对应 D8 的「每一次被拒的删除类发布各算一次准入」。判被拒是不是空间不够用的是同一个谓词：`compaction_publish_refused_short_of_space` 对 `RelocationBatchRefused::Publish(cause)` 直接调 `refusal_is_short_of_space(cause)`（`crates/singlefs-core/src/mounted_session.rs:4066-4068`）。

**推抬 F 之后会话的 `current` 换到推到的那一版，后续用户改动看到的是对的那一版吗？** 是。另发那批在一份拷贝 `version_to_relocate_on` 上发，`push_one_floor_raise_within_the_admission_budget` 就地推进它（签名 `current: &mut TransactionOutput`，`crates/singlefs-core/src/mount.rs:4778`）；发完无论结局，先 `*current = PoolVersion::of_a_version_of_trees(version_to_relocate_on)`（`crates/singlefs-core/src/mounted_session.rs:2080`），发成再覆盖为 `relocated`；`of_a_version_of_trees` 按有无文件分 `WithFile` 与 `WithTreesAndNoFile`（`crates/singlefs-core/src/transaction.rs:2441`）。三种没发成的结局（`NoSpaceAfterRaisingTheFloor`、`RefusedNotForSpace`、`FloorRaiseFailedWhilePushingForSpace`）都走这一句，所以 `current` 停在「推到的那一版」，没推成时就是改动发成的那一版。发布路径本身在被拒时把分配器退回发布之前那一份（`crates/singlefs-core/src/mount.rs` 同一函数的文档），所以 `current` 与分配器是同一条时间线。

**三种结局与用户定案对得上吗？**

- 推满交回 `NoSpaceAfterRaisingTheFloor`、不写字节：`Stopped(stop)` 支返回，位置在 `crates/singlefs-core/src/mounted_session.rs:2202`；被拒是空间准入或落点在任何写之前（该批没写）；此前推的抬 F 是用户定案要的，已落盘。
- 不是空间不够交回 `RefusedNotForSpace`、不重试：`if !compaction_publish_refused_short_of_space(...)` 立刻 return（2171-2172），不进推抬 F 那一段。
- 抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`：`Err(cause)` 支（2211）。

**三个只供测试的开关在生产路径上恒为关吗？** 是（靠「没有调用者」，不靠类型）。三个开关都是 `CompactionOfThisMount` 的私有字段，产品值在唯一构造点 `of_a_new_mount`（`crates/singlefs-core/src/compaction_plan.rs:886`、888、890）；该结构体在产品代码里只在 `crates/singlefs-core/src/mount.rs:7927` 构造；三个 `force_…_for_tests` 设置函数在 `crates/singlefs-core/src` 里只有定义，没有调用（`grep -rn 'force_the_relocation_kept_for_the_snapshots_refused\|force_the_floor_raise_for_the_relocation_kept' crates` 只命中 `compaction_plan.rs` 和 `crates/singlefs-harness/tests/data_unit_read_with_one_copy_failing_is_counted.rs`）。提醒：设置函数是 `pub`，`MountedSession.compaction` 本来就是 `pub` 字段（`crates/singlefs-core/src/mounted_session.rs:120`），不是 `#[cfg(test)]`；与同文件里别的「只供测试」开关同一做法，生产恒关靠约定。三个开关各有强制进入的用例：被空间拒 871、921、1017 起的三条，不是空间不够 1108 起，抬 F 报错 1201 起，块设备真错（故障注入，不用开关）1351 起（`crates/singlefs-harness/tests/data_unit_read_with_one_copy_failing_is_counted.rs` 的 `.force_the_…_for_tests(` 调用点：877、928、1024、1115、1208、1216、1321 行）。满足 fs-design 第 2 条。

**排空被拒那一支真没动吗？** 没动。该分支在 `MountedSessionWithPendingChanges::run_compaction_in_the_background`，`crates/singlefs-core/src/mounted_session.rs:5951` 起；`Err(refusal)` 那一支在整理起之前 `return Err(…PublishOfThePendingChangesRefused…)`（`Err(refusal)` 在 5989 行，`return Err(` 在 6000 行一带）。补丁里 `mounted_session.rs` 的全部 hunk 的旧文件行号落在 28–2703 之间（`awk '/^@@/' ` 过改动全文里 `mounted_session.rs` 各段），都在该分支（该分支在冻结副本第 5951 行起，远在最大旧行号之后）之前。该分支文本我读过，调用 `publish_the_pending_changes` 失败时先复位 `start_judgement` 再 `return Err`，之后才有 `run_the_background_compaction_that_starts`，顺序没变。

### 口径提醒（不改判定）

1. **「再试一遍」在代码里是循环到推满，不是只试一次。** 条款与用户选项字面是「推抬 F 再试一遍」；代码按会话走空那一套：每被空间拒一次就推一串、再试，至多约 8 轮（预算 `PUBLISHES_PER_ADMISSION_AT_MOST` 内）。与 D8 已定项 16 第 6 条会话走空的写法一致，也与实现者文档「与会话走空同一套」一致；但记录里用户选的是「推抬 F 再试一遍」，没有说「直到推满」。读成「一遍」＝「推一串、再试一次」的话，代码多了循环。我判相同口径，是因为文档、用户任务描述里的「推满交回」和 D8 都按循环写。
2. **推满时不判「被宽护卡住」。** 会话走空那一套在 `NoSpaceAfterRaisingTheFloor` 里带 `wide_protection_at_the_ceiling`（`crates/singlefs-core/src/mounted_session.rs:5027` 一带）；另发版的 `NoSpaceAfterRaisingTheFloor` 变体只带 `last_refusal`、`stop`、`floor_raises`（`crates/singlefs-core/src/mounted_session.rs:228-232`）。记录里这是实现员交代码轮的一条（`records/2026-10-13-里程碑四预开发.md:1437`「推满时不判被宽护卡住……不另定，交代码轮攻」）。我没判它该不该有。
3. **另发这批放在前台同一次调用里。** D24 已定项 2 第 2 条「不许进前台路径」讲的是后台重活；记录里主 agent 已「认」放前台并交代码轮攻（同一行 1436）。不在正推射程。
4. **开关造的拒绝不点名盘**（`short_devices: Vec::new()`，`crates/singlefs-core/src/mounted_session.rs:2228`）：是造出来的，不是真算的，`fs-design` 第 2 条要的就是「无视判据直接走指定分支」。

## T4　`publish_the_change_once` 的挪动

**判定：成立。**

### 依据

- 逐字对：我把 HEAD 里旧函数 `publish_the_change_once`（`git show HEAD:crates/singlefs-core/src/mounted_session.rs`，从 1882 行起）的 `match change { … }` 整段，与冻结副本新函数 `publish_the_change_on_the_writer`（`crates/singlefs-core/src/mounted_session.rs:2282`，`match change {` 在 2289 行）的 `match change { … }` 整段，用 `diff` 比：`IDENTICAL`（两段各 102 行；命令与输出在草稿目录 `~/.local/share/singlefs/work/m4-close-code-r3-forward/` 的 `old_fn.txt`、`new_fn.txt`，sha256 分别是 656846be…、2f8085e8…）。十一个臂（`Unlink`、`Overwrite`、`SequentialWrite`、`NewInodes`、`CreateSnapshot`、`DestroySnapshot`、`Truncate`、`OverwriteOfAnInode`、`SequentialWriteOfAnInode`、`WriteAtOffset`、`DataChangesPublishedTogether`）每臂调的函数与参数次序逐个相同。
- 参数借用：旧函数里 `writer` 是 `let mut writer = PoolWriter::new(…)` 的本地值，各臂写 `&mut writer` 得 `&mut PoolWriter`；新函数里 `writer` 是形参 `mut writer: &mut PoolWriter<'_, Device>`，各臂 `&mut writer` 是 `&mut &mut PoolWriter`，由自动解引用降成 `&mut PoolWriter`。十一个被调函数的第一个形参都是具体类型 `pool: &mut PoolWriter<'_, Device>`（transaction.rs：`publish_unlink_intent` 11196、`publish_overwrite` 9773、`publish_sequential_write` 9830、`publish_new_inodes` 9890、`publish_create_snapshot` 10340、`publish_snapshot_destruction_intent` 10418、`publish_size_change` 11373、`publish_overwrite_of_an_inode` 9958、`publish_sequential_write_of_an_inode` 10009、`publish_partial_write` 11849、`publish_data_changes_published_together` 10260），没有泛型 `W` 去吃 `&mut &mut`，所以降成同一个引用。
- 外层 `publish_the_change_once` 在 `crates/singlefs-core/src/mounted_session.rs:2273` 之前建 `PoolWriter`、设 `set_container_tree_capacities`，调新函数，之后把写入口上的读回账并进去（2273 `add_the_reads_of_the_writer`），再把结果交回；`published` 的结果值没有被改写。
- 变异表第 2471 行锚点：`crates/mutations.tsv` 第 2471 行第四列以 `thread_local! { … }\n    match change {\n        UserChange::Unlink {` 开头；新文件里 `^    match change {$` 恰好一处（`grep -c` = 1，第 2289 行，缩进 4 个空格，下一行 `UserChange::Unlink {`）。锚点的缩进、前后两行都还在。
- `allow(clippy::needless_borrow)` 掩住真问题吗：它挂在整个新函数上（`crates/singlefs-core/src/mounted_session.rs:2278`），这个函数体只有一个 `match`，十一个臂里被它抑制的是同一类「对 `&mut` 形参再借一层」的提示；`allocator`、`previous`、`change` 都是直接传、没有多余借用。我没看到它盖住别的 lint 实例。另外 `mut writer` 形参的 `mut` 必须有（取 `&mut writer`）。

### 什么会推翻

T4：任何一个臂的调用在新旧函数里不逐字相同（`diff` 非空），或某个被调函数改成了泛型接收 `&mut &mut PoolWriter`，或 `crates/mutations.tsv` 第 2471 行锚点的缩进与新文件不符。

## 什么现象会推翻各格的判定

- T1 成立：任一读者在两片叶交叠的镜像上收下（不拒），或拒出的 `RecoveryFailure` 不是 `InvariantViolated { invariant: "I-9.12", detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM }`；或在一张合法的多片叶镜像（含单条叶、含被借过之后分隔 key 变化的叶）上被这个函数拒掉；或 `grep` 找出第五个解 inode 叶记录的产品代码入口。
- T2 部分成立：把 (a) 一次被空间拒的部分写在同一块坏单元上读回 N 次，会话累计是 1 而不是 N（则 (a) 不成立）；把 (b) 退的那一版建失败后所选版再建，挂载时计数等于所选版自己数到的（则 (b) 不成立）。若有人读出 C946 的「计数各加一」本来就是「读次数」，则 (a) 只是文档口径，不是偏差。
- T3 成立：另发那批在 `publish_the_compaction_step_once` 之外另有一条落盘调用；或预算与会话走空不是同一个函数同一个口径；或三个开关在 `crates/singlefs-core/src` 里有产品调用；或排空被拒那一支在起整理之前不再 `return Err`。
- T4 成立：见上节末尾。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 四个读者判 inode 叶交叠 | 成立 | 四处调同一个函数、里层失败值逐字相同，没有第五个读者；空叶四处处理不一致是改动之前就有的，不属本补丁。 |
| T2 只坏一份的计数 | 部分成立 | 落点、两份都坏不计、不加盘上字节、重开 0 起都对；被空间拒重试会重计（文档写明的口径）、挂载退版失败后不清零、`read_at` 失败不并进累计；冷读与挂载建版只计不搬是主 agent 认下的口径，kb 字面还没跟。 |
| T3 有活快照时另发一批搬迁与被拒推抬 F | 成立 | 同一事务层与同一个预算函数；`current` 换到推到的那一版；三开关生产恒关（靠没有调用者）；排空被拒那一支没动；「再试一遍」在代码里是循环到推满，口径提醒见上。 |
| T4 `publish_the_change_once` 的挪动 | 成立 | 十一个臂 `diff` 逐字相同，被调函数形参都是具体类型，变异表 2471 锚点在新文件里恰好一处且缩进未变；`allow` 没有盖住别的实例。 |

## 要主 agent 跑的命令

我没有往统一任务服务交任何任务，也没有起脚本、测试或编译。下面这些写成整条交主 agent；「待主 agent 跑」。`--cwd` 请主 agent 选带这五份补丁的工作区（主工作区或一份新拷贝），不要在冻结副本里编（冻结副本只读）。

1. 四个读者判叶交叠（T1 的实测；这四条在合入后的两包测试里本来就在内，要的是单独看一眼是绿的）：
   `task-server/task submit -- cargo test -p singlefs-harness --test inode_tree_readers_refuse_overlapping_inode_leaves`
2. 计数与另发一批搬迁的整份用例（T2 / T3 的实测；注意用例 871、921、1017 里「被拒」的是另发那一批，不是用户改动本身，所以它们覆盖不了 T2 (a)）：
   `task-server/task submit -- cargo test -p singlefs-harness --test data_unit_read_with_one_copy_failing_is_counted`
3. T2 (a) 的反证用例（现有用例没有，要新写，下面是我建议的形状，由主 agent 或实现员写）：可写挂载里把现行那一版第 1 个单元的头一条位置条目那一份弄坏，再造一次用户改动的空间拒（不用只供测试的开关，让池真的差一个准入，例如盘填到准入刚好不够、之后靠推抬 F 回收放行），发部分写；断言发成后 `old_data_unit_reads_back_with_one_copy_failing_in_this_mount()` 的值：我的读法是大于 1（每轮被拒的发布各读回一次），若等于 1 则 (a) 被推翻。重型：否（内存稀疏盘）。
4. T2 (b) 的反证用例（现有用例没有）：造一张退版镜像，让「退到的那一版」建到一半失败（例如在它的一个数据单元上两份都坏，前面另一个单元只坏一份），断言挂载交回后 `unit_reads_with_one_copy_failing_while_mounting()` 的值；我的读法是包含退版失败前那几次，若不含则 (b) 被推翻。
5. T4 的 `allow` 与锚点（由主 agent 核，我只读不编）：
   `task-server/task submit -- cargo clippy -p singlefs-core --all-targets -- -D warnings`
   （主 agent 之前的合入后验证 clippy 红在 harness 测试第 167 行，那是已认的另一处；这条看 `needless_borrow` 在 `mounted_session.rs` 里零报。）

## 没做什么

- 没替主 agent 采纳或出判决；这份只回答「代码做的是不是条款与用户定案说的」。没往统一任务服务交任务，判决要引的数（测试通过、计数读数）都没有复跑，上面第 1–5 条是命令。
- 没判攻方格的内容（过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）；T3 里 D24 前台路径、推满不判被宽护卡住两条只记了口径，没判该不该。
- 没读禁读清单里的东西（`research/prompts/m4-close-code-r3-opus-output.md`、`m4-close-code-r3-opus-model/`）。
- 没写模型，所以没有 `rerun.sh` 与 sha256 清单；唯一的产物是 T4 的两份摘出来的函数文本（草稿目录 `old_fn.txt`、`new_fn.txt`），用于 `diff`。
- 池级 checker（`crates/singlefs-checker`）、GPU 恢复内核 `crash_judge_recovery.wgsl` 没跟叶序判法这一处，正文明说不在这一轮；我没读。

## 交回前的引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-close-code-r3-sonnet-output.md --root /home/fy5090/code/singlefs --root /home/fy5090/.local/share/singlefs/work/m4-close-code-r3/repo`，退出码 0，末行：「✓ 核了 11 处引文、71 处裸行号，对上 11 处，没判 71 处（其中裸行号没有引文可比 71 处；逐处列在上面）」。71 处裸行号只核了行号在文件范围内，不比原文；其中冻结副本里的代码行号，绝大多数我在写之前用 `grep -nF` 或 `sed -n` 对过那一行的原文；少数标「一带」的是区间起点的近似。
