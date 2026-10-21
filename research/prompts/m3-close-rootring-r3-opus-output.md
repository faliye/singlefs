# m3-close-rootring-r3 云端攻方腿（Opus）报告（2026-10-20）

第三轮（最终轮），前几轮判决：`research/prompts/m3-close-rootring-r1-main-verification.md`、`research/prompts/m3-close-rootring-r2-main-verification.md`。攻的是**系统配置落地补丁打上之前**的 `crates/`（主工作区此刻的样子，拷进草稿目录的仓副本；`crates/` 下全部 `.rs` 的 sha256 清单再取 sha256 是 b523d85891b7327b57d1a760417241421b4749bd7891d00f019189826e375eb2，与第二轮攻方留的开工拷贝同一个数）。补丁只 grep 过：它不碰 F2 判法函数。副本上量出的数只算副本上的数，不是入库装置上的数。

## 复跑

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-rootring-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen <开工时 crates/、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default 的一份拷贝> --work <新的空草稿目录>
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改了 `rerun_steps`：把 `tests/` 下两份测试与一个公共模块拷进冻结副本的 `crates/singlefs-harness/tests/`，各跑一次 `cargo test -p singlefs-harness --test <目标> -- --nocapture --test-threads 1`（线程 31），抽出 `R3NARROW` / `R3ZERO` 行与 `out/` 逐字节比，再用 `tools/crosscheck_r2_same.py` 把这一轮的输出与第二轮攻方 `out/same.out` 逐格比。我用新拷的冻结副本（`~/.local/share/singlefs/work/m3-close-rootring-r3-opus/frozen/`）与新草稿目录跑过一次，退 0，原样输出：

```
name=rerun_compare file=narrow.out same=yes lines=43
name=rerun_compare file=zero.out same=yes lines=14
name=r2_same_out_crosscheck lines=26 agree=26
```

| 文件 | sha256 |
|---|---|
| `tools/rerun.sh` | cf5a3e564201735e67e4e1a9ccc6da7602beb75bdd794a2f978b7f39d890aab5 |
| `tools/crosscheck_r2_same.py` | aa531824cf66faee178e0a55dd279cf785c13cddf2e87a235e8974bb3d46ad45 |
| `tests/rootring_r3_common/mod.rs` | 7bd60f0cd5e31ea507b8340604e6fae49c52d05a198b4f681ee0e4cc7d2da538 |
| `tests/rootring_r3_attack_narrowed_same_instance.rs` | e412c68fdaf7ea6ec8b21132cbf581a263fb3e08c04067d7836a09fb177521ee |
| `tests/rootring_r3_attack_zero_unit_roots_after_a_full_ring.rs` | d6f86ba5523639bf87813ad3cdad98c1509955e39a34a9d44323282026761f7e |
| `out/narrow.out` | f2fbc778d33eef1d50e5ec485aa09d3872852711ccb413755ba0a3492c23e685 |
| `out/zero.out` | 70737591c6c9438d81edcb6ae441ba18bec342911f3c19ad107c3204eafa9867 |

模型目录 `research/prompts/m3-close-rootring-r3-opus-model/`。造盘面用入库 harness 的内存稀疏盘，只读导入 `tests/common_unwitnessed_publish`、`tests/common_root_ring_holes` 与清扫场景模块 `tests/common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs`。core 不改：F2 判法照抄一份（从第二轮攻方模型拷来），「所选根的实例表读不出」那一支三取一（今天 `exact` / 第二轮形态 `same` / 这一轮收窄形态 `narrow`），取 `exact` 时与 core 的函数逐格比，40 格 `replica_mismatches=0`。

## 各格判定一览

