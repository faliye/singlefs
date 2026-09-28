文件：.claude/kb/decisions/23-journal的角色与格式.md
旧串：
```
| G23.2 | 甲丙差不收敛 | 攒批变大时，甲与丙的差**不**收敛 | **已答（对乙），但有界**：E16（journal 的角色：WAL vs 意图日志） 多流负载上攒批 1 → 10 时 3.40× → **3.71×**，不收敛反而扩大；**而这只在「批 ≤ 流数」时成立**——56 格扫描显示峰值落在批 ≈ 流数，16 流那一行批 50 已降到 1.99、批 200 降到 1.24。⚠️ 丙尚未测 |
```
新串：
```
| G23.2 | 甲丙差不收敛 | 攒批变大时，甲与丙的差**不**收敛 | **「差」取甲 / 丙每批块数之比**（用户 2026-09-28 定；今天用乙的数代丙）：E16（journal 的角色：WAL vs 意图日志） 第五次跑显示，按今天的宽度收不收敛随树高变，不是「批 ≤ 流数」的界——真实基线 1 TiB（4 层、根下 11 个孩子）上流数 ≤ 16 先涨后收敛（峰值在批 ≈ 流数），流数 ≥ 24 从批 = 1 起就收敛（`research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out` 的 `row_summary` 行，流数 64 那一行第 1818、1840 行：批 1 到批 64 比值从 3.500000 降到 3.169231）；树高 ≤ 3 全部流数从批 = 1 起收敛，树高 ≥ 5 扫到的流数全部先涨后收敛，树高 4 时门槛约为 2 × 根下孩子数（只在三方攻方副本上量过，主 agent 复跑逐字节相同，入库装置没重做，判决 `research/prompts/d23-e16r5-r1-main-verification.md` 第一节汇总表）。原先「3.40× → 3.71×」那组数是旧宽度（扇出恒 128、树高恒 4）下的。新写法被攻过零轮。⚠️ 丙尚未测 |
```
依据：判决 research/prompts/d23-e16r5-r1-main-verification.md 第一节汇总表、第四节判定第二条；用户第六节第 1、3 题定案

文件：.claude/kb/decisions/23-journal的角色与格式.md
旧串：
```
- E16（journal 的角色：WAL vs 意图日志）：延后祖先的臂只在 checkpoint 发根、从没在 fsync 时发过根；攒批吃不掉延后祖先的收益只在「批 ≤ 并发流数」时成立——它改的是「乙的收益形状」那条已降级依据的强度，不是结论。
```
新串：
```
- E16（journal 的角色：WAL vs 意图日志）：延后祖先的臂只在 checkpoint 发根、从没在 fsync 时发过根；攒批吃不掉延后祖先的收益随树高变（第五次跑：树高 ≤ 3 全部流数从批 = 1 起收敛，树高 4 门槛约为 2 × 根下孩子数，树高 ≥ 5 扫到的流数全部先涨后收敛）——它改的是「乙的收益形状」那条已降级依据的强度，不是结论。
```
依据：判决 research/prompts/d23-e16r5-r1-main-verification.md 第一节汇总表、第四节判定第二条；用户第六节第 3 题定案

