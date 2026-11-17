# e203-b8-criterion-r1 正推腿报告：B8 判据改成按这一次碰到的单元算

批次：`records/2026-10-13-里程碑四预开发.md#收口修补二批`。立场：正推（从已定条款、`crates/` 与装置推「判据该是什么」，再与 40 份产物逐格比）。不判别、不选臂。

## 复跑与文件

模型目录 `research/prompts/e203-b8-criterion-r1-sonnet-model/`（读 `research/results/` 下 E206 的 40 份产物，纯读、纯算，不起编译）。复跑命令在「要主 agent 跑的命令」一节。每个文件的 sha256（`sha256sum` 现算）：

```text
12138b7fd47e86de1aaedc03bf075b581eeb0e1aac52c46dcf8d8c6effe93f44  extract.py
97b402cbbce762b085fe5c6fa31090fa7b54ca7b82205d882f80399b359d6485  checks.py
30802e2922f50e8f4eecaf375af4c46cffe6470fc99fb7c000f48dba8d6bd62a  criterion.py
439ca9d5ce14b14eb602134568ae5fc0a1a482c0db40bb766758ffb34fc4962f  tools/rerun.sh
1a440e86c9afbe876c061078a319c15d7e9a820d6a3bb74e514bda5ff54a8cdd  pubs.tsv（extract.py 的输出，7717 行，不入模型目录；rerun.sh 重算后核 out/pubs.sha256）
```

`out/` 下 16 份留存输出（rerun.sh 逐份 `diff -r`）。这一轮我是在草稿目录里拷同一份脚本直接跑出来的（只读产物，没经服务），所以数都是「我量的」，判决要引的由主 agent 经服务复跑后才算数。

## 实现今天的样子（我的观测）

