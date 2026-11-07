# e203-forks-r1 攻方腿（Opus）报告（2026-11-07）

<!-- doc-lint:not-numbers B1 B4 B7 B8 X1 X2 Y1 Q2 Q4 Q5 Q6 Q8 Q9 Q10 F1 F2 F3 F5 F6 -->

立场：假设主 agent 在正文第五节四格里的倾向是错的，找反例与矛盾。只攻第一轮分工（过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）。没读禁读清单里的文件。

## 复跑命令与文件 sha256

模型目录 `research/prompts/e203-forks-r1-opus-model/`：一个只读产物的小脚本（不编译、不碰 `crates/`、不造历史，没有盘，所以不涉及崩溃状态枚举），复跑命令在末尾「要主 agent 跑的命令」一节。

```text
c421ed2a31e40171932232f911f172907c48bf675737b814a5c1fb7706b0e5d5  residual_check.py
e3ac778163386ee0cb2cb57e02ff9ac4cd73ef58337d9a89ec6bc7e6f290e7ed  inputs/q6_lines.txt
94016e1128dc92a86942e99cec7212aee8a26eaba27e38274e5dfe6faf6643a8  tools/rerun.sh
```

`inputs/q6_lines.txt` 是任务 9595 日志里 `grep '^name=q6 '` 的 24 行原样（日志在仓外，复跑时不读它）。报告里所有算出来的数是我用 `grep`/`awk` 读产物整行算的（推的，主 agent 还没复跑）；`residual_check.py` 是同一组算法写成的脚本，复跑后逐行对本报告的数。读的计数模型源文件 `scripts/experiment-apparatus/e203_admission_scan_cost.rs` sha256 `cf1e5709f313a528a0825ca32e52b312933e2621e7a07f8489e0c06cc64385ab`，mtime 早于任务 9595 日志（推它就是那次编的那一份，任务的编译目录已不在，核不了）。

引冻结源树写成 `pristine-source/…`，相对 `~/.local/share/singlefs/work/e203-runner-segment-2-r2/`。

## 各格判定一览

| 格 | 判定 | 一句话 | 主要出处 |
|---|---|---|---|
| 1 B1 判法与单调前提 | 打中（两处） | ① 拿 X2 读数判 B1 翻面，与跑前写下的修订 3「X2 量过之后照严的读法 Q4 不判」相反，是跑后换判据；② 格里写的前提「扫类步数随记录数单调不减」在产物里有反例（u 轴重挂载那一种，合计步数两处下降）。能站住的只有更窄的一条：调用方那一份整份拷的步数里只由 U 定的那一部分与 u、D、F 无关（冻结源树里核得到），由它给的下界在 X2 上 1.32–1.42 秒，这是零轮的新判法 | 登记第 680 行；u 轴产物两行；`pristine-source/…/allocator.rs:469`、`transaction.rs:12609` |
| 2 B8 的 F2 打中归属 | 打中（三处） | ① 正文点名的三行是在 X2（D = 10、F = 10）上排的，不是在 F2 打中的 D、F 两轴高端上排的；② D、F 两轴的 12 格里，B8 的涨几乎全来自 s36 一个输入项（查询次数取 B1 每个照抄角色一查，正比于 M），这一项与登记 B8 一列同一行写的做法「整表换成差量」、与 B8 常驻内存里已经算了的「按角色索引」自相矛盾——是登记的式子写错，不是模型漏算，也不是 B8 的定义留着它；扣掉这一项，12 格里 11 格不随轴涨，剩 F 轴同一次挂载覆盖写那一格（第 1 轮固定点 22 轮）α ≈ 0.58 仍涨；③ s25 的调用数只记了两处找里的一处，B8 式子吃的调用次数在 8 格上都恰好是实际走查次数的一半 | 模型 `:2645`–`:2655`；登记第 283、287 行；产物 D、F 高端两行；计数 diff 第 178、194 行 |
| 3 B4 读法越不越位 | 打中（覆盖写那一种）；没打中（删除类） | 式子那一判就是 ENOSPC 准入（冻结源树注释写明是 D28（挂载期承诺量） 已定项 1 的式子，拒成 `SpaceAdmissionRefused`），它的需求由两道之间的固定点算出（`demand_of_the_roles_on_each_device(&settled.resolved.rewritten_roles, …)`）；把这一段读成不在运行时决策路径里，正是 `fs-design.md` 禁的「在需要答案的那一刻还没有答案」。B4 只能当改条款交，不能当读法。反过来，删除类发布不判式子那一判，那几种发布上两道之间那一段本来就不在任何准入之前，登记读法 1 把它算进「准入之前」是登记的毛病。另：正文给 B4 的「外推到用户规模 4.3–404 秒」用的是 Q5 判了不可信的同一个外推法 | `pristine-source/…/transaction.rs:12882`、`:12887`–`:12888`、`:12891`；`.claude/rules/fs-design.md:23`、`:34` |
| 4 漏掉的候选 | 打中（漏了三个） | 「式子判吃最坏数」（照 D3（空间分配） 已定项 15 的字面，进门前按最坏数判，固定点挪到准入之后）、「只换两份整份拷」（s1 撤回日志、s24 差量视图，其余照 B1）、「B8 映射表随版维护」（改正 s36 之后的 B8）；各自要的代价数列在第四节，都是零轮 | 第四节 |

## 一、格 1：B1 判法与单调前提

