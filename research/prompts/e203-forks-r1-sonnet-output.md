# e203-forks-r1 正推腿报告（Sonnet，2026-11-07）

范围：正文第五节四格。方案按 `crates/` 今天的实现来谈，具体读的是冻结源树 `pristine-source`（下文路径一律写成 `pristine-source/crates/singlefs-core/src/<文件>:<行>`，根是 `~/.local/share/singlefs/work/e203-runner-segment-2-r2/`；`diff -u` 还原后 sha256 我核过，`cd pristine-source && sha256sum -c --quiet ../pristine-instrumented-files.sha256` 退 0、打 `SHA_OK`）。

没写模型。唯一拷来改过的东西是给模型多打一段只读数的补丁 `~/.local/share/singlefs/work/e203-forks-r1-sonnet/patch/q6-rows.patch`（sha256 `b43f53e5af4c6f06aeea5ab34ee1a25221cbd26084e9e5e891e7a8d2725576ec`，24 行）与起它的脚本 `.../tools/q6-rows-run.sh`（sha256 `2ac05d67770df34be47790e80cb527cb0a1241c5b2d203197d50a240671c0f92`），都没编译、没跑，见「要主 agent 跑的命令」。

引文规矩：kb、规则、登记、模型的引文都先 `grep -nF` 过一次，行号是那份文件自己的；冻结源树与产物行的引文同样。凡写「手算」「awk 汇总」的数，是我读产物行自己加的，当「推的」，不当模型输出。

## 一页结论

| 格 | 结论 |
|---|---|
| 1 B1 | 把 X2 写成 E203 的 Q4 判定或 F1 打中：与登记冲突（把「不判」放宽成「判」）。把 X2 当本轮对岔路的下界推论、E203 的 F1 仍记 unknown：不改登记判据，登记没写这条路。这条推论要的单调前提，只对整份拷分配器那一行成立且只需要「U 部分长度只由 U 定」（冻结源树逐行核到）；对其余扫类各行，「步数随记录数不减」代码不保证、实测有反例。 |
| 2 B8 | 正文说的三行是 `q6_top_rows` 那一行，那一行挑的是 U 最大的点（X2），不是 D、F 两轴的高端点。D、F 高端点上贡献最大的是 `mapped_units_whole_table` 一行（D 轴 99.2%、F 轴同挂载 97.5%）。这一行是登记自己「做法」与「式子」两句不相容：做法说整表换成差量，式子吃的是 B1 量出的全表查询次数，模型照式子写，没漏。另两行（`allocator_release_linear_find`、`record_intersection_filter`）是 B8 定义留着的对数项，F 轴同挂载上靠固定点 22 轮的乘数涨。B8 有收严形态可出，记零轮。 |
| 3 B4 | 「两道之间不在运行时决策路径之内」这个读法站不住：两道之间那一段是式子判自己的输入（`settled.resolved` 喂给 `admit_on_every_device`），里面有分配与生命周期判定。B4 只能是改条款（`.claude/rules/fs-design.md:23` 与 D8 已定项 16 射程两处），不是读法。 |
| 4 候选 | 五个：B9 去容量项、B10 分块共享拷贝、B11 B8 收严形态、B12 留 B1 加明示例外行、B13 例外行加 B11 其余项。每个的定义、翻面观测、要登记的实验在第四节。 |

## 背景材料里先改的几处

1. 正文第五节第 2 格与题面「贡献最大的三行」：来自 `name=q6_top_rows point=user_scale_unit_area`（日志 `/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-9595.attempt-1.log` 第 1807 行），是 X2 点上的三行；模型挑点的规则是三轴数值之和最大者，U 一项压倒其余：「// 贡献最大的三行（高端点）：B8 在三段合计里步数最大的三行，与它们吃的差量输入名。」（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2645`）、「counts.points[*point_name].unit_area_slots」（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2651`）。登记 F2 打中之后要的是「列出贡献最大的三行与它们吃的差量输入」（`research/prompts/e203-preregistration.md:614`），按轴高端点列，这份产物里没有。
2. 正文第五节第 3 格「外推到用户规模 4.3–404 秒」是 Q9（两道之间那一段）；Q4（准入之前扫类全合计）的五轮在 10.5–6155 秒（日志第 2445–2448 行），两个量别混。
3. `q8 arm=b1` 的 P0+S1 合计在 u、D、F 三轴上 α 为 0.014、0.009、0.024（日志第 1800、1803、1806 行，各带 `band=DoesNotGrow`），是被整份拷分配器那一项（基点 808k）稀释出来的，不等于「落点判之前今天已经不随记录涨」。去掉整份拷之后，`before_landing` 段各行步数之和（awk 汇总同挂载覆盖写第 1 轮，推的）基点 373、F=10⁴ 点 94586，F 轴 α 约 0.80（手算 ln(94586÷373)÷ln(1000)）；产物 `research/results/e203-admission-scan-cost-counts-base-2026-11-06-r2.out`、`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out`。

## 一、B1：拿 X2 直接读数判翻面，合不合登记的判据（问题 1）

### 1.1 登记里这一条是怎么写的（原文）

- 岔路单 B1 行（转述；原文 `git show ff5dca7d22b1:research/prompts/m4-admission-delta-forks.md`）：翻面观测只写了一个方向，用户关心规模上一次覆盖写准入之前的扫类挂钟低于 1 毫秒则倾向留 B1；够判条件是跨三个数量级的单元区与记录数上步数与挂钟钉绝对值。
- Q4 的判法：「5 个都 ≥ 2 毫秒记」（`research/prompts/e203-preregistration.md:329`）；F1：「Q4 四格（两档 D_user × 两种挂载状态）都记」（`research/prompts/e203-preregistration.md:613`）。
- F6：「Q5 任一格核不过（步数相对差 > 10%，或挂钟比出 [0.5, 2]）」（`research/prompts/e203-preregistration.md:618`），同一行：「X2 量过之后 Q4 在 U 那一部分改用 X2 的量，记录数那一部分仍外推」（`research/prompts/e203-preregistration.md:618`）。
- 修订 3：「X2 量过之后照严的读法 Q4 不判」（`research/prompts/e203-preregistration.md:680`）。
- 产物对应：日志第 2459 行 Q4 四格都是 `AtLeastOneMillisecond`、同行 `not_judged_because_the_extrapolation_is_not_trusted=true`；第 2461 行 `name=verdict clause=F1 fired=unknown`；第 2460 行 `Q5 cells_passed=2 cells_failed=8`。