文件：.claude/kb/decisions-history.md
旧串：
```
- 依据：`research/prompts/c245-r3-main-verification.md`（快照全 OK、引文 36 / 36、攻方探针在整仓副本上复跑逐行相同；按跑前条款第一条甲站住）。
```
新串：
```
- 依据：`research/prompts/c245-r3-main-verification.md`（快照全 OK、引文 36 / 36、攻方探针在整仓副本上复跑逐行相同；按跑前条款第一条甲站住）。

#### D23（journal 的角色与格式） 已定项 1：门禁表 G23.2（甲丙差不收敛） 与依据段 E16（journal 的角色：WAL vs 意图日志） 那一条前半句，按比值读法改写为随树高变

> 快查·改前：G23.2（甲丙差不收敛） 那一格写『已答（对乙），但有界』，界定条件是『批 ≤ 流数』，用的是旧宽度下 3.40× → 3.71× 那组数；依据段 E16（journal 的角色：WAL vs 意图日志） 那一条前半句写『攒批吃不掉延后祖先的收益只在「批 ≤ 并发流数」时成立』。
>
> 快查·改后：『差』定为甲 / 丙每批块数之比（今天用乙的数代丙，丙尚未测）；G23.2（甲丙差不收敛） 那一格改写为收不收敛随树高变（真实基线 1 TiB 流数 ≤ 16 先涨后收敛、流数 ≥ 24 从批 = 1 起收敛，树高 ≤ 3 全部收敛、树高 ≥ 5 全部先涨后收敛，树高 4 门槛约为 2 × 根下孩子数），标明旧数据是旧宽度下的、新写法被攻过零轮；依据段前半句同步改写，后半句不动。

- 改前：`.claude/kb/decisions/23-journal的角色与格式.md` 第 77 行 G23.2（甲丙差不收敛） 那一格是『**已答（对乙），但有界**：E16（journal 的角色：WAL vs 意图日志） 多流负载上攒批 1 → 10 时 3.40× → **3.71×**，不收敛反而扩大；**而这只在「批 ≤ 流数」时成立**——56 格扫描显示峰值落在批 ≈ 流数，16 流那一行批 50 已降到 1.99、批 200 降到 1.24。⚠️ 丙尚未测』；第 86 行依据段 E16（journal 的角色：WAL vs 意图日志） 那一条前半句是『攒批吃不掉延后祖先的收益只在「批 ≤ 并发流数」时成立』。
- 改后：G23.2（甲丙差不收敛） 那一格改写为『「差」取甲 / 丙每批块数之比……树高 4 时门槛约为 2 × 根下孩子数……新写法被攻过零轮』（正文见 D23（journal 的角色与格式） 已定项 1）；依据段 E16（journal 的角色：WAL vs 意图日志） 那一条前半句改写为『攒批吃不掉延后祖先的收益随树高变』，后半句『它改的是「乙的收益形状」那条已降级依据的强度，不是结论』不动。
- 用户原话：2026-09-28 弹窗第 1 题选『比值』（G23.2（甲丙差不收敛） 的『差』定为甲 / 丙每批块数之比）、第 3 题选『不走，直接写回』（G23.2（甲丙差不收敛） 那一格与依据段 E16（journal 的角色：WAL vs 意图日志） 那一条照第一节汇总表与入库产物写，标『被攻过零轮』）（`research/prompts/d23-e16r5-r1-main-verification.md` 第六节）。
- 依据：`research/prompts/d23-e16r5-r1-main-verification.md` 第一节汇总表、第四节判定第二条、第五节第 1、3 题、第六节用户定案；`research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out` 第 1818、1840 行。
```
依据：判决 research/prompts/d23-e16r5-r1-main-verification.md 第六节用户定案；D23（journal 的角色与格式） 已定项 1 正文改动同批

文件：.claude/kb/experiments/16-journal的角色WALvs意图日志.md
旧串：
```
其中扇出旋钮的四个点（`one_tebibyte_file`、`full_root_level_four`、`small_fanout_one_tebibyte_file`、`large_fanout_one_tebibyte_file`）单元数都是同一个 1 TiB 数（33692212），树高恒为 L=4，唯一变的是根下孩子数——11（今天真实扇出 144/147）、147（G3，满员）、17（小扇出 128/128）、2（大扇出 296/296）。
```
新串：
```
其中扇出旋钮的四个点（`one_tebibyte_file`、`full_root_level_four`、`small_fanout_one_tebibyte_file`、`large_fanout_one_tebibyte_file`）树高恒为 L=4，唯一变的是根下孩子数——`one_tebibyte_file`、`small_fanout_one_tebibyte_file`、`large_fanout_one_tebibyte_file` 三点单元数同为 1 TiB 数（33692212），根下孩子数分别是 11（今天真实扇出 144/147）、17（小扇出 128/128）、2（大扇出 296/296）；`full_root_level_four`（G3，满员）单元数是 457419312、根下孩子数 147。
```
依据：research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out 第 3509 行 geometry_config full_root_level_four units=457419312

