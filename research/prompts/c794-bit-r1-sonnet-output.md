# C794 盘上隔离位：正推腿报告（Q1–Q5、Q8），2026-10-15

腿：云端正推（Sonnet）。立场：从条款与冻结副本的实现推出每一格该取哪个读法，逐格核它与相邻条款（D19 已定项 5、D23 已定项 14、I-3.1、I-3.11）一致不一致。
冻结副本简称「冻结副本」= `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo`（引代码行时写它的绝对路径，行号按它）。

## 零、怎么量的、哪些是量的哪些是推的

- 量法：把冻结副本拷进腿自己的草稿目录，只在拷贝上加探针（环境变量开关；改动全文是模型目录里的 `src/probe-instrumentation.diff`，5 个源文件），探针测试放在拷贝的 harness 测试目录。冻结副本一个字节没动。
- 模型目录 `research/prompts/c794-bit-r1-sonnet-model/`：`tools/rerun.sh`（整条经内存包装跑，16 个步骤，末行 `name=rerun_compare compared=16 differing=0` 才算复跑一致）、`src/`（探针改动与测试）、`out/`（每步的结论行，已去掉耗时字段）。
- 复跑核对（量的）：基线出完之后用新草稿目录复跑，首趟发现并行测试里 `PROBE-Q8 rollback` 行的次序不稳，摘要里去掉这类行与 `PROBE-Q3 call=` 行，受影响的 8 个基线重出；之后三趟复跑合计覆盖 16 个步骤（11 个、4 个、1 个），三趟末行都是 `differing=0`。
- 标注：每条结论后写「（量的）」「（推的）」。「量的」都能在模型目录 `out/` 里找到原样行；「推的」是从代码或条款读出来的。
- 探针造镜像的两条规矩：不改盘上字节的造法优先（让写者那一侧自己写出来，journal 记录与根都一致）；必须改字节时，只当辅证，并写明被我改字节引出的副作用（例：I-8.10 红是改了根而 journal 记录里的新根段没改，不是被判格的现象）。

## Q1 已释放位与隔离位同时置着的记录怎么认

判定：取「三处读者都拒 / 都判红」。core 读者与 GPU 恢复内核的拒站得住；池级 checker「当已释放、不豁免、不另判红」这一半站不住（量的：它对 core 拒收的镜像全绿）。拒的射程合 D22。

条款：
- 位的出处只写了最高位：`.claude/kb/decisions/03-空间分配.md:129`：「最高位借作已释放标志（0 = 仍分配、1 = 已释放），不占表达跨度值的位」。次高位、两位同置怎么读，条款没写。
- 同类先例：`crates/singlefs-checker/invariants/01-块自描述.md:11`：「偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收」。没登记的位组合一律拒收，是仓里已有的前向兼容做法。
- 撕裂挡在父校验和上：`.claude/kb/decisions/22-单元原子性怎么合成.md:385`：「子块的任意字节子集是旧 / 新 / 零 / 垃圾，都与父里的校验和失配」。分配记录叶是有父指针的普通单元，崩溃与丢写造不出一片校验和过得去、位却不合法的叶。
- 同一条射程句的另一半：`.claude/kb/decisions/22-单元原子性怎么合成.md:387`：「可检测（父指针校验和失配），不可恢复。」

