# m3-ring-r1 攻方腿报告（Opus，2026-09-30）

接续：上一条攻方腿撞限额中断，没写报告。它落在 `/tmp/claude-1000/m3-ring-r1-opus/` 的仓副本、原型与 E156 副本五个挂点的产物核过（编得过、退出码 0、与这一条重量的数对得上）之后接着用；这一条加了挂点 5、必红世界、I-3.9 单值开关、前缀截点装置，重编后全部直接量与枚举重跑。副本上的数都是副本上的数，不算入库装置上的数。

## 复跑

```
bash research/scripts/run-with-memory-cap.sh 6G bash research/scripts/capped.sh 6 \
  bash research/prompts/m3-ring-r1-opus-model/scripts/rebuild-and-run.sh <草稿目录> [build|direct|zero-single|cuts|enumerate|enumerate2|enumerate3|e156|all]
```

脚本把仓拷到草稿目录、打上 `core-and-checker.diff`、放进原型 crate、编译、按批跑。`build` 加 `zero-single` 这一条这次从头验过（输出与 `results/h1/E-*.out` 逐字节相同）；其余几批的脚本就是这次实际跑的那几份，只把路径换成了环境变量，没从头再验。

模型目录 `research/prompts/m3-ring-r1-opus-model/` 各文件 sha256（产物 182 个文件的 sha256 在 `results/SHA256SUMS`）：

```
5218f3d0814c349038c3478ee5b7247aebbd515ffcf0fb12c1a88cce2e845cbe  core-and-checker.diff
ffb4b32a837c6325dd2eccc16302fdee75adbe49fd6ed94ef2a6de436f376b87  e156_fill_probe.diff
cf7288716ed23156f14ca8db77c683a3b140dfc05d6217aaa422e28ab9a2c307  ring-hole-proto/Cargo.toml
504b196b62c99621be8a9beb7ff035c8281b6e70939e8d82a9879737c5a7aa5c  ring-hole-proto/src/lib.rs
eb6a5cd88ae2b17bd8a2cff49c31f8b241d3f57cb47bdac891c2a25d091e0f67  ring-hole-proto/src/bin/ring_hole_crash_states.rs
3855e41a9c1e6e346510735908638691670cb6e29aa6afef9baeb3332e4322d2  ring-hole-proto/src/bin/ring_hole_direct.rs
9704844ecdc8bce7d6589346b9926bc5dddf6c5e898287ec7391d25c48f8dd79  ring-hole-proto/src/bin/ring_hole_prefix_cuts.rs
7d4d6ef4a18a3fca5635062e79b45bcb5e9de328c3764c4d1f9153325ecbb985  scripts/e156_peaks.py
d4607f830da6c0d1260d156211804bb257c79c8c48ec2c192e66dc7589a4d7e3  scripts/rebuild-and-run.sh
10eae5bfcd96e24660d809dc53f84a890e60b62df10bde8dc00fd169118ca606  scripts/run_cuts.sh
d18eeb7ba5c6197cb23f4bf14438e7687d8086ec166aa4e35b2f68ab8152a7ea  scripts/run_direct.sh
b75d9a41101ef755984a729b65aa8daf69e1fcdfc98e0a0149c3913419939c4f  scripts/run_e156_modes.sh
55b4ced006c20c5a12451e21c7fabcc55d55a084e8c7871373433111e18e3dcd  scripts/run_enumerations_2.sh
5227a1d0b22bf1c89cc8b640125af39aebc0c86fd0edd15805065ff7bb2e5a30  scripts/run_enumerations_3.sh
129033a37db7934b3ab311549742bcc038bc02800fc87db8ba588416ec161161  scripts/run_enumerations.sh
d11cd24aac63230bf0f2ecb3ed6eacfc17efe6432bcd410cb614e74cc216f324  scripts/run_zero_single.sh
1371d727c2ad18b6c01770e302f378a799d243b5d01caaf6e4cde64771fff635  results/SHA256SUMS
```

挂点编号（副本开关 `HoleFillPoint`）：0 不补；1 = P1 取号之后单独一段 [根槽 FUA][屏障]；2 = P2 根前（补洞写排在写行那次根槽写之前，分配器在写行那次发布之后记补洞）；3 = P2 根后；4 = P3 暖机之后；5 = P2 根前、分配器在写行那次发布取落点之前就记补洞。P0 没做（取号之前写盘，撞「取号是第一个写」，没有实测价值）。

## 各格判定一览