### 1.1 拿 X2 读数判 B1 翻面是跑后换判据（打中）

跑前写死的路径是三句连着的：

- F1 只在 Q4 四格都记「≥ 1 毫秒」时触发，打中之后「B1 行翻面（不留 B1），照记」（`research/prompts/e203-preregistration.md:613`）。
- F6 打中之后「X2 量过之后 Q4 在 U 那一部分改用 X2 的量，记录数那一部分仍外推」（`research/prompts/e203-preregistration.md:618`）——X2 只替换 U 那一部分，记录数那一部分照旧靠外推。
- 修订 3 在第三段跑之前又写死「X2 量过之后照严的读法 Q4 不判」（`research/prompts/e203-preregistration.md:680`）。

产物照这三句判了：`name=verdict clause=F1 fired=unknown`、`name=verdict quantity=Q4 … not_judged_because_the_extrapolation_is_not_trusted=true`、`name=verdict quantity=Q5 cells_passed=2 cells_failed=8`（任务 9595 日志第 2459–2461 行，日志在仓外）。登记的「B1 行够判」要「Q5 核过」（`research/prompts/e203-preregistration.md:645`），Q5 八格没过，所以照登记 B1 行没够判。

拿 X2 的直接读数说 B1 翻面，是在产物出来之后把 Q4（外推的 T）换成一个新量（X2 上一行的实测挂钟当下界）。按 `evidence-discipline.md`「臂的定义也在「跑前写死」之列」那三步：记一次输（照最弱读法：F1 未判，B1 行没够判，输的是登记的外推法，不是 B1 也不是 B8）；只许收严；写明收严在哪。下界读法比 Q4 多要的是一条前提（见 1.2），少要的是 Q5 那一道核对，说不上是收严。它还有一个跑前那一份没有的不对称：下界只能判「≥ 1 毫秒」、判不出「< 1 毫秒」，登记要求的「结果反过来接不接受」（「照判倾向留 B1，先照失败条款 F1 查 s1、s24 的计数与计时，不回头改门槛」，`research/prompts/e203-preregistration.md:340`）在下界读法上没有对面那一支——它是看到数之后才建的、只会往一边判的判法（`evidence-discipline.md`「不许先有结论再建模型」第三问）。

四问：
1. 分不分辨臂：分辨。下界只压 B1（s1 在 B8、B4 下都是撤回日志、步数 0）。
2. 系统看不看得到：与被判系统无关，问的是实验判法。
3. 字面分句：X2 读数满足的是岔路单 B1 行翻面观测「一次覆盖写准入之前的扫类挂钟低于 1 毫秒则倾向留 B1」的否定（「低于 1 毫秒」不成立），不是 F1 的触发分句（「Q4 四格都记 ≥ 1 毫秒」）。岔路单那一句只说低于 1 毫秒时倾向留 B1，没说高于时翻面；正文把它写成「判 B1 翻面」是把一句单向的倾向条件读成了双向。
4. 改法碰不碰得到：下面 1.3 的窄前提能把下界做成一条可登记的新判据，碰得到这一格；但它是新判据、零轮。

### 1.2 「扫类步数随记录数单调不减」在产物里有反例（打中）

格里写的前提是整个扫类合计随记录数单调不减。产物里不成立：u 轴、重挂载之后、覆盖写第 1 轮，B1 准入之前扫类合计（P0、S1、S2 三段、类型「扫」各行，与 Q2 同一个和；端点与 `name=q2 axis=u state=remounted kind=overwrite low_end=2522728 high_end=6216500` 逐个对上）逐点是（推的，我按产物整行加的）：

| u | 1 | 10 | 100 | 1000 | 10⁴ |
|---|---|---|---|---|---|
| 扫类合计 | 3368382 | 2522728 | 3398300 | 2617855 | 6216500 |
| 整份拷份数 | 4 | 3 | 4 | 3 | 5 |

两处下降（1 → 10、100 → 1000），同一次挂载那一种是单调的。下降的来由是固定点轮数在点与点之间上下跳：u = 100 那一点两道之间拷了 3 份（「calls=3 steps=2426115」，`research/results/e203-admission-scan-cost-counts-unlinks_100-2026-11-06-r2.out:3070`），u = 1000 那一点 2 份（「calls=2 steps=1619246」，`research/results/e203-admission-scan-cost-counts-unlinks_1000-2026-11-06-r2.out:3218`）。Q5 那一格（`name=q5 axis=u state=remounted predicted_steps=-5186595`，日志第 2455 行）把外推推成负数，也是这个下降。所以「X2 的记录数只到 10、用户规模上只会更大」不能靠整个合计的单调性。

### 1.3 能站住的窄前提（冻结源树里核得到）

只看调用方那一份整份拷（s1）：
- 每次经 `publish_version_of_trees_relocating_units_after_a_check` 进门的发布都拷一份，无条件：「let allocator_before_this_publish = allocator.clone();」（`pristine-source/crates/singlefs-core/src/transaction.rs:12609`）。产物里 18 个点都进了这一段（`name=verdict clause=stop_4 publishes_that_skipped_the_caller_or_the_judge=0`，日志第 2441 行）。
- 被拷的三张逐槽位图长度在建空闲图时按单元区槽数定死，不随内容变：「allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],」（`pristine-source/crates/singlefs-core/src/allocator.rs:469`），`isolated`（`:477`）、`held_until_floor_takes_effect`（`:480`）同形。
- 所以 s1 的步数 = 2 × (3U + 5⌈U/64⌉) + 别的几张表的长度，后一项 ≥ 0、随记录数只增不减。产物里每个点的 `copy_elements` 行都是 `from_the_unit_area` 不变、`other_tables` 非负（例：D = 10⁴ 那一点 `from_the_unit_area=806912 other_tables=23615`，日志第 954 行）。

