# 续做推满之后卸载怎么做成：云端正推腿报告（m3-unmount-after-short-continuation-r1，第一轮，2026-10-03）

立场：Q1、Q2、Q4，从条款原文与冻结副本的代码推。冻结副本 `/tmp/claude-1000/m3-unmount-after-short-continuation-r1/tree/`（只读）；我的拷贝 `/tmp/claude-1000/m3-unmount-forward/tree/`
（加了一份探针测试和候选甲的原型）；模型与脚本、原样输出在 `research/prompts/m3-unmount-after-short-continuation-r1-sonnet-model/`（README.md 讲每个文件）。
引的 kb 行号是 kb 文件自己的行号；引的代码行号是冻结副本的（本报告引到的九份 `.rs` 与仓里现在的 sha256 前 12 位逐份相同，所以 cite-check 默认根也对得上）。
没有读禁读清单里的任何文件。方案按 `crates/` 今天的实现谈。

## 0　先说结论（每条下面有出处与推翻条件）

1. **Q1**：在准入式子一直判着的盘上，我没造出这一形。补足环窗口的小盘 81 格里 45 格走得到删除（其余 36 格是文件长不到那么大：准入先拒了写），45 格全是续做走空、卸载成、再挂成；
   准入边缘再 2 格同样。这一形只出在「环窗口装不下、又用只供测试的开关关掉准入」造出来的窄盘上（探针 108 格里 40 格；单元区 ≤ 384 槽）。
   从式子推：这一形要求某次删除类发布每盘新占的槽数超过 W（D28 已定项 1 给的上界）；我量到的真实占用：每批约 3.8 槽对 W = 30，整趟走空的总吃量约 190 槽对环窗口 750 槽。
2. **Q2**：甲、乙新、丙·mkfs 三条站得住（各自代价见 2.1、2.2、2.4），乙偏离条款，丙·式子无效。
   - **甲**（删除类发布的落点那一判另留「卸载那一串」的槽）：条款上只动 D28 已定项 1 一句；在 33 格窄盘上卸载做成 31 格（原代码 6/32），做不成的 1 格是开删之前槽就紧、挂载自己的写行与暖机又把它吃到卸载要的数以下。
   - **乙新**（我提的新候选：卸载那一串预演被拒时，只把新 F 写进每块盘的系统配置、不发空发布）：零槽需求；25 格 Short 都在 ≤ 4 次再挂内续做走空；池级 checker 在抽的 4 格上全绿。
   - **乙**（预演被拒就一个字节不写、F 不抬）：字面上违反 D16 已定项 1「正常卸载」那一行，且 25 格 Short 里有 22 格连再挂都被拒，停在这一形；它字面上「卸载做成」，F 没抬，要用户知情。
   - **丙·式子**（把卸载的最坏量并进环窗口 / 保留池的式子）：在 Q1 找到的格上无效（那些盘上式子本来没判）。
   - **丙·mkfs 拒窄盘**：Q1 的第一条判据直接触发；kb 里已有同形的欠账 C628。
3. **Q4**：每条候选要改的句子在第三节，行号都是现查的。（Q3 代价数不归这条腿。）

## 1　Q1　这一形出在哪些盘上

### 1.1　探针「不补足环窗口」的造法（探针源码已删，我从仓里同名助手读出来的）

- 助手 `start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off`（`crates/singlefs-harness/tests/common_admission/mod.rs:85`）：
  参数取 6 MiB 环那一档，单元区只有 N 槽、不加补足，mkfs 之后、取号与第一个文件之前关准入（助手的文档写成「取号 1、暖机之后」，代码里 `set_space_admission` 在 `acquire_instance` 之前，我按代码）：
  「allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);」（`crates/singlefs-harness/tests/common_admission/mod.rs:119`）。
- 「补足」指 harness 的小盘档位把盘放大到环窗口扣得起：「const SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 815;」（`crates/singlefs-harness/src/history.rs:146`），
  256 / 240 槽两档是「const SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 739;」（`crates/singlefs-harness/src/history.rs:151`）。
  所以 384 槽那一档补足后单元区 = 384 + 815 = 1199 槽，256 档 995，240 档 979；而「不补足」是单元区 192 到 384 槽，环窗口 650 到 750 槽（探针输出的 `rw=` 字段）比整个单元区还大。
- 结论：探针的盘是「式子在它上面连一个普通写都放不过」的盘，准入被只供测试的开关关了才让历史走得下去。

### 1.2　从式子推哪些几何会出这一形

记号（每块盘、单位槽）：F = 删除开始时没挡的槽；take = 一次删除类发布净新占的槽；S = 每区根槽数（探针盘 S = 8）；N_u = 卸载那一串要的槽。