| 格 | 判定 | 依据（量过 / 推的） |
|---|---|---|
| H1 在 E156 取样点上多扣降到 0 | P1、P2（根前、根后）、P3 四个都降到 0（前洞、后洞，S ∈ {4, 8, 16}，k ∈ {1, 2, 4}） | 量过，E156 副本五个挂点 |
| H1 判据的盲区 K1 | **打中，不分辨臂**：崩在新实例写行那次发布（记录已持久、根槽没持久）留下的洞，四个挂点都补不了，后洞多扣峰值 S = 8 时 350 槽、S = 4 时 158 槽，与不补逐格相同。E156 的洞只造在工作负载的发布上，取样点里没有这一类 | 量过，直接量矩阵 A、C |
| H1 前洞补了放不放得出槽 K6 | 前洞补上之后扣住的槽数不变（61 对 61），「多扣」归零只是那些槽有了根引用，没有一槽被放出来；后洞才真放（398 → 342） | 量过 |
| H2 判据字面（候选集引用已复用的单元） | **没打中**：八段全量枚举（合计 1048837 态）加 24 组前缀截点（合计 1448 个截点）里没有一个状态让枚举器的池级 checker 或内容 oracle 红在「被引用的单元被改写」上。带文件后洞那段历史 4325419 态超过 10⁶ 没全量跑，是这一格最大的空白 | 量过（范围见 H2 一节） |
| H2 K2（分辨臂） | **P2 根后、P3 打中，P1、P2 根前不中**：崩在「写行那次根已持久、补洞写没持久」之后，下一次挂载择到的根已越过洞，洞永远不再补。P3 这个窗口跨整段暖机 | 量过，全量枚举 |
| H2 K3（分辨臂） | **P2（分配器在写行之后记）与 P3 打中，P1 不中**：补洞写落盘之后最新根里的账是按补洞之前的门槛算的，I-3.1 在合法状态红。P3 连挂载做完的静止盘面都红 | 量过，前缀截点与直接量 |
| H2 P2 根前的另一笔 | 层 0 的记录核对器把补洞根当成一次发布的根、去它前面找单元，65569 态里 58983 态判红（盘面没坏，是装置按位置认发布） | 量过；原因是推的 |
| H2 必红世界 K4 | 补洞写成全零或写成所选根的字节：I-3.1 红，但只红在假根成为环里最旧那一步（29 步里 1 步）；checker 没有「槽位与 txg 对得上」的不变量 | 量过 |
| H3 | 可补的洞补上之后 I-3.9 单值判不红；不可补的后洞、读不出的 slot(h) 上单值判红。C380 不能关，对象换成「比最新根旧一整圈以上的残留根」与「读不出的根槽」 | 量过（被抛弃根、低于 F 的根两类没量） |
| 今天的代码 K7 | 不补（挂点 0）时，带后洞的池上下一次挂载加用户写之后 I-3.1 在合法状态红（50 个跟进状态里 32 与 35 步） | 量过，全量枚举的对照 |
| H4 | S-修 不要新写代码路径：任何一次系统配置写本来就落在坏的那一槽；要改的只有拒的判据。提一个零轮形态「一步界」 | 推的，没量 |
| H5 | I-3.8 草案的条件前一半要整轮清扫，`crates/singlefs-core/src/` 里没有清扫；连崩挂载留下的 (j, 0, 0) 行正是删不掉的那一类 | 推的加算术 |
| H6 | C463 的「当场自愈」与 H4 是同一个闸：一块盘少于两槽自证过时可写挂载本来就拒 | 推的 |

## H1　补洞的形态

### 副本上做了什么

`core-and-checker.diff` 四处改动：`mount.rs` 加挂点开关与补洞写（读阶段 `effective_root.checkpoint_txg > chosen_root.checkpoint_txg` 时把 `effective_root` 的根记录经现成的 `CommitStep::WriteRootRecordForceUnitAccess` 写进 slot(h)）；`allocator.rs` 加 `record_hole_fill`（不走逐个加一的断言，盖掉 slot(h) 的旧占用者、按新门槛回收，预演的拷贝上同样记）；`transaction.rs` 让 P2 的补洞写并进写行那次的根槽步骤；`walk.rs` 加 I-3.9 单值开关。多扣的量法（`ring-hole-proto/src/lib.rs` 的 `disk_metrics`）：最新根那棵账里已释放且释放代大于门槛的槽，减去候选根（有效且 txg ≥ F_生效）还引用的槽。

### 量到的：E156 取样点（前任跑的，挂点 0–4；这一条重编之后没重跑，挂点 0–4 的代码路径这一条没动）

`results/e156/peaks.txt`，列是 S、ρ、位置、k，然后每个挂点四个数（Δ 实时峰值、Δ 每次峰值、h_缺 峰值、观测点数）：

```
$ grep -n '^8 1 back 1\|^8 1 front 4\|^16 1 back 1\|^4 1 back 1\|^TOTAL mode[01]' results/e156/peaks.txt
3:4 1 back 1 | 174 174 12 25 | 0 0 0 25 | 0 0 0 25 | 0 0 0 25 | 0 0 0 25
15:8 1 front 4 | 34 34 4 35 | 0 0 0 35 | 0 0 0 35 | 0 0 0 35 | 0 0 0 35
16:8 1 back 1 | 366 366 24 49 | 0 0 0 49 | 0 0 0 49 | 0 0 0 49 | 0 0 0 49
30:16 1 back 1 | 716 716 48 97 | 0 0 0 97 | 0 0 0 97 | 0 0 0 97 | 0 0 0 97
40:TOTAL mode0.out position=front cells=16 peak_delta_real=34 peak_delta_every=34 max_h_que=4 observations=572
41:TOTAL mode0.out position=back cells=16 peak_delta_real=716 peak_delta_every=716 max_h_que=48 observations=1030
42:TOTAL mode1.out position=front cells=16 peak_delta_real=0 peak_delta_every=0 max_h_que=0 observations=572
43:TOTAL mode1.out position=back cells=16 peak_delta_real=0 peak_delta_every=0 max_h_que=0 observations=1030
```

挂点 2、3、4 的 TOTAL 行与挂点 1 相同（同一文件第 44–49 行）。照跑前判据的字面，四个挂点都「在 E156 前洞与后洞取样点上把多扣降到 0」。不补时后洞的峰值是 174（S = 4）、366（S = 8）、716（S = 16），比正文第二节引的前洞 10–34 大一个量级；这一条自己的直接量在 S = 4、8 上得到同样的 174 与 366（`results/h1/run_direct.log` 第 13、37 行），两套量法对得上。E156 副本是关掉第一段停机条款跑的（`E156_FILL_PROBE_IGNORE_STOPS=1`，锚点模型不认识补洞，照常会停）。

### K1　四个挂点都补不了的一类洞（打中，不分辨臂）

历史：进程在 txg h − 1 之后正常退出；下一次可写挂载取了号，写行那次发布（txg h，新实例 j）的记录已持久、根槽没持久时断电；再下一次可写挂载。许可它的两句代码：

- 恢复不施加别的实例的记录：`crates/singlefs-core/src/recovery.rs:3075`：「下一条的实例代号与所选根不同即停——所以只有所选根自己那个实例的记录是候选；所选根是 mkfs 的第 0 代根时一条都不施加。」所以这次挂载 `effective_root` = `chosen_root`，手里没有 txg h 的根。
- 新实例的第一个 txg 却把它算进去：「新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`crates/singlefs-core/src/mount.rs:1092`）。h 被跳过，slot(h) 留着 h − 3S 的根。