| 格 | 判定 | 打中的东西（一句） | 量过 / 推的 |
|---|---|---|---|
| Z2 收窄形态在三形上落哪一版 | 没打中 | 第二轮攻方 `cross`：见证到的那一版属更旧实例，收窄形态照今天落 R，恢复 `Ok`（第二轮打中的那一格修到）；C751 登记那一形：落 P，只读从 Pm 改落 P；`:656`：三处不变 | 量过（副本，26 格） |
| Z2 同实例见证、同实例的根却注定读不通 | **打中（与 C751 登记那一形同一故障预算）** | 同一实例里清扫发布重写了实例表：见证到的那一版在清扫之前、所选根在清扫之后，收窄形态收进清扫发布那一条——它与所选根共用读不出的新表——恢复从今天的 `Ok(3,22)` 变成 `Fail@(3,23)` | 量过（副本，`out/narrow.out`） |
| Z2 同实例见证而同实例的根已被抛弃 | 没造出 | 同一实例的根不被同一实例的表抛弃；撞号归 C555（第二轮已归） | 推的 |
| Z3 三处交 F2 的记录同一份 | 部分打中（条文层） | 「各重扫」给的是同一种来源，不是同一份；单故障加一次断电之内重扫与第一遍逐条相同，改落点要两样故障（推的） | 读码 |
| Z4 C773 登记那一形转不转绿 | 前提量过，结论推的 | 今天的代码上造出同形：挂载那一刻一槽读回全零、钉在 6，写行与两次暖机三条根 I-3.1 红在记账多的一边（30、42、54 槽），卸载抬 F 到 22 之后转绿；合成形态的门槛与「钉带出来的那一截」罩得住这一形（推的） | 前提量过（`out/zero.out`），合成形态推的 |
| Z4 合法历史上 I-7.18 误红：写者的 t 取哪个数没写 | **打中（条文层，零读故障零断电）** | 分配器记根时的回收门槛用挂载那一刻的 F、抬 F 之后不跟着抬；卸载抬 F 到 22 之后的根带 F = 22、环里最旧根 12–14。t 取分配器那一刻的门槛（12–14）时，checker 照「根自己带的 F」现算 22，求等判红 | 前提量过（`out/zero.out`），后果推的 |
| Z4 零单元发布（树表 0 条）上的 I-7.18 | 打中（条文层，推的） | 零单元发布在根落盘之后才记根；它的 t 不是照抄就是记根之前算的，环转满之后都比 checker 按镜像现算的低一格；I-3.1 在这类根上报不适用，I-7.18 合成形态没写 | 推的 |
| Z4 C684 那一形 | 没打中 | 合成形态的「被抛弃根引用着的槽不并」把 C684 那一槽挡在外面，钉着与没钉着都照旧红 | 推的 |
| Z4 要同步的格式件 | 补漏 | 背景只点了字节表、根记录宽、journal 记录头；另有 GPU 判器两份 WGSL、`layout/01-first-txn` 三种语言的「457 / 512 槽」行、17 份 `.rs` 引宽度常量 | 读码（命令数的） |
| Z5 前两轮报出、没进任何一格的 | 列出 8 条 | 见 Z5.1 | 读码 |
| Z5 数能不能核 | 第二轮攻方 26 格核上；第二轮正推的数要它的核心层开关补丁 | 第二轮 `same.out` 26 格四个字段在这一轮独立造的盘面上逐格对上 | 量过（副本） |

## Z2　C751 收窄形态

收窄形态只改 F2 一臂：今天所选根的实例表读不出时只认恰好等于见证到的那一版（`crates/singlefs-core/src/recovery.rs:6996`「None => order == witnessed_publish,」），收窄后多认「见证到的那一版与所选根同实例时，与所选根同实例的根」，区间与取最新照旧。模型里写成 `witness_is_of_the_chosen_instance`（`research/prompts/m3-close-rootring-r3-opus-model/tests/rootring_r3_common/mod.rs:168`）。

### Z2.1 三形各落哪一版（没打中，量过，副本）

第二轮攻方三形照它的造法重造（384 槽小盘、S = 8，ow / sw3 × k = 3、5），三种规则各算一遍，触发照今天（每处看自己那一样读者）。`R3NARROW_SUMMARY group=second_round_shapes cells=26 replica_mismatches=0 recovery_ok_today_fails_narrowed=0 recovery_fails_today_ok_narrowed=0`。逐形（`out/narrow.out` 里对应的格，F2 交回 / 恢复 / 只读）：

| 形 | 今天 `exact` | 第二轮形态 `same` | 收窄 `narrow` |
|---|---|---|---|
| `cross` 崩在第 2 条根、`table+cfg`（ow k = 5） | (1,9) / `Ok(1,9)` / `Ok(2,11)` | (2,10) / `Fail@(2,10)` / `Ok(2,11)` | (1,9) / `Ok(1,9)` / `Ok(2,11)` |
| `registered`（ow k = 5） | (1,7) / `Fail@(1,7)` / `Ok(1,7)` | (1,8) / `Fail@(1,8)` / `Ok(1,8)` | (1,8) / `Fail@(1,8)` / `Ok(1,8)` |
| `pinned656`（ow k = 5） | (1,8) / `Ok(1,8)` / `Ok(1,8)` | 同左 | 同左 |

读法：第二轮打中的那一格（见证到的那一版属更旧实例）收窄形态退回今天的取法，恢复读得出；登记那一形见证到的 Pm 与所选根 R 同实例，收窄形态照收同实例的 P，只读从 Pm 改落已确认的 P；`:656` 见证到的 P 属实例 1、所选根 Row 属实例 2，三种规则都落 P。26 格里收窄与今天在恢复的成败上逐格相同（汇总行两个计数都是 0）；F2 交回的那一条在见证与所选根同实例的格上可以不同（例：`cross` ow k = 3 崩在第 3 条根、`table+cfg`，今天 (2,8)、收窄 (2,9)，两条都与所选根共用读不出的表，恢复都失败）。

