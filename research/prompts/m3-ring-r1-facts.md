# 里程碑三二·4（环与根的条款空白）实现侧事实表

写于 2026-09-30。批次 `records/2026-09-30-里程碑三开工第一轮.md#1`。只读，没改仓内文件，没跑 cargo。
行号都是这一次 `grep -n` / `sed -n` / `awk 'NR==…'` 现取的；标「推的」的是从代码读出来的推论，没跑过。
树：`fcc4d340` 之上（工作区只多一份未跟踪的 records 文件）。

## 最要紧的三条

1. 根槽位置是 txg 的纯函数（`crates/singlefs-core/src/root_ring.rs:107-117`：区域 `txg mod 3`、槽 `(txg div 3) mod S`），没有「下一槽怎么选」的状态。所以「洞」= 某个 txg 没有根，它那一槽（`target_for_publish(h)`）要么从没写过（前洞），要么留着上一圈 `h − 3S` 的根（后洞；E156 PQ1 实测 `hole_txg=16 … hole_slot_content=root_txg_4`）。后洞槽里那条旧根按实例表判仍然有效，于是 `oldest_valid_root()`（`allocator.rs:998-1006`）被拉低，回收门槛 `max(F_生效, 环里最旧有效根)`（`mount.rs:1154-1159`）跟着低。
2. 在 slot(h) 补一条 txg h 的根，现有 `CommitStep::WriteRootRecordForceUnitAccess` 就写得到（它按根记录的 txg 算槽，`transaction.rs:293-318`）；拦它的是分配器根环表：`record_root_written_by_this_process` 断言 txg 逐个加一（`allocator.rs:1641-1647`），挂载时表的起点又设成 `first_txg − 1`（`mount.rs:1586-1599`）。读侧不按 txg 反推槽位：core 全扫每一槽（`recovery.rs:1097-1104`），checker 源码里查不到槽位公式（见 (a) 表末行命令）。
3. 8 条 `crash-case:` 用例的起点状态都没有洞（静态看）。唯一一条流中途造出洞（txg 5）的是「预置残留记录」那条（`crash_enumeration_fixed_script_stream.rs:1177`），它不在 `crash-case:` 行里；崩溃注入快档的二次崩溃起点可能带洞，但没有断言钉住。

## (a) 根环：槽数、根槽写路径、落点、留洞的情形

