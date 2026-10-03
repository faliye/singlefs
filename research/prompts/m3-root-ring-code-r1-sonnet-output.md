# m3-root-ring-code-r1 云端正推腿（Sonnet）报告：K1、K3、K4、K5

立场：逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行。方案按 `crates/` 今天的实现谈。只跑 harness 档，线程上限 15，经 `run-with-memory-cap.sh`，`--release`。
基线：冻结副本拷贝（31 份被判文件 sha256 与主 agent 快照逐份对得上；今天的主工作区只有 `crates/singlefs-checker/src/walk.rs` 与快照不同，别的会话在 4640 行之后加了约 74 行，见下「行号约定」）。
草稿、探针、脚本、原样输出在 `research/prompts/m3-root-ring-code-r1-sonnet-model/`（`README.md` 写了怎么复跑）。

## 行号约定与环境

- kb 条款行号是今天 kb 文件自己的行号，写前逐条 `grep -nF` 现查（脚本 `scripts/cl.sh`：今天主工作区与冻结副本各查一次）。
- `crates/` 行号：冻结副本与今天主工作区逐行相同，唯独 `walk.rs` 4675 行之后相差 74 行（今天 = 冻结 + 74）；本报告 `walk.rs` 一律写今天的行号，引文在冻结副本里逐字同，行号 −74。
- 开跑前 `ps` 看到别的会话的 `cargo` 在跑（pid 2122155，攻方腿编译 `opus_r3_return_probe`），不是性能测量；按定义加 `nice -n 19` 照跑。包装拒过「同一会话同时只许一条」，设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（主 agent 同时派了两条腿）。包装还有两次 250（撞自己上限）：一次编译旧树时 1400M 太小，改 2000M 重跑；一次是 6G 超峰值表 2 倍被拒，设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 重跑。这两次的输出不算结果，没有用。
- 基线 harness 档：十个相关目标（T1）10 个二进制全绿；`singlefs-checker --lib` 9 条、`singlefs-core --lib` 177 条全绿（命令：`cargo test --offline --release -p singlefs-core -p singlefs-checker --lib`，输出没有存进 `logs/`）。

## 先给结论（判定一览在末尾）

我这条腿认为可能打中的有四条，其余站住：

| 编号 | 格 | 一句话 | 证据强度 |
|---|---|---|---|
| H1 | K1/K4 | 池级 checker 不判卸载记号 4 字节位 0 之外的位，与读者、恢复收下的记录集不同：同一镜像上恢复丢那条记录，checker 把它算进重放前缀 | 盘面实测，两边读数并排 |
| H2 | K1/K5 | 「卸载记号从新根段来、不再照抄所选根」没有会红的用例：把 `recovery.rs` 改成照抄所选根，全 harness 档 122 个二进制 858 条测试全绿 | 变异实测 + 一条能红的补洞用例草稿 |
| H3 | K1 | 写行那次发布的记录新根段里「实例表指针、分配记录树根」两项：没有变异行、没有观察者；改坏一项全 harness 档 122 个二进制 856 条全绿（只改分配记录树根一项的跑了十个相关目标） | 变异实测 |
| H4 | K4 | 「写行之后第一个文件版本、崩在记录已持久根槽没持久」那一形上，池级 checker 分不出新根段里分配记录树根的对错：I-3.1 不适用，I-4.1 / I-3.10 在对的记录与错的记录上读数相同，下一次可写挂载把它施加之后仍相同 | 盘面实测 |

另有写回与欠账类的线索（不算打中）：kb 还写取甲、卸载记号其余位那一格条款没写、`format-const` 的 `stale=` 没登记 311 派生串、E142 干跑模型仍是 311 头；见 K5 与「线索」。
K3（R2 谓词与三处取用）、K5（两个钉值的独立复算）在我这条腿上站住。

## K1　新根段三字段与恢复施加（A）

### K1-1　七项各自取哪里、三处装记录写的是不是这次写的那条根上的

| 推导 | 位置 |
|---|---|
| 条款：施加 = 换七个字段，三项不再照抄所选根 | 「施加一条记录 = 把所选根的这七个字段换成记录里的（实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根）」（`.claude/kb/decisions/23-journal的角色与格式.md:450`）；判决「施加处只有卸载记号、实例表指针、分配记录树根三项不从新根段来」（`research/prompts/m3-ring-r3-main-verification.md:23`） |
| 代码：恢复拼根只有 fsid 照所选根，其余取记录 | 「filesystem_identifier: root.filesystem_identifier,」（`crates/singlefs-core/src/recovery.rs:3581`）；「unmount_marker: record.new_unmount_marker,」（`crates/singlefs-core/src/recovery.rs:3582`）；「instance_table: record.new_instance_table,」（`crates/singlefs-core/src/recovery.rs:3588`）；「allocation_record_tree_root: record.new_allocation_record_tree_root,」（`crates/singlefs-core/src/recovery.rs:3590`）。取的是 `records_in_key_order[index]`（`crates/singlefs-core/src/recovery.rs:3576`），与改之前同一个选法 |
| 代码：三处装记录都先造 `root`、再用 `root.` 的字段装七项、落盘也用同一个 `root` | 零单元发布：`let root = RootRecord {`（`crates/singlefs-core/src/transaction.rs:1364`）…「new_instance_table: root.instance_table,」（`crates/singlefs-core/src/transaction.rs:1397`）…「root_slot: root.to_slot(pool.root_slot_bytes()),」（`crates/singlefs-core/src/transaction.rs:1412`）；写行发布：`let root = RootRecord {`（`crates/singlefs-core/src/transaction.rs:1877`）…第 1928–1930 行同三项…第 1970 行同一个 `root` 落盘；带文件发布：`let root = RootRecord {`（`crates/singlefs-core/src/transaction.rs:8582`）…第 8639–8641 行…第 8660 行 |
| 全仓只有这三处造生产记录 | `grep -n 'JournalRecord {' crates/singlefs-core/src/transaction.rs` 里结构体字面量只有 1379、1906、8620 三处（别的命中是 `WrittenJournalRecord {`；core 之外的字面量在测试与 checker 档装置里）；缺一项编译不过 |