### Z2.2 打中：同一实例里清扫发布重写了实例表（量过，副本）

收窄形态的前提是「同实例的根与所选根共用那张实例表」只在见证到的那一版属别的实例时才坏。一个实例里重写实例表的不止写行那一次：清扫发布也重写（`crates/singlefs-core/src/instance_table_sweep.rs:721`「instance_table: InstanceTablePlan::Rewrite(self.instance_table.clone()),」；写行那一次是 `crates/singlefs-core/src/mount.rs:5041`「instance_table: InstanceTablePlan::Rewrite(instance_table),」）。于是同实例的见证可以落在清扫之前、所选根在清扫之后。

历史（`tests/rootring_r3_attack_narrowed_same_instance.rs` 的 `sweep` 组，环洞池两块盘、S = 4）：
1. 入库清扫场景的前半（`pool_with_a_dead_instance`）：实例 1 写到 txg 5，一次顺序写崩在第一条记录之前；实例 2 可写挂载、覆盖写到 txg 19。
2. 实例 3 可写挂载，清扫阈值取「有一行就扫」：写行 txg 20、暖机 21、22，挂载末尾清扫发布 txg 23 删实例 1 那一行、重写实例表（`R3NARROW_SWEEP_PRECONDITION instance=3 rows_deleted=true`）。
3. 实例 3 接着覆盖写 24、25、26。
4. 崩在 txg 24 的根槽之后、它那次轮换之前（一次断电）。
5. 读的那一刻：txg 24 的根指着的实例表（清扫发布写的那一张）两份读回全零，每块盘较新的系统配置槽（txg 23 那次轮换写的）读回全零——与 C751 登记那一形、第二轮 `cross` 那一形同一个故障预算。较旧的系统配置槽是 txg 22 那次轮换，见证到的那一版 = (3,22)，与所选根同实例。

`out/narrow.out` 原样一行：

```
R3NARROW shape=sweep crash_after_root=(3,24) this_publish_rewrote_the_instance_table=false fault=table+cfg chosen=(3,24) table_readable=false witnessed=(3,22) today rec=Ok(3,22) ro=Ok(3,24) rw=Fail | f2_core=(3,22) | exact: f2=(3,22) shares_chosen_table=false rec=Ok(3,22) ro=Ok(3,24) rw=Ok(3,22) | same: f2=(3,23) shares_chosen_table=true rec=Fail@(3,23) ro=Ok(3,24) rw=Fail@(3,23) | narrow: f2=(3,23) shares_chosen_table=true rec=Fail@(3,23) ro=Ok(3,24) rw=Fail@(3,23) | replica_exact_matches_core=true
```

读法：今天 F2 只认 (3,22)，它用清扫之前那张读得出的表，真跑恢复 `Ok(3,22)`；收窄形态看到见证与所选根同实例，收进区间 [(3,22), (3,24)) 里最新的同实例根 (3,23)——清扫发布自己，与所选根共用那张读不出的表（`shares_chosen_table=true`），恢复照报失败。只读挂载今天就不退（所选那一版打得开）；可写挂载真跑今天就拒（最新根实例表读不出那一拒），模型那一列只量建上一版。这一组 14 格里只有这一格分辨今天与收窄（`R3NARROW_SUMMARY group=sweep cells=14 replica_mismatches=0 recovery_ok_today_fails_narrowed=1 recovery_fails_today_ok_narrowed=0`）；窗口是「清扫之后第一次发布的根已落、轮换没落」。

由用户决定的几步放开扫过的：清扫之后再写几次（0–3 次，崩在每一条根之后都判了）、故障带不带系统配置那一样；固定的是前缀（实例 2 写满环、实例 3 清扫）与故障种类。清扫阈值「有一行就扫」是只供用例的取值（`crates/singlefs-core/src/instance_table_sweep.rs:73`「pub const ANY_ROW: Self = Self(0);」）；产品路径阈值取 369（`crates/singlefs-core/src/instance_table_sweep.rs:71`「pub const PLACEHOLDER_UNTIL_C690: Self = Self(INSTANCE_TABLE_PAGE_RECORDS - 1);」）或交给挂着时的后台清扫，清扫发布在同一实例中途重写实例表这件事产品路径上照样发生——阈值只决定多少行之后才扫，没量产品阈值下的那一形（推的）。

推翻条件：一份实现里清扫发布不换实例表指针（照抄旧链），或 F2 在收同实例的根时另判实例表指针。

### Z2.3 同实例见证而同实例的根其实已被抛弃（没造出，推的）