量到的（后洞，K = 3S + 4 次覆盖写里的峰值；`results/h1/run_direct.log`，row=1 是这一类洞）：

```
$ grep -n 'A-s8-back-row1-m[0-5] \|A-s4-back-row1-m1 ' results/h1/run_direct.log | sed 's/name=direct_mount //; s/ name=direct_peak.*peak_over/ peak_over/'
20:A-s4-back-row1-m1 rc=0 chosen=15 effective=15 row=17 instance=3 row_crash_holes=[] peak_over_withheld=158 peak_released_by_window_floor=174 red_steps=0 
43:A-s8-back-row1-m0 rc=0 chosen=27 effective=27 row=29 instance=3 row_crash_holes=[] peak_over_withheld=350 peak_released_by_window_floor=366 red_steps=0 
44:A-s8-back-row1-m1 rc=0 chosen=27 effective=27 row=29 instance=3 row_crash_holes=[] peak_over_withheld=350 peak_released_by_window_floor=366 red_steps=0 
45:A-s8-back-row1-m2 rc=0 chosen=27 effective=27 row=29 instance=3 row_crash_holes=[] peak_over_withheld=350 peak_released_by_window_floor=366 red_steps=0 
46:A-s8-back-row1-m3 rc=0 chosen=27 effective=27 row=29 instance=3 row_crash_holes=[] peak_over_withheld=350 peak_released_by_window_floor=366 red_steps=0 
47:A-s8-back-row1-m4 rc=0 chosen=27 effective=27 row=29 instance=3 row_crash_holes=[] peak_over_withheld=350 peak_released_by_window_floor=366 red_steps=0 
48:A-s8-back-row1-m5 rc=0 chosen=27 effective=27 row=29 instance=3 row_crash_holes=[] peak_over_withheld=350 peak_released_by_window_floor=366 red_steps=0 
```

这类洞与可补的洞还会接在一起：工作负载的发布崩出洞 28，下一次挂载（挂点 1）补上它、又崩在自己写行那次，留下洞 29，再挂载之后峰值 352（同一文件第 85 行 `C-s8-back-row0-repeats1-m1`，`row_crash_holes=[29]`、`peak_over_withheld=352`）。补洞把可写挂载变成「每次都可能再造一个补不了的洞」的那一步并没有动。连崩 6 次写行（洞 28–34 共 7 个）时峰值是 266（第 84 行）：洞数多了峰值反而降，因为每个洞都让一条一圈之前的旧根留在候选集里、它引用的槽不算多扣——这里的「多扣」量法对连着的洞偏低，扣住的总槽数没单独比，不拿这个数说「洞多了更好」。

前洞上这一类洞多扣为 0（`A-s8-front-row1-m0`：`peak_over_withheld=0`）：写行那次没被施加，没有释放代写着 h 的记录，slot(h) 也没有旧根。所以 K1 只在环转过一圈之后才有代价。

四句：
1. 分不分辨臂：不分辨。P1、P2、P3 与不补在这一格逐数相同，病根在共用的前提「能补的只有 effective_root 那一版」（正文第二节第 5 条）。照 `evidence-discipline.md`「判据自己也会写错」那张表第一行，不拿它判臂，另立一笔账。
2. 系统当时看不看得到判别它的东西：看得到。挂载时环里有一条自证过、实例不是所选根实例、txg 大于最新根的记录，`first_txg_of_new_instance` 正是读了它才跳号的；缺的是一条能写进 slot(h) 的有效根。
3. 满足判据的哪个分句：H1「降不到 0 的写明剩多少、剩在哪一类洞」——剩 350（S = 8）与 158（S = 4），剩在「新实例写行那次发布崩出的后洞」。E156 的取样点不含这一类，所以判据前半句（取样点上降到 0）照样成立；这是取样点的盲区，不是哪个挂点的输。
4. 跑前条款的改法在这一格还中不中：四个挂点全中；对照臂 P-无 的一个具体形态不中（下一小节）。

### 对照臂的一个具体形态：一圈窗口（只在我的模型上量过、被攻过零轮）

门槛不取「环里最旧有效根」，取「txg 大于 最新根 − 3S 的有效根里最旧的那条」：比最新根旧一整圈以上的根只可能是某个洞的槽里留下的。`disk_metrics` 把这个门槛下还扣住的槽数与今天的门槛并排量了（只是盘面上的度量，分配器、checker、回退候选集都没按它改）。`peak_released_by_window_floor` 是它比今天的门槛多放出的槽数峰值：可补的后洞不补时 382（S = 8）、190（S = 4），补了之后 0；不可补的后洞上 366（S = 8）、174（S = 4），而四个挂点在这一格放出 0。挂载刚做完那一步，窗口门槛放出的与补洞放出的相同：

```
$ for m in 0 1; do grep -h 'step=mounted' results/h1/A-s8-back-row0-m$m.out | grep -o 'point=.*withheld_under_window_floor=[0-9]*'; done
point=NotFilled hole=28 step=mounted newest=31 oldest_valid=4 floor=4 missing=[5, 6, 7, 28] withheld=398 over_withheld=52 hole_slot=Some(4) window_floor=8 withheld_under_window_floor=342
point=P1AfterAcquisition hole=28 step=mounted newest=31 oldest_valid=8 floor=8 missing=[] withheld=342 over_withheld=0 hole_slot=Some(28) window_floor=8 withheld_under_window_floor=342
```

它不是用户定的「重排」，也不解决 I-3.9 在可补洞上的区间（洞那一版的释放代仍然没有根）。只当 K1 那一格「有没有别的出路」的一个数。

### K6　前洞：补了不放槽

