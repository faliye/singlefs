# 根环实现第 9 批代码轮第三轮：云端攻方（Opus）报告（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 K1-a K1-b K1-c K1-d K1-e K2-a K2-b K2-c K3-a K3-b K3-c RS1 RS2 RS3 R2 Q5 F5 A2 -->

攻击面：K1（重点）、K2、K3。所有代码行号按冻结副本 `/tmp/claude-1000/m3-root-ring-code-r3/tree/` 引；kb 行号按主仓 kb 文件现查。原型都在冻结副本的拷贝上（草稿目录 `/tmp/claude-1000/root-ring-r3-attack/tree/`），副本上的数不是入库装置上的数。只跑 harness 档与一个自建的非 checker 档二进制（见「复跑」）。

## 复跑

模型目录 `research/prompts/m3-root-ring-code-r3-opus-model/`：

```
bash research/prompts/m3-root-ring-code-r3-opus-model/rebuild-and-run.sh <草稿目录> k1-scan k1-long k1-long-369 k1-fix-a k1-adversarial k1-reread k1-reread-fix-b k2-zeros enum-0 enum-2
```

脚本把冻结副本 rsync 到 `<草稿目录>/tree`，打 `probes.patch`（四份文件：`instance_table_sweep.rs` 加两个只在环境变量开着时生效的改法 A2 / B；两份 harness 用例文件末尾各加一条 `r3_attack_*` 原型用例与一个读回方式参数；工作区 `Cargo.toml` 加成员 `crates/r3-attack-enum`），放进 `r3_attack_sweep_probe.rs` 与 `r3-attack-enum/`，每段经 `research/scripts/capped.sh 10` 与 `run-with-memory-cap.sh 4G` 跑，输出落 `<草稿目录>/out/<段>.log`。判读行以 `SCAN` / `LONG` / `REREAD` / `RS3Z` / `ENUM` 开头。`results/` 里是这一轮原样留存的输出（文件名与段名的对应：`scan-40`=k1-scan，`long-60`=k1-long，`long-420-t369`=k1-long-369，`fixA-long`=k1-fix-a，`adv-w0`/`adv-w6`=k1-adversarial（`PROBE_WARM` 0 / 6），`reread`=k1-reread，`reread-fixB`=k1-reread-fix-b，`rs3z`=k2-zeros，`enum-n0`/`enum-n2`=enum-0 / enum-2，`p1-16` 是 `k1_unmount_cycles` 在 `PROBE_CYCLES=16` 下的输出，`long-partial` 是 k1-long 在 `PROBE_MOUNTS=30 PROBE_CHECK_EVERY=1` 下的输出）。`long-60.log` 是加 `partial=` 字段之前跑的，复跑会多出这一段，其余逐字段应相同；改法 A2 / B 两段代码在没设 `R3_ATTACK_FIX_A` / `R3_ATTACK_FIX_B` 时不生效，前后跑的几段同一个行为。