一个实例的表只写比它旧的实例的行，同实例的根不被同实例的表抛弃；清扫只删行，不加行。唯一的形仍是实例代号被重用（C555），第二轮已归到两种取法共有的前提，这里不重复。

### Z2.4 一条两头都不坏的条件（我提的，只在我的模型输出上逐格推过、被攻过零轮）

把「见证到的那一版与所选根同实例」换成「见证到的那一版与所选根**共用同一个实例表指针**」（两样都在根记录里，不读表）。逐格推（推的，没改模型重跑）：登记那一形 Pm 与 R 共用 ⇒ 收同实例的 P，只读修到；`:656` 与第二轮 `cross` 见证不共用 ⇒ 照今天；Z2.2 那一格 (3,22) 用清扫之前那张表、不共用 ⇒ 照今天落 (3,22)、恢复读得出；其余同实例格两种条件同结局。

## Z3　C755「三处交 F2 判法的记录同一份（触发之后各重扫环）」

第二轮攻方已指出今天三处来路不同、两次独立挂载各读各的（第二轮攻方报告 Y3.3），这一格不重复那一句，只看补上的条文本身与重扫多读的那一遍。今天三处的入参（读码）：恢复在走读失败那一刻重扫（`crates/singlefs-core/src/recovery.rs:7115`「// 走读失败才再扫一遍环拿记录给退回的判法（只读、不写盘；走读做成的恢复不多扫）。」），只读挂载用挂载开头那一遍（`crates/singlefs-core/src/mounted_read.rs:628`「let records = scan_journal(reader, &system_configuration);」），可写挂载用读阶段那一份（`crates/singlefs-core/src/mount.rs:8023` 传的 `&records`）。

### Z3.1 「同一份」与「各重扫」两个词说的不是一件事（条文层，读码）

「各重扫」给出的是三份各自扫出的记录，同一种来源、同一个时刻（触发之后），不是同一份字节。条文要么写「同一种来源」，要么写明在一处挂载之内哪几样判断共用这一遍。后一条有一处今天就分开取：可写挂载退过去之后建那一版用的末条记录取读阶段那一份（`crates/singlefs-core/src/mount.rs:8029`「record_standing_for_the_version(&records, &version_fallen_back_to);」），系统配置落地补丁又在 F1 再判之前另扫一遍（C754「重扫」）。只把 F2 的入参换成重扫的那一份，同一次可写挂载里就有读阶段、F2 前、F1 再判前三遍，F2 认的「见证到的那一版」与建那一版取的末条记录可以出自不同的两遍。条文要连「F2、建退到的那一版要的那条记录、F1 再判同用触发之后那一遍」一起写（推的，没造盘面）。

### Z3.2 重扫多读的环在单故障下改不改落点（没打中，推的）

- 持续的读故障两遍读到的一样，重扫与第一遍逐条相同。
- 只有瞬时读故障才让两遍不同；它本身就是一样故障。触发 F2 要所选那一版的读者读不通，单故障加一次断电之内那要另一样读故障（提交协议的屏障保证根落了单元就落了），所以「重扫与第一遍不同、又走到 F2」要两样故障。
- 反过来，瞬时故障落在第一遍、让所选那一版换成施加记录重建的那一版时，F2 第一句（所选那一版不是所选根就不退）在两种取法下一样，不看记录。

所以单故障加一次断电之内重扫不改落点；两样故障时改，而且今天的只读与可写挂载用的是第一遍，换成重扫只是把受影响的那一遍从第一遍挪到重扫那一遍，不减少暴露。推翻条件：一样故障就能同时让所选那一版走不通、又让环里一条记录在两遍之间读得不同（例如那条记录与所选那一版的某个单元在同一段坏区上）。

## Z4　C773 合成形态

### Z4.1 C773 那一形在今天的代码上重造（前提量过，副本）；合成形态下转不转绿（推的）

登记行的复现盘面已不在。照登记行的机理重造一形（`tests/rootring_r3_attack_zero_unit_roots_after_a_full_ring.rs` 的 `pinned` 组）：环洞池 S = 4，实例 2 覆盖写到 txg 19（环里 8–19）；实例 3 可写挂载那一刻根环区域 0 槽 2（住着 txg 18）读回全零——写者把门槛钉在那一槽可能住过的最旧 txg 6；之后写行 20、暖机 21、22，再卸载（抬 F 到 22，根 23、24、25）。崩在每一条根的根槽之后，池级 checker 判 I-3.1。`out/zero.out` 原样（几行整行，未截）：

