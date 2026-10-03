# m3-root-ring-code-r2 云端攻方（Opus）报告（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 K4 E2 B2 RR4 S7 -->

被判对象是冻结副本 `/tmp/claude-1000/m3-root-ring-code-r2/tree/`（下文 `crates/...` 的行号都取它；`mount.rs`、`allocator.rs` 与主仓逐字节相同，`walk.rs`、`crash_judge_checker.wgsl` 主仓已被别的会话改过、行号以冻结副本为准）。副本上量出的数只在副本上，不算入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-root-ring-code-r2-opus-model/`。复跑（工作目录任意，草稿目录自取一个空目录）：

```
bash research/prompts/m3-root-ring-code-r2-opus-model/scripts/rebuild-and-run.sh <草稿目录> e2 e2-sweep fh sir avc fz
```

它把冻结副本拷进 `<草稿目录>/tree`、在工作区 members 里加原型 crate `e2-attack-proto`（冻结副本的 crates 一个字节不改），经 `run-with-memory-cap.sh 5G` 与 `capped.sh 15` 编 release；改法 A 的副本在 `<草稿目录>/fixA/tree`（冻结副本加 `fixA.diff`）。每件经 `run-with-memory-cap.sh 2G` 与 `capped.sh 1` 跑，输出进 `<草稿目录>/out/<名>.out`，末行 `rc=`。`fz` 一批 10 件并行（`fuzz-batch.sh`，退出码文件 `b<批号>-<件号>.rc`，`wait` 等齐之后数件数）。腿自己跑出来的输出在 `results/out/`、`results/out-fixA/`，`results/SHA256SUMS`（132 行，sha256 301dc3aa0135bfa83632c7c36356d514079f3a365116344182b2f66527a77bb7）。`results/out/` 里的输出是边改原型边跑的：`b1-*.out` 跑在加 I-3.5 计数之前，`b1`–`b3` 跑在加拒因与窗口外目标打印之前，`e2-*`、`sw-*` 跑在加回退开关之前；用今天的源码复跑会多出那几行或字段，其余逐字段应相同。

模型文件 sha256：

```
8ea4f3b9c1aaa1062981e52fd2885c53156be167cbdb3dc600d8b81aee815ae3  ./fixA.diff
f15a01830fecf010a8fdf44c3aec1bd2912938c058e8906088ab7fb47605d3f3  ./proto/Cargo.toml
6a4afb6971a7e21b458dfab56cf19e6cf9fc64dbf8c3960cc912b1f8c4cdd860  ./proto/src/bin/appearance_fuzz.rs
7c3d8a6fb30923532aa7dfdb10951d0ae5fd9c15266848d3b1f9d53c819bec9e  ./proto/src/bin/applied_version_carrier.rs
ce070de5fcc1d466f50c3b144c713014cf214b52d52da60303dcbe0669818164  ./proto/src/bin/e2_reread_flip.rs
7fcf762a1d28facf30f51b827d864fa3a8068795cb25c10ad4c4304dbbf33562  ./proto/src/bin/front_hole_eio.rs
b95a76272cdb4cb3a16e0655b2580fb492467b5589372302b3f5e02ba347af39  ./proto/src/bin/skipped_instance_rows.rs
1beedd9a6e86158ab8a5da59c1023d1b6c981f37f85f38345757ea1b6ab8343a  ./proto/src/lib.rs
86b6e8b710af1309564c6dde7c802c63a30007c5a4c7e4d976970ff5a5b9ef0b  ./scripts/build.sh
40cffde43ef3d3bddf706925d109f4d7ca960b3f7d95ba2565f85142b060f4b3  ./scripts/fuzz-batch.sh
9c06ab74f3299d5a15146d637ba05c1e169e83616f44997e3af4e0f310ab6fcc  ./scripts/rebuild-and-run.sh
39af7d74dc61467b0048e3241173f56d412d05d6c063092416b2704a7e3a33b9  ./scripts/run.sh
```

## 各格判定一览

| 格 | 攻的形状 | 判定 |
|---|---|---|
| K1 E2 重读的四种结局 | 「读得出而自证不过」那一格：槽里住着一条有效根 h，挂载那一刻读错、h 成了窗口里最小的那一刻那一次重读读坏（瞬时，盘上字节完好） | **打中**：实现按「当没有根」把那一槽从根环表拿掉，回收门槛越过 h；盘上全落、没有崩溃的那一版上池级 checker 判 I-3.1 红，记账比遍历少 196608 字节（不安全方向）。扫的 12 组参数里 12 组红，同参数的对照（重读读得出）12 组 0 组红，挂载也读得出的对照 1 组 0 组红。条款字面没写这一格 |
| K1 E2 改法 A 的代价 | 改法 A（重读自证不过同仍读不出，按记的最大 txg 算）在前洞（那一槽从没写过）加挂载那一刻读错上 | 改法 A 下 I-3.1 红（记账比遍历多，安全方向），今天的实现绿：两种历史在重读那一刻读到的东西相同，是一条判别子观测不到的岔路，交主 agent |
| K1 E2 被抛弃、仍读不出两格，换表那一刻 | 写行前后两张实例表判不同的情形 | 没打中（推的，见 K1 第 3 小节） |
| K1 预演不重读那一句 | 「它按最大 txg 算、回收得只会更少」 | 没打中：进程内重读能读出的只有那个最大 txg 或窗口外 / 没有，句子在今天的取号下成立（推的） |
| K1 B2：F 越过读不出的有效根 | 8-1 报告第五节第 1 点那一形 | 条款字面许可（「根槽这一次读坏」那一行：挂载那一刻读不出、之后没写过的槽当没有根），不算打中；代价是那一个状态出候选集 |
| K1 RR4 抬到 B2 上限 | 有没有用例或变异钉住 | 推的：抬到上限时门槛恒等于新 F，RR4 那一半在这一形上不起作用，没有可钉的东西；8-1 自己的第一趟证红也是这个结果 |
| K2 回退候选集的窗口 | 锚、次序、模型映射；随机历史里拿攻方按条款算的候选（窗口里）与窗口外的有效根去回退 | 没打中（一次抽样）：窗口外的目标 15 次全报 `OutsideTheOneRevolutionWindow`、0 次做成；窗口里的目标被拒的 240 次全是 `VersionWithoutFile`（攻方的候选没滤带不带文件）；模型那一判在理想模型里走不到（8-1 自报） |
| K3 I-3.5 出现规则 | 随机合法历史（覆盖写、空发布、抬 F、回退、崩溃截点、卸载）在每个动作之后与每一命收尾的盘面上跑池级 checker | 没打中（一次抽样）：1000 段不同的历史，其中带 I-3.5 计数的三批 22696 个盘面 I-3.5 全判成立、0 个不适用、0 个违例，其中 2181 个盘面攻方按条款算的候选集相邻 txg 有洞 |
| K3 8-2 自取读法 ③ | 中间实例缺行算接不上 | 没打中：崩掉的那次挂载留下的实例号，下一次写行写成 (j, 0, 0)，两种截法都量过 |
| K3 8-2 自取读法 ②、④ | 同 txg 并组、整对不适用 | 没造出同 txg 几条候选的合法历史，没量 |
| K4 GPU 走读内核取「由记录施加出来的那一版」的实例表与分配记录树 | CPU 从那条记录的新根段取，GPU 从它施加在其上的那条根取 | 线索（两边代码不同口径），合法历史上两边取到的指针相同（1 段历史量过），没跑 GPU |

## K1　E2、B2、RR4 落地

### 1. E2 重读「读得出而自证不过」⇒ 当没有根：打中

**历史**（`e2_reread_flip 4 20 2 flip`，`FLIP_ONLY_AT_ANCHOR=1`；S = 4，环长 12）：

1. mkfs → 实例 1 第一个文件 → 覆盖写到 txg 20，每次全落。根环槽 X = (区域 0, 槽 2) 住着 txg 18 的有效根（h = 18）。
2. 实例 2 可写挂载，挂载那一刻读 X 报设备读错。挂载照条款记成「能住的最大 txg」：输出 `MOUNTED instance=2 current=22 hits=20 x_in_table=RootRingSlot { region: 0, slot: 2 }: UnreadableAtMountInsideTheWindow { highest_txg_it_can_hold: CheckpointTxg(18) }`。许可它的是「挂载时读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的」（`.claude/kb/decisions/16-发布语义.md:35`）。
3. 读错撤掉，覆盖写推到 txg 28，每次 X 都读得出、表不动（输出 `PUBLISHED txg=23` … `txg=28` 七行都是 `UnreadableAtMountInsideTheWindow`）。
4. 发 txg 29 = h + 3S − 1 那一次覆盖写时，读 X 读出来有一个字节取反（读故障，盘上字节不变；这一次发布里读了 X 三次，三次都坏，之前之后都好）。记 29 这条根时 X 是窗口里最小的那个，E2 重读它，读到自证不过。实现照 `crates/singlefs-core/src/mount.rs:1296`「| RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {」交「RereadOfARootRingSlotUnreadableAtMount::NoValidRoot」（`crates/singlefs-core/src/mount.rs:1297`），分配器照 `crates/singlefs-core/src/allocator.rs:1906`「RereadOfARootRingSlotUnreadableAtMount::NoValidRoot => {」把那一槽拿掉：「occupancy.occupants.remove(&ring_slot);」（`crates/singlefs-core/src/allocator.rs:1907`）。成员注释写明取法：「/// 读得出而自证不过，或读出来的根按实例表被抛弃：当没有根（与挂载那一刻读得出而自证不过的槽同一个处置）。」（`crates/singlefs-core/src/allocator.rs:1109`）。输出 `PUBLISHED txg=29 x_reads=3 x_in_table=（表里没有这一槽）`。
5. 那一槽出了根环表，门槛越过 18（按 D16 的可再分配谓词推的；量到的是下面那一行 I-3.1）。读故障撤掉之后，txg 29 那一版（全落、没有崩溃）的盘面上池级 checker：

```
ANCHOR txg=29 hits=35 x_in_table=（表里没有这一槽） red=["I-3.1：盘 0：记账的已分配 Some(2031616)，遍历全部有效根得到 2228224（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 29、环里自证过的根槽 12 个、最老的自证过的根 txg 18、遍历的候选根槽 12 个、并进遍历的由记录施加出来的版本 0 个、被实例表判"]
```

记账少 2228224 − 2031616 = 196608 字节（12 个 16 KiB 槽）：盘上 txg 18 那条根自证过、按实例表有效、在窗口里（锚 29，窗口 txg > 17），它引用的槽在记账里成了空闲。同一组参数的对照臂：`clean`（挂载那一刻读错、重读读得出）`ANCHOR txg=29 … ValidRootReadWhenTheSlotUnreadableAtMountWasReread … red=[]`；`none`（挂载也读得出）`red=[]`。

**扫过的参数**（`e2-sweep`，读故障都只在发 h + 3S − 1 那一次里）：S ∈ {4, 6}，P = 3S + 8，back ∈ {1, 2, 4}（h = P − back），内容 3000 / 20000 字节，`flip` 12 组全红（txg h + 3S − 1 那一版上 I-3.1 红），`clean` 12 组全绿；内容 60000 字节的 12 组在建前缀时就停（`ContentExceedsDataUnit { bytes: 60000, capacity: 32634 }`），不算。

**数据层没造出**：下一次发布（txg h + 3S，盖掉 X）的单元写在 12 组里都没落到 h 引用的槽上（`unit_writes_over_h_refs=[]`）；读故障撤掉之后在 txg 29 那一版上管理员回退到 (实例 1, txg 18) 做成了、读回来是 h 那一版的内容（`ROLLBACK_AT_ANCHOR=1`：`AFTER_ROLLBACK newest=30 reads_version_h_content=true red=[]`，之后三次覆盖写 0 红）。所以今天量到的是记账与 D16 定义的门槛不符，不是已经丢数据；分配器挑别的槽的那几个几何上落不落到 h 的槽，这一轮没扫到。

**崩溃状态全量没跑**：txg 30 那次发布的层 0 枚举域是 `closed_form_state_count=1048583`（超过约 10⁶ 一点），按随机历史第 1 批的耗时推（10 件并行挂钟 2 分 7 秒，每件约 915 个盘面，含发布，单个盘面的 checker 不超过约 0.14 秒），全量单线程约 40 小时，远超 40 分钟，没跑（推的）；打中靠的是全落的那一版，不依赖崩溃状态。

**四句**：

1. 分不分辨臂：分辨。被分的是「重读读得出而自证不过」怎么记——今天的「当没有根」红，改法 A（按记的最大 txg 算）在同样 12 组上 0 组红（量过，`results/out-fixA/fa-*.out`）。
2. 系统当时看不看得到判别它的东西：看不到。重读那一刻读到「自证不过」的有两种合法历史：(i) 那一槽真的没有根（前洞：根槽从没写过，挂载那一刻另报了一次读错）；(ii) 那一槽住着有效根、这一次读坏（本例）。今天的取法在 (ii) 上错、错在不安全方向；改法 A 在 (i) 上错、错在安全方向（见第 2 小节量的数）。这是第一轮 K2-e 那一族「判别子观测不到」：池级 checker 读盘、读故障不留在盘上。
3. 满足的是判据字面的哪一句：I-3.1 的「已分配统计对得上」那一判（记账小于遍历）；对条款，D16 已定项 1 那一节把门槛定义成「窗口里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg」，盘上 txg 18 正是这样一条根，门槛越过了它。E2 那一句只写了两种结局：「先重读那一槽，读得出且自证过就照读出来的根算，仍读不出才按记的最大 txg 算」（`.claude/kb/decisions/16-发布语义.md:35`），「读得出而自证不过」条款没写，是实现自取的，8-1 报告第五节没把它列进自取读法。D16 同一节的「根槽这一次读坏」那一行写「这一次读不出或自证不过就重读一次」（`.claude/kb/decisions/16-发布语义.md:38`），把它当成会发生的读故障，所以 (ii) 在条款自己的故障模型里。
4. 跑前条款给的改法：正文第五节没列改法（打中 ⇒ 改法进第 9 批）。我提的几个见下表。

| 改法（只在我的模型上量过、被攻过零轮） | (ii) 读坏一次的有效根（本例 12 组） | (i) 前洞加挂载读错（2 组） |
|---|---|---|
| 今天：自证不过 ⇒ 当没有根 | 红 12 / 12，不安全方向（量过） | 绿 2 / 2（量过） |
| A：自证不过与仍读不出同一处置，按记的最大 txg | 绿 12 / 12（量过） | 红 2 / 2，各一个盘面，记账多（量过） |
| B：自证不过再重读一次，仍自证不过才当没有根 | 本例读故障持续整次发布（那一次里读 X 三次都坏），B 照样红（推的） | 绿（推的） |
| C：自证不过 ⇒ 这一次不改表、下一次记根再重读（窗口最小的那一槽在下一次发布就被盖掉，等于 A） | 同 A（推的） | 同 A（推的） |

抬 F 那一遍的门槛走同一个函数（`oldest_valid_root_in_the_window_for_raising_the_floor` 里调 `reread_of_a_root_ring_slot_unreadable_at_mount`），同一形在抬 F 上也成立（推的，没量）。

### 2. 改法 A 的代价：前洞加挂载那一刻读错

**历史**（`front_hole_eio 4 9 mount`）：实例 1 写到 txg 8，txg 9 那次覆盖写在根槽写之前崩（记录已持久、根槽没写；环还没转过来，那一槽 (区域 0, 槽 3) 从没写过、读出来自证不过）→ 实例 2 可写挂载，挂载那一刻读那一槽报读错（之后照读）→ 覆盖写到 h + 2·3S，每一版之后跑池级 checker。只有一次崩溃、一次读故障。

| 副本 | h = 9 | h = 10 | 挂载也读得出的对照 |
|---|---|---|---|
| 冻结副本 | 22 个盘面全绿 | 21 个盘面全绿 | 两组全绿 |
| 改法 A | txg 20 那一个盘面红，其余绿 | txg 21 那一个盘面红，其余绿 | 两组全绿 |

改法 A 下红的那一行（`results/out-fixA/fh-s4-h9-mount.out`）：

```
PUBLISHED txg=20 x_in_table=RootRingSlot { region: 0, slot: 3 }: UnreadableAtMountInsideTheWindow { highest_txg_it_can_hold: CheckpointTxg red=["I-3.1：盘 0：记账的已分配 Some(2228224)，遍历全部有效根得到 2097152（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 20、环里自证过的根槽 11 个、最老的自证过的根 txg 10、遍历的候选根槽 11 个、并进遍历的由记录施加出来的版本 0 个、被实例表判"]
```

这是记账多（安全方向），与第一轮 K2-e 同形，E2 仍读不出那一支今天本来就留着这一形。改法 A 在冻结副本的三份 harness 用例上照过（副本上的数）：`root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window`「test result: ok. 1 passed; 0 failed」、`raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window`「test result: ok. 2 passed; 0 failed」、`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over`「test result: ok. 24 passed; 0 failed」。冻结副本上 E2 那份新用例文件只有 1 条用例（`a_root_ring_slot_unreadable_only_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window_and_the_allocated_statistic_holds`），重读读出的是窗口外的 txg 0；「重读自证不过」「重读读出被抛弃的根」「重读仍读不出」三种结局都没有用例，所以改法 A 不碰任何现有用例。

交主 agent 的岔路（零轮）：重读那一次读到「自证不过」时，今天的取法在一次读坏的有效根上错在不安全方向（记账少、门槛越过盘上一条有效根），改法 A 在从没写过的槽上错在安全方向（记账多一格、只在那一版上）。

### 3. 被抛弃、仍读不出、写行前后换表（没打中，推的）

- 仍读不出：照记的最大 txg 算，与 E2 原句一致；第一轮 K2-e 那一形照旧（已知、用户定 E2 时知情）。
- 进程内重读能读出的 txg 只可能是记的最大那一个或窗口外：挂载时区间取 (N − 3S, first_txg − 1]（`crates/singlefs-core/src/mount.rs:1669`「/// 挂载时读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行，」），一个槽的两个 txg 差 3S（`crates/singlefs-core/src/mount.rs:1695`「// target_for_publish 只看 txg mod 3S（区域 = txg mod R，槽 = (txg div R) mod S），所以一个槽在区间里的最小 txg 落在区间头」）；进程内的锚 ≥ first_txg，小的那一个从第一次记根起就在窗口外。所以「记下的 txg 高于真住在那一槽的根」只在那一槽真住着窗口外的根或没有根时发生，那时把它当窗口里一条根只让门槛更低（安全方向）。
- 被抛弃：重读读出被抛弃的根当没有根，那一槽从根环表拿掉、它引用的落点不进隔离；挂载那一刻读不出时影子账本来就没罩它（8-1 报告第五节第 2 点自己写了「它引用的落点挂载时影子账没罩，离开根环时也就不清」），重读之后不罩不比挂载时更松。病根在挂载那一刻影子账读不到那一槽，与 E2 无关，没往下造。
- 换表：挂载时装最新那条根的表（`crates/singlefs-core/src/mount.rs:1913`），写行那次的表定下来就换（`crates/singlefs-core/src/mount.rs:4291` 起那一处）。两张表判法不同只在「上一实例 txg 高于所选那一版」的根上；写行那次记根时会被重读的槽能住的最大 txg 是 first_txg − 3S + 1，比所选那一版小，两张表对它判得一样；之后最新那条根带的就是写行那一张，与 checker 同一张。没造出两张表判法不同而那一槽正好被重读的历史。
- 预演不重读那一句（`crates/singlefs-core/src/mount.rs:1310`「/// 抬 F 那一串在分配器拷贝上的整串预演（`rehearse_the_publishes_raising_the_floor`）不重读：它按最大 txg 算、回收得只会更少，」）：照上一条，真发时重读只会读出同一个 txg（回收一样）或窗口外 / 没有（回收更多），句子成立；「读得出而自证不过 ⇒ 当没有根」让真发回收得更多，方向也对。

### 4. B2：F 越过窗口里一条读不出的有效根

8-1 报告第五节第 1 点那一盘面（X = 21 一直读不出，上限按 B2 取 22，F 抬到 22，X 落到 F 之下、出了回退候选集）。条款：抬 F 的上限那一行「都只在一圈窗口里的根上取」（`.claude/kb/decisions/16-发布语义.md:37`），读根环那一格「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根」（`.claude/kb/decisions/16-发布语义.md:38`）。所以 F 越过 X 是条款字面许可的，不算打中。代价写明：读故障让 X 那一个状态从候选集里没了；条款「最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现」那一条只管写失败，读故障下保留几个状态条款没说。这一形下 checker 的 I-7.9 红是第二节列的已知项（I-7.9「之前的根」取全环），不算。

### 5. RR4 抬到 B2 上限那一形

抬 F 那一遍门槛 = max(新 F, 窗口里最旧有效根)。抬到上限时新 F 就是窗口里一条有效根的 txg（某块盘最新的有效根，或第 4 新的非空有效根，或最旧有效根），窗口里最旧有效根不大于它，门槛恒等于新 F：RR4 并不并进挂载时读不出的槽，在这一形上门槛都一样（推的）。8-1 报告第六节第 8 行写的「第一趟没红（用例抬到 B2 之后的上限 22，F 高于 X、门槛由 F 定）」是同一件事。所以「抬到上限」那一形没有 RR4 可钉的东西，用例抬到 6 才分得出 RR4 是对的；不算缺口。

## K2　回退候选集的窗口（没打中，一次抽样）

- 锚：`roll_back_by_a_forward_publish` 把现行那一版的 txg 交给 `rollback_candidate`（`crates/singlefs-core/src/mount.rs:6039`），抬 F 那一遍同取「let window_anchor = current.root.checkpoint_txg;」（`crates/singlefs-core/src/mount.rs:2859`）；D16 已定项 1 的锚「进程内每记一条根时取这条根」，挂着时现行那一版就是这个进程记到的最新根，两处同一个。
- 次序：成员次序是 NotInRing、BelowEffectiveFloor、OnAbandonedTimeline、OutsideTheOneRevolutionWindow（`crates/singlefs-core/src/mount.rs:767`）、VersionWithoutFile（`crates/singlefs-core/src/mount.rs:769`），`rollback_candidate` 照这个次序判。一条窗口外的目标同时低于 F 或被抛弃时报前两者之一，同时不带文件时报窗口外；模型一侧收的是「应报的拒因集合」（`required_refusals`），报哪一个都在集合里，不会两边判不同。
- 模型：窗口用固定的环长（「checkpoint_txg.0 + ROOT_RING_REGIONS * MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM」，`crates/singlefs-harness/src/model.rs:442`），与模型自己的根环位置同一个常量；理想模型不建崩溃，根环里没有窗口外的根，新拒因在模型对拍里走不到（8-1 报告第五节第 6 点自报），所以模型对拍里也就没有一条历史能让两边判不同，这一判只靠 `rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_is_refused_before_any_write` 那一条用例钉。
- 量的：随机历史第 4、5 批（`appearance_fuzz`，第 5 批 `FUZZ_ANY_TARGET=1` 让目标也从窗口外的有效根里挑）：窗口外的目标 15 次，全报 `exclusion: OutsideTheOneRevolutionWindow`，做成 0 次（`ACCEPTED_OUTSIDE` 0 行）；窗口里的目标被拒 240 次（第 4 批 117、第 5 批 123），全是 `exclusion: VersionWithoutFile`（攻方的候选没滤带不带文件，是攻方装置的口径，不是实现的错）。

## K3　I-3.5 出现规则（没打中，一次抽样）

**随机合法历史**（`appearance_fuzz`，不注读故障）：一段历史 2 到 5 命，每命一次可写挂载加 0 到 3S + 6 个动作（覆盖写、空发布、准入抬 F、回退到攻方按条款算的候选），收尾是崩溃（前 k 段全落加第 k 段任一子集）、正常卸载或直接退出；每个动作之后（那一刻全落）与每命收尾的盘面跑池级 checker。用户动作都放开随机取，只固定起点（mkfs 加一个随机长度的前缀）。

| 批 | S | 动作分布（覆盖写 / 空发布 / 抬 F / 回退） | 历史段数 | 判过的盘面 | I-3.5 成立 / 不适用 / 违例 | 候选集有洞的盘面（其中成立） | 别的红 |
|---|---|---|---|---|---|---|---|
| 1 | 4 | 50 / 12 / 12 / 26 | 300 | 9154 | 没计（这一批跑在加计数之前；违例清单里没有 I-3.5） | 没计 | I-7.9 43 段 |
| 2 | 4 | 35 / 10 / 10 / 45 | 250 | 7234 | 7234 / 0 / 0 | 663（663） | I-7.9 34 段 |
| 3 | 6 | 50 / 12 / 12 / 26 | 200 | 7977 | 7977 / 0 / 0 | 883（883） | I-7.9 46 段 |
| 4 | 4 | 同第 2 批，同种子（加了拒因打印，结果逐行与第 2 批同） | 250 | 7234 | 7234 / 0 / 0 | 663（663） | I-7.9 34 段 |
| 5 | 4 | 同第 2 批，目标可以落在窗口外 | 250 | 7485 | 7485 / 0 / 0 | 635（635；这一批「有洞」把窗口外的根也算进候选） | I-7.9 27 段 |

I-7.9 红是第二节列的已知项（池级 checker 的 I-7.9「之前的根」还取全环），这里不算打中；它们都出现在后一命看前一命抬 F 的那条根时（例：第 1 批种子 1「life1/step0/txg32: 实例 2 txg 24 那条根把回退下界 F 从 16 抬到 17，高于它之前的根算出的抬 F 上限 16」）。崩溃截点是历史的一部分（下一命从哪里起），这一节不对截点之外的崩溃状态下结论。

**8-2 自取的四处读法**：

- ① 只判出现规则、③ ④ 没实现：第二节列为已知，不算。
- ② 同 txg 几条并组、gap-row 取「组里任一实例接得上就算」（「.any(|previous_instance| bridged_from(*previous_instance))」，`crates/singlefs-checker/src/walk.rs:491`）：要同 txg 的候选来自不同实例。新实例第一次发布的 txg 取根环与 journal 里见过的最大 txg 加一，合法历史上我没造出同 txg 的两条候选（随机历史里一次都没有），没量；按条款「同 txg 的几条并成一条（R⁻ 侧引用集合取并…）」，R⁻ 侧并成一条之后它的实例取哪一个条款没写，这一读法比「每个实例都要接得上」宽：宽的一边多照常判（gap-row 下用户可见树里的单元也判），在合法镜像上误红要那一对真的有洞、而 R 只接在组里一个实例上，推的。
- ③ 中间实例缺行算接不上（「.filter(|row| row.instance > previous_instance && row.instance < version_instance)」，`crates/singlefs-checker/src/walk.rs:475`）：量了实例 2 的可写挂载崩在写行那次根槽写之前（`sir-row`）与一个写都没落（`sir-zero`）两种：前者实例 3 写行之后最新根的实例表是 `[InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(8), applied_transaction_high_water: 0 }, InstanceRow { instance: InstanceGeneration(2), selected_root_txg: CheckpointTxg(0), applied_transaction_high_water: 0 }]`，跳过的实例号有 (2, 0, 0) 那一行，I-3.5 `Holds`；后者不跳号（实例 3 的挂载取到的还是实例 2）。实现今天不留缺行的跳号，这一读法在合法历史上碰不到。
- ④ 整对不适用读成那一对不判（「if !previous_references_are_complete || !version.references_are_complete {」，`crates/singlefs-checker/src/walk.rs:544`）：只让判得少，不会在合法镜像上误红；随机历史里 I-3.5 一次不适用都没有，说明那几批里走读一次都没断。

**看过、没造反例的一处**：落点 → 用户可见树的标签按第一次走到它时记（「self.unit_of_each_placement.entry(placement).or_insert(unit);」，`crates/singlefs-checker/src/walk.rs:1078`，标签那张表同一个写法）。同一个落点在两版里是不同的单元、或在两版里挂在不同的树下，标签会沿用第一版的。候选集里的两版共用一个落点而内容不同，本身就是 I-7.4 那一类的违例，合法镜像上不出现；树 ID 在回退前后不变。所以只影响坏镜像上的判别力，推的，没量。

**走读两处修正对 I-3.1、I-3.10、I-5.4 的影响**：上面五批里这三条一次都没红。

## K4　I-8.10 七段、RR1、RR3 与 GPU 两侧

- 七段偏移：core 编码新根段 99 起依次树表指针 86、映射根 86、水位 8、F 8、实例表指针 86、分配记录树根 86、卸载记号 4，共 364，fsid 在 463，MAC 16，头到 487（`crates/singlefs-core/src/journal.rs` 编码那一段）；GPU 那一判「&& bytes_equal(r.base, segment + 360u, root.base, 20u, 4u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:709`）与前六段的偏移逐个对得上（188 → 根记录 170，274 → 342）。在两份 wgsl 里按旧值 311、287、295、196、67 搜，剩下的 185、271、279 都是新根段里前四段的偏移（起点 99 没变），67 那两处按上下文是单元头里的字段偏移，与点名项上限无关。
- RR1：GPU 记录有效那一判「if ((rd32(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 360u) & 0xFFFFFFFEu) != 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:306`）与 core 读者（卸载记号位 0 之外非 0 交 None）同口径；CPU checker 那一侧这一轮没逐行核。
- **线索（没跑 GPU）**：「由记录施加出来、根槽从没落盘的那一版」的实例表与分配记录树，CPU checker 从那条记录的新根段取（「parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER]);」，`crates/singlefs-checker/src/walk.rs:1350`），GPU 走读内核从它施加在其上的那条根取（「// （WALK_KIND_VERSION：树表与映射取自那条记录，实例表与分配记录树取自它施加在其上的那条根；没有那条根就不走这两样）。」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2594`；「carrier_base = applied_on_base;」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2609`；那条根取「同实例、txg 比它小、txg 最大的那一条」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3020`）。第二节列的 S7 说的是 GPU 恢复内核拼新根，这是 GPU checker 走读，另一处。两边取到的指针不同时 I-3.1 等走读类判定的位会分开。合法历史上：带文件的覆盖写记录的实例表照抄上一版、分配记录树根恒零；写行那次的记录不被施加（下一次写行把那个实例写成 (j, 0, 0)，见 K3 ③）；我量的一段（树表 0 条的池，第一次暖机记录已持久、根槽没落，再挂一次）里那一版的两条指针与它施加在其上的那条根逐字相同（`avc-mount`：`same_instance_table_as_applied_on=Some(true) same_allocation_root_as_applied_on=Some(true)`）。所以今天合法历史上推的是两边取到同一个值；只在外来字节或以后记录里的实例表不再照抄时分开。没在 GPU 上跑对拍（checker 档、要显卡），见「要主 agent 跑的命令」。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| K1 E2 | 重读读得出有效、读出窗口外的根、仍读不出、挂载那一刻读得出（对照） | S ∈ {4, 6}，back ∈ {1, 2, 4}，两种内容长度；只有「读坏一次」那一格红 |
| K1 E2 改法 A 的代价 | 前洞加挂载读错 | S = 4，h ∈ {9, 10}，各推两圈 |
| K1 B2 / RR4 | 按条款读、按 8-1 第五节第 1 点与第六节第 8 行的盘面推 | 没另造盘面 |
| K2 | 随机历史里回退到窗口里与窗口外的目标 | 第 4、5 批 500 段，窗口外 15 次、窗口里被拒 240 次、做成的次数见各段 `log=` 里的 `R<txg>` |
| K3 | 随机合法历史，覆盖写 / 空发布 / 抬 F / 回退 / 崩溃截点 / 卸载 | 1000 段不同的历史，S ∈ {4, 6}，journal 环 1 MiB，盘 256 MiB；带 I-3.5 计数的 22696 个盘面，2181 个有洞 |
| K3 ③ | 崩掉的挂载留下的实例号 | 两种截法 |
| K4 | 偏移逐个对、记录施加版本两边取的指针 | 一段历史 |

## 这条腿自己的限度

- 打中的那一格要两次读故障（挂载那一刻读错，之后那一次读坏），读坏是我在原型的读包装里注入的（读出来改一个字节），不是装置里已有的故障类；我按 D16 已定项 1「根槽这一次读坏」那一行把「自证不过」当成会发生的读故障，这一前提主 agent 要核。
- 打中只在全落的那一版上量了 I-3.1；单元写真落到 h 引用的槽上（数据层的坏）没造出来；txg h + 3S 那次发布的崩溃状态全量（1048583 个）没跑。
- 随机历史不注读故障、不缩 journal 环，环上有洞只来自崩溃截点；崩溃截点只取了一个，不是那一次发布的崩溃状态全量。I-3.5 的判别力（坏镜像上该红的红不红）没攻。
- 改法 B、C 没实现；改法 A 只在三份 harness 用例上照过，变异表、checker 档都没跑。
- 一次抽样：K2、K3、K4 的「没打中」照 `.claude/rules/three-way-inference.md` 要再抽一次才记。
- 原型跑出来的崩溃状态全量：0 个（`enumerate_layer0_versions` 一次都没跑满；`closed_form_state_count` 只算了一次，1048583）。

## 要主 agent 跑的命令

1. 复跑这条腿的全部输出（工作目录主仓根，约 15 分钟，内存包装与线程数在脚本里：编译 5G、每件 2G、并行 10 件、每件 1 线程）：
   `bash research/prompts/m3-root-ring-code-r2-opus-model/scripts/rebuild-and-run.sh <新草稿目录> e2 e2-sweep fh sir avc fz`
   看 `<新草稿目录>/out/e2-s4-p20-b2-flip-once.out` 的 `ANCHOR` 行（I-3.1 红）与 `e2-s4-p20-b2-clean.out` 的 `ANCHOR` 行（`red=[]`），`<新草稿目录>/fixA/out/fa-*.out` 的 `ANCHOR` 行（都不带 I-3.1），与 `results/` 里同名文件比（`b1`–`b5` 用的是加打印之前的二进制，复跑会多 `REFUSED` / `OUTSIDE_TARGET` 行与 `b1` 的计数字段，`FUZZ` 行其余字段应相同）。
2. 要坐实到入库装置上：在 `crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` 那一类装置里加一个只在一次读上把根槽改坏的读包装，照第 1 小节的历史（S = 4，实例 1 写到 20，槽 (0, 2) 挂载读错，发 txg 29 那一次读坏），断言 txg 29 那一版 I-3.1 成立——今天的实现应当红。这是装置改动，归实现员。
3. GPU 那条线索：要在显卡上看「由记录施加出来的那一版」两边判不判同一个，需要一条施加版本的实例表或分配记录树根与它施加在其上的那条根不同的盘面（外来字节）跑 GPU 判器对拍；属于 checker 档，提交时或用户要求时由主 agent 跑。

## 没做什么

- 没判正推腿那几格（K5、K6），没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑 checker 档、GPU、变异表、门禁；只跑了自己的原型（release）与冻结副本拷贝上的三份 harness 用例。
- 没读禁读清单里的文件。

## 草稿目录

`/tmp/claude-1000/m3-root-ring-code-r2-opus/`：交回之前删掉了两份仓副本与编译目录（`tree/`、`target/`、`fixA/tree/`、`fixA/target/`、`fixA/target-test/`），留下 `out/`、`fixA/out/` 的原始输出与几份日志（都已拷进模型目录 `results/`），`model-sha.txt`。