| 问 | 事实 | 出处 |
|---|---|---|
| 区域数 R | 格式常量 3 | `crates/singlefs-format/src/lib.rs:238` `pub const ROOT_RING_REGIONS: u64 = 3;` |
| 每区槽数 S 从哪来 | 系统配置槽偏移 362 那一字节；mkfs 写 8，读时判闭区间 [4, 16]，越界拒挂载 | 写：`crates/singlefs-core/src/system_configuration.rs:46`（`ROOT_RING_SLOTS_PER_REGION_OFFSET: u64 = 362`）、`:436` 定位；读：`:506-509` 经 `RootRingSlotsPerRegion::from_system_configuration_field`（`root_ring.rs:62-77`）；常量 `lib.rs:247`（下界 4）、`:252`（上界 16，C506 占位）、`:256`（mkfs 值 8） |
| 环长 | 3S（mkfs 默认 24） | `root_ring.rs:113-116` |
| 下一槽怎么选 | 不选：`target_for_publish(txg, S)` 纯算，区域 `txg % 3`、槽 `(txg / 3) % S` | `root_ring.rs:107-117` |
| 区域在哪块盘 | 区域 0/1/2 归盘 0/1/0 | `lib.rs:265` `ROOT_RING_REGION_DEVICES: [u32; 3] = [0, 1, 0];`；写时查表 `transaction.rs:301-302` |
| 根槽写（唯一的 CommitStep 成员） | `CommitStep::WriteRootRecordForceUnitAccess { checkpoint_txg, root_slot }`，槽由 txg 现算，FUA 写到那个区域归属的盘 | 成员定义 `transaction.rs:128-132`；执行 `transaction.rs:293-318`（`perform_commit_step` 在 `:269`） |
| 根槽写在发布里的位置 | 单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置轮换 → 屏障；三条发布路径与重发共用 | `persist_publish_writes` `transaction.rs:1052-1078`；`persist_the_root_then_rotate_the_system_configuration` `transaction.rs:1206-1222` |
| 走它的发布路径 | 带单元的（`publish_version` `transaction.rs:4947`，写行、暖机、抬 F、卸载、管理员回退都经它）、零单元（`publish_without_units` `:1233`）、树表 0 条上写行（`publish_instance_table_on_version_without_file` `:1395`）、重发冻结的那次（`resend_the_frozen_publish`，`:1178` 调 `persist_publish_writes`） | 各行号为 `grep -n` 现取 |
| mkfs 的根 | 第 0 代根 FUA 写进三个区域的槽 0（不经 CommitStep） | `make_filesystem.rs:565`（`WriteDurability::ForceUnitAccess`）、`:608-611` `genesis_root_ring_slots` |
| 新实例第一个 txg | `max(环里自证过的根的 txg, 环里全部自证过的记录的 checkpoint_txg) + 1` | `mount.rs:1092-1114`，式子在 `:1114` `checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))` |
| 同一进程内的 txg | 每次发布取现行版 + 1；回退也是 + 1 | 暖机 `mount.rs:2642-2646`；回退 `mount.rs:5080-5082` |
| 洞①：记录已持久、根槽没持久的崩溃 | 恢复施加那条记录（所选版本 txg h），下次可写挂载第一个 txg = h + 1（`:1114` 里 records 那一半），h 永远没有根 | `mount.rs:1114`；实测 E156 PQ1（下面命令） |
| 洞②：记录自证得过、但施加前验证失败 | 恢复停在 h − 1，但 `first_txg_of_new_instance` 取的是「全部自证过的记录」的 max，h 仍被跳过（推的，没造过可写挂载这一格） | `mount.rs:1101-1105`；撕裂注入那一格只做到恢复：`crash_enumeration_fixed_script_stream.rs:1502` |
| 洞③：写失败后冻结、没重发就退出 | 发布冻结、重发用同一 txg（`transaction.rs:1080-1089` 文档、`:1178`），重发成了不留洞；进程在重发之前退出且记录已持久，就回到洞①（推的） | 同左 |
| 洞④：根槽读不出或自证不过（读侧的洞） | 读根环时跳过，不进 `RootRingOccupancy`、不进 `highest_root_txg`；对回收门槛与 I-3.9 等同没有这条根 | `recovery.rs:1073-1094`（`Unreadable => continue`、`NotSelfVerified => {}`）；`allocator.rs:941-942` 文档 |
| 不是洞但同样让有效根序列断开：崩溃恢复抛弃的根 | 按最新根的实例表判被抛弃的根占着槽、但不算有效根，不进 `oldest_valid_root` | `mount.rs:1161-1163` `abandoned_by_table`；`allocator.rs:998-1006`；`mount.rs:1539-1543` |
| 管理员回退 | 向前发一次 current + 1，不跳 txg，不留洞 | `mount.rs:5080-5082` |
| 读侧与槽位公式 | core 读根环按「全部槽」扫，不按 txg 反推；checker 源码里没有槽位公式 | `recovery.rs:1097-1104` `every_root_ring_slot`；命令见下 |

```
$ grep 'name=pq1 ' research/results/e156-alloc-basis-counts-2026-09-26-r2.out | cut -c1-400
E7RESULT name=pq1 position=front hole_txg=5 prefix_applied=1 first_txg_of_new_instance=6 ring_has_root_at_hole_txg=false hole_slot_content=never_written a16_applicable=true
E7RESULT name=pq1 position=back hole_txg=16 prefix_applied=1 first_txg_of_new_instance=17 ring_has_root_at_hole_txg=false hole_slot_content=root_txg_4 a16_applicable=true
$ grep -rn 'checkpoint_txg % \|checkpoint_txg\.0 % \|% ROOT_RING_REGIONS\|target_for_publish' crates/singlefs-checker/src/; echo "exit=$?"
exit=1
```

## (b) C566：多扣的槽在哪算、式子、洞怎么进式子、有没有用例