```
R3ZERO group=pinned root=(3,20) units_written=14 F=0 oldest_root_in_the_ring=9 checker_threshold=9 threshold_at_the_last_publish_with_units=9 I-3.1=Violated(盘 0：记账的已分配 Some(2818048)，遍历全部有效根得到 2326528（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 20、环里自证过的根槽 12 个、最老的自证过的根 txg 9、遍历的候选根槽 12 个、并进遍历的由记录施加出来的版本 0 个、被实例表判抛弃的根槽 0 个、回退下界 F 0、低于 F 的根槽 0 个)
R3ZERO group=pinned root=(3,23) units_written=12 F=22 oldest_root_in_the_ring=12 checker_threshold=22 threshold_at_the_last_publish_with_units=22 I-3.1=Holds
R3ZERO_SUMMARY group=pinned roots=6 i31_red=3
R3ZERO_SUMMARY group=clean roots=6 i31_red=0
```

20、21、22 三条根红在记账多的一边，差 491520、688128、884736 字节（30、42、54 槽）；同一段历史不注读故障（`clean`）6 条根全绿。这就是 C773 登记的那一类（钉只在内存里），登记行「差的槽释放代全在 (21, 25]」那一形的几何不同、机理相同。

合成形态下（推的，没实现字段）：写者 t = max(F, min(环里最旧有效根, 钉)) = max(0, min(9, 6)) = 6，p = 6（区域 0 槽 2，3 × 2 + 0）；checker 取根自己带的 F 现算 max(0, min(9, 6)) = 6，相等；「钉带出来的那一截」= (6, 9]——这一形差的槽释放代都在这一截里（释放代 ≤ 8 的在实例 2 那几次发布里已回收，推的），并进来转绿。I-7.19：槽 (0,2) 在镜像上住着实例 2 的 txg 18，不是本实例的根，不红。这几步都靠「写者如实」，坏实现那几类第二轮已攻，不重复。

推翻条件：在这份盘面上数出 20 号根记账里释放代落在 (6, 9] 的已释放槽数，不等于 30。

### Z4.2 打中：写者的 t 取哪个数没写；取分配器记根时的门槛，卸载之后每一条根 I-7.18 误红（前提量过，后果推的）

合成形态只写「根记录加门槛 t」与「checker 取根自己带的 F 现算、要求等于 t」，没写写者的 t 是哪个数。今天分配器里现成的回收门槛是记根那一刻按挂载时的 F 算的：`crates/singlefs-core/src/allocator.rs:2611` 那一行取 `occupancy.effective_floor`，而它的注释写着「/// 挂载时生效的 F。抬 F 生效之后不跟着抬：释放代 ≤ 新 F 的抬 F 自己已经回收过，之后释放的释放代都高于新 F，」（`crates/singlefs-core/src/allocator.rs:1266`）。

同一份 `out/zero.out`，`clean` 组（零读故障、没有断电之外的故障）：

```
R3ZERO group=clean root=(3,23) units_written=12 F=22 oldest_root_in_the_ring=12 checker_threshold=22 threshold_at_the_last_publish_with_units=22 I-3.1=Holds
R3ZERO group=clean root=(3,24) units_written=12 F=22 oldest_root_in_the_ring=13 checker_threshold=22 threshold_at_the_last_publish_with_units=22 I-3.1=Holds
R3ZERO group=clean root=(3,25) units_written=12 F=22 oldest_root_in_the_ring=14 checker_threshold=22 threshold_at_the_last_publish_with_units=22 I-3.1=Holds
```

卸载把 F 抬到 22 之后的三条根带 F = 22，环里最旧的根 12–14；挂载时的 F 是 0（20–22 三条根带 F = 0）。t 取分配器记根那一刻的门槛 max(0, 12) = 12（推的，照上面两行代码），checker 照根自己带的 F 现算 max(22, 12) = 22，求等判红——每一次正常卸载之后的盘面都红。第二轮攻方 Y4.2 打的是 checker 一侧取 F_生效，这一格是写者一侧，两头都取「根自己带的 F」才对得上。第二轮正推 Y4.2 的字段表写过「F 取这条根自己带的那一个，不取 `occupancy.effective_floor`」，这一句没进合成形态的登记口径（第二轮判决 Y4 那一格与记录里用户认的那一行都只写 checker 一侧）。

改法：合成形态补「t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)」（第二轮正推那一句）。推翻条件：实现里 t 的采样点在抬 F 之后另有一处把 F 并进去。

### Z4.3 零单元发布（树表 0 条）上的 I-7.18：合成形态没写（打中，条文层，推的）

