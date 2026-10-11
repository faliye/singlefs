# 第 37 批代码轮第二轮：云端攻方腿（Opus）报告（2026-10-09）

轮名 m3-formal-code-b37-r2；攻第一轮的修补本身，C1、C3、C4，重点 C1、C3。代码引文读冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree/`，写 `crates/…` 相对路径与那棵树里的行号（开工时被引的六份文件与主工作区 sha256 前 12 位逐个相同）。没读禁读清单里的两样。没跑任何 checker 档测试，没编 Rust。

## 复跑

模型只建「一次正式跑出哪几条对照」这一层，纯 Python：

```text
cd /home/fy5090/code/singlefs/research/prompts/m3-formal-code-b37-r2-opus-model
bash tools/rerun.sh        # 默认冻结副本根 ~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree，仓根取这个目录往上三层
f8d05d588a5abf2d5d3abffb4c9cc39233a05fef4c830909d72b8fff057619f4  c13_family_axis.py
4db0e8accad80095c82f8c4815cf1a326b3a179ce83d0cc1cabc74d73d9a14fb  tools/rerun.sh
fbf57a07ea3bb5434b61e277c4484762364d5c65b030448c4fdbffdfd252e3a4  expected-output.txt
```

模型从冻结副本的源码里抽判据枚举、`Criterion::family` 的分族、`is_judged_under` 六条臂、改后单测写死的五份集合与五个调用；从 `scripts/experiments/replay.sh` 抽直接起装置、走这张判据表的 15 条驱动；再拿那 15 条驱动登记的产物里 `name=control criterion=` 的判据集合逐条核模型算的「今天每条正式跑有哪几条对照」（核一：15 条全相同，不同就退 4）。这一道核会红：把第 6193 行换成 `=> Family::LegalHistory,` 的一份假根上，核一 11 条不同、退 4（草稿 `~/.local/share/singlefs/work/m3b37r2-opus/selfred.txt`）。限度：读数一律当「跑了就红」，只管覆盖、不管数值；读数层的几格是读码判的，没进模型。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| C1 写死的五份集合对登记 | 没打中 | 五份集合逐条对得上原登记 5.4、r2 5.4/5.5、r3 5.6/5.7、r5 5.3/5.4（每档判哪几条） |
| C1 单测的族与正式跑的族 | **打中**（口径，C1-甲） | 单测五个调用里四个取 L+A+M，`main` 起不出这种族组合；正式跑的单位是 (档, 族)，15 条里没有一条与单测的哪个调用同档同族 |
| C1 r3 档写了、正式跑不跑 | **打中**（现状，原有，C1-乙） | 单测给 r3 写死 29 条；r3 唯一一条正式跑（对抗一族）只出 8 条对照，却在 A 域判了 J6Qn 与 J7 五格（judged 4 500 万量级）；r2 直接建表那一档同形（写死 28、正式跑 4），按 `--profile` 并上 `--table both` 那几条才齐 |
| C3 族这一维的改坏 | **打中**（C3-甲） | 70 种「一条判据挪族」里单测放过且某条正式跑少了对照的 48 种；族过滤与 `is_judged_in_every_family` 6 种里放过且少了的 3 种；第 6193 行一行改坏让 r3 整档没有一条 A 族对照与拆行对照，单测照过 |
| C3 反盘次序、标记奇偶另一种 | **打中**（C3-乙，造的写法） | 臂上多一个看盘次序或奇偶的条件，12 种里单测全放过，8 种让正式跑的反盘次序或 odd 那几条少了对照 |
| C3 读数「全红」与空集 | **打中**（读数层，C3-丙） | 集合只比到判据一级，一条判据少几个读数单测看不见：r5 odd 下 C8a、C8c 的 C 域读数（登记 r5 5.4 写明的）删掉，哪条单测都不红；J5a_vs_J5b 今天 18 个读数全是 0 = 0 |
| C3 按标签比集合 | **打中**（弱，C3-丁） | 单测拿 `label()` 比；1 190 种标签相撞全放过；两步「撞标签 + 那一条在 r5 不判」照过、正式跑少对照 |
| C3 「只加不删」在五档之内 | 没打中 | 集合相等，五档里多跑一条判据就红；族过滤去掉族条件这类「每族都跑」的只加，单测放过、正式跑只多不少（2 种） |
| C3 `--table scan`、r4 | 没打中 | 正式跑没有一条带 `--table scan`；`--profile r4` 在这张判据表上解成与 r3 同一个档，r4 的正式跑只有 `root_choice_faults` 一族，不走这张表 |
| C3 挂钟 | 没打中（推的） | 改前改后都是 5 个调用；改后少一个带拆行（n = 3）的档，按正式跑的计时推，不比改前长 |
| C4 四行锚点、筛法、红在哪 | 没打中 | 四行原文各命中 1 次；前三行在改后单测上分别只有 r5、两份表都取的 r2、r1 那一个调用判红，在改前单测上都放过 |
| C4 第 3497 行只有 nff 杀得掉 | 没打中 | 全仓钉这两种次序的只有 nff 那条单测；e158 产物不出这两种，换次序看不见 |
| C4 四行罩不住的改坏 | **打中**（C4-甲） | r2、r3 两个调用没有一行变异只靠它判红：单测删掉其中一个，四行照死；族那一维（C3-甲）、盘次序与奇偶（C3-乙）、读数层（C3-丙）变异表里一行都没有 |
| C4 三处注释 | 没打中 | nff 两处「第 189、201 行」、25、image.rs 四种不适用都与代码对得上 |

## C1　写死的五份集合

### 逐档对登记（没打中）

单测写死的字面：基础 18 条（`criteria_of_every_profile`）、J6Qn 与多盘 5 条、J7 五条、J8 四条、T 两条、拆行一条，五档各取几样（`r2_criteria` = 基础 + J6Qn/多盘 + J7）。逐档对登记：

- r1 = 基础 18 条：原登记 5.4 那张表 J0、A0、J1–J4、J5a、J5b、「J5a 与 J5b 分得开」、J6、AS 三条、A-互、A-结构、A-限、IA1、IA2，正好 18 行。
- r2 = 基础 + J6Qn + J7a 三格、J7b、J7c + 多盘四条：r2 登记 5.4「原登记 5.3 的 J0–J6、A0、AS 三条、A-互、A-结构、A-限、IA1、IA2 原样，另加」J6Qn、J7；多盘「任一份在即在」在 5.5，J1、J4 两条搬到多盘与 MC 上 J5 那条见第七节与修订 3（装置第 7223、7290 行的文档注释）。T 两条只在 `--table both`（r2 登记第 815 行「每个带 `--table both` 的跑法都带（不管选哪一族）」）。
- r3 = r2 + 拆行一条：r3 登记 5.1「`r3` 档 = `r2` 档全部原样」，5.7 新判据只有 Q3 四行一条（`i812_states` 那一行写明「不是假结局」）。
- r5 = r3 去掉 J7 五条、加 J8 四条：r5 登记 5.3「J8 无出处（只报，不是判据）」，J8a 三格是一条判据，所以「J8 五格」是四条判据、五个读数格。

五份集合与各档的 `judges_*`、`compares_the_two_record_tables` 逐条相同（模型「原样：改后单测 过」那一行就是拿 `is_judged_under` 现算的每档集合与字面比的结果）。没找到「一档判了、集合里漏写」的判据。

### C1-甲（打中，口径）：单测的族组合，`main` 起不出来

单测五个调用的族：r1 取「let legal_and_adversarial = [Family::LegalHistory, Family::Adversarial];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18279`），另四个取「let all_families = [Family::LegalHistory, Family::Adversarial, Family::MultiDisk];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18280`）。`main` 的族只来自命令行：不给 `--family` 是「let mut families = vec![Family::LegalHistory, Family::Adversarial];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17388`），`--family all` 是「"all" => {}」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17419`）照旧取 L+A，多盘只有单独一族「families = vec![Family::MultiDisk];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17416`）。所以 L+A+M 这个调用在任何一次正式跑里都不会出现，四个调用测的是一个起不出来的组合。