| 问 | 事实 | 出处 |
|---|---|---|
| 扣槽的式子（core，一处定义） | 回收门槛 = `oldest_valid_root.map_or(effective_floor, \|oldest\| effective_floor.max(oldest))`，即 max(F_生效, 环里最旧有效根)；已释放且释放代 ≤ 门槛的记录才回收 | 式子 `mount.rs:1158`（函数 `:1154-1159`）；回收谓词 `allocator.rs:1567` `.filter(\|record\| record.is_released && record.generation <= floor)` |
| 挂载时算一次 | 读根环，去掉按最新根实例表判被抛弃的，取最小 txg，按门槛回收 | `mount.rs:1539-1547` |
| 这个进程每写一条根算一次 | 记根、盖掉旧占用者、按同一门槛回收 | `allocator.rs:1637`（函数）、`:1649-1652`（按 txg 算槽、插表）、`:1654-1655`（门槛）、`:1668`（回收） |
| 环里最旧有效根 | 根环表里 `ValidRoot` 的最小 txg；读不出、自证不过的槽不在表里 | `allocator.rs:998-1006`；`allocator.rs:941-942` |
| 洞怎么进式子（后洞） | 洞 h 的槽 = `target_for_publish(h)`，这一槽留着上一圈 txg h − 3S 的根，它仍是有效根，于是最旧有效根停在 h − 3S，直到 txg h + 3S 盖掉那一槽；这一段里门槛比没洞时低，差的正是 h_缺 = txg − h + 1 个 txg（推的，对照 E156 锚点模型） | 模型：`e156_allocation_basis_counts.rs:1206`（崩溃那个 txg 不进环）、`:1212-1213`（门槛 = 环里最小 txg）、`:1225`（释放代 > 门槛即扣）、`:1262`（后洞 h_缺 闭式 `txg - hole + 1`）；实测 PQ1 `hole_slot_content=root_txg_4`（(a) 表下的命令） |
| 洞怎么进式子（前洞） | 前洞槽从没写过，不拉低门槛；多扣来自洞那一版没有根：那一版分出、下一版就释放的单元，寿命里没有一条根，按 G12（寿命区间）可放、按门槛谓词要扣到门槛越过释放代（推的，对照锚点模型的 `referenced` 与 `withheld` 之差） | 模型 `e156_allocation_basis_counts.rs:1214-1232` |
| 实测量 | S = 8、ρ = 1、实时点，前洞 h_缺 = 1、2、3、4 时 Δ 峰值 10、18、26、34（逐步 + 8，推的拟合 8·h_缺 + 2，只有 4 个点）；12 个取样点逐点相同 | 下面命令；报告 `research/prompts/e156-r4-runner-report.md` 第 9 行 |
| h_缺 的定义 | [环里最旧有效根, 最新根] 里环中没有自证合法根的 txg 个数 | `e156_allocation_basis_counts.rs:1589-1599` `missing_txg_count` |
| 有没有用例钉住 | `crates/` 里没有一条测试断言「多扣的槽数」；钉住的只有 E156 装置：`replay.sh` 第 192 行对产物 exact 比对，`crates/mutations.tsv` 有 22 行点着这个装置；`C566` 这个编号在 `crates/`、`research/scripts/`、`.claude/gate.d/` 里零命中 | 下面命令 |

```
$ grep -n 'name=q1c s=8 rho=1 timing=real quantity=q1c_front ' research/results/e156-alloc-basis-counts-2026-09-26-r2.out | cut -c1-400
3935:E7RESULT name=q1c s=8 rho=1 timing=real quantity=q1c_front samples=73 judgement_two_slots=above judgement_one_slot=above smallest_h_que_above_two=1 smallest_h_que_above_one=1 first_crossing_two=key1:k1:positionfront:workload_publishes1:txg7 by_h_que=1:peak10:sum314:count38:positive38,2:peak18:sum420:count24:positive24,3:peak26:sum208:count8:positive8,4:peak34:sum102:count3:positive3
$ grep -n 'E156' research/scripts/replay.sh | head -1
192:E156|@driver_e156||e156-alloc-basis-counts-2026-09-26-r2.out|exact
$ grep -c 'e156_allocation_basis_counts' crates/mutations.tsv
22
$ grep -rn 'C566\|C380\|C596\|C597' crates research/scripts .claude/gate.d | grep -v '^crates/singlefs-checker-tier/src/bin/e158'
crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1081:    /// 「前」位置第一个洞最早落的 txg（C380 逐字的 txg 5）。
```

