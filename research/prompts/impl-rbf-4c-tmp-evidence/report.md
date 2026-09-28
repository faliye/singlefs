# 实四丙交回：故障注入大档新发现与坏盘输入那条红的查因（只查原因，`crates/` 一行没改）

时刻：2026-09-26（开工）。草稿目录 `/tmp/claude-1000/impl-rbf-4c/`，日志都在 `logs/`，探针源码在 `probes/`。

## 一、结论一览

大档规模（种子基起 512 段、每段 30 步、每段 6 个注入点、两块 4 GiB 盘、每步跑 checker）在今天工作区的副本上重跑，25 条新发现逐条与实四乙的 `scan-findings.log` 相同（23 段）。

先更正派发里的一句：25 条里 **offset 445 那条注入的是 `ReadReturnsCorruptedBytes`（读回翻位），不是 `WriteIsSwallowed`**（`logs/scan-work.log` 的 FINDING 行原样）。其余 24 条都是写被吞。

| 组（签名） | 条数 | 判 | 落点 | HEAD `73ba4a4` 上 |
|---|---|---|---|---|
| ① `["I-2.1","I-3.10","I-4.8","I-7.4"]` | 11 | (ii) | 被吞的是一次单元写，落在复用过的槽上，槽里留着上一个单元（头读得出）⇒ 多出 I-3.10 | 扫描里 9 条 |
| ② `["I-2.1","I-4.8","I-7.4","I-9.1"]` | 7 | (ii) | 被吞的是 inode 树根那一次写，落在从没写过的槽上（读出全零、类标签 0）⇒ 多出 I-9.1 | 扫描里 4 条 |
| ③ `抬 F 的上限` | 3 | 339、284：(ii)；**445：(i)** | 339 / 284 是根槽 FUA 被吞；445 是 `mount.rs` 算上限时一次根槽读坏就把那条根悄悄丢掉 | 扫描里 0 条；同注入点在 HEAD 上重放同样红（见 §四） |
| ④ `冷启动读回` | 2 | 330：(iii)；284：(ii) | 两条都**既没读回没确认的内容，也没丢已确认的内容**（内容逐字节等于模型提交过的那一版） | 扫描里 4 条，两种形态都有 |
| ⑤ `["I-2.1","I-3.10","I-3.11","I-4.8","I-7.4"]` | 1 | (iii) | 多出来的 I-3.11 是 checker 的隔离豁免判据与实现的隔离判据不一致（`walk.rs:3318`） | 扫描里 0 条；构造的同形在 HEAD 上逐字相同（§四） |
| ⑥ `["I-2.1","I-3.1","I-3.10","I-4.8","I-7.4"]` | 1 | (ii) | 根槽 FUA 被吞、根环已转过一圈 ⇒ 那个槽里留着 24 代之前的旧根，它的单元已被合法回收复用 | 扫描里 0 条；同机理在 HEAD 上逐字相同（§四） |

**新旧**：六组的机理在 HEAD 的代码上都有，没有一组是这一轮没提交的改动新带进来的。扫描在两份代码上抽到的注入点不同：抽样按每步调用数摆点，而这一轮改了每步写几次（抬 F 先写系统配置等），生成器也把回退操作从 `CloseAndMountRollback` 换成了 `RollBackWhileMounted`。所以只看扫描计数会误判「HEAD 上没有」；③⑤⑥三组都是拿同一注入点，或构造的同形注入点，在 HEAD 上直接重放核的。

**第 2 件**：(ii) 期望过时。实三把随机历史的回退操作换掉之后，写死的那段历史（种子基 + 4）第 3 步从「回退挂载做成、取号实例 2」变成了「`TargetNotACandidate(VersionWithoutFile)` 被拒」。盘面跟着回到槽 50304、根 (2, 9)。用例里三处期望串过时：本条那处，以及同一张表第 12 条坏法的两处（今天被第 10 条挡在前面、还没走到）。错误成员一个没变；`RowPublishAdmissionRefusedBeforeAcquisition` 那层包装 HEAD 上就有，用例按子串比，不是红的原因。HEAD 上这条用例绿。

## 二、第 1 件逐组

证据来源都是这三样：
- 大档扫描：`logs/scan-work.log` / `logs/scan-head.log`（`probes/zz_probe_scan.rs`）；
- 单注入点深探针：`probes/zz_probe_one_fault.rs`，逐步打出操作、结局、盘上可读的根、模型根环与上限、被注入的那次调用落在什么结构，再重开并跑 checker。日志 `logs/one-work-<offset>-<call>.log`，一行汇总在 `logs/summary-work-points.txt`；
- 读注入只摆在点名步里的探针：`probes/zz_probe_armed_at_step.rs`。