### 1.2 X2 的直接读数（量的）

- 整份拷分配器一行，同一次挂载覆盖写第 1 轮：「exclusive_nanos=1420202116」（`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out:112`）；重挂载第 1 轮：「exclusive_nanos=1319344068」（`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out:865`）。其余各轮同一文件 `grep -n 'row=whole_allocator_copy_before_the_judgement' … | grep -o 'exclusive_nanos=[0-9]*'`，正文已抄全十个（1.32–1.42 秒）。
- 那一次拷的元素数（计数构建，同一点）：「segment=caller row=whole_allocator_copy_before_the_judgement calls=1 steps=6610216367」（`research/results/e203-admission-scan-cost-counts-user_scale_unit_area-2026-11-07.out:2354`）。
- 十个读数最小 1.318 秒，是 Q4 门槛 2 毫秒的约 659 倍。

### 1.3 合不合登记的判据：分三层

1. 门槛、五轮、四格这几样一个字没动：十个读数每个都 ≥ 2 毫秒，条件满足的是 Q4 判法本身。动的是量的位置：G-user 是 (U=2³⁰−1408, u=10⁶, D_user, F=10³)，X2 是 (2³⁰−1408, 10, 10, 10)，只有 U 对上。所以它不是「同一个量直接量到了」，是 G-user 的一个子形状。
2. 登记对「X2 量过之后」有明文：F6 写 U 部分改用 X2、记录部分仍外推，修订 3 再写「照严的读法 Q4 不判」。把 X2 写成 E203 自己的 Q4 判定、或把 F1 记成 fired=true，是把登记写下的「不判」放宽成「判」。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:111` 的「新形态要被原来那套判据判得更严或相同；放宽一个字都不行」，这一步不合：不合的是写进 E203 的判定，不是拿数。
3. 本轮判的是岔路，不改登记（正文第一节末段）。这一层登记没写：把 X2 当作「G-user 上整份拷分配器那一行的下界」写成本轮推论，E203 的 Q4、F1 原样留 unknown，这不改任何一个字的判据，只是多一条推论，推论自己要列前提并挂「什么现象会推翻」（1.5）。三方规则本来就要求推论走三方，不是这一轮的特例（`.claude/rules/three-way-inference.md@适用范围`）。
4. 一句附带：就算不论毫秒门槛，B1 对 `.claude/rules/fs-design.md:23` 的「**不许**，且代价不许随盘容量增长」也是红的：整份拷分配器是容量项，U 轴 α 0.998（日志第 1797 行，P0+S1 合计 102281 → 103286626；B1 计数阳性对照 `alpha=0.998444` 在日志第 2397 行）。D8 已定项 16 射程字面只写容器总数与记录总数（`.claude/kb/decisions/08-核心索引结构.md:516`），容量一词在 `.claude/rules/fs-design.md:23` 与变更史的读法句里（`.claude/kb/decisions-history.md:3543`）。所以 B1 翻面不只靠 1 毫秒这个量规；留 B1 要么靠条款明示一个例外（第四节 B12），要么靠改代码。

### 1.4 「用户规模上只会更大」靠什么成立：只对整份拷分配器那一行需要，核到的行

这一条下界推论只用 s1 一行，前提拆成三句，逐句核：

- P1 s1 的元素数里由 U 定的那部分，长度只由 U 定、挂载后不再长缩。冻结源树 `DeviceFreeMap` 八张表都是建图时按单元区槽数与段数一次定长：「allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],」（`pristine-source/crates/singlefs-core/src/allocator.rs:469`）、`used_per_segment` 与 `unreleased_per_segment` `:470`–`:471`、`isolated` `:477`、`isolated_per_segment` `:478`、`held_until_floor_takes_effect` `:480`–`:484`、`held_per_segment` `:485`、`unblocked_slot_pairs_per_segment` `:488`。逐字段的增删命令：`grep -rnE '(allocated|isolated|used_per_segment|unreleased_per_segment|isolated_per_segment|held_until_floor_takes_effect|held_per_segment|unblocked_slot_pairs_per_segment)\.(push|resize|truncate|extend|clear|pop|insert|remove|drain|retain|append)' pristine-source/crates/singlefs-core/src | wc -l` 输出 1，唯一命中是 `pristine-source/crates/singlefs-core/src/mount.rs:1912` 的局部参数 `isolated: &mut BTreeSet<(u32, u64)>`（`pristine-source/crates/singlefs-core/src/mount.rs:1909`），不是字段。表数合 3U + 5⌈U/64⌉ 每盘，与登记第四节第 7 条同；产物里 `from_the_unit_area=6610214436` 对 G-user 与 X2 是同一个数（日志 `name=copy_elements point=user_scale_unit_area`）。
- P2 整份拷是 `derive(Clone)` 一路按集合拷：`DeviceFreeMap`（「#[derive(Clone, Debug)]」，`pristine-source/crates/singlefs-core/src/allocator.rs:303`）与 `PoolAllocator`（`pristine-source/crates/singlefs-core/src/allocator.rs:1866`，其余集合 `records`、`cluster_segments`、`reclaimed` 在 `:1876`–`:1879`）；其余集合的克隆时间 ≥ 0，不会让总数小于 U 部分。
- P3 同机同构建同单线程：计时头 release、单线程（`name=verdict clause=void_7 … timing_header_is_release_opt_level_3_single_thread=true`，日志第 2444 行）；G-user 的 U 部分与 X2 同一个 U，所以不是外推，是同一块内存同一个 `Vec<bool>`。

三句成立 ⇒ T_s1(G-user) ≥ T_s1(X2) ≈ 1.3 秒，且不需要「记录数越大步数越大」这条单调前提。

### 1.5 「扫类步数随记录数不减」对其余各行：核到的行与结论

登记第三节标「扫」的其余各行，我读了循环体的按下面三类分；类里的行号都是冻结源树的。

| 类 | 行 | 冻结源树 文件:行 | 单调？ |
|---|---|---|---|
| 全表走一遍（步数 = 被走表的长度） | s2 | `pristine-source/crates/singlefs-core/src/transaction.rs:4231`–`:4235` | 表长单调则单调；表长在这条历史族里是否单调是实测问题，代码不保证 |
| | s5 | `pristine-source/crates/singlefs-core/src/transaction.rs:7697`–`:7719` | 同上（别的文件数） |
| | s7 | `pristine-source/crates/singlefs-core/src/transaction.rs:4116`–`:4130` | 同上 |
| | s9、s17 | `pristine-source/crates/singlefs-core/src/transaction.rs:8627`–`:8635`、`:8960`–`:8968` | 同上（墓碑 / 谱系记录条数；墓碑第一版不回收，D18 已定项 24，单调） |
| | s11 | `pristine-source/crates/singlefs-core/src/transaction.rs:7883`–`:7889` | 同上 |
| | s13、s20 | `pristine-source/crates/singlefs-core/src/code_two_tree.rs:650`、`:656`、`:659`、`:685`–`:690` | 同上（树里 key 数） |
| | s14 | `pristine-source/crates/singlefs-core/src/allocator.rs:578`–`:590` | 随聚簇段数；重挂载后从空起，跨挂载状态不单调 |
| | s15、s16、s30 | `pristine-source/crates/singlefs-core/src/transaction.rs:13346`、`:13350`；`pristine-source/crates/singlefs-core/src/allocation_record_tree.rs:316`、`:372`–`:373`、`:434`–`:437` | 同上（记录条数、节点数） |
| | s18 | `pristine-source/crates/singlefs-core/src/transaction.rs:4271`–`:4280` | 过滤全表 + 排序，单调；尾部有一个 `find` 早退 |
| | s21、s22、s32 | `pristine-source/crates/singlefs-core/src/transaction.rs:6949`、`:9514`–`:9518`、`:4728`–`:4731` | 同上 |
| | s36 | `pristine-source/crates/singlefs-core/src/transaction.rs:9209`、`:9250`–`:9262`、`:9425` | 同上（映射条目数 M） |
| 走到命中为止（步数 = 命中位置，被表长从上封顶） | s4、s6、s12 | `pristine-source/crates/singlefs-core/src/transaction.rs:4203`–`:4206`、`:4195`–`:4197`、`:4375`–`:4377` | 代码不保证 |
| | s25 | `pristine-source/crates/singlefs-core/src/allocator.rs:2209`–`:2211`、`:2262`–`:2265` | 代码不保证 |
| | s26、s28、s29 | `pristine-source/crates/singlefs-core/src/allocator.rs:973`–`:987`、`:1005`–`:1013`、`:1042`–`:1051` | 代码不保证（命中位置随空槽在哪） |
| | s27 的 `find` 半 | `pristine-source/crates/singlefs-core/src/allocator.rs:1172`–`:1175` | 同上；`filter` 与 `retain` 两半是全表（`:1152`–`:1160`、`:1182`） |
| 混合 | s8 | `pristine-source/crates/singlefs-core/src/extent_tree.rs:231`、`:239`–`:245` | 上段位置数 × 上一版节点数（切片 `contains` 线性），单调于上段节点数 |
| 随这一次碰到的角色数 | s33（阴性对照） | `pristine-source/crates/singlefs-core/src/admission.rs:1035`–`:1038` | 不随总数 |
| 只读到入口、没读循环体 | s3、s10、s19、s23 | 锚点 `pristine-source/crates/singlefs-core/src/transaction.rs:8387`、（s10 同 s9 的 `:8672` 一带）、`:6578`、`:6706`、`:4635`–`:4640` | 未核 |

表长本身代码不保证单调，三处能缩：「records.retain(|record| !(record.device == device && record.slot == record_slot));」（`pristine-source/crates/singlefs-core/src/allocator.rs:1182`）、`reclaimed.remove` 在 `pristine-source/crates/singlefs-core/src/allocator.rs:1164`、重挂载后 `cluster_segments` 从空起（`pristine-source/crates/singlefs-core/src/allocator.rs:1872`–`:1876` 的注释）。

实测里的反例（awk 汇总，推的；同挂载覆盖写第 1 轮，三段求和；命令见「要主 agent 跑的命令」第 3 条，输出原样）：

```
unlinks_1 same_mount: release_find=40872 user_slot=480 empty_seg=0 root_hdr=182 rec_inter=24612
base same_mount: release_find=58180 user_slot=80 empty_seg=0 root_hdr=182 rec_inter=52296
unlinks_100 same_mount: release_find=53676 user_slot=88 empty_seg=0 root_hdr=182 rec_inter=85172
```

u 轴 1 → 10 上 `user_data_slot_linear_scan` 480 → 80（降），10 → 100 上 `allocator_release_linear_find` 58180 → 53676（降）。同一点同一挂载五轮里整份拷的元素数 821753、821741、821735、821735、821739 而分配记录数 7792 → 7806 在涨（`research/results/e203-admission-scan-cost-counts-unlinks_10000-2026-11-06-r2.out:509`、`:728`、`:939`、`:1439`），表长合计也不是单调的。

**结论（B1）**：作为 E203 判定：冲突，不可写；作为本轮推论（下界）：登记没写这条路，不改判据，前提 P1–P3 核到，不依赖「步数随记录数不减」；「步数随记录数不减」对其余扫类各行，代码不保证、u 轴实测有降的格，不能当前提用。

**什么现象会推翻**：(a) 冻结源树里 `DeviceFreeMap` 那八张表在挂载之后有字段被增删（上面的 grep 命中字段）；(b) 在 U = 2³⁰−1408 的形状上量到整份拷分配器一行 < 1 毫秒（把 `allocator.clone()` 换成别的实现，或量的是另一块内存）；(c) 计时的 Q5 外的另一次 X2 复量读数与这十个差一个数量级。

## 二、B8：F2 在 D、F 两轴打中，贡献最大的行落在 B8 定义的哪一项（问题 2）

### 2.1 先说清是哪个点上的三行

`name=q6_top_rows … rows=allocator_release_linear_find:506,mapped_units_whole_table:376,record_intersection_filter:354` 是 X2 点（日志第 1807 行）；挑点规则见「背景材料里先改的几处」第 1 条。D、F 两轴 F2 打中的是 `name=q6 axis=D … high_end=429120`（日志第 1801 行）与 `name=q6 axis=F … high_end=3213078`（日志第 1804 行），高端点是 `data_units_10000`、`files_10000`。下面按这两个点自己的行读（量的是产物行，加总与 α 是手算，推的）。

### 2.2 D、F 高端点上各行的 B8 贡献

B8 式子取自模型：s36 `input.mapped_units_changed_over_all_rounds + input.mapped_units_query_levels,`（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:310`）；s25 `input.calls_in_the_row("allocator_release_linear_find") * lg(input.allocation_records),`（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:276`）；s27 `input.calls_in_the_row("record_intersection_filter") * lg(input.allocation_records)`（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:285`）。