正式跑的单位是 (档, 族)：对照只跑选中的那一族（「/// 阳性对照那段循环：遍历判据表的每一行（只跑选中的那一族、这一档判的判据）。」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7750`），族过滤在「if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7758`）；replay.sh 里 15 条直接起装置的驱动每条只带一族（例：「--bounds segment1 --family adversarial --profile r3 | grep」，`scripts/experiments/replay.sh:763`）。kb 也是按族记的：「② 阳性对照（合法历史一族 35 个读数、对抗一族 10 个）」（`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:26`）。模型输出（原样）：

```text
== C1：单测五个调用（族组合 main 起不起得出 / 正式跑里有没有同档同族的一条）
  r1               r1       L+A    main 起得出：是；同档同族的正式跑：无；写死 18 条
  r2               r2       L+A+M  main 起得出：否；同档同族的正式跑：无；写死 28 条
  两份表都取的 r2        r2+both  L+A+M  main 起得出：否；同档同族的正式跑：无；写死 30 条
  r3               r3       L+A+M  main 起得出：否；同档同族的正式跑：无；写死 29 条
  r5               r5       L+A+M  main 起得出：否；同档同族的正式跑：无；写死 28 条
```

r1 那一个 L+A 是 `main` 起得出的（不带 `--family`），但正式跑里 r1 三条是 `--family legal` 两条、`--family adversarial` 一条，也没有 L+A 的。单测注释说的是「正式跑用的五档（r1、r2、两份表都取的 r2、r3、r5，`scripts/experiments/replay.sh` 起这个装置的那几条驱动）」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18271`）：档对上了，族没对上。这一格的后果在 C3-甲（族这一维的改坏单测全看不见）。

### C1-乙（打中，现状，原有）：r3 档写了 29 条，正式跑 r3 只有 8 条

按 (判据集, 两份表) 把正式跑并起来，对单测写死的那一份（模型原样）：

```text
  r1               写死 18；正式跑并起来 18；写了而正式跑这一档没有对照的 0 条：[]
  r2               写死 28；正式跑并起来  4；写了而正式跑这一档没有对照的 24 条：['J0', 'J1', 'J2', 'J3', 'J4', 'J5a', 'J5b', 'J5a_vs_J5b', 'J6', 'IA1', 'IA2', 'A0', 'AS_whole_table', 'AS_consecutive_gap_continues', 'AS_consecutive_gap_stops', 'A_mutual', 'A_structure', 'A_limit', 'J6Qn', 'J7a_instance_table', 'J7a_allocation_record_tree_root', 'J7a_filesystem_identifier', 'J7b', 'J7c']
  两份表都取的 r2        写死 30；正式跑并起来 30；写了而正式跑这一档没有对照的 0 条：[]
  r3               写死 29；正式跑并起来  8；写了而正式跑这一档没有对照的 21 条：['J0', 'J1', 'J2', 'J3', 'J4', 'J5a', 'J5b', 'J5a_vs_J5b', 'J6', 'IA1', 'IA2', 'J6Qn', 'J7a_instance_table', 'J7a_allocation_record_tree_root', 'J7a_filesystem_identifier', 'J7b', 'J7c', 'multi_disk_any_copy', 'J1_multi_disk', 'J4_multi_disk', 'J5_multi_disk']
  r5               写死 28；正式跑并起来 28；写了而正式跑这一档没有对照的 0 条：[]
  只按 --profile 并：r1 18 条
  只按 --profile 并：r2 30 条
  只按 --profile 并：r3 8 条
  只按 --profile 并：r5 28 条