零单元发布只出现在树表 0 条的版本上（`crates/singlefs-core/src/transaction.rs:1291`「/// 一次零单元发布的参数（D16（发布语义） 已定项 9：树表 0 条时空发布写零个单元）。」），它不经分配器，记根在根落盘之后（`crates/singlefs-core/src/allocator.rs:2581`「/// 零单元发布在根落盘之后调）；这次发布失败时调用方把分配器整个换回发布之前（`transaction::publish_version` 与」）。所以零单元发布的根里那个 t 不是照抄上一版的，就是记根之前算的——都不含这条根挤掉的那一槽。没有文件的池挂载几次（每次写行加两次空暖机）环就转满；环满之后一条空暖机的根挤掉环里最旧的根，checker 按镜像现算的「环里最旧有效根」比写者那一刻高一格，求等判红。今天 I-3.1 在这类根上报不适用（`crates/singlefs-checker/src/walk.rs:18297`「judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");」），合成形态的 I-7.18 没写这一格；第二轮正推的拟稿另有「零单元发布抄来的只要求 t 不高于现算」一句，也没进合成形态的登记口径。改法：I-7.18 在最新根下面没有记账树时同样报不适用，或照第二轮正推那一句只判单边。没造盘面（推的）。

### Z4.4 C684 那一形结局（没打中，推的）

我先试的打法：C684 那一槽释放代落在「钉带出来的那一截」里时，那一截不看引用并进来会把它也并了。合成形态写明「被抛弃根引用着的槽不并」，C684 那一槽正是只剩被抛弃根引用，挡在外面——写者挂载时那一槽钉着（截里）与没钉着（截外）都照旧不并、照旧红，与用户定的「不并」一致。连带的一条没核的前提（推的）：C773 登记那一形的调查目录名带「abandon」（`.claude/kb/checks-owed.md:523` 那一行的路径），若那 44 槽里有被抛弃根也引用着的，「被抛弃根引用着的不并」会把它们挡在截外，登记那一形就不全转绿；复现盘面不在，核不动。第二轮正推说「那 44 槽没有任何根引用」也是推的。

### Z4.5 字段宽度、落点与要同步的格式件（补漏，读码）

宽度沿用第二轮正推的算术（根记录 457 → 466、journal 新根段 364 → 373、记录头 487 → 496，`crates/singlefs-format/src/lib.rs:234`「pub const ROOT_RECORD_BYTES: u64 = 457;」、`crates/singlefs-format/src/lib.rs:247`「pub const JOURNAL_HEADER_BYTES: u64 = 487;」），这一轮没找到算错的地方。要同步的件比背景列的多：

