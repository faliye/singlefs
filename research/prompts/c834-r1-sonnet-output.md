# C834 第一轮 正推腿报告（Sonnet，2026-10-15）

立场：正推。从条款与冻结副本（`/home/fy5090/.local/share/singlefs/work/c834-r1/repo/`，与主仓 `crates/` 里 `mount.rs`、`recovery.rs`、`transaction.rs` 与三份 harness 测试文件共六份逐字节相同，`cmp` 现查过）推出每个候选在各格上的判定与代价，给出哪一个站得住。
模型与驱动在 `research/prompts/c834-r1-sonnet-model/`（补丁、测试、`tools/rerun.sh`、`out/`），说明见同目录 README.md。行号：代码引的是主仓路径（与冻结副本同行号），kb 引 kb 文件自己的行号。标「推的」的是没量过的推理，其余都有命令输出。

## 0. 候选与读法（本报告统一用这几个名字）

| 名 | 读法 | 在模型补丁里 |
|---|---|---|
| K1 | 今天的实现：择到那一份系统配置的实例代号比那条根新就放过 | `research/prompts/c834-r1-sonnet-model/patch/mount-c834-variants.patch:84` 「"k1" => chosen_instance > root.instance,」 |
| K2 | 每盘取全部自证槽里最大的，盘间取最小 | 同补丁 `:86` 「"k2" => smallest.is_some_and(|number| number > root.instance),」 |
| K2 另读 | 各盘各槽取最小（「逐盘取最小」的另一种读法） | 同补丁 `:119` 「"k2_min_over_every_slot" => per_device_slot_instances」 |
| K3 | 要求盘上有带比那条根更大的号的根或 journal 记录 | 同补丁 `:88` 「"k3" => {」 |
| K4 | 照 I-7.7 第 ② 句：盘间不等时较大者不出现在任何根、记录里就取较小，相等取其值 | 同补丁 `:93` 「"k4" => {」 |
| K5 | 外部审核的「跨盘取槽代号最大」 | 同补丁 `:167` 「"k5" => largest.is_some_and(|number| number > root.instance),」 |
| K6 | K3 的凭据，再加「那条根点名的中央映射树根那个槽里现在是别的、自证过的、实例代号比它新的索引节点」（本腿新提，被攻过零轮） | 同补丁 `:104` 「"k6" => {」 |

K4 里「单元写序」那一路（I-7.7 第 ② 句三样之一）没实现：要扫整个单元区读单元头，挂载这一判里读不起（推的）；见 G2。
补丁只改 `placements_in_the_accounts_of_newer_readable_roots` 里那一句判（读法由环境变量 `C834_VARIANT` 选），其余产品代码一字不改。

## 1. G1 现象坐实

**判定：站得住（坐实）。** 造得出，两种交盘次序走不同的支，结局一个拒、一个放过，放过的那一支写掉了 R 的全部单元。

读到的实现（冻结副本，行号同主仓）：

- 择系统配置按盘表次序逐盘走：「for device in reader.device_identities() {」（`crates/singlefs-core/src/recovery.rs:1102`）；每盘取世代号较大的那一槽：「if one.quantities.slot_generation > zero.quantities.slot_generation {」（`crates/singlefs-core/src/recovery.rs:1139`）；全池第一块交得出的盘那一份当择到的：「None => chosen = Some(best_on_device),」（`crates/singlefs-core/src/recovery.rs:1147`）；之后的盘只比 fsid 与设备数：「|| previous.immutable.device_count != best_on_device.immutable.device_count」（`crates/singlefs-core/src/recovery.rs:1151`）。
- 用它的那一判：「None if journal_instance_in_the_system_configuration」（`crates/singlefs-core/src/mount.rs:2078`）接「<= root.instance =>」（`crates/singlefs-core/src/mount.rs:2079`），走重读再拒的那一支；比那条根新则落到「None => continue,」（`crates/singlefs-core/src/mount.rs:2089`）不交也不拒。传进去的是「system_configuration.quantities.journal_instance,」（`crates/singlefs-core/src/mount.rs:3124`）。
- 取号写盘按盘表次序逐盘写：「for index in 0..pool.devices.len() {」（`crates/singlefs-core/src/transaction.rs:1050`）。第一块写完、第二块还没写就断电，两盘的实例代号不等。

造法（模型目录 `tests/c834_order_of_devices_decides_the_newer_root_judgement.rs`）：
入库助手起 384 槽小盘，第一个文件之后覆盖写 5 次，再录一次发布 R（实例 1、txg 9），崩在 R 的根槽落盘之后、系统配置轮换之前。
- 历史 `partial`：无读故障的一次可写挂载 M0 取号，盘面截在第一块盘的系统配置槽写之后（「partial.apply_writes(&m0[..=system_configuration_writes[0]]);」，`research/prompts/c834-r1-sonnet-model/tests/c834_order_of_devices_decides_the_newer_root_judgement.rs:152`）。
- 之后一次可写挂载 M2，R 的全部记录、中央映射树根、树表读回全零（入库用例 `a_newer_readable_root_whose_only_record_mapping_root_and_tree_table_read_back_zero_refuses_the_writable_mount_before_any_write` 的那三段；没有取号写的盘面上它拒，对照行 baseline）。
- 同一张盘面，盘表交 [盘 0, 盘 1] 与 [盘 1, 盘 0] 各挂一次。

原样输出（K1 即今天的实现；`research/prompts/c834-r1-sonnet-model/out/all-k1.log` 里 `grep -E '^history=(baseline|partial)|variant=k1 history=(baseline|partial)'` 的全部输出，拒因串被测试自己截成 150 字符）：

```
history=baseline 系统配置各盘各槽的实例代号=[(DeviceIdentity(0), [1, 1]), (DeviceIdentity(1), [1, 1])] 这张盘面本身池级 checker 红=[]
ROW variant=k1 history=baseline 盘序=0,1 => 拒 NewerStateStillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { unapplied_publish: RollbackTarget { instance: In | 这次挂载新写掉的 R 单元份数=None（挂载前盘面上已被写掉 0 份）
ROW variant=k1 history=baseline 盘序=1,0 => 拒 NewerStateStillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { unapplied_publish: RollbackTarget { instance: In | 这次挂载新写掉的 R 单元份数=None（挂载前盘面上已被写掉 0 份）
history=partial 系统配置各盘各槽的实例代号=[(DeviceIdentity(0), [1, 2]), (DeviceIdentity(1), [1, 1])] 这张盘面本身池级 checker 红=[]
ROW variant=k1 history=partial 盘序=0,1 => 做成 落到(instance,txg)=(1, 8) 写行根落盘前断电的盘面上池级 checker 红=["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | 这次挂载新写掉的 R 单元份数=Some(14)（挂载前盘面上已被写掉 0 份）
ROW variant=k1 history=partial 盘序=1,0 => 拒 NewerStateStillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { unapplied_publish: RollbackTarget { instance: In | 这次挂载新写掉的 R 单元份数=None（挂载前盘面上已被写掉 0 份）
```

读法：盘表 [盘 0, 盘 1] 时择到盘 0 那一份（实例代号 2），`2 <= 1` 不成立，落到「None => continue,」，M2 做成，写行根落盘之前新写掉 R 的 14 份单元，那一刻的盘面上池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4（与判决 `research/prompts/m3-batch37-43-code-r1-main-verification.md:25` 「写行那次根落盘之前写掉 R 的 14 份单元，池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4」同一组）；盘表 [盘 1, 盘 0] 时择到盘 1 那一份（实例代号 1），`1 <= 1` 成立，重读后仍不成，拒，拒在任何写之前。**两种次序的结局不同，C834 的推测坐实。**

违反哪条不变量：放过那一支之后，那一刻的盘面上 I-7.2（最新根可完整遍历）、I-7.4（近 K 代块未被复用）、I-2.1、I-4.8 红，行号 `crates/singlefs-checker/invariants/02-根环.md:9`（I-7.2）、`:11`（I-7.4）、`crates/singlefs-checker/invariants/04-校验和.md:6`（I-2.1）、`crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:11`（I-4.8）。

**同一次量出的、C834 之外的新现象（历史 `complete`）：** 取号两块盘都写完、之后一个字节没写（盘 0、盘 1 两槽都是 [1, 2]），M2 同样的读故障：K1 在两种盘序下都「做成」并新写掉 R 的 14 份单元，checker 红同一组。
这个盘面是 kb 认的合法态：「已写进系统配置的新代号不回卷——回卷只管取号自己那几次系统配置写」（`.claude/kb/decisions/23-journal的角色与格式.md:480`）、「这个号就此烧掉，下一次可写挂载按 max + 1 取下一个」（同文件同行）、「这是已知行为，不当缺陷修」（同文件同行）。
也就是说实现注释的前提（转述 `crates/singlefs-core/src/mount.rs:2070`：系统配置里的号比那条根新，就是那条根之后已有一次挂载取过号、写掉了它的单元）在 kb 认的合法态下不成立，与盘序无关。

什么现象会推翻 G1：在冻结副本拷贝上按 README 复跑，`out/summary.txt` 里 `variant=k1 history=partial` 两行结局相同；或 `history=complete` 不再新写掉 R 的单元。

## 2. G2 凭据：盘上留下什么能当「那条根之后已有一次挂载取过号、写掉了它的单元」

先定判法。这句话要判的事实是「R 的单元在盘上已被别的写覆盖」。它能直接量：入库助手 `units_of_r_overwritten_on` 对盘面逐字节比 R 写出的单元（`crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs`），输出里的「挂载前盘面上已被写掉 N 份」就是它。该拒 ⟺ N = 0（单元还在、只是这一刻读不出，放过就会写掉它们）；该放过 ⟺ N > 0（单元已被写掉，拒了就再也做不成可写挂载，`crates/singlefs-core/src/mount.rs:2070` 的注释用过这个理由）。
入库用例钉的是同一条：「写行那次发布的根落盘之前，M1 写下的一切都没碰 R 的单元」（`crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs:634`）。

九段历史（都在同一份 R 上，M2 的读故障除注明外同 baseline：R 的全部记录、映射根、树表读回全零；每段两种盘序各挂一次）：

| 历史 | 盘上是什么 | N（`out/summary.txt` 现取） | 该怎么判 |
|---|---|---|---|
| baseline | 没有取号写，入库用例的盘面 | 0 | 拒 |
| partial | 取号只落第一块盘（盘 0 [1,2]，盘 1 [1,1]），之后一字未写 | 0 | 拒 |
| complete | 取号两盘落完（都是 [1,2]），之后一字未写（kb 认的烧号态） | 0 | 拒 |
| units_only | 取号完成；R 的根槽、单元、第一条记录在那次挂载里读回全零，那次挂载的写行发布单元写了、记录与根没落；M2 里只有 R 的记录读回全零 | 14 | 放过 |
| urj | 同上但写行发布的记录也落了（入库 urJ 那一形），M2 无读故障 | 14 | 放过 |
| urj_row_record_also_hidden | urj，M2 里写行发布的记录也读回全零（二重故障） | 14 | 放过 |
| later_mount_records_landed_r_units_intact | 无读故障的 M0 择的是 R，写行发布的单元与记录落了、根没落，R 的单元一份不少；M2 里 R 的账读不出 | 0 | 拒或放过都不丢 R 的单元（下面「安全」那条用例钉；表按今天的结局钉「放过」） |
| acquisition_failed_on_device_1_rolled_back | 取号第二块盘写报错，盘 0 那份回卷：盘 0 两槽 [1,2]，世代号较新的那一槽带旧号 | 0 | 拒 |
| acquisition_barrier_failed_rolled_back_on_both | 取号两盘写完后屏障报错，两盘都回卷 | 0 | 拒 |

结局矩阵（每格 = 盘序 [盘 0, 盘 1] / [盘 1, 盘 0] 的结局，两序相同只写一个；「做成[14]」= 做成且这次挂载新写掉 R 的 14 份单元）。`python3` 现从 `out/summary.txt` 算出：

```
历史 \ 候选 : k1 | k2 | k2_min_over_every_slot | k3 | k4 | k5 | k6 | k6_mapping_ge | k6_only_mapping_root | k6_records_ge
baseline : 拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 做成[14]
partial : 做成[14]/拒 | 拒 | 拒 | 拒 | 拒 | 做成[14] | 拒 | 拒 | 拒 | 做成[14]
complete : 做成[14] | 做成[14] | 拒 | 拒 | 做成[14] | 做成[14] | 拒 | 拒 | 拒 | 做成[14]
units_only : 做成 | 做成 | 拒 | 拒 | 做成 | 做成 | 做成 | 做成 | 做成 | 做成
urj : 做成 | 做成 | 拒 | 做成 | 做成 | 做成 | 做成 | 做成 | 做成 | 做成
urj_row_record_also_hidden : 做成 | 做成 | 拒 | 拒 | 做成 | 做成 | 做成 | 做成 | 做成 | 做成
later_mount_records_landed_r_units_intact : 做成 | 做成 | 拒 | 做成 | 做成 | 做成 | 做成 | 做成 | 拒 | 做成
acquisition_failed_on_device_1_rolled_back : 拒 | 拒 | 拒 | 拒 | 拒 | 做成[14] | 拒 | 拒 | 拒 | 做成[14]
acquisition_barrier_failed_rolled_back_on_both : 拒 | 做成[14] | 拒 | 拒 | 做成[14] | 做成[14] | 拒 | 拒 | 拒 | 做成[14]
```

（列名：k1 = K1，k2 = K2，k2_min_over_every_slot = K2 另读，k3 = K3，k4 = K4，k5 = K5，k6 = K6，k6_only_mapping_root、k6_records_ge、k6_mapping_ge = K6 的三个变异。）

逐候选判定（对照上表「该怎么判」）：

| 候选 | 判定 | 依据（上表） |
|---|---|---|
| K1 今天的实现 | 站不住 | partial 两序结局不同（做成[14] / 拒）；complete 两序都做成[14]。回卷两段它答对（择到的是世代号较新的那一槽，带旧号） |
| K2 逐盘最大、盘间最小 | 站不住 | 与盘序无关，但 complete 与「两盘回卷」答错（做成[14]）：烧掉的号、回卷后留在旧槽里的号都被当成「取过号并写过」 |
| K2 另读 各盘各槽取最小 | 站不住 | 与盘序无关，但 units_only、urj、urj_row_record_also_hidden、later_mount 全拒：写行发布的根没落、轮换没写，两盘各槽里仍有旧号。urj 拒 = 违反入库用例 urJ |
| K3 要有带更大号的根或记录 | 站得住，有代价 | 九段里七段答对且与盘序无关；units_only 与 urj_row_record_also_hidden 拒（R 的单元已被写掉 14 份，该放过）。两段都是二重故障（写行发布断在记录之前、或记录也读不出，并且 R 自己的记录读不出） |
| K4 照 I-7.7 第 ② 句 | 站不住 | 相等的实例代号不分「取了没写」与「取了也写了」：complete 与「两盘回卷」答错（做成[14]），与 K2 同一类。 ② 句管的是不等的态，条款对相等态没话说 |
| K5 跨盘取最大 | 站不住 | partial 两序都做成[14]，回卷两段也做成[14]：它让 C834 的结局与盘序无关，但稳定走到错的那一支（与 checks-owed 那一行的推断相符，现在量过） |
| K6 K3 加映射根槽凭据 | 站得住 | 九段全答对、与盘序无关；见下「代价」 |

安全那一条用例（`research/prompts/c834-r1-sonnet-model/tests/c834_order_of_devices_decides_the_newer_root_judgement.rs:373`，调入库助手 `mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions`）：later_mount 一段，K1、K3、K6 做成之后写行根落盘前与之后三次用户动作，R 的根还在环里时 R 的单元一份不少。原样输出行 `SAFE variant=k1 结局=Ok((InstanceGeneration(1), CheckpointTxg(8)))` 三个候选各一行，见 `out/all-k1.log`、`out/all-k3.log`、`out/all-k6.log` 里 `grep SAFE`。
为什么「有更大号的 journal 记录」够放过（推的，只有这一段有量）：那条记录所属的发布建在 R 上时，它自己的账里就含 R 的单元，预隔离会护住它们；建在更旧的 P 上时，它的单元已经覆盖了 R 的单元。第二种若只盖掉 R 的一部分单元（没造这一形），剩下的没被护：判不了。

站得住的两个候选的代价数（量的，来自上面矩阵与补丁）：

| | K3 | K6 |
|---|---|---|
| 九段里答错的段数 | 2（units_only、urj_row_record_also_hidden：该放过而拒） | 0 |
| 答错时的后果 | 取号之前拒可写挂载（一个字节不写），R 的记录读不出的话一直拒，不会自己好 | 无（本九段里） |
| 挂载这一判多读什么 | 不多读：`roots` 与 `records` 是这一判已有的参数（`crates/singlefs-core/src/mount.rs:2031` 起的函数签名） | 多读那条根点名的中央映射树根的两个落点各 16 KiB，只在「记录全读不出且映射现算不成」这一支里读 |
| 改 `crates/` 哪里（实现里没有的，写成加在哪一处） | `mount.rs` 的 `placements_in_the_accounts_of_newer_readable_roots`：把「None if journal_instance_in_the_system_configuration <= root.instance」那一句换成凭据判；函数参数 `journal_instance_in_the_system_configuration` 与调用处「system_configuration.quantities.journal_instance,」（`crates/singlefs-core/src/mount.rs:3124`）删掉 | 同左，凭据判里再加读映射根槽 |
| 被攻过几轮 | 零轮（本腿新提） | 零轮（本腿新提） |

两个都站得住，差别只在二重故障那两段该不该放过，且 K6 带一个只在本装置上量过的识别法：**按跑前条款第三行，这是岔路，交主 agent 写岔路单。** 岔路的翻面观测（量出什么数会让选择换面）：K6 的映射根识别法在多层映射树的盘面上是否仍成立（本装置的 384 槽小盘映射树只有一层，下面 G4 有一个存活变异正是这个）；二重故障（写行发布断在记录之前，且 R 的记录读不出）的出现率比读两个 16 KiB 的实现复杂度更重要还是更不重要。

K6 的识别法只认「索引节点」这一类单元头：`crates/singlefs-core/src/unit.rs` 的 `parse_index_node`；映射根被别的类（数据单元、打包记录单元）覆盖时它读不出实例代号，退回 K3 的结局（拒）。这一形没造：判不了。（推的）

「取号写到一半崩掉」「取了号没写」「取了号也写了（urJ）」三种历史各判成什么，上面矩阵里 partial、complete、urj 三行逐候选已给；与盘序无关的有 K2、K2 另读、K3、K4、K5、K6，有关的只有 K1。

什么现象会推翻 G2 的判定：
- 推翻「K6 站得住」：构造一段 R 的单元已被写掉（`units_of_r_overwritten_on` 非空）、K6 却拒的历史（识别法漏掉的类），或一段 R 的单元还在、K6 却放过且之后写掉了它们的历史。
- 推翻「K3 站得住」：构造一段 R 的单元还在、K3 却放过的历史。安全那一条用例是一段这样的历史的候选，K3 没被它打中。
- 推翻「K2、K4、K5 站不住」：复跑 `tools/rerun.sh` 后 complete 或回卷两段不再新写掉 R 的单元。

## 3. G3 同一个量吗

**判定：不是同一个量。判法：站得住的 K3/K6 不再读择到那一份的实例代号，不需要跟着改别处；其余读法的差别要在条款里写明。**

仓里读系统配置实例代号的口径（行号现取）：

| 在哪 | 读法 |
|---|---|
| 本判（`crates/singlefs-core/src/mount.rs:3124`） | 择到那一份（盘表第一块交得出的盘上世代号较大的槽）的号：「system_configuration.quantities.journal_instance,」 |
| 取号（`crates/singlefs-core/src/transaction.rs:886`） | 每块盘两槽全部自证过的槽里最大的，再与根环里全部根的实例代号取大加一：「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」 |
| 落后支（`crates/singlefs-core/src/mount.rs:5366`） | 每盘取世代号最大的那一槽：「.max_by_key(|slot| slot.quantities.slot_generation);」，号取它的：「instance_generation: newest_self_verified.quantities.journal_instance,」（`crates/singlefs-core/src/mount.rs:5436`） |
| 池级 checker I-7.7 | 每盘取全部自证槽里最大的，盘间不等时较大者不许出现在任何根、记录、单元写序里：「各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里」（`crates/singlefs-checker/invariants/02-根环.md:14`） |

三种口径：本判是「某一盘上世代号较新的那一槽」，取号与 checker 是「全部自证槽里最大的」，落后支是「每盘世代号较新的那一槽」。世代号较新的那一槽带的号可以低于另一槽：取号回卷之后正是这样（历史 `acquisition_barrier_failed_rolled_back_on_both` 的盘面，两盘两槽 [1,2] 而较新的槽带 1；K1 在这一段答对、取最大的读法答错，矩阵里 K2、K4、K5 的最后两行）。
所以「改成逐盘逐槽取最大」不是把几处读法统一，而是把本判改成会在回卷后答错的那一种（推的依据 + 矩阵实测）。

`checks-owed.md` C834 那一行写「其余读运行期计数的地方都逐盘逐槽重读取最大」（`.claude/kb/checks-owed.md:555`）。实测落后支那一处是「每盘取世代号最大的那一槽」而不是取号意义上的最大：**冲突**，两边整行并排：
- `.claude/kb/checks-owed.md:555`：其余读运行期计数的地方都逐盘逐槽重读取最大
- `crates/singlefs-core/src/mount.rs:5366`：`.max_by_key(|slot| slot.quantities.slot_generation);`
这条冲突不改变结论（落后支比的是 (实例代号, journal tail) 与那一版的位置，不是本判的问题），但 C834 行里「其余」二字不准。

择到的那一份还剩别的字段在用：回退下界与 journal tail 另有逐盘取最大的读法（「.map(|system_configuration| system_configuration.quantities.rollback_floor)」，`crates/singlefs-core/src/recovery.rs:1919`；「.map(|slot| slot.quantities.journal_tail)」，`crates/singlefs-core/src/mount.rs:6992`），不依赖择到哪一块盘。择到那一份的 `journal_instance` 只在 `crates/singlefs-core/src/mount.rs:3124` 一处进判（`grep -rn 'journal_instance' crates/ --include=*.rs` 对 `mount.rs` 的命中里，读择到那一份的只有 3124）。这一句我没有逐个核它在 harness 与 checker 档里的读法，只核了 core：判不了 harness 与 checker 档。

什么现象会推翻：`mount.rs` 里另有一处读择到那一份的 `journal_instance` 做判断（我只 grep 到 3124 一处）。

## 4. G4 会红的用例与变异

**用例**（模型目录 `tests/c834_order_of_devices_decides_the_newer_root_judgement.rs`，放进 `crates/singlefs-harness/tests/` 跑，harness 档、不碰 checker 档）：

- `the_judgement_follows_the_clause_table_under_both_device_orders`（`research/prompts/c834-r1-sonnet-model/tests/c834_order_of_devices_decides_the_newer_root_judgement.rs:395`）：九段历史 × 两种盘序，每格断言「结局 = G2 表里的该怎么判」，并断言做成的格新写掉 R 的单元为 0 份；任一格不符就红，红时逐格打印 `MISMATCH`。
- `units_of_r_stay_when_a_later_mounts_records_landed_on_top_of_r`（同文件 `:373`）：later_mount 一段做成之后，入库助手核写行根落盘前与三次用户动作之后 R 的单元一份不少。
- `print_the_outcome_of_each_history_under_both_device_orders`（同文件 `:338`）：不断言，打印矩阵用。
- 历史怎么造：partial、complete 截 M0（无读故障的取号挂载）的写到第一块、第二块盘的系统配置槽写；urj、units_only 截 M1（R 的根槽、单元、第一条记录读回全零的取号挂载）的写到写行根之前、第一条记录之前；两段回卷用 `FaultInjectingBlockDevice` 让取号第二块盘的写报错、或写之后那道屏障报错（`image_after_a_rolled_back_acquisition`，同文件 `:95`）。

**变异**（K6 的实现为被测，变异在补丁里各一条；结果摘自 `out/summary.txt`，MISMATCH 行数现数）：

| 变异 | 对应补丁行 | 结局 |
|---|---|---|
| 改回读择到那一份的实例代号（= K1） | 补丁 `:84` | 红，3 格（partial 盘序 0,1；complete 两序） |
| 删映射根那一句（= K3） | 补丁 `:88` | 红，4 格（units_only、urj_row_record_also_hidden 各两序） |
| 删根与记录那一句，只留映射根（`k6_only_mapping_root`） | 补丁 `:126` | 红，2 格（later_mount 两序） |
| 记录那一句的 `>` 改 `>=`（`k6_records_ge`） | 补丁 `:137` | 红，10 格 |
| 映射根那一句的 `>` 改 `>=`（`k6_mapping_ge`） | 补丁 `:152` | **存活，0 格红** |

存活变异的原因（推的，量到的只有结果）：384 槽小盘上映射树只有一层，映射根读得出时映射就现算得出，不走进这一判；读不出时（读回全零）`parse_index_node` 失败，`>` 与 `>=` 都不给凭据。要杀它得造多层映射树、根读得出而下面某个节点读不出的盘面：这一形没造，交下一个代码批实现员造或按等价变异登记。

**已有入库用例改不改结局**（K6 补丁下，`C834_VARIANT=k6`，非 checker 档的 harness 测试，各经 `capped.sh 31` 与 `run-with-memory-cap.sh 16G`）：
- 第一批 9 份测试文件，含 urJ 所在的 `recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 与 C694 的 B 相关的 `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`、`abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs`、`writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs`：`out/existing-batch1-k6-result-lines.txt` 现数 9 份文件，58 条通过，0 条失败。
- 第二批 18 份：`out/existing-batch2-k6-result-lines.txt` 现数 268 条通过，0 条失败，24 条是仓里标了慢档的 `ignored`（没跑）。
- 没跑的：同一 grep 列出的 `random_histories.rs`、`acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs` 两份（我没有先估它们的挂钟，没起）；checker 档测试按定义不跑。**判不了** K6 对这两份的影响。
- 这 27 份在 K1（今天的实现）下我没有逐份复跑当对照：入库当前是绿的由主工作区提交时的整轮负责，这里只报 K6 下全绿。

结论：K6 不改这 27 份入库用例的结局（326 条通过）。这些用例分不分得出 K1 与 K6 没有对照：我没在 K1 下复跑它们，判不了；本报告新写的结局表测试才是分得出的那一条（G2 矩阵：K1 下红 3 格）。

什么现象会推翻：把补丁里 K6 的那一支换成 K1，上面 27 份里有一份变红（那就是已有用例本来就分得出，C834 不会漏）；或结局表测试在 K6 下红。

## 5. G5 落在哪条决策的哪个分项

**判定：D23（journal 的角色与格式）已定项 14 已有覆盖这一判的句子，且与实现冲突；新条款改写那一句，不另立项。**

已有的句子（`.claude/kb/decisions/23-journal的角色与格式.md:392`，在「#### 已定项 14」一节内，该节起于第 355 行、止于第 459 行之前）：
「较新可读根那次发布的记录一条都读不出、账也读不出时，映射现算得出就隔离，现算不成不隔离、不拒（主 agent 2026-10-08 定）」

实现（`crates/singlefs-core/src/mount.rs:2078`、`:2079`、`:2089`）：「None if journal_instance_in_the_system_configuration」「<= root.instance =>」走「重读一次，仍不成拒」；「None => continue,」才是不隔离、不拒。注释里写的理由（`crates/singlefs-core/src/mount.rs:2069`）：「那条根之后没有别的挂载取过号，它的单元还在，不拒就会在写行那次发布里写掉它们」。

**冲突**，两边整行并排：
- `.claude/kb/decisions/23-journal的角色与格式.md:392`：现算不成不隔离、不拒（主 agent 2026-10-08 定）
- `crates/singlefs-core/src/mount.rs:2079`：`<= root.instance =>`（这一支走拒，且 `grep -nF '取过号' .claude/kb/decisions/*.md` 零命中：Y2 那一收严 kb 里没有）

别处：
- D18（块里携带什么信息）已定项 11（`.claude/kb/decisions/18-块里携带什么信息.md:319`）：「由 I-7.7（系统配置实例代号不低于根环） 的第 ② 句判成立（较大的号没有任何根、记录、单元带着），不另开例外，下一次挂载按 max + 1 修复」。这一句管取号回卷期间两盘不等的态，是 K3/K4/K6 的锚：不等态里较大的号没有任何东西带着。它没有说本判该怎么读。
- D23 已定项 16（`.claude/kb/decisions/23-journal的角色与格式.md:480`）：「这个号就此烧掉，下一次可写挂载按 max + 1 取下一个」，是 `complete` 一段的出处。
- D22（单元原子性怎么合成）已定项 8（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）：「可写挂载读系统配置的见证按盘读」。没有一句说本判读哪块盘的哪一槽；`grep` 全 kb 找不到一条规定「择系统配置取盘表第一块交得出的盘那一份」的条款（只有 `.claude/kb/experiments/158-择根与修复四岔路.md:1039` 描述了实现这样做）。**规则没说。**
- I-7.7（`crates/singlefs-checker/invariants/02-根环.md:14`）不改：它是池级一致性判，不是挂载这一判的读法。

另外 C834 行本身（`.claude/kb/checks-owed.md:555`）里「（推的，没量过）」那一句，现在量过了（G1）；它的「候选」括注：「候选：逐盘取最小、要求有带那个号的根或记录、按 I-7.7（系统配置实例代号不低于根环） 第 ② 句判」，对应本报告 K2 另读、K3、K4，三者在矩阵里都站不住或有代价。

## 6. 条款草案（K6 形；推的，交主 agent 判，本腿不采纳）

按 `.claude/rules/implementation-first.md` 的写法，改法写成对实现的改动：

- **判据**：可写挂载读阶段，根环里比所选那一版新、按实例表有效、读得出的根 R，那次发布的记录一条都读不出、账读不出、映射现算也不成时，**当且仅当盘上已读到下面三样之一才不隔离也不拒**（放过）：① 根环里有实例代号比 R 大的自证根；② 读阶段扫出的 journal 记录里有实例代号比 R 大的；③ R 的根点名的中央映射树根的两个落点上，有一个现在是自证过的索引节点、它的实例代号比 R 大。三样都没有，照「记录不齐」那一支办：重读一次，看 C694 的 B 凭据，仍不成就在取号之前拒可写。
- **不读什么**：这一判不再读择到那一份系统配置的实例代号；系统配置里的号只说「取过号」，不说「写过」（烧掉的号、取号回卷留在旧槽里的号都是合法态，`.claude/kb/decisions/23-journal的角色与格式.md:480`）。
- **射程**：只管这一支；不改 I-7.7（它判盘面一致性，不是这一判的读法）；不改取号的 max 读法（`crates/singlefs-core/src/transaction.rs:886`）；`choose_system_configuration_naming_devices_without_a_mountable_slot`（`crates/singlefs-core/src/recovery.rs:1092`）仍取盘表第一块交得出的盘，因为 fsid 与设备数逐盘核对过，回退下界与 journal tail 另有逐盘取最大的读法（G3）；别的可变字段我没逐个查。
- **改 `crates/` 哪里**：`crates/singlefs-core/src/mount.rs` 的 `placements_in_the_accounts_of_newer_readable_roots`（第 2031 行起）：第 2078–2079 行那一句守卫换成上面的凭据判；删参数 `journal_instance_in_the_system_configuration` 与调用处第 3124 行；改注释（第 2020–2021 行、第 2066–2071 行）。D23 已定项 14 第 392 行那一句「现算不成不隔离、不拒」按判据改写，并补一句 Y2 收严（kb 里目前没有）。
- **会红的用例与变异**：G4。

## 7. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 现象坐实 | 站得住（坐实） | partial 一段两种盘序一拒一放，放过那一支新写掉 R 的 14 份单元、checker 红 I-2.1、I-4.8、I-7.2、I-7.4；complete 一段两序都放过，是 C834 之外的新现象 |
| G2 K1 今天的实现 | 站不住 | partial 随盘序变、complete 两序都判错；回卷两段答对 |
| G2 K2 逐盘最大再盘间最小 | 站不住 | 与盘序无关，complete 与两盘回卷答错 |
| G2 K2 另读 各盘各槽取最小 | 站不住 | urj 拒，违反入库用例 urJ；units_only、urj_row_record_also_hidden 同样该放过而拒 |
| G2 K3 要有带更大号的根或记录 | 站得住，有代价 | 九段七段答对且与盘序无关；二重故障两段该放过而拒 |
| G2 K4 照 I-7.7 第 ② 句 | 站不住 | 相等的号不分取了没写与写了，complete 与两盘回卷答错 |
| G2 K5 跨盘取最大 | 站不住 | partial 两序都放过并新写掉 14 份，让 C834 稳定走错支 |
| G2 K6 K3 加映射根槽凭据 | 站得住 | 九段全答对、与盘序无关；零轮；映射根识别法只认索引节点 |
| G2 K3 对 K6 | 岔路（交主 agent） | 差别只在二重故障两段与 16 KiB×2 的读；两个都没被攻过 |
| G3 同一个量吗 | 不是同一个量 | 本判取世代号较新的槽，取号与 checker 取全部槽最大，落后支取世代号较新的槽；K3/K6 不依赖择到那一份；C834 行「其余……取最大」与 `crates/singlefs-core/src/mount.rs:5366` 冲突 |
| G4 用例与变异 | 站得住 | 九段×两序的结局表；5 个变异里 4 红 1 活（映射根 `>` 改 `>=`，小盘上等价）；27 份入库用例在 K6 下 326 条全过，2 份没跑 |
| G5 落点 | 冲突 | D23 已定项 14 第 392 行「现算不成不隔离、不拒」与 `crates/singlefs-core/src/mount.rs:2079` 走拒冲突；新条款改写那一句 |
| 跑前条款 1（G1 造不出 ⇒ C834 改推的） | 不触发 | G1 坐实 |
| 跑前条款 2（某候选被造出判错历史 ⇒ 出局） | 触发 | K1、K2、K2 另读、K4、K5 各有可复现的历史（矩阵里「做成[14]」或方向错的格） |
| 跑前条款 3（两个以上候选站得住 ⇒ 岔路单） | 触发 | K3 与 K6 |

## 8. 要主 agent 跑的命令

复核本报告的数（不是必须，只是想核时用）：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/c834-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen /home/fy5090/.local/share/singlefs/work/c834-r1/repo --work <新的草稿目录> --model research/prompts/c834-r1-sonnet-model
```

工作目录随意；内存上限 16G；脚本里每条 cargo 经 `capped.sh 31`；约 2 分半钟；跑完看两行：`name=c834_summary_compare ... 逐行相同` 与 `name=c834_rerun_done lines=253 sha256=46a29e50776af1abe6940b03b65d0b27fcf6260e500110da5251eb23e9258a2d`。
我这样跑了四次：第一、二次（当时摘要里还有一道截行的 `cut`，候选少一个）逐行相同；第三次去掉 `cut`；第四次加了「K2 另读」，现在 `out/` 里的是第四次的，第四次之后没有重复跑。

## 9. 没做什么

- 不判别：不替主 agent 采纳候选、不出判决；第 6 节的条款草案是 K6 形的写法，不是采纳。
- 没改产品代码：补丁只在草稿副本里用，不进 `crates/`；没提交、没 git 写操作。
- 没起 checker 档测试、门禁阶段、QEMU、GPU。
- 没造的历史：多层映射树（存活变异的原因）；写行发布的单元只盖掉 R 的一部分（G2 末段「判不了」）；映射根被数据单元或打包记录单元覆盖（K6 的识别法漏这一类）。
- 没复跑：入库 27 份用例在 K1 下的对照；`random_histories.rs` 与 `acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs` 两份在 K6 下。
- 没读：外部审核报告原文（只用背景材料转述）；攻方腿的任何产出（在禁读清单里）。
- 没核 harness 与 checker 档里别处读 `journal_instance` 的读法是否与结论相关（G3 末段）。
- 复核不了的事：无（本轮没有要虚机或网络的量）。
- 草稿目录里的冻结副本拷贝、编译目录、四次复跑目录交回前删掉。

## 10. cite-check 末行

命令：`python3 scripts/agents/cite-check.py research/prompts/c834-r1-sonnet-output.md --root /home/fy5090/code/singlefs`

```
  ✓ 核了 17 处引文、33 处裸行号，对上 17 处，没判 33 处（其中裸行号没有引文可比 33 处；逐处列在上面）
```