| 点（同挂载覆盖写第 1 轮） | s36 `mapped_units_whole_table` | s25 `allocator_release_linear_find` | s27 `record_intersection_filter` | B8 合计 q6 的 high_end |
|---|---|---|---|---|
| D=10⁴ | 查询层数 425656（「queries=30404 query_levels=425656」，`research/results/e203-admission-scan-cost-counts-data_units_10000-2026-11-06.out:162`）| 86 次 × lg(22144)=15 → 1290（「calls=86 steps=3178170」，`research/results/e203-admission-scan-cost-counts-data_units_10000-2026-11-06.out:157`）| 36 次 × 15 = 540 加相交条数 | 429120，s36 占 99.2% |
| F=10⁴ | 查询层数 3132794（「queries=223771 query_levels=3132794」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:163`）| 2402 次 × lg(47624)=16 → 38432（「calls=2402 steps=115233774」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:157`）| 168 次 × 16 = 2688 加 152 | 3213078，s36 占 97.5% |
| F=10⁴ 重挂载 | 285894（「queries=20421 query_levels=285894」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:4124`）| 58 次（`calls=58`，同文件）| 38 次 | 288728（日志第 1843 行），s36 占 99.0% |

F=10⁴ 同挂载的次大项：s25 的 38432，释放判定查询层数 15890（「queries=2336 query_levels=15890」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:155`），s30 的 `dirty` 之和 5460 一带。要全表请跑「要主 agent 跑的命令」第 1 条的补丁（每点每行一行 `name=q6_rows`），腿这里只给手算到的几行。