注：E156 执行报告写的装置路径是 `crates/singlefs-harness/src/bin/…`，今天它在 `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`（`ls crates/singlefs-checker-tier/src/bin` 现查）。

## (c) 可写挂载今天那一串：次序、落点、对应的提交步骤

入口 `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`（`mount.rs:4430`），取号之后交给 `establish_instance`（`mount.rs:3688`）。「提交步骤」一列写 `transaction::CommitStep` 成员；D17 已定项 2 登记的那一套是 `singlefs_harness::segments::StepKind`（`crates/singlefs-harness/src/segments.rs:21-30`：`ZeroFill`、`UnitWrite`、`JournalRecord`、`RootRecordFua`、`SystemConfigurationSlot`、`Barrier`），括号里对上。

| # | 步 | 落点 | 写不写盘 / 提交步骤 |
|---|---|---|---|
| 1 | 盘数、重复身份 | `mount.rs:4439-4440` | 不写 |
| 2 | 择系统配置（S 在这里读进来） | `mount.rs:4441` | 不写 |
| 3 | 核参数与盘表、盘够不够到单元区起点 | `mount.rs:4445-4454` | 不写 |
| 4 | 读阶段：择根、扫 journal、重放（「恢复」在这里，只在内存里施加记录得到 `effective_root`）、N-配置 见证，见证为真重读一次 | `mount.rs:4465`（`read_stage_settled_at_most_on_the_one_reread` `:4391`、`read_stage` `:4353`） | 不写（读者类型是 `PoolReader`；「取号是这次挂载的第一个写」`mount.rs:3812`） |
| 5 | 重建上一版 | `mount.rs:4487` | 不写 |
| 6 | 下一计数器、新实例第一个 txg（洞在这里定下：`max(环, 记录) + 1`） | `mount.rs:4489-4495`（式子 `:1114`） | 不写 |
| 7 | 树 ID 水位 | `mount.rs:4496` | 不写 |
| 8 | 重建分配器：读根环、按门槛回收、影子账、装根环表（表的 `latest_recorded_txg` = first_txg − 1） | `mount.rs:4502`（函数 `:1489`；回收 `:1544-1547`；装表 `:1586-1599`） | 不写 |
| 9 | 暖机计划（纯算） | `mount.rs:3709`（`warm_up_publish_txgs` `:3054`） | 不写 |
| 10 | 要取的号、要写的行、实例表重写 | `mount.rs:3711-3723` | 不写 |
| 11 | 空间准入 | `mount.rs:3737` | 不写 |
| 12 | 整串预演（写行 + 暖机，分配器拷贝上） | `mount.rs:3767` | 不写 |
| 13 | 逐盘核「带不带所选那一版」 | `mount.rs:3815` | 不写 |
| 14 | 取号 | `mount.rs:3838` → `write_acquired_instance` `transaction.rs:827` | `CommitStep::Barrier`（`transaction.rs:838`）→ 逐盘系统配置槽写（`transaction.rs:853`，**直接调 `write_system_configuration_slot`，不经 `CommitStep` 成员**；`StepKind::SystemConfigurationSlot`）→ `CommitStep::Barrier`（`transaction.rs:868`） |
| 15 | 写行那次发布（带文件 / 零单元 / 树表 0 条只写实例表三臂，txg = first_txg） | `mount.rs:3872`、`:3892`、`:3911` | `persist_publish_writes`（`transaction.rs:1052-1078`）：`WriteUnitToEveryDevice`* → `Barrier` → `WriteJournalRecordToEveryDevice`* → `Barrier` → `WriteRootRecordForceUnitAccess` → `RotateSystemConfigurationSlots` → `Barrier`（`transaction.rs:1206-1222`）；零单元那臂根落盘后才记根环表 `mount.rs:3906` |
| 16 | 暖机空发布，逐个 txg + 1，至多 3 次 | `mount.rs:3946`（`publish_empty_after` `:2818`） | 同第 15 步那一串 |
| 17 | 准入不够时推抬 F | `mount.rs:4005` → `raise_the_floor_through` `:2348` → `write_the_raised_floor_into_every_system_configuration`（`mount.rs:2521`） | `Barrier`（`transaction.rs:564`）→ 逐盘系统配置槽写（`transaction.rs:574`，直接调）→ `Barrier`（`transaction.rs:590`）→ 空发布同第 15 步那一串 |