判定：一致。三处里 `root` 在造记录之前定下、之后没有再改（读过 1364–1412、1877–1970、8582–8660 全段）。

### K1-2　头宽、偏移、点名项容量与 kb 标记

- kb 标记五个逐个对：脚本 `scripts/kb_markers_check.py` 把 kb 里 `format-const: JOURNAL_*` 与 `crates/singlefs-format/src/lib.rs` 的 `pub const` 比，输出在 `logs/kb_markers_check.txt`，D23 里的五行：
  `JOURNAL_HEADER_BYTES kb=487 code=487 OK`（kb 行 160）、`JOURNAL_HEADER_TEN_FIELD_BYTES 78/78 OK`（161）、`JOURNAL_NAMED_ENTRIES_PER_RECORD 64/64 OK`（319）、`JOURNAL_NEW_ROOT_SEGMENT_BYTES 364/364 OK`（450）、`JOURNAL_NAMED_ENTRY_BYTES 56/56 OK`（481）。
  同一条命令还报了 `experiments-history.md`、`decisions-history.md`、`experiments/43-…md` 里的 307、277 与 487 不等：那是历史节与变更史，不在门禁射程，不算。
- 偏移手算（不经 core、不经 checker）：`scripts/hand_decode.py` 自写 CRC-32C（先过公开校验值），按 kb 字段表解 `build_pool` 盘上三条记录与三条根槽。条款偏移「新根段 99、fsid 463、MAC 471、头末 487」（`.claude/kb/decisions/23-journal的角色与格式.md:154`），99 + 364 = 463、463 + 8 + 16 = 487，脚本里是断言；容量 ⌊(4096 − 487) / 56⌋ = 64（脚本打印 `named entries per 4096 record: 64`，kb 159 行写 64 个点名项、余 3609）。
  原样输出（`logs/hand_decode.txt`）：三条记录的 `header_csum`、`payload_csum` 都复算对上；每条记录新根段的树表指针、映射根指针、树 ID 水位、F、实例表指针、分配记录树根六段与同 txg 根槽 [36:122]、[256:342]、[122:130]、[130:138]、[170:256]、[342:428] 逐字节相同（18 个 True），卸载记号与根 flags 相同，记录 fsid 8 字节 = 根 fsid 前 8 字节。
- checker 自己的偏移是字面量不是常量推出来的：「const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;」（`crates/singlefs-checker/src/lib.rs:708`）、「const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: usize = 463;」（`crates/singlefs-checker/src/lib.rs:709`）。
- 写者与读者：「writer.put_u32(self.new_unmount_marker.flags_bits());」（`crates/singlefs-core/src/journal.rs:252`）；写完有 `assert_eq!` 核新根段 364；读完核头 487。

判定：一致。推翻它的现象：kb 五个标记之一与 `lib.rs` 不等，或手解出来七段里有一段与同代根的同名段不等；两样都没出现。

### K1-3　卸载记号「位 0 之外有位为 1 当记录损坏」在读者、恢复、池级 checker 三处是不是同一个口径

依据：这一格条款今天没写。kb D23 已定项 4 只写记录标志位的「其余位写 0，读到非 0 当损坏」（`.claude/kb/decisions/23-journal的角色与格式.md:152`），D22 已定项 7 写根 flags「其余位第一版恒 0、非 0 拒收」（`.claude/kb/decisions/22-单元原子性怎么合成.md:137`）；这一格的来源是用户 2026-10-03 弹窗：「卸载记号位 0 之外非 0 当记录损坏」（`records/2026-09-30-里程碑三开工第一轮.md:455`）。实现员在代码里也自报「条款没写这一格」（`crates/singlefs-core/src/journal.rs:279`）。