### 2.3 去掉 s36 查询项之后，轴上的 α 还中不中（手算，推的）

low_end 取日志 q6 行（1748、1748、1874），high_end 减去上表 s36 查询层数；α = ln(high ÷ low) ÷ ln(1000)。

| 轴、状态 | high_end 减 s36 查询层数 | α | 登记档（≥0.5 涨、≤0.25 不涨） |
|---|---|---|---|
| D 同挂载 | 429120 − 425656 = 3464 | 0.099 | 不涨 |
| F 同挂载 | 3213078 − 3132794 = 80284 | 0.554 | 仍涨 |
| F 重挂载 | 288728 − 285894 = 2834 | 0.060 | 不涨 |

F 同挂载剩下的涨，靠的是固定点轮数乘数：该点 `fixed_point_rounds=22`（「fixed_point_rounds=22」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:1044`），重挂载 2（`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:4304`）；s25 的 2402 次调用 ≈ 22 轮 × 约 109 次。登记写死 B8 不动这个乘数：「B8 不改调用结构，调用次数取 B1 量的」（`research/prompts/e203-preregistration.md:215`）。同一份产物里其余点的轮数（命令见第 4 条）：除 `files_10000` 同挂载 22、`unlinks_10000` 同挂载 6、`diagonal` 同挂载 5 之外都在 2–4。

### 2.4 每一行落在 B8 定义的哪一项：模型漏了，还是定义留着

岔路单 B8 的定义（转述；原文 `git show ff5dca7d22b1:research/prompts/m4-admission-delta-forks.md`）：B7 的六件，加调用方那一份分配器拷贝改撤回日志或差量视图，释放判定索引随一版增量维护，正推那 8 类逐项改。那 8 类是第三轮判决写的：「正推格二：改完还剩 8 类随总数涨的步」（`research/prompts/m4-admission-delta-r3-main-verification.md:23`），其中有「版本侧两张全索引」，没有「每一版把 `mapped_units` 整张重新逐项问一遍上一版的 key」这一件。登记把岔路单 B8 展开成每行的做法与式子（`research/prompts/e203-preregistration.md:247`、`:274`–`:283`）。

| 行 | B8 定义里的项 | 登记的做法 / 式子 | 冻结源树实现行 | 判 |
|---|---|---|---|---|
| s36 `mapped_units_whole_table` | 岔路单 B8 没有点名；登记补成「`mapped_units` 整表换成差量，按角色查走随一版维护的索引」（`research/prompts/e203-preregistration.md:283`）| 同一行式子「每轮 dm + 查询次数 × lg(M)」（`research/prompts/e203-preregistration.md:283`）；模型的查询项读 B1 量的全表查询：「product.query_levels_of(&at, segments, "mapped_units_whole_table");」（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2430`），dm 项「mapped_units_changed_over_all_rounds: 2 * rewritten_per_round.iter().sum::<u64>(),」（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2462`）| 对本版每个数据单元与每个别的文件的单元逐个「mapped_units.push((identity, carried(identity)));」（`pristine-source/crates/singlefs-core/src/transaction.rs:9209`，另 `:9140`、`:9260`），再「let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units」（`pristine-source/crates/singlefs-core/src/transaction.rs:9425`）| 登记自己的「做法」与「式子」两句不相容：做法说整表换成差量（查询只剩差量），式子留着 B1 的全表查询次数。模型照式子写，没漏；漏的是登记没证明前一句推得出后一句（`evidence-discipline.md` 那条「臂的定义里同时写了「怎么做」和「做完会怎样」」）。|
| s25 `allocator_release_linear_find` | B7「取空槽与记录按索引找」 | 「分配记录按 (盘, 槽) 建有序索引」、式子 c25 × lg(R_a)（`research/prompts/e203-preregistration.md:274`）| 今天是早退线性找：「.find(|record| record.device == device.device && record.slot == placement.slot)」（`pristine-source/crates/singlefs-core/src/allocator.rs:2265`）、`record_for` 的 `.find`（`pristine-source/crates/singlefs-core/src/allocator.rs:2211`）| 定义留着：岔路单 B8 的翻面观测自己写了「（加树高）」，对数项是它允许的涨。它在 D、F 轴上贡献小；F 同挂载上靠轮数乘数涨，乘数是登记写死不动的。|
| s27 `record_intersection_filter` | 同上 | 「同 s25 的索引做区间查」、式子 c27 × lg(R_a) + ι（`research/prompts/e203-preregistration.md:276`）| 今天 `records.iter().filter(...)` 全表加 `iter_mut().find` 加 `retain`（`pristine-source/crates/singlefs-core/src/allocator.rs:1152`–`:1160`、`:1172`–`:1175`、`:1182`）| 定义留着，同上。|

答「是模型式子漏了，还是定义留着」：F2 在 D 轴与 F 轴重挂载上打中，几乎全靠 s36 一行，这一行是登记两句话不一致；在 F 轴同挂载上，去掉 s36 仍中（0.554），靠的是定义留着的轮数乘数与对数项。三行里只有 s36 是 F2 的病因，另两行只是 X2 点上碰巧排在前面。

### 2.5 B8 能不能出一个收严的新形态

登记的合法出路只有三步（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:104` 一带）：记一次输、只许收严、写明收严在哪。F2 的输已按最弱读法记在先：B8 照登记在 D、F 轴上输（日志第 1801、1804、1840、1843 行）。收严的新形态我起名 B11（原 B8 不改名；原因：不是 B8 的澄清，是多要求了两件事）：