这一条给的是步数的下界；挂钟的下界是推的（同一台机器上多拷字节不会更快）。同一 U 上加记录数的那一对是量过的，方向一致：U = 2²⁰ 时 (10, 10, 10) 第 1 轮 s1 独占 202201 纳秒（「exclusive_nanos=202201」，`research/results/e203-admission-scan-cost-timing-unit_area_2_pow_20-2026-11-06-r2.out:111`），X1 = (2²⁰, 10⁴, 10⁴, 1000) 第 1 轮 2765105 纳秒（「exclusive_nanos=2765105」，`research/results/e203-admission-scan-cost-timing-diagonal-2026-11-06.out:111`）。X2 上 s1 十轮 1318225192–1422421328 纳秒（量的，`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out:112`「exclusive_nanos=1420202116」、`:865`「exclusive_nanos=1319344068」），比 2 毫秒大近三个数量级；用户规模上只靠这一份就过门槛，不用外推、也不用整个合计单调。

结论：B1 翻面能成立的形态是「新判据 Q4 下界：X2 上 s1 独占计时五轮都 ≥ 2 毫秒、两种挂载状态都如此，前提是 1.3 那条窄单调性」，它要写成新登记的一条（零轮），不能写成 F1 打中。什么会推翻它：用户规模的某种形状上覆盖写不经 `publish_version_of_trees_relocating_units_after_a_check` 进门（停机条款 4 那一类），或分配器换成稀疏表示让三张位图长度随内容变。

## 二、格 2：B8 的 F2 打中归属

### 2.1 正文点名的三行不是 F2 打中那几格的三行（打中）

`name=q6_top_rows point=user_scale_unit_area …` 那几行的点是 X2。模型取「高端点」的写法是按 U + u + D + F 之和取最大的那一个点（「// 贡献最大的三行（高端点）：B8 在三段合计里步数最大的三行，与它们吃的差量输入名。」，`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2645`；「counts.points[*point_name].unit_area_slots」，`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2651`），和里 U = 1073740416 压倒一切，所以永远取到 X2 = (2³⁰ − 1408, 10, 10, 10)。X2 不在 D 轴、也不在 F 轴上，B8 在 U 轴上不涨（`name=q6 axis=U … band=DoesNotGrow`）。F2 要「列出贡献最大的三行与它们吃的差量输入」（`research/prompts/e203-preregistration.md:614`），模型列的是一个 B8 不涨的点上的三行。正文第五节第 2 格拿 `allocator_release_linear_find`、`mapped_units_whole_table`、`record_intersection_filter` 当 F2 的病根去问「模型漏算还是定义如此」，问的对象就错了。

四问里第 3 问（满足的是哪个分句）：F2 的触发分句「Q6 在任一根轴上……记「B8 仍随总数涨」，α ≥ 0.5」照字面在 D、F 两轴 12 格上都满足（量的读数，推的判档）；打中之后那半句「列出贡献最大的三行」由模型写错了点，这是装置的缺陷，不改触发本身。

### 2.2 D、F 两轴上的涨几乎全是 s36 的一个输入项，那一项是登记式子写错（打中）

B8 的 s36 一行在模型里是「input.mapped_units_changed_over_all_rounds + input.mapped_units_query_levels,」（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:310`），后一项从产物读「product.query_levels_of(&at, segments, "mapped_units_whole_table");」（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2430`），照的是登记「每轮 dm + 查询次数 × lg(M)」（`research/prompts/e203-preregistration.md:283`）。模型与登记一致，不是模型漏算。

那个「查询次数」在 B1 里是什么：冻结源树建这一版 `mapped_units` 整表时，照抄的每个角色按角色查一次上一版的索引——「照抄的角色约有上一版那么多个，每个按角色取上一版的 key」（`pristine-source/crates/singlefs-core/src/transaction.rs:9073`），计数构建在那个索引的每次查找上记一次（`research/results/e203-admission-scan-cost-counts-counters-diff-2026-11-06-r2.diff:324`「query_attributed(index_entries);」）。所以查询次数 ≈ 每轮 M，是整表的量，不是差量：

- D = 10⁴：「queries=30404 query_levels=425656」（`research/results/e203-admission-scan-cost-counts-data_units_10000-2026-11-06.out:162`），M = 10145、3 轮。
- F = 10⁴：「queries=223771 query_levels=3132794」（`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:163`），22 轮。
- 基点：「queries=53 query_levels=318」（`research/results/e203-admission-scan-cost-counts-base-2026-11-06-r2.out:160`）。

而登记同一行 B8 的做法写的是「`mapped_units` 整表换成差量，按角色查走随一版维护的索引」（`research/prompts/e203-preregistration.md:283`），B8 常驻内存里也已经算了这份随版维护的索引「`mapped_units` 按角色 M × (σ_unit + 8)」（`research/prompts/e203-preregistration.md:287`）。整表换成差量之后，照抄的那 ≈ M 个角色不再被逐个列出、逐个查；登记的式子却把 B1 逐个查的次数原样带进了 B8。这是 `evidence-discipline.md`「臂的定义里同时写了「怎么做」和「做完会怎样」，先证明前一句推得出后一句」那一条：做法推不出这个式子。登记 Q6 一行自己写过「B8 的式子吃的是量出来的差量（λ_ar、m_round、dirty、δ、查询次数）」（`research/prompts/e203-preregistration.md:331`），可 s36 的「查询次数」量出来的不是差量。