```
$ for m in 0 1; do grep -h 'step=mounted' results/h1/A-s8-front-row0-m$m.out | grep -o 'point=.*withheld_under_window_floor=[0-9]*'; done
point=NotFilled hole=6 step=mounted newest=8 oldest_valid=0 floor=0 missing=[6] withheld=61 over_withheld=8 hole_slot=None window_floor=0 withheld_under_window_floor=61
point=P1AfterAcquisition hole=6 step=mounted newest=8 oldest_valid=0 floor=0 missing=[] withheld=61 over_withheld=0 hole_slot=Some(6) window_floor=0 withheld_under_window_floor=61
```

前洞时门槛是第 0 代根（环还没转满），补不补都是 0；补上 txg 6 的根之后那 8 槽变成「被候选根 6 引用」，多扣按定义归零，扣住的总数没变。判据「把多扣降到 0」在前洞上量的是记账口径，不是放出来的空间；交用户的代价数里前洞这一行写「放出 0 槽」。

### 其余几问

- 补哪几个洞：副本只补 `effective_root` 那一个。更早的洞、K1 那一类、记录自证得过但没被施加的（事实表洞②，没造）都不补。
- 写几次、段序列：三个挂点都是多 1 次根槽 FUA 写；根槽写是 FUA，录制流在 FUA 写之后切段，所以 P2 的两个根槽写落在两个相邻的段里，不是同一段的子集。S = 4 前洞那条流：不补 46 次写 9 段 327711 态，P1、P2、P3 都是 47 次写 10 段 327712 态（`results/predecessor/count_all.log`；事实表挂点表 P2 那一行写的崩溃状态翻倍在副本上没出现）。P1 另多一道屏障。
- 分配器入口：要另开（副本的 `record_hole_fill`），而且记的时点本身是一格（见 H2 的 K3）。
- 回退候选集：后洞补上之后候选集仍是 24 条，少了 h − 3S（那条比最新根旧一圈的根）、多了 h；不补时那条旧根在候选集里再留 3S 个 txg。回退扫描（每个跟进状态对每条候选根回退一次再覆盖写两次）里没有 checker 红，被拒的都是 `TargetNotACandidate`。

## H2　崩在补洞中间

### 取样与估时

枚举用 `singlefs_checker_tier::crash` 的 `enumerate_layer0_selecting_versions_observing_each_state`（每段都展开、原地覆写三态），每个状态跑恢复、内容 oracle、池级 checker、记录核对器；盘是内存稀疏盘，每段历史的前缀只建一次、从内存拷。每次跑之前用 `closed_form_state_count` 与带撕裂的计数算全量。

| 历史 | 挂点 | 全量状态数 | 跑没跑 |
|---|---|---|---|
| 树表 0 条的池，洞造在同一实例的暖机发布上（后洞，S = 4，可补） | 0 | 65568 | 跑了 |
| 同上 | 1、2、3、4、5 | 各 65569 | 跑了 |
| 带文件，前洞，S = 4 | 1、3 | 各 327712 | 跑了 |
| 带文件，前洞，S = 4 | 5 | 327712 | 跑到 163872 态时后台任务到 30 分钟上限被停，没重起，这一段不算结果（`results/h2/file-s4-front-m5.stopped-partial.out`） |
| 带文件，后洞，S = 4 与 S = 8 | 任一 | 4325419（超过 10⁶） | 不跑，交主 agent |
| 带文件，前洞，S = 8 | 任一 | 327712 | 没跑（缩掉的几何） |

估时：先跑的一段 65569 态 6 线程 110.5 秒，约 593 态每秒；第一批 3 段估 6 分钟（实测 127、128、129 秒）；第二批 1 段 65568 态加 3 段 327712 态估 30 分钟（实测 118、729、644 秒，第四段没跑完）；第三批 2 段估 4 分钟（实测 119、126 秒）。原计划带文件后洞 S = 4 三段，算出 4325419 态之后换成前洞。缩掉的：带文件的历史只留 S = 4 前洞、挂点只留 1 与 3；树表 0 条那段只取 S = 4。留下的每段历史崩溃状态没缩。这一条腿连前任一共跑的全量：前任 65569，这一条 1048837 加没跑完的 163872，合计 1278278 态。

每个状态之后的跟进（下一次可写挂载、建文件或覆盖写 2 次、对每条候选根回退一次再覆盖写 2 次，每步跑池级 checker）只在「单元段落了 0 个、1 个或全部」的状态上做（50–70 个），这是缩过的，不是枚举域。

带文件后洞那段补了一个不是枚举域的装置：被测挂载那条流的每个前缀截点（`ring_hole_prefix_cuts`，S ∈ {4, 8} × 前洞后洞 × 六个挂点，24 组 1448 个截点），每个截点跑 checker、下一次挂载、2 次覆盖写。前缀截点是枚举域的子集，打中的是可达状态，没打中不算证据。

### 判据字面：候选集引用已复用的单元——没打中

八段全量枚举里，枚举器自己的内容 oracle 与池级 checker 零红（各段 `PROTO name=tally` 行 `oracle_violations=0 … checker_violated=[]`）；前缀截点里挂点 1、5 零红。照跑前判据，没有挂点因 H2 出局。限度写在前面：树表 0 条那段历史没有已释放的单元，补洞抬门槛放不出任何槽，这一格在它上面是空转；带文件前洞那两段门槛是 0，同样放不出槽。真正会放槽、会复用的只有带文件后洞，那段只有 73 个前缀截点（每个挂点），没全量跑。

挂点 5（分配器在写行那次取落点之前就记补洞、补洞写却在写行那次的单元与记录之后才落盘）按代码推有一个窗口：门槛先抬、旧根 h − 3S 还在盘上且有效时，它引用的槽已经可以被写行那次发布拿去写。挂点 5 的 242 个前缀截点（后洞 146 个）上没红；是分配器这几次恰好没拿那几槽，还是 checker 在这些截点上看不出，没分清。这一条是推的，没打中。

### K2　补洞写排在写行那次根槽之后，洞会永远留下（P2 根后、P3 中，P1、P2 根前不中）