- `crates/` 里没有 B8：`grep -rln 'by_delta\|byDelta\|差量视图\|DeltaView' crates/` 零命中（命令无输出）。B8、B4 的步数是装置 `scripts/experiment-apparatus/e206_fixed_point_cost.rs` 的 `fn by_delta_rows`（305 行）按 E203 登记第五节式子代入产物里量的输入算的，不是在 B8 实现上量的。B4 的拼法在 `scripts/experiment-apparatus/e203_admission_scan_cost.rs` 的 `fn narrowed_steps_by_segment`（471 行）。
- B1 的步数是产物里 `name=row` 行的 `steps`（在拷来的 `crates/` 上插计数量的）。固定点的轮数上界在 `crates/singlefs-core/src/transaction.rs` 的 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`（`grep -n` 命中 1974、2045 行）。
- 产物：`ls research/results/e206-fixed-point-cost-*-2026-11-09.out | grep -vc report` 得 40。其中 2 份是 `plain` 构建（不插计数，没有 `name=row`），6 份是对照（`closure_off`、`single_step`），其余 32 份是 16 个点 × 两臂（p1、p2a）。判据读数只用 p1、p2a 这 32 份；p1 的逐轮读数按 E206 页是「作废，仅作线索」，下面 p1 的格都当线索写。
- 装置的轴 `scripts/experiment-apparatus/e206_fixed_point_cost.rs` 第 1391 行 `const AXES: [(&str, [&str; 4]); 3] = [`：E206 的 Q8 只有 files、data_units、unlinks 三根轴，没有盘容量（单元区大小）那根轴；`q8_peak` 在同文件 2059 行 `axis_line("q8_peak", arm, axis, &points, items, &|item| {` 取量窗全部发布的最大值。

## 先校验我的重算（校验路径会不会错）

我从产物逐次发布重写了 B8 的式子（同一套式子，`extract.py` 的 `by_delta`），对合并报告里的 `name=q8` 行比，再单独核一个装置没打的恒等式（`checks.py calls`）：

```text
$ python3 -I checks.py validate pubs.tsv research/results/e206-fixed-point-cost-report-2026-11-09.out
missing ('files_1000_units_100__p1', 'remounted') ('unread', 'unread')
report_q8_lines 76 same 75 diff 0 missing 1
$ python3 -I checks.py calls research/results
publishes 6465 calls==4*sum_rel 6465 other 0
```

- 75 行 `first_publish` 与 `peak` 逐位相同，0 行不同，1 行报告自己写 `unread`（量的）。
- 6465 次发布（p1、p2a 两臂全部窗内发布）里 `allocator_release_linear_find` 的调用数都等于「各轮 `placements_released` 之和 × 4」（量的）。这条恒等式对分母的选法很要紧，见 Q1。
- 重算只印证 B8 的数（推的那一路）。B1、B4 的数是我按 `ROW_TABLE`（`e203_admission_scan_cost.rs` 775 行）的「扫」类行把产物的 `steps` 加起来、按 B4 的规则拼出来的，合并报告没有这两列，**没有装置的数可对**；只对了一处：`whole_copy_elements`（`name=state` 行）与 `name=row … row=whole_allocator_copy_before_the_judgement` 的 `steps` 同值（`files_10000` p2a 第一次发布两处都是 881216，量的）。

## Q1：分母写成哪个量；大改动能不能撑大分母遮掉随总量涨的项

### 1.1 分母取「第一轮释放数 + 第一轮重写角色数」

`T(t) = round=1 行的 placements_released + roles_rewritten`（下限 1），取自产物 `name=round` 行，一次发布一个值，只依赖用户这一次改动本身、不依赖后面固定点走几轮。

- 两项都要。窗内第一轮 `roles_rewritten` 大于 `placements_released` 的有 1186 次、共 7717 次发布（量的：`extract.py` 输出 `pubs.tsv`，`rw_first > rel_first`；其中 1184 次在 W3（建一个文件、写、再 unlink），2 次在 W2；新写的角色没有旧落点可释放）；反过来 `data_units_10000` 同一次挂载 W2 第 1 次第一轮释放 10145、重写 7（`out/bigchange.txt` 的 T 列：24、114、1028、10152，加起来是 T，量的）。单取释放漏新写，单取重写漏整文件替换。
- 两项相加不是重复计数：释放数是旧版被换掉的单元，重写数是新版新写的角色，物理上是两批槽。

### 1.2 不取后几轮的量（对照，量的，`out/denominators.txt`；一单元覆盖写 W1 第 1 次，同一次挂载，B8 步数）

```text
arm point rounds T_first rel_last final_roles sum_rel B8 | B8/T_first B8/rel_last B8/sum_rel
p2a base 2 14 14 14 21 2102 | 150.1 150.1 100.1
p2a files_10000 2 22 84 84 95 15440 | 701.8 183.8 162.5
p2a files_1000_units_100 3 30 448 452 911 168373 | 5612.4 375.8 184.8
p1 files_10000 22 22 84 84 1201 152344 | 6924.7 1813.6 126.8
p1 files_1000_units_100 64 30 432 0 18922 2701726 | 90057.5 6254.0 142.8
```

- 分母取各轮释放数之和（`sum_rel`，也就是 B8 式子里 s24 那一行吃的东西）：`B8/sum_rel` 在全部 16 个点两臂上落在 100 到 185 之间（`denominators.txt` 整表）。P1 在 `files_10000` 走 22 轮、步数从 2102 到 152344（72 倍），这个比值只从 100.1 到 126.8。原因是产物里每次发布 `allocator_release_linear_find` 的调用数恒等于 4 × 各轮释放数之和（`checks.py calls`：6465 / 6465），B8 最大的一项 `allocator_release_linear_find` 就是调用数 × lg(记录数)，其余项在这批产物上也跟着它走（`B8/sum_rel` 100 到 185，量的），除以它几乎是定义出来的常数。这正是登记自己防过的那一句「门槛不是 B8 定义的同义反复」（`research/prompts/e203-preregistration.md:331`）。所以**各轮之和不能当分母**。
- 分母取最后一轮释放数（`rel_last`，固定点收敛后碰到的全部单元）：把固定点连锁带出来的角色吞进分母。`files_1000_units_100` p2a 的一单元覆盖写，第 1 轮重写 15，第 2 轮 450（`research/results/e206-fixed-point-cost-files_1000_units_100-p2a-2026-11-09.out:546`，`round=2 roles_rewritten=450 nodes_rewritten=431 placements_released=448`），收敛后 `final_roles=452`（同文件 520 行）：同一个一单元改动，碰到的单元从 base 的 14 涨到 448（量的）。分母取它，比值只从 150 到 376；取第一轮，比值从 150 到 5612，连锁的涨在分子侧露出来。三臂共用的连锁涨法不该被分母吞掉，**取第一轮**。
- 只取释放数、只取重写数的缺口见 1.1。

### 1.3 大改动撑大分母的三条路与我的堵法

1. 靠各轮之和稀释：上面 `B8/sum_rel` 一行已证（P1 的 72 倍落成 1.27 倍）。堵法：分母只取第 1 轮。
2. 靠常数项稀释：每次发布有一块与改动大小无关的底（B1 在 `base` 上一次发布约 2.5×10⁶ 步，量级由整份拷分配器给：`whole_copy_elements` 约 8.8×10⁵，调用方段一次加固定点每轮一次，推的）。把所有发布都除以 T，小改动那一端被底撑得很高，大改动一端把底摊薄，端到端 α 被压低。实例（量的，`out/bigchange.txt`，`data_units` 轴同一次挂载 W2 第 1 次，P1、P2a 同值）：B1 每单元 106777.5、23365.6、11543.9、129170.2，端到端 α 0.028（「不随」），可最后一个十倍段的局部斜率 1.049。这是线性查找随记录总数涨的真增长，被底盖住：同一格 `row=allocator_release_linear_find` 每次调用的步数 base 425.6（`e206-fixed-point-cost-base-p2a-2026-11-09.out:8359`，calls=164 steps=69798）、`data_units_10000` 10645.1（`e206-fixed-point-cost-data_units_10000-p2a-2026-11-09.out:8347`，calls=122196 steps=1300702006），记录数 1140 到 22334（`name=state` 行），都是量的。堵法写在判据形态里：只有改动大小本身随轴涨的种类才除以 T；别的种类（改动大小在轴上不变，或涨得慢）读原步数、不除；每单元的种类再加「任一相邻两点局部斜率 ≥ 0.5 记随」。
3. 靠大改动盖住「随总量涨的加性项」：`data_units_10000` W2 第 1 次的 B8 是 3231038，一单元改动（W1 第 1 次）的 B8 只有 base 的 2102 与 `data_units_10000` 的 4040（`out/denominators.txt`），占 0.07% 到 0.13%，在比值里看不见。堵法：一单元覆盖写那一类（W1）永远读原步数、不除。

### 1.4 判据的精确形态（我的提案，待判；一个字都没往宽里写）

```text
对象     窗内每次发布（产物 name=settle 一行一次）；列 X = B1、B4 落点判之前（P0+S1）、B4 全段（S2 里没在 P0、S1 调到的行取 B1）、B8。
N_X(t)   该次发布准入之前三段里 X 的扫类步数（B8 照 E203 登记第五节式子；B1 取产物 steps；B4 照 narrowed_steps_by_segment）。
T(t)     第 1 轮 placements_released + roles_rewritten，下限 1。
种类 k   (挂载状态, 窗内操作, 操作序号)，在一根轴的四个点上都有才读（每根轴 160 种，两种挂载状态各占一半）。
读数     T_k 在这根轴上的 α >= 0.5 的种类读 N/T（每单元），其余读 N（原步数，不除）。
α_k      轴上端到端 α（files、data_units、unlinks：10 到 10^4；U：2^14 到 2^24）；每单元种类另看相邻两点局部斜率，任一 >= 0.5 就取那个斜率。
档       三档同 E203：α >= 0.5 随，α <= 0.25 不随，之间不判（不当通过）。
一格     列 × 轴 × 臂；各种里最差的一档：任一种「随」记随；全部「不随」记不随；其余记不判。
轴       files、data_units、unlinks、U（单元区大小）四根，两种挂载状态各自判（种类里带着状态）。
```

阈值 0.25、0.5 与 E203 登记第六节同，不动；唯一新阈值是「每单元种类的局部斜率 ≥ 0.5 记随」，加在翻面一侧、不加在通过一侧（通过仍只看端到端 ≤ 0.25）。

## Q2：记一次输、只许收严、写明收严在哪

照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:101` 起的小节（标题起头是「臂的定义也在」）的三步表（`evidence-discipline.md:106` 起头那句「合法的出路只有三步，缺一步就算新实验，重跑」）。

### 2.1 记一次输（按最弱读法，原样，不删）

F2 字面（`research/prompts/e203-preregistration.md:614`：`Q6 在任一根轴上（任一种挂载状态、任一种被量的发布）记「B8 仍随总数涨」，α ≥ 0.5`）在两臂上都打中，整行（`research/results/e206-fixed-point-cost-report-2026-11-09.out`）：

```text
706:name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255
671:name=q8_peak arm=p1 axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255
660:name=q8_first_publish arm=p1 axis=files low_end=2102 high_end=152344 alpha=0.620 band=grows local_slopes=0.045,0.475,1.340
661:name=q8_peak arm=p1 axis=files low_end=3763 high_end=152344 alpha=0.536 band=grows local_slopes=0.030,0.356,1.221
695:name=q8_first_publish arm=p2a axis=files low_end=2102 high_end=15440 alpha=0.289 band=between local_slopes=0.045,0.295,0.526
```

695 行不是打中（介于，不判），但它也不是「不随」：登记 Q6「全部轴 ≤ 0.25」没满足。p1 的三行出自作废的逐轮读数，只作线索，照抄不删。

### 2.2 只许收严：这一步做不到逐格「更严或相同」，我不把它写成收严

我把旧判据的打中格与新形态的格逐个比（新形态读数在 `out/kinds-*.txt`，量的）：

| 臂 × 轴 | 旧（产物行） | 新形态 | 变化 |
|---|---|---|---|
| p2a data_units | 706 行 α 0.978 随 | 160 种全部不随（每单元那一种：端到端 0.102，局部斜率最大 0.26） | **放宽** |
| p1 data_units | 671 行 α 0.978 随 | 同上 | **放宽** |
| p1 files | 660、661 行随 | 1 种随（W1 第 1 次同一次挂载，α 0.620，与 660 行同一个读数）、31 种不判 | 同 |
| p2a files | 696 行 `q8_peak` 0.204 不随；695 行 0.289 介于 | 157 种不随、3 种不判（0.289、0.263、0.256） | 严（见 2.3 第 2 条） |
| 两臂 unlinks | 不随 | p2a 160 种不随；p1 159 种不随、1 种不判 | 同或严 |
| 两臂 U | E206 的 Q8 没有这根轴 | B8 两臂 160 种全部不随 | 新增格 |

放宽的只有 data_units 轴上的一种发布：同一次挂载、操作 W2、序号 1（`T` 在这根轴上 24、114、1028、10152，α_T = ln(10152/24)/ln(1000) = 0.875，量的 `out/bigchange.txt`）。这是旧判据打中的 706、671 两行背后的同一次发布（B8 3231038，两臂逐项相同）。别的格没有被放宽，没有出现旧判据不中、新形态中的 B8 格。

放宽有理由，也有边：理由是 α 在这一格分不开「随这次改动的大小涨」与「随总量涨」（调查员 `~/.local/share/singlefs/work/e206-b8-data-units-peak/result.md` 第 95–97 行，转述；我独立量了的部分是：每单元步数 156.8、150.8、174.9、318.3，`out/bigchange.txt`）；边是它只覆盖这一种发布。这符合 `evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 的「判别子观测不到」一型（表里写「改判据，改完是新一轮」），与用户定的「新一轮」同向。放宽的格不靠这份判据本身证成：判据是见过这 40 份产物之后才定的，这 40 份只能当回扫（`evidence-discipline.md@不许先有结论再建模型` 第一问「这个模型是不是先于我的倾向就有了」，答：没有，先有打中、后有判据），证成要靠没见过的数，见 Q3 的「没读的格」与 3.4。

### 2.3 写明收严在哪：新形态比旧形态多要求了什么（`evidence-discipline.md:112`：说不出就是放宽）

1. **每种发布单独算 α，不取各种里的最大值再算 α。** 旧的 `q8_peak` 在每个点上先取全部发布的最大值，两端可能取自不同的发布（`scripts/experiment-apparatus/e206_fixed_point_cost.rs:2059`）。新形态同一种发布在四个点上各读一个数。p2a files 轴因此从 696 行的「不随」变成「3 种不判」（量的 `out/kinds-p2a-B8.txt`）。
2. **加盘容量那根轴（单元区 2¹⁴ 到 2²⁴，四点）。** `fs-design.md:23`（运行时决策路径「不许，且代价不许随盘容量增长」）字面说的是盘容量，E206 的 Q8 三根轴（`e206_fixed_point_cost.rs:1391`）都不是它。
3. **每单元的种类加局部斜率翻面。** 端到端 α 会被小改动端的常数底压低（1.3 的第 2 条：B1 的 0.028 对 1.049）；局部斜率只加在翻面一侧。
4. **「不判」不记通过。** 旧登记也这样写（`research/prompts/e203-preregistration.md:322`：`两者之间记「介于两档」、不判`），新形态逐格报「不判」的种类数。
5. **T 本身涨得慢也不记功。** α_T 不到 0.5 的种类读原步数、不除以 T。如果一律除以 T，files 轴上 T 从 14 涨到 22 会把 α 往下拉约 0.07，我不让它拉。
6. **两种挂载状态各自判，不合并。** 种类里带着状态。

### 2.4 什么现象会推翻 Q2 的结论

- 没见过的发布上，B8 的每单元局部斜率 ≥ 0.5，或某一种改动大小不变的发布在新点上 α ≥ 0.5：放宽的那一格盖住了真增长。
- 再造一种「改动大小随轴涨、但步数随总量涨」的发布，新形态判不随：判据分不开。

## Q3：40 份产物上 B1、B4、B8 各判什么；`files_then_big_*` 还要不要量

三列用同一把尺（Q1 1.4 的形态）。数全是我从 `pubs.tsv` 现算的（量的输入、推的式子：B8、B4 的步数是式子代入，B1 是产物里量的步数），输出在 `out/kinds-<臂>-<列>.txt`。每格写「各种发布的档计数」：`no`＝不随，`between`＝不判，`grows`＝随；一格的档取最差的。p1 的格出自作废的逐轮读数，只作线索（标「线索」）。

### 3.1 p2a（现行臂）

```text
列        files                          data_units                     unlinks         U
B1        不判（60 种 between，最大 0.383） 随（1 种 grows：W2 第 1 次，每单元局部斜率 1.049） 不随（160）      随（160 种全 grows，最大 0.927）
B4 P0+S1  不随（160）                     不随（160）                     不随（160）      不随（160）
B4 全段   不判（98 种 between）            随（1 种 grows）                不随（160）      随（160 种全 grows）
B8        不判（3 种 between：0.289、0.263、0.256；157 种不随） 不随（160）  不随（160）      不随（160）
```

### 3.2 p1（线索）

```text
列        files                               data_units                 unlinks               U
B1        随（1 种）、73 种不判                 随（1 种）                  不随（160）            随（160）
B4 P0+S1  不随（160）                          不随（160）                 不随（160）            不随（160）
B4 全段   随（1 种）、105 种不判                随（1 种）                  不随（160）            随（160）
B8        随（1 种：W1 第 1 次，α 0.620）、31 种不判  不随（160）          不随（159）、1 种不判  不随（160）
```

### 3.3 三列各判什么（只说读数，不选臂）

- **B1**：随。盘容量轴 160 种发布全部随，最大 0.927（`out/kinds-p2a-B1.txt`），常数底就是整份拷分配器；data_units 轴那一种大改动的每单元读数在最后一个十倍段涨 1.049。与 E203 页 B1 的判（`.claude/kb/experiments/203-准入之前扫类的步代价.md:54`：`B1 今天 | 够判：留不住`）同向，这一步没有改它。
- **B4**：要分开说。E203 登记里 B4 判的是落点判之前那两段（`e203-preregistration.md` Q8，P0+S1，臂定义「条款只管 P0 与 S1」）：这两段步数每臂四根轴 640 种、两臂共 1280 种全部不随（常数 36 到 191 步，`B4_pre`）。两道之间那一段（S2）里没被改的行仍取 B1，全段合计在盘容量轴上随（160 种）、data_units 轴随（1 种）；这是 B4 条款本来就不管的那一段，只报。E203 页 B4 行记着另一件事：「覆盖写的式子判就是 D28（挂载期承诺量） 已定项 1 的 ENOSPC 准入，需求取自两道之间算出的结果，读法越位」（`.claude/kb/experiments/203-准入之前扫类的步代价.md:56`），那是读法问题，归判决，这一步不碰。
- **B8**：p2a 上新形态不翻面（没有任何一种「随」），但也不是干净的「不随」：files 轴 3 种不判。这 3 种都是改动大小不变、碰到的单元自己在涨的发布（T 在 files 轴上 14 到 22，收敛后碰到的单元 14 到 84，`out/denominators.txt`），固定点连锁带出来，三臂共用的那一部分（E206 的 Q3 `final_roles`，p2a files 轴 22 到 84，`research/results/e206-fixed-point-cost-report-2026-11-09.out:690` 的 `alpha=0.194 band=does_not_grow`，它取的是全部发布里的最大值，不是同一种发布，所以与上面 3 种不判不矛盾）。p1（线索）在 files 轴照旧翻面，与旧判据同一个读数（660 行 0.620）。旧判据在 data_units 轴上的两处打中（706、671）在新形态下不再打中，这是 Q2 记下的唯一放宽。

### 3.4 没读的格（不当通过）与轴外的点

1. **每单元读法只在一根轴上有格**：`per_unit_kinds` 在 files、unlinks、U 三根轴上是 0（量的，`out/kinds-*-B8.txt` 每行 `per_unit_kinds=`），data_units 轴只有 1 种（同一次挂载 W2 第 1 次），重挂载那一半也没有。原因是窗内操作在这些轴上改动大小几乎不变（W1 第 1 次的 T 在 files 轴四个点上是 14、14、20、22，`out/denominators.txt` 与 `out/kinds-p2a-B8.txt` 的 T 列）。所以「步数只随这一次碰到的单元数涨」这句话，在 40 份产物里只有一根轴上被检验过；别的三根轴上的 B8「不随」靠的是固定大小改动的原步数读数，不是每单元读数。要补需要一种窗内操作，改动大小随 files、unlinks、U 的值涨（如整批删掉一部分文件）：新装置，交实验设计员，不在这一步。
2. **轴外的点**：`files_30000`、`files_1000_units_10`、`files_1000_units_100` 不在 E206 的三根轴上（`e206_fixed_point_cost.rs:1391`），新旧形态都没读。我把 16 个点混在一起，用一单元覆盖写第 1 次（同一次挂载）的 B8 对活单元数（`name=state` 的 `mapped_units`）做了一次回归，输出 `out/offaxis.txt`：p2a 的 OLS 斜率 0.419、两端点 α 0.55；p1 0.65、0.844；B4 P0+S1 −0.034（量的输入、我的组合读法，**登记里没有这根轴，零轮**）。p2a 的 0.55 是 `files_1000_units_100`（`mapped_units` 101581，B8 168373，碰到的单元 448）撑起来的。这决定新判据里要不要加一根「活单元总数」轴，我不替主 agent 定；加了，p2a 在这一格会打中。

### 3.5 `files_then_big_*` 要不要量

- **判 B1、B4、B8 不需要。** 「总量在、改动不在」那类发布窗里已有：一单元覆盖写第一个文件（W1）在 `data_units_*` 点上总量含那个大文件、改动不碰它（T 14 到 18，B8 2102 到 4040，`out/denominators.txt`）；重挂载之后的 W2（碰的是小文件）T 恒 14，B8 2252 到 5134（`out/kinds-p2a-B8.txt` data_units 轴，`('remounted', 'W2', '1')`，α 0.119，量的）。`files_then_big_*` 的 files 与 data_units 同时涨（登记第五节取样点表），拆不出哪根轴。
- **只在一件事上还有用**：新形态是见过这 40 份产物之后定的（Q2 2.2），它们当不了证实；`files_then_big_*` 没跑过，是少有的没见过的点。要拿它当证实就跑前写死：p2a 在 `files_then_big_10000` 上，每一种发布的 B8 不超过 `files_10000` 同一种的 2 倍（`files_10000` 的最大是 15440）；超过 2 倍的那一种点名、按「推翻」记。2 倍是我的取法（推的：总量翻一倍加对数项）。
- 调查员的判读线（`result.md` 第 133–138 行）是「< 100000 支持、≥ 1000000 推翻」，落在中间不判；这一条比它严。入口在主工作区起，主工作区已带 P2a，测量补丁锚点可能对不上（调查员的话，转述，推的）。

## Q4：运行时决策路径「代价不许随盘容量增长」，新判据还罩不罩得住

句子在 `.claude/rules/fs-design.md:23`：`不许**，且代价不许随盘容量增长`（整行：运行时决策路径一格）。D8（核心索引结构） 已定项 16 射程另一句在 `.claude/kb/decisions/08-核心索引结构.md:516`：`准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径`。

罩得住的：

- **盘容量，字面意义的那根轴。** 新形态把单元区大小（2¹⁴ 到 2²⁴）加成一根轴。B8 两臂 160 种发布全部不随，B1 160 种全部随（`out/kinds-p2a-B8.txt`、`out/kinds-p2a-B1.txt`，量的输入）。分辨力有一次现场验证：我把 B8 的每次发布加上 B1 在调用方段与落点判之前的步数（模拟「B8 在这两段退回 B1」，`criterion.py defect`），U 轴 160 种立刻全部变成随（`out/defect-p2a-B8.txt` 的 U 行 `{'grows': 160}`）；files 轴原来的 3 种不判被注入的常数底（约 2.5×10⁶ 步）盖成了 160 种不随（同文件 files 行 `{'no': 160}`），这是注入的底太大的副作用，不是判据放宽。
- **改动大小不变的发布，对 files、unlinks、data_units 三根轴。** 读原步数、不除，与旧判据同一个量；所以「分母撑大遮掉总量项」在这一类上没有可乘之机。

罩不住的：

1. **「遍历」的语义。** `fs-design.md:33` 起头的那条 ⚠️ 说被禁止的不是遍历，是「在需要答案的那一刻还没有答案」与「审计与被审计用同一段代码」。步数量的是走了多少步，量不出答案在不在。新判据没有碰这一半，旧判据也没有。
2. **碰到的单元自己随总量涨。** 一单元覆盖写（W1）在 `files_1000_units_100` p2a 上第 1 轮重写 15，第 2 轮 450（`research/results/e206-fixed-point-cost-files_1000_units_100-p2a-2026-11-09.out:546`），收敛后 452 个角色（同文件 520 行）；base 上是 14（`out/denominators.txt`）。三臂共用（固定点的不动点两臂相同：`.claude/kb/experiments/206-发布固定点轮数的代价.md` 的 F3 一条写「没打中，P2a 与 P1 的不动点 15 行逐个相同」），新形态对它的读法是 files 轴 3 种「不判」。如果把 `fs-design.md:23` 读成「一次改动的总代价」，这一块仍然随盘上文件数涨，新判据既不判它过，也没有为它另设门槛；它被留在「不判」里，没有被分母吞掉（Q1 1.2 的取第一轮，就是为了这个）。
3. **改动大小随总量涨的发布。** 整文件替换 10⁴ 个单元的 B8 3231038 步，新形态记「每单元不随」。D8 射程那句话写的是「不随盘上容器总数、记录总数涨」，字面上这一次发布的步数是随记录总数涨的；新形态把「总数」读成「没碰到的那部分」。**这是读法的改动，不是量出来的**：采用新判据，这句话与 `fs-design.md:23` 的措辞（以及 `research/prompts/e203-preregistration.md:614` F2 行「每次发布准入之前扫类步数只随这一次碰到的单元、容器、记录数（加树高）涨」）与判据不再逐字相符。要不要改措辞是判决与用户的事；改措辞放宽了条款，也要照 `evidence-discipline.md:106` 起的三步走，我不替它们写。
4. **每单元读数只有一根轴上的一种发布**（Q3 3.4 第 1 条），别的轴上「只随碰到的单元涨」没被检验。
5. **轴外点**（Q3 3.4 第 2 条）：16 个点混在一起，p2a 一单元覆盖写的 B8 对活单元数端点 α 0.55。这根轴登记里没有，加不加由判决定。

小结：「盘容量」那根轴罩得住；「遍历的语义」罩不住（步数量不了）；「碰到的单元自己涨」与「每单元读数的覆盖」是新判据留下的洞，报告里列成「不判」与「没读」，没有记成通过。

什么现象会推翻 Q4：在单元区大小轴上新点（比 2²⁴ 更大，或与 files 同时涨）出现 B8 的固定大小种类 α ≥ 0.5；或再造一个窗内操作，让改动大小随 files 轴涨，B8 的每单元读数局部斜率 ≥ 0.5。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 分母 | 取第 1 轮 `placements_released + roles_rewritten`（下限 1） | 各轮之和是同义反复（calls = 4 × 各轮释放数之和，6465 / 6465），最后一轮吞连锁；单取一项漏另一类发布 |
| Q1 大改动撑分母 | 有三条路，已堵 | 各轮之和稀释、常数底稀释（B1 0.028 对 1.049）、大改动盖加性项（固定大小种类读原步数） |
| Q2 记一次输 | 已记 | 706、671、660、661 行打中，695 行介于，原样不删 |
| Q2 只许收严 | 做不到逐格；放宽 1 格族 | 放宽的是 data_units 轴同一次挂载 W2 第 1 次（两臂同值 3231038）；其余同或严 |
| Q2 写明收严在哪 | 6 条 | 每种单独算 α、加盘容量轴、局部斜率翻面、不判不当通过、T 涨得慢不记功、挂载状态分开 |
| Q3 B1 | 随 | U 轴 160 种全随；data_units 每单元局部斜率 1.049 |
| Q3 B4 | 落点判之前（P0+S1）不随；全段随 | 640 种每臂全不随；S2 里没改的行取 B1，随 |
| Q3 B8 | p2a 不翻面也不是干净的不随；p1（线索）files 翻面 | p2a files 轴 3 种不判（0.289、0.263、0.256）；其余 637 种不随 |
| Q3 `files_then_big_*` | 判三列不需要；只作没见过的点的证实 | 总量在改动不在的发布窗里已有；若跑，跑前写死「不超过 files_10000 同种的 2 倍」（推的） |
| Q4 | 盘容量轴罩得住；遍历语义、碰到的单元自涨、读法措辞罩不住 | 洞列在 Q4 的 2 至 5 条，记「不判」「没读」，不记通过 |

## 要主 agent 跑的命令

1. 复跑模型（纯读 `research/results/` 下 E206 的 40 份产物，重算 `pubs.tsv` 与全部读数，对 `out/` 逐份 `diff -r`）：

```text
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=三方腿模型 e203-b8-criterion-r1-sonnet-model 的 rerun.sh，纯读 E206 的 40 份产物重算读数; 范围=1 个脚本、读 40 份产物、不编不起被测程序; 为什么=复跑正推腿报告里新判据的读数，对 out 目录里 16 份留存输出逐份比' task-server/task submit -- bash research/prompts/e203-b8-criterion-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/e203-b8-criterion-r1-snapshot --work /home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r1-rerun --model /home/fy5090/code/singlefs/research/prompts/e203-b8-criterion-r1-sonnet-model
```

   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。`--frozen` 目录这个脚本不读（只为过模板的参数检查）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=0`；`task-server/task log <号>` 里有 `name=rerun_roots … products=40` 与 `name=rerun_compared files=16 differences=0`；没有 `✗` 行。任何一份 `diff` 有差异，整条判红、报告里的数作废重算。
   - 前提：新 shell 里起，`--work` 是不存在或空的目录。
2. 我没有对 `rerun.sh` 本身跑过：我在草稿目录里拷同一份 `extract.py`、`checks.py`、`criterion.py` 直接跑了同样的 16 步（只读产物、不经服务），输出就是 `out/` 里的 16 份；`rerun.sh` 里的 shell 部分（`mapfile`、`diff -r`、目录拼接）只做过 `bash -n`。所以 1 若红在 shell 部分而不是读数，先修脚本再谈数。

`out/` 16 份的 sha256（`sha256sum research/prompts/e203-b8-criterion-r1-sonnet-model/out/*` 现算）：

```text
3b73710186ed397394494e61f995a125f507735a62a2b059d058ddece05a4edf  out/bigchange.txt
54b73498092a2d1a624e371fe1c7c83ecf59c8bcda0d3936aa61ba45824b0cc4  out/calls.txt
db1575e3a572b25dd77a60b364c552ac6d6529e969c0621e2e94b9796657a961  out/defect-p1-B8.txt
4cdfc43f84a936c319dda47b7058b1fb04709e0911572a51dc4af18cbaa2eb0f  out/defect-p2a-B8.txt
41251a616fc662ddc0fb38fc5b898f6bdbf5742e115aa5df494636f092ffb50f  out/denominators.txt
32c204d9e429ad2b32f75abe305ed36bc8b017b2149aa60ba6dbd2189f07717d  out/kinds-p1-B1.txt
285df42bf57e4b7b35822c457af68ee8643aad48c606409b14ac936be70ce93d  out/kinds-p1-B4_pre.txt
297123896b34918a52bc4da080450ae96b056a3390f973580870e85f36ef1a1d  out/kinds-p1-B4.txt
333739c80bf6707e6c048fc9c6f4bfd70c8b5eca98c435183ca18e7499d672ff  out/kinds-p1-B8.txt
cbcbee5379313bba04a32c596854670d0dc7c383aa081c9401d15ae2fb07fc7c  out/kinds-p2a-B1.txt
fa38b70edf27b58a354b4f1a075bde6bd584789aac990d984b243ee4e90294a3  out/kinds-p2a-B4_pre.txt
f3ef509c0b3e29d22e987fe53dcadb79f8865dcf30c66468a79261abcffa56ff  out/kinds-p2a-B4.txt
d89cc641230b5c691be07817c5ac36ff2fc0fd04c629f050b9d4ab7e6c231ad8  out/kinds-p2a-B8.txt
5cd08bf25fce1934e265ea364a4ff667769d041b3edb78aef22bc97dc3e76e74  out/offaxis.txt
f9e3d5239b4c74a11c0f488d71dded0dfc6bfda5c5446231901f7467d6889b75  out/pubs.sha256
c4d66c372f514ab2b6e6e32d51386a1d77cf83f43f8c85e2f2986e5f337720fc  out/validate.txt
```

## 没做什么

- 没选臂，没写「选 P1 还是 P2a」「选 B1、B4、B8 哪条」，没判「B8 仍随还是不稳定」；上面「p2a 不翻面也不是干净的不随」是对新形态读数的描述，不是判决。
- 没改 kb、规则、登记、装置；没改 D8（核心索引结构）射程那句与 `fs-design.md:23` 的措辞，只列了读法与措辞不再逐字相符（Q4 第 3 条）。
- 没跑 `files_then_big_*`，没起任何实验、没往统一任务服务交任务；判决要引的数（本报告所有读数）都是我在草稿目录里直接跑出来的，主 agent 复跑之前一律按「推的」。
- B1、B4 的步数我自己拼的（B8 的数对了合并报告 75 行，B1、B4 没有装置的数可对，只对了整份拷那一项，见「先校验」）。
- 没证明新形态的每单元局部斜率 0.5 这一条是不是最合适的翻面线：它只让 B1 的 data_units 轴翻面、没让 B8 翻面，B8 同一格的最大局部斜率是 0.26（`out/bigchange.txt`，最后一个十倍段：释放查找的对数项 12 到 15 加一轮，推的），离 0.5 不远处还有 0.25 的通过线；这条线是我取的，没被攻过。
- 没量「改动大小随 files、unlinks、U 涨」的发布，每单元读数在这三根轴上是空的（Q3 3.4 第 1 条）。
- 没读第二份样本：这条腿只抽了一次。「没打中」（B8 p2a 新形态下没有任何一种随）一次不算，按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 要再抽一次才记。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r1-sonnet`（4.4M，没有仓副本与编译目录，只有 `pubs.tsv`、脚本与输出）。

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/e203-b8-criterion-r1-sonnet-output.md` 的末行（贴的是这一次真跑的原样；「没判」的裸行号引文，上面每处都配了原文片段或命令，写报告时现 grep 过）：

```text
  ✓ 核了 2 处引文、23 处裸行号，对上 2 处，没判 23 处（其中裸行号没有引文可比 20 处；逐处列在上面）
```