- GPU 判器：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:87`「const ROOT_RECORD_BYTES: u32 = 457u;」，另有 `crash_judge_checker.wgsl` 一份写着算法类型、nonce、MAC 那 29 字节从 428 起（命令 `grep -rlE 'ROOT_RECORD_BYTES|JOURNAL_HEADER_BYTES|JOURNAL_NEW_ROOT_SEGMENT_BYTES|\b457u\b|\b487u\b|\b364u\b' crates --include=*.wgsl` 数出 2 份）。新判 I-7.18、I-7.19 与 I-3.1 并「钉带出来的那一截」在 GPU 判器那一份 I-3.1 上也要搬，合成形态没点名。
- 引这三个宽度常量的 `.rs`：`grep -rlE 'ROOT_RECORD_BYTES|JOURNAL_HEADER_BYTES|JOURNAL_NEW_ROOT_SEGMENT_BYTES' crates --include=*.rs | wc -l` 数出 17 份（format、core、checker、harness、checker 档的崩溃事实表与 e158、e165 两个装置都在里面）。
- kb 字节表：`.claude/kb/layout/01-first-txn.md` 有「457 / 512 槽」的行（`grep -rl '457 / 512' .claude/kb/layout | wc -l` 数出中英日三份）。

## Z5　未定项清单的完整性与数能不能核

### Z5.1 前两轮报出、没进这一轮任何一格的（读码，逐条指出处）

1. C751 收窄之后的新结局：第二轮正推 Y2.4 第三条「表读不出、见证到的那一条根读不出而区间里有同实例的根：今天不退，改后退到同实例的根，多挂上」——收窄形态在见证与所选根同实例时仍有这一格，「用例要钉」没进格。
2. Z2.2 这一形（同一实例里清扫发布换表），这一轮新报，零轮。
3. C755 保底（退到的那一版自己读不通、所选那一版读得通时留在所选那一版）：第二轮正推「规则没说」，判决只写了「并集只用来判退」。
4. C755 取并集时两种挂载判退前要先跑走读的挂载耗时：第二轮攻方 Y3.2 推的，没进格、没量。
5. GPU 判官：C751 一臂（第二轮正推 Y2.1 表里 `crash_judge_recovery.wgsl` 那一行）、C755 并集（「规则没说」）、C773 三条新判（Z4.5），三笔都要搬，没有一格管。
6. I-7.19 自己的一格误红：读坏钉着的槽、这个进程对它的根槽写报失败而实际落了盘（第二轮正推 Y4.6 末条，两样故障）——没进格。
7. 「被抛弃根独占又被钉扣着没回收的槽」在合成形态下红，误红还是该红没定（第二轮正推 Y4.4 末段）。
8. Z4.2、Z4.3：t 的取法与零单元发布上的 I-7.18，这一轮新报，零轮。

### Z5.2 前两轮的数能不能核

| 数 | 出自 | 能不能在入库代码上核 | 这一轮做了什么 |
|---|---|---|---|
| 第二轮攻方 `same.out` 26 格 | 第二轮攻方模型 | 能：只读导入入库 harness，F2 照抄一份、取今天时与 core 逐格比 | 我另造同三形，26 格的 core F2、第二轮形态 F2、恢复两列四个字段逐格对上（`name=r2_same_out_crosscheck lines=26 agree=26`，量过，副本；照抄的判法从第二轮拷来，与它共用一段代码，不算独立路径，core 那一列是独立的） |
| 第二轮攻方 `unify.out` 183 行、Y3 34 / 8 格 | 第二轮攻方模型 | 能（同上） | 没核；主 agent 已复跑 `same=yes` |
| 第二轮正推 Y1 122 张镜像、2094 个根；Y3 16 → 46 格 | 第二轮正推模型 | 开关 0 那一列是入库代码；裸公式与例外两列、Y3 的同实例开关要它的核心层开关补丁（`tools/core-switch.patch`），不是入库代码 | 没核；开工时 `ps` 看到主 agent 那条 16G 复跑在排队 |
| 第一轮 50 / 100 格 | 第一轮攻方 `union.out` | 能（harness 测试，core 不改） | 没核；第一轮判决复跑 `same=yes` |

另：这一轮主工作区 `crates/` 的 `.rs` 清单哈希与第二轮攻方的开工拷贝相同（开头那个数），第二轮在副本上量的数就是今天代码上的数。

## 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看得到判别它的东西吗 | 满足判据字面的哪一个分句 | 跑前条款给的改法在打中的格上还中不中 |
|---|---|---|---|---|
| Z2.2 清扫换表，同实例见证收进共用坏表的根 | 分辨：今天 `exact` 不中（恢复 `Ok(3,22)`），收窄 `narrow` 与第二轮 `same` 中（`Fail@(3,23)`） | 看得到：所选根、见证到的那一版、候选根的实例代号与实例表指针都在根记录里，不读表 | 正文 Z2「有没有一形同实例见证而……」那一问的近旁：同实例见证，被收的同实例根不是被抛弃，而是与所选根共用读不出的表——恢复从读得出变成照报失败 | 收窄形态：中；Z2.4「共用实例表指针」条件：不中（推的） |
| Z3.1 「同一份」与「各重扫」 | 条文层，不是臂 | — | 正文 Z3「交 F2 的记录是否逐条相同」：不是，同一种来源三份 | 「各重扫」照字面：不修可写挂载里 F2 与建退到那一版取记录分属两遍那一处 |
| Z4.2 写者 t 取挂载时的 F | 分辨「t 含根自己的 F」与「t 取分配器记根时的门槛」两读：后者在正常卸载之后红 | 看得到：根带的 F、环里的根都在盘上 | 正文 Z4「合法历史上 I-7.18 会不会误红」 | 补第二轮正推那一句（t = max(根自己的 F, 分配器门槛)）：不中（推的） |
| Z4.3 零单元发布 | 条文层：合成形态没写这一格 | 看得到：最新根下面有没有记账树 | 同上 | I-7.18 在没有记账树时报不适用，或只判单边：不中（推的） |

## 几个改法各修哪一格

| 改法 | Z2 第二轮 `cross` | Z2 登记那一形只读 | Z2 `:656` | Z2.2 清扫换表 | Z4.2 卸载之后 | Z4.3 零单元 |
|---|---|---|---|---|---|---|
| 收窄形态照字面 | 修（量过） | 修（量过） | 不变（量过） | 坏（量过） | — | — |
| Z2.4「见证与所选根共用实例表指针才收同实例」（我提的，只在我的模型输出上逐格推过、被攻过零轮） | 修（推的） | 修（推的） | 不变（推的） | 修（推的） | — | — |
| 合成形态照字面（t 不定义） | — | — | — | — | 看实现：t 取分配器门槛则坏（推的） | 坏（推的） |
| t = max(根自己的 F, 分配器门槛)（第二轮正推那一句，未进登记口径） | — | — | — | — | 修（推的） | — |
| I-7.18 没有记账树时报不适用（我提的，零轮） | — | — | — | — | — | 修（推的） |

## 没打中的形状

- Z2：第二轮三形 26 格（ow、sw3 × k = 3、5；`cross` 崩在这次挂载每条根之后 × 两种故障，登记形 4 格，`:656` 2 格），收窄与今天在恢复成败上 0 格不同（量过）；清扫组 14 格（崩在实例 3 写行、两次暖机、清扫、三次覆盖写每条根之后 × 两种故障）只有 1 格打中（量过）；同实例被抛弃没造出（推的）。
- Z3：单故障加一次断电之内重扫不改落点（推的，Z3.2）。
- Z4：C773 同机理的一形在合成形态下转绿（推的，前提量过）；C684 那一形照旧不并（推的）；字段宽度算术没找到错。
- 取样：两份测试各一种几何（384 槽小盘 S = 8；环洞池 S = 4）；先跑一小段量了时长（窄形态那份单线程整份 4–6 秒，零单元那份 3–8 秒），全量就是这些，没有缩。

## 这条腿自己的限度

1. 崩溃状态没照崩溃点重放的枚举域取：没调 `enumerate_crash_states`，经枚举函数跑的状态数是 0。原因：这一轮被攻的三样都要读故障（F2 只在读不通时走），枚举域本身不注读故障；每段历史只取「某条根的根槽落了、它那次轮换没落」这一类崩溃点（它在枚举域里），打中的格只要一个反例。checker 档那个包的原型要连 RocksDB 一起在新编译目录里编，这一轮挂钟里没起。「没打中」的几句只在这些崩溃点上成立。
2. 恢复与只读挂载的「今天」是真跑；三种规则下的结局是在测试里照抄判法现算的（照抄的判法取今天时与 core 逐格比，40 格全同）；可写那一列只量建上一版，不含重建分配器读最新根实例表那一拒。
3. Z4 的合成形态没实现字段与新判：Z4.1 后半、Z4.2、Z4.3 的后果都是按代码推的；量过的只有今天 checker 的 I-3.1 与根带的 F、环里最旧根这几个数。
4. 「一样故障」照 C751、C755 登记行的数法：一个单元两份同时读回全零记一样，每块盘较新的系统配置槽同时读回全零记一样。
5. Z2.2 用的清扫阈值是只供用例的「有一行就扫」；产品阈值下同形没量。
6. 副本上的数，不是入库装置上的数；Z2.4、Z4.3 的改法是我提的，被攻过零轮。

## 进程与负载

开工前 `ps` 看到别的会话的 checker 档崩溃枚举（`cargo test --release -p singlefs-checker-tier`）、crates 变异轻型复跑、形式化轻型、崩溃点重放轻型、第二轮正推与 C656 两条判决复跑在排内存，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio` 进程（`grep fio` 只命中别人编译命令里的文件名）；照常 `nice -n 19`。内存上限派发提示没给，第一次编译取 4G 排了约 300 秒没排上（slice 账上 36–38 GiB / 40.1 GiB），我停掉自己那一次排队（一个 `proc.py stop`，命令一行都没跑，包装报退 143），改 2G 起：编译 65 秒，三次运行各几秒，复跑脚本一次 64 秒（含在新树里编译），都退 0。第一次编译被工具移到后台，等到完成通知，没有留着的进程。