- 多要求 1（收严在 s36）：每一版的 `mapped_units` 随版共享、只按差量改，携带的角色不再逐个向上一版索引查 key，s36 的查询次数只剩差量条目数；
- 多要求 2（收严在轮数）：固定点轮数有常数上界 k，或每轮的分配器活动只吃本轮增量。

新形态仍要被原判据（Q6：全轴、两种挂载状态、三种发布，α ≤ 0.25）判，一个字不放宽。手算：多要求 1 单独够 D 轴与 F 轴重挂载，不够 F 轴同挂载（0.554）；两件都要才可能过，所以它是个组合，被攻过零轮（`.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算` 里「腿提的收严、主 agent 判决里推的组合都算被攻过零轮」）。另有一格要留意：u 轴同挂载覆盖写 α 0.271 在「介于两档」带（日志第 1798 行），新形态要把它也压到 0.25 以下才进 Q6 的「不随」档，我没分解它由哪几行来。

**什么现象会推翻本节**：(a) 补丁打出的每点每行里，D=10⁴ 或 F=10⁴ 点上最大的一行不是 `mapped_units_whole_table`；(b) 去掉 s36 查询项后模型重算的 F 同挂载 α < 0.25（我手算 0.554）；(c) 冻结源树 `pristine-source/crates/singlefs-core/src/transaction.rs:9209` 这条循环在 `previous` 之外另有按差量走的分支（我读到的 `match` 都是逐位置循环）。

## 三、B4：把两道之间那一段读成不在运行时决策路径之内，站不站得住（问题 3）

### 3.1 条款原文

- `.claude/rules/fs-design.md:17` 小节题：「## 记账是事务的副产品，不是事后的遍历」。
- `.claude/rules/fs-design.md:23` 表里一行：「**运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定）」（`.claude/rules/fs-design.md:23`），同一行右格：「**不许**，且代价不许随盘容量增长」（`.claude/rules/fs-design.md:23`）。
- 同一节判「被禁止的」是什么：「在需要答案的那一刻还没有答案」（`.claude/rules/fs-design.md:34`）。
- D8 已定项 16 射程：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径」（`.claude/kb/decisions/08-核心索引结构.md:516`）；同一行前文把 `.claude/rules/fs-design.md:23` 那一格读成「只管准入之前」（用户 2026-10-31 定）。
- 用户那句话的原文：「管，只管准入之前」（`.claude/kb/decisions-history.md:3545`）；同一天的变更条目：「在这一格只管准入之前，准入之前的步代价不随盘上容器总数、记录总数涨，准入之后装单元、拷字节不算运行时决策路径」（`.claude/kb/decisions-history.md:3543`）。
- 前两轮已判（转述，不当证据，只作对照）：第一轮「要连两道一起算，不只第一道」（`research/prompts/m4-admission-delta-r1-main-verification.md:20`）；第二轮把 B4 判成条款变更、且要改两处（`research/prompts/m4-admission-delta-r2-main-verification.md:29`）。

### 3.2 冻结源树里两道与两道之间是什么