### 「补洞」若挂进可写挂载，候选挂点（整张表都是推的）

补的对象先说清：能补的只有「这次挂载在读阶段由记录施加出来的那一版」——`effective_root` 的 txg 大于 `chosen_root` 的 txg 时（两者在 `mount.rs:4458-4461` 同时交出），它就是那个洞 h，而上一个实例的行正好写 `(effective_root.instance, effective_root.checkpoint_txg)`（`mount.rs:4516-4520`），按有效规则「有行 (i, Ti) 且 T ≤ Ti 即有效」（`recovery.rs:1396-1404`）这条根写进去就是有效根。更早留下的洞，那一版只在当时的记录里，今天挂载手里没有它的根记录字节。

| 挂点 | 在哪两步之间 | 用得上的现成东西 | 撞上的现有约束 |
|---|---|---|---|
| P0 取号之前 | 第 13 步与第 14 步之间 | 根环表、`effective_root` 都在内存里 | 「取号是这次挂载的第一个写」（`mount.rs:3812`）；取号之前拒的各条都承诺「盘上逐字节不变」；D23 已定项 16 要求取号两写之后一道屏障、才发本实例第一个非系统配置写（`.claude/kb/layout/01-first-txn.md:401`） |
| P1 取号之后、写行之前 | 第 14 步末道屏障（`transaction.rs:868`）与第 15 步之间 | `CommitStep::WriteRootRecordForceUnitAccess` 按根记录的 txg 算槽，写 txg h 的根自然落 slot(h)（`transaction.rs:293-318`），再跟一道 `Barrier` | 多出一段 [根槽 FUA][屏障]，可写挂载那条流的段序列变了，要重登 `layout/01-first-txn.md` 八并过单测 `registered_segment_sequences_match_every_recorded_path`（D17 已定项 2 第 5 条）；分配器根环表按 txg 逐个加一断言（`allocator.rs:1641-1647`），插一条 txg h < first_txg 的根要另开一个入口，不能走 `record_root_written_by_this_process` |
| P2 并进写行那次发布 | 第 15 步里、写行那次的根槽 FUA 前后 | 同一段里多一个 `RootRecordFua`，StepKind 集合不变 | 段内两个根槽写任意子集持久，崩溃状态翻倍；`PublishWrites` 只装一条 `root_slot`（`transaction.rs:1027-1038`，`root_slot` 在 `:1034`），冻结重发要一起带上 |
| P3 暖机之后 | 第 16 步之后 | 本实例的根已覆盖每块盘 | 暖机那几次记根时门槛已经按带洞的环算过一次回收；回收是按门槛只增的，晚补只影响之后 |

补洞写会盖掉 slot(h) 上 txg h − 3S 的旧有效根（后洞）；盖掉之后最旧有效根前移，门槛随之抬（就是 C566 要的效果），但那条旧根也不再是回退候选（推的）。

## (d) C380：判法在哪，洞补上之后还剩不剩对象