扣掉 s36 的查询项之后（推的，我按产物算，`residual_check.py` 第一组行复跑）：

| 轴 | 挂载 | 发布 | Q6 的 α | 查询项占高端 | 扣掉后低端 → 高端 | 扣掉后 α |
|---|---|---|---|---|---|---|
| D | 同一次 | 覆盖写 | 0.797 | 99.19% | 1430 → 3464 | 0.128 |
| D | 同一次 | 意图 | 0.734 | 99.34% | 1462 → 1874 | 0.036 |
| D | 同一次 | 最后一批 | 0.728 | 99.30% | 1541 → 1999 | 0.038 |
| D | 重挂载 | 覆盖写 | 0.787 | 99.17% | 1556 → 3584 | 0.121 |
| D | 重挂载 | 意图 | 0.665 | 99.27% | 2416 → 2090 | −0.021 |
| D | 重挂载 | 最后一批 | 0.729 | 99.15% | 1533 → 2427 | 0.067 |
| F | 同一次 | 覆盖写 | 1.088 | 97.50% | 1430 → 80284 | **0.583** |
| F | 同一次 | 意图 | 0.794 | 99.10% | 1462 → 3907 | 0.142 |
| F | 同一次 | 最后一批 | 0.729 | 99.05% | 1541 → 2742 | 0.083 |
| F | 重挂载 | 覆盖写 | 0.729 | 99.02% | 1556 → 2834 | 0.087 |
| F | 重挂载 | 意图 | 0.666 | 99.25% | 2416 → 2168 | −0.016 |
| F | 重挂载 | 最后一批 | 0.730 | 99.05% | 1533 → 2746 | 0.084 |

剩下涨的那一格（F、同一次挂载、覆盖写）来自调用结构：那一次覆盖写走了 22 轮固定点（「fixed_point_rounds=22」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:1044`），同一点第 2 轮只有 2 轮（「fixed_point_rounds=2」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:1259`，第 3–5 轮也是 2）；s25 的调用数也从基点的 42 涨到 2402（「calls=2402」，`research/results/e203-admission-scan-cost-counts-files_10000-2026-11-06.out:157`）。B8 的定义写明「调用结构（几轮、调几次）不变」，这一格才是「B8 的定义本身留着它」的那一种。它只来自 Q6 取的第 1 轮（「let at = round_key(point, state, kind, 1);」，`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2309`），同一格换成第 2–5 轮推的不涨——这一格是单次取样，正文不能写成「B8 在 F 轴上随文件数涨」的全称。

四问：
1. 分不分辨臂：s36 的查询项只进 B8（s36 只在 S2 被调到，B4 在 S2 取 B1 量的），打中只落在 B8 上；但病根是式子把 B1 的整表结构抄进了 B8，它分辨的是「登记写的 B8 式子」与 B1，不是「B8 这条路」与 B1。
2. 系统看不看得到：照 B8 自己的做法，发布那一刻系统手里有随版维护的按角色索引与这一轮的 dm，用不着逐个列照抄的角色。
3. 字面分句：F2 触发分句在 12 格上照字面满足；按最弱读法记一次输：登记的 B8 在 D、F 两轴上输给了 F2。
4. 改法碰不碰得到：改正 s36（查询只给 dm 那几条）碰得到 11 格，碰不到 F 轴同一次挂载覆盖写那一格（22 轮固定点）；那一格要的是另一个改法（固定点轮数有界），B8 的定义不许改调用结构。

### 2.3 B8 能不能出收严的新形态

不能叫「收严」。改正 s36 是把式子改成推得出做法的那一种，按原来那套判据（Q6 的步数）判出来的步数更少，是被原判据判得更松；它多要求的东西（每一版的 `mapped_units` 随版维护、照抄的角色不再列出）在 B8 的常驻内存里本来就算了，说不出比旧形态多要求了什么。照 `evidence-discipline.md` 那三步，缺「只许收严」一步就算新实验：叫它「B8 映射表随版维护」，零轮，只在我的读数上算过，要登记成新的一次（见第四节）。即便改正了，F 轴那一格（推的 α ≈ 0.58，加上 2.4 节 s25 的修正推的约 0.60）仍照 F2 判 B8 行翻面，除非再加「固定点轮数有界」这一条，那就是改 B8 的定义，是另一个候选。

### 2.4 s25 的调用次数少记了一半（打中，④ 那一类）