```

这几个数由产物独立核过（核一：模型算的每条驱动的对照集与产物里 `name=control` 的判据集合 15 条全相同）。

- r2 直接建表那一档：正式跑只有多盘一条（「--bounds segment1 --family multi_disk --profile r2 | grep」，`scripts/experiments/replay.sh:738`），第二段 `driver_e165_segment2` 经杀伤界驱动起、产物里没有 `name=control` 行。但只按 `--profile` 并上 `--table both` 那几条，r2 档 30 条齐。所以 r2 这一行是单测把 `--table both` 拆成另一档造出来的，按登记的「档」（`--profile`）不缺。
- r3 档按哪一种并法都缺：r3 的正式跑只有对抗一族那一条，产物收尾行「E7RESULT name=verdict scope=run bounds=segment1 family=adversarial states=64513748 self_checks_passed=15 control_readings=80 controls_not_red=0 void_conditions=0 halt_conditions=0 profile=r3 marker_parity=even」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out:379`），80 个读数只出自 A 族 7 条与拆行一条。同一份产物在 A 域照判 J6Qn 与 J7：「E7RESULT name=verdict domain=A ring_slots=64 smallest_n=1 largest_n=4 criteria=r2 j6qn_premise_holds_violations=85400 j6qn_premise_holds_judged=45149528 j6qn_premise_fails_violations=0 j6qn_premise_fails_judged=0 j7_judged=45149528 j7a_instance_table_violations=0 j7a_allocation_record_tree_root_violations=1473356 j7a_filesystem_identifier_violations=0 j7b_violations=0 j7b_judged=1473356 j7c_violations=0 j7c_judged=43676172 first_returned_root_txg_is_the_largest=0」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out:209`）。判出这一行的是「    if run_profile.judges_j7() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8218`），不看族；登记 r2 5.4 也写 J6Qn 判「每个域（两族、多盘）」。所以 r3 档在正式跑里判了 J6Qn、J7 五格（A 域的判决行另带 J6 的前提两栏），r3 的哪一份产物里都没有它们的对照；单测的 r3 调用取 L+A+M，把这 21 条全跑出来，照过。

不是这一批引入的：改前单测的 r3 调用同样取全族。r3 登记 5.1「对抗一族（`--family adversarial`）每个 (域, n) 多出 Q3 的几行（5.6）；去掉这几行之后与 `r2` 档同一条命令的输出逐字节相同（TV2 核）。」（`research/prompts/e165-r3-prereg.md:424`）——r3 档只登记了对抗一族那一跑；J6Qn、J7 的对照数值在 r2 与 r3 上相同（拆行只动对抗一族的 I-8.12 计数），说「r2 那几跑的对照罩得住 r3」要另立一条跨档的读法，那正是 B1-甲 否掉的「几档并起来判」。

**四问。** ① 分不分辨臂：分辨「单测按档、全族」与「正式跑按 (档, 族)」两种口径；不分辨改前改后（两版单测都取全族）。② 被判的东西当时看不看得到：看得到，`positive_control_readings` 第一个参数就是族，单测给什么族全在单测手里。③ 满足的是判据字面的哪一句：正文 C1「r1 只取合法、对抗两族，r2–r5 取三族，与正式跑各档起的族对不对得上；有没有一档正式跑判了、集合里没写（或写了、正式跑不判）」——r2–r5 取三族与正式跑对不上；r3「写了、正式跑不跑对照」而正式跑照判。条款一侧：V2 只管「没红」，r3 正式跑缺对照不触发 V2（第一轮已指出同一处）。④ 跑前条款给的改法：正文第五节没给改法，不适用；我提的改法见文末，改法甲对 C1-乙 不起作用（它照今天的产物钉，r3 只钉 8 条），要不要给 r3 补 L 族那一跑是条款的事，交主 agent。

**推翻条件。** 正式跑里有一条 r3 档的非对抗族跑法（replay.sh 或杀伤界驱动的产物里出 `name=control criterion=J6Qn` 且带 `profile=r3`）；或 `main` 有一条路能把 L+A+M 一起交给 `positive_control_readings`。

## C3　新单测还能放过什么

### C3-甲（打中）：族这一维的改坏，单测全看不见

单测每档只比「这一档跑出对照的判据集合」（「let readings = positive_control_readings(families, run_profile, thread_count);」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18368`），而它给的族是 L+A+M（C1-甲）。一条判据归哪一族（`Criterion::family`）、族过滤那一行、T 两条「不管选哪一族都跑」（`is_judged_in_every_family`）三处只在单族的跑法里起作用，单测一处都碰不到。`Criterion::family` 在全仓只被第 7758 行这一处用；正式跑里判不判走的是域自己的族（「if !selection.families.contains(&domain.family()) {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17797`），所以这三处改坏只少对照、不少判定。

模型扫三类（原样，节选）：

```text
== 甲 `Criterion::family` 里一条判据挪到另一族：共 70 个；单测红 18；单测过而某条正式跑少了对照 48；单测过、正式跑只多不少 0；单测过、正式跑不变 4
   单测过而某一档（按 --profile 把它的正式跑并起来）整档少了对照的 9 个：
     A0 的族 A → L ⇒ [('r3', ['A0'])]
     AS_whole_table 的族 A → L ⇒ [('r3', ['AS_whole_table'])]
     AS_consecutive_gap_continues 的族 A → L ⇒ [('r3', ['AS_consecutive_gap_continues'])]
     AS_consecutive_gap_stops 的族 A → L ⇒ [('r3', ['AS_consecutive_gap_stops'])]
     A_mutual 的族 A → L ⇒ [('r3', ['A_mutual'])]
     A_structure 的族 A → L ⇒ [('r3', ['A_structure'])]
     A_limit 的族 A → L ⇒ [('r3', ['A_limit'])]
     J6Qn_i812_split 的族 A → L ⇒ [('r3', ['J6Qn_i812_split'])]
     J6Qn_i812_split 的族 A → M ⇒ [('r3', ['J6Qn_i812_split'])]
   单测过、正式跑不变的：['T_table 的族 L → A', 'T_table 的族 L → M', 'T_row 的族 L → A', 'T_row 的族 L → M']
  例：J0 的族 L → A ⇒ 单测过；少了对照的正式跑 6 条，第一条 driver_e165 少 ['J0']
== 乙 族过滤那一行与 `is_judged_in_every_family`：共 6 个；单测红 1；单测过而某条正式跑少了对照 3；单测过、正式跑只多不少 2；单测过、正式跑不变 0
   单测过而某一档（按 --profile 把它的正式跑并起来）整档少了对照的 1 个：
     族过滤写死成 &Family::LegalHistory ⇒ [('r3', ['A0', 'AS_consecutive_gap_continues', 'AS_consecutive_gap_stops', 'AS_whole_table', 'A_limit', 'A_mutual', 'A_structure', 'J6Qn_i812_split'])]
  例：`is_judged_in_every_family` 交回 false ⇒ 单测过；少了对照的正式跑 3 条，第一条 driver_e165_r2_segment4_table_both_adversarial 少 ['T_row', 'T_table']
```