文件：.claude/kb/experiments/16-journal的角色WALvs意图日志.md
旧串：
```
**第二段：「流数 ≥ 24 时峰值落到批=1」由根下孩子数主导，不是树高。** 第二段按扇出、记录宽两个旋钮各跑一遍网格，
```
新串：
```
**第二段：「流数 ≥ 24 时峰值落到批=1」由根下孩子数主导，不是树高。** ⚠️ 被三方打中、待重做（判决 `research/prompts/d23-e16r5-r1-main-verification.md`，2026-09-28）：d23-e16r5-r1 攻方把树高、根下孩子数、扇出分开扫，树高是开关（树高 ≤ 3 全部流数从批 = 1 起收敛、树高 ≥ 5 扫到的流数全部先涨后收敛）、扇出零作用、根下孩子数只在树高 4 时定门槛；这条分开扫的数据只在三方攻方副本上量过，主 agent 复跑逐字节相同，入库装置没重做（用户 2026-09-28 定只改抄错的数，不跑第三段）。第二段按扇出、记录宽两个旋钮各跑一遍网格，
```
依据：判决 research/prompts/d23-e16r5-r1-main-verification.md 第一节汇总表、第四节判定第五条；用户第六节第 2 题定案

文件：.claude/kb/experiments/16-journal的角色WALvs意图日志.md
旧串：
```
| D23（journal 的角色与格式） 已定项 1 | 支撑 | 2026-09-28 待三方：第五次跑第二段显示，「流数 ≥ 24 时峰值落到批=1」主要由根下孩子数主导，不是树高——G3（与 G2 同高 L=4，根下孩子数从 11 改成满员 147）上甲行一路延伸到流数≈128–384，大扇出点（同高、根下孩子数=2）上则流数=2 就已经丙行。依据段引的「攒批吃不掉延后祖先的收益只在批≤并发流数时成立」这条适用范围，因此依赖一个此前没单独测过的旋钮（根下孩子数），不是一个只由树高定的常量射程；这句话该不该升级判法、要不要在依据段里补一条条件，留给三方判 |
```
新串：
```
| D23（journal 的角色与格式） 已定项 1 | 支撑 | 2026-09-28 判决 `research/prompts/d23-e16r5-r1-main-verification.md`：定案不受影响，压在『两根轴不正交』这一条结构性依据上，与收益多少无关；G23.2（甲丙差不收敛） 那一格与依据段 E16（journal 的角色：WAL vs 意图日志） 那一条前半句按比值读法、随树高变改写（正文见 D23（journal 的角色与格式） 已定项 1） |
```
依据：判决 research/prompts/d23-e16r5-r1-main-verification.md 回看决策表；第六节用户定案

文件：.claude/kb/experiments-history.md
旧串：
```
- **依据**：`research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out` 的 `name=geometry_config`/`name=q1_summary`/`name=row_summary` 行。
```
新串：
```
- **依据**：`research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out` 的 `name=geometry_config`/`name=q1_summary`/`name=row_summary` 行。

#### 判决 d23-e16r5-r1：full_root_level_four 单元数抄错改正，第二段归因句标『被三方打中、待重做』，G23.2（甲丙差不收敛） 与依据段按比值读法改写

根据判决 `research/prompts/d23-e16r5-r1-main-verification.md`（用户 2026-09-28 定案）：`.claude/kb/experiments/16-journal的角色WALvs意图日志.md` 第 110 行『单元数都是同一个 1 TiB 数（33692212）』改正——`full_root_level_four` 的单元数是 457419312（第二段产物 `research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out` 的 `geometry_config` 行），另外三点仍是 33692212；第二段『由根下孩子数主导，不是树高』那句标『被三方打中、待重做』——d23-e16r5-r1 攻方把树高、根下孩子数、扇出分开扫，树高是开关、扇出零作用、根下孩子数只在树高 4 时定门槛，这条数据只在三方攻方副本上量过、主 agent 复跑逐字节相同、入库装置没重做（用户定只改抄错的数，不跑第三段）。D23（journal 的角色与格式） 已定项 1 定案不受影响；G23.2（甲丙差不收敛） 那一格与依据段 E16（journal 的角色：WAL vs 意图日志） 那一条前半句按比值读法、随树高变改写。

- **依据**：`research/prompts/d23-e16r5-r1-main-verification.md` 第一节、第四节、第五节第 2 题、第六节；`research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out` 的 `geometry_config` 行。
```
依据：判决 research/prompts/d23-e16r5-r1-main-verification.md 第六节用户定案；experiments/16 页正文改动同批