登记把 s25 的两处找都列在同一行（`allocator.rs:2265`、`:2211`），B8 式子 c25 × lg(R_a) 的 c25 是这一行被调了几次。计数构建只在 `:2211` 那一处（`record_for`）记调用：「call(crate::transaction::e203_scan::Row::AllocatorReleaseLinearFind);」（`research/results/e203-admission-scan-cost-counts-counters-diff-2026-11-06-r2.diff:178`）；`:2265` 那一处（`release` 里每块盘一次）只记表长与步，不记调用：「scale_to_the_hit(crate::transaction::e203_scan::Row::AllocatorReleaseLinearFind, self.records.len());」（`research/results/e203-admission-scan-cost-counts-counters-diff-2026-11-06-r2.diff:194`），那一段 hunk 里没有 `call(`。两处每找一次都记一份表长，所以「调用处表长之和 ÷ 记录数」就是实际找了几次。8 格（基点、D、F 高端、X2 × 两种挂载状态，覆盖写第 1 轮，两道之间）上找的次数都恰好是记的调用数的 2.000 倍（推的，我按产物算；例：D = 10⁴「calls=86 steps=3178170」「scale_to_the_hit=3808768」，`research/results/e203-admission-scan-cost-counts-data_units_10000-2026-11-06.out:157`，3808768 ÷ 22144 = 172 = 2 × 86）。照记的调用数，每次找平均走 36955 步，大于表长 22144，这一点只靠一份产物就能看出调用数不对。方向：B8 的 s25 少算一半，偏向 B8；在 F 轴那一格上把扣掉 s36 之后的 α 从 0.583 推到约 0.60。修订 4 依据里那一句「s25 base 同一次挂载覆盖写第 1 轮 `calls=42 steps=43550`……连上一版表长的上界都给不了」就是这个少记的调用数造成的，当时改成了调用处读表长，B8 吃的调用次数没跟着改。

## 三、格 3：B4 读法越不越位

### 3.1 式子那一判就是 ENOSPC 准入，它的答案由两道之间那一段算出（覆盖写：打中）