- 读者：「UnmountMarker::of_unmount_marker_field_of_a_new_root_segment(reader.get_u32())?;」（`crates/singlefs-core/src/journal.rs:346`），判在 `if field & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {`（`crates/singlefs-core/src/root_record.rs:44`），位 0 之外有位 ⇒ `None` ⇒ `parse` 交 `None`。单测三个值 0x2、0x3、0x8000_0000（`crates/singlefs-core/src/journal.rs:533`），变异行 2158 钉它（`crates/mutations.tsv:2158`，下文 K1-5 列表）。
- 恢复：扫环只收 `parse` 交得出的记录：「if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {」（`crates/singlefs-core/src/recovery.rs:3348`）。与读者同一个口径（同一个函数）。
- 池级 checker：不同。`check_journal_record` 对新根段只原样切出来：「record[JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET].to_vec();」（`crates/singlefs-checker/src/lib.rs:828`），函数里没有一行读 [459, 463)；对记录标志其余位它明说不拒、留给 I-8.9 判红：「不在这里拒其余位非 0 的记录，I-8.9」（`crates/singlefs-checker/src/lib.rs:750`），而卸载记号那 4 字节今天没有任何不变量判它（`.claude/kb/invariants.md` 的 I-8.10 只比四段，`crates/singlefs-checker/src/walk.rs:6180` 的文档写「四段」）。
  盘面实测（`sonnet_probe_k1_k4.rs` 的 `probe_marker_bits_core_set_vs_checker_prefix`，`logs/run5.txt`）：新池新建文件盘面，txg 3 的根槽撕掉、记录 3 留着，把记录 3 新根段里卸载记号字段改成某个值、树表指针的整单元校验和翻一位、重封头校验和；两块盘一致。原样输出：
  - `marker=1`：`core scan keys=[(1, 1), (1, 2), (1, 3)] prefix_applied=1`；checker `I-4.1: Violated("所选根（实例 1、txg 2）之上重放 1 次发布、1 条记录（末条计数器 3）施加出的那一版走不完：…")`
  - `marker=2`：`core scan keys=[(1, 1), (1, 2)] prefix_applied=0`；checker 同一句 `I-4.1: Violated(… 重放 1 次发布、1 条记录（末条计数器 3）…)`
  - `marker=0x80000000`：同 `marker=2`。
  即同一镜像上恢复把记录 3 当损坏（扫环键里没有 (1, 3)、施加 0 条），checker 把它算进重放前缀。

H1 的推翻条件：在同一盘面上 checker 的重放前缀也不含记录 3（读数里「重放 1 次发布、1 条记录」变成 0），或条款明写 checker 只判「同代根已有的四段」、不判卸载记号。
注意它不是代码把条款做错：读者与恢复同口径、checker 是另一份独立解析（D13 已定项 5：「checker 与实现之间只共享一样东西」，`.claude/kb/decisions/13-验证路线.md:138`）；差在 checker 没有对应判据。

### K1-4　首个文件、卸载、实例表指针三条用例钉的是不是条款说的那一形

- 首个文件：写行之后（分配记录树根住根记录、不空）发第一个文件版本，它的记录落了、根槽没落；施加出来的根与当初写的根整条相等：「read_only.effective_root, first_file.root,」（`crates/singlefs-harness/tests/rebuilt_root_takes_every_new_root_segment_field_from_the_record.rs:107`）。形对（判决「首个文件、卸载两种补洞」那一形），七项含 fsid 都被整条相等钉住。变异行 2160（分配记录树根照抄所选根）、2162（记录里写成实例表指针）钉它，见下表。
- 卸载：卸载那一串第二次空发布（txg 8）根槽没落、记录落了：「tear_the_root_slot_written_for_txg(&mut image, 8);」（同文件 161），比较「read_only.effective_root, second_unmount_publish.root,」（同文件 176）。**形只对一半**：所选根是 txg 7 的根，它自己已带卸载记号；记录 8 的记号也是「带」。「取记录里的」与「照抄所选根」在这盘面上给同一个值，用例分不出（H2）。条款要的是「实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根」（`.claude/kb/decisions/23-journal的角色与格式.md:450`）。
- 实例表指针：用例自己说今天没有一种被施加的发布改它：「实例表指针那一项今天没有一种被施加的发布改它」（`crates/singlefs-harness/tests/rebuilt_root_takes_every_new_root_segment_field_from_the_record.rs:10`）；盘面是改出来的（新根段里的实例表指针换成校验和对不上的落点、重封头）。它钉的是「施加取记录里的」，不是判决那一形（专用清扫发布，今天 `crates/` 里没有）；这是 7-2 自己写明的，用户 2026-10-02 定「回到甲」之后条款只要求取法，不要求有这种发布。

### K1-5　钉它们的变异行与我自己补跑的变异

`crates/mutations.tsv` 里 7-2 加的行：2157 到 2175 与 K1/K4 有关的是 2157–2163、2172–2175。逐项推到测试：

| 改动点 | 变异行 | 它点名的测试 | 我的补跑 |
|---|---|---|---|
| 记录写者新根段不写实例表指针 | 「根环补洞甲（D23 已定项 15，用户 2026-10-02「回到甲」）：journal 记录新根段不写实例表指针（写全零）」（`crates/mutations.tsv:2157`） | `journal.rs` 单测 round_trip | 没补跑 |
| 读者不判卸载记号其余位 | 「根环补洞甲：读者不判新根段卸载记号那 4 字节的其余位（位 0 之外非 0 的记录照收）」（`crates/mutations.tsv:2158`） | `journal.rs:531` 单测 | 没补跑 |
| 恢复卸载记号不取记录（回到恒不带） | 「根环补洞甲：由记录重建的根的卸载记号不取新根段（回到「由记录施加的一律不带」）」（`crates/mutations.tsv:2159`） | 卸载用例（`...from_the_record.rs:115`） | 没补跑 |
| 恢复分配记录树根照抄所选根 | 「根环补洞甲：由记录重建的根的分配记录树根照抄所选根（首个文件那一版施加出来还指着写行那棵旧分配记录树）」（`crates/mutations.tsv:2160`） | 首个文件用例 | 没补跑 |
| 恢复实例表指针照抄所选根 | 「根环补洞甲：由记录重建的根的实例表指针照抄所选根（专用清扫发布崩在记录已持久、根槽没持久那一形施加出旧链）」（`crates/mutations.tsv:2161`） | 实例表用例 | 没补跑 |
| 带文件发布记录里分配记录树根 | 「根环补洞甲：带文件的发布的记录新根段里分配记录树根不写这次的根上的（写成实例表指针）」（`crates/mutations.tsv:2162`） | 首个文件用例 | 没补跑 |
| 带文件发布记录里卸载记号恒不带 | 「根环补洞甲：带文件的发布（卸载那一串走这一路）的记录新根段里卸载记号恒写「不是卸载那一串写的」」（`crates/mutations.tsv:2163`） | 卸载用例 | 没补跑 |

变异表里**没有**的点，我补跑了（`scripts/mutant.sh`，每次改一行、核锚点、跑、还原；日志在 `logs/`）：

| 补跑变异 | 位置 | 结果 |
|---|---|---|
| M-C：带文件发布记录的 `new_instance_table` 写成 `root.mapping_root` | `crates/singlefs-core/src/transaction.rs:8639` | 红：`checker_known_bad_images` 一个二进制 69 条里 21 条红（cargo 在第一个红的二进制后停，其余目标没跑，`logs/mut-M-C.log`） ⇒ 这一点有观察者，钉得住 |
| M-D：零单元发布记录的 `new_unmount_marker` 写成「带」 | `crates/singlefs-core/src/transaction.rs:1399` | 红：2 条（`checker_known_bad_images` 的 `each_release_generation_…`、`rewriting_an_older_root_rechains_…`） ⇒ 钉得住（走记录与根逐字节比较） |
| M-A：写行发布记录的 `new_instance_table` 写成 `previous_root.instance_table` | `crates/singlefs-core/src/transaction.rs:1928` | **存活**：全 harness 档 122 个二进制、856 条测试 0 失败（含草稿探针 5 条，非探针 851 条；`logs/mutants-full-suite.txt`） |
| M-B：写行发布记录的 `new_allocation_record_tree_root` 写成空指针 | `crates/singlefs-core/src/transaction.rs:1929` | **存活**：T1 十个目标 0 失败（全套没跑） |
| M-F：恢复的卸载记号写成 `root.unmount_marker`（照抄所选根） | `crates/singlefs-core/src/recovery.rs:3582` | **存活**：全 harness 档 122 个二进制、858 条测试 0 失败（含草稿探针 7 条，非探针 851 条；`logs/mutants-full-suite.txt`） |

H3 为什么没有观察者：写行记录是新实例的第一次发布，恢复施加到它之前就因实例代号不同而停（「下一条的实例代号与所选根不同即停」，`.claude/kb/decisions/23-journal的角色与格式.md:369`），用例头也这么写：「改它的写行是新实例的第一次发布、从不施加」（`crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs:5`）；池级 checker 的 I-8.10 今天只比四段（扩到七段进下一批，不算本轮）。所以这两项今天写错没有任何东西会红；推翻 H3 的现象：有一条今天就存在的测试或不变量在 M-A、M-B 下变红——全套已跑 M-A，没红。
H2 的补洞用例（草稿，`sonnet_probe_k1_k4.rs` 的 `probe_unmount_marker_distinguishes_record_from_chosen_root`）：两次卸载空发布（txg 7、8）的根槽都撕掉、两条记录留着，所选根是 txg 6（不带记号），施加两条。原样输出：冻结副本上 `chosen txg=6 marker=NotWrittenByTheUnmountSequence; effective txg=8 marker=WrittenByTheUnmountSequence; prefix_applied=2`、`test result: ok`（`logs/run9-frozen.txt`）；M-F 下断言红，left 的 `unmount_marker: NotWrittenByTheUnmountSequence`、right 的 `WrittenByTheUnmountSequence`（`logs/run9-mutF.txt`）。H2 的推翻条件：M-F 下已有测试变红（全套 0 失败，没有）。

## K3　系统配置 R2（C）

### K3-1　R2 的谓词与定案、D22 已定项 8 射程逐字相同吗

条款：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。
代码：「.any(|slots| slots.slots_reporting_a_read_error > 0)」（`crates/singlefs-core/src/recovery.rs:3308`）与非「.any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)」（`crates/singlefs-core/src/recovery.rs:3311`）按 `&&` 连起来（3303 起的整个函数）；`both_slots_self_verified` 数的是自证过且 fsid 与本池相同的槽数是否等于每盘槽数（`recovery.rs:3261`）。判定：一致。