| 问 | 事实 | 出处 |
|---|---|---|
| 判法 | I-3.9：最新根那棵账里每条已释放记录，取候选集里还引用它的最新一条根 L 与 L 之后第一条根 T，释放代要落在 `(L, T]` | `crates/singlefs-checker/src/walk.rs:3247-3256`（文档）、`:3257`（`judge_release_generations`）、`:3300-3306`（取 T、判区间） |
| 为什么是区间 | 文档原话：「根环里的 txg 可以有洞——一次发布的根槽从没写过、而它的 journal 记录在挂载时被施加」；残留记录那条流 txg 5 没根、释放代 5、T = 6，按「等于 T」判 12 个合法状态红 | `walk.rs:3252-3255`；不变量行 `.claude/kb/invariants.md:142` |
| 候选集怎么来 | 读得出、自证过的根（checker 自己读，全扫每一槽）里，去掉按最新根实例表判被抛弃的、txg < F_生效 的（最新根总留着） | `crates/singlefs-checker/src/image.rs:613-624` `valid_roots`；`walk.rs:5959-5977` |
| 候选集 < 2 条 | 整条报不适用 | `walk.rs:3403-3413` |
| 洞补上之后：残留记录那一格 | slot(5) 有了 txg 5 的根，它不再引用被释放的落点，T = 5，区间收成单值（推的） | 同上判法 |
| 洞补上之后仍剩的对象 ① | 补不了的旧洞：只能补本次挂载施加出来的那一版（见上一节开头），更早的洞在它的槽被下一圈盖掉之前一直在（推的） | `mount.rs:1114`（每次挂载只跳过一个 txg 区段） |
| ② | 被抛弃的根、低于 F 的根被滤出候选集，它们的 txg 在候选序列里同样是空当；L 与 T 之间隔着它们时区间照样不止一个值（推的；与洞无关） | `walk.rs:5963-5966` |
| ③ | 读不出或自证不过的根槽不进候选集（读侧的洞），区间同样放宽（推的） | `image.rs:617-621` |
| ④ | 补洞那次写本身崩在中间：slot(h) 上旧根已坏、新根没自证过，这一槽读不出，回到 ③（推的） | — |
| 小结 | 洞补上之后，I-3.9 还有对象，但对象从「根槽从没写过的 txg」换成了「被滤掉或读不出的根」；要把 I-3.9 收成单值，还得另判这两类（推的） | — |

## (e) C596：撕裂的系统配置轮换之后只剩只读挂载

| 问 | 事实 | 出处 |
|---|---|---|
| 轮换怎么写 | 每块盘：世代号 = 这块盘两槽里自证过的最大世代号 + 1，槽 = 世代号 mod 2 | `transaction.rs:361-366`（世代号）、`:409`（槽偏移 `(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing`） |
| 撕裂留下什么 | 那一槽自证不过；另一槽还是上一世代 | 用例造法：`crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs:17-37`（把世代 5 那一槽清零） |
| 拒在哪一判 | N-配置 见证读：任一块盘自证过的槽少于 2 ⇒ `SomeSystemConfigurationSlotUnreadOrUnverified` ⇒ 判据按真 ⇒ 重读一次仍真 ⇒ `NewerStateStillUnreadableAfterOneReread`，在取号之前 | 读数 `mount.rs:4304-4313`；按真 `mount.rs:404`；重读与拒 `mount.rs:4391-4428`；成员文档写明「撕裂的系统配置轮换也落在这里：代价是那之后只剩只读挂载」`mount.rs:434` |
| 取号之后又读不出 | 逐盘核与取号那一遍又读到缺槽，同一个拒因 | `mount.rs:3226-3245`（`refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`），调用处 `mount.rs:3822` 与取号的 map_err（`mount.rs:3838` 起） |
| 为什么是「永远」 | 能重写那一槽的只有取号写（`transaction.rs:853`）、发布末尾轮换（`transaction.rs:324`）、抬 F 先写（`transaction.rs:574`），都在拒点之后；只读挂载的入口只拿 `&dyn PoolReader`，不写（`mounted_read.rs:553`）。所以那一槽没有机会被重写（推的，由代码路径读出；量的见下一行） | 同左 |
| 量过没有 | 攻方在副本上量过：撕裂轮换 5 个状态连挂 6 次、6 种用户动作下每次只读，拒可写都在取号之前、盘上逐字节不变 | `research/prompts/unreadable-at-mount-r2-main-verification.md` L1 |
| 用例 | 有「一次可写挂载被拒、盘上不变、只读照常」三条：盘 0 最新槽坏、盘 1 最新槽坏、全好对照 | 同一用例文件 `:74`、`:83`、`:93`（`#[test]` 在前一行）；断言拒因 `:47-65`、盘上不变 `:66-69`、只读照常 `:70` |
| 没钉住的 | 「连挂多次仍拒」「之后永远只读」没有 `crates/` 用例；出路（管理员工具或只读下定点修复）没有代码 | `grep -rn '永远只读' crates --include=*.rs` 零命中；这份用例文件头只写「代价……见同一判决 L1」（`:5`），不断言 |
| 变异行 | `crates/mutations.tsv:1423`（C331 甲：见证读缺一槽不判不出）、`:1424`（取号那一刻缺一槽照样取号） | `grep -n 'C331' crates/mutations.tsv` |