历史：带洞 h 的池上可写挂载，写行那次（txg h + 1）的根槽已持久、补洞写还没持久时断电。下一次挂载择到 h + 1 那条根，`effective_root` 不再大于 `chosen_root`，洞 h 没有人再补。

```
$ grep -n 'all_states_by_roots\|state_class count=1 ' results/h2/file-s4-front-m3.out
105:PROTO name=all_states_by_roots count=262155 roots_landed=r7=0,r6=0,r8=0
106:PROTO name=all_states_by_roots count=1 roots_landed=r7=1,r6=0,r8=0
107:PROTO name=all_states_by_roots count=65547 roots_landed=r7=1,r6=1,r8=0
108:PROTO name=all_states_by_roots count=9 roots_landed=r7=1,r6=1,r8=1
113:PROTO name=state_class count=1 r7=1,r6=0,r8=0|chosen=7 effective=7 crash_floor=0 next_fill=false after_next_mount: newest=10 floor=0 missing=[6] hole_slot=None
```

P2 根后是 1 个状态（两次根槽 FUA 之间）。P3 是一整段：`results/h2/wf-warm-s4-m4.out` 第 106、107 行，写行那次根已落而补洞写没落的状态 12 + 9 个，第 113、114 行的跟进里下一次挂载之后 `missing=[7, 8, 9, 10, 18]`、`floor=6`，洞 18 还在、门槛还被 slot(18) 里的旧根 6 拉着。挂点 1 与 2 的六段枚举里没有「写行那次根已落、补洞写没落」的状态（`all_states_by_roots` 四行都是补洞根先于写行根）。

四句：分辨臂（P1、P2 根前不中）；系统看得到（下一次挂载时 h 的记录还在 journal 环里，但所选根已经越过它，今天的读阶段不再把它交出来）；它不是 H2 判据的字面（没有单元被复用），落在 H1「降不到 0 的写明剩在哪一类洞」；改法「补洞写必须先于写行那次根槽持久」在这一格不中——P1 与 P2 根前都满足。

### K3　补洞写落盘之后最新根的账对不上环（P2 分配器后记、P3 中，P1 不中）

一条根里的账是按「这条根写下去之后环里最旧的有效根」算的门槛回收的。补洞写另外盖掉一条旧根（h − 3S），最旧有效根再往前挪；最新根若是在分配器记下补洞之前算的账，I-3.1（已分配统计对得上）就多记了一截。

```
$ for m in 0 1 2 3 4 5; do grep -h 'name=cuts_summary' results/h2/cuts-s8-back-m$m.out | cut -d' ' -f5-; done
point=NotFilled cuts=72 red_at_crash=0 red_after_next_mount=0 red_in_suffix=0
point=P1AfterAcquisition cuts=73 red_at_crash=0 red_after_next_mount=0 red_in_suffix=0
point=P2BeforeRowRoot cuts=73 red_at_crash=21 red_after_next_mount=0 red_in_suffix=0
point=P2AfterRowRoot cuts=73 red_at_crash=21 red_after_next_mount=0 red_in_suffix=0
point=P3AfterWarmUp cuts=73 red_at_crash=1 red_after_next_mount=27 red_in_suffix=0
point=P2RecordedBeforeRowPublish cuts=73 red_at_crash=0 red_after_next_mount=0 red_in_suffix=0
```

S = 4 后洞六行的数与此相同（`results/h2/run_cuts.log`）。P2 的 21 个截点从补洞根与写行根都落盘起、到第一次暖机的根落盘为止，红的原文（`results/h2/cuts-s8-back-m3.err`，21 行相同）是「记账的已分配 Some(6553600)，遍历全部有效根得到 6094848」。P3 是挂载做完的那个盘面自己红（截点 72），以及任何一个截点上再挂一次、挂完就红（27 个截点）：带后洞的池只要用挂点 4 挂成一次，静止盘面 I-3.1 就是红的，要到下一次发布才绿（`results/h1/run_direct.log` 第 17、41 行 `red_steps=1`）。P1 在「补洞根已落、写行根没落」的截点上是绿的（`results/h2/cuts-s8-back-m1.out` 第 4 行，`crash_newest=28 crash_floor=5 crash_violated=[]`）：那时最新根就是 h 自己，它的账是崩掉的那个实例按「h 盖掉 h − 3S」算的。

四句：分辨臂；系统看得到（分配器知道自己哪一刻记的补洞）；**不是 H2 判据的字面**——账多记是漏放的方向，不是复用，照字面不让挂点出局；它打中的是「合法崩溃状态上池级 checker 红」，这一条在跑前判据里没有格，交主 agent 判归哪里。改法有两个：把分配器的记法挪到写行那次之前（挂点 5，前缀截点上绿，但带来上面那个推的窗口），或者用 P1（补洞写先持久、再记、再发布）。P3 没有对应的改法：补洞之后没有下一次发布来带新账，除非再加一次空发布。

### K7　对照：今天的代码在带后洞的池上 I-3.1 本来就红

挂点 0 的全量枚举（`results/h2/wf-warm-s4-m0.out`）：枚举器自己零红，但 50 个跟进状态里，下一次挂载之后建文件那一步 32 个状态红、再覆盖写那一步 35 个状态红，都是 I-3.1 多记（`results/h2/wf-warm-s4-m0.err`）。补洞之后同一处只剩带着不可补洞 19 的那几个状态红（挂点 1 是 3 个，2、3、5 是 4 个，4 是 14 个）。这是 C566 题面里「洞上 I-3.1 在合法状态判红」那个候选的实测，不是补洞引进的；它说明 K1 那一类洞不光多扣，还让 I-3.1 在合法状态红，补洞管不到。

### P2 根前的另一笔：层 0 的记录核对器