都在同一个函数里：「fn judge_the_publish_before_any_read_or_write<Device: BlockDevice>(」（`pristine-source/crates/singlefs-core/src/transaction.rs:12784`）。

- 落点判（`admit_the_units_landing_on_every_device`）在 `pristine-source/crates/singlefs-core/src/transaction.rs:12817`–`:12829`，注释自己叫它「准入先判这次的单元落不落得下」（`pristine-source/crates/singlefs-core/src/transaction.rs:12801`）。
- 两道之间：「let settled = settle_the_allocation_record_tree(」（`pristine-source/crates/singlefs-core/src/transaction.rs:12831`）。固定点在分配器的拷贝上取落点：「分配记录树的固定点在分配器的拷贝上走」（`pristine-source/crates/singlefs-core/src/transaction.rs:12773`）、「取不到交回 `PlacementRefused`，排在式子那一判之前」（`pristine-source/crates/singlefs-core/src/transaction.rs:12802`）。
- 式子判在 `pristine-source/crates/singlefs-core/src/transaction.rs:12907`–`:12916`，注释叫它「空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取）」（`pristine-source/crates/singlefs-core/src/transaction.rs:12882`），它的输入里有 `settled.resolved`：「&settled.resolved,」（`pristine-source/crates/singlefs-core/src/transaction.rs:12921`）；需求的定义：「需求 = 这次新写的全部角色（普通分配加固定点」（`pristine-source/crates/singlefs-core/src/transaction.rs:12883`）。
- 实现的作者自己在两道之间的步上写过「运行时决策路径」：「映射 key 集决定中央映射树重写哪几个节点，进分配」（`pristine-source/crates/singlefs-core/src/transaction.rs:6947`），按 inode 找记录的折半找：「每写一条找一次（运行时决策路径，不遍历 inode 树）」（`pristine-source/crates/singlefs-core/src/transaction.rs:4212`）。（代码注释不是条款，只说明实现的人读这条规则时把这一段算在里面。）
- 「ENOSPC 准入」指的是哪一判：D3 把「准入不等式的权威形式」挪去了 D28 已定项 1（「准入不等式的权威形式与准入的在飞合成挪去了 D28（挂载期承诺量） 已定项 1 / 2」，`.claude/kb/decisions/03-空间分配.md:5`），式子判就是那个不等式的执行点。

### 3.3 判

**这个读法站不住。** 理由四条，彼此不共用前提：

1. 文字：`.claude/rules/fs-design.md:23` 点名的四样里，两道之间有两样：分配（固定点取落点，`pristine-source/crates/singlefs-core/src/transaction.rs:12801`–`:12802`）与生命周期判定（释放判定，经 `settle_the_allocation_record_tree` 调，登记第三节 s23）。D8 射程的豁免只点「装单元、拷字节」，且限定「准入之后」；两道之间在式子判之前。
2. 机制：`.claude/rules/fs-design.md:34` 说被禁止的是「在需要答案的那一刻还没有答案」。式子判要的「需求」是两道之间的固定点算出来的（`pristine-source/crates/singlefs-core/src/transaction.rs:12883`、`:12921`）：把这一段读成不在路径里，就是把「要答案的那一刻还没有答案」那件事本身豁免了。
3. 「准入」指哪一道：用户的原话只说「准入」，没说第一道。同一个用户 2026-09-27 定的「准入先拒」把落点判也叫准入（`pristine-source/crates/singlefs-core/src/transaction.rs:12801`）。B4 要的是「准入 = 落点判」，那么式子判（ENOSPC 准入，`.claude/rules/fs-design.md:23` 点名）就落在「准入之后」，它自己是运行时决策路径，内部又有 s13 全量规划（登记第三节 s13「S1、S2（式子读数）」）。取哪一道，都得不到「两道之间不在射程」。
4. 对照：前两轮判决的结论同向（3.1 末条），但这里的理由 1–3 独立从条文与源码读出，不借它们。

式子判本身算不算「ENOSPC 准入」：算（`pristine-source/crates/singlefs-core/src/transaction.rs:12882`、D3 `:5`、`.claude/rules/fs-design.md:23` 的名字）。

**B4 只能是改条款**：D8 已定项 16 射程那一句（`.claude/kb/decisions/08-核心索引结构.md:516`）与 `.claude/rules/fs-design.md:23` 那一格各改一处，并在变更史写用户原话。这是用户的决定，不是读法。

**B4 的代码部分、给用户看的三段数**（量的，X2、同挂载、覆盖写第 1 轮，`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out`）：调用方段「segment=caller nanos=1420204846」（`:159`）、落点判之前「segment=before_landing nanos=103038」（`:160`）、两道之间「segment=between nanos=2778548107」（`:161`）；Q9 五轮 2.766–2.792 秒（日志第 1934 行）。两道之间的步数 13220563557（日志第 1934 行）里 13220431980 是整份拷分配器两次（「calls=2 steps=13220431980」，`research/results/e203-admission-scan-cost-counts-user_scale_unit_area-2026-11-07.out:148`），占 99.999%；挂钟上整份拷两次占 2.497 秒（「exclusive_nanos=2496909549」，同产物 `…timing…:145`），段里还有 0.281 秒不归任何一行（段挂钟减行独占，量的差；成因没查，推的：两份拷贝在行守卫之外 drop）。所以 B4 留下的就是 X2 上 2.78 秒，B4 改的 P0 那 1.42 秒是 4.2 秒里的三分之一。

**什么现象会推翻本节**：(a) 用户明确说 2026-10-31 那句话里的「准入」只指落点判（那就是改条款，由用户做）；(b) 冻结源树里 `settled.resolved` 不再是式子判的输入（现在 `pristine-source/crates/singlefs-core/src/transaction.rs:12921` 是）；(c) `.claude/rules/fs-design.md:23` 那一行的括号里去掉「分配」「生命周期判定」。

## 四、漏掉的候选与各自要的代价数（问题 4）

岔路单三行之外，前两轮已判打中的 B2、B3、B5、B6、B7 不重提。下面五个是这一轮读码与 X2 的数才冒出来的，一律记零轮（被攻过零轮）；它们的数除特别标明外是推的，不在这一轮判谁赢。

先说两件事让候选分得开：X2 上的挂钟几乎全是 U 一个量：P0 的整份拷 1.420 秒、两道之间的两份整份拷 2.497 秒，扫类其余各行在 X2 上加起来只有 132480 步（awk 汇总，推的，`name=row` 同挂载覆盖写第 1 轮三段求和，`research/results/e203-admission-scan-cost-counts-user_scale_unit_area-2026-11-07.out`；重挂载 209442）。而记录项（s36 等）要到 D、F 轴上才涨。所以「改哪几行代码」和「条款怎么写」是两个独立的旋钮，下面的候选是这两个旋钮上有意义的几个格。

另外，用户那句「这个一次拷贝 不是在预料中吗  我们没有优化这部分吧」有代码旁证：调用方那份拷贝上方有注释「整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝」（`pristine-source/crates/singlefs-core/src/transaction.rs:12608`）。它预料到的是 450 KiB 的量级，不是 16 TiB 上的 7.8 GB 与 1.3 秒；这是 B12 的出发点，不是替它下结论。

### B9 去容量项（只改整份拷分配器两处，条款不动）

- 定义：调用方那份「let allocator_before_this_publish = allocator.clone();」（`pristine-source/crates/singlefs-core/src/transaction.rs:12609`）与固定点每轮「let mut rehearsal = allocator.clone();」（`pristine-source/crates/singlefs-core/src/transaction.rs:13393`）换成撤回日志或叠在分配器上的差量视图；失败路径现在是整份换回（`pristine-source/crates/singlefs-core/src/transaction.rs:1596` 一带的 `std::mem::replace(allocator, allocator_before_this_publish)`），改成按日志倒回。别的行一行不动。同形的 `pristine-source/crates/singlefs-core/src/transaction.rs:1859`、`:1964` 与抬 F 预演 `pristine-source/crates/singlefs-core/src/mounted_session.rs:1746` 一并改不改，是实现面的问题，登记 s34 乘数。
- 翻面观测：① 去掉 s1、s24 两行之后 B1 的扫类步数对 U 轴 α ≤ 0.25（我手算 2¹⁴ → 2²⁴ 上同挂载 44732 → 132480 约 0.157，重挂载 81235 → 209442 约 0.137，推的，含照抄类各行，不是模型输出）；② G-user 形状上剩下的扫类挂钟五轮都 ≤ 0.5 毫秒 ⇒ 倾向 B9 够（B8 不必）；五轮都 ≥ 2 毫秒 ⇒ B9 不够，记录项也要改。
- 现有数：X2 上 S1 段挂钟 0.103 毫秒（量的，`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out:160`）；撤回日志的字节数是 B8 模型的推的 768（日志第 1890 行 `undo_log_bytes=768`），D=10⁴ 864、F=10⁴ 同挂载 4032（第 1877、1882 行）。
- 要登记：(a) 一条「拷贝关掉」的下界臂：计时构建里把 12609、13393 两处 clone 去掉（只在不失败的历史上有效），量 X2 与 Y1 上 S1+S2 挂钟，不实现撤回日志；(b) X2 上两道之间有 0.281 秒不归任何一行（3.3），先查它是不是两份拷贝的 drop；是 ⇒ B9 把它一并去掉，不是 ⇒ B9 之后还剩这个数；(c) 记录轴到更大的点（Y1 已挪进复跑全量；D、F 取 10⁵）上的剩余行挂钟，因为 Q5 外推核不过。

### B10 分块共享拷贝（改拷贝的形状，不改它按容量走的事实）

- 定义：`DeviceFreeMap` 八张表（`pristine-source/crates/singlefs-core/src/allocator.rs:304`–`:357`）按固定槽数分块，块用 `Arc` 共享，`clone` 只复制块指针，写时复制那一块。条款字面不动；容量项没有消失（α_U 仍约 1），只是常数缩小，所以要用户把 `.claude/rules/fs-design.md:23` 的「代价不许随盘容量增长」读成「不随容量按字节涨」才站得住：这是读法候选，不是 B9 的同类。
- 翻面观测：单元区 2³⁰−1408 槽上一次整份拷挂钟 < 1 毫秒，且写一槽的放大（复制一块）每次发布 ≤ 一块字节数；U 轴 α 仍近 1（这一点和 B9 区分）。
- 要登记：新微实验，块槽数三档（4096、32768、262144），量 clone 与首写放大在 U=2²⁴ 与 2³⁰ 上；现有可用的数只有 B1 整拷的带宽（7784618016 字节 ÷ 1.32–1.42 秒 ≈ 5.5–5.9 GB/s，字节数取 `research/prompts/e203-preregistration.md` 第四节第 7 条，推的），不够估块指针那一头。

### B11 B8 收严形态（在 B8 上多要求两件事）

定义与收严在哪见 2.5：s36 只按差量问上一版、固定点轮数有常数上界。翻面观测：原判据 Q6 全轴、两种挂载状态、三种发布，α ≤ 0.25。要登记：(a) 把 B1 计数构建里 s36 的 `queries` 拆成「携带角色」与「新写角色」两个读数（现在合在一个 `queries=…` 里，如 `research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:163`）；(b) 固定点轮数在更多同挂载点上的分布与 22 轮点的成因（现有读数 2–4 轮，只有 `files_10000`、`unlinks_10000`、`diagonal` 三个同挂载点是 22、6、5）；(c) 改 s36 式子后的模型复算，这是对现有产物的纯算术，不需要重跑量的那一路。

### B12 留 B1，条款明示一个例外行（不改 crates）

- 定义：D8 已定项 16 射程（`.claude/kb/decisions/08-核心索引结构.md:516`）与 `.claude/rules/fs-design.md:23` 那一格各补一句，点名「发布前整份拷分配器（登记 s1、s24）按单元区线性，第一版接受」，并记欠账一行写触发条件（盘规模或里程碑）。`crates/` 不动。
- 翻面观测：用户给的可接受上界 T*。B12 站住当且仅当 T* 不低于拷贝挂钟，或把适用盘规模限在拷贝低于 T* 的单元区之下。
- 现有数：X2 一次覆盖写整份拷共 3 份（P0 一份加固定点 2 轮各一份），合 1.420 + 2.497 = 3.9 秒（量的），重挂载 4 份（拷贝元素数 26440865660 = 4 × 6610216415，量的，`…counts-user_scale_unit_area-2026-11-07.out`）；固定点轮数乘数在 22 轮点上是 23 份，X2 尺寸上没有这个点，推的 ≈ 23 × 1.3 秒。拷贝挂钟随 U 线性（B1 计时阳性对照五轮 α：同挂载 0.957–1.003，日志第 2402 行；重挂载 1.061–1.094，第 2408 行）；按 5.5–5.9 GB/s，一次拷贝 1 毫秒约对应每盘单元区 12 GiB（推的：每槽两盘共 7.25 字节，1 毫秒 5.5–5.9 MB）。
- 要登记：(a) 大 U 加大 F 同挂载一个点（看轮数乘数在 16 TiB 尺寸上是多少）；(b) 用户定 T* 与触发条件。拷贝挂钟随 U 的曲线已有（U 轴四档加 X2），不用重量。

### B13 B12 加 B11 其余项（例外行加差量）

- 定义：拷贝两行按 B12 写成明示例外，其余扫类各行按 B11。
- 翻面观测：把 s1、s24 两行从 B11 的 Q6 合计里拿掉，全轴、两种挂载状态、三种发布 α ≤ 0.25；拷贝两行的次数乘数（1 + 固定点轮数）不在例外里，轮数上界 k 写进条款。
- 要登记：B11 与 B12 各自的登记合起来，加一条：例外行次数的上界（k）用现有轮数分布给出。

### 代价数一览（量的 / 推的 / 缺）

| 候选 | 现有数 | 缺的数（要登记） |
|---|---|---|
| B9 | X2 S1 0.103 毫秒（量的）；撤回日志 768–4032 字节（推的）；去拷贝后 U 轴 α 约 0.14–0.16（手算，推的）| 拷贝关掉的下界臂在 X2、Y1 上的挂钟；0.281 秒不归行的成因；D、F 到 10⁵ 的剩余行挂钟 |
| B10 | 整拷带宽 5.5–5.9 GB/s（推的）| 块大小三档的 clone 与写放大 |
| B11 | s36 查询 425656 / 3132794（量的）；去掉后 α 0.099 / 0.554 / 0.060（手算，推的）| 查询拆分读数；轮数分布与 22 轮成因；模型复算 |
| B12 | X2 拷贝 3.9 秒（同挂载）、4 份（重挂载）；1 毫秒 ↔ 约 12 GiB 每盘（推的）| 大 U 加大 F 同挂载点；T* 与触发条件（用户）|
| B13 | B11 与 B12 的合集 | 轮数上界 k |

**什么现象会推翻本节**：(a) 「拷贝关掉」的下界臂在 X2 上 S1+S2 仍 ≥ 1 秒（那 B9 不是只靠去拷贝，容量项在别处还有）；(b) 补丁打出的每点每行里 D、F 高端点上最大的不是 s36（B11 的主刀就落空）；(c) 22 轮点上整份拷份数不是轮数加 1（B12 的乘数估计就不对）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1a 把 X2 写成 E203 的 Q4 判定或 F1 打中 | 冲突 | F6 与修订 3 写的是「不判」，改成「判」是放宽，不是收严（`research/prompts/e203-preregistration.md:618`、`:680`）。 |
| 1b X2 当本轮对岔路 B1 的下界推论，E203 的 F1 仍记 unknown | 规则没说 | 登记没写这条路；门槛与四格不动；前提 P1–P3 在冻结源树核到（`pristine-source/crates/singlefs-core/src/allocator.rs:469`–`:488`、`:303`、`:1866`）。 |
| 1c 「扫类步数随记录数不减」 | 冲突（扫类其余各行）；对 s1 的 U 部分不需要这个前提 | 走到命中为止的八类代码不保证，u 轴实测 480→80、58180→53676 有降。 |
| 2a 正文的三行出处 | 冲突 | 三行是 X2 点上的（日志第 1807 行）；D、F 高端点上最大的是 s36 一行（99.2%、97.5%）。 |
| 2b s36 落在 B8 定义哪一项 | 冲突（登记内部） | 登记的「做法」说整表换成差量，「式子」吃 B1 的全表查询次数；模型照式子写，没漏。 |
| 2c s25、s27 | 一致 | B8 定义里「按索引找」留着的对数项；F 同挂载靠固定点 22 轮的乘数涨。 |
| 2d B8 的收严形态 | 规则没说 | B11：s36 只按差量问、固定点轮数有上界，被攻过零轮，不放宽原判据。 |
| 3a B4「两道之间不在运行时决策路径」读法 | 冲突 | 两道之间是式子判的输入（`pristine-source/crates/singlefs-core/src/transaction.rs:12921`），里面有分配与生命周期判定（`.claude/rules/fs-design.md:23`）。 |
| 3b B4 作为条款变更 | 一致（对前两轮判决同向） | 要改 D8 已定项 16 射程（`.claude/kb/decisions/08-核心索引结构.md:516`）与 `.claude/rules/fs-design.md:23` 两处，是用户的决定。 |
| 4 漏掉的候选 | 规则没说 | B9 去容量项、B10 分块共享拷贝、B11 B8 收严形态、B12 留 B1 明示例外行、B13 例外行加 B11 其余项，一律零轮，代价数见第四节表。 |

## 要主 agent 跑的命令

1. 补丁起不起由主 agent 定（只读数，不进判决）。在仓根经任务服务起，输出写新目录，腿没编译、没跑：

   `task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/e203-forks-r1-sonnet/tools/q6-rows-run.sh --repo /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/<主 agent 新目录>`

   跑完看三样：末两行里 `name=q6 axis=F state=same_mount kind=overwrite` 必须与任务 9595 日志第 1804 行逐字相同（`high_end=3213078`），不同就是文件清单与 9595 不一致，先停；`grep -c 'name=q6_rows ' <新目录>/report.out` > 0；`grep 'name=q6_rows point=files_10000 state=same_mount kind=overwrite' <新目录>/report.out` 与 `point=data_units_10000` 各取最大三行，对 2.2 节手算（s36 应是最大一行）。脚本里 18 个计数产物与 18 个计时产物的清单是我按文件名推的（八个取样点用 `-r2`，其余单份），以任务 9595 的命令行为准。
2. 腿没起别的任务；这一节的其余格都是读文件与手算，标「待主 agent 跑」的只有 2.3 节去掉 s36 查询项后的 α（需要再改一次模型，腿没写），与第四节 B9 的「拷贝关掉」下界臂（要登记后才能写）。

下面是只读命令，主 agent 直接起（原样可复现我引的数）：

```
# 冻结源树核哈希（打 SHA_OK）
cd ~/.local/share/singlefs/work/e203-runner-segment-2-r2/pristine-source && sha256sum -c --quiet ../pristine-instrumented-files.sha256 && echo SHA_OK
# 1.5 节反例：同挂载覆盖写第 1 轮，三段求和
cd research/results && for pt in unlinks_1 base unlinks_100 unlinks_1000 unlinks_10000; do f=e203-admission-scan-cost-counts-$pt-2026-11-06-r2.out; grep 'name=row ' $f | grep "state=same_mount kind=overwrite round=1 " | sed -E 's/.*segment=([a-z_]+) row=([a-z_0-9]+) calls=([0-9]+) steps=([0-9]+) .*/\1:\2:\4/' | awk -F: -v pt=$pt '{k=$2; s[k]+=$3} END{printf "%s same_mount: release_find=%d user_slot=%d root_hdr=%d\n", pt, s["allocator_release_linear_find"], s["user_data_slot_linear_scan"], s["root_header_linear_find"]}'; done
# 2.3 节轮数
cd research/results && for f in e203-admission-scan-cost-counts-*-2026-11-0[67]*.out; do grep -H 'name=publish ' $f | grep 'kind=overwrite round=1 ' | grep -o 'point=[a-z_0-9]* state=[a-z_]* \|fixed_point_rounds=[0-9]*' | paste -sd' '; done | sort -u
# 引文核对（pristine-source 的路径前缀要第二个 --root）
python3 scripts/agents/cite-check.py research/prompts/e203-forks-r1-sonnet-output.md --root /home/fy5090/code/singlefs --root ~/.local/share/singlefs/work/e203-runner-segment-2-r2
```

## 没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没写模型。补丁与脚本没编译、没跑；2.2、2.3、第四节里标「手算」「awk 汇总」的数是我读产物行加出来的，当推的，不当模型输出。
- 冻结源树里 s3、s10、s19、s23 只读到入口，没读循环体（1.5 节表末行）；s34 抬 F 预演乘数、D28 式子的九项没读。
- 两道之间 0.281 秒不归行的成因没查（3.3、B9）。
- 没核 E203 计时守卫的范围是不是罩住了 `allocator.clone()` 之后的 drop；没核 Y1（挪进全量复跑，修订 6）。
- 岔路单原文走 `git show ff5dca7d22b1:…`，没落成可引的树内文件，所以岔路单的句子都写成转述。