单测红的 18 个全是「基础 18 条挪进多盘一族」（r1 调用不带 M）；别的挪法只要挪去的那一族在 L+A+M 里就照过。「只多不少」两种是「族过滤去掉族条件」与 `is_judged_in_every_family` 交回 true（每族都跑全部对照，正式跑多出对照行，单测同样放过）。

**具体坏法，一行，单测放过**（锚点在冻结副本里各命中 1 次，模型「点名的一行写法」原样）：

```text
  第 6193 行 `| Self::J6QnSplitByI812 => Family::Adversarial,` 换成 `=> Family::LegalHistory,`：锚点命中 1 次；单测 过；少了对照的驱动 5 条 ['driver_e165_adversarial', 'driver_e165_r2_segment4_table_both_adversarial', 'driver_e165_r3_segment1_adversarial', 'driver_e165_r5_segment3_adversarial', 'driver_e165_r5_segment3_odd_marker_adversarial']；整档少的 [('r3', ['A0', 'AS_consecutive_gap_continues', 'AS_consecutive_gap_stops', 'AS_whole_table', 'A_limit', 'A_mutual', 'A_structure', 'J6Qn_i812_split'])]
  第 6197 行 `| Self::MultiDiskAppliedSetAndLandedVersion => Family::MultiDisk,` 换成 `=> Family::LegalHistory,`：锚点命中 1 次；单测 过；少了对照的驱动 4 条 ['driver_e165_r2_segment3_multi_disk', 'driver_e165_r2_segment4_table_both_multi_disk', 'driver_e165_r2_segment4_reversed_disk_order_multi_disk', 'driver_e165_r5_segment3_multi_disk']；整档少的 []
  第 6203–6206 行 `is_judged_in_every_family` 的 matches! 换成 `false`：锚点命中 1 次；单测 过；少了对照的驱动 3 条 ['driver_e165_r2_segment4_table_both_adversarial', 'driver_e165_r2_segment4_table_both_multi_disk', 'driver_e165_r2_segment4_reversed_disk_order_multi_disk']；整档少的 []
  第 7758 行 `&criterion.family()` 换成 `&Family::LegalHistory`：锚点命中 1 次；单测 过；少了对照的驱动 9 条 ['driver_e165_adversarial', 'driver_e165_r2_segment3_multi_disk', 'driver_e165_r2_segment4_table_both_adversarial', 'driver_e165_r2_segment4_table_both_multi_disk', 'driver_e165_r2_segment4_reversed_disk_order_multi_disk', 'driver_e165_r3_segment1_adversarial', 'driver_e165_r5_segment3_adversarial', 'driver_e165_r5_segment3_multi_disk', 'driver_e165_r5_segment3_odd_marker_adversarial']；整档少的 [('r3', ['A0', 'AS_consecutive_gap_continues', 'AS_consecutive_gap_stops', 'AS_whole_table', 'A_limit', 'A_mutual', 'A_structure', 'J6Qn_i812_split'])]
```

拿第一行走一遍：「            | Self::J6QnSplitByI812 => Family::Adversarial,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6193`）是 A 族八条共用的那一臂，换成 `=> Family::LegalHistory,` 之后：

1. 单测五个调用的族都含 L（第 18279、18280 行），八条照跑，五份集合不变，罩全表不变，单测过。
2. `driver_e165_r3_segment1_adversarial`（`--family adversarial --profile r3`）里第 7758 行把八条全滤掉，`control_readings=0`；`main` 照判 A 族各域（第 8185 行那一行判决）与拆行（「if run_profile.splits_j6qn_by_i812() && domain.family() == Family::Adversarial {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8257`），V2 只看跑了的读数，不触发。r3 档没有别的正式跑，整档没有一条 A 族对照、一条拆行对照。
3. 另四条对抗一族的正式跑同样一条 A 族对照都没有；合法一族那几条反倒多跑八条（都红，不报）。

第三行对着登记字面：r2 登记第 815 行要 T 两条「每个带 `--table both` 的跑法都带（不管选哪一族）」，装置注释同句（「/// T-表与 T-行判的是每个域上两种建表法的逐态比，对照只拿 L 域跑一遍；每个带 `--table both` 的跑法都带这两条对照，不管选的是哪一族。」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6201`）。改坏之后 `--table both` 的对抗与多盘三条正式跑没有 T 对照，单测的「两份表都取的 r2」调用带 L，照过。

谁还抓得到：只有 replay.sh 逐字比正式产物（`name=control` 行少了），那是重型的实验复跑；`positive_control_readings` 在单测以外只有 `main` 第 17769 行一处调用。

