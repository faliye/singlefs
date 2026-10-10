# m3-c694-r1 云端正推（Sonnet）报告：K1、K3、K4（2026-10-07）

立场：从条款与冻结副本 `~/.local/share/singlefs/work/m3-c694-r1/tree/` 今天的实现推「对的世界」，给候选的定义与落点；不替主 agent 采纳。K2（候选代价的攻防）不归我，只在落点上标「要碰的崩溃点」。
引用写法：kb 写 `文件@小节标题`，代码写冻结副本里的 `crates/…:行号`（行号都是这一轮 `grep -nF` 现取的）。草稿目录 `~/.local/share/singlefs/work/m3-c694-r1-sonnet/`（仓副本 `repo/`、编译目录 `target/`、日志）。

## 零、读到的前提（代码，主 agent 观测复核）

- 影子账的算法只有一处：`crates/singlefs-core/src/mount.rs:1483`「fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(」。三个入口都进它：可写挂载 `crates/singlefs-core/src/mount.rs:2218`「ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(」；抬 F 那一串 `crates/singlefs-core/src/mount.rs:3511`「match isolate_slots_referenced_only_by_abandoned_roots(」，而抬 F 串有两个调用方：会话抬 F `crates/singlefs-core/src/mount.rs:2907`「let raised = raise_the_floor_through(」，卸载 `crates/singlefs-core/src/mount.rs:2958`「let raised = raise_the_floor_through(」。所以「卸载被拒、会话写被拒、再可写挂载被拒」是同一处判定的三个入口，改一处三处一起变。
- 拒的那一支：账读不出就计数 `crates/singlefs-core/src/mount.rs:1524`「unreadable += 1;」，从映射现算不成，重读一次仍不成，`crates/singlefs-core/src/mount.rs:1542`「None => return Err(」，构造 `crates/singlefs-core/src/mount.rs:1544`「StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {」。
- 抬 F 串里读根环那一遍是现读：`crates/singlefs-core/src/mount.rs:3498`「let roots = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;」。所以 C694 那条根（挂载那一刻槽读回全零）在卸载时「读得出、按实例表被抛弃」，进了 `roots`，才有账可读。
- 复现（我在自己的副本上复跑，编译目录自己的，今天，日志 `run-01-baseline-min.log`，原样）：最小一条 `MINIMAL unmount=Err NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(9) } })`，末行 `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.53s`。

## 一、K1　C694：对的世界是什么

### 1.1 条款今天各管到哪一半（整行抄，行号去 kb 文件现查）

被抛弃根的账读不出时的处置，条款原话（D23 已定项 14 正文）：
`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「被抛弃根的账（树表或分配记录树）读不出时不跳过：从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报，映射也现算不成就重读一次，仍不成在取号之前拒可写、只读照常」。
同一节里条款自己认下的代价：`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载」。
C554 乙-配置续只管系统配置见证过的那一半，没见证到的那一半条款写明留着：`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「影子账留着的理由收窄成系统配置没见证到的那一形」。
D16 已定项 1 对全零槽的口径：`.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读`：「读回全零的槽不并进上限、F 可能越过住在全零槽里的有效根（用户认的代价）」；同一处：「读坏的并进准入抬 F 的上限，读回全零的不并」。

### 1.2 今天的盘面上量到的（我的副本，harness 档）

副本：`~/.local/share/singlefs/work/m3-c694-r1-sonnet/repo-base/`（冻结树原样加调查员三份用例草稿与我的探针 `crates/singlefs-harness/tests/m3c694_probe.rs`）。探针每格做四件事：起点盘面上池级 checker（`singlefs_checker::walk::check_pool_image`）的红名；正常卸载（真盘）；卸载之后再可写挂载加卸载；另造一份同一起点、会话里接着覆盖写 40 次，记第一次被拒是第几次、被抛弃的根还在不在环里、最后盘面的红名。格 = 宽 384 / 256 / 240 × 第一个文件之后覆盖写 k ∈ {1,2,3,5,8} × 挂载后再覆盖写 k2 ∈ {0,1,2,4}，共 60 格（4 GiB 那档调查员量过 0 拒，没重跑）。

命令（每档宽一条，三档；日志在草稿目录）：`C694_WIDTHS=<宽> bash tools/tb.sh run-07-probe-base-<宽> repo-base m3c694_probe m3c694_probe_after`。计数（`cat run-07-probe-base-*.log | grep -c …`，今天复跑，原样）：

| 数（60 格里） | 今天（base） | 候选甲（W1，废根放行） | 候选乙（W4，点名落点预隔离） |
|---|---|---|---|
| 起点造得出（`PROBE W=` 行去掉 `START_FAILED`） | 58 | 59 | 58 |
| 起点盘面池级 checker 红 I-7.4 | 44 | 45 | 0 |
| 正常卸载 `unmount=Err` | 11 | 0 | 0 |
| 卸载之后再可写挂载 `remount=Err` | 11 | 0 | 0 |
| 会话覆盖写第一次被拒、原因是 `cause: NewerStateStillUnreadableAf…` | 11 | 0 | 0 |
| 会话覆盖写第一次被拒、原因是 `NoSpaceAfterRaisingTheFloor` | 15 | 15 | 22 |
| 会话覆盖写 40 次之后盘面池级 checker 无红（`reds_at_end=[]`，会话 59 / 58 / 58 格） | 35 | 47 | 58 |

读法：
- 今天 11 个拒与调查员报的一致（`grep -c 'unmount=Err'` 出 11），都是 k = 5、宽 384 / 256 / 240，被抛弃的根 txg 9。
- **44 / 58 个起点在拒不拒之前，池级 checker 已经红 I-7.4（近 K 代块未被复用）**：被抛弃的根引用的单元在取新实例那次挂载里被复用，是 11 个拒之外的更宽一片；拒只落在其中树表与中央映射根都被写掉的 11 个。所以今天的拒从来没有「护住一份绿的盘」，它拒的时候盘面早已红了。
- 今天这 11 个起点（会话那一遍的 11 格，宽 384 / 256 / 240 各 4、4、3）会话覆盖写做了 0 到 13 次就被拒，被抛弃的根始终留在环里（`abandoned_still_in_ring=true`，现行 txg 停在 24 / 15 / 13），盘面一直红 I-7.4；甲之下 40 次都做成（这 11 格 `reds_at_end=[]`），被抛弃的根离开根环，盘面转绿。
- 另 15 个 `NoSpaceAfterRaisingTheFloor` 全在宽 240（k = 1、2、3、8），今天与甲相同，不属于 C694 这一拒。乙多出的 7 个也是 `NoSpaceAfterRaisingTheFloor`（宽 240 的 k = 5 与宽 256 的 k = 8）：乙在副本里把点名落点隔离到进程结束（没做「根槽被覆盖时放开」），是代价的上界，见 1.5。

### 1.3 对的世界（我推出的）

三个问题各答一句，再列依据。

**（一）这几份盘面上正常卸载、会话写、再可写挂载该不该做得成：该做得成（登记行的 U1）。**
依据：
1. 拒不护任何东西。今天 44 / 58 个起点在做任何卸载之前，池级 checker 已经红 I-7.4（见 1.2）：被抛弃根引用的单元在取新实例那次挂载里已经被发出去写掉。拒发生在那之后，拒掉的是此后每一次抬 F、卸载、再挂，换不回已经被写掉的单元。
2. U2 的后半句（拒得合法、之后某一次卸载做得成）在产品路径上不成立，是个自己锁死自己的闩：被抛弃根离开根环只能靠后面的发布，而发布要先过准入推抬 F（同一处影子账），再挂也过同一处。量到：11 个里 11 个再可写挂载被拒（`remount=Err`），会话覆盖写做了 0 到 13 次就被拒，被抛弃的根留在环里不动。只有「进程永不退出、池大到从不需要推抬 F、一路写到那条根被轮出环」才走得出，这不是产品路径。`crates/` 里也没有修复入口：`grep -rniE 'fn (repair|scrub|force_unmount|salvage)' crates/singlefs-core/src` 零命中（今天复跑，输出 `hits=0`）。
3. 条款认下的「只剩只读挂载」是给另一类读不出的：`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换` 里写「C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载」，挂在撕裂的系统配置轮换、持续读错那一串代价后面，说的是读错、读不到；账的单元被这个池自己的新实例写掉那一形没有被它点名。

**（二）被抛弃根的账读不出、它引用的槽已经被新实例写掉时，影子账还有没有东西要护：对被写掉的那几个单元没有；对这条根「还能不能被打开」有一个判据。**
- 影子账护的是「被抛弃的根万一被恢复择中，要还读得出来」（D23 已定项 14「射程」里「影子账与按实例表判抛弃（`abandoned_by_table`）留着，理由是崩溃恢复，不是管理员回退」那一条：恢复落在被抛弃的根上读不出，第一轮 H6）。一条根要被打开，先要读到它的树表与中央映射根两个根指针（`crates/singlefs-core/src/mounted_read.rs:376` 先读映射根、`crates/singlefs-core/src/mounted_read.rs:403` 读树表；冷走读 `crates/singlefs-core/src/recovery.rs:4836` 读树表）。这两个单元任一个永远读不出，这条根就永远打不开，它名下剩下的单元护不护都不改变「打不开」。
- 「永远读不出」要有凭据，不能凭「读不出」本身：读错、读回全零、自证不过都可能是瞬时的（D23 已定项 14 那一段「重读一次」），不能当凭据。能当凭据的是：那个槽上读回来的是一个读得出、自证过、诞生身份（诞生 txg、实例、出生序号）与指针不同的码 2 节点，说明槽已被后来的单元占了。我在副本里按这个判据做了甲（见 1.4），全网格的 11 个拒都放行，而调查员的推翻条件那一格（把树表槽与映射根槽写零再卸载，写零不是被后来的单元占了）照拒：`FALSIFIER zero tree_table+mapping_root unmount=Err NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot …)`（`run-02-w1-min.log`）。
- 不护会误放的一形（举一形，丙，见 1.4）：把「重读一次仍读不出」一律改成「计数、放行」。造法：被抛弃根的树表与映射根这两个单元没被写掉，只是这个挂载期间两遍都读不出（瞬时故障比一次重读长）；放行之后不隔离它引用的槽，新实例发出去；故障撤了，再挂时这条根读得出而它的单元已被复用，恢复一旦落到它上面（新实例的根都坏了）读不出，就是 C314 / C554 要挡的丢数据。我没把这一形做成跑通的用例（没量），推的；它是甲要带「被后来的单元占了」那道凭据、不能只凭读不出的理由。

**（三）挂载那一刻根槽读回全零而之后读得出：D16 与 D23 各管到哪一半、哪一半没人管。**

| 一半 | 谁管 | 管成什么样（行号去文件现查） |
|---|---|---|
| 回收门槛：全零槽可能住着有效根，钉在它可能住过的最旧 txg | D16 已定项 1「根槽这一次读坏」那一行的「读坏的槽与读回全零的槽」 | 实现 `crates/singlefs-core/src/allocator.rs:995`「AllZeroAtMount {」；管的是**已释放**的槽什么时候可再分配 |
| 准入抬 F 的上限：全零槽不并 | 同一行与 D16「抬 F 的上限」那一行 | 「读回全零的槽不并进上限、F 可能越过住在全零槽里的有效根（用户认的代价）」 |
| 系统配置见证过比所选那一版新的发布，读阶段重读、仍真则拒可写 | D23 已定项 14 的 C554 乙-配置续 | 只管见证到的那一半；条款自己写「影子账留着的理由收窄成系统配置没见证到的那一形」 |
| **没见证到的那一半：根槽落了、系统配置轮换没落；根槽读回全零** | **没人管** | 挂载看不见这条根，不隔离它的槽。它那次发布**新分配**的单元（树表、映射根、点名单元）从没释放过，不在回收门槛管的「已释放」集合里，所选版本的账里又没有，当空闲槽发给新实例。`crates/singlefs-core/src/allocator.rs:988` 注释自己写「读不出的被抛弃根影子账本来就罩不到」 |
| 之后这条根读得出、成了被抛弃根、账读不出（单元被自己写掉了） | **没人管**（条款只写了读不出的一般处置） | 今天归到 D23 已定项 14 的「重读一次，仍不成在取号之前拒可写」，不分「瞬时读不出」与「被后来的单元占了」 |

两个「没人管」就是 C694 的全部：第一个造成损坏（44 / 58 个起点红 I-7.4），第二个把损坏变成拒绝。

### 1.4 候选的定义与落点（只写定义、落点、碰什么；攻防归 K2）

四个候选，全部在冻结树的副本里做成了（`repo-sw/`，候选开关 `crates/singlefs-core/src/mount.rs` 里的 `candidate_switches`，默认全关 = 今天的实现）。落点行号是冻结树 `crates/singlefs-core/src/mount.rs` 的。

**甲（W1，废根放行）**。定义：影子账里一条被抛弃根账读不出、从映射现算不成、重读一次仍不成时，再判一条——这条根的树表指针与中央映射根指针，每条位置条目读回来都是读得出、自证过（`parse_index_node` 过）、而诞生身份（诞生 txg、实例、出生序号）与指针里写的不同的码 2 节点（槽已被后来的单元占了）；两个指针都是 ⇒ 这条根已不可能被打开，计进 `abandoned_roots_unreadable`、落点表记成空、不拒可写；任一个不是（读错、读回全零、自证不过、同诞生身份而校验和不符、指针是空指针）⇒ 照今天拒。落点：`crates/singlefs-core/src/mount.rs:1542`「None => return Err(」前加一个守卫分支，加一个只读判定的新函数；三个入口（`mount.rs:2218`、`mount.rs:2907` 与 `mount.rs:2958` 经 `mount.rs:3511`）同时变。副本改动 `patches/w1-dead-root.diff`，58 行。碰的崩溃点：没有新的落盘写（判定只读），挂载取号之前与抬 F 串第一次发布之前这两个位置的读多几次。要新条款：要，D23 已定项 14 影子账那一句补「树表与中央映射根都已被后来的单元写掉的被抛弃根按不可打开处理」，C393 那一支的拒只留给「读不出而不能证明被占了」。

**乙（W4，点名落点预隔离）**。定义：可写挂载读阶段之后，水位之上（`(checkpoint_txg, 实例)` 大于所选那一版）、没被施加的记录点名的落点——新根段里的树表、映射根、实例表、分配记录树根四个指针与全部点名项，逐盘同槽——在这个进程里先隔离，新实例不发。取新实例那次挂载影子账看不见这条根，这些记录是它唯一留下的账：量到取新实例那次挂载看到的盘面上，14 个起点每个都恰有 1 条这样的记录（`JOURNAL … records_above_water=1`，点名 10 个单元，且点名的槽包含被抛弃根的树表槽与映射根槽，`names_tree_table_slot=true names_mapping_root_slot=true`，日志 `run-08-journal-probe.log`）。落点：`crates/singlefs-core/src/mount.rs:6083`「let rebuilt = rebuilt_allocator(」之后、`allocator.set_deletion_switches` 之前，用读阶段已经有的 `records` 与 `effective_root`，不用另读盘；一个新函数。副本里隔离到进程结束（偷懒），正确的寿命应是「那一条根的根环槽被这个进程写掉时放开」，挂在 `RootRingOccupant::AllZeroAtMount`（`crates/singlefs-core/src/allocator.rs:995`）上，让它带落点、被写掉时走 `clear_isolation_no_abandoned_root_in_the_ring_backs` 那条路（没做，推的）。要新条款：要（D16 已定项 1 那一行或 D23 已定项 14）。碰的崩溃点：不写盘，只多隔离几个槽，但挂载之后的空间账变了（见 1.5 的代价）。**它防的是损坏本身**（44 / 58 个起点起点就不红），代价是只在 journal 记录还在环里时有效。

**丙（W2，读不出一律放行）**，对照用，不推荐。定义：C393 那一支「重读一次仍不成就拒」改成「计数、放行」。落点同甲，守卫换成无条件。要新条款：要，推翻 D23 已定项 14 现行那一句。

**丁（U2，条款写明拒得合法，不改 `crates/`）**。要求「之后的某一次卸载做得成」。落点：没有——`crates/` 里没有修复入口（1.3 的 grep `hits=0`），而再挂与会话写都过同一处，这条根走不出环；产品路径上 U2 后半句成立只在「进程永不退出」时。

### 1.5 代价与分辨（有量的写量，没量的写「推的」）

- **丙造得出一形丢数据（量了）**：`m3c694_a_temporarily_unreadable_account_under_each_candidate`（`repo-sw/crates/singlefs-harness/tests/m3c694_switches.rs`，日志 `run-10-temporary.log`）：W4 开着造一个单元完好的被抛弃根（384 槽、覆盖写 5 次），把它的树表与中央映射根的两个槽（两块盘，4 个副本）写零当作「这次挂载期间读不出」，再可写挂载，会话覆盖写 `writes` 次，然后「故障撤了」（仍全零的槽还原）。原样（节选，同一行里只留结局）：

```
TEMPORARY writes=1 W1=false W2=false … mount=Err… session_writes_done=0 account_unit_copies_overwritten_by_the_session=0/4 reds_after_the_fault_is_gone=[]
TEMPORARY writes=1 W1=true W2=false … mount=Err… session_writes_done=0 account_unit_copies_overwritten_by_the_session=0/4 reds_after_the_fault_is_gone=[]
TEMPORARY writes=1 W1=false W2=true … mount=Ok abandoned_unreadable=1 session_writes_done=1 account_unit_copies_overwritten_by_the_session=4/4 reds_after_the_fault_is_gone=[I-7.4]
```
今天与甲都拒挂载、盘不动、故障撤了仍绿；丙放行，会话写一次就把被抛弃根的 4 个账单元副本全覆盖，故障撤了池级 checker 红 I-7.4。覆盖写 3 次、6 次同样（`TEMPORARY writes=3`、`writes=6` 各三行），20 次时那条根已被轮出环，红消失（该行 `reds_after_the_fault_is_gone=[]`）——所以窗口是这条根在环里的那段时间。这也是甲为什么要带「槽被别的合法节点占了」这道凭据：写零那一格甲照拒。
- **甲不放过今天拒得对的盘面（量了一半）**：调查员的推翻条件那一格（树表与映射根槽写零）甲之下照拒，输出见 1.3（二）。读错、读回全零、自证不过、同诞生身份而校验和不符这几种我没各造一个盘面，只是判定函数里它们都落到「不算被占」（推的，读码）。
- **甲 / 乙 / 丙之下 31 条相关既有用例的结局**：`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable`（8）、`an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping`（13）、`root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread`（7）、`unreadable_abandoned_root_slot`（3）：今天（`regress-base-*.log`）与甲（`regress-w1-*.log`）都全绿，各一行 `test result: ok. 8 / 13 / 7 / 3 passed`。乙与丙这四个目标没跑（乙会改 `unreadable_abandoned_root_slot` 里钉着「没见证到的被抛弃根不隔离、不计数」那条的判据，应当红，推的，没跑）。
- **乙的空间代价（量了上界）**：会话覆盖写 40 次内第一次被拒是 `NoSpaceAfterRaisingTheFloor` 的格数：今天 15、甲 15、乙 22（多出的 7 格在宽 240 与宽 256）；副本里乙隔离到进程结束，是上界，寿命改对之后只会少。
- **乙没覆盖的盘面（推的）**：被抛弃那次发布的 journal 记录也不在盘上（读不出、或已被环转走）时没有可隔离的落点，乙无效；甲仍然接得住「已经损坏」那一半。已经按今天的代码损坏、躺在盘上的池，只有甲能放行。
- **丁**：无代价数可给，因为它没有出路。
- 没量：甲在 `unreadable_abandoned_root_slot` 之外的整套 harness 档回归（只跑了上面 4 个目标，31 条）；任何 checker 档（不归我跑）。

## 二、K3　C695：树表 0 条那一版

### 2.1 先读空映射根是不是 bug：是

- 条款：根记录里中央映射树根指针那一行（字段表「中央映射树根指针」，`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 7：根记录的字段表`：「mkfs 与暖机写全 0」）；树表 0 条的版本没有映射条目可放，全零就是「没有这棵树」。代码里「没有根的树」的约定也是整条指针全 0：`crates/singlefs-core/src/pointer.rs:289`「/// 没有根的树：整条指针全 0（I-2.5 对全零指针整体豁免升序判定）。」
- 实现自己对这个约定有两种读法。多数读者先看树表、树表 0 条就收口，映射树要用时才读（见 2.3 的表）；池级 checker 把全零的映射根当「这棵树走不了」、不记失败：`crates/singlefs-checker/src/walk.rs:1920`「if pointer.all_zero {」。只有 `open_pool_for_read` 一处先读映射根：`crates/singlefs-core/src/mounted_read.rs:376`「let mapping_tree = read_code_two_tree(」，空指针当成真指针读盘 0 槽 0；它自己的枚举写着应当走到的那个错：`crates/singlefs-core/src/mounted_read.rs:195`「/// 树表 0 条：这一版还没发布过文件版本，一个文件都打不开（不是坏盘）。」，而在它之前就已经报了坏盘的错，树表那一判 `crates/singlefs-core/src/mounted_read.rs:410`「if tree_table.entries.is_empty() {」一次都走不到。`grep -rn TreeTableHasNoEntries crates/` 只有定义与这一处构造，没有一条用例钉它（今天复跑：`grep -rn 'TreeTableHasNoEntries' --include=*.rs crates` 命中 2 行，`mounted_read.rs:196` 与 `mounted_read.rs:411`）。
- **范围比 C695 写的宽**：两块盘都在时，根环里每一条树表 0 条的根（mkfs 第 0 代 (0, 0)、暖机 (1, 1)、写行 (1, 2)）单独 `open_pool_for_read` 都报 `Walk(UnitUnreadable { slot: SlotNumber(0) })`。我复跑（`m3c695_switches.rs` 的 `c695_every_ring_root_opened_with_both_devices_present`，`run-11-c695-none.log`，原样）：

```
RING region=0 slot=0 on_dev0 root=(0,0) mapping_root=[d0s0,d0s0] open_both_devices=Err Walk(UnitUnreadable { slot: SlotNumber(0) })
RING region=0 slot=1 on_dev0 root=(1,3) mapping_root=[d0s1481,d1s1481] open_both_devices=Ok
RING region=1 slot=0 on_dev1 root=(1,1) mapping_root=[d0s0,d0s0] open_both_devices=Err Walk(UnitUnreadable { slot: SlotNumber(0) })
RING region=2 slot=0 on_dev0 root=(1,2) mapping_root=[d0s0,d0s0] open_both_devices=Err Walk(UnitUnreadable { slot: SlotNumber(0) })
```
  所以一个建好、暖过机、还没写第一个文件的池，只读挂载（它的最新根就是 txg 2）今天也报这个像坏盘的错；C695 的单盘幸存只是让恢复落到了这一版。推翻「是 bug」的现象：有条款写明树表 0 条的版本在只读挂载下就该报 `Walk(UnitUnreadable …)`；我没找到（kb 里 `只读挂载` 与树表 0 条同现的条款零命中，见 2.2）。

### 2.2 对的结局：报 `TreeTableHasNoEntries`；换根与「挂成空」不取

条款没有写这一格（C695 登记行自己也写「条款写明不承诺」是一条出路）。从已定的几处推：
1. **不换根**。D16 已定项 1「正常卸载」那一行写明（`.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读`：「恢复择根不看 F，丢一块盘时择的是剩下那块盘上最新的根」），用户 2026-10-03 定甲 GA；单盘幸存那块盘上最新的根就是 txg 2，别的根环槽里没有带文件的根（本例带文件的 txg 3 在丢掉的 0 号盘上）。换根要推翻这条定案。
2. **报错名而不是「挂成空」**：可写挂载与冷走读把树表 0 条当「没有文件」的合法状态（`crates/singlefs-core/src/recovery.rs:2510`「if tree_table.entries.is_empty() {」交 `RebuiltVersion::WithoutFile`，`crates/singlefs-core/src/recovery.rs:4846`「if tree_table.entries.is_empty() {」交 `Ok(None)`），卸载在这一版上一个字节不写、做成（D16 已定项 1「正常卸载」那一行末尾）。只读挂载若挂成空，`MountedPoolForRead` 的 `extent_tree`、`inode` 根等字段都不是 `Option`（`crates/singlefs-core/src/mounted_read.rs:333`），要改结构；而 `TreeTableHasNoEntries` 与它的文档已经在，且「不是坏盘」那句把它与读不出分开了。所以推出的最小结局是：这一版只读挂载报 `Open(TreeTableHasNoEntries)`。「挂成空」是岔路：可以做，要改 `MountedPoolForRead` 的字段与 `open_file`，我没做、没量。
3. 与 C683 衔接：C683 要的是池级 checker 能读单盘镜像、并且「分得开的观测只有只读挂载与读回（`DataUnitChecksumMismatchEverywhere`、`ERR:Open`）」（`.claude/kb/checks-owed.md` C683 那一行）。改成 `TreeTableHasNoEntries` 之后 `ERR:Open` 这一类里「这一版没有文件」与「盘坏」分开了，是 C683 读法的前提而不是与它冲突；单盘幸存的承诺范围（这一形算不算承诺）仍归 C683 先定，这一格不代它定。与 C563 衔接：C695 的择根没有经过「读坏当没有根」——剩下那块盘上 (1, 1) 与记录重建出的 (1, 2) 都是正常择到的（`recover_effective=Some((1, 2))`，见调查员与我的输出），读坏当没有根（C563）在这条链上没有出场。

### 2.3 同一版在别的入口的顺序（逐处，行号冻结树）

| 入口 | 先读什么 | 树表 0 条怎么走 | 映射根全零怎么走 |
|---|---|---|---|
| 只读挂载 `mount_read_only`（`crates/singlefs-core/src/mounted_read.rs:578`）→ `open_pool_for_read`（`mounted_read.rs:362`） | 映射根 `mounted_read.rs:376`，再树表 `mounted_read.rs:403` | 树表那一判 `mounted_read.rs:410`，走不到 | 当真指针读盘 0 槽 0，报 `UnitUnreadable { slot: 0 }` |
| 可写挂载重建上一版 `rebuild_version_in_the_unit_area_starting_at`（`crates/singlefs-core/src/recovery.rs:2492`） | 树表 `recovery.rs:2504` | `recovery.rs:2510` 交 `WithoutFile` | 映射树到第一次要回退时才读（`recovery.rs:581` 的 `OnceCell`），0 条版本不碰 |
| 冷走读 `walk_to_file_in_the_unit_area_starting_at`（`recovery.rs:4799`）、恢复 `recover` | 实例表、树表 `recovery.rs:4836` | `recovery.rs:4846` 交 `Ok(None)` | 同上，懒读 |
| 影子账读被抛弃根的账 `placements_referenced_by_root`（`crates/singlefs-core/src/mount.rs:1662`） | `tree_table_has_no_entries` | `Ok(true)` 那一臂取根记录里的指针，不读映射 | 不碰 |
| 影子账的映射现算 `placements_referenced_by_root_computed_from_the_mapping`（`mount.rs:1593`） | 映射树 | 不先判 | 当真指针读，交 `None`（注释自己写「映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内」） |
| 管理员回退判候选 `mount.rs:6380` | `tree_table_has_no_entries` | 交 `RollbackCandidateExclusion::VersionWithoutFile`，这一版不是候选 | 不碰 |
| 池级 checker `walk_tree_table_and_central_mapping_root`（`crates/singlefs-checker/src/walk.rs:1630`） | 树表，再映射根 | 树表读得出就往下 | `walk.rs:1920` 不记失败，`NotWalkable` |

结论：**两处是离群的**，`open_pool_for_read`（C695）与映射现算（C694 一侧，K4 里再提）；其余入口要么树表先判、要么映射树懒读、要么全零当没有树。

## 三、K4　回扫：「读回全零的根槽当没有根」还有哪几处在用

前提的字面：根槽读回全零（或读不出、自证不过）就当那一槽没有根，往下走。读根环的地方全仓只有下面几处，我逐处读了代码（`grep -n 'read_root_ring_slot(\|read_root_ring_slot_telling_never_written_apart(\|every_root_ring_slot(' crates/singlefs-core/src/*.rs` 命中 18 行（含 `every_root_ring_slot` 自己的定义），今天复跑，归到下表 9 处）。「撞出同一形」指：这一处看不见的根，名下新分配的单元之后被当空闲槽发出去，再在某个后来的挂载里成为「账读不出的被抛弃根」。

| # | 位置（冻结树） | 全零 / 读不出怎么处置 | 会不会撞出 C694 那一形 |
|---|---|---|---|
| 1 | 择根 `crates/singlefs-core/src/recovery.rs:1268`「pub fn choose_root(」 | 当没有这一槽，落到更旧的根 | **是源头**：落到更旧的根、而那条看不见的根名下新分配的单元没人护（本题）。这里本身是对的（恢复要往下走），缺的是第 2、4 行那一头的账 |
| 2 | 影子账 / 抬 F / 回收门槛读的候选根 `crates/singlefs-core/src/recovery.rs:1189`「fn visit_valid_roots_with_ring_slots<Reader: PoolReader + ?Sized>(」 | 读不出直接 `continue`，不自证的也跳过（C563 登记的就是这一处） | 同形：看不见的根没进 `roots`，影子账没有它 |
| 3 | 实例表清扫 `crates/singlefs-core/src/instance_table_sweep.rs:459`「/// 读根环每个槽：读不出、读成功而全零 ⇒ 这一次不删；读得出而自证不过 ⇒ 重读一次，仍不过 ⇒ 这一次不删；都自证过就交回每一条根。」 | **拒**（这一次不删行） | 不撞：这一处已经按「全零可能是看不见的根」保守，条款在 D18 已定项 11（`crates/singlefs-core/src/mount.rs:1733` 的注释点名「清扫把全零与读不出同待遇」） |
| 4 | 可写挂载重建分配器 `crates/singlefs-core/src/mount.rs:1953`「fn rebuilt_allocator<Device: BlockDevice>(」，分类 `crates/singlefs-core/src/mount.rs:1743`「/// 读得出、每个字节都是 0：多半从没写过，也可能是一次读坏的有效根（回收门槛照这一种钉住）。」 | 记成 `AllZeroAtMount`（`crates/singlefs-core/src/allocator.rs:995`），只钉回收门槛，**不带落点、不进影子账** | **是 C694 的发生处**：钉住的是已释放的槽，看不见的根新分配的槽不在其内（1.3 表）。乙加在这一处 |
| 5 | 抬 F 的上限 `crates/singlefs-core/src/mount.rs:2465`「fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(」 | 全零不并进上限 | 用户认下的代价（D16「读回全零的槽不并进上限、F 可能越过住在全零槽里的有效根（用户认的代价）」）；形不同：越过的是**有效**根不是被抛弃根，C694 的拒不会从这里长出来；它的丢数据面由 E173 判过，不重判 |
| 6 | 卸载 / 准入抬 F 串读的根环 `crates/singlefs-core/src/mount.rs:3287`「fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(」 | 知道住着根的槽重读仍坏就拒；挂载那一刻读坏的当没有根 | 这一处是 C694 **被拒的地方**：根环那一遍现读，看见了此刻读得出的那条根（1.1），影子账再去读它的账 |
| 7 | journal 扫描 `crates/singlefs-core/src/recovery.rs:3400`「pub fn scan_journal(」 | 读不出、自证不过的记录槽不进记录表 | 同前提不同资源：一条记录读回全零会切断前缀。它造成的损失是「退到更早那一版」，条款已认下（D23 已定项 14 「点名单元的两份都读得出…才施加」那一段的代价）；与根槽一起读回全零才会没有账可用，这时乙无效（1.5） |
| 8 | 系统配置槽 `crates/singlefs-core/src/recovery.rs:619`「NotSelfDescribing,」 | 这一槽不可择，换槽换盘 | 不撞：C554 乙-配置续的见证值把「没读到的更新状态」拒成拒可写（D23 已定项 14），没见证到的那一半就是本题 |
| 9 | 单元读 `crates/singlefs-core/src/recovery.rs:371` 的 `read_unit_via_locations`（一份读不出换另一份，校验和不符当读不出） | 逐条位置条目试 | 本题第二个缺口就在这里：它只回「读不出」，分不开「瞬时」与「被占了」（甲在它旁边补那一道判据） |

读法：
- 同一个前提真正会撞出 C694 那一形的只有第 1、2、4 行连起来的那一条链；第 3 行已经按它保守，第 5 行是用户认下的另一种代价，第 7、8 行有各自的兜底。**没有发现第二处独立撞出同一形的地方**（读码，没各造一个盘面；第 5 行与第 7 行的盘面我没造，是推的）。
- 另一处同族的偏差不在「当没有根」上，在 C694 一侧的映射现算 `mount.rs:1593`：它先读映射树、不判全零，对树表 0 条的被抛弃根（映射根空指针）一律交 `None`，函数注释自己写了「映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内」。这条根的树表单元读不出、又要走现算时，现算不成就走到 C694 的拒。这一形（被抛弃的是树表 0 条那一版、且它的树表读不出）我没造出盘面（没量），登记为推的；改法与 K3 同族（映射根全零当没有树，先取根记录里直接持有的实例表与树表指针）。
- 推翻「没有第二处」的现象：任何一处读根环的新代码把读回全零直接当成没有根、又没有走第 4 行那一道钉住（`grep` 上面那三个读根环函数以外的 `RootRecord::parse_slot` 调用）。今天 `grep -n parse_slot crates/singlefs-core/src/*.rs` 里 `RootRecord::parse_slot` 的非测试调用点只有四处：`instance_table_sweep.rs:484`（第 3 行）、`recovery.rs:1257`（第 1、2 行共用的 `read_root_ring_slot`）、`recovery.rs:3265`（按字节取根记录原样，不是处置）、`mount.rs:1767`（第 4 行的分类）；都在上表里。

### 2.4 候选的定义、落点与量到的

**K3 甲（只在映射根是空指针时先判树表 0 条）**。定义：`open_pool_for_read` 里映射根是整条全零时，先读树表单元，0 条就报 `TreeTableHasNoEntries`；映射根不是空指针的版本读序、读次数一字不变；映射根全零而树表有条目（损坏）仍落到今天的路径，报 `Walk(UnitUnreadable …)`。落点：`crates/singlefs-core/src/mounted_read.rs:376`「let mapping_tree = read_code_two_tree(」之前加一段，约 15 行。要新条款：不必，条款没写这一格；`TreeTableHasNoEntries` 本来就是它的文档（`mounted_read.rs:195`）写的结局。量到（`m3c695_switches.rs`，`C695_SWITCHES=k3c`，`run-11-c695-k3c.log`，原样）：

```
  W=384 lost=dev0 rec=Some((1, 2)) chosen_root=Some((1, 1)) mapping_root=[d0s0,d0s0] outcome=Err("只读挂载：Open(TreeTableHasNoEntries)") unit_reads_at_slot_0=[]
RING region=0 slot=0 on_dev0 root=(0,0) mapping_root=[d0s0,d0s0] open_both_devices=Err TreeTableHasNoEntries
RING region=0 slot=1 on_dev0 root=(1,3) mapping_root=[d0s1481,d1s1481] open_both_devices=Ok
RING region=1 slot=0 on_dev1 root=(1,1) mapping_root=[d0s0,d0s0] open_both_devices=Err TreeTableHasNoEntries
RING region=2 slot=0 on_dev0 root=(1,2) mapping_root=[d0s0,d0s0] open_both_devices=Err TreeTableHasNoEntries
```
宽 256、240、4 GiB 的丢 0 号盘四格同样（`grep -c 'Open(TreeTableHasNoEntries)' run-11-c695-k3c.log` 出 4）；盘 0 槽 0 一次没再读（`unit_reads_at_slot_0=[]`，今天是两次读不到）；丢 1 号盘那四格不变（读得出 1 个文件）。回归：下列 8 个 harness 目标在甲之下全绿，共 110 条（`SINGLEFS_K3=c`，`regress-k3-c-*.log`，每条一行 `test result: ok`）：`mounted_recovery_reads_each_candidate_once_in_order` 7、`mounted_read_by_offset` 12、`tree_nodes_and_the_central_mapping_on_mounted_read` 12、`core_review_tree_table_duplicates_and_slot_one_search` 12、`rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root` 37（它对每条版本调 `open_pool_for_read`）、`system_configuration_choice_names_an_empty_pool_and_devices_that_lost_both_slots` 3、`a_lone_surviving_device_reads_back_what_it_read_before_the_unmount` 4、`unlink_mount_continuation_and_the_emptied_version` 23。这 8 个目标今天（不加甲）我只跑了前两个（7、12 条全绿，`regress-k3-none-*.log`），其余没跑，因为甲之下没有红的，没有需要对照的。

**K3 乙（无条件先读树表）**，对照用。定义：不论映射根是什么，先读树表、0 条先报。「先判树表 0 条会不会让别的版本读错」的答案：**会改读次数**。副本里我做成「先读一次树表、判完仍按原次序读」（多读一次树表，`SINGLEFS_K3=r`），`mounted_read_by_offset` 12 条里红 1 条：

```
assertion `left == right` failed: 只读了映射树根那一个节点（第一条位置条目就中）
  left: 2
 right: 1
```
（另两个目标 `tree_nodes_and_the_central_mapping_on_mounted_read` 12、`mounted_recovery_reads_each_candidate_once_in_order` 7 在乙之下全绿；`regress-k3-r-mounted_read_by_offset.log`，用例 `central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries`，`mounted_read_by_offset.rs:1240`）。把树表读挪到映射树之后原有那一次读的话读序也变，两种做法都碰那几条钉着读次数与读序的用例，所以甲只在映射根全零时才提前读。真正「挪序」（不多读）的版本我没做，推的：碰同一批钉读序的用例。

**K3 丙（只读挂载挂成空）**。没做。要改 `MountedPoolForRead`（`crates/singlefs-core/src/mounted_read.rs:331`、`mounted_read.rs:333` 的 `extent_tree`、`extent_tree_root` 等不是 `Option`）与 `open_file`；是否取它是岔路，条款没有一句话在选。

**可写挂载、恢复、池级 checker 碰同一版**：见 2.3 的表，除 `open_pool_for_read` 外读序都已是「树表先、映射后（或懒读、或全零当没有树）」，甲之后只读挂载与它们一致。单盘镜像上可写挂载本题没造（`admit_the_writable_device_count` 的单盘处置我没查），推的：C683 之外不属于这一格。

## 四、模型与复跑

- 模型目录 `research/prompts/m3-c694-r1-sonnet-model/`：`candidates.diff`（冻结树上的候选开关补丁，189 行，只动 `crates/singlefs-core/src/mount.rs` 与 `mounted_read.rs`，默认全关 = 今天的实现）、`tests/m3c694_switches.rs`、`tests/m3c695_switches.rs`（探针，引调查员两份复现草稿的造法）、`rerun.sh`、`compare.sh`、`expected/`（7 个抽出文件）、`README.md`。
- 复跑命令：`SINGLEFS_REPO=/home/fy5090/code/singlefs FULL=1 THREADS=4 bash research/prompts/m3-c694-r1-sonnet-model/rerun.sh`，在空目录 `~/.local/share/singlefs/work/m3-c694-r1-sonnet-rerun/`（开跑前 `ls -A | wc -l` 出 0）里从头跑通一次：编译经 6G、跑经 4G 内存包装，线程 4，自己的编译目录，只跑 harness 档。结束时 `bash compare.sh compare <抽出目录> expected/` 原样输出：`全同（7 个文件）`。`compare.sh` 抽取时去掉 `test … ... ` 前缀与耗时行、`LC_ALL=C sort`，不读 `SINGLEFS_REPO`。
- 1.2 的表是这一次从头跑的抽出文件算的（`grep -c` 数，今天，原样）：`probe-none.txt` 60 行 `PROBE W=`、`unmount=Err` 11、`remount=Err` 11、`reds_at_start=[I-7.4]` 44；`probe-w1.txt` 对应 60 / 0 / 0 / 45；`probe-w4.txt` 60 / 0 / 0 / 0；会话那一遍 `cause: NewerStateStillUnreadableAf` 11 / 0 / 0，`NoSpaceAfterRaisingTheFloor` 15 / 15 / 22，`reds_at_end=[]` 35 / 47 / 58。与我先前在三份单独副本上量的逐项相同。
- 开工后收到主 agent 两条消息：线程上限 7 → 5 → 4。第一次改之前已经起的两条后台回归（`regress-base-*`、`regress-w1-*`、`regress-k3-*`）按 7 起、没中途改；从 W4、候选开关的编译与跑起（`tools/*.sh` 改成 `capped.sh 4`）都是 4。

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-（一）卸载 / 会话写 / 再可写挂载该不该做得成 | 推出：该（U1） | 拒不护任何东西（44 / 58 个起点在拒之前 I-7.4 已红）；U2 后半句是闩，11 / 11 再挂被拒，`crates/` 无出路 |
| K1-（二）影子账还有没有东西要护 | 对已被占的单元没有；对「根还能不能打开」有一道凭据 | 凭据 = 树表与映射根槽被诞生身份不同的合法节点占了；写零不算（调查员推翻条件那一格甲照拒） |
| K1-（二）不护会误放的一形 | 丙（读不出一律放行）造出丢数据 | 覆盖写 1 次就把 4 个账单元副本全写掉、I-7.4 红；今天与甲都拒、盘不动 |
| K1-（三）D16 / D23 各管哪一半 | 两处「没人管」 | 没见证到的那一半的新分配单元不被护；根读得出之后账读不出不分瞬时与被占 |
| K1 候选甲（废根放行） | 量到：11 拒全放行、31 条相关既有用例全绿 | 只解已损坏的盘；不防损坏 |
| K1 候选乙（点名落点预隔离） | 量到：起点 0 红 I-7.4、11 拒全消 | 防损坏本身；寿命没做对（上界），journal 记录不在时无效 |
| K3 先读空映射根是不是 bug | 是 | 全零 = 没有这棵树；离群读者只有 `open_pool_for_read`；mkfs 第 0 代、暖机、写行三条根都受影响 |
| K3 对的结局 | 报 `TreeTableHasNoEntries`；不换根；「挂成空」是岔路 | 条款没写这一格，从 D16 择根不看 F 与枚举文档推出 |
| K3 候选甲（映射根全零时先判）/ 乙（无条件先读） | 甲 110 条全绿；乙红 1 条（读次数 2 对 1） | 乙碰钉读次数的用例，甲不碰 |
| K4 回扫 | 同形只有 choose_root → 影子账看不见 → 重建分配器钉回收门槛不带落点 这一条链 | 其余几处已保守或是用户认下的代价；映射现算对树表 0 条版本交 `None` 一形没造出盘面 |

## 六、没做什么

- K2（候选代价的攻防）、岔路的采纳与判决不归我，没做。
- 没跑任何 checker 档、重型测试、全量门禁；只跑上面写明的 harness 目标。
- 没在入库装置上重做数：甲、乙、丙、K3 的数都在冻结树的副本上量，按 `three-way-inference.md` 不进 kb，要引由主 agent 重做。
- 没做：乙的寿命（根环槽被写掉时放开）；journal 记录读不出时的变体；树表 0 条版本被抛弃且树表读不出的盘面（K4 末条）；`unreadable_abandoned_root_slot` 在乙之下的结局；K3 丙；单盘镜像上可写挂载的结局；4 GiB 那一档的复跑；甲的假阳性盘面（读错、读回全零等逐个造）。
- 没核实的前提：取新实例那次挂载「整次发布的单元写与根槽读回全零」这个故障在真实设备上多常见——探针沿用调查员 / 攻方的造法（把那次发布的单元写与根槽读回全 0），频率没量。
- 草稿与编译目录：`~/.local/share/singlefs/work/m3-c694-r1-sonnet/`（仓副本 `repo/`、`repo-base/`、`repo-w4/`、`repo-sw/`，日志与 `tools/`）与 `~/.local/share/singlefs/work/m3-c694-r1-sonnet-rerun/`（空目录复跑的工作区）；各 `target*` 编译目录交回前删掉，仓副本与日志留着，为的是报告里引的日志文件名（`run-*.log`、`regress-*.log`）主 agent 要核时在。

## 七、什么现象会推翻

- K1「该做得成」：找到一个起点，被抛弃根的树表与映射根槽都被诞生身份不同的合法节点占了，而之后某一次恢复仍把它择中并打开成功（那时「已不可打开」的凭据就错了，甲要撤）；或找到一条产品路径，在这类盘面上不靠绕开准入就能把那条根轮出环（U2 后半句成立，丁不是死路）。我在 11 个起点上没看到（再挂 11 / 11 被拒）。
- K1「拒不护任何东西」：一个起点在卸载之前池级 checker 绿、而拒的那一次本来会让它变红。44 / 58 红在先，没看到绿的被拒的起点。
- K1 甲的凭据：一个「槽被别的合法节点占了」判真、而那条根其实还能打开的盘面（例如两条位置条目里一条被占、另一条的读错是瞬时）；我的判定要求两条位置条目都被占，这一形被排除，但没造盘面验。
- K3「是 bug」：条款或用例写明树表 0 条版本只读挂载报 `Walk(UnitUnreadable …)`；`grep` 没有。K3 甲「不碰别的版本」：映射根非空的版本在甲之下读次数或读序与今天不同；110 条里没有。
- K4「只有一条链」：别的读根环的地方把全零当没有根、又不走重建分配器那一道钉住，并且在新实例里造出被复用的单元；我读码没找到，也没造第二个盘面。