## (f) C597：实例表

| 问 | 事实 | 出处 |
|---|---|---|
| 行的定义 | `kind 1 \| 实例代号 4 \| 所选根的 checkpoint_txg 8 \| 最大已施加事务号 W 8 \| flags 1 \| 预留 66`，共 88 字节 | `crates/singlefs-core/src/instance_table.rs:17-25`；`crates/singlefs-format/src/lib.rs:113` `INSTANCE_ROW_BYTES: u64 = 88` |
| 片与链 | 一片 370 条记录（`lib.rs:119`），末条是链指针，所以一片 369 行；根记录指第 0 片，第 k 片指第 k + 1 片；0 行也占一片 | `instance_table.rs:4-7`、`:119-125`（`instance_table_pages_for_rows`）、`:136-141` |
| 写入点 | 每次可写挂载的写行那次发布：给 [max(上一个实例, 1), 新号) 里每个实例各一行（上一个实例写 (i, T, W)，中间没写过行的实例写 (i, 0, 0)），接在原表后面，COW 重写整条链 | 列行 `mount.rs:3086-3111`；接行 `mount.rs:1014-1024`（`rows.extend_from_slice(rows_written)` 在 `:1019`）；发出去 `mount.rs:3872`、`:3911` |
| 删行 | core 里没有删行代码 | 命令见下（`exit=1`，零命中） |
| 条款里的删行条件 | 不变量 I-3.8 已写了一个回收条件（整轮清扫……根环里没有该实例发布的根、且根环每个槽都读成功），状态列写「回收那一半没有输入，第一版行只增不删，删行的条件没人判」 | `.claude/kb/invariants.md:141` |
| 容量上限 | 行数与片数本身没有上限。间接的边：① 实例代号 u32，盘上最大号已是 `u32::MAX` 时取号之前拒（`transaction.rs:695-711`；`mount.rs:3711-3717`）；② 写行那次要点名整条链，一次发布的记录条数上限 = 环字节 ÷ 4096 ÷ 3，默认环 768 MiB 时是 65536 条、每条 67 项，超了在动分配器之前拒 `JournalRecordsOfThePublishExceedTheLimit`（`transaction.rs:5383-5407`；`system_configuration.rs:88-90`；算术见下）；两万四千多行（67 片）才开始跨记录（`transaction.rs:142-143`） | 同左 |
| 空间上随行数涨的 | 实例切换预留 = (N_switch + 1) × (32768 × max(1, ⌈(rows0 + N_switch) ÷ 369⌉) + R × c_max × 16384)，每 369 行多一片；调查员实测第 367 轮跨片、每盘 104 → 112 槽 | 式子 `crates/singlefs-core/src/admission.rs:183-222`；实测 `research/prompts/closeout-recheck-2026-09-28/c335-investigator-report.md:23` |
| 满了会怎样 | 准入不够：上一版带文件的照写行，写完行与暖机之后推抬 F 再判；上一版树表 0 条的在取号之前拒 `SpaceAdmissionRefusedBeforeAcquisition`，盘上不变 | `mount.rs:3737-3760`（`WithFile` 分支 `:3747`、`WithoutFile` 分支 `:3752-3753`）；推抬 F `mount.rs:4005` |

```
$ grep -rn 'rows\.retain\|rows\.remove\|rows\.truncate\|rows\.drain\|rows\.clear' crates/singlefs-core/src/ ; echo "exit=$?"
exit=1
$ echo $((805306368/4096/3))
65536
```

## (g) 崩溃枚举登记流的起点状态有没有洞（静态）

`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` → `8`。逐条看它点名的用例怎么造起点：