冻结源树：
- 两道都拒成同一种空间拒绝：落点那一判 `:12829`、式子那一判 `:12930` 都是「.map_err(PublishError::SpaceAdmissionRefused)?;」（`pristine-source/crates/singlefs-core/src/transaction.rs:12930`）。
- 式子那一判的注释写明它是哪一条：「空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取）」（`pristine-source/crates/singlefs-core/src/transaction.rs:12882`）；D28 已定项 1 的定案：「分配准入按这个式子算可用」（`.claude/kb/decisions/28-挂载期承诺量.md:24`）。
- 它的需求是两道之间那一段固定点的产出：「let demand = demand_of_the_roles_on_each_device(」（`pristine-source/crates/singlefs-core/src/transaction.rs:12887`），参数「&settled.resolved.rewritten_roles,」（`pristine-source/crates/singlefs-core/src/transaction.rs:12888`），`settled` 是 `:12831` 那一次 `settle_the_allocation_record_tree` 的结果；判不判也看它（`publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)`，`:12902`）。

`fs-design.md` 那一格：「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |」（`.claude/rules/fs-design.md:23`），并点名被禁的是「「在需要答案的那一刻还没有答案」，以及「审计与被审计用同一段代码」。」（`.claude/rules/fs-design.md:34`）。对覆盖写，两道之间那一段就是 ENOSPC 准入在被问到那一刻现算自己的答案；B4 把它「读成不在运行时决策路径一格里」，与这一格的字面相反，也与 D8（核心索引结构） 已定项 16 射程用户那一读「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径」（`.claude/kb/decisions/08-核心索引结构.md:516`）里的「准入」相反——用户免掉的只是准入之后，式子那一判本身就是准入。登记读法 1 也写的是「两道准入都算（第一轮判决第二节第一格）」（`research/prompts/e203-preregistration.md:21`）。

所以 B4 站不住「读法」这个身份：它只能是一条改条款（同时改 D8 已定项 16 射程与 `fs-design.md` 那一格，后者是规则文件），而且改的不只是「代价随不随盘容量」那半句，还要在 ENOSPC 准入上放开「被问到那一刻还没有答案」——那是这一格最核心的禁令。岔路单 B4 行把自己叫「改射程」、正文第五节第 3 格叫「读成」，前者对、后者越位。

四问：
1. 分不分辨臂：分辨，只打 B4。
2. 系统看不看得到：式子那一判在判的那一刻看得到的就是固定点算出的 `rewritten_roles`；B4 下它还是靠现算，判法本身不变。
3. 字面分句：打在 `fs-design.md:23` 的「ENOSPC 准入」与 `:34` 的「在需要答案的那一刻还没有答案」上，不在 F5（F5 只判落点判之前，没打中照旧）。
4. 改法碰不碰得到：第四节「式子判吃最坏数」碰得到（判的那一刻有答案，固定点挪到准入之后）；B4 自己碰不到。

### 3.2 删除类发布上 B4 的读法反倒是对的，登记读法 1 在那几种发布上多算了（没打中 B4，打中登记）

冻结源树：「删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判」（`pristine-source/crates/singlefs-core/src/transaction.rs:12891`），判不判那一支是「!step.is_a_deletion_class_publish()」（`pristine-source/crates/singlefs-core/src/transaction.rs:12900`）。unlink 的意图那次与最后一批不过式子那一判，它们的两道之间那一段之后没有任何准入在等它的答案；照用户那一读，那一段在这两种发布上已经是「准入之后」。登记读法 1 把三种发布的 S2 都算进「准入之前」（Q2、Q6、Q8 的 unlink 各格都吃它）。对 B8：F2 在 unlink 两种发布上的 8 格（D、F × 两种挂载 × 意图、最后一批）里，涨的那一项（s36）只在 S2；照用户那一读，这 8 格判的是一段不受条款管的代码，只剩覆盖写的 4 格是 F2 该判的。这一条不改 F2 在覆盖写上的判定。

### 3.3 正文给 B4 的「用户规模上 4.3–404 秒」用的是被 Q5 判不可信的外推（打中，内部矛盾）

Q9 在用户规模上的数「同 Q4 外推」（登记 Q9 一行）。Q5 八格核不过（`name=verdict quantity=Q5 cells_passed=2 cells_failed=8`），登记据此判 Q4「外推不可信，不判」。同一个外推法给的 `name=q9_user_scale_nanos … nanos_of_each_round=403797602001,…` 却被正文当成 B4 的代价数交出去。不能一边说 Q4 的外推不可信所以 B1 不判，一边拿同一外推给 B4 报价。能用的只有 X2 上直接量的：两道之间那一段第 1 轮 2778548107 纳秒（量的，`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out:161`），其中两份整份拷独占 2496909549 纳秒（量的，「exclusive_nanos=2496909549」，`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out:145`），约 90%。用 1.3 那条窄单调性，s24 每轮至少一份（每轮「let mut rehearsal = allocator.clone();」，`pristine-source/crates/singlefs-core/src/transaction.rs:13393`），B4 下用户规模上两道之间至少约 1.2 秒一次覆盖写（推的下界，零轮）；上界照登记没有可信的数。

## 四、格 4：漏掉的候选

三个都是零轮，写成要登记的实验，不在这一轮判它赢。名字是新起的，不沿用 B 族的号（B 族下一个号是不是已被别的轮用过，我没核）。

### 4.1 「式子判吃最坏数」

定义：式子那一判的需求不由固定点现算，改用进门之前就有的最坏数（落点那一判已经这样做：「固定点按 ckpt_cost 的最坏数」，`pristine-source/crates/singlefs-core/src/transaction.rs:12803`），式子判完再走固定点（仍在任何读写之前，固定点交回的那几种拒绝照旧盘上逐字节不变）；P0 那一份整份拷照 B8 换撤回日志。依据是条款的字面：D3（空间分配） 已定项 15「算不过就拒绝。这样预留量动态贴着实际需求」那一段要的是进门前先按最坏情况算：`.claude/kb/decisions/03-空间分配.md:295`「算不过就拒绝。这样预留量动态贴着实际需求」，今天的实现在式子那一判上算的是精确需求。它与 B4 的区别：B4 改条款、放开 ENOSPC 准入上「现算答案」；它不改条款，把两道之间那一段挪到准入之后，照用户那一读（D8（核心索引结构） 已定项 16 射程）准入之后不受代价条款管。

要的代价数：① 每次发布最坏数比精确需求多扣多少槽（三种发布 × 18 个点，可以从现有计数产物里的 ckpt_cost 读数与 `rewritten_roles` 现算，推的）；② 接近满盘的历史上，最坏数判拒而精确数放行的发布有几次（假性 ENOSPC，要新造历史，量的）；③ 准入之前的步数（推的同 Q8 的 B4 臂，P0 + S1，`name=q8_user_scale … narrowed_steps=66`）；④ 挪到准入之后的那一段在 X2 上的挂钟（量的已有：两道之间 2.75–4.16 秒一次覆盖写，见 3.3）——条款不管它，但这是用户每次覆盖写都要等的时间，交用户时要单列。

### 4.2 「只换两份整份拷」

定义：s1 换撤回日志、s24 换差量视图（B8 那两行的做法），别的行照 B1。从现有计数产物推（推的，我按产物算的扫类合计减两份整份拷那几行；同一次挂载、覆盖写第 1 轮）：

| 点 | 扫类合计 | 其中整份拷 | 剩下 |
|---|---|---|---|
| 基点 B0 | 2503416 | 2424579 | 78837 |
| U = 2²⁴ | 309991237 | 309858870 | 132367 |
| X2 | 19830780337 | 19830647970 | 132367 |
| D = 10⁴ | 8252955 | 3321028 | 4931927 |
| F = 10⁴ | 163060965 | 20267968 | 142792997 |

U 轴上剩下的不涨（2²⁴ 与 X2 逐个相同），D 轴 α ≈ 0.60、F 轴 α ≈ 1.09（推的），照登记的两条带仍随记录轴涨，过不了 D8 已定项 16 射程那一句；它买到的是用户规模上压倒性的那一项（X2 上合计的 99.999%）。要的代价数：撤回日志与差量视图的峰值（Q7 已有 B8 的推的数，X2 上各 768 字节）、X2 上换掉之后两道之间还剩多少挂钟（量的残差已有，见第五节 5.4：0.27–0.41 秒）、D、F 轴上剩下那几行各是哪几行。

### 4.3 「B8 映射表随版维护」与「落点判之前按 B4、两道之间按 B8」

前者是 2.3 节那一形：s36 只给这一轮进出 `mapped_units` 的条目查索引。正文举的组合（P0、S1 按 B4 即 B8 的改法，S2 按 B8）在登记的式子上与 B8 逐行相同（B4 的 P0、S1 本来就取 B8 一列，S2 再取 B8 就是 B8），不是一个新的候选；它值得单列只在 s36 改正之后。要的代价数：改正 s36 之后 Q6 每轴每格的档（复跑计数模型、不重跑历史，推的）；F 轴同一次挂载覆盖写那一格五轮各自的档（现在只取第 1 轮）；s25 调用数改正之后的同一组档；常驻内存照 Q7 不变。

## 五、计数模型与冻结源树、产物对不上的地方（攻击面 ④；两边都查，不默认哪边对）

| 号 | 哪里对不上 | 模型那边 | 冻结源树 / 产物那边 | 判哪边错 | 影响 |
|---|---|---|---|---|---|
| 5.1 | s36 的查询次数 | 照登记吃 B1 的查询次数（`e203_admission_scan_cost.rs:310`、`:2430`） | B1 里那是每个照抄角色一查（`pristine-source/…/transaction.rs:9073`），≈ 每轮 M | 登记第五节 s36 一行自相矛盾（做法「整表换成差量」推不出式子）；模型照登记，没错 | F2 在 D、F 两轴 12 格里 11 格只靠这一项打中（2.2） |
| 5.2 | s25 的调用次数 | c25 取产物的 `calls` | 两处找只有 `:2211` 记调用，`:2265` 不记（计数 diff 第 178、194 行）；8 格上走查次数都是 `calls` 的 2.000 倍 | 计数构建插漏；登记把两处列在同一行，c25 该数两处 | B8 的 s25 少一半，偏向 B8；不翻档（2.4） |
| 5.3 | 「贡献最大的三行」取哪个点 | 按 U + u + D + F 之和取最大（`:2645`–`:2655`），恒取 X2 | F2 打中在 D、F 两轴高端 | 模型错（F2 原文要的是打中那几格的三行） | 正文第 2 格问错了三行（2.1） |
| 5.4 | F3（残差）与 Q3、Q10 | `timing_report` 只拿计时产物算轴（`:1851` 起），轴上的点取自产物里的 `name=point` 行 | 计时产物里 `name=point` 一行都没有（`grep -c 'name=point '` 计时两份都是 0），所以 Q3、Q10 一行都没打（日志里 `name=q3`、`name=q10`、`name=q10_share` 共 0 行），F3 却报 `fired=false`（日志第 2463 行） | 模型错：没判被报成了没打中；它的单测（`:4236` 起）造的计时产物带 `name=point` 行，所以没红 | 照产物现算（推的）：两道之间那一段的残差（段挂钟减各行独占计时之和）在 U 轴两端（2¹⁴ → 2²⁴）重挂载覆盖写五轮 α = 0.746、0.682、0.691、0.703、0.699，五轮都 ≥ 0.5，F3 应当打中；同一次挂载五轮 0.44–0.50，落在中间带。X2 上这一残差一次覆盖写 0.27–0.41 秒（十轮），是清单外、在式子判之前、随 U 涨的步，三条臂都留着它 |

5.4 的后果按登记 F3 的「打中之后」：「记「清单不全」；B8、B4 两行的结论标「只对清单内的步」」。B8 的「不随总数涨」、B4 的两道之间那一段的报价，都只对清单内的步；清单外那一份在 X2 上比 1 毫秒大两个数量级以上。四问：分不分辨臂——不分辨（三条臂共用这一份残差），照「打中不分辨臂」办，不拿它判臂，另立一笔账先补清单；字面分句——F3「某一段的残差在某根轴上 5 轮都 α ≥ 0.5」，满足的是 S2、U 轴、重挂载覆盖写（推的，主 agent 复跑模型时把 `name=point` 行并进计时产物再判一次即可坐实或推翻）。

## 六、几个改法各修哪一格

| 改法 | 1 B1 判法 | 2 B8 F2 | 3 B4 读法 | 5.4 残差 | 量过 / 推的 |
|---|---|---|---|---|---|
| 新判据「Q4 下界」（X2 上 s1 五轮都 ≥ 2 毫秒、两种状态，前提 1.3） | 修（B1 行「倾向留 B1」不成立） | 不碰 | 不碰 | 不碰 | 推的（X2 的 s1 读数是量的，判据是新的、零轮） |
| 改正 s36（只查 dm） | 不碰 | 修 11 / 12 格，F 轴同一次挂载覆盖写那一格不修 | 不碰 | 不碰 | 推的，待主 agent 复跑 `residual_check.py` |
| s25 调用数两处都记 | 不碰 | 不修，让 B8 更重一点 | 不碰 | 不碰 | 推的 |
| 「式子判吃最坏数」 | 不碰 | 不碰 | 修（判的那一刻有答案） | 不碰（残差挪到准入之后，条款不管，挂钟还在） | 推的 |
| 计时产物并进 `name=point` 行再判 F3 | 不碰 | 不碰 | 不碰 | 修（F3 从「没判」变成有判定） | 推的，待主 agent 跑 |

上表的改法都只在我的读数上算过、被攻过零轮。

补 5.4：unlink 两种发布同形（推的，同一算法）：重挂载之后意图那次五轮 0.712、0.719、0.682、0.719、0.723，最后一批 0.701、0.714、0.695、0.713、0.705；同一次挂载意图那次 0.480–0.500、最后一批 0.434–0.478。我没套登记作废条款 5 的包含自检（Σ 独占计时 ≤ 段挂钟，十个格都满足，残差为正），也没按 `round_is_usable` 筛轮；主 agent 照模型的口径复跑时若有轮被筛掉，F3 那一格会变成「有效轮不满五轮」而不是打中。

## 七、没打中的形状

- **X2 那一份整份拷会不会在用户规模上不发生**：查了调用方的拷贝是不是有条件的（无条件，`transaction.rs:12609`）、18 个点里有没有发布绕过调用方那一段（0 次，`stop_4` 那一行）、三张逐槽位图的长度是不是随内容变（不变）。没造出 X2 那一份在更大记录数下变小的形状。
- **B4 的 F5（落点判之前不涨）会不会被清单外的步打穿**：P0、S1 两段的残差在 U 轴两端五轮、两种挂载状态上 α 都在 0.009–0.215（推的，同 5.4 的算法），不涨；F5 没打中照旧。
- **B4「改的是函数，不是调用点」该不该把 s1 的撤回日志带到 s24**：s1 与 s24 都是 `allocator.clone()`，但 B8 对 s1 的改法是去掉调用点（撤回日志），不是改 `clone` 本身，读法第 8 条够不着 s24；判成没打中。
- **B8 的 s27、s30 在 D、F 轴上是不是另有漏算**：扣掉 s36 之后它们在 11 格上不涨，没再往下拆。
- **四根轴之外的旋钮**（环长、盘数、截断、快照、整理）：没扫，照登记第八节第 4 条只在「6 MiB 环、两块盘、覆盖写与 unlink 上」。

取样范围：18 份计数产物与 18 份计时产物里，覆盖写第 1 轮的全部行（F 轴那一格另看了第 2–5 轮的 `fixed_point_rounds`），u 轴五个点两种挂载，U 轴两端与 X2 的十轮计时。这些「没打中」都是一次抽样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不能单独支撑结论。

## 八、这条腿自己的限度

- 报告里所有算出来的数是我用 `grep`/`awk` 读产物整行算的（推的），主 agent 还没复跑；`residual_check.py` 我没跑过（照共用约束「子 agent 不交任务」，模型交主 agent 跑），它与我的 `awk` 是同一组算法的两份写法，不是两条独立路径。
- 5.4 的「残差随 U 涨」是什么步，我没定位；只知道它在两道之间、清单外、X2 上 0.27–0.41 秒。
- 计数模型源文件是不是任务 9595 编的那一份，只按 mtime 推。
- 冻结源树的引文用的是 `pristine-source/` 下的行号，与主工作区今天的行号不同；跑前登记里的 `文件:行` 指的是另一份冻结副本（登记第 5 行点名的 `m4-admission-delta-r1/repo/`），我核了 `allocator.rs:2211`、`:2265` 两行在 `pristine-source/` 里也是那两处，别的行没对。
- 没读设计轮三份判决与正推腿报告，「B 族下一个号」与候选重名没核。

## 九、要主 agent 跑的命令

1. 复跑这条腿的模型（只读产物，几秒；照 `.claude/rules/three-way-inference.md` 核查第 3 步，先把模型目录拷进主 agent 自己的草稿目录，在拷贝里跑）。
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`（命令里全是绝对路径）。
   - 命令：
     ```text
     task-server/task submit -- bash <主 agent 草稿目录>/e203-forks-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/e203-runner-segment-2-r2/pristine-source --work <主 agent 草稿目录>/e203-forks-r1-opus-rerun
     ```
   - 跑完看：`TASK_SERVER_RESULT` 那一行 `type=` 是完成、`exit=0`；`task-server/task log <号>` 末行 `name=rerun_done lines=64`（推的行数：12 + 12 + 8 + 16 + 10 + 6）；`<work>/residual_check.out` 里逐行对本报告：
     - `name=b8_without_mapped_units_queries` 12 行的 `alpha=` 对 2.2 节表最后一列（F、same_mount、overwrite 那一行 `alpha=0.583 band=GrowsWithTheAxis`，其余 11 行 `band=DoesNotGrow`）；
     - `name=b1_scan_total_on_the_u_axis state=remounted` 五行 `scan_total=` 依次 3368382、2522728、3398300、2617855、6216500，`name=monotone_on_the_u_axis state=remounted … steps_where_the_total_decreased=2`，同一次挂载那一行 `=0`；
     - `name=release_find_calls` 8 行都是 `finds_over_counted_calls=2.000`；
     - `name=caller_copy_unit_area_part` 16 行 `other_tables=` 都 ≥ 0；
     - `name=x2_timing` 10 行 `caller_copy_nanos=` 在 1318225192–1422421328，`settling_share=` 约 0.90；
     - `name=s2_residual_on_the_u_axis state=remounted kind=overwrite alphas=0.746,0.682,0.691,0.703,0.699 all_five_at_least_half=True`。
   - 对不上的，以复跑的为准，报告里那一格作废。

2. 5.4 的坐实不在这条腿能交的命令里：要么改计数模型让 `timing_report` 从计数产物取点的轴值（改 `scripts/experiment-apparatus/e203_admission_scan_cost.rs`，归实验执行员），要么在副本里给计时产物补 `name=point` 行再交一次 report（产物原件不改）。两条都要另派，我不写命令。

## 十、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务；判决要引的数（第二至五节的全部推的数）没复跑，命令在第九节。
- 没读禁读清单里的正推腿报告与这一轮的判决；没读设计轮三份判决。
- 没建仓副本、没编译、没造历史；副本上的数一个都没有。
- 没给 B8、B4 写新的式子跑出数；第四节的三个候选只列了要的代价数。
- 「没打中」那几条只抽了一次样。

## 十一、草稿与清理

草稿目录 `/home/fy5090/.local/share/singlefs/work/e203-forks-r1-opus/` 里只有几份小文本（岔路单 `git show` 出来的一份、拼 `rerun.sh` 用的片段），没有仓副本、编译目录与库拷贝，没有要删的。模型目录 `research/prompts/e203-forks-r1-opus-model/` 是交付物，留着给主 agent 复跑。