挂点 2 与 5 的两段全量枚举各有 58983 个状态被记录核对器判红（`LAYER0_FINDINGS signatures=2 red_states=58983 states=65569`，签名 `publish=instance7_txg18_root_write20`，原文「恢复自称的那一版该有的单元两份都缺席」），挂点 1、3、4 为 0。盘面没坏（同一批状态 oracle 与池级 checker 零红）；推的原因是核对器按位置把一次根槽写当成它前面那些单元与记录的那次发布的根，补洞根排在写行那次的记录之后、写行根之前，被认成了写行那次的根。选 P2 根前要连这个核对器一起改。

### 必红世界 K4

把补洞写成「盖掉 slot(h) 而不带 txg h 的根记录字节」，两种：全零槽；所选根（txg h − 1）的字节。分配器照常以为 slot(h) 住着有效根 h。S = 8、28 次覆盖写、挂点 1、3、4 × 前洞后洞共 12 格：每格 I-3.1 红，但挂点 1、3 只红 1 步（29 步里假根成为环里最旧那一步；后洞的挂点 4 另加挂载做完那一步）。后洞两种的原文（`results/h1/B-s8-back-zero-m1.err`、`results/h1/B-s8-back-chosen-m1.err`）：

```
VIOLATION I-3.1: 盘 0：记账的已分配 Some(5799936)，遍历全部有效根得到 5603328（…最新根 txg 51、环里自证过的根槽 23 个、最老的自证过的根 txg 29…）
VIOLATION I-3.1: 盘 0：记账的已分配 Some(5799936)，遍历全部有效根得到 6062080（…最新根 txg 51、环里自证过的根槽 24 个、最老的自证过的根 txg 27…）
```

（上面两行中间的省略号是这里略去的字段，整行在那两个文件里。）第二行是危险的方向：有效根走读到的比账上多，slot(28) 里那条 txg 27 的根引用着分配器已经当成可再分配的槽。两点要带走：一，必红世界要跑满一圈（3S 次发布）才红，停得早的用例看到的是绿；二，全零那一种与一次撕裂的根槽写在盘上分不出，checker 里也没有槽位公式（事实表 (a) 末行），「slot(h) 里住的根 txg 就是 h」没有不变量盯着。只在我的模型上量过。

## H3　洞补上之后的 I-3.9

副本的 checker 加了开关 `I39_SINGLE_VALUE=1`：把「释放代落在 (L, T]」收成「释放代等于 T」。S = 8，挂载之后覆盖写 6 次、回退到 最新 − 5 那条根、再覆盖写 3 次，每步判一次（10 步）。红的步数（`results/h1/run_direct.log` 第 93–108 行的 `red_steps`，区间判法的对照全是 0）：

| 洞 | 不补（挂点 0） | 补（挂点 1） |
|---|---|---|
| 前洞，工作负载的发布崩出来的（可补） | 10 | 0 |
| 后洞，可补 | 10 | 0 |
| 前洞，写行那次崩出来的（K1） | 0 | 0 |
| 后洞，写行那次崩出来的（K1） | 10 | 10 |
| 后洞，可补，但 slot(h) 读不出（补洞写成全零） | — | 7 |

三件事：

1. 可补的洞补上之后，单值判法在这几段历史上站得住，回退之后也是（只量了挂点 1、S = 8、这一种回退）。
2. K1 的后洞上单值判法红，而且红的不是洞那个 txg：`results/h1/D-s8-back-row1-m1-single.err` 的 10 行是同一个槽「释放代 5 不在 (4, 8] 里」一路到「(4, 18] 里」。L 是 slot(h) 里留下的那条比最新根旧一圈的根（txg 4），它后面的根被一条条盖掉，T 每发布一次往后挪一格。C380 题面说的「洞很长时区间会松」在这里不要求洞长：一个洞就让这一类记录的区间宽度涨到 3S − 1。
3. slot(h) 读不出时（撕裂的补洞写之后不再有机会重补的情形，或持续读错的槽）：洞那一版做的释放写着 28，候选集里 27 之后是 29，单值判红 7 步（`results/h1/E-s8-back-zero-m1-single1.err`）。

所以：I-3.9 不能整条收成单值，C380 不能关。量到的剩余对象是两类——「比最新根旧一整圈以上的残留根做 L」（只要有补不了的后洞就有）与「洞那一版的根槽读不出」。事实表 (d) 的另两类（被抛弃的根、低于 F 的根被滤出候选集）没造历史、没量。区间写法要不要改：若采「一圈窗口」（候选集不收比最新根旧一圈以上的根），第一类就没了，那是零轮形态、没实现。C380 的验收办法里那句「把区间收成单值，那 12 个合法状态必须由绿转红」在补洞之后要换对象：残留记录那条流补上洞之后应当由红转绿，判别力自证得另找一条带不可补后洞的流。

## H4　撕裂轮换之后永远只读（找反例，全是推的，没量）

1. **S-修 不是一个新的写**。系统配置槽的世代号取「这块盘自证过的最大世代号加一」、槽位取世代号模 2（`crates/singlefs-core/src/transaction.rs` 第 361–366 行与第 409 行）。一槽自证不过时，不管坏的是新的那一槽还是旧的那一槽，下一次系统配置写（取号、轮换、抬 F 都走同一个 `write_system_configuration_slot`）算出来的槽位都正好是坏的那一槽。所以「顺带重写撕裂那一槽」今天的写法已经会做，拦住它的只有取号之前那一判。S-修 的代价数是 0 行写路径代码，问题全在判据放不放行——它和 S-辨 是同一个决定，不是两条可以并列比较的候选。
2. **拒因的注释自己写了这笔账**：「撕裂的系统配置轮换也落在这里：代价是那之后只剩只读挂载（同一判决 L1）。」（`crates/singlefs-core/src/mount.rs:434`）
3. **S-辨 按「撕裂」与「读不出」分，上一轮量过会丢写**：`unreadable-at-mount-r2-main-verification.md` L2 表里 甲-读错 丢写 120、全是全零形。全零的槽分不出是撕裂的轮换还是一份确认过的见证烂掉了。
4. **零轮形态「一步界」**（我提的，没实现、没量、被攻过零轮）：一块盘上自证不过的那一槽，世代号至多比同盘自证过的那一槽大 1，也就是至多多见证一次系统配置写。所选那一版的末条计数器若已经大于同盘自证过那一槽的 tail，藏着的那一槽见证不了比所选那一版更新的发布，判据可以判假；相等时照旧判不出、拒。撕裂的是发布末尾的轮换时，那次发布的根槽 FUA 先于轮换，所选那一版就是它，放行；撕裂的是取号写或抬 F 先写时两者相等，仍然永远只读。它要成立还得核：取号写带的 tail 是现有最大值（`transaction.rs` 第 850–851 行读出来是），一次发布切成多条记录时计数器怎么比，多盘各自的界怎么合。这三样我没核完。
5. S-留 的反例方向：出路若是「只读挂载下定点修复」，它要在只读路径里写盘，撞上一轮判决 L1 第三条记的「只读挂载不写盘」在 kb 里没有条款、只是今天代码的行为。