### ① I-3.10 那组（11 条，offset 59 96 120 204 313 335 387 447 454 482 490）：(ii)

- 被吞的都是单元写（`unit_write`）。I-2.1 与 I-3.10 点名的（盘, 槽）与被吞那次写的盘和槽号（偏移 ÷ 16384）逐条相同。
- I-3.10 的两个数是「这次写的 txg」对「槽里旧单元的诞生代」，旧的都更小：59: 27/5，96: 23/3，120: 27/5，204: 14/0，313: 37/5，335: 28/3，387: 21/0，447: 24/4，454: 32/6，482: 23/3，490: 26/5（`REOPEN CHECKER I-3.10` 行）。
- 机理：丢的那一份落在一个复用过的槽上，槽里还是上一个单元，头读得出。于是 I-3.10 射程 ③（`.claude/kb/invariants.md:140`，「那个槽上读不出可用的单元头时这一条没有对象」）的反面成立，它照判、判红；另三条就是白名单那一组。
- 该归哪一格：说谎的设备留下的不一致那一格（`FaultInjectionTally::faults_where_a_lying_device_left_the_image_inconsistent`，判法在 `fault_injection.rs:1920` `lying_device_may_leave`）。
- 今天没罩住的原因：白名单 `INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE`（`fault_injection.rs:1915`）是用户 2026-09-21 定的两组（`.claude/kb/milestone/02-second-txn.md:421`「白名单写死两组、名单外照报」）。I-3.10 是 2026-09-23 才实现的（`invariants.md:140` 状态列），白名单早于它。
- 为什么不是悄悄读回没提交过的内容：11 条重开都是 `CommittedByTheModel`。重开判定按根 key 与内容逐字节比（`model.rs:1855` 起），比过了才记这一格。

### ② I-9.1 那组（7 条，offset 46 87 149 176(call 663) 193 309 386）：(ii)

- 被吞的都是单元写。I-2.1 点名「树 12（种类 2）的根」，也就是 inode 树根，盘和槽与被吞那次写相同。I-9.1 报「树表指向的 inode 树根类标签是 0，不是 2」。
- 机理：丢的是 inode 树根这一份，而那个槽从没写过，读出全零、类标签 0。I-9.1（`invariants.md:272`）按树表指的那个单元判类标签，所以红；全零的头没有可用单元头，I-3.10 报不适用，这一组也就没有 I-3.10。
- 格与白名单同 ①。I-9.1 早就有；白名单按快档 24 段定，快档没抽到「丢的正好是 inode 树根」这一形。
- 7 条重开都是 `CommittedByTheModel`：读的是另一块盘上那一份，读回的内容逐字节等于模型提交过的那一版。

### ③ 抬 F 的上限（3 条）：339、284 判 (ii)；445 判 (i)

**offset 339**（种子 7463871032432355452，写被吞，整池第 145 次写，第 4 步覆盖写）：(ii)
- 被吞的是 (1,6) 那条根的根槽 FUA 写（盘 0 偏移 1056768，`logs/one-work-339-145.log` 的 FIRED 行）。
- 第 5 步抬 F：实现从盘上现读根环，(1,6) 不在盘上，报上限 0；模型根环里有 (1,6)，答 3。
- 实现照 D16 已定项 1「抬 F 的上限」（`.claude/kb/decisions/16-发布语义.md:36`：min(每块盘上最新的**持久**有效根, 第 4 新的非空**持久**有效根)）按盘上真落了的根算。模型不知道设备说了谎，按确认过的根算。
- 该归哪一格：说谎的设备那一格。今天 `lying_device_may_leave` 把一切 `ModelDisagreement` 排除在外，这一形罩不住。
- 可判的条件（推的，没实现）：实现报的上限，等于把被吞的那几条根从模型根环里拿掉之后模型重算出的上限。
- 丢一条根只会让「每块盘最新」与「第 4 新的不同状态」变旧、不会变新，所以这一形只会把上限压低（偏保守）。这是推的，没有逐形穷举。
- 重开：journal 里 (1,6) 的记录在，恢复施加之后走到 (1,6)，读回 9052 字节，`CommittedByTheModel`。

**offset 284**（种子 7463871032432355397，写被吞，第 571 次写，第 22 步覆盖写）：(ii)，同 339
- 被吞的是 (3,22) 的根槽 FUA 写（盘 1 偏移 4222976）。
- 第 23 步抬 F：实现上限 11、模型 14；实现把 F 抬到 11 并发了 (3,23)–(3,25) 三条空发布（`logs/one-work-284-571.log`）。