```
41405fae31656213763619f3b66b3005d96bf864a79c8a48d66c73b6cfeaee4d  ./probes.patch
fda213a16b19acb1a8a22525329f000291070e20e21f59dee53d0a5c1b9672d6  ./r3-attack-enum/Cargo.toml
6efce4e1020d2dd8c98eb1f71576032c9adca36ecbd623eb4674540e8e470d82  ./r3-attack-enum/src/main.rs
31e5ea6c707727cf29b3cc7e95d5e56318340f2fb0eb6a5ca5010f7a85c97dbd  ./r3_attack_sweep_probe.rs
0b8bc379a2217c706b83f7728b5aca723bc91ee2fa2f047b13de19fd4b7bcc04  ./rebuild-and-run.sh
51098f40fd111d6f3c69c70e362b657a763cfef80d7642a6c914a7ff5675ae84  ./results/adv-w0.log
ab7746bf7112fafef0605365592c60446d27a022178d59a20ba846e5d76694c8  ./results/adv-w6.log
7252c5fd77583c7ca89558a192b3bba56ec240143942026a6a7902b8058d746d  ./results/enum-n0.log
1e6d84eef9fea05b2ca040ad571f06da08a0d570c6bf4b63c3b72ff065296375  ./results/enum-n2.log
778c368b4cabd20a928c44a11528a37c77efca1e007aba2fb44fc9e0be500f03  ./results/fixA-long.log
beb93612d3832c5f958bc9440237c20ef3c0ed42e2256629e0643cd82e50f811  ./results/long-420-t369.log
d2e86648944b37c0331ac605fc2c2368da26a1af8a1a07e43d87adff875d9ad9  ./results/long-60.log
0acecd40af5cbe2971276215753fd6f65b0ca9a39b8422950596e6406dd94249  ./results/long-partial.log
9f299cdb8fcdb58cf794ff909fb247cd777b5c3e3b146be32ea2fccbc3886b4a  ./results/p1-16.log
1089fb8ed194b6256b249b20ce1ebe9724aff3a186e7e29c644ba42eb529f6ba  ./results/reread-fixB.log
56b5d446358adb17e92bd217e0005951ba7c4b7079fa9ce695bc8a09592dad84  ./results/reread.log
7a7d48d6b39ed6e965b0c23663c21e00c2e8799717006953fe90945fc822aca2  ./results/rs3z.log
83d2165859914ff6f5cfa2b10a2486fd6847c11f66b437cff200d59b7392663b  ./results/scan-40.log
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-a 清扫器自取的「根环里一条可读根的账读不出就不删」 | **打中（量过）** | 无故障的合法历史「覆盖写 1 次后正常卸载」与「覆盖写 1 次后不卸载（崩在最后一次发布之后）」交替，每次挂载末尾的清扫都停在 `AccountOfARingRootUnreadable`：60 次挂载行数 59、一行没删；产品阈值 369 下 420 次挂载行数 419，越过阈值之后 50 次清扫 50 次停 |
| K1-b 抹之前那次重读读失败 | **打中（量过）** | 一个孤儿那一槽判的那次读照读、抹之前那次重读报设备错（或读回全零）：那一个头不抹，清扫发布照发、行照删；删行之后按删前那一行判未发布的孤儿剩 1 个。层 0 全量 41032 个崩溃态里删行生效的 12 个态全部剩 1 个，oracle 与池级 checker 0 违例（看不见）；不注入的对照 41033 态剩 0 |
| K1-c 清扫器自取的「池里有写序实例大于挂载根的单元就不删」 | 没打中（推的） | 取号式子与回卷次序让盘上不会出现这样的本池单元；没造出 |
| K1-d 认领规则按整单元跳把孤儿跳过（漏回收） | 没打中（推的） | 要「第一槽留着旧码 1 / 码 3 头、第二槽住着孤儿节点」，今天的分配器取不出这种落点；没造出 |
| K1-e 崩在抹头与清扫发布之间 | 没打中（量过，一段历史） | K1-b 的对照流全量 41033 态：恢复、oracle、池级 checker 0 红，F5 剩 0；带读故障那一条只在删行生效的态上红（K1-b） |
| K2-a Q5「只并读不出的槽」 | **打中（量过）** | X 那一槽从第四次挂载起读回全零（读得出而自证不过）：根环表不记它，准入抬 F 上限算成 22、F 抬到 22 越过盘上那条非空有效根 X = 21；池级 checker 判 I-7.9 红。设备报错那一形照旧停在 21、0 红 |
| K2-b `mount_transparency.rs` 的 R2 与 core | 没打中（读代码对照） | 逐槽读法、两条谓词与 core 一字对得上 |
| K2-c RS1、RS2 | 没攻 | 时间给了 K1、K2-a |
| K3-a「部分没判」并进「不适用」 | 没打中；线索一条 | 层 0 与崩溃注入并进不适用只会少算判过；`bad_disk_input.rs` 的「一条都没判」把部分没判算成判过（推的，没造出影响） |
| K3-b I-9.28「树表读不出」那一格的边界 | 线索（量过） | K1-a 那条无故障历史上 30 张挂载后盘面 15 张报 I-9.28 部分没判，原因都是 F 之下、树表已被合法复用的旧根 |
| K3-c I-7.9 两窗口交空漏红 | 没打中（推的） | 交空只在镜像比被判那条根新出约一圈时出现，那时撑上限的根本来就被盖掉；抬 F 那一刻附近的镜像两窗口重合 |

## K1 清扫器

### K1-a 「账读不出就不删」让一行在无故障的合法历史上永远删不掉（打中，量过）

**历史**（`r3_attack_sweep_probe.rs` 的 `one_mount_with` / `k1_periodic_schedule_long_run`；S = 4 的 12 槽环、两块 128 MiB 内存稀疏盘，`common_root_ring_holes` 的参数）：mkfs → 第 0 次可写挂载写第一个文件、正常卸载 → 之后每次可写挂载交替做「覆盖写 1 次、正常卸载」与「覆盖写 1 次、不卸载（进程停在最后一次发布落盘之后）」。每次挂载开关关着、阈值取「有一行就扫」（产品阈值那一跑取 369）。没有任何读写故障，没有在飞的写被截。

每一步许可它的条款：每次可写挂载都写行（`.claude/kb/decisions/18-块里携带什么信息.md:309`「**每次可写挂载都写行**」）；正常卸载把 F 抬到现行那一版的 txg（`.claude/kb/decisions/16-发布语义.md:36`「准入不够时抬，正常卸载时抬到现行那一版的 txg」）；F 抬过之后释放代不高于 F 的槽可再分配（同一份文件第 33 行「可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」）。于是下一次挂载的写行、暖机与覆盖写复用了根环里 F 之下那几条旧根独占的分配记录树节点，那几条根还留在环里、自证得过，它们的账读不出。清扫器在这一格自取不删（`crates/singlefs-core/src/instance_table_sweep.rs:357`「|failure| NoRowDeletedThisRound::AccountOfARingRootUnreadable {」）；条款只写了跳过可读根的账（`.claude/kb/decisions/18-块里携带什么信息.md:314`「再跳过根环里任一可读根的账里仍分配的槽」），账读不出怎么办没写。

量到的（`results/long-60.log`，第 60 次挂载那一行与汇总行原样）：

```
LONG mount=59 inst=60 rows_after_sweep=59 sweep=Swept(NoRowDeleted(AccountOfARingRootUnreadable { root: RollbackTarget { instance: InstanceGeneration(57), checkpoint_txg: CheckpointTxg(257) }, failure: UnitUnreadable { slot: SlotNumber(1182) } })) red=[]
LONG-SUMMARY schedule=1U,1C mounts=60 threshold=0 last_rows=59 red_total=0 counts={"NotSweptAtOrBelowTheThreshold ( rows: 0, threshold: InstanceTableRowCountThreshold(0) }": 1, "Swept(NoRowDeleted(ARootRingSlotIsUnreadableOrAllZero ": 1, "Swept(NoRowDeleted(AccountOfARingRootUnreadable ": 56, "Swept(NoRowDeleted(NoCandidateRow))": 2}
```

产品阈值 369、420 次挂载（`results/long-420-t369.log`，最后一次挂载那一行原样；汇总行 36293 字节，分布用命令数）：

```
LONG mount=419 inst=420 rows_after_sweep=419 sweep=Swept(NoRowDeleted(AccountOfARingRootUnreadable { root: RollbackTarget { instance: InstanceGeneration(417), checkpoint_txg: CheckpointTxg(1877) }, failure: UnitUnreadable { slot: SlotNumber(1184) } }) red=[]
$ grep '^LONG-SUMMARY' long-420-t369.log | grep -oE '"Swept[^"]*": [0-9]+|"NotSwept[^"]*": [0-9]+' | sed -E 's/rows: [0-9]+/rows: n/' | awk -F': ' '{a[$1]+=$NF} END{for(k in a) print a[k], k}' | sort
370 "NotSweptAtOrBelowTheThreshold ( rows
50 "Swept(NoRowDeleted(AccountOfARingRootUnreadable "
```

越过阈值之后的 50 次清扫 50 次停在同一个成员，实例表涨到 419 行、长出第二片（一片装 369 行，`INSTANCE_TABLE_PAGE_RECORDS - 1`）。池级 checker 在抽查的每张盘面上 0 违例（`red_total=0`）。

**用户动作放开扫过**：每次挂载的覆盖写次数 k ∈ 0..=5 与「卸载 / 不卸载」两样都是用户决定的。① 周期 1 与周期 2 的全部 78 种调度各跑 40 次挂载（`results/scan-40.log`）：77 种删得到行，只有 `[(1, true), (1, false)]` 一种一行都没删（「SCAN schedule=[(1, true), (1, false)] mounts=40 deleted=0 blocked_by_account=36 longest_blocked_streak=36 no_candidate=2 other=2 last_rows=39 red_at_end=0」）。② 深搜：每一步在 12 种选择里挑一种让下一次挂载的清扫不删行（`k1_adversarial_schedule_keeps_every_sweep_from_deleting`），从 mkfs 起与从 6 次「覆盖写 1 次、卸载」的前缀起都找得到 12 步长的路，找到的都是同一个交替（`results/adv-w0.log`「ADV warm=0 goal=12 found=true budget_left=2322 path=[(0, true), (1, true), (1, false), (1, true), (1, false), (1, true), (1, false), (1, true), (1, false), (1, true), (1, false), (1, true)]」）；从 2、3、5、7 次前缀起 3000 次预算内找不到、从 8 次前缀起又找得到（这几份输出只在草稿目录 `/tmp/claude-1000/root-ring-r3-attack/logs/adv-w*.log`，没进模型目录；复跑 `k1-adversarial` 段改 `PROBE_WARM`）。所以这是一条窄而可达的调度：用户每次挂载写一点、隔一次正常卸载一次，就够了；不是装置把用户动作写死造出来的。

**跑前条款的四句**：
1. 分不分辨臂：分辨。清扫器今天的读法（账读不出就不删）在这段历史上一行不删；改法 A2（账读不出而那条根的 txg 低于现行那一版的 F，就跳过它的账）在同一段历史上每次都删（下一段）。
2. 系统当时看不看得到判别它的东西：看得到。账读不出的那条根的 txg 与现行根带的 F 都在清扫手里（`current_root.rollback_floor`），回收门槛已许可它独占的槽被复用这件事由 D16 已定项 1 的谓词定，不是猜。
3. 满足判据字面的哪一句：K1 格「清扫器自取的两格（根环里一条可读根的账读不出就不删……）会不会在一段合法历史上让一行永远删不掉」的前一格。条款为删不掉认下的代价只有「一个根槽持续读不出」（`.claude/kb/decisions/18-块里携带什么信息.md:314`「代价是一个根槽持续读不出时行永远删不掉，C335（根槽持续读不出时实例表只增不减）」），这段历史上没有任何槽读不出，不在 C335 的射程里。
4. 跑前条款给的改法：背景材料第五节只写「打中 ⇒ 改法进下一批」，没有点名改法。

**改法 A2（只在我的模型上量过、被攻过零轮）**：副本 `instance_table_sweep.rs` 在 `R3_ATTACK_FIX_A=1` 时，账读不出而 `root.checkpoint_txg < current_root.rollback_floor` 的根跳过它的账，其余照旧停。同一段 60 次挂载（`results/fixA-long.log`，最后一次挂载那一行原样）：

```
LONG mount=59 inst=60 rows_after_sweep=2 sweep=RowsDeleted(rows=[57],heads=0) red=[]
```

抽查的 13 张盘面池级 checker 0 违例，行数稳在 2。A2 只量了这一段历史，没跑带孤儿的历史，也没跑 K1-b 那一类读故障；它用现行根带的 F，不用系统配置里的 F（F_生效可以更高，用小的那个只会少跳）。对「那条根低于 F、账读不出、它的账里还罩着一个按候选行判未发布的单元」这一形是不是安全，是推的：那条根不在回退候选集里（候选要 txg ≥ F_生效），它独占的槽已被许可复用。另一个候选（账读不出就把整条根按「它的账罩住全池」处置）等于今天的读法，碰不到这一格。

| 改法 | K1-a 那一格 |
|---|---|
| 今天（账读不出就不删） | 中（量过：60 次挂载 59 行，阈值 369 下 50 / 50 次停） |
| A2（低于 F 的根账读不出就跳过） | 不中（量过：60 次挂载行数 2，13 张盘面 0 红） |
| 只按一圈窗口跳过（窗口外的根账读不出就跳过） | 中（推的：这段历史上账读不出的那几条根都在 12 槽的窗口里，挡它们的是 F，不是窗口） |

### K1-b 抹之前那次重读读失败：头不抹、行照删（打中，量过）

**历史**：用例文件 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 的 `pool_with_a_dead_instance(None)`（实例 1 一次 16 个数据单元的顺序写崩在第一条记录写之前，留下孤儿；实例 2 覆盖写到 txg 19，实例 1 的根全部轮出环），接着实例 3 可写挂载、末尾清扫有一行就扫。唯一的故障：第一个孤儿那一槽的读，第 1 次（判的那一次）照读、从第 n 次起报设备错或读回全零（用例共用的 `FailingReadsOfARange::FromTheNthOnward(n)`）。n = 2 正落在 `erase_the_heads_and_publish_the_sweep` 抹之前那次重读上（n = 3 那一形里这一槽只被读两次，读故障落空，是对照）。

许可它的代码：重读读失败与「读出来已经不是那个单元」走同一臂（`crates/singlefs-core/src/instance_table_sweep.rs:545`「.and_then(unit_head_read_by_sweep);」把读失败的 `None` 与认不出头并成 `None`，第 552 行「if !still_the_unpublished_unit_judged {」之后 `continue`），之后照发清扫发布、照删行。条款要的是这一轮里一次读失败都没有（`.claude/kb/decisions/18-块里携带什么信息.md:314`「且其中没有该实例的未发布单元、那次清扫里没有任何一次读失败（读不到不等于不存在，扇区维）」），判的那一半对读失败是这样做的（同一份代码第 399 行「return Err(NoRowDeletedThisRound::AReadOfAPlacementFailed { placement });」），抹的那一半没有。

量到的（`results/reread.log`，原样）：

```
REREAD read_back=DeviceError nth=2 rows_after=[InstanceGeneration(2)] first_orphan_head_still_there=true orphans_left=1 of 36 outcome=Swept(RowsDeleted { rows_deleted: [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(5), applied_transaction_high_water: 0 }], heads_erased: [PlacementOfAUnitHead { device: DeviceIdentity(0), slot: SlotNumber(1
REREAD read_back=Zeros nth=2 rows_after=[InstanceGeneration(2)] first_orphan_head_still_there=true orphans_left=1 of 36 outcome=Swept(RowsDeleted { rows_deleted: [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(5), applied_transaction_high_water: 0 }], heads_erased: [PlacementOfAUnitHead { device: DeviceIdentity(0), slot: SlotNumber(1
```

实例 1 那一行删了，第一个孤儿的头还在，`orphans_left=1 of 36` 是按删前那一行 (1, 5, 0) 判未发布、写序实例 1 的头还剩几个（与用例里 F5 观察者同一个走法）。行一删，这个孤儿按「无行 ⇒ 已发布」成了已发布的单元。n = 3 两形 `orphans_left=0 of 36`。

**层 0**（`r3-attack-enum`，建池与 F5 观察者照 checker 档 `crash_points_of_the_instance_table_sweep.rs` 另写一份，孤儿 2 个数据单元那一版；读故障包在录制器外面，只在真跑那一次读上生效，崩溃态恢复时不在）：用 `enumerate_layer0_in_state_slices` 加 `full_expansion`（每一段都展开，与 `enumerate_layer0_versions` 同一个枚举域，多一个逐态观察者），每个态跑两种 journal 政策的恢复、多版本 oracle、池级 checker，再照 F5 数。跑之前用 `closed_form_state_count` 算过：41007 与 41008，都不到 10⁶。原样：

```
ENUM-TALLY nth=2 states=41032 violations=0 failed_states=0 checker_violated_states=0 checker_first={} deletion_effective_states=12 f5_left_states=12 f5_left_max=1 i928_not_applicable_or_partial=None seconds=325
ENUM-TALLY nth=0 states=41033 violations=0 failed_states=0 checker_violated_states=0 checker_first={} deletion_effective_states=12 f5_left_states=0 f5_left_max=0 i928_not_applicable_or_partial=None seconds=223
```

删行生效的 12 个态里，带读故障那一条 12 个都剩 1 个孤儿，对照 0 个；两条流上恢复、oracle、池级 checker 都是 0 违例。也就是说这一形只有看得见删前那一行的 F5 观察者判得出，checker 档那份用例的流里没有读故障，碰不到它。

**跑前条款的四句**：
1. 分不分辨臂：分辨。改法 B（重读读失败就这一轮不删）在设备报错那一形上不删行、36 个孤儿都在（`results/reread-fixB.log`「REREAD read_back=DeviceError nth=2 rows_after=[InstanceGeneration(1), InstanceGeneration(2)] first_orphan_head_still_there=true orphans_left=36 of 36 outcome=Swept(NoRowDeleted(AReadOfAPlacementFailed { placement: PlacementOfAUnitHead { device: DeviceIdentity(0), slot: SlotNumber(1114) } }))」）。
2. 系统看不看得到：设备报错那一形看得到（`PoolReader::read` 交空）。读回全零那一形重读本身看不出与「真被抹了 / 被覆盖了」的区别，判别子在单次读里观测不到；但挂载末尾那一支判与抹之间没有别的写，模块自己的注释也说了「挂载末尾那一支两步之间没有别的写，重读只多读一次」（`crates/singlefs-core/src/instance_table_sweep.rs:506`），系统知道这一点。
3. 满足的分句：D18 已定项 11「回收」一条「那次清扫里没有任何一次读失败」（设备报错那一形，字面）；读回全零那一形落在「其中没有该实例的未发布单元」——盘上还有，清扫没看见。
4. 改法：背景材料第五节没点名改法。

**改法（只在我的模型上量过、被攻过零轮）**：

| 改法 | 设备报错那一形 | 读回全零那一形 |
|---|---|---|
| 今天 | 中（量过，剩 1 个） | 中（量过，剩 1 个） |
| B：重读读失败这一轮不删（副本 `R3_ATTACK_FIX_B=1`） | 不中（量过，36 个都在、行没删） | 中（量过：`results/reread-fixB.log` 里 Zeros nth=2 那一行仍是 `orphans_left=1 of 36`） |
| C：挂载末尾那一支不重读、照判的那一刻读出的头直接抹；后台那一支重读出的不是那个单元时，这个进程的分配器说那一槽空着就当读异常、这一轮不删 | 不中（推的） | 不中（推的：挂载末尾不再读第二次；后台那一支多一道分配器核） |

### K1-c 「池里有写序实例大于挂载根的单元就不删」（没打中，推的）

判法在 `crates/singlefs-core/src/instance_table_sweep.rs:410`「if head.writer_instance > current_root.instance {」。要它在合法历史上挡住一行，盘上要有一个本池单元的写序实例 m 大于这次挂载的实例。实例 m 写单元之前，m 已经写进这次挂载独占打开那个集合里每一份系统配置，取号失败的回卷也在单元之前（`.claude/kb/decisions/18-块里携带什么信息.md:317`「回卷写发生在任何单元之前」）；之后任何一次可写挂载取 max + 1，比 m 大。两块盘的池上独占集合就是全部盘，造不出。三块盘以上「过半集合」不全时的那一形 mkfs 今天不收，没造。被拒那一挂载的半截写（写行那次发布崩在根槽之前）留下的单元写序实例是 m，下一次挂载取到 m + 1，也不中。

### K1-d 认领规则按整单元跳把孤儿跳过（没打中，推的）

漏回收要的盘面：槽 s 留着一个已释放的码 1 或码 3 单元的旧头（跨 s、s + 1），槽 s + 1 住着候选实例的一个码 2 孤儿。清扫在 s 认出旧头就跳过 s + 1，那个孤儿不抹；而 F5 观察者用的是同一个走法（`crash_points_of_the_instance_table_sweep.rs` 的 `unpublished_units_of_the_row_left_on`，`claimed_until` 两槽），也看不见。造这个盘面要分配器在 s 空着时把 s + 1 单独发给一个节点：用户数据只取两槽都空的偶数起点（`crates/singlefs-core/src/allocator.rs:734`「/// 用户数据落点：起点槽号最小、偶数槽（32768 对齐）、两槽都空」），节点的开放段要整段 64 槽全空才开（第 766 行「self.used_per_segment[*segment] == 0」），回落从单元区起点找最低的空槽（第 778 行起的 `lowest_commit_generated_fallback_slot`）。一个两槽单元的两槽同时释放、同时回收，s 空着时节点先落 s、盖掉旧头。我没造出 s 被挡而 s + 1 空着的状态，所以记「没打中」；它与 F5 观察者共用前提这件事是真的，分配器换了取法就会露出来（推的）。

### K1-e 崩在抹头与清扫发布之间（一段历史上没打中，量过）

K1-b 的对照流（不注入读故障，孤儿 2 个数据单元）全量 41033 个态：恢复都做成、oracle 与池级 checker 0 违例、删行生效的 12 个态 F5 剩 0（上一节 `ENUM-TALLY nth=0` 那一行）。这与 checker 档 `crash_points_of_the_instance_table_sweep.rs` 判的是同一段历史，我没有另造带码 2 / 码 3 孤儿的历史去跑层 0。清扫两段上的崩溃态再挂载那一判（同一份用例 `every_crash_state_inside_the_two_sweep_phases_remounts_writable_and_ends_without_the_deleted_row`）我没另跑：K1-a 已经说明再挂载那次清扫可以被 `AccountOfARingRootUnreadable` 一直挡着，那一份用例「挂载之后实例 1 那一行要么已删、要么这一次挂载又删了」的期望在它自己的历史上成立，换成 K1-a 那种调度就不一定（推的，没跑）。

## K2 RS1–RS3 与 R2

### K2-a Q5「只并读不出的槽」：读得出而自证不过的槽让 F 越过盘上的非空有效根（打中，量过）

**历史**：用例文件 `crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 的盘面原样（实例 1 覆盖写到 17；三次可写挂载崩在写行根槽之前留下洞 18、19、20；再一次挂载写下 X = 21、空发布两次；第四次可写挂载推空发布到锚 29，准入抬 F 抬到上限）。我只改了 X 那一槽的读故障怎么交回：用例那一形是「设备报错」，我加「报成功、字节全 0」（用例共用的 `UnreadableRangeReadBack::Zeros`），从第四次挂载起每次读都这样。盘上 X 那一槽的字节一直完好。

许可它的一步：根环表只记设备报读错的槽（`crates/singlefs-core/src/allocator.rs:950`「挂载那一刻设备报读错的槽，落在挂载时的一圈窗口里」），条款同（`.claude/kb/decisions/16-发布语义.md:37`「窗口里挂载那一刻读不出、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它（用户 2026-10-04 定，零轮」），用户 2026-10-04 定 Q5 只并读不出的槽。读回全零的那一槽在挂载那一刻是「读得出而自证不过」，不进根环表，抬 F 上限按读得出的根算。

量到的（`results/rs3z.log`，原样两行）：

```
RS3Z device-error-every x_recorded_unreadable=Some(CheckpointTxg(21)) ceiling=CheckpointTxg(21) floor_after=CheckpointTxg(21) x_on_disk=true i79=["Holds"] violated=[]
RS3Z zeros-every x_recorded_unreadable=None ceiling=CheckpointTxg(22) floor_after=CheckpointTxg(22) x_on_disk=true i79=["Violated(\"实例 6 txg 30 那条根把回退下界 F 从 0 抬到 22，高于它之前的根算出的抬 F 上限 21：每块盘上最新的有效根 {0: 29, 1: 28}，非空有效根 [21]，空不空判不了的有效根 []，最旧有效根 txg 21\")"] violated=["I-7.9:实例 6 txg 30 那条根把回退下界 F 从 0 抬到 22，高于它之前的根算出的抬 F 上限 21：每块盘上最新的有效根 {0: 29, 1: 28}，非空有效根 [21]，空不空判不了的有效根 []，最旧有效根 txg 21"]
```

另三形（读回全零只坏第一次、设备报错只坏第一次、不坏）上限都是 21、I-7.9 成立（同一份输出的另三行）：挂载自己的那一次重读把它们读回来了。全零这一形池级 checker 只红 I-7.9 一条。这一盘面上 22–29 都是空发布，X 那个状态在 22 上还在，所以量到的是 checker 在合法历史上判红，不是用户状态丢了；F 越过一个后面没有同状态空发布的非空根是不是同样可达，是推的，没造。

**跑前条款的四句**：
1. 分不分辨臂：分辨 Q5 的两个候选（`research/prompts/m3-candidate-hole-r1-forks.md:13` 那一行：「只并挂载那一刻读不出的槽；读不出与自证不过的都并」）。今天这一臂红（量过）；「都并」那一臂把 X 记进根环表、上限停在 21（推的：没在副本上实现，它与设备报错那一形走同一个根环表成员，那一形量过是 21、0 红）。
2. 系统看不看得到：池级 checker 看不到（盘上 X 完好，读故障不留在盘上，`.claude/kb/invariants.md:63`「这些槽 checker 看不见，checker 只按镜像上读得出的根判（推的：读故障不留在盘上）」），与 RS3 原来那一形同一个「判别子观测不到」，所以只能在实现这一侧改。实现在挂载那一刻看得到「读得出而自证不过」，环转过一圈之后窗口里每一槽都被窗口里的发布写过（洞那一槽留着的是上一圈自证得过的根），所以窗口里自证不过的槽只会来自读故障或盘上坏字节（推的）。
3. 满足的分句：Q5 自己那一行的翻面观测（同一份岔路单第 13 行「某一类槽住着一条有效根，F 越过了它」），也是 K2 格「F 越过一条真住着非空有效根的槽」。
4. 改法：岔路单给的另一个候选「读不出与自证不过的都并」在这一格上不中（推的）；今天这一臂中（量过）。「都并」的代价没量：第一圈还没写满时窗口里从没写过的全零槽也会被并进上限，F 会被一个从没住过根的槽挡到这个进程把它写掉为止（推的，一次挂载之内至多一圈）。

### K2-b `mount_transparency.rs` 照 R2 判的几支与 core（没打中，读代码对照）

逐槽读法：harness 两槽偏移是 `slot_index * slot_spacing_in_bytes`（`crates/singlefs-harness/src/mount_transparency.rs:1340`「for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {」），core 是 `[0, slot_spacing_in_bytes]`（`crates/singlefs-core/src/recovery.rs:3296`「for offset in [0, slot_spacing_in_bytes] {」）；两边都是读交空计「报读错」、读得出的按 `SystemConfiguration::parse_slot` 加 fsid 相同算自证。判不出的谓词两边同一个合取（harness 第 1371 行「some_slot_reported_a_read_error && !some_device_self_verified_both_slots」，core 第 3330 行「.any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)」前面那个 `any(... > 0)`）。「某块盘一份自证过的都没有」core 在读阶段判完之后另报成员、按判据为真，harness 并进同一个「按真」。崩溃镜像是静的，core 的「重读一次」在镜像上读到的与第一次相同，harness 只读一次不构成口径差。没找到两边判法不同的盘面，没跑模型。

### K2-c RS1、RS2

没攻。

## K3 I-7.9 两窗口、I-9.28 与「部分没判」

### K3-a 「部分没判」并进「不适用」之后有没有把该判的状态算成判过（没打中；线索一条）

层 0 的计数把部分没判与不适用记进同一格、不进评估过那一格（`crates/singlefs-checker-tier/src/crash.rs:1493`「InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {」），层 0 两份用例钉的是「评估过 + 不适用 = 状态数」与「至少在一个状态上真被评估过」，部分没判落在不适用那一边只会让「评估过」变少，不会变多。崩溃注入单列一格（`crates/singlefs-checker-tier/src/crash_injection.rs:512` 那一句注释「判成「部分没判」（`InvariantVerdict::PartiallyJudged`）的次数：不红，也不算判绿。」）。我在全仓 `InvariantVerdict::PartiallyJudged` 的每处穷举 match 里没找到把它算成违例的臂；算成「判过」的只有一处：坏盘输入的「checker 一条都没判」把部分没判归进判了那一边（`crates/singlefs-checker-tier/src/bad_disk_input.rs:1320`「let judged_nothing = report.iter().all(|(_, verdict)| match verdict {」，第 1324 行「| InvariantVerdict::PartiallyJudged(_) => false,」）。一张坏盘镜像上别的条都不适用、只有 I-9.28 报部分没判时，它不会被记成「checker 一条都没判」；这与文件头「它不是红，也不是成立」的口径不一致。没造出这样的镜像（推的：I-9.28 报部分没判要有读得出、自证过的根，那时 I-7.x 的根环几条多半已判成立），记线索。读的范围：冻结副本上 `grep -rn 'PartiallyJudged' --include=*.rs crates | grep -v '^crates/singlefs-checker/src/walk.rs' | wc -l` 是 101 行，我逐处读了前 40 行列出的消费者，其余 61 行没读。

### K3-b I-9.28「树表读不出」那一格在合法历史上是常态（线索，量过）

K1-a 那段无故障调度跑 30 次挂载、每次挂载之后都跑池级 checker（`results/long-partial.log`）：30 张盘面里 15 张 I-9.28 报部分没判，15 张理由都是「树表读不出」，读不出的都是 F 之下、树表单元已按回收门槛被复用的旧根（原样一行）：

```
LONG mount=1 inst=2 rows_after_sweep=1 sweep=Swept(NoRowDeleted(ARootRingSlotIsUnreadableOrAllZero { ring_slot: RootRingSlot { region: 0, slot: 3 } })) red=[] partial=["I-9.28:比到的 11 棵树诞生 txg 都对得上；这几条读得出、自证过的根树表指针非零而条目没读全，没进比较：盘 0 上的根（实例 0、txg 0）：树表读不出；盘 1 上的根（实例 1、txg 1）：树表读不出；盘 0 上的根（实例 1、txg 2）：树表读不出"]
```

判定字面照 `.claude/kb/invariants.md:315`「各处都对得上，但有一条读得出、自证过的根树表指针非零而树表读不出、或条目比字段表短 → 部分没判」，不算判错。它说明的是边界：「树表读不出」不分「回收门槛已许可复用」与「本不该读不出」，于是只要有过一次正常卸载，I-9.28 在随后约一圈的盘面上就只报部分没判；层 0 里这些态计进不适用。I-9.28 的依据（C342 重发的树 ID）恰好落在被复用那几条旧根上时，这一条看不见它（推的）。改法候选：低于 F_生效、或在一圈窗口外的根，树表读不出不进「没判到」那张单子（推的，没实现；K1-a 的改法 A2 是同一个读数）。

### K3-c I-7.9 两窗口交空时坏镜像漏红（没打中，推的）

交为空要镜像的最新有效根 N 与被判那条抬 F 的根 r 满足 N − 3S ≥ r − 1，即镜像比 r 新出将近一圈。那时 r 之前的根在环里本来就被盖掉了，只锚在 r 上的窗口算出来的上限同样没有输入，两窗口交空与否不改结论。r 落盘那一刻附近的镜像上 N = r，两个窗口重合，与只取一个窗口时同一个判法；层 0 枚举的正是这些态。我没造坏镜像去量，K2-a 那一形（F 抬过头的合法镜像）上 I-7.9 在 N = 30、r = 30 时照红，说明交不空时判得到。

## 没打中的形状

| 形状 | 取样 | 结局 |
|---|---|---|
| 周期 1 / 2 的用户调度（每次挂载 k ∈ 0..=5 次覆盖写 × 卸载 / 不卸载），40 次挂载 | 78 种 | 77 种删得到行，池级 checker 0 红（`results/scan-40.log`） |
| 同一组参数的定长调度，16 次挂载（`k1_unmount_cycles`：k ∈ 0..=4 × 全卸载 / 全不卸载 / 交替） | 15 段、240 次挂载 | 每段都删过行；`AccountOfARingRootUnreadable` 10 次，都在下一次挂载删掉（`results/p1-16.log`） |
| 清扫器第二格「写序实例大于挂载根」 | 读取号与回卷条款、两盘池 | 造不出（K1-c，推的） |
| 认领规则跳过孤儿 | 读分配器三条取法 | 造不出（K1-d，推的） |
| 抹头与清扫发布之间崩溃 | 一段历史全量 41033 态 | 0 红（K1-e，量过） |
| `mount_transparency.rs` 的 R2 | 两份判法逐句对照 | 一致（K2-b） |
| 部分没判在穷举 match 里被算成成立或违例 | `grep -rn PartiallyJudged --include=*.rs crates` 除 walk.rs 之外命中的前 40 行（命中数见 K3-a 末句） | 没有；坏盘输入那一处算成「判了」（K3-a 线索） |

## 这条腿自己的限度

- 几何只取了 `common_root_ring_holes` 那一组（S = 4 的 12 槽环、两块 128 MiB 盘、1 MiB journal 环）；K1-a 只在一种调度上打中，别的 S 与盘数下那条调度还中不中没扫。
- K1-a、K2-a 的历史没有崩溃态枚举：K1-a 的结论是「存在一段合法历史上行永远不删」，它的每一步都是落盘完成的状态，我没有对那 420 次挂载的写流跑层 0；K2-a 同样只看了抬完之后那一张盘面。层 0 只跑了 K1-b 与它的对照两条流，合计 82065 个状态（41032 + 41033），跑前闭式算的是 41007 与 41008。
- `enum-0` 那一次我在 `run.sh` 外面误套了 `timeout 230`（定义不许给批设限时）；它 223 秒正常跑完、退出码 0，输出完整，结论不受影响，记在这里。
- 改法 A2、B 只在副本上、只在各自那一段历史上量过；C 与「都并」两个改法没实现。
- 内存包装：上限按段取 1G–4G，带 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 与 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（同一会话里回收门槛那条腿也在跑）；有两次 252（排不上，命令没跑）后改小上限重跑，没有一次 250–254 的输出被当成结果。开跑前 `ps` 看到主 agent 的 `cargo test --release -p singlefs-checker-tier`（32G）在跑，没有性能测量在跑。
- 只抽了一次样：「没打中」各格照规则要再抽一次才算。

## 没做什么

- K2-c（RS1 三种结局、RS2 按哪张表）没攻；K3 没造坏镜像、没攻 I-7.8 认领。
- 没跑任何 checker 档测试二进制与门禁阶段；`r3-attack-enum` 是副本里新建的非 checker 档包，链接 checker 档库、调它的枚举函数。
- 没改主工作区任何文件（只写了这份报告与模型目录）。草稿目录 `/tmp/claude-1000/root-ring-r3-attack/` 里的仓副本 `tree/`（含 `target/` 编译目录）交回前删掉。

## 要主 agent 跑的命令

没有非跑不可的；核 K1-a、K1-b、K2-a 三处打中照「复跑」一节那一条命令跑全部段，判读行见各节引的原样输出。