## H5　实例表删行（找反例，推的加算术）

I-3.8 那一行的回收条件：「行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）」（`.claude/kb/invariants.md:141`）。

- 条件的前一半今天没有输入：`grep -rln '整轮清扫\|fn scrub\|scrub' crates/singlefs-core/src` 零命中。只做后一半（根环里没有该实例的根）不够：行还管着该实例写了没发布的单元，那正是前一半要清扫判的。
- K1 与它直接相关：连崩的可写挂载每次留下一个实例 j，它没有一条根进过环（后一半立刻成立），却必有写行那次写下、没发布的单元（前一半管的正是它）。下一次挂载给它写 (j, 0, 0)。崩溃循环里涨得最快的就是这一类行，它们恰好是没有清扫就删不了的。
- 「根环每个槽都读成功」：一个根槽持续读不出时条件永远不成立，删行永远做不了（C335 那份调查的情形）。
- 删行那一写落在哪：写行那次发布本来就 COW 重写整条链，删行并进去不多一次写；崩在中间时那次发布不被施加（K1 同一条机理），旧链原样。这一格没找到反例。
- 不做的代价（算术，式子在 `crates/singlefs-core/src/admission.rs` 第 183–222 行，N_switch = 3、一片 32768 字节 = 2 槽、369 行一片）：

| 可写挂载次数 | 片数 | 链本身每盘 | 切换预留里链那一半每盘 | 写行那次要点名的记录条数 |
|---|---|---|---|---|
| 10⁴ | 28 | 56 槽（0.875 MiB） | 224 槽（3.5 MiB） | 1 |
| 10⁶ | 2711 | 5422 槽（84.7 MiB） | 21688 槽（338.9 MiB） | 41 |

  10⁶ 次时每次可写挂载要把 84.7 MiB 的链整条重写一遍（每盘），写行那次发布切成 41 条记录。超过 24723 行（67 片）之后写行那次是多记录发布。

## H6　系统配置冗余（找反例，推的）

- C462：可写挂载在择系统配置之前已经按盘数拒了（`admit_the_writable_device_count`，`crates/singlefs-core/src/mount.rs` 第 4439 行），零设备走得到 `choose_system_configuration` 的是只读挂载（`mounted_read.rs` 第 555 行）与恢复的另外几个入口（`recovery.rs` 第 828、4224 行，`mount.rs` 第 755、3630 行）。「由上层挡」要在这几处各挡一次；单列成员一处就够。代码自己留了话：`crates/singlefs-core/src/recovery.rs:1049`：「池里一块盘都没有：没有哪一块盘点得出名，保持这个函数原先对空池的判定不变（见报告里的设计问题）。」按「成员按调用方要做的决定分」判：零设备时调用方该查的是自己交进来的盘表，几份系统配置不一致时该查的是哪块盘是外来的，两个决定不同，单列成员站得住。没找到反例。
- C463：「可写挂载当场自愈重写」今天走不到。一块盘两槽全废时它自证过的槽数是 0，少于 2，可写挂载在取号之前按 H4 那一判拒掉；C463 题面的「照常挂上」只对恢复与只读挂载成立。所以自愈重写与 H4 的放行是同一个闸、同一个写（H4 第 1 条：世代号从 0 加一，先写槽 1，下一次轮换写槽 0，两次系统配置写补齐），可以合成一条条款；但合起来之后「报出来」这一半不能省——只读挂载与恢复永远不写，只能靠报。要核的一处：那块盘的世代号从 1 重新起算，比别的盘小得多；`choose_system_configuration` 跨盘取的是第一块交得出的盘那一份（`recovery.rs` 第 1020–1029 行只比 fsid 与设备数），我没找到跨盘比世代号的地方，没逐个调用点核。

## 几个改法各修哪一格

「量过」指副本上这一次的产物里有那一格的数；「推的」指按代码推、没实现或没跑。

| 改法 | E156 取样点上的多扣 | K1 不可补的后洞 | K2 洞永远留下 | K3 账对不上环 | 记录核对器 | I-3.9 单值 |
|---|---|---|---|---|---|---|
| P1 取号之后单独一段 | 降到 0（量过） | 不修，350 / 158（量过） | 不中（量过，三段全量枚举里没有那种状态） | 不中（量过，前缀截点 0 红） | 0 红（量过） | 可补的洞上绿、K1 后洞上红（量过） |
| P2 根前，分配器后记（挂点 2） | 降到 0（量过） | 不修（量过） | 不中（量过） | 中，21 个截点（量过） | 58983 态红（量过） | 没量 |
| P2 根前，分配器先记（挂点 5） | 没跑 E156；直接量峰值 0（量过） | 不修（量过） | 不中（量过） | 前缀截点 0 红（量过）；门槛先抬的窗口（推的） | 58983 态红（量过） | 没量 |
| P2 根后（挂点 3） | 降到 0（量过） | 不修（量过） | 中，1 个状态（量过） | 中，21 个截点（量过） | 0 红（量过） | 没量 |
| P3 暖机之后 | 降到 0（量过） | 不修（量过） | 中，写行根到补洞写之间整段（量过） | 中，挂载做完的盘面就红（量过） | 0 红（量过） | 没量 |
| 一圈窗口（我提的，零轮） | 后洞与补洞放出的槽相同，前洞 0（量过，只是盘面度量） | 放出 366 / 174（量过，只是盘面度量） | 不适用（不写盘） | 没实现（推的：分配器、checker、回退候选集要一起改） | 不适用 | K1 后洞那一类对象消失（推的）；可补洞那一版的释放代仍没有根（推的） |