文件：.claude/kb/checks-owed.md
旧串：
```
| C594 | 每次轮换之前现算整池 F 的读开销没量 | 每次发布轮换系统配置之前，每块盘现算一次整池 F（`crates/singlefs-core/src/transaction.rs` 的 `effective_rollback_floor_rereading_unreadable_reads_once`），要读整个根环与最新根的实例表链；这笔读开销随根环槽数与实例表长度涨，没人量过，只由 C245（系统配置槽的写频率，两条条款说反话） 第三轮攻方腿按代码步骤推出 | 一个计数实验：按根环槽数与实例表片数扫，报每次发布这一步读几个槽、几字节，钉绝对值 | 用户 2026-09-28 定先记账不量，等性能那一轮一起量 | 2026-09-28 `research/prompts/c245-r3-main-verification.md` 第三节 Q1 与交用户表第 3 行 |
```
新串：
```
| C594 | 每次轮换之前现算整池 F 的读开销没量 | 每次发布轮换系统配置之前，每块盘现算一次整池 F（`crates/singlefs-core/src/transaction.rs` 的 `effective_rollback_floor_rereading_unreadable_reads_once`），要读整个根环与最新根的实例表链；这笔读开销随根环槽数与实例表长度涨，没人量过，只由 C245（系统配置槽的写频率，两条条款说反话） 第三轮攻方腿按代码步骤推出 | 一个计数实验：按根环槽数与实例表片数扫，报每次发布这一步读几个槽、几字节，钉绝对值 | 用户 2026-09-28 定先记账不量，等性能那一轮一起量 | 2026-09-28 `research/prompts/c245-r3-main-verification.md` 第三节 Q1 与交用户表第 3 行 |
| C595 | 记账伪影与分开扫未进入库装置 | E16（journal 的角色：WAL vs 意图日志） 第五次跑入库装置两处欠：① 256 MiB 那一点同一 checkpoint 窗口重写同一单元时，甲按操作次数重复计块（产物里 1.969697、2.109173 两个远端比值是伪影，攻方副本改正后核得 0.9926）；② 『三个旋钮（树高、根下孩子数、扇出）分开扫』没有并进入库装置，今天分开扫的数据只在三方攻方副本上量过 | 在入库装置（`research/e7-index-bench/src/bin/e16_fifth_run_peak_under_today_widths.rs`）里修记账（同一 checkpoint 窗口内重写同一单元只计一次，不按操作次数重复计）、加三个旋钮各自独立扫描的模式，重跑；钉一条绝对值断言（256 MiB 那一点重写同一单元的远端比值核到 0.9926 附近）与一条变异（改回按操作次数计块，断言必须变红） | 无，等用户开 | 判决 `research/prompts/d23-e16r5-r1-main-verification.md` 第二节 H7、第五节第 2 题、第六节 |
```
依据：判决 research/prompts/d23-e16r5-r1-main-verification.md 第二节 H7；第五节第 2 题；第六节用户定案（只改抄错的数，装置与伪影不动，登记欠账）