**offset 445**（种子 7463871032432355558，`ReadReturnsCorruptedBytes { byte_index: 17519073561180467213, bit_index: 7 }`，整池第 1578140 次读，第 27 步抬 F）：**(i) 实现的错**
- 被坏的那次读是盘 1 偏移 4198400，即区域 1 槽 1。第 26 步之后那里是 (4,28)（`logs/one-work-445-1578140.log` 的 disk 行）。
- 实现报上限 27，模型 28。(4,28) 那一读自证不过，就被当成「这个槽没有有效根」丢掉，第 4 新的不同状态因此往旧挪了一格。
- 落点：`crates/singlefs-core/src/mount.rs:984` `rollback_floor_ceiling`，第 990 行取 `readable_roots`；再往下是 `crates/singlefs-core/src/recovery.rs:700` `visit_valid_roots_with_ring_slots`，其中第 713 行读不出就 `continue`，第 716 行 `parse_slot` 自证不过同样跳过。一次读错（报错的 `ReadFails` 也一样）被当成那条根不存在，不重读、不交回错误。
- 违反：D16 已定项 1「抬 F 的上限」（`16-发布语义.md:36`）的值是在全部持久有效根上算的，这条根持久、有效，只是这一次没读对。
- 445 本身偏保守（27 < 28）。**反方向我造出来了，两份代码都红**：
  - 设置：种子 7463871032432355452（基 + 339）第 1 步「抬 F 到 1」，条款上限 0（非空状态不足 4 个，取最旧有效根 (0,0)）。
  - 注入：只在这一步里，对盘 0 偏移 1048576（(0,0) 的根槽）的读注一次 `ReadFails`，或一次读回翻位。工作区是这一步里第 2 次读这个偏移，HEAD 是第 1 次；工作区第 1 次读注上去不影响结局，第 2 次才是算上限那一读。
  - 结果：实现把最旧有效根当成 (1,1)，上限算成 1，**接受了 F=1**。模型判「模型说该拒、实现做成了」，同一步 checker 判 **I-7.9 红**：「实例 1 txg 5 那条根把回退下界 F 从 0 抬到 1，高于它之前的根算出的抬 F 上限 0」（`logs/armed-339-s1-work-read_fails-n2.log`、`logs/armed-339-s1-head-read_fails.log`、`logs/armed-339-s1-head-read_corrupt.log`）。
  - 结论：设备交回的一次读错被吞掉，上限越过了条款值，F 被抬过了头。
- 最小复现（写死）：
  - 种子 + 注入点照上面，`HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES`、`GenerationWeights::BROAD`、30 步；
  - 探针 `probes/zz_probe_armed_at_step.rs`，环境变量 `SEED_OFFSET=339 ARM_STEP=1 DEVICE=0 AT=1048576 FAULT=read_fails NTH=2`（HEAD 上 `NTH=1`）。
- 修法没定：重读一次，还是照树表那一格读不出就拒（`RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`，「读不出要走修复，不按空或非空猜」），或者别的。怎么分辨「崩溃撕裂的根槽」和「这一次读坏」条款没写，见 §五 问 1。

### ④ 冷启动读回（2 条，逐条判）

**offset 330**（种子 7463871032432355443，写被吞，第 466 次写，第 14 步管理员回退）：**(iii)，对拍胶水的错**
- 被吞的是回退那次向前发布 (5,19) 的根槽 FUA 写（盘 1 偏移 4218880）。
- 第 15 步冷启动：恢复择根（施加之前）择到 (5,18)，journal 里 (5,19) 的记录在，施加之后实际走的是 (5,19)。读回 12064 字节，内容哈希 `f2152dfebfb33bc6`，**与模型 (5,19) 那一版逐字相同**（`logs/one-330-hash.log`：`REOPEN effective_root=Some((InstanceGeneration(5), CheckpointTxg(19))) outcome=FileRead root=(InstanceGeneration(5), CheckpointTxg(18)) len=12064 … hash=#f2152dfebfb33bc6`；模型 `(5,19):12064#f2152dfebfb33bc6`、`(5,18):3514#0362d40ebca7259c`）。
- 对不上的只是根的 key：`crates/singlefs-harness/src/history.rs:2880`（`apply_cold_start_recover`）拿 `observed_read_back(&report.outcome)`（`model_comparison.rs:376`），报的是施加之前所选的根。崩溃注入与重开那一路早已改用施加之后实际走的根（`observed_read_back_after_a_crash`，`model_comparison.rs:397`，注释写明两者只在「记录已持久、根槽没落」时不等），历史里的冷启动这一步没跟着改。说谎的设备在不崩溃的历史里造出了同一个窗口。
- **既不是读回没确认的内容，也不是丢了已确认的内容**：读回的正是最新确认的 (5,19)。同一段历史最后那次重开走 `observed_read_back_after_a_crash`，判 `CommittedByTheModel`（走到 (5,19)）。
- HEAD 扫描里 offset 115、330、375 三条是同一形。