## 要主 agent 跑的命令

没有必须的。判决要复跑时照「复跑」一节：工作目录仓根，`--frozen` 用我留着的开工拷贝 `/home/fy5090/.local/share/singlefs/work/m3-close-rootring-r3-opus/frozen/`（`crates/` 全部 `.rs` 的清单哈希 b523d85891b7327b57d1a760417241421b4749bd7891d00f019189826e375eb2），内存上限 2G 够，线程 31；看两行 `name=rerun_compare` 都是 `same=yes` 与 `name=r2_same_out_crosscheck lines=26 agree=26`。

## 没做什么

- 没改 core，没实现 C751 收窄、C773 字段与三条新判；收窄形态的结局在测试里现算，C773 那几格的后果是读码推的。
- 没打系统配置落地补丁，没在补丁后的树上攻；补丁只 grep 过，F2 判法函数不在它改的范围。
- 没调崩溃枚举函数（限度第 1 条），没跑 checker 档、崩溃点重放、GPU 判器、变异、门禁。
- 没判辩方那几格，没替主 agent 采纳，没读禁读清单里的文件。
- Z5 的数只核了第二轮攻方 `same.out`；第二轮正推的数、第一轮的数没核。
- 草稿目录留着开工拷贝 `/home/fy5090/.local/share/singlefs/work/m3-close-rootring-r3-opus/frozen/`（约 35M，判决复跑当 `--frozen` 用，主 agent 用完可删）与 `logs/`；仓副本 `repo/`（2.1G，含编译目录）与复跑那一次的新树 `rerun1/`（2.0G）已删。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-rootring-r3-opus-output.md` 的末行原样：

```
  ✓ 核了 16 处引文、3 处裸行号，对上 16 处，没判 3 处（其中裸行号没有引文可比 3 处；逐处列在上面）
```