### K3-2　撕裂、fsid 不符、全零、报读错各落哪一边

- 报读错只认 `PoolReader::read` 交空：「None => slots_reporting_a_read_error += 1,」（`crates/singlefs-core/src/recovery.rs:3279`）。
- 读得出（`Some`）的槽：只有「if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {」（`crates/singlefs-core/src/recovery.rs:3281`）且「== *filesystem_identifier」（`crates/singlefs-core/src/recovery.rs:3283`）才进 `self_verified`，别的（撕裂、全零、fsid 不符）既不记读错也不进自证：落在「读得出、没自证」那一边，不触发判不出的前一半。
- 钉它们的用例：撕裂（两盘最新槽清零、读得出）⇒ 照常可写：`a_writable_mount_goes_ahead_over_a_torn_system_configuration_rotation_on_every_device`（`crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs:163`，变异行「系统配置见证读取 R2：撕裂的轮换（读得出、自证不过）也算报读错（回到甲）」`crates/mutations.tsv:2171`）；全零与读错并存 ⇒ 判不出、拒：`...reads_back_zeros`（同文件 139，变异行 2169）；一盘读错、另一盘两槽都自证 ⇒ 可写：同文件 120（变异行「系统配置见证读取 R2：判不出的后一半拿掉（有槽报读错就判不出，回到甲-读错）」`crates/mutations.tsv:2167`）。
- fsid 不符：**没有专钉它落哪一边的用例**（`tests/` 里与「另一个池」有关的只有整块盘换掉的几份，不碰单槽 fsid）；条款只写「自证过」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`），没说一个校验和对、fsid 不符的槽算不算自证；代码按「不算」。条款没说；只有一个池里混进别的池的槽才走得到，合法历史走不到（mkfs 先把 [0, 根环基址) 清零）。

### K3-3　三处是不是同一处定义；有没有留着取甲

- 读阶段：「let comparison = if system_configuration_witness_is_undecidable(&slots_of_each_device) {」（`crates/singlefs-core/src/mount.rs:4996`），槽读法是 `newer_publish_witness` 里按盘读一次（`mount.rs:4965`）。
- 取号：「if system_configuration_witness_is_undecidable(&slots_of_every_device) {」（`crates/singlefs-core/src/transaction.rs:705`）。
- 两处共用 `recovery.rs:3303` 一个函数，读槽共用 `system_configuration_slots_of_one_device_read_once`。取甲的「某盘自证过的槽少于每盘槽数就按真」在 `crates/` 里已没有：`grep -n SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE crates/singlefs-core/src/*.rs` 只剩 mkfs 写槽、轮换槽号、与 `both_slots_self_verified`（`recovery.rs:3261`）三类用法。
- 例外是故意留的：逐盘核「一份自证过的都没有」：「let Some(newest_self_verified) = newest_self_verified else {」（`crates/singlefs-core/src/mount.rs:3487`）与取号里的「if slots_of_this_device.self_verified.is_empty() {」（`crates/singlefs-core/src/transaction.rs:720`），各有自己的拒因成员（`mount.rs:4253` 起），不并进判不出。它在 D18 里有条款落点：可写的前提之一是「独占打开成功且系统配置读得通」（`.claude/kb/decisions/18-块里携带什么信息.md:316`）；「留着」本身是用户 2026-10-03 弹窗：「R2 之下「某块盘一份自证过的都没有就拒」留着，写回 D23 已定项 14 / D22 已定项 8 射程」（`records/2026-09-30-里程碑三开工第一轮.md:455`）。
- 写回落后：D23 已定项 14 那一段今天还写着「这里的实现还是 2026-09-28 取甲的那一版」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。代码与用户定案是 R2，kb 这一句是过期的；那一段「留着」的拒也还没写进 kb（上面 `records/…:455` 说要写回）。不是代码错，是写回欠账。

### K3-4　R2 加留着的拒，有没有一段合法历史让池永远只读而条款不要

我找到的：
1. 一盘一槽持续读错、另一盘最新一槽被普通断电撕裂（读得出、不自证）：没有一块盘两槽都自证，且有读错 ⇒ 判不出 ⇒ 每次可写挂载都拒、那一槽没有机会被重写。这就是用例 139 钉的「混合形」，用户在 R2 里接受了它的代价（D23 已定项 14 射程里明写「池只剩只读挂载」（`.claude/kb/decisions/23-journal的角色与格式.md:408`），同一行还写了「一次可写挂载都做不成时怎么走出来，没有条款」（同行））。所以条款不是「不要」，是「要，且欠一条走出来的路」。
2. 一块盘两槽都读不出、另一块两槽都自证：R2 判得出，被留着的拒挡住，永远只读。需要一块盘整个系统配置区两槽都坏：撕裂造不出（槽按世代轮换，同一次写只撕一槽），只有一块盘的系统配置区整体失效。D2「降级期间只读」之下这一形本来就只读。
没找到的：单故障（一块盘的一槽读错）加普通崩溃的任何历史让池永远只读——这一形用例 120 钉了可写。推翻上面「没找到」：一条只含一次单槽故障与普通崩溃的历史走到「每次可写挂载都拒」。

### K3-5　改钉的两条判拒用例，判的还是原来那两件事吗

- 合法的那条：由「撕了盘 0 一槽、拒可写在取号之前」改成「两盘最新一槽都报读错」，函数名 `a_refusal_when_the_newest_system_configuration_slot_of_every_device_reports_a_read_error_is_legal`（`crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs:179`）；仍判「拒因在 D23 已定项 14 单子上 ∧ 那条判据在镜像上复算为真 ∧ 盘逐字节不变 ∧ 只读读出在允许集合里」，且新增断言钉拒因是 R2 的判不出。
- 非法的那条：同一个盘面、把「拒之后盘变了」交给判法 ⇒ 判非法，只差「盘被改了」一条：`..._that_changed_the_disk_is_illegal`（同文件 278）。
- 判法本身（`crates/singlefs-harness/src/mount_transparency.rs` 的 `newer_publish_witnessed_on_the_crash_image`，1320 行起）仍写「某盘自证过的槽少于每盘槽数就按真」（1337–1341 行），不是 R2；主 agent 第二节已列「仍按取甲判」。它对改钉后的盘面（两盘最新槽都读错、每盘只剩一份自证）与 R2 给同一个值，所以这两条用例今天判得对；它放得过的是「R2 不该拒而取甲会拒」的盘面（撕裂轮换），harness 判法分不出多拒——已知没做，不算打中。
判定：判的还是原来那两件事。推翻它：用同一个判法对一张 R2 下不该拒的盘面（撕裂的轮换）交一次拒，它判合法——会判合法，但那是已知没做的那一格。

## K4　池级 checker 跟上（D）

### K4-1　「写行之后第一个文件版本、崩在记录已持久根槽没持久」那一形上的读数

盘面（`sonnet_probe_k1_k4.rs` 的 `probe_first_file_after_a_row_record_only`、`probe_first_file_after_row_wrong_alloc_root_in_record`、`probe_next_mount_after_first_file_after_row_record_only`；原样输出在 `logs/run2.txt`、`logs/run8.txt`、`logs/run10.txt`）：mkfs → 第一次可写挂载（取号 1、暖机）→ 第二次可写挂载（取号 2、写行、暖机，分配记录树根写进根记录、不空）→ 同一次挂载里发第一个文件版本（txg 5，它的根分配记录树根是空的）→ 把 txg 5 的根槽撕掉、记录留着。

| 读数 | 实现 | 池级 checker |
|---|---|---|
| 整份盘面（根槽没撕） | `effective_root txg=5 instance=2 above_water=0` | 违例 0 条 |
| 根槽撕掉（记录单落） | `effective_root txg=5 instance=2 above_water=1 prefix_applied=1`（与 `first_file.root` 整条相等，见 K1-4） | 违例 0 条；I-3.1 报 `NotApplicable("最新的根下面还没有记账树（第 0 代树表）")`，I-4.1 `Holds`，I-3.10 `Holds` |
| 再做一次可写挂载把它施加、写行之后 | 可写挂载成功 | I-3.1、I-3.9、I-3.10、I-7.4、I-9.16、I-8.10 全 `Holds`；I-4.1 `NotApplicable`（所选根之上没有要施加的了） |

所以这一形上两边**没有分歧**：实现挂得上，checker 零违例。但这个读数里 I-3.1 在单落那一刻是「不适用」：checker 只有在最新根的实例表里已经有这个实例的行时才把「由记录施加、根槽没落」的那一版当作施加过（「row.instance == record.instance」`crates/singlefs-checker/src/walk.rs:6082`，「&& record.checkpoint_txg <= row.published_checkpoint_txg」`crates/singlefs-checker/src/walk.rs:6083`），记录单落、之后没有任何一次挂载时那一行还不存在，I-3.1 的统计也只看根环里最新根（它是写行那次的暖机根，树表 0 条）。

H4：同一盘面上把记录 5 新根段里的分配记录树根改成「写行那一版的、非空的」指针（错的；条款：「mkfs 的第 0 代与带文件的一版写全 0」（`.claude/kb/decisions/22-单元原子性怎么合成.md:146`）），重封头与后续反向链。原样输出（`logs/run8.txt`）：
- `right record (alloc root empty): ["I-3.1=NotApplicable(…)", "I-3.10=Holds", "I-4.1=Holds"]`
- `WRONG record (alloc root = previous row version's non-empty root): ["I-3.1=NotApplicable(…)", "I-3.10=Holds", "I-4.1=Holds"]`
- 再做一次可写挂载之后（`logs/run10.txt`）：`RIGHT record` 与 `WRONG alloc root in record` 两份的读数逐项相同（上表第三行），可写挂载也都成功。
所以池级 checker 在这一形上区分不出记录新根段里分配记录树根的对错；实现一侧也没有读数告诉它错了；只有 core 的用例（首个文件用例，变异 2160、2162）判得出。判定：条款（D23 已定项 15「七个字段换成记录里的」）今天在 checker 上对这一项没有会红的判据，I-8.10 扩到七段（用户 2026-10-03 定、进下一批）也只对「同代根还在」的记录有用，「根槽从没落盘」的那一版它比不到——这一形的判据要另立。
H4 的推翻条件：在这盘面上 checker 的任一不变量对错的那份记录判红（我跑了 I-3.1、I-3.9、I-3.10、I-4.1、I-7.4、I-8.10、I-9.16，都没红），或条款明写这一项在带文件的一版上不受约束（D8 已定项 8 说带文件的一版分配记录树住树表条目，根记录那一项是冗余——如果是冗余、错了也无害，则 H4 降为「checker 没有对应判据，但无害」）。
同盘面第三个变体（记录里的实例表指针换成上一版的）无意义：这一次发布没有改实例表，前后指针相同（探针里 `first_file.root.instance_table == previous_instance_table` 为 true），不作证据。

### K4-2　(实例代号, 计数器) 键，两边是不是收下同一批记录

- 键：恢复扫环按 (实例代号, 计数器) 先到先得：「let first_copies = first_copy_of_each_record_key(&scanned_record_keys);」（`crates/singlefs-core/src/recovery.rs:3359`）；checker 「.entry((record.instance, *counter))」（`crates/singlefs-checker/src/walk.rs:5486`）、前一条按「.get(&(record.instance, previous_counter))」（`crates/singlefs-checker/src/walk.rs:5509`）。盘次序都是各自读者给的盘次序（核心 `reader.device_identities()`，checker `reader.devices()`）；合法历史里同键的两份镜像逐字节相同，次序不同不会改变收下的内容。
- 测试：核心 `journal_scan_keeps_the_first_copy_of_a_duplicated_record_key.rs`（1 条用例）；checker 两条单测，变异行 2172（「池级 checker 跟上（I-4.1 那一行状态句）：恢复眼里的环按计数器建键…」`crates/mutations.tsv:2172`）与 2173。两条单测造的是 `ScannedJournalRecord`，不是同一份盘面上让核心与 checker 各读一遍的对拍；没有这样的对拍用例（`tests/` 里没找到同时调 `scan_journal` 与 `check_pool_image` 比键的）。
- 收下的集合不同的两处：
  1. H1（K1-3）：读者拒、checker 收的记录。
  2. checker 每盘的表按计数器一个键：`records.insert(` 后第一个参数是「record.counter,」（`crates/singlefs-checker/src/walk.rs:5387`）。同一块盘上两条不同实例、同计数器的记录会互相挤掉（后读到的槽赢），而恢复两条都留。写者造不出这个盘面（同一计数器落同一个槽，一个槽一条记录）；只有挪了位的记录能造出来。这一条我没造盘面，读代码得的，不当打中。
- 共用代码：没有。`crates/singlefs-checker/Cargo.toml` 的依赖只有「singlefs-format = { path = "../singlefs-format" }」（`crates/singlefs-checker/Cargo.toml:9`），与 D13 已定项 5「只共享一样东西」同（常量模块）。窗口的实现 checker 自己一份（`OneRevolutionWindow`，K2 不是我的格）。

判定：键一致；收下的集合差 H1 一处。

## K5　改钉（E）

### K5-1　两个钉值能不能不经 core 独立算出

- 反向链钉值 2 660 362 816：
  - 用例自己就有第二条路：同一个循环里 checker 自己的逐位 CRC-32C 对盘上前一条记录的 487 字节头现算：「.map_or(0, back_chain_of_record_header);」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:530`），再与读出的反向链比；钉值在「view.back_chain, E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION,」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:566`），常量在「const E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION: u32 = 2_660_362_816;」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:80`）。说明里「另由池级 checker 自己那份逐位 CRC-32C … 现算一遍再比，两边对得上，不是只抄实现的数」与实情相符。
  - 我的第三条路（`scripts/hand_decode.py`，自写 CRC-32C，不经 core 也不经 checker）：记录 3 盘上反向链 2660362816，CRC32C(记录 2 的 487 字节头，头校验和 32 字节按零) = 2660362816，`equal=True`；记录 2 的反向链 750278362 = CRC32C(记录 1 的 487 字节头) 同样相等。对照：把记录 2 当 311 字节头算得 295996559，不是旧钉值 2213468031（字节已变，对照本就不该等）。原样输出在 `logs/hand_decode.txt`。
  - 局限：三条路读的都是 core 写出来的字节；独立的是 CRC 与字段位置，不是「字节该是什么」。字段位置的独立性由 K1-2 手解对 kb 偏移表保证（18 个 True）。研究装置那一路今天没有：E142 干跑模型 `research/e7-index-bench/src/bin/e142_new_pool_file_creation_dry_run.rs` 里还是「const JOURNAL_HEADER_BYTES: u64 = 311;」（该文件 102 行）与「const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 188;」（该文件 106 行），用例说明里「E142 装置按新头宽重跑之后要逐字对上」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:79`）是一笔还没还的欠账，不是已做的核对。
- 默认环整盘 CRC 4 048 568 891：「const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 4_048_568_891;」（`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:309`）。摘要输入是「盘号、扇区号、扇区 512 字节」按盘、按写过的扇区升序（同文件 273 行 `crc32_of_the_images`）。我转储同一段流程的整条输入流（1172 个扇区、614128 字节），用 Python 自写 CRC-32C 算：`python crc32c over fed stream: 4048568891  pinned 4048568891 equal: True`（`logs/python-crc-and-sector-diff.txt`；转储用例的原样输出在 `logs/run7.txt`，`test-crc=4048568891`）。
- 「改之前」的值：把本轮三份补丁在冻结副本拷贝上反向应用（第 1 份在 `mount.rs` 两处冲突、手工还原，步骤在 `README.md`），同一个转储用例得 `test-crc=1733593715`，与改钉前的钉值相同；新旧两份输入流逐扇区比：1172 对 1172 个扇区里不同的恰好 10 个，盘 0 与盘 1 各 5 个，偏移 0x1000000、0x1001000、0x1002000、0x1002200、0x1002400——journal 环起点 1024 槽 × 16384 = 0x1000000，即环里第 0、1、2 个记录槽（`logs/run-old.txt`、`logs/python-crc-and-sector-diff.txt`）。与说明里「差异只落在 journal 环第 0–2 个记录槽里」相符。
判定：两个钉值站住；说明与实情相符；独立的研究装置那一路是欠账。推翻它：Python 的 CRC 或手解里任一项与钉值不等（都等）。

### K5-2　格式常量改了之后，派生形态在被判文件里有没有漏改

按 `.claude/rules/format-evolution.md`「改一个格式常量」第 1 步逐类搜（字面量、消息串、预留宽读写、下标、倍数、商、测试钉的产物值）。搜法：`grep -rnE '\b(311|3785|3789|287|295|188|67)\b'` 对 `crates/` 下 `.rs`（checker 档除外，那是主 agent 第二节列的已知没做），再逐个核命中。
- 字面量 311 / 287 / 295：被判文件里没有留着的（`journal.rs`、`lib.rs`、`walk.rs`、harness 用例全改了；`journal.rs:507-509` 的 287、188 是新布局里实例表指针的起点与 99 + 188，是对的）。
- 消息串：`expect("311")` 全改成 `expect("487")`、`expect("188")` → `expect("364")`、`expect("67")` → `expect("64")`（`crates/singlefs-core/src/journal.rs`、`transaction.rs`）。
- 倍数与商：点名项切分单测 `vec![1, 64, 64, 9]`（`crates/singlefs-core/src/transaction.rs:9220`）：2 个数据单元 + extent 根 + 130 片叶容器 + 5 个角色 = 138，最后一个事务 137 = 64 + 64 + 9 ✓；`rewritten_roles(0, 59)`（`crates/singlefs-core/src/transaction.rs:9241`）：1 + 59 + 5 = 65 = 64 + 1 ✓。
- 钉的产物值：K5-1。
- 漏改但不挡断言的（不是漏改会静默失效，只是名字与注释说旧值）：
  - `sequential_write_one_record_per_data_unit.rs`：用例名 `fn sequential_write_of_67_data_units_at_the_named_item_limit_publishes_67_records() {`（`crates/singlefs-harness/tests/sequential_write_one_record_per_data_unit.rs:167`）与「/// 67 是一条 journal 记录的点名项上限。」（同文件 165）。它断言的是记录条数 = 数据单元数、末条之前每条只点名一个数据单元，都不依赖 64 或 67；67 现在越过上限 64，仍在「越过上限」那一侧，不是「正好在上限」；断言没失效，用例名在说谎。
  - `write_request_split.rs`：「for expected_unit_count in [66u64, 67, 68, 143, 144, 145] {」（`crates/singlefs-core/src/write_request_split.rs:208`）：测的是切分覆盖每个文件字节恰一次，与记录容量无关；门槛列表停在旧上限上，新上限 64/65 不在列表里，但这一函数不读记录容量，不丢判别力。64/65 那一侧的边界由 `transaction.rs:9220`、`9241` 的单测钉着。
  - `multi_unit_file_write_and_read.rs:199`、`multi_record_publishes_of_transaction_zero.rs:5-6`：注释里写 67/68，断言不依赖。
  - 变异表行名里的 311：「记录头 311：池级 checker 的新根段偏移没跟着后挪 4 字节」（`crates/mutations.tsv:452`）、「一条记录装的点名项写成 66（不等于 (4096 − 311) ÷ 56 的整数部分）」（`crates/mutations.tsv:1074`）：行名，原文与替换文（`= 99` → `= 95`、`= 64` → `= 63`）是对的。
- kb 一侧（不在被判文件里，写回欠账）：D23 已定项 4 的标题还写「头 311 字节」（第 148 行：`#### 已定项 4：记录头约束在单个原子单元内，头 311 字节`）；I-8.6 一行还写「记录的 311 字节头」（`.claude/kb/invariants.md:95`）；`JOURNAL_HEADER_BYTES` 的标记 `stale=` 里登记的是 277…307 一路的旧串，没有 311 的任何派生串（`.claude/kb/decisions/23-journal的角色与格式.md:160` 那一行），第 2 步没做，门禁今后也盯不住它们。
判定：被判文件里没有静默失效的派生形态；有三处名字或注释说旧值（上面前两条），kb 与研究装置各有一笔没还的旧值。推翻它：按同一条 grep 在被判 31 份文件里找到一条断言依赖 67 或 311 的（我没找到）。

## 线索（不算打中，供主 agent 与攻方腿对照）

- L1　`e165_replay_bounded_enumeration.rs` 给每条生成的记录的新根段后三项一律带 r1 档那一份常量，文件自己写明：「要重新登记（实现员 2026-10-03 只为让装置照新的记录格式编得过，登记与判据一个字没改）」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:383`）；常量来自「fn fields_beyond_the_projection_carried_by_every_generated_record() -> FieldsBeyondTheProjection {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:384`）。r2、r3 档里「投影之外抄所选根」的 J7 判据与新施加规则对不上，是 checker 档的事，我没跑（重型）。主 agent 第二节列了「checker 档里硬写旧格式的几处」，这一处是同一族但不在那张清单里。
- L2　K4-2 第 2 点的每盘按计数器一个键，写者造不出，读代码得的。
- L3　`mount.rs` 里 `refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage` 在逐盘核拒时拼出一个「重读过」的读数（`first_read` 与 `reread` 同一份见证值），实际没有重读；只影响拒因里的字段，不影响拒不拒。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 七项取值、三处装记录 | 一致 | 恢复只有 fsid 照所选根；三处都是先造根、再用根的字段装七项、落盘同一个根 |
| K1-2 头宽、偏移、容量、kb 标记 | 一致 | 五个 kb 标记全等；手解盘面的七段与同代根逐段相同；64 = ⌊3609 / 56⌋ |
| K1-3 卸载记号其余位口径 | 读者 = 恢复一致；checker 冲突（H1）；条款没写 | 同一盘面恢复丢记录 3、checker 把它算进前缀；D23 / D22 今天都没写这一格，来源是用户 2026-10-03 弹窗 |
| K1-4 三条用例的形 | 首个文件一致；卸载只钉一半（H2）；实例表指针钉的是取法 | 卸载用例的所选根已带记号，「取记录」与「照抄所选根」分不开；实例表指针那一形今天没有这种发布 |
| K1-5 变异行覆盖 | 点名的行对；缺两处（H2、H3） | M-F（照抄所选根）、M-A（写行记录实例表指针）全 harness 档存活；M-B 在十个相关目标上存活 |
| K3-1 R2 谓词 | 一致 | 与 D22 已定项 8 射程逐字相同，代码 `recovery.rs:3303` 起 |
| K3-2 读错、撕裂、全零、fsid 不符 | 读错 / 撕裂 / 全零一致；fsid 不符：规则没说 | 读错只认 `read` 交空；fsid 不符落在「没自证」一侧，无用例、条款只写「自证过」 |
| K3-3 三处同一定义 | 一致；kb 写回冲突 | 读阶段与取号共用一个函数，取甲写法已无；kb D23 已定项 14 还写取甲 |
| K3-4 永远只读 | 单故障没有；两类双故障有（已知代价） | 一槽读错 + 另一盘普通撕裂（用例 139）与一盘两槽都坏，条款认下并欠走出来的路 |
| K3-5 改钉两条判拒用例 | 一致 | 判的仍是「拒因合法」与「拒后盘变了判非法」；判法本身仍取甲（已知没做） |
| K4-1 K4 那一形的读数 | 无分歧；区分度缺（H4） | 实现与 checker 都不红；checker 分不出记录里分配记录树根的对错，I-3.1 不适用 |
| K4-2 键与收下的集合 | 键一致；集合差 H1 | (实例代号, 计数器) 两边同；读者拒的记录 checker 收 |
| K4-3 共用代码 | 没有 | checker 只依赖 `singlefs-format` |
| K5-1 两个钉值 | 一致 | Python 自写 CRC 复算两个钉值与旧值，新旧盘面逐扇区只差 journal 环 10 个扇区；E142 干跑模型仍是 311，欠重跑 |
| K5-2 派生形态 | 被判文件里没有静默失效 | 67 只留在用例名与注释里；kb 标题、I-8.6、`stale=` 各有旧值未登记 |

## 要主 agent 跑 / 复核不了

- 没有要重型命令的活。复核 L1 要跑 checker 档（e165 的 r2、r3 档 J7），按定义我不跑。
- 复核不了：D22 已定项 8 射程里「用户定，原话在变更史」那几句原话我没读（只读了条款与调度记录里的转述）；用户 2026-10-03 弹窗的原话同样只有调度记录那一句转述（`records/2026-09-30-里程碑三开工第一轮.md:455`）。
- 变异补跑的环境：别的会话的 `cargo` 同时在跑（开跑前 `ps` 看到 pid 2122155，攻方腿），我加了 `nice -n 19`；包装的两次 250 在开头「行号约定与环境」里写了，输出没用。

## 没做什么

- K2 不是我的格，没看一圈窗口的四处同改与边界；K2 那五处没造盘面。
- 没跑 checker 档任何东西（e165、e170、crash 放量），没跑 crates 变异整表，没跑 fmt、clippy、`gate.sh`。
- 没造 K4-2 第 2 点的盘面；M-B 没跑全套（只跑了十个相关目标）。
- 没替主 agent 采纳或出判决；上面「H1–H4」只是我这条腿认为可能打中的候选，坐实与改法归主 agent。
- 没写 `crates/`；探针与脚本在模型目录。

## 草稿目录里留着的东西

交回前已删：`/tmp/claude-1000/m3-root-ring-code-r1-sonnet/work/`（冻结副本拷贝加探针，含 13G 编译目录）与 `/tmp/claude-1000/m3-root-ring-code-r1-sonnet/old/`（三份补丁反向应用后的树，含编译目录）；探针、脚本与原样输出已进 `research/prompts/m3-root-ring-code-r1-sonnet-model/`，复跑照 `README.md`（从冻结副本重新拷）。
`/tmp/claude-1000/m3-root-ring-code-r1-sonnet/` 里还留着 2.1M：`dump/`（盘面转储，`hand_decode.py` 与 Python 复算读它）、`patch1.patch` 到 `patch3.patch`（从 diff 文件切出来）、各 `*.log`。这些不入库：都是复跑中间物，源与结论摘录已在模型目录。