**offset 284**（与 ③ 的 284 同一注入点）：(ii)，前一条对不上的连带
- 重开走到 (3,25)，读回 19504 字节，哈希 `d37817c735811b9e`，**与模型提交过的 (3,22) 逐字相同**（`logs/one-284-hash.log`）。
- (3,23)–(3,25) 是第 23 步抬 F 的空发布，只推 F、不动内容。历史在第 23 步因 ③ 的上限对不上而停下，模型没认这三条根，于是「模型从没提交过这一版」。
- **既不是读回没确认的内容**（内容是 (3,22) 的），**也不是丢了已确认的内容**（最新确认的就是 (3,22)）。
- 落点：`crates/singlefs-harness/src/fault_injection.rs:2067`–`2075`。重开的放行集只并了「注入那一步」测量跑写出的版本；说谎的设备让历史停在注入之后的另一步，停下那一步实现已经写出的根不在集合里。
- HEAD 扫描里 offset 503 同一形（前一条是「写的实例表行」对不上）。

### ⑤ 多出 I-3.11 那组（1 条，offset 285）：(iii) checker 的错

种子 7463871032432355398，写被吞，第 633 次写，第 27 步可写挂载（写行 txg 25 + 暖机 txg 26）。被吞的是写行那次发布里分配记录树一个节点在盘 1 上那一份（槽 50246，复用过的槽，里面是诞生代 3 的旧节点）。