1. 卸载那一串要的槽：抬 F 那一串的 txg 数至多 `ROOT_RING_REGIONS` 次（2 或 3），每次空发布写 6 或 8 槽（探针里 `slots_each` 的分布：73 格 `[6, 6]`、38 格 `[6, 6, 6]`、其余 `[8, …]`），
   所以 N_u 在 12 到 24 槽。回收来的槽在这一串生效之前扣住不能用：「ReclaimedReuse::HeldUntilFloorTakesEffect,」（`crates/singlefs-core/src/mount.rs:2589`），
   预演在这之后整串在拷贝上走：「let placements_rehearsed = match rehearse_the_publishes_raising_the_floor(」（`crates/singlefs-core/src/mount.rs:2599`）。
   ⇒ 卸载做不成 ⇔ 此刻没挡的槽不够 N_u（且不是已经抬到上限之外还能回收的）。
2. 删除类发布只判落点：「let units_landing_is_judged = match allocator.space_admission() {」（`crates/singlefs-core/src/transaction.rs:6232`），
   条款也这样写：「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判」（`.claude/kb/decisions/28-挂载期承诺量.md:36`）。
   落点这一判要的是自己的角色加 ckpt_cost（探针盘上 ckpt_cost = 9 到 15 块），实测被拒时要 14 到 16 槽、缺 1 到 13 槽。
3. 一趟走空对没挡的槽的净消耗：换下的槽进 defer，要等一圈（3S 次发布）才回来，所以前 3S + 1 次发布每次净吃 take，之后吃一个还一个。
   条款的上界：环窗口 = (3S + 1) × W，「let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;」（`crates/singlefs-core/src/admission.rs:394`），
   W 的底数是「pub const DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST: u64 = 24;」（`crates/singlefs-core/src/admission.rs:298`）加每层的 min 项；探针盘上 W = 28 到 30（`rw` ÷ 25）。
4. 式子判着时，最后一次放行的普通写之后：F ≥ 环窗口 + R0，R0 = 保留池 + 切换预留 + 其余扣项。式子本身：
   「可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)」（`.claude/kb/decisions/28-挂载期承诺量.md:19`）。
   R0 我量到的：补足 384 档起点 free = 1170，环窗口 750，同一刻准入读数的可用 = 240 槽（`research/prompts/m3-unmount-after-short-continuation-r1-sonnet-model/outputs/edge384.out` 里 `grow-refused` 那几行的 `available`，3932119 字节 ÷ 16384），所以 R0 ≈ 1170 − 750 − 240 = 180 槽（推的，三个数各自是量的）。
5. 于是停下时的 F_end ≥ 环窗口 + R0 − (3S + 1) × take_real。出这一形要 F_end < N_u ≤ 24，即 take_real > W + (R0 − 24) ÷ (3S + 1) > W。
   **推出来的必要条件：某次删除类发布每盘新占的槽数超过条款给的上界 W，或者有一类消耗不在「式子 + 环窗口」账里（例如关掉准入的只供测试的开关）。**
   量到的真实占用：K = 1、补足 384 档、文件 110 个单元、崩在意图后 50 批的格，起点 free = 937，崩的那一刻 747，一共吃 190 槽 ÷ 50 批 ≈ 3.8 槽 / 批，W = 30（`outputs/edge384.out` 第 2 行）。上界松四到八倍。
6. 几何轴（S = 8 的探针盘，单位区槽数 N）：第一个文件在 N = 750 起放得进（725 放不进），一次覆盖写在 N = 900 起放得进（875 放不进）（`outputs/first-file-threshold.out`，步长 25；写法同 `first_file_threshold` 那个测试）；
   关准入造出来的 Short 在 N ≤ 384，N ≥ 416 的 63 格全部走空 + 卸载成。两段之间隔了一倍多：式子判着时能有文件的盘，离这一形远。

### 1.3　在拷贝上量的格（每格：续做停因 / 卸载结局 / 卸载预演被拒那一次缺的槽数）

做法：一格 = 盘型 + 先覆盖写 O 次 + 文件 U 个单元 + 意图之后崩在第 B 批（直接发意图与 B 批、丢会话）+ 删除批大小 K；崩了重新可写挂载（准入判着、影子账开），
看续做结局；再在盘面的拷贝上试：正常卸载、不卸载直接再挂、卸载后再挂。探针 `probe_unmount.rs`，脚本 `run-par.sh`；全部经 `run-with-memory-cap.sh` 加 `capped.sh 10` 加 `nice -n 19` 起。
开跑前 `ps` 看到别的会话有一趟 31 线程的 `cargo test`（整包慢档）在跑、整机负载 50 到 140，没有性能测量进程；内存包装拦了「同一会话两条日常」，我按它的出路设了并行理由变量。

| 盘型（K = 1 除非写明） | 格数 | 续做停因 | 卸载结局 | 再挂（不卸载）|
|---|---|---|---|---|
| 补足的小盘 240 / 256 / 384 槽档，O ∈ {0, 4, 12}，U ∈ {10, 40, 100}，B ∈ {0, 1, 3}（准入全程判着） | 81 格，其中 45 格文件长得到、36 格 `grow-refused`（式子先拒了那次写） | 45 格都走空（`outcome=Empty`），无 Short | 45 / 45 卸载成（`slots_each` 2 或 3 次） | 45 / 45 再挂成、续做空 |
| 补足 384 档的准入边缘：U = 110、O = 0、B ∈ {0, 50} | 2 格走得到（U ≥ 125 或 O = 4 的 14 格先被准入拒） | Empty | 2 / 2 卸载成 | 成 |
| 不补足的窄盘 192 / 256 / 320 / 384 槽，O ∈ {0, 2, 4}，U ∈ {10, 14, 20}，B ∈ {0, 1, 2} | 108 格 | 40 格 Short，停因 40 / 40 是 `RaiseRefusedForSpace`（抬 F 那一串自己被空间拒）；`AtCeiling` 0 格 | 44 格卸载被拒：40 格 Short，另 4 格队列已走空而卸载仍被拒 | Short 的 40 格全部再挂被拒 |
| 窄盘往大推：384（再 4 格）、416 / 448 / 480 / 512 / 640 / 768 / 896 / 1024 槽，O ∈ {0, 4}，U ∈ {14, 40}，B ∈ {0, 2} | 71 格 | 384 槽 U = 40 的 4 格 Short；N ≥ 416 的 63 格全是 Empty | Short 的 4 格卸载被拒；其余全成 | |
| 窄盘 1800 / 2200 槽，U ∈ {300, 480}，K ∈ {16, 144}，B ∈ {0, 2}（文件 3 到 30 批） | 16 格 | Empty | 16 / 16 卸载成 | 成 |

命令与原样输出（`research/prompts/m3-unmount-after-short-continuation-r1-sonnet-model/outputs/`；行数是我这一次数的）：

```
$ grep -h '^CELL' narrow.out | awk '…'          # 窄盘 108 格
cells 108 Short 40 其中停因RaiseRefusedForSpace 40 AtCeiling  | unmount-err 44 其中Short 40 Empty 4 | Short且不卸载直接再挂也被拒 40
$ grep -h '^CELL' narrow.out | grep -o 'RehearsalRefused(pub [0-9]/[0-9]' | sort | uniq -c   # 卸载预演在第几次/共几次被拒
      1 RehearsalRefused(pub 1/2
      5 RehearsalRefused(pub 1/3
     21 RehearsalRefused(pub 2/2
     12 RehearsalRefused(pub 2/3
      5 RehearsalRefused(pub 3/3
$ …续做最后被拒的那次删除类发布缺几个槽（落点那一判报的 available / demand，每块盘）
缺1槽×2  缺2槽×4  缺3槽×5  缺4槽×7  缺5槽×6  缺6槽×3  缺7槽×5  缺8槽×3  缺9槽×1  缺11槽×2  缺13槽×2
$ grep -h '^CELL' pad-grid-old-format.out | awk '…'    # 补足的小盘
cells 81 grow-refused(写不进那么大) 36 walked 45 outcome=Empty 45 unmount-ok 45 remount-ok(Empty) 45
```

卸载预演被拒那一次缺的槽数：被拒的 44 格报的都是 `PlacementRefused`（取不到落点），错里不带数；我按「卸载那一串走到被拒那次发布的槽数 − 此刻没挡的槽」估：
估缺 0 槽 × 2、1 槽 × 11、2 槽 × 10、3 槽 × 6、4 槽 × 4、5 槽 × 2、6 槽 × 5（推的，每次空发布按 6 槽，没挡的槽取续做停下时的 `free`，3 到 14）。
我试过用「多给 δ 个单元区槽、看 δ 多大转成功」量精确值，机器负载 140 时一趟 4 分钟只出 0 格，没量成，写在「没做什么」。

### 1.4　Q1 的判定

- **在准入判着的盘上没造出**：补足小盘 45 格 + 边缘 2 格全部「续做走空、卸载成、再挂成」；候选甲的回归格（`outputs/jreg-*.out`，补足盘与 1800 / 2200 槽窄盘共 37 格（j3 那一行因 4 分钟超时少跑 3 格），6 格被准入先拒，31 格走空且卸载成）同样不红。这是一次抽样，不是证明。
- **窄盘上造得出，而且稳**：窄盘 108 格里 40 格；它们的共同点是 F_end（3 到 14 槽）< N_u（12 到 24 槽）。这一形不依赖队列还有没有东西：4 格队列已空的窄盘格卸载同样被拒。
- 第一个文件版本那一次发布不扣环窗口，但那次发布只能写一个数据单元：我试过 `First(N, L)`（准入判着、第一个文件一次发 L 个单元），L ≥ 100 全部被拒在 `ContentExceedsDataUnit`
  （「CELL First(750, 100) O0 U1 B0 K16 => first-file-refused ContentExceedsDataUnit { bytes: 3230866, capacity: 32634 }」（`research/prompts/m3-unmount-after-short-continuation-r1-sonnet-model/outputs/first-i1.out:5`））。
  所以「第一个文件不扣环窗口」不能用来在窄盘上堆出大文件，长大靠的普通写要扣环窗口。
- 推翻这条判定的现象：任何一段用户历史（准入一直判着、没用只供测试的开关），在续做停下时每块盘没挡的槽少于 N_u。最可能的出处在我没量的三处：
  ① 大盘上被删文件散在分配记录树很多片叶上、take_real 逼近 W（我的盘最多 3 片叶，W 里 min 项几乎不起作用）；② 崩溃造出的「被抛弃根独占量」叠上走空（我的崩溃都是整条根落盘的，没丢过根）；
  ③ 一次挂载里续做被打断多次（我没注入续做中间的崩溃）。这三处归攻方腿去打，我只标「没量」。
- 按本轮判据第 1 条：我这一条腿「只在环窗口装不下的窄盘上造得出这一形」。

## 2　Q2　候选：卸载怎么做成

### 2.0　代码上卸载为什么做不成，什么让它做成

- 卸载就是准入抬 F 那一串换了入口：`unmount`（`crates/singlefs-core/src/mount.rs:2171`）调 `raise_the_floor_through`；那里先把释放代 ≤ 新 F 的槽回收并扣住（「ReclaimedReuse::HeldUntilFloorTakesEffect,」（`crates/singlefs-core/src/mount.rs:2589`）），
  再整串预演（「let placements_rehearsed = match rehearse_the_publishes_raising_the_floor(」（`crates/singlefs-core/src/mount.rs:2599`）），预演过了才「match write_the_raised_floor_into_every_system_configuration(」（`crates/singlefs-core/src/mount.rs:2623`）。
  所以卸载需要 N_u 个**此刻就没挡**的槽；回收来的帮不上它自己。
- 卸载抬到现行 txg、不判上限：「卸载抬 F」（D16 已定项 1「抬 F 的上限」那一行）比续做里推的准入抬 F（min(最新持久有效根，第 4 新的非空根)）高，回收得多。
  这就是为什么「卸载做成」不只是收尾：它做成之后，下一次挂载重建分配器就按新 F 回收，续做能接着走（见下面甲、乙新的循环数据）。
- 判据第 2 条「一条候选在 Q1 任一格上卸载仍做不成 ⇒ 出局」。我对每条候选按 Q1 的格报；甲有 1 格做不成，我照实写，不替主 agent 判出局。

### 2.1　甲　删除类发布给卸载留量

**改 `crates/` 哪几处**（原型就是第 2 项的三行，模型目录里的 `transaction-rs-reserve-prototype.patch`）：
1. `crates/singlefs-core/src/admission.rs`，挨着「pub fn units_of_a_publish_to_land(」（`crates/singlefs-core/src/admission.rs:764`）加一个纯函数：卸载那一串最坏要的槽 = `ROOT_RING_REGIONS` × ckpt_cost。
   次数的来源与「fn txgs_of_the_publishes_carrying_the_floor_to_every_device(」（`crates/singlefs-core/src/mount.rs:2714`）同一个常数（至多 3 次）。
2. `crates/singlefs-core/src/transaction.rs` 的「pub fn prepare_the_version_publish<Device: BlockDevice>(」（`crates/singlefs-core/src/transaction.rs:6208`）里落点那一判（6232 起）：
   `deletion_and_lineage.step` 是意图 / 批 / 最后一批时，把这个数加进 `UnitsOfAPublishToLand::commit_generated_slots` 再交 `admit_the_units_landing_on_every_device`。意图、会话走空、挂载续做三处共用这一个函数，所以三处一起变。
3. 用例：「fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_with_the_queue_left_and_its_publishes_on_disk(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:440`）
   今天断言「卸载在预演里拒」，要改成断言卸载做成；它造 Short 的那块窄盘在甲之下多半会走空（数据见下），得换一种造法（把没挡的槽再标成已分配）才能继续钉 `StillShortAfterTheFloorRaises` 这一支；
   `crates/mutations.tsv:2133` 有一行点名这条用例作判官，改名要同步。

**每格做不做得成**（拷贝上的原型，预留取 3 × ckpt_cost，窄盘格 U ∈ {20, 40}，192 / 256 / 320 / 384 槽，O ∈ {0, 4}，B ∈ {0, 2}，K = 1，外加 160 槽一格）：

```
$ for b in base1 c3 r24 r18 r12; do … python3 ../summarize.py $b-*.out | awk '…'; done      # base1 = 原代码；c3 = 预留 3×ckpt_cost；r24 / r18 / r12 = 预留固定 24 / 18 / 12 槽
base1 cells 32 Short 25 unmount-ok 6 unmount-ERR 25 mount/other 1
c3 cells 33 Short 4 unmount-ok 31 unmount-ERR 1 mount/other 1
r24 cells 28 Short 2 unmount-ok 28 unmount-ERR 0 mount/other 0
r18 cells 14 Short 2 unmount-ok 13 unmount-ERR 1 mount/other 0
r12 cells 14 Short 3 unmount-ok 11 unmount-ERR 3 mount/other 0
```

- 做不成的 1 格：192 槽、O = 4、U = 40、B = 0（`outputs/c3-n192.out`）：开删之前每块盘没挡的槽只有 38，意图之后 25，挂载自己的写行与暖机又把它吃到 11（`steps=0`），已经低于 N_u；甲只管删除类发布，管不到这一段。
  另 1 格（同盘 B = 2）连可写挂载都被取号之前的预演拒（`PlacementRefusedBeforeAcquisition…`），没有会话，谈不上卸载。
- 预留取固定数时：12 槽不够（3 格仍被拒），18 槽差 1 格，24 槽全过；按 3 × ckpt_cost（27 到 45 槽，ckpt_cost 在这些盘上 9 到 15，`outputs/ck-*.out` 的 `ckpt_cost=`）取，比量到的 N_u（12 到 24）宽，是安全侧。
- 甲之下仍 Short 的 4 格，3 格停因 `AtCeiling`（抬 F 已到上限）、卸载做成：`um-cycles`（卸载、再挂、再卸载…）在 2 到 4 轮内续做走空，每轮卸载后池级 checker 全绿
  （`outputs/jchk-*.out`：`um-cycles=u[green],3:S,u[green],3:S,u[green],3:S,u[green],29:E` 一类）。这一点是甲的循环：每次卸载抬到现行 txg，下一次挂载回收得动。
- 160 槽 O = 0、U = 40 一格（`outputs/d24-d160.out`，预留固定 24）：卸载每轮都做成，但 8 轮里续做每轮 0 步（`um-cycles=…0:S,0:S,0:S,…`），走不动——盘小到挂载自己的写行与暖机一轮就吃光卸载放出来的槽。
  这种盘上甲保证「卸载做成」，不保证「队列走得空」。

**对续做的影响**：甲让被拒发生在还剩 ≥ N_u 个槽的时候，于是为这次被拒推的抬 F 那一串能真跑。同一批格上队列走空的格数从 6 / 32 升到 28 / 33，续做从停在 0 到 19 步变成走完（例：256 槽 O0 U40 B0 原代码 9 步停、队列留 1、`free` 14；甲之下 40 步走完、推了 5 串、`free` 164；
320 槽 O4 U40 B0 原代码 13 步停、`free` 5；甲之下 40 步、推 3 串、`free` 198；`outputs/base1-*.out` 与 `outputs/c3-*.out`）。

**对会话里用户改动的影响**：只有 unlink 的意图与走空的批（删除类发布）提前被拒，提前量 = 预留 = 3 × ckpt_cost 槽 / 盘（探针盘 27 到 45 槽，约 0.4 到 0.7 MiB）；
ckpt_cost 的上界随单元区线性涨，源码注释写两块 4 GiB 盘 529 个节点（「代价随单元区线性涨（两块 4 GiB 盘 529 个节点）」（`crates/singlefs-core/src/admission.rs:924`）），4 GiB 上光分配记录树那一项就 529，预留至少约 1600 槽（ckpt_cost 另三项没算，会多一点），仍不到环窗口 25 × 368 = 9200 槽的两成（推的）。
普通写、`df`、准入式子一字不动（预留不是常驻扣项，只出现在删除类发布的落点判里）。补足小盘的回归格上不红（37 格里 6 格是准入先拒，其余 31 格走空 + 卸载成，`outputs/jreg-*.out`）。

**条款上站不站得住**：站得住。D28 已定项 1 本来就写删除类发布「落点那一判照判」（`.claude/kb/decisions/28-挂载期承诺量.md:36`），甲只是把这一判的需求多加一项；不碰式子、不碰 `df`、不动格式。
推翻条件：式子判着的盘上出现任何一格甲的预留把一次本该放行的 unlink 意图拒掉（预留 ≤ 45 槽，式子判着的边缘格没挡的槽约 930，大 20 倍）；或窄盘格里出现 N_u 超过 3 × ckpt_cost 的卸载。

### 2.2　乙新　卸载预演被拒时只把新 F 写进系统配置（我提的新候选，零轮）

乙说的是「一个字节不写」；这条是它的另一种：**预演被拒时仍做 SysPre 那一步（每块盘系统配置写新 F、过一道屏障），不发空发布**。系统配置槽是固定结构，写它不要落点，所以零槽需求。

**条款依据**（为什么 F 这样就生效了）：F_生效取三项的最大，其中一项是系统配置：「F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)」（`.claude/kb/decisions/16-发布语义.md:39`）；
代码在 `crates/singlefs-core/src/recovery.rs`：「.max(highest_in_the_system_configurations)」（`crates/singlefs-core/src/recovery.rs:1892`）。
下一次挂载重建分配器时回收就按这个 F_生效：「reclaim_floor(effective_floor, oldest_valid_root),」（`crates/singlefs-core/src/mount.rs:1646`），
新实例的第一条根带它：「新实例的根带的回退下界 = 恢复后生效的 F（recovery::effective_rollback_floor）」（`crates/singlefs-core/src/mount.rs:1174`），
而这条根不算 I-7.9 的抬 F 的根：「新实例的第一条根没有同实例的前一条，不算抬」（`.claude/kb/invariants.md:63`）。

**改 `crates/` 哪几处**：
1. `crates/singlefs-core/src/mount.rs` 的 `raise_the_floor_through`：预演被拒那一支（2599 起）对 `RaiseFloorEntry::Unmount` 不再「换回分配器、返回 Err」，换回分配器之后直接走 SysPre（2623 的 `write_the_raised_floor_into_every_system_configuration`），跳过发布循环，交回「没发布」。
   （写系统配置之前该核的东西——调用方参数与盘表、F 不降——这个函数前面已经核过。）
2. 「pub enum Unmounted {」（`crates/singlefs-core/src/mount.rs:2063`）加一个成员，例如 `FloorRaisedInTheSystemConfigurationOnly`，带 F 与这一步的写账。
   **代价**：`Unmounted::` 的使用者在 mount.rs 之外有 17 个文件（我数的，清单在模型目录 `outputs/unmounted-users.txt`），多数是 checker 档的崩溃枚举与装置，穷举 `match` 都要补一支，我没法跑那一档。
   不加成员、复用 `FloorRaisedToTheCurrentVersion` 而 `publishes` 为空，编译过但让「卸载一定发了空发布」的调用方（例如 `crash_enumeration_fixed_script_stream.rs`）静默变样，不推荐。
3. 用例：「fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_with_the_queue_left_and_its_publishes_on_disk(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:440`）的卸载断言改成新成员 + 再挂续做走空。

**每格做不做得成**（原代码、预留 0；`base1-*`，窄盘 192 / 256 / 320 / 384 槽，K = 1）：

```
$ grep -h '^CELL' base1-*.out | grep 'outcome=Short' | grep -o 'cfgF-cycles=[^ ]*' | awk '…'
Short cells 25 cfgF cycles end Empty 25
rounds 1 12
rounds 2 7
rounds 3 4
rounds 4 2
$ grep -h '^CELL' base1-*.out | grep 'outcome=Short' | grep -o 'cfgF\[wrote=[a-z]*\]' | sort | uniq -c
     25 cfgF[wrote=true]
```

- 25 / 25 格：写系统配置成功，再挂（每轮：写新 F → 可写挂载）在 1 到 4 轮内续做走空；轮次分布 12 / 7 / 4 / 2。其中卸载本身做不成的 25 格里原本有 22 格连不卸载直接再挂都被拒。
- 192 槽 O4 U40 B0（甲做不成的那一格）：`cfgF-cycles=7:S,33:E`（`outputs/c3-n192.out`）。
- 每轮池级 checker 全绿：`outputs/chk-*.out` 4 格，每一轮写 F 之后（`w=green`）与挂载之后（`m=green`）都绿，例如 `cfgF-cycles=13:S[w=green,m=green],16:S[w=green,m=green],2:E[w=green,m=green]`。
  我没证这个 `checker_violations_on` 在这些盘面上会红（没往里塞已知的坏），所以这四格的「绿」只能当没看到红。
- 慢的一格：160 槽 O0 U40 B0（预留固定 24 的盘面，`outputs/d24-d160.out`）8 轮里续做每轮只走 1 到 7 步、没走空。盘小到每轮挂载的写行与暖机就吃掉大半放出来的槽。

**对续做的影响**：不改续做本身；卸载 + 再挂的循环取代了「推满就留队列」：每轮挂载续做走几步、再卸载，循环到队列空。
**对会话里用户改动的影响**：无（只动卸载的退路）。`df`、准入、环窗口都不动。

**条款上站不站得住**：D16 已定项 1「正常卸载」那一行字面写「推一串空发布把 F 抬到现行那一版的 txg」「这一串写的根在根记录 flags 里带卸载记号」（`.claude/kb/decisions/16-发布语义.md:42`）——这条在预演被拒的那一格偏离字面：F 抬了（经系统配置那一项），空发布与记号没有。
其余已定的东西不动：I-7.12 要的是系统配置 F 不低于同盘根上的 F，「抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根」（`.claude/kb/invariants.md:66`），这里系统配置更高、根更低，方向对。
checker 与层 0 不读系统配置里的 F 那笔欠账 C556 大半已还（checker 的回退候选集下界取 F_生效，层 0 记录核对器的回收谓词也算系统配置槽带的 F，`.claude/kb/checks-owed.md:662`），
欠的是「层 0 那一侧别处的 oracle 读不读系统配置里的 F 没有普查」（同一行末尾）；乙新要用它，这一句得先清。
`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`（checker 档，我没读它的内容，按文件名与 C556 那一行推它钉的是 SysPre 之后崩的状态）；乙新让「系统配置 F 高于所有根」这个状态在卸载之后长期存在，要补一格「卸载退路」的崩溃枚举。
推翻条件：某一格按乙新卸载之后再挂，续做走不空且 `outputs` 里池级 checker 红；或某个读 F 的 oracle 只看根上带的 F，在乙新留下的盘面（系统配置 F 高于所有根）上误红。

### 2.3　乙　卸载预演被拒时一个字节不写，交回「卸载做成、F 没抬」

**改 `crates/` 哪几处**：和乙新同一处——`raise_the_floor_through` 预演被拒那一支对卸载入口不返回 Err，交回新成员（`Unmounted` 加一支，17 个文件要补穷举，同上）——但不做 SysPre。

**每格**：「做成」是字面上做成：没有写，当然不会失败。可它之后的状态在 Q1 的格上：
- 不卸载直接再挂，Short 的 40 格全被拒（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`，取号之前的预演就取不到树表的落点），原因同上：盘上没动，挂载自己的写行与暖机就没槽。
  `base1` 那 25 格里再挂被拒 22 格、挂得上 3 格；挂得上的 3 格（384 槽，U = 40）这一次续做走空（`remount-ok(Empty)`，`outputs/base1-n384.out`；为什么这一次能走空我没查，推的：第一次挂载自己的发布把根环往前推了）。
- 所以乙在窄盘上是「挂载、走不动、卸载（空操作）、再挂载」：22 / 25 格第一步再挂就走不通，卡住；3 / 25 格自己松开。

**条款**：不满足 D16 已定项 1「正常卸载」那一行：那一行写的是推一串空发布把 F 抬到现行那一版（`.claude/kb/decisions/16-发布语义.md:42`），
射程里用户选时知情的代价是「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（`.claude/kb/decisions/16-发布语义.md:57`）。乙之下 F 不抬：
- 回退候选集不缩（仍是 4 个不同状态里还在环里的那些），崩溃兜底比条款承诺的「卸载之后」更宽；这是好处，也是偏离。
- 续做的空间不会因卸载而多：下一次挂载的 F_生效 = 上一次的，回收得到的槽同上一次；循环在 22 / 25 格上不会自己松开。
站得住的只有一种读法：把乙限定在「预演被拒」这一格，并承认这一格卸载不换来空间。我不推荐把它单独当答案；它作为乙新的对照臂有用。
推翻条件：没法单靠量推翻「乙等于没抬 F」；能推翻「乙卡住」的是 22 格里任何一格不靠 F 就再挂成、续做走空（3 格自己松开的已在上面）。

### 2.4　丙　腿提的别的形态

**丙·式子**　把卸载那一串的最坏量并进环窗口或保留池的式子（`ring_window_on_one_device`，`crates/singlefs-core/src/admission.rs:383`；保留池见 D28 已定项 4）。
在 Q1 找到的格上无效：那些盘上准入被关（探针）或根本放不过一个写（环窗口 650 到 750 槽 > 单元区），式子里多一项不改变这些盘的历史。在式子判着的盘上本来没出这一形。
按判据第 2 条，Q1 找到的每一格都仍做不成，出局。顺带：并进环窗口只让准入多扣；并进保留池才让 `df` 少报（D16 已定项 1 的 `df` 行有保留池这一项），用户可见。

**丙·mkfs 拒这类几何**（判据第 1 条触发的那一条）：
- 改 `crates/singlefs-core/src/make_filesystem.rs` 的 `check_geometry`（`crates/singlefs-core/src/make_filesystem.rs:352`，今天最后一类容量判据是「if ring_bytes > smallest / 4 {」（`crates/singlefs-core/src/make_filesystem.rs:392`）），
  多拒一类：单元区槽数 < 环窗口 + R0 + 一次写的需求。kb 里这条已经是欠账 C628：「mkfs 的盘容量判据（`crates/singlefs-core/src/make_filesystem.rs` 的 `UnitAreaBeyondDevice`、`JournalRingTooLargeForDevice` 这几样）只判固定结构装不装得下，不判第一次可写挂载的准入……过不过得了」（`.claude/kb/checks-owed.md:490`），
  那一行已写「mkfs 的最小盘判据与准入的扣减项共用一处常量」。`ring_window_on_one_device` 今天要一个 `PoolAllocator`，mkfs 时还没有，得拆一个只读几何的纯函数。
- 阈值（S = 8 的探针盘）：单元区 750 槽起第一个文件放得进，900 槽起一次覆盖写放得进（`outputs/first-file-threshold.out`）；Short 在 ≤ 384 槽。mkfs 的底线取「式子在第一次覆盖写上放行」约 900 槽（单元区约 14 MiB / 盘）是一个可对的点，窄盘档位的真实值要现算（环窗口随 N_ℓ 变，N_ℓ 随单元区变，是个不动点）。
- 每格：Q1 里 N ≤ 384 的窄盘 mkfs 直接拒，Short 格不再存在，卸载问题在这些盘上不出现。
- 用例代价：用不补足窄盘的助手有 3 个文件——`crates/singlefs-harness/tests/common_admission/mod.rs`、`a_floor_raise_refused_for_space_counts_as_short_of_space.rs`、`admission_raises_the_floor_before_refusing.rs`
  （`grep -rln 'start_on_narrow_devices_without_…switched_off'` 命中三份，我数的）；它们要么改用补足的盘，要么给 mkfs 一个只供测试的绕过开关（`.claude/rules/fs-design.md` 的做法）。
  随机历史里关准入的取样点用的是补足档（`crates/singlefs-harness/src/history.rs:5914` 一带，我没逐个核）。
- 这一条不改任何运行时行为：它只是把「卸载在这一形上做不成」从「要处理」变成「造不出来」。代价是 mkfs 拒了一批今天收下的盘（≤ 约 900 槽的单元区，即几十 MiB 以下的盘）。
- 推翻条件：找到一格式子判着、单元区 ≥ mkfs 新底线的盘上，续做停下时没挡的槽 < N_u（即 Q1 的推翻条件）；有就说明这条拦不住。

**丙·卸载一律只写系统配置**（乙新的强形态：不预演、不发空发布，永远只做 SysPre）：不依赖盘上有没有槽，Q1 的每一格都做得成；但它改掉了所有正常卸载的行为——
卸载记号（D22 已定项 7 flags 位 0）不再有人写，I-7.9 的「带卸载记号那一支」与它的坏镜像、两条变异、C557 的检查成了死代码，卸载 2 或 3 次空发布的写放大数（D16 射程里的 8 个单元 / 279040 字节）全部作废。
改的东西比乙新多一个量级，而乙新已经够；除非主 agent 想把「卸载写不写根」整个重新定，不建议。（零轮）

### 2.5　一张对照表（我量的格上；「做成」= 卸载返回成功且之后能写）

| 候选 | 窄盘 Short 25 格（base1）卸载 | 再挂能续做走空 | 条款偏离 | `crates/` 改动面 |
|---|---|---|---|---|
| 原代码 | 32 格里 6 格成（都是队列已空的格）；Short 的 25 格全被拒 | 22 / 25 再挂被拒，3 格自己走空 | — | — |
| 甲 | 31 / 33 格成（含原 Short 25 格里 24 格）；1 格不成、1 格没会话 | 循环 2 到 4 轮走空；160 槽 U40 格走不空 | D28 已定项 1 一句 | admission.rs、transaction.rs 各一处；1 条用例换造法 |
| 乙新 | 零槽需求；25 / 25 写成功 | 25 / 25 在 1 到 4 轮内走空 | D16 已定项 1 两行 | mount.rs 两处 + `Unmounted` 加成员（17 个文件）；要补 C556 欠的那一句 |
| 乙 | 「做成」但 F 不抬 | 同原代码：22 / 25 再挂被拒，3 格自己走空 | D16 已定项 1 一行 + 射程 | 同乙新的 `Unmounted` 面 |
| 丙·式子 | 不改变这些盘 | — | D28 已定项 1 | — |
| 丙·mkfs | 造不出这些盘 | — | 新增一条 mkfs 几何句 | make_filesystem.rs；3 个用例文件换盘 |

甲与乙新互补（零轮）：甲在删除途中就留量，让多数格走空；乙新兜住甲剩下的那 1 格（192 槽 O4 U40 B0：甲做不成，乙新 `7:S,33:E`）。
两条不共用同一段代码、同一次采样。

## 3　Q4　条款要补的句子（Q3 代价数不归这条腿，没做）（只写改哪一句、改成什么意思；行号是 kb 文件自己的行号，现查）

**甲**
- D28 已定项 1「接线」那条：「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判」（`.claude/kb/decisions/28-挂载期承诺量.md:36`）。
  加半句：删除类发布的落点那一判另要求发布之后每块盘没挡的槽仍不少于卸载那一串最坏要的槽数（`ROOT_RING_REGIONS` × ckpt_cost，每块盘），不够按空间不够拒，会话与挂载续做照旧推抬 F 再试。
  同一条的「环窗口」括注（`.claude/kb/decisions/28-挂载期承诺量.md:29`）不改，只在射程里写明这份留量不是常驻扣项、不进 `df`、不进式子。
- D8 已定项 16 第 6 条两句（同在 `.claude/kb/decisions/08-核心索引结构.md:463`）：「会话走空时删除类发布（意图那次与分批的每一批）被落点那一判以空间拒，推一串抬 F、回收后再试」里的「落点那一判」要注明含卸载留量；
  「推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常（与 D16（发布语义） 已定项 1「准入」那一行同一定案）」后面补：留量之下正常卸载在这一形上做得成；挂载自己的写行与暖机在续做之前就把槽吃到卸载那一串要的数以下的盘，留量管不到，不保证（我量到 1 格）。
- D16 已定项 1「正常卸载」行（`.claude/kb/decisions/16-发布语义.md:42`）：加一句指回 D28：续做推满仍放不下的挂载上，卸载那一串的槽由删除类发布的落点那一判留着。「准入」行（`.claude/kb/decisions/16-发布语义.md:43`）的「挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常」不改。

**乙新**
- D16 已定项 1「正常卸载」行（`.claude/kb/decisions/16-发布语义.md:42`）：「推一串空发布把 F 抬到现行那一版的 txg」后面加例外：这一串在预演里被拒（取不到落点）时，改为只把新 F 写进每块盘的系统配置、过一道屏障，不发空发布、不带卸载记号，交回新成员；
  F_生效 仍到现行那一版的 txg（经「生效」行里系统配置那一项，`.claude/kb/decisions/16-发布语义.md:39`），下次挂载照旧走恢复。
- 「抬 F 那一串」行（`.claude/kb/decisions/16-发布语义.md:41`）：「空发布推到每块盘上都有一条带新 F 的根为止」加「卸载入口的预演被拒除外」。
- 射程「正常卸载的代价」（`.claude/kb/decisions/16-发布语义.md:57`）：「每次卸载多 2 或 3 次空发布」后加：预演被拒的盘上这一次不发；「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」仍成立，不改。
- D8 已定项 16 第 6 条那句「正常卸载照常」（`.claude/kb/decisions/08-核心索引结构.md:463`）：后面注「（预演被拒时走只写系统配置的退路，D16 已定项 1）」。
- `.claude/kb/checks-owed.md:662`（C556）：「层 0 那一侧别处的 oracle 读不读系统配置里的 F 没有普查」得先清；新欠一条「卸载退路的崩溃枚举」。invariants.md 的 I-7.9 / I-7.12 两行不改。

**乙**
- D16 已定项 1「正常卸载」行（`.claude/kb/decisions/16-发布语义.md:42`）加例外：预演被拒时一个字节不写，F 不抬，交回新成员；射程「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（`.claude/kb/decisions/16-发布语义.md:57`）要改成「预演被拒的盘除外」；
  D8 已定项 16 第 6 条「正常卸载照常」（`.claude/kb/decisions/08-核心索引结构.md:463`）改成「正常卸载做成，预演被拒时 F 不抬」。
  还要明写：这一格下一次挂载可能被取号之前的预演拒（量到 22 / 25），管理员要腾空间只能回退或读写以外的办法。

**丙·mkfs**
- 新写一句 mkfs 几何判据：单元区槽数不得少于环窗口 + 保留池 + 切换预留 + 一次写的需求（按 D28 已定项 1 的式子逐项现算）。kb 里 mkfs 盘容量判据没有已定项承载，只在欠账里：`.claude/kb/checks-owed.md:490`（C628），
  放进 D28 已定项 1 还是 D3 已定项 10 ④ 一带（`.claude/kb/decisions/03-空间分配.md:204` 管单元区起点）是个没定的位置，行号待查。
- D8 已定项 16 第 6 条与 D16 已定项 1「准入」行里「推满仍放不下时挂载照样交回可写」两处不改，但射程要写「在 mkfs 接受的几何上量到 0 格」。C628 由「先定改法」改成已定。

**丙·式子**：出局，不写。**丙·卸载一律只写系统配置**：要改 D16 已定项 1「正常卸载」「抬 F 那一串」两行、D22 已定项 7 的卸载记号位、invariants.md 的 I-7.9 带记号那一支与 C557，范围远大于 Q4 要的，只记名字。

## 4　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 补足环窗口的小盘（式子判着） | 一致（没造出） | 45 + 2 + 31 格全部走空 + 卸载成；一次抽样，三处没量（见 1.4） |
| Q1 不补足的窄盘（关准入） | 一致（造得出） | 108 格 40 格 Short；停因 40 / 40 是抬 F 那一串被空间拒；卸载 44 格被拒（含 4 格队列已空）；缺的槽估 0 到 6 槽 / 盘 |
| Q1 几何轴 | 一致 | 第一个文件 750 槽起放得进、覆盖写 900 槽起；Short 在 ≤ 384 槽，≥ 416 的 63 格无 |
| Q2 甲 | 一致（条款上站得住，改 D28 一句） | 33 格卸载成 31；做不成的 1 格是进续做之前槽就没了；队列走空的格 6 / 32 → 28 / 33；用户改动提前 3 × ckpt_cost 槽 |
| Q2 乙新（新） | 冲突，但 F_生效 的定义已含系统配置项 | 左：「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`）；右：乙新——预演被拒时只写系统配置、不发空发布（F_生效 经「生效」行仍到现行那一版）。25 / 25 在 ≤ 4 轮内走空，零槽需求；要补 C556 末句与崩溃枚举；`Unmounted` 加成员波及 17 个文件 |
| Q2 乙 | 冲突 | 左：「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`）；右：乙的行为——预演被拒时一个字节不写、F 不抬。25 格里 22 格再挂被拒 |
| Q2 丙·式子 | 一致但无效 | 在 Q1 找到的格上不改变这些盘的历史 |
| Q2 丙·mkfs | 规则没说 | kb 里没有 mkfs 盘容量判据的已定项，只有欠账 C628；动 `check_geometry` 一处 + 3 个用例文件 |
| Q4 | 一致 | 每条候选要改的句子与行号见第 4 节 |

## 5　没做什么

- 没跑 checker 档，没在入库装置上重做：副本上的数不进 kb（`.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）。探针和原型都在副本里，模型目录里只有探针源码、补丁、脚本和原样输出。
- 没量：① 大盘上被删文件散在许多片叶里的 take_real（我的盘最多 3 片叶）；② 丢根的崩溃（被抛弃根独占量叠上走空）；③ 续做中间被打断多次的历史；
  ④ 用「多给 δ 个槽」扫出精确的缺槽数（机器负载 50 到 140，一趟 4 分钟只出 0 格，放弃）；⑤ 补足盘上 K = 144 的格（只在 1800 / 2200 槽窄盘上跑了 K = 144）；
  ⑥ 160 / 176 槽更窄盘上乙新的轮数（只有预留 24 的 160 槽 U40 一格，8 轮没走空）。
- 没证 `checker_violations_on` 在这些盘面上会红，所以乙新、甲的「checker 全绿」只当没看到红。
- 没试编 `Unmounted` 加成员后 17 个文件的补法；没读 checker 档那些文件的内容（计数来自 grep）。
- 不替主 agent 判出局、不选候选；第 2 条判据在甲上的 1 格、乙新 / 乙 / 丙·式子上的结果我只报。
- 草稿：`/tmp/claude-1000/m3-unmount-forward/` 里 `out/` 与各 `*.out`、`lines-*.txt` 是跑出来的原样输出（已拷进模型目录 `outputs/`，`pad.out`、`edge384.out` 因整段 Debug 太长拷成了截断版，行首的字段是全的）；
  副本 `tree/`（含 `target/`）是我建的编译目录与仓副本，交回前已删。