三处读者各怎么判（冻结副本）：
- core 读者拒：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/allocation_record_tree.rs:747`：「if record.is_released && record.is_quarantined {」，报 `InvariantViolated` I-1.1，与它上面的几何拒（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/allocation_record_tree.rs:741`：「|| !self.geometry.record_fits_in_its_leaf(&record)」）同一个循环、同一类。
- GPU 恢复内核同拒：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2335`：「if (((span_field >> ALLOCATION_RECORD_RELEASED_BIT) & (span_field >> ALLOCATION_RECORD_QUARANTINED_BIT) & 1u) == 1u) {」。
- 池级 checker 不拒：叶循环只判记录落不落在叶里（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker/src/walk.rs:3721`：「allocation_record_fits_in_the_leaf(」，判 I-1.1），解码把两位都掩掉，已释放的记录进豁免函数就被跳过（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker/src/walk.rs:7041`：「|| !record.is_quarantined」之前一行是 `record.is_released`）。GPU 池级判器同形：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1613`：「if (sw(a + 6u) != 0u || sw(a + 7u) == 0u || c_referenced_start(device, slot, reference_limit)) {」。

量的（模型目录 `out/q1b.txt`、`out/q1b_checker_red.txt`、`out/q1.txt`）：让写者那一侧自己写出两位同置的记录（探针开关放开写者的断言，走完整发布路径），池级 checker 对比 core 恢复：

```
PROBE-Q1B after-B-no-anomaly checker_violated=[] recover=FileRead(2500 bytes)
PROBE-Q1B C-both-bits-on-both-devices checker_violated=[] recover=Failed(InvariantViolated { invariant: "I-1.1", detail: "分配记录的已释放位与隔离位同时置着：条款没定这一组合，第一版不认" })
PROBE-Q1B C-both-bits-on-device-0-only checker_violated=[] recover=Failed(InvariantViolated { invariant: "I-1.1", detail: "分配记录的已释放位与隔离位同时置着：条款没定这一组合，第一版不认" })
```

同一份镜像，core 在任何写之前拒，池级 checker 一条不变量都不红。辅证（改字节的造法，`out/q1.txt`）：把最新根的分配记录树里已释放记录的两位同置、校验和逐道重算，池级 checker 只多出 I-8.10 红，I-1.1、I-3.1、I-3.9、I-3.11 都是 Holds；I-8.10 红是改了根而 journal 记录里的新根段没改，是改字节的副作用，不是被判格的现象（对照「不改」的基线是 `[]`）。在叶循环里加一道 I-1.1（探针开关 `PROBE_CHECKER_RED_ON_BOTH_BITS`）之后：

```
PROBE-Q1B C-both-bits-on-both-devices checker_violated=["I-1.1"] recover=Failed(InvariantViolated { invariant: "I-1.1", detail: "分配记录的已释放位与隔离位同时置着：条款没定这一组合，第一版不认" })
PROBE-Q1B C-both-bits-on-device-0-only checker_violated=["I-1.1"] recover=Failed(InvariantViolated { invariant: "I-1.1", detail: "分配记录的已释放位与隔离位同时置着：条款没定这一组合，第一版不认" })
```

加了这一道之后，现有六个测试目标（回退、隔离回退、释放核校验和、新写的隔离位豁免用例、故障注入快档、随机历史）全绿：`out/regression_with_checker_red.txt` 六行 `test result: ok`（量的）。

崩溃状态走不到这个组合（量的）：隔离那次发布的窗口 1113 个状态、回退那次发布清位 949 个、不清位 949 个，恢复一律 `FileRead`（`out/crash_isolating.txt`、`out/crash_rollback.txt` 的 `recover_histogram`）。与 22:385 一致：只有写者缺陷、校验和重算过的人为伪造、或内存里位翻转之后才算的校验和，才走得到。

另两个读法为什么出局：
- 「三处都宽容」（读者把已释放记录上的隔离位掩掉）：要在 `parse` 里归一，否则重建出来的分配器下一次改写这条记录撞写者的断言（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/allocator.rs:63`：「!(self.is_released && self.is_quarantined),」，推的）；而且这个组合正是「释放漏清位」这一类写者缺陷的产物，宽容等于不检出它；与 I-1.6 那一行（01-块自描述.md 第 11 行）的先例反向（推的）。
- 「core 拒、checker 不红」（实现员今天的样子）：量的上面三行。崩溃点重放里这个差不会漏（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker-tier/src/crash.rs:949`：「CrashPointReplayOracleViolationKind::ReadFailed,」，恢复 Failed 一律是违例），漏的是单独拿池级 checker 判一张镜像的用法。

D22 射程：合。D22 已定项 19 把「撕裂」挡在父校验和上，这一拒只对「校验和对得上而内容不合法」的盘面生效，与同一循环里的几何拒同类；D23 已定项 14 的「失败的处置按失败的性质分两支」（`.claude/kb/decisions/23-journal的角色与格式.md:398`：「**失败的处置按失败的性质分两支**」）管的是写失败的探针写与落点复核，不管读侧的拒，所以这一拒不进那两支，也不冲突（推的）。

不能复核的：GPU 两个内核在这一轮不起，GPU 池级判器要不要同改只能按代码读：扫描内核解到「已释放」就跳过，与 CPU 同形，加红要在 `r_allocation_tree` 里解 `span_field` 处同改（推的）。

什么现象会推翻：(a) 找到一条合法路径（例如某处把释放拆成「先置位后清位」两次发布）会让已释放+隔离同置落盘：崩溃窗口里会出现 `Failed(InvariantViolated … 同时置着)` 的恢复结局；本轮三个窗口 3011 个状态里 0 次。(b) 在池级 checker 加红后，现有目标出现新红：本轮 0 个。

## Q2 回退重新引用隔离单元时清不清位

判定：取「清」。不清的读法留下一个具体盲点（量的）。清的那一写在逐盘验之后、发布失败整体换回之前，随回退那次发布落盘。

条款：
- 隔离的定义是「没有任何根引用的记录」：`crates/singlefs-checker/invariants/05-空间记账.md:8`：「隔离的记录：已分配而没有任何根引用的记录，按单元豁免」（I-3.11 那一行 18 同句）。回退把单元重新引用之后它不再属这一类。
- 回退本来就在这次发布里改这些单元的账：`.claude/kb/decisions/23-journal的角色与格式.md:384`：「改回已分配，分配代写回 R_old 那一版账里的分配代」（复活行），清位是同一类改动，同一次发布落盘。
- 拒在先：`.claude/kb/decisions/23-journal的角色与格式.md:388`：「同样逐盘验，有一份读不出或对不上就拒」；失败不留痕：`.claude/kb/decisions/23-journal的角色与格式.md:390`：「崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同」。

清的那一写在回退的哪一步（冻结副本，量的行号 + 推的次序）：
- 逐盘验隔离单元：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:8225`：「every_copy_of_the_isolated_units_the_target_references_reads_back(」。
- 然后备份分配器：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:8291`：「let allocator_before_the_rollback = allocator.clone();」。
- 然后复活、清位、释放：清位在 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:8302`：「allocator.clear_quarantine_of_the_records_of(Placement {」。
- 发布失败整体换回：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:8368`：「*allocator = allocator_before_the_rollback;」。
所以清位在「任何写之前拒」那一判之后（拒绝时一个字节、一个分配器字段都没动）。

量的（`out/matrix.txt`，瞬时隔离：盘上字节不动，只让被换下那份的读先报错两次）：清位 / 不清位 × 回退后写者正常 / 回退后某次覆盖写漏释放（把那两条记录放回覆盖写之前的值，记账同步）：

| 读法 | 回退后写者 | 覆盖写 D E F 之后 | 抬 F 之后 |
|---|---|---|---|
| 清位 | 正常 | `[]` | `[]` |
| 清位 | 漏释放 | `["I-3.11"]` | `["I-3.1", "I-3.11"]` |
| 不清位 | 正常 | `[]` | `[]` |
| 不清位 | 漏释放 | `[]` | `[]` |

最后一行就是盲点：单元曾被隔离、被回退重新引用、之后写者漏释放，记录留在已分配又带着旧位，池级 checker 全绿。原样行：

```
PROBE-R [no_clear=false forgotten_release=true] 5 after raising F to 6 (newest txg 9): checker_violated=["I-3.1", "I-3.11"] bit_records_referenced(union=0,newest=0) records[d0(released=false,bit=false,gen=3) d1(released=false,bit=false,gen=3)]
PROBE-R [no_clear=true forgotten_release=true] 5 after raising F to 6 (newest txg 9): checker_violated=[] bit_records_referenced(union=0,newest=0) records[d0(released=false,bit=true,gen=3) d1(released=false,bit=true,gen=3)]
```

不清位而写者正常的格里，之后的普通释放把位清掉（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/allocator.rs:1899`：「record.is_quarantined = false;」是释放分支），所以盲点只在「写者漏释放」这一层，不是每次都有（量的，表中第三行）。

清位随同一次发布落盘（量的）：回退之后写者的记录 `bit=false`（`out/matrix.txt` 里「after rollback to A」那一行）；回退那次发布的窗口里 949 个崩溃状态，池级 checker 一律 `[]`，恢复一律 `FileRead`，最新根引用着带位记录的状态 0 个（`out/crash_rollback.txt`：`by_newest_root=0`）。不清位的对照窗口里同样 949 个状态全 `[]`，但根落盘之后的 10 个状态最新根引用着带位记录（`by_newest_root=10`），这些状态今天的 checker 判不出（要靠 Q3）。

位跨重挂（量的，`out/remount.txt`）：冷启动可写挂载之后重建的分配器里，两块盘上的记录仍是 `bit=true`。所以隔离后重挂再回退，清位照样走得到。

什么现象会推翻：(a) 回退之外还有一条路径能让隔离单元被重新引用（以后的快照克隆、重放）而没清位：会在最新根引用着带位记录的镜像上出现，Q3 的探针计数 `newest` 大于 0；(b) 清位发生在逐盘验之前或发布失败时没换回：拒绝用例里分配器字段变了（`rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs` 里 `rolling_back_to_the_version_referencing_an_isolated_unit_with_a_copy_that_does_not_verify_is_refused_before_any_write` 钉着这个，冻结副本里绿）。

## Q3 池级 checker 要不要把「隔离位置在有根引用的记录上」判违例

判定：两个都站得住（加 / 不加）。若加，「有根引用」只能取「最新根走读」，不能取「任一走过的版本」：后者在合法的刚隔离状态上红（量的）。

「有根引用」有两种口径，代码里两个现成的集合：
- 任一走过的版本：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker/src/walk.rs:12974`：「&start_slots_referenced_by_every_walked_version,」（I-3.1 的豁免用它）。
- 最新根：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker/src/walk.rs:13024`：「&start_slots_referenced_by_the_newest_root,」（I-3.11 的豁免用它）。

量的（探针只计数「带位、未释放、却被这个集合引用」的记录，不改任何判定；`out/matrix.txt`、`out/crash_isolating.txt`、`out/crash_rollback.txt`）：

| 状态 | 任一走过的版本口径 | 最新根口径 |
|---|---|---|
| 隔离那次发布之后、被换下的旧根还在环里（合法） | 2 条（矩阵里「after B」那一行，四格都是） | 0 条 |
| 隔离发布的窗口 1113 个崩溃状态 | 10 个状态 > 0 | 0 个 |
| 回退发布（清位）窗口 949 个崩溃状态 | 939 个状态 > 0 | 0 个 |
| 回退之后（清位） | 0 | 0 |
| 回退之后（不清位，写者缺陷） | 2 条（矩阵里「after rollback to A」那一行，不清位的两格） | 2 条 |
| 不清位的回退窗口 949 个状态 | 949 个 > 0 | 10 个 > 0（根落盘之后） |

读法：任一走过的版本口径把合法状态判红（隔离窗口里 10 个、回退窗口里 939 个状态，外加刚隔离之后的稳态），出局。最新根口径在清位世界里 0 个误判，其中包括 Q2 那一写落盘之前的全部崩溃状态（949 个，`by_newest_root=0`）；它抓到的是不清位的回退（写者缺陷）与「写者把位置在活记录上」这一形（后者没单独造，推的：形与前者相同，最新根引用着带位的记录）。

不加的后果：Q2 取「清」之后，漏清只会在「之后再漏一次释放」时才暴露（Q2 表最后一行）。加了之后漏清当场红。代价：池级 checker 多一道判格，GPU 池级判器同改（扫描内核已有 `reference_limit` 两遍，推的），I-3.1 / I-3.11 条款或新开一条要写进去；现有测试里没有一处在「带位记录被旧根引用」的状态上跑 checker（量的：六个目标里 `PROBE-Q3` 行 0 行，`out/regression_with_checker_red.txt`），所以加红口径选错也不会被现有测试发现，要靠本轮这类探针。

什么现象会推翻：(a)「最新根口径在某个合法状态上 > 0」：把这个口径接到 I-3.11 之外的版本（由记录施加出来的那一版、被抛弃的根）时要重量，本轮只量了最新根与候选集里的走读；(b) 「不清位」世界里 checker 其实有别的路径判出盲点：量的 0。

## Q4 回退判「隔离单元」那一处要不要改成看位

判定：两个都站得住（按推断 / 看位），差别只出现在写者有缺陷或位丢了的状态上；写者正确时两个读法逐例同判（量的）。若必须选一个，推的偏向「不改」：看位读法在缺陷态上让回退放行一个有一份坏拷贝的单元，推断读法拒。

条款：
- `.claude/kb/decisions/23-journal的角色与格式.md:388`：「被隔离的那些（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：读盘核出一份对不上、两份都留在已分配）」——条文用「读盘核出对不上」这个事件定义隔离，没说怎么认；同一格的另一半：「其余 cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」。
- 今天的代码用「现行账里仍分配、现行那一版不引用」当这个事件的代理，并自称只有一种来路：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:7729`：「现行账里仍分配而现行那一版不引用，只有释放之前读盘核出一份对不上、两份都留在已分配」；分类在 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:7817`：「let is_referenced_by_the_current_version = locations」与 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:7823`：「isolated_units_to_check_on_every_device.push((*unit, *locations));」。
- 写者两块盘一起置位、整单元同存同清（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/transaction.rs:3726`：「// 两条位置项各指池里一块不同的盘（调用方判过；第一版池里恰好两块盘，mkfs 断言过），两份都留：按位置项的次序（设备身份升序）各交一份。」），所以看位时「任一盘带位」与「每盘带位」在写者正确的状态上没有区别。

量的：
- 写者正确：探针里把分类换成看位（任一盘带位 / 每盘带位 / 带位且现行版本不引用）或把守卫去掉（`inference_without_guard`，即 C572 判决 A2 那一行变异），现有四个目标（回退、隔离回退、释放核校验和、新写的隔离位豁免用例）全绿，四行 `test result: ok`，五个变体的摘要逐行相同（`out/variant_*.txt`）。现有用例分不开两个读法，也杀不死 A2 那条变异（判决 K2.3 同形）。
- 写者缺陷或位丢了（`out/q4.txt`）：

```
PROBE-Q4 mode=inference healthy-isolation+bad-copy: Err(IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50180) })
PROBE-Q4 mode=inference leaked-unit(no bit)+bad-copy: Err(IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50180) })
PROBE-Q4 mode=inference isolation-whose-bit-got-lost+bad-copy: Err(IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50180) })
PROBE-Q4 mode=bit_any healthy-isolation+bad-copy: Err(IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50180) })
PROBE-Q4 mode=bit_any leaked-unit(no bit)+bad-copy: Ok(回退做成)
PROBE-Q4 mode=bit_any isolation-whose-bit-got-lost+bad-copy: Ok(回退做成)
```

（`bit_all` 与 `bit_any` 三行相同。）不带位的那一类的去向：看位读法下它落进「仍分配、同分配代、不动」那一支，不进逐盘验，回退照做，新根引用一个有一份坏拷贝的单元；推断读法下它进逐盘验，拒。这一类在写者正确时不存在（它只能是写者漏释放，或位丢了）。

与 C572 判决（`research/prompts/code-backlog-b2-code-r1-main-verification.md`）对不对得上：
- K2 验的是推断读法：`research/prompts/code-backlog-b2-code-r1-main-verification.md:22`：「23670 次回退里隔离单元清单只 2 次非空、都出自 C572 用例」。改成看位，这一格的计数口径换成「带位的清单」，数仍可复核，判决本身不翻。
- A2 要加一行变异把守卫改成恒真：`research/prompts/code-backlog-b2-code-r1-main-verification.md:33`：「那一行变异红在新用例上」。看位读法下分类不再用那个守卫，这一行变成等价变异，要换成别的变异（例：把看位改成恒真）；「看位且现行版本不引用」读法保留守卫，但守卫只在「带位而被现行版本引用」的状态上有区别，Q2 清位之后公开入口造不出这个状态（量的：清位世界最新根引用带位记录 0 个），用例只能伪造分配器状态。
- 看位读法依赖 Q2 清位与位不丢；位跨重挂活着（量的，Q2 一节 `out/remount.txt`），清位照做。

什么现象会推翻：(a) 找到写者正确的状态上两个读法分类不同：把分类换成看位后现有目标出现红（量的 0 个；要补的探针：快照持有的单元，今天没有快照，推的）；(b) 看位读法的缺陷态放行被证明是合法的（例：条款明说漏释放的单元可以不验）：目前条文没这句。

## Q5 豁免按记录还是按单元

判定：取「按记录」。「按单元」出局：它在写者缺陷造出的不对称镜像上全绿，而 core 在同一镜像上拒（量的）。

条款：
- 条文原来的「按单元」是为「读盘只核出一份坏」服务的：`crates/singlefs-checker/invariants/05-空间记账.md:8`：「它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免」（I-3.11 那一行 18 同句）。判据是单元一级的读盘结果，所以豁免也得按单元广播。有了位，广播由写者做。
- 对称是写者的不变量：`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`：「另一块盘上那一份对得上也一起留，各盘的账保持对称」。

量的（`out/asymmetric.txt`；写者缺陷：只有一块盘上的记录带位）：

```
PROBE-Q5 sym-control after raising F to 5: per_record_violated=[] per_unit_violated=[] recover=FileRead
PROBE-Q5 asym-bit-only-on-d0 after raising F to 5: per_record_violated=["I-3.1", "I-3.11"] per_unit_violated=[] recover=Failed:InvariantViolated { invariant: "E142 走读同款", detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数" }
PROBE-Q5 asym-bit-only-on-d1 after raising F to 5: per_record_violated=["I-3.1", "I-3.11"] per_unit_violated=[] recover=Failed:InvariantViolated { invariant: "E142 走读同款", detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数" }
```

- 写者两块盘一起置位（对称）的镜像上，两个读法同判（上面 `sym-control` 那一行，两个都 `[]`，恢复 `FileRead`）。
- 崩溃状态上同判：隔离窗口 1113、回退窗口（清位）949、回退窗口（不清位）949 个状态，「按单元读法的判定集合 ≠ 按记录读法的判定集合」的状态数都是 0（`out/crash_isolating.txt`、`out/crash_rollback.txt` 的 `per_unit_vs_per_record_verdict_differs=0`）。「只置了一块盘就崩」造不出：两块盘的记录住同一棵树，新树由根一次生效，一块盘的叶的两份拷贝要么都是新内容要么校验和对不上父（22:385），读到哪份都是同一内容（推的）；量到的 0 与此一致。
- 不对称只来自写者缺陷：按记录读法红 I-3.11（抬 F 之后 I-3.1 也红），与 core 的拒同向：core 逐盘比（槽, 跨度, 代, 已释放, 隔离）集合，不同就拒（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/recovery.rs:4750`：「if !allocation_records_are_one_per_device(records, device_identities, PlacementsOnEachDevice(0))」，集合里加隔离位在 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/recovery.rs:4488`：「is_quarantined: record.is_quarantined,」）；池级 checker 自己没有这一格对称判（冻结副本 `walk.rs` 里「E142 走读同款」零命中，量的），按单元读法下它对不对称的镜像无话可说。
- 小处：core 拒的文案 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/recovery.rs:4747`：「各盘的（槽, 跨度, 代, 已释放）集合不同」没带隔离位（量的，见上面输出里的 detail）。

条款要改的句子（推的）：I-3.1（05-空间记账.md 第 8 行）与 I-3.11（同文件第 18 行）里「按单元豁免」那一整句要换，换成「按记录豁免：已分配、没有根引用、跨度段隔离位置着」；「按单元」的出处（主 agent 2026-09-25 同日定）在换句时一并写清它服务的是旧判据。

什么现象会推翻：写者正确的状态（含崩溃状态）上出现「一块盘带位、另一块不带」：按记录读法会误红。本轮量的 3011 个崩溃状态与对称对照里 0 个；要推翻还需要一条合法路径只写一块盘的位（例：以后的异构池让两盘落点不同——到那时「按单元」的分组键 `(槽, 跨度)` 本来也用不了，推的）。

## Q8 理想模型要不要观测隔离位、判活单元的记录不带位

判定：两个都站得住（加 / 不加）。加了抓的是「活单元记录上带着位」这一类（Q2 漏清、写者误置位），但今天的随机历史与故障注入撞不上它的前提，量到的检出是 0；代价小（5 行）。

模型今天的样子：
- 模型对每个角色的单元只比三样：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/model.rs:3370`：「.all(|record| !record.is_released && record.generation == *written_at);」。观测结构 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/model.rs:1093`：「pub struct ObservedAllocationRecord {」只有设备、代、已释放三个字段；转换处 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/model_comparison.rs:353`：「is_released: record.is_released,」。
- 模型对隔离没有任何概念：`grep -nE '隔离|quarantin' model.rs model_comparison.rs`（冻结副本）零命中，退出码 1（量的）。所以加了也只能判「活单元不带位」，判不了「该带位的被换下单元带了位」。
- 执行器已经看得到位的变化，并有意忽略：`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/history.rs:1839`：「let differs_from_a_record_before_only_in_the_quarantine_flag = |record: &AllocationRecord| {」，`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/history.rs:1852`：「.filter(|record| !differs_from_a_record_before_only_in_the_quarantine_flag(record))」。执行器一侧加一道「发布之后现行版本引用的单元，记录不带位」是同样便宜的另一个落点（推的）。

代价（量的，拷贝上实改）：模型改动 5 行（观测结构加一个字段、对拍条件加一个 `&& !record.is_quarantined`、转换处一行、两处测试构造各一行）；一条单测钉住：活单元记录带位，模型判 `AllocationGeneration`（`out/q8_model.txt`：`PROBE-Q8 model verdict on a stale bit: AllocationGeneration`）。

能抓到什么、撞不撞得上（量的）：
- 六个现有目标合计（回退、隔离回退、释放核校验和、新写的隔离位豁免用例、故障注入默认档、随机历史默认档）：10 次回退走到清位那一步，隔离清单非空的 1 次，是 C572 的专用用例（`out/regression_with_checker_red.txt` 末行）。单跑随机历史默认档是 6 次与 0 次，单跑故障注入默认档 0 次回退（草稿目录日志，不在模型目录里）。
- 故障注入大档（C600 登记的参数，种子基 7463871032432355113、256 段、30 步、6 次，debug 构建）：跑到限时被停（草稿目录里的日志，不进模型目录），停前 596 次回退走到清位那一步，隔离清单非空的 0 次。没跑完，不当「没打中」用。
- 判决里的旧数：`research/prompts/code-backlog-b2-code-r1-main-verification.md:22`：「23670 次回退里隔离单元清单只 2 次非空、都出自 C572 用例」。
- 所以模型加了位观测之后，今天的取样对 Q2 漏清的检出是 0；真正钉着「回退之后位清掉」的是专用用例（冻结副本 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs:213`：「!record.is_released && !record.is_quarantined && record.generation == CheckpointTxg(3),」）。要让模型这条线有用，得让生成器造得出「隔离 + 回退到引用它的根」（故障注入里读报错再回退），那是另一个题。

什么现象会推翻：生成器扩到能造隔离 + 回退之后，在不清位的变异上模型对拍红：本轮没有这条生成器，量不到。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 | 取「三处都拒 / 都判红」 | core 与 GPU 恢复内核的拒站得住（同类先例 I-1.6、几何拒）；池级 checker 当已释放不判红这一半站不住：同一份镜像 core 拒、checker 全绿（量的）；崩溃状态 3011 个走不到；D22 已定项 19 不冲突 |
| Q2 | 取「清」 | 不清留盲点：曾隔离、回退重新引用、之后写者漏释放，checker 全绿（量的 4 格矩阵）；清位在逐盘验之后、发布失败换回之前，随同一次发布落盘；位跨重挂活着 |
| Q3 | 两个都站得住 | 加则只能取最新根口径：任一走过的版本口径在合法的刚隔离状态上红（回退窗口 939 / 949 个状态）；最新根口径在清位世界 0 误判，抓得到不清位 |
| Q4 | 两个都站得住 | 写者正确时逐例同判（5 变体 × 4 目标全绿）；写者缺陷或位丢了，看位读法放行有一份坏拷贝的回退，推断读法拒；若选推的偏向不改；A2 的变异行在看位读法下变等价变异 |
| Q5 | 取「按记录」 | 对称镜像与崩溃状态上与按单元同判；写者造出的不对称镜像上按单元全绿而 core 拒；条款 I-3.1 / I-3.11 的「按单元」整句要换 |
| Q8 | 两个都站得住 | 加 5 行、能判活单元带位；随机历史与故障注入今天撞不上前提（量到 0 次检出）；模型没有隔离概念，判不了该带位的 |

## 要主 agent 跑的命令

复跑整个模型目录（预计前台超过 4 分钟，主 agent 用 `run_in_background` 起；耗时来自编译和三个崩溃窗口）：

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/c794-bit-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo --work <新的草稿目录> --model research/prompts/c794-bit-r1-sonnet-model
```

工作目录随意；看末行 `name=rerun_compare compared=16 differing=0`（脚本内部经 `capped.sh default` 起 cargo；`RERUN_STEPS` 设空格分隔的步骤名只跑几步，`CARGO_TARGET_DIR` 不设就用 `<新的草稿目录>/target`）。

没跑完、可选的一格（Q8 的故障注入大档）：跑完上面的复跑后，在 `<新的草稿目录>/tree` 里：

```
SINGLEFS_FAULT_INJECTION_FIRST_SEED=7463871032432355113 SINGLEFS_FAULT_INJECTION_SEEDS=512 SINGLEFS_FAULT_INJECTION_OPERATIONS=30 SINGLEFS_FAULT_INJECTION_FAULTS=6 bash scripts/hosts/run-with-memory-cap.sh 24G bash scripts/hosts/capped.sh default cargo test --release -p singlefs-harness --test fault_injection_fast_tier -- --ignored --exact fault_injection_large_tier_from_the_environment --nocapture
```

跑完数输出里 `PROBE-Q8 rollback isolated_units=` 后面不是 0 的行数（探针行只在拷贝里有）。预期整条因 C600 登记的已知红而红，不影响数这几行。

## 没做什么

- 没起 GPU，没跑 checker 档测试，没跑任何门禁阶段；GPU 两个内核（恢复内核与池级判器）只按代码读，Q1 的 GPU 一侧是推的。
- 没动冻结副本与主工作区；所有改动在草稿目录的拷贝里，且只经环境变量开关生效（探针开关全部在模型目录 `src/probe-instrumentation.diff`）。
- Q3 的「写者把位置在活记录上」没单独造镜像（与不清位的回退同形，推的）。
- Q8 的故障注入大档没跑完（限时被停），只有部分计数；快照持有的单元（Q4 推断读法可能多验的另一类）今天没有快照，没量。
- 没替主 agent 采纳或出判决；Q3、Q4、Q8 写「两个都站得住」是量的取舍，岔路单由主 agent 按跑前条款定。
- 这是正推腿的一次抽样：「没找到」的那些（Q1 的合法组合路径、Q5 的合法不对称路径）只找了一次，不当「没打中」用。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/c794-bit-r1-sonnet-output.md --root /home/fy5090/code/singlefs` 末行：

  ✓ 核了 44 处引文、0 处裸行号，对上 44 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