照跑前判据：H2 的出局条款（候选集引用已复用的单元）没有一个挂点中；H1 的进表条款（取样点上降到 0 且 H2 没有那种状态）四个挂点字面上都满足。K2、K3 分辨得出臂，但不在跑前判据的字面里；K1 不分辨臂。这三样怎么进判决由主 agent 定。我自己提的「一圈窗口」「一步界」与「槽位与 txg 对得上的不变量」都只在我的模型上量过或只是推的，被攻过零轮。

## 没打中的形状

- 「补洞写落盘、旧根 h − 3S 被盖掉之后，它引用的槽被复用，而某个崩溃状态里候选集还引用它们」：P1 的补洞写是 FUA 加屏障之后才记、才发布，三段全量枚举（65569、65569 前任那一次、327712）与挂点 1 的 242 个前缀截点零红。但会真的放槽的带文件后洞只有前缀截点（146 个）。
- 「补洞写撕裂（slot(h) 变成读不出）之后下一次挂载出事」：枚举域里的原地覆写三态包含它，树表 0 条那五段零红；下一次挂载仍从记录拿到 h、重补。只在没有已释放单元的池上量过。
- 「补上的根 h 被实例表判成被抛弃」：没出现。h 的实例是上一个实例，写行那次给它写的行正是 (i, h)；「补洞根已落、写行根没落」时 h 自己是最新根。
- 「回退到补上的那条根、或回退之后补洞出事」：回退扫描（树表 0 条六段各 385–396 次、带文件前洞两段各约 500 次回退）里 checker 零红。
- 挂点 5 的「门槛先抬」窗口：推得出，242 个前缀截点没红，没分清是没复用还是看不出。
- 用户动作放开的范围：跟进状态上扫了「建文件或不建、覆盖写 2 次、对每一条候选根各回退一次再写 2 次」；直接量里覆盖写次数取到 3S + 4、连崩写行取 0–6 次。没扫删除、多文件、抬 F、卸载再挂。K2、K3 分辨臂的结论不依赖后缀（红在崩溃状态本身或下一次挂载刚做完），K1 的数依赖覆盖写的负载。

## 这条腿自己的限度

- 全部是副本上的数。副本的 `record_hole_fill` 是原型写法，绕开了根环表逐个加一的断言，没有按 `crates/` 的规矩写测试。
- 带文件后洞没有全量枚举（4325419 态，超过 10⁶ 不许跑）。H2「没打中」在最要紧的那段历史上只有前缀截点，照「没打中一次不算」，这一格不能当结论用。
- 几何只有 S ∈ {4, 8}（E156 副本另有 16）、两块盘、块 512、环默认大小；负载只有单文件 3000 字节覆盖写。
- 「多扣」的量法是我自己的（`disk_metrics`），与 E156 的 Δ 在可比的三格上相等（174、366 与前洞 8 对 E156 的 k = 1 前洞 8），别的格没对。
- E156 副本五个挂点是前任跑的、关掉了第一段停机条款；这一条没重跑，只核了文件在、退出码 0、后洞峰值与直接量一致。
- H4–H6 没有一格量过。
- K3 里 P2 的 I-3.1 红出现在前缀截点装置上；树表 0 条那几段全量枚举里池级 checker 零红是因为那里没有已释放单元，不是 P2 没事。

## 什么现象会推翻这些结论

- K1：恢复改成施加别的实例的记录，或新实例第一个 txg 不再把没施加的记录算进去；或者主 agent 在入库装置上造同一段历史、量到多扣为 0。
- K2：读阶段在所选根越过洞之后仍把洞那一版交出来补。
- K3：一条根的账不再按「写下去之后的环」的门槛算，或 I-3.1 的口径改成认写这条根时的门槛。
- K4：checker 加了槽位不变量，或必红世界第一步就红。
- H3：在一条带不可补后洞的流上单值判法不红。
- H2 没打中那一格：带文件后洞全量枚举里出现 oracle 或池级 checker 红。

## 没做什么

- 没跑 checker 档测试、门禁、变异、入库的 E156；没改仓里 `crates/`，没做 git 写。
- P0 没做；洞②（记录自证得过但施加前验证失败）、多记录发布崩在记录段中间、被抛弃根与低于 F 的根两类 I-3.9 对象没造历史。
- 带文件后洞全量枚举（4325419 态）没跑；带文件前洞挂点 5 跑到一半被后台时限停掉、没重起；带文件前洞挂点 0、2、4 与 S = 8 没跑。
- 「一圈窗口」「一步界」没实现。不判正推腿的格，不替主 agent 采纳。
- 草稿：这一条建的 `/tmp/claude-1000/m3-ring-r1-opus-2/replay`（复跑脚本验证用的仓副本与编译目录，1.2G）已删；前任建的 `/tmp/claude-1000/m3-ring-r1-opus/target`（784M）与 `/tmp/claude-1000/m3-ring-r1-opus/repo`（205M）这一条接着用过，交回前已删；前任的 `/tmp/claude-1000/m3-ring-r1-opus/e156/`（E156 副本五个挂点的原始输出 8.5M，没入库，模型目录里只有峰值表与 sha256）、`h1/`、`h2/`、`tools/` 留着；这一条的 `/tmp/claude-1000/m3-ring-r1-opus-2/` 只剩日志与产物原件（已拷进模型目录）。