| crash-case 行 | 用例 | 起点怎么造 | 起点环上有洞？ |
|---|---|---|---|
| layer0-first-stream | `crash_enumeration_new_pool_file_creation_stream.rs` | `base = pool.memory_pool_after_mkfs()`（`:459-460`） | 没有：mkfs 之后只有第 0 代根 |
| layer0-second-stream | `crash_enumeration_fixed_script_stream.rs:797`（全量，`#[ignore]`） | `prepare` 里 `base = pool.memory_pool_after_mkfs()`（`:385`）；流里的「进程重开」是正常退出后重开（`:235`），录制流本身不崩 | 没有 |
| floor-raise-pushed-by-the-session | `crash_enumeration_floor_raise_pushed_by_admission.rs` | 一个会话里连发覆盖写到准入拒，之后 `base = pool.image()`（`:110`） | 没有：之前没有崩溃 |
| c561-sigma-full | `record_checker_judges_absence_by_the_persisted_set.rs` | `build_pool`（`:191`）录历史；σ 之前整段持久（`:751`） | 没有 |
| layer0-multi-record-publish-stream | `crash_enumeration_multi_record_publish_stream.rs` | `base = pool.memory_pool_after_mkfs()`（`:180`） | 没有 |
| layer0-tree-split-streams | `crash_enumeration_tree_split_streams.rs` | `base: pool.memory_pool_before_the_current_version()`（`:300`），不崩的前缀 | 没有 |
| layer0-position-addressed-tree-streams | `crash_enumeration_position_addressed_trees.rs` | `make_filesystem` 后建 base（`:303`、`:313-314`） | 没有 |
| crash-injection-fast-tier | `crash_injection_campaign.rs:135`（`#[ignore]`） | 随机历史从 mkfs 之后或第一个文件之后起（`crates/singlefs-harness/src/history.rs:238-243`）；每个崩溃状态上起可写挂载、发一次布、再摆三个二次崩溃（`:39`） | 第一次崩溃的起点没有；二次崩溃的起点可能有：第一次崩溃截在「记录已持久、根槽没持久」时，恢复施加记录、可写挂载跳过那个 txg（推的）。`:295` 只断言「至少一次把发布截在根落盘之前」，没断言其中有记录已持久的 |

不在 `crash-case:` 行里、但流里确有洞的：`crash_enumeration_fixed_script_stream.rs:1177` 的「预置残留记录」那条（没标 `#[ignore]`，随 checker 档 `cargo test` 跑）。基镜像 = mkfs 之后加一条残留记录与它点名的单元（`:1122` 起；`:1028-1031` 文档），重开可写挂载施加它、写行发布取 txg 6，断言原文 `"txg 1–4 与 6–8，txg 5 的根槽从没写过"`（`:1150`）。它只展开 B 的根槽段之后的小段（`:1174`），洞是流中途造出来的，C380 的 txg 5 就是它。

崩溃放量流水线的流都从 mkfs 出发（`crates/singlefs-checker-tier/src/crash_amplification.rs:3`）；`grep -ln 'residual\|残留' tests/crash_amplification_*.rs tests/common_crash_points/*.rs src/crash_amplification.rs`（在 `crates/singlefs-checker-tier` 下）零命中，放量流里没有预置残留记录的那一条。

## 什么现象会推翻这些结论

- (a)/(b)：`target_for_publish` 不再是 txg 的纯函数，或分配器根环表不再按 txg 取最旧有效根，洞进门槛的方式就不是上面写的那样。
- (c) 挂点表：取号不再是第一个写，或 `record_root_written_by_this_process` 的逐个加一断言去掉，P0 / P1 的约束就变了。
- (d)：checker 的候选集不再滤被抛弃的根与低于 F 的根，② 那一类对象就没了。
- (e)：只读挂载或取号之前的某一步开始写系统配置槽，「永远」就不成立。
- (f)：`crates/singlefs-core/src/` 里出现删行代码。
- (g)：8 条用例里任何一条改成从带残留记录或崩溃后挂载的镜像起步。

## 没做什么

- 没改仓内任何文件；没跑 cargo build / test，没跑门禁、变异、E156 复跑；没提交。
- 标「推的」的各行（洞②③、前洞的多扣机理、8·h_缺 + 2 的拟合、挂点表整张、(d) ①–④、(e)「永远」的代码路径、(f) 记录条数上限、(g) 崩溃注入二次崩溃起点）都没用程序验过。
- C583 不在这次要答的七问里，没查。
- 草稿目录里只有这份报告，没建编译目录或仓副本。