**四问。** ① 分不分辨臂：分辨「单测给全族」与「照正式跑的 (档, 族) 钉」——改法甲（文末）在模型上把 48 + 3 个放过的全判红。不分辨改前改后单测（两版都取全族）。② 当时看不看得到：看得到，族就是调用的参数。③ 判据字面：正文 C3「五档之外正式跑还有没有别的档……该进来」的推广——缺的不是档，是族；条款一侧对着原登记 5.4「对照那段循环遍历判据表的每一行」（`research/prompts/e165-preregistration.md:508`）与 r2 登记第 815 行「不管选哪一族」。④ 改法：正文没给，不适用。

**推翻条件。** 有一条非重型的测试按单族调 `positive_control_readings` 或直接钉 `Criterion::family`（今天 `grep -n 'positive_control_readings('` 只命中定义、`main` 与这条单测）；或第 7758 行之外 `main` 另按 `Criterion::family` 决定判不判（那样挪族同时少判，变成另一种错）。

### C3-乙（打中，造的写法）：反盘次序与标记奇偶另一种

单测五个调用都取偶数奇偶（「let parity = MarkerParity::EvenVersionsWrittenByTheUnmountSequence;」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18278`）、默认盘次序；正式跑里有反盘次序两条（例：「--bounds segment1 --family multi_disk --only M,MC --profile r2 --table both --disk-order reversed | grep」，`scripts/experiments/replay.sh:753`）与 r5 odd 两条（例：「--bounds segment1 --family legal --profile r5 --marker-parity odd | grep」，`scripts/experiments/replay.sh:864`）。`is_judged_under` 拿的是带旋钮的原档（第 7761 行），对照自己才把盘次序与建表法退回默认（「.with_reported_disk_order(ReportedDiskOrder::FirstDeviceBeforeSecond);」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6303`）。所以一条臂多一个看盘次序或奇偶的条件，单测看不见：

```text
== 丙 `is_judged_under` 一条臂多一个看盘次序或奇偶的条件：共 12 个；单测红 0；单测过而某条正式跑少了对照 8；单测过、正式跑只多不少 0；单测过、正式跑不变 4
  例：臂 judges_j8 加「奇偶取 odd 时不判」 ⇒ 单测过；少了对照的正式跑 1 条，第一条 driver_e165_r5_segment3_odd_marker_legal 少 ['J8a', 'J8a_unmount_marker', 'J8b', 'J8c']
  例：臂 compares_the_two_record_tables 加「盘次序反过来时不判」 ⇒ 单测过；少了对照的正式跑 2 条，第一条 driver_e165_r2_segment4_reversed_disk_order_legal 少 ['T_row', 'T_table']
  例：臂 base 加「奇偶取 odd 时不判」 ⇒ 单测过；少了对照的正式跑 2 条，第一条 driver_e165_r5_segment3_odd_marker_legal 少 ['IA1', 'IA2', 'J0', 'J1', 'J2', 'J3', 'J4', 'J5a', 'J5a_vs_J5b', 'J5b', 'J6']
```

这一类是造的：今天的臂一个都不看盘次序与奇偶，要改坏成这样得新写一个条件，不像 C3-甲 那样换一个词就成。它说明的是单测只在 (r1|r2|r3|r5, even, normal) 上取样，判据那一侧（`is_judged_under`）对另外两个旋钮的任何写法都不受约束。整档并起来不缺（反盘次序与 odd 各有一条同档的默认跑法），只按单条正式跑算才少。

奇偶那一半在读数一级另有实的：见 C3-丙 第一条。

### C3-丙（打中，读数层）：集合只比到判据一级

单测比的是「跑出读数的判据标签集合」（「.map(|reading| reading.criterion.label())」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18371`），再逐个判读数红。一条判据有几个读数、读的是哪几个域，单测不管；所以一条判据少掉一部分读数、只要还剩一个红的，单测过。这一级没进模型，下面是读码判的、各给锚点（原文命中数用 python `str.count` 在冻结副本上数）：

1. **r5 odd 下 C8a、C8c 的 C 域读数**。登记 r5 5.4：「C8a、C8c 在 C 域 Nc ≤ 3 上只要求 > 0（C 域所选根随落盘变，钉值没算）。」（`research/prompts/e165-r5-prereg.md:472`）。装置在 odd 下另加这几个读数（「MarkerParity::OddVersionsWrittenByTheUnmountSequence => {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7539`，列哪几格由参数「cells_above_zero_on_the_monotone_domain_under_odd: &[Counter],」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7492` 给）。坏法：C8a 那一份第 7591–7594 行的两格换成 `&[]`（原文「Counter::J8aInstanceTableViolations,\n Counter::J8aAllocationRecordTreeRootViolations,\n ],\n run_profile,」命中 1 次），或 C8c 第 7641–7643 行第二个 `&[Counter::J8bViolations]` 换成 `&[]`（命中 1 次）。谁看得见：改后单测只跑 even，跑不到；C8a 自己那条单测只取前 8 个读数（「.take(8)」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:19492`），C8c 那条只取前 5 个（「.take(5)」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:19567`），C 域读数排在后面；只有 C8d 那条钉了读数个数（「assert_eq!(odd.len(), 3, "odd 另有 C 域那一条：{odd:?}");」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:19610`）。正式跑 `driver_e165_r5_segment3_odd_marker_legal` 少 3 个登记写明的读数（kb 记的「C8a 两格、C8c、C8d 各要求 > 0（读数 51、51、51、100）」里的前三个），V2 不触发。哪条单测都不红——推的，按代码推、没跑。
2. **基础 18 条的读数个数**。18 条里只有 J3 有一条钉读数个数的检查（第 8580 行 `check_the_in_flight_limit_control_on_both_rings`）；其余 17 条的对照在单测里只经过这一条全表单测（`grep -n 'Criterion::J2RootPlusDisk'` 只命中第 6488 行对照分支与第 18284 行字面）。例：J2 那条对照第 6503–6508 行的「J1 违例数 = 0」读数（原登记 5.4「J2 违例数 > 0，同时 J1 违例数 = 0（两种取法分得开）」）整段删掉，J2 还剩两个 > 0 的读数，单测过；正式跑少了登记要的「两种取法分得开」那一半。AS 三条的「A-互 那个数 > 0」读数同理：删掉之后剩下的是「违例数 = A-互 那个数」，A-互 为 0 时 0 = 0 也算红，登记 5.4「A-互 三个数在界内全为 0 时这三条对照做不出来」那一道就没了。
3. **「全红」本身被 0 = 0 满足**。今天的 J5a_vs_J5b 对照 18 个读数全是 `value=0 must_see=equals_0`（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family.out` 里 `grep 'criterion=J5a_vs_J5b'` 18 行，九个集合 × C、C6 两域）。原登记 5.4 这一行要的就是「五个集合上 J5a 违例数逐个等于 J5b 违例数」，kb 也照这样记了，不算错；但它说明单测的「读数全红」对这一条判据没有判别力：假结局换成什么都不做的理想恢复，两列同为 0，照样全红。这一条只报，不算打中。

**四问**（第 1、2 条）。① 分辨的是「按判据比集合」与「按读数比」两种钉法；不分辨改前改后。② 看得到：读数的 `measured_where` 就在 `ControlReading` 里。③ 正文 C3「读数「全红」这条断言在每档有没有被空集放过」——判据一级不空，读数一级可以少到只剩一个；第 1 条另对着 r5 登记 5.4 第 472 行。④ 正文没给改法。

**推翻条件。** 第 1 条：另有单测在 odd 下钉 C8a、C8c 的读数个数，或启动自检里有它（今天 `R2_SELF_CHECKS` 两项都不是）。第 2 条：另有单测钉基础 18 条的读数列表。

### C3-丁（打中，弱）：集合按标签比

两份集合都是 `BTreeSet<&str>`，元素取 `label()`（第 18371、18375 行），罩全表也按标签（「controlled_in_some_profile.contains(criterion.label()),」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18391`）。`Criterion` 本来派生了 `Ord`（宏里「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:2137`），装置里也没有一条检查标签互不相同。模型：

```text
== 丁 判据标签撞上另一条的（单测按标签比集合）：共 1190 个；单测过 1190
  丁 两步：J8a_unmount_marker 标签改成 J8a，同时它那一臂在 r5 档不判 ⇒ 单测 过；正式跑少了对照的驱动 2 条