- I-2.1 / I-3.10 / I-4.8 / I-7.4 四条与 ① 同一个投影：I-3.10 是分配代 25 对诞生代 3，I-4.8 / I-7.4 落在候选根 txg 25 上。
- **I-3.11 另有来由**，分三步：
  1. 实现一侧：同一步的暖机发布（txg 26）换下这个节点，释放之前读盘核。盘 1 那一份与位置项里的校验和对不上，重读仍对不上，按 D19 已定项 5 硬规则 1（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:106`、`:112`）隔离，两块盘上的记录都留在已分配。代码：`crates/singlefs-core/src/transaction.rs:2842` 比的是 `crc32_castagnoli(&copy) != location.unit_checksum`，即位置项里的校验和。
  2. 实证：不注入那一遍里，(6,26) 下这个槽的记录是「已释放、代 26」；注入那一遍里是「未释放、代 25」，两盘都是（`logs/one-285-dump.log`：`REOPEN records@(6,26) d0 s50246 span1 gen25 relfalse | d1 s50246 span1 gen25 relfalse`；不注入那遍 `U   records@newest(6,26) d0 s50246 span1 gen26 reltrue | d1 s50246 span1 gen26 reltrue`）。
  3. checker 一侧：I-3.11 的隔离豁免在 `crates/singlefs-checker/src/walk.rs:3272` `quarantined_slots_exempted_per_device`，判「这个单元有没有一份坏」用的是 `check_unit`（第 3318 行，magic、头校验和、载荷 CRC，即自证）。盘 1 那一份是一个完整的旧节点，自证照样过，于是不豁免。记账「已分配 − defer」比最新根走读多出这一槽（盘 0：2654208 − 2342912 = 311296，走读 294912，差 16384）。
- 判据两边不一致：实现按「与位置项里的校验和对不上」隔离，checker 按「自证不过」豁免。丢一次写落在复用槽上时，前者成立、后者不成立。I-3.11 那一行写的是「checker 自己读出来读不出或校验和对不上」（`.claude/kb/invariants.md:141`），没说对的是哪一个校验和。checker 取了自证；按 D19 已定项 5 的隔离条件，应当对位置项里的那一个。
- 重开 `CommittedByTheModel`（(6,26)，28573 字节）。

### ⑥ 多出 I-3.1 的那组（1 条，offset 176 第 766 次写）：(ii)

种子 7463871032432355289，写被吞，第 21 步可写挂载。

- 被吞的是 (6,29) 那条根的根槽 FUA 写：盘 0 偏移 7344128，区域 2 槽 1。根环 24 槽已经转过，那个槽里原来是 (1,5)，写被吞之后 (1,5) 还在（`logs/one-work-176-766.log`：重开时根环 `roots[24]=(1,5)F0@RootRingSlot { region: 2, slot: 1 }` … `(6,28)` `(6,30)`，没有 (6,29)）。
- 实现以为 (1,5) 已经被盖掉，环里最旧有效根往前挪了。按 D16 已定项 1「可再分配」（`16-发布语义.md:33`）回收只被 (1,5) 引用的槽，之后复用了其中的 50176（(1,5) 的账里它是分配代 0 的那一条，即 mkfs 时实例表的落点；现在盘 1 上那里是一个诞生代 30 的单元，是哪一类单元没核）。
- 于是五条都红在这条复活的旧根上：
  - I-4.8 / I-7.4：「候选根 txg 5」；
  - I-2.1：盘 0 槽 50176 的实例表对不上 (1,5) 的指针；
  - I-3.10：(1,5) 的账里 50176 分配代 0，槽里单元诞生代 30；
  - I-3.1：候选并集比记账多 8 槽（5013504 − 4882432 = 131072）。多出的是 (1,5) 独占、已被回收的槽，这一点是推的：差值方向与机理对得上，没有逐槽核。
- 条款：「环里最旧有效根」那一行写「写失败的槽按旧内容算」（`16-发布语义.md:34`），罩的是报了错的写。设备说谎时实现无从知道，属于设备丢的。
- 该归说谎的设备那一格。今天白名单里单独的 `["I-3.1"]` 罩不住，因为同一条旧根还连带了另外四条。
- 重开 `CommittedByTheModel`（(6,30)，0 字节）。最新根不是这条旧根，恢复不会择到它；管理员回退到它时，复活集逐份读回核 CRC 会拒（`mount.rs` 的 `every_copy_of_the_resurrected_units_reads_back`，按代码读，没另跑）。

### 与用户定案「fsync 失败后两个盘掉一个盘不能认」的关系

- 上面 24 条写被吞里，丢的都是一块盘上的一份。之后的写照样发往两块盘，重开读回的都是模型提交过的版本：读另一块盘上那一份，或施加 journal 记录。没有一条是「池退成只剩一块盘可用」那一形。
- ⑤ 里实现的读盘核把坏的那一份核了出来、按条款隔离。
- 丢掉的那一份之后没有修复，那个单元只剩一份好的，要等下一次改写或释放。这不在这批条款里，没查。

## 三、第 2 件：`every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking`

判 **(ii) 期望过时**，是实三（`/tmp/claude-1000/impl-rbf-3/`，`records/2026-09-24-里程碑二收尾调度.md:181`「`mount_rollback*` … 都删」）换掉随机历史的回退操作带来的。

- HEAD 上这条用例绿（`logs/bad-disk-head.log`：`test every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking ... ok`）；工作区上红，与实四乙原样一致（`logs/bad-disk-work.log`）。
- 历史变了：写死的那段历史是种子基 + 4、16 步。两份代码第 3 步（从 0 数）逐字对照（`logs/hist4-head.log:7`、`logs/hist4-work.log:7`）：
  - HEAD：`CloseAndMountRollback(RingRoot { index_from_newest: 2 })` → `Applied(Mounted { instance: InstanceGeneration(2), publishes: 3, …`
  - 工作区：`RollBackWhileMounted(RingRoot { index_from_newest: 2 })` → `Refused { member: "RollbackError::TargetNotACandidate(VersionWithoutFile)" }`
- 生成器的抽数没变，只是同一个抽数现在落到挂着时回退。环里第 3 新的是一条暖机根（无文件），按 D23 已定项 14「候选集 … ∧ 带文件」（`.claude/kb/decisions/23-journal的角色与格式.md:378`）被拒。之后第 4、5 步由「前提不满足」变成照常抬 F 被拒，第 6 步第一个文件被拒，第 11 步挂载取到实例 2 而不是 3。盘面因此回到用例注释（第 343–346 行）里说的 C511 之前那一版：实例表落点槽 50304、恢复所选根 (2, 9)。
- 过时的三处期望串（`logs/bad-disk-probe-work.log`，探针把断言换成逐条打印，其余 23 处照旧对得上）：
  - 第 10 条坏法，可写挂载那一侧：期望 `slot: SlotNumber(50368)`，今天 `50304`（第 407–408 行）；
  - 第 12 条坏法，恢复那一侧：期望 `FileRead（实例 3 第 13 代根`，今天 `FileRead（实例 2 第 9 代根，32633 字节）`（第 417 行）；
  - 第 12 条坏法，可写挂载那一侧：期望 `slots: [SlotNumber(50368), SlotNumber(50369)]`，今天 `[SlotNumber(50304), SlotNumber(50305)]`（第 418–420 行）。
  - 用例今天红在第 10 条，第 12 条被挡在后面还没走到。
- 错误成员一个没变。外层 `RowPublishAdmissionRefusedBeforeAcquisition` 在 HEAD 上就包着（HEAD 那次输出：`Err(RowPublishAdmissionRefusedBeforeAcquisition { instance_to_acquire: InstanceGeneration(4), cause: ReleaseTargetAlreadyReleased { … slot: SlotNumber(50368) } })`）。用例拿 `how.contains(member)` 比子串，包装不是红的原因。
- 第 343–346 行那段注释（「拿掉那道拒绝之后…挪到 50368…变成 (3, 13)」）也跟着过时。

## 四、HEAD 对照

取法：`git archive 73ba4a4 crates Cargo.toml Cargo.lock | tar -x -C /tmp/claude-1000/impl-rbf-4c/head`，另拷 `.cargo`。工作区副本用 `rsync -a --exclude target --exclude .git`。两份各用自己的 target。没做 git 写操作。

**扫描计数**（同一组大档规模，同一个探针 `zz_probe_scan.rs`）：

| 组 | 工作区 | HEAD |
|---|---|---|
| ① `["I-2.1","I-3.10","I-4.8","I-7.4"]` | 11 | 9 |
| ② `["I-2.1","I-4.8","I-7.4","I-9.1"]` | 7 | 4 |
| ③ 抬 F 的上限 | 3 | 0 |
| ④ 冷启动读回 | 2 | 4 |
| ⑤ `[…,"I-3.11",…]` | 1 | 0 |
| ⑥ `["I-2.1","I-3.1","I-3.10","I-4.8","I-7.4"]` | 1 | 0 |
| （只在 HEAD）模型说该成、实现拒了（都是抬 F 时 `WriteFails`） | 0 | 3 |
| （只在 HEAD）写的实例表行 | 0 | 1 |
| （只在 HEAD）`["I-2.1","I-3.1","I-4.8","I-7.4"]` | 0 | 1 |
| 合计：条数 / 有新发现的段数 | 25 / 23 | 22 / 20 |

大档那条 `#[ignore]` 用例在 HEAD 上跑同样会红。

**逐注入点重放**：扫描计数不能直接比，两份代码在同一种子上抽到的注入点不同。每步写几次变了，生成器也把回退操作换了——512 段里 336 段的 30 步操作序列在两份代码上逐步相同（回退操作按改名对齐），17 段前 26 步以上相同且没有回退（`logs/ops-work.log`、`logs/ops-head.log`）。所以 ③⑤⑥ 在两份代码上拿同一个逻辑注入点（第几步、这一步里第几次写或读）直接比：

| 组 | 注入点 | 工作区 | HEAD |
|---|---|---|---|
| ③ 339 | 种子基+339，第 145 次写被吞（(1,6) 根槽） | 上限 模型 3 / 实现 0 | 相同（`logs/one-339-head.log`） |
| ③ 445 的机理 | 种子基+339 第 1 步，盘 0 偏移 1048576 的读注 `ReadFails` 或翻位 | 接受 F=1、I-7.9 红 | 相同 |
| ⑤ | 种子基+232 第 29 步第 7 次写被吞（盘 0 槽 50371，分配记录树叶） | `["I-2.1","I-3.10","I-3.11","I-4.8","I-7.4"]`，I-3.11「4751360 减 4440064，不等于 294912」 | 逐字相同 |
| ⑥ 的机理 | 种子基+232 第 18 步第 31 次写被吞（txg 29 的根槽，区域 2 槽 1） | 旧根 (1,5) 留在环里；`["I-2.1","I-3.10","I-4.8","I-7.4"]`，I-3.10「盘 1 槽 50176 … 分配代 0 … 诞生代号是 30」 | 逐字相同 |

- 日志：⑤ 是 `logs/one-sweep-232-s29-w7-{work,head}.log`，⑥ 是 `logs/one-stale-232-s18-w31-{work,head}.log`，③ 445 见 §二。
- ⑥ 原样那一条（种子基+176）的五条签名没在 HEAD 上复出：那段历史第 7 步起就因回退操作换名而分叉。复出的是同一机理的另一条（这一条正好没有多余的 I-3.1）。
- 同一批构造里，种子基+232 第 19 步、第 28 步的根槽被吞在两份代码上都给出「写的实例表行」，与 HEAD 扫描 offset 503 同形；第 23、24、26、27 步给出白名单里的 `["I-3.1"]`（`logs/one-stale2-232-*`）。

**判新旧**：六组没有一组是这一轮新带进来的。③ 的上限、⑤ 的隔离判据、⑥ 的回收、④ 的胶水，在 HEAD 与工作区上对同一注入点的行为逐字相同。扫描计数差是抽样差。

## 五、交主 agent 的问题（条款没写、或不归我定）

1. **③ 445（(i)）的修法**：抬 F 算上限时，一个根槽读不出或自证不过，怎么处置？
   - 候选：照 D19 N1 重读一次；照树表那一格读不出就拒（`RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`）；或别的。
   - 条款没写怎么分辨「崩溃撕裂的根槽」（该当它不存在）和「这一次读坏」（不该）。
   - `visit_valid_roots_with_ring_slots` 还有别的调用方（择根、回退候选），一读坏就丢一条根这件事在那些地方的后果我没查。
2. **白名单要不要改**（①②⑥、③ 339 / 284、④ 284 都卡在这里）：`INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE` 是用户 2026-09-21 定的「写死两组、名单外照报」，改它要用户点头。几种改法（推的，都没实现）：
   - (a) 名单加 ① ② ⑥ 三组；
   - (b) 不按签名，按注入点认：每条违例点名的（盘, 槽）就是被吞那次写的，或者红在被吞根槽里留下的那条旧根上；
   - (c) 「抬 F 的上限」按模型根环去掉被吞的根之后重算的上限比；
   - (d) 重开的放行集并进「历史停下那一步」实现已写出的根。
   - 不改的话，大档今天与 HEAD 上都红。快档（24 段）在实四乙交回前的主工作区那次运行里绿（`/tmp/claude-1000/impl-rbf-4b/logs/final-second_transaction_supplement_three_fault_injection.log:162`），这一轮我没跑快档。
3. **⑤（(iii)）**：`walk.rs:3318` 的豁免判据改成对位置项里的校验和（与 D19 已定项 5 硬规则 1 同一个），还是改 I-3.11 / I-3.1 那一行的措辞（`invariants.md:141`、`:131`「校验和对不上」没说对哪一个）？
   - 两条不变量用的是同一个函数，I-3.1 那一侧大概率有同样的缺口。
   - HEAD 扫描那条 `["I-2.1","I-3.1","I-4.8","I-7.4"]`（offset 309，树表单元写被吞）可能就是它，我没核。
4. **④ 330（(iii)）**：`history.rs:2880` 改用施加之后实际走的根，是 `crates/` 改动，归下一批实现。
5. **第 2 件**：用例第 343–346、407–408、417、418–420 行照今天的历史改，还是换一段不经回退的写死历史，由你定。
6. 「写的实例表行」这一形：今天的大档扫描没抽到，构造的注入点上两份代码都给出。没判，下一次大档可能撞上。

## 六、推翻条件

- ①②：出现一条，I-2.1 或 I-3.10 点名的（盘, 槽）不是被吞那次写的，就不是纯投影，要重判。
- ③ 339 / 284：把被吞的根从模型根环拿掉之后重算的上限，不等于实现报的，就不是这一形。
- ③ 445 判 (i)：条款明写「上限按这一次读得出的根算」，就改判 (ii)，成了设计问题。今天的实据是构造那一次 I-7.9 红：F 被抬过条款上限，它是盘上的违例，不是模型的读法。
- ④ 330 / 284：重开读回的内容哈希不等于模型那一版（330 的 (5,19)、284 的 (3,22)），就是读回了没确认的内容，要改判。今天两条都相等。
- ⑤：注入那一遍 (6,26) 下 50246 的记录是「已释放」，就不是隔离判据不一致，要另查。今天是「未释放、代 25」。
- ⑥：重开时根环里没有 (1,5)，就不是旧根复活。今天有。
- 新旧：同一逻辑注入点在 HEAD 与工作区上结局不同，就有新带进来的成分。今天两份代码都跑过的 10 个逻辑点位（339 那次写被吞、339 第 1 步那次读、232 第 18 / 19 / 23 / 24 / 26 / 27 / 28 / 29 步；第 29 步比了第 7、11、13 次写三处）结局全部相同。
- 第 2 件：HEAD 上那条用例红，或两份代码第 3 步的操作与结局相同，就不是这个原因。

## 七、跑了什么（命令与原样末行）

负载：开工时 `ps` 没有 cargo、qemu、fio、gate 在跑，没等锁。收到「线程上限 16 → 6」之前起的两趟扫描用 `capped.sh 16`、探针内部 14 线程；之后一律 `capped.sh 6`。

- 编译（不经内存包装）：每份 `cd <副本> && CARGO_TARGET_DIR=<副本自己的 target> nice -n 19 bash research/scripts/capped.sh <16|6> cargo test --offline --release -p singlefs-harness --test <探针> --no-run`。末行都是 `exit 0`（`logs/build-*.log`）。
- 大档扫描：
  - 命令：`cd <副本> && SCAN_START=0 SCAN_COUNT=512 SCAN_THREADS=14 CARGO_TARGET_DIR=<target> nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 16 cargo test --offline --release -p singlefs-harness --test zz_probe_scan -- --ignored --nocapture`
  - 工作区：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 203.81s` / `exit 0`
  - HEAD：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 170.47s` / `exit 0`
- 第 2 件那条用例，直接执行编出来的测试二进制（`… run-with-memory-cap.sh 8G bash … capped.sh 16 <二进制> every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking --exact --nocapture`）：
  - HEAD：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.97s` / `exit 0`
  - 工作区：`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.75s` / `exit 101`
  - 逐条打印的探针 `zz_probe_bad_disk.rs`：工作区 `MATCH` 23 处、`MISMATCH` 3 处；HEAD `MATCH` 26 处。
- 单注入点深探针：`bash one.sh <work|head> <名> SEED_OFFSET=… CALL=…|SEGMENT_STEP=… WITHIN=… FAULT=…`，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 6`。
  - `batch-points.sh`：24 个注入点 × 两份代码，48 次全部 `exit 0`（`logs/batch-points.log`）；
  - 构造点位：`batch-stale.sh`、`batch-stale2.sh`、`batch-sweep.sh`，逐次末行都是 `exit 0`。
- 读注入摆在点名步里的探针：`SEED_OFFSET=339 OPS=30 ARM_STEP=1 DEVICE=0 AT=1048576 FAULT=read_fails [NTH=2] … zz_probe_armed_at_step-… --ignored --nocapture`。
  - 工作区 NTH=2：`test result: ok. 1 passed; … finished in 0.04s`；HEAD NTH=1：`… finished in 0.03s`；
  - 探针本身只打印不断言，红不红看日志里的 `ENDING` 与 I-7.9 行。

## 八、没做什么

- 一行 `crates/`、`crates/mutations.tsv` 都没改；没提交，没做 git 写操作（`git archive` 只读）。
- 没改代码，所以没跑 fmt、clippy、build 的交回验证，也没跑登记给实现员的门禁阶段。
- 层 0、QEMU、herd7、变异表、全量 `cargo test`、`check.sh`、`gate.sh` 都没跑。故障注入快档这一轮没跑。大档那条 `#[ignore]` 用例本身没跑：用的是同参数的探针，逐段调同一个 `inject_faults_into_history`，执行参数 `CHECKED_ON_FOUR_GIBIBYTE_DEVICES` 与大档那条的三项相同。
- 没判的：
  - HEAD 独有三组（模型说该成、实现拒了 ×3；写的实例表行；`["I-2.1","I-3.1","I-4.8","I-7.4"]`）；
  - 构造时见到的「写的实例表行」那一形；
  - ⑥ 原样五条签名没在 HEAD 上复出，只复出了机理；
  - `visit_valid_roots_with_ring_slots` 的其余调用方；
  - 「丢一条根只会压低上限」是推的，没穷举；
  - I-3.1 一侧是不是也有 ⑤ 的豁免缺口，没核。
- 草稿产物（日志、探针、脚本）没入 `research/results/`：我的写范围只到 `crates/`、`litmus/` 与 `/tmp/claude-1000/`，这一轮又规定不改 `crates/`。要留的话由主 agent 拷。

## 九、草稿与清理

交回前删掉的（删前 `du -sh`）：
- 编译目录 `target-work` 862M、`target-head` 853M、`target-work-p` 1.1G、`target-head-p` 1.1G；
- 仓副本 `work/` 5.9M、`head/` 5.8M。

删前 `ps` 查过，没有进程在用它们。留下的（目录合计 11M）：
- 这份报告、`progress.md`；
- `logs/`：扫描、深探针、构造点位、第 2 件；
- `probes/`：探针源码 `zz_probe_scan.rs`、`zz_probe_history.rs`、`zz_probe_one_fault.rs`、`zz_probe_one_fault_head.rs`、`zz_probe_armed_at_step.rs`、`zz_probe_ops.rs`、`zz_probe_bad_disk.rs`；
- 脚本：`one.sh`、`batch-*.sh`、`summarize.py`、`points.txt`；
- `supp3.txt`：里程碑「增补 3」那一节的摘录，只供阅读。