```

单撞标签不少对照，只让产物里两条判据同名；配上一处少跑，单测就看不见那一处。两步才成，所以记弱。

### C3 没打中的几格

- **「只加不删」在五档之内**：判。某一档多跑一条判据，那一档的集合多一个元素，`assert_eq!` 红在「"{profile_name} 档跑出对照的判据与登记给这一档的那一份不同"」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18379`）。模型里的两例（原样）：「T 臂 → compares || (judges_j7 && !splits)（r2 档多跑 T 对照）：单测 红；单独判得红的调用 ['r2']」。跨族的「只加」（每族都跑全部对照）单测不判，见 C3-甲 的「只多不少」两种；它们只让正式跑多出读数行，不少对照。
- **空集**：判据一级不会空——五份写死的集合都非空，集合相等就逼着每条登记的判据至少有一个读数，「全红」那一圈不会空转。读数一级见 C3-丙。
- **`--table scan`**：replay.sh 里没有一条带它（15 条驱动的参数模型逐条抽过）；scan 档 T 两条不判、对照也不跑（`compares_the_two_record_tables` 只认 `Both`），别的对照退回直接建表。没有正式跑，就没有缺口。
- **r4**：参数解析里「"r3" | "r4" => Ok((」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17529`）解成同一个 `RunProfile::r3`，这张判据表上 r4 与 r3 逐字段相同；r4 的正式跑只有 `--family root_choice_faults`（`main` 第 17641 行起另走一套，不调 `positive_control_readings`）。r4 进单测不多罩任何东西。
- **反盘次序、odd 作为「档」**：整档并起来不缺（C3-乙 末段）；缺的只在单条跑法与读数一级。
- **挂钟**（推的，没跑）：改前改后都是 5 个调用，每个调用的对照都要跑对抗一族（族取 L+A+M），占大头的是对抗一族那几条。正式跑的计时：对抗一族 r3（带拆行、A6 扫到 n = 3）「E7TIMING name=elapsed scope=positive_controls elapsed_seconds=12.709 threads=8」（`research/results/e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family-timing.out:1`），r2「E7TIMING name=elapsed scope=positive_controls elapsed_seconds=5.609 threads=6」（`research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family-timing.out:1`），合法与多盘一族都在 0.1 秒以内。改前带拆行的调用三个（r3、r5、两份表都取的 r3），改后两个（r3、r5），所以改后不比改前长；量级按这几行推是每次几十秒，同机负载不同，没量。

## C4　四行变异与注释

### 四行锚点、筛法、红在哪（没打中）

冻结副本 `crates/mutations.tsv` 第 3383、3407、3496、3497 行，按第 2 列文件、第 3 列原文（`\n` 换回换行）用 python `str.count` 数：四行原文各命中 1 次，替换文都还没在文件里。前三行第 5 列 `-p singlefs-checker-tier --bin e165_replay_bounded_enumeration`（整个 bin 的单测），第 6 列点名这条单测；第 3497 行筛 `-- publish_writes_line`，点名的「    fn publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros() {」在 nff 装置第 1966 行。

前三行施加在 `is_judged_under` 上（模型原样）：

```text
  第 3383 行 基础臂 → !judges_j8()：改后单测 红在 r5；改前单测 过；单独就能判红的调用 ['r5']
  第 3407 行 T 臂 → compares && splits：改后单测 红在 两份表都取的 r2；改前单测 过；单独就能判红的调用 ['两份表都取的 r2']
  第 3496 行 基础臂 → judges_j6qn()：改后单测 红在 r1；改前单测 过；单独就能判红的调用 ['r1']
```

红在集合相等那一条（第 18377 行起的 `assert_eq!`），与主 agent 在副本上 `prove-red.sh` 报的「r5 档 / 两份表都取的 r2 档 / r1 档跑出对照的判据与登记给这一档的那一份不同」一致。改前单测（改动全文 `-` 那几行：r1 段只判红、r2 段只判没有拆行、r3、r5、r3+both 三档并起来罩全表）三行全放过：第 3383、3407 行被换掉之前锚在单测上，换成锚在被测的 `is_judged_under` 上之后，在改前单测下是活的，这正是要的（第 3383 行的那一种主 agent 已在改前单测上现跑 `1 passed`）。

### 第 3497 行只有 nff 那条杀得掉（没打中）

- 全仓钉 `tombstone_tree_node` 这类字段名的只有 `crates/singlefs-core/src/write_accounting.rs` 与 nff 装置两份（`grep -rln tombstone_tree_node crates` 原样两行）。
- 别处按 `IN_REPORT_ORDER` 走的有 `mount.rs`、`crash_injection.rs`、e158、两份 harness 测试：harness 那两份与 `mount.rs` 只用总数或逐种取值，与次序无关；`crash_injection.rs` 用 `any`；e158 逐种打行但只打写过的种类，它的产物里一行 `DeletedInodesTreeNode`、`TombstoneTreeNode` 都没有（`e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out` 的 `kind=` 计数），换次序看不见。
- core 自己那条单测只判「每种一笔」「和对得上」「名字互不相同」，换次序照绿（第一轮已核）。

所以这一行只有 nff 那条单测杀得掉，说法成立；推翻条件：另有一份产物或单测按次序逐字比含这两种的行。

### C4-甲（打中）：四行罩不住的改坏

1. **r2、r3 两个调用没有一行变异靠它判红。** 模型把单测五个调用逐个去掉，再施加三行（原样）：

```text
  去掉「r1」那个调用：三行里还活的 ['3496']
  去掉「r2」那个调用：三行里还活的 无
  去掉「两份表都取的 r2」那个调用：没改坏的代码上单测就红（「罩全表」那一条兜住，这一个调用去不掉）
  去掉「r3」那个调用：三行里还活的 无
  去掉「r5」那个调用：没改坏的代码上单测就红（「罩全表」那一条兜住，这一个调用去不掉）
```

单测以后被人删掉 r2 或 r3 那一个调用，变异门照绿。只有那一个调用判得红的改坏是有的，模型各举一例：「拆行臂 → splits && judges_j8()（r3 档少拆行对照）：单测 红；单独判得红的调用 ['r3']」，「T 臂 → compares || (judges_j7 && !splits)（r2 档多跑 T 对照）：单测 红；单独判得红的调用 ['r2']」。把它们写成变异行，r2、r3 两个调用才有人守。

2. **族、盘次序与奇偶、读数层三类，变异表里一行都没有。** C3-甲 的四个一行写法、C3-丙 的两处 `&[]`，今天的单测都放过（模型与读码），加进变异表就是活的变异；要它们死，得先改单测（文末改法甲、乙）。
3. 「罩全表」那一圈（第 18389 行起）也没有一行变异只靠它判红；它兜住的是「删掉一个调用」，不是被测代码的改坏。这一条只报。

### 三处注释（没打中）

- nff 第 1460、2444 行「该文件第 189、201 行」：harness 那份「    let warm_up_operation_count;」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:189`）与「        warm_up_operation_count = stream.operations().len();」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:201`），对。
- nff 第 2446 行「25、2、20、35、57」与断言「            vec![25, 2, 20, 35, 57],」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2476`）一致；23 → 25 的来由那句与第一轮核的清零步数一致。
- image.rs 第 212 行的四种不适用，与判法第 9017–9036 行四个 `not_applicable` 分支（`AnchorUndecided`、`ChosenRootTxgHasNoSuccessor`、`StoppedAtANamedUnitThatDoesNotVerify`、`NothingApplied`）逐个对上，walk.rs 自己的注释也是这四种（「/// 一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过（起点含介质损坏）都报不适用。」，`crates/singlefs-checker/src/walk.rs:9001`）。

## 改法与各修哪一格

都只在我的模型上量过或按代码推的，被攻过零轮。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲：单测的调用改成照正式跑的 (档, 族) 逐条来（今天 15 条：r1 L、r1 A、r2 M、r2+both L/A/M、r2+both+reversed L/M、r3 A、r5 L/A/M、r5 odd L/A），每条的期望集写死成字面（取今天那条驱动产物里 `name=control` 的判据集合） | C1-甲、C3-甲、C3-乙（判据一级） | 量过（模型）：今天的单测放过且正式跑少对照的，族挪动 48 个、族过滤 3 个、盘次序与奇偶 8 个，改法甲下全红；C4 三行照红。原样输出见下 |
| 甲的挂钟 | — | 推的：对抗一族的单元 5 个（r1、r2+both、r3、r5、r5 odd），与今天 5 个调用各跑一遍对抗一族的量相当；合法与多盘单元按正式跑计时各在 0.1 秒以内 |
| 乙：再钉读数一级，每个单元钉 (判据, `measured_where`) 的列表或至少每条判据的读数个数 | C3-丙 第 1、2 条 | 推的；没进模型 |
| 丙：集合元素改用 `Criterion`（它派生了 `Ord`），另加一条「标签互不相同」 | C3-丁 | 推的；模型里改法甲照按标签比，1 190 种照放过，两步那一例照过 |
| 丁：变异表补 C3-甲 的四个一行写法、C3-丙 的两处 `&[]`、C4-甲 那两例（只靠 r2、r3 调用判红的） | C4-甲 | 推的；锚点各命中 1 次是数过的；甲、乙落地之前它们是活的 |
| 戊：r3 档要么补一条非对抗族的正式跑，要么在登记里写明 r3 档的 J6Qn、J7 对照由 r2 档那几跑担 | C1-乙 | 条款的事，交主 agent；甲对这一格不起作用（照今天的产物钉，r3 只钉 8 条） |

```text
== 改法甲（被攻过零轮，只在这份模型上量过）：单测改成照正式跑的 (档, 族) 逐条钉，期望集写死成今天那条驱动产物里的判据集合
   钉 15 个 (档, 族)；原样 过
   甲 分族挪动：今天的单测放过且正式跑少了对照的 48 个，改法甲下仍放过 0 个 []
   乙 族过滤：今天的单测放过且正式跑少了对照的 3 个，改法甲下仍放过 0 个 []
   丙 盘次序 / 奇偶：今天的单测放过且正式跑少了对照的 8 个，改法甲下仍放过 0 个 []
   C4 三行：今天的单测放过且正式跑少了对照的 0 个，改法甲下仍放过 0 个 []
   丁 标签相撞：改法甲按标签比时放过 1190 / 1190（改用 BTreeSet<Criterion> 比，标签相撞不改集合；两步那一例改法甲下 过）
```

## 要主 agent 跑的命令

都是 checker 档，这条腿不跑。工作目录取一份仓副本（不是冻结副本、不是主工作区）；内存上限取 `scripts/experiments/replay.sh` 第 29 行 `REPLAY_MEMORY_CAP` 的默认值 8G，线程 15。

1. 坐实 C3-甲：副本里把 e165 装置第 6193 行「            | Self::J6QnSplitByI812 => Family::Adversarial,」换成 `            | Self::J6QnSplitByI812 => Family::LegalHistory,`，跑
   `SINGLEFS_HEAVY_TESTS=user-request bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 15 cargo test --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- every_criterion_has_a_positive_control_and_every_control_is_red`
   看 `test result:`：`1 passed` 即坐实单测放过。同一副本 `SINGLEFS_HEAVY_TESTS=user-request bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 15 cargo run --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds light --family adversarial --profile r3`，看 `name=verdict scope=run` 那一行：`control_readings=0`、`controls_not_red=0`、`void_conditions=0` 即坐实「r3 判了 A 族与拆行而没有对照、V2 不报」。
2. 坐实 C3-甲 第三行：另一份副本把第 6203–6206 行 `matches!(…)` 换成 `false`，同一条单测 `1 passed`；`cargo run … -- --bounds light --family adversarial --profile r2 --table both` 输出里 `grep -c 'name=control criterion=T_'` 为 0。
3. 坐实 C3-丙 第 1 条：另一份副本把第 7591–7592 行 C8a 那两格换成空（`&[]`），跑整个 bin 的单测 `cargo test --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`（同样经包装与 `SINGLEFS_HEAVY_TESTS=user-request`），看全过；`cargo run … -- --bounds light --family legal --profile r5 --marker-parity odd` 里 `grep -c 'name=control criterion=J8a '` 由 10 变 8（今天 odd 合法一族产物是 10）。
4. 复核模型：把模型目录拷到草稿目录，`bash tools/rerun.sh`，应报「✓ 模型输出与 expected-output.txt 逐字相同（93 行）」。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| C1 | 五份字面逐条对登记各档「判哪几条」 | 原登记 5.4、r2 5.4/5.5、r3 5.1/5.6/5.7、r5 5.1/5.3/5.4，35 条判据 × 5 档 |
| C1 | 一档判了、字面漏写 | 同上，以 `judges_*` 现算的集合与字面比（模型「原样：改后单测 过」） |
| C3 | 五档之内「只加」 | 读码 + 模型两例；集合相等必红 |
| C3 | 判据一级空集 | 五份字面都非空，读码 |
| C3 | `--table scan`、r4 | replay.sh 15 条驱动的参数（模型逐条抽）、参数解析第 17529 行 |
| C3 | 挂钟 | 正式跑计时产物两行，推的 |
| C4 | 锚点多命中、编不过、筛不到、红在别处 | 四行 `str.count`、第 5、6 列对测试名、模型判红的调用 |
| C4 | 第 3497 行别处杀得掉 | `grep -rln tombstone_tree_node crates`、`IN_REPORT_ORDER` 的六处用法逐个读、e158 产物的 `kind=` 计数 |
| C4 | 三处注释与代码 | harness 第 189、201 行，nff 第 2476 行，walk.rs 第 9001、9017–9036 行 |

## 这条腿自己的限度

- 一次抽样；「没打中」各格不支撑「没有」。
- 模型只管覆盖、不管数值：C3-甲 里「合法一族多跑的八条读数红不红」、C3-乙 那几种写法的读数，是按对照函数不看族、不看盘次序推的。
- C3-乙 是造的写法（今天的臂不看这两个旋钮）；C3-丙 读数层是读码判的，没进模型、没跑。
- 正式跑取 replay.sh 里直接起装置的 15 条；经杀伤界驱动起的几条（产物里没有 `name=control` 行）只按产物看，没核驱动里每一次起装置的参数。
- C1-乙 是现状、不是这一批引入的；它算不算违条款，取决于 V2 与原登记 5.4「这一段作废」按单条跑法还是按段读，这个读法我没定。
- 引文行号全取自冻结副本；开工时主工作区这几份与冻结副本逐字相同。

## 没做什么

- 没编 Rust、没跑任何 cargo 与 checker 档测试，也没跑 harness 档（这一格用不上）。
- 没判辩方那几格（C2）；没替主 agent 采纳；没碰 `crates/`。
- 草稿目录 `~/.local/share/singlefs/work/m3b37r2-opus/`：模型草稿与四次输出（`run1.txt`–`run4.txt`）、证红用的假根 `fakeroot/`（只有一份改过一行的 e165 装置源码，不是仓副本、不是编译目录）与它的输出 `selfred.txt`。
