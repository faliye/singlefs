# 第 37 批代码轮第三轮：云端攻方腿（Opus）报告（2026-10-09）

轮名 m3-formal-code-b37-r3；攻逐单元那一份改法本身，D1、D3、D4，重点 D1、D3。代码引文读冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r3/tree/`，写 `crates/…` 相对路径与那棵树里的行号（开工时 e165 装置与 `crates/mutations.tsv` 在冻结副本与主工作区 sha256 逐个相同：`a62469c7…`、`bcec1fc7…`）。`replay.sh`、杀伤界驱动与产物读主工作区。没读禁读清单里的两样。没编 Rust，没跑任何 cargo 与 checker 档测试。

## 复跑

模型只建「一次跑出哪几条对照」这一层，纯 Python：

```text
cd /home/fy5090/code/singlefs/research/prompts/m3-formal-code-b37-r3-opus-model
bash tools/rerun.sh        # 默认第三轮、第二轮两份冻结副本根，仓根取这个目录往上三层
adfd5f5d1e2cdca47968f27e7a38673a5f40fa881083ee1323dea2a7cab65bf8  d13_units.py
0f85f991be2984fa1f4e0149ba90b307a2e23d067c361893854a4977d4a2053d  tools/rerun.sh
e3f2073f808aa1abeefd60fbff8d3bea1cc8ae35e19560e611347c0c8087993a  expected-output.txt
```

模型从冻结副本读判据表（标签、`Criterion::family`、`is_judged_in_every_family`、`is_judged_under` 各臂）与 `positive_control_readings` 的两道 `continue`（读成表达式求值，所以变异行的替换文能整份重读）、改后单测的 14 个单元、`main` 起对照的那一行；从第二轮冻结副本读第二轮单测（只判 D4 的「第二轮单测下放过」）；从 `scripts/replay.sh` 第 699–880 行读直接起装置的驱动参数，按 `split_off_the_profile_options` 与 `parse_the_command_line` 的默认值解成 (族, 档)，再拿 `replay.sh` 登记行对的产物里 `name=control` 的判据集合核模型算的每条正式跑的对照集（核一：15 条全相同，不同退 4）。核一会红：e165 装置第 6193 行换成 `=> Family::LegalHistory,` 的假根上，核一 11 条不同、退 4（草稿 `~/.local/share/singlefs/work/m3b37r3-opus/selfred.txt`）。D4 那几行的红在哪个单元与主 agent 的 `prove-red` 日志（`~/.local/share/singlefs/work/m3-formal-37c-run/prove-red-logs/001–005.log`）逐个相同。限度：读数一律当「跑了就红」，只管判据一级。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| D1 14 个单元是不是恰好是正式跑的全部 | 没打中 | 第 699–880 行直接起装置的 18 条里去掉 root_choice 两族 3 条剩 15 条，第 703、706 行同一单元，14 个单元一一对上；杀伤界驱动编的是 `--root` 那棵冻结根，跑的不是今天的装置 |
| D1 14 份集合对点名产物 | 没打中 | 14 份写死集合与注释点名的产物、与 replay.sh 登记行对的产物都相同；r1 三条驱动的注释点 2026-10-04 产物、登记行对 2026-10-01 产物，两份逐字节相同（文字上不准，不影响钉值） |
| D1 2026-10-04 产物与今天的装置 | 没打中（单测这一侧） | 今天的非测试代码就是第五段 5-甲 的装置 `8006b251…`（本机各轮冻结副本逐个 diff，改动全在这条单测里）；集合一级由主 agent 那次 `1 passed` 直接坐实；6 个 r2 单元的产物出自第一段根、之后没在任何新编的二进制上复跑过，这是复跑登记的事，不是单测的 |
| D1-乙 同组单元写死的集合相同 | 打中（弱，两步） | 14 个单元里 11 个与另一个单元写死的集合逐字相同（分 5 组）（只差档、奇偶、建表法或盘次序）；单测里手写的档写成同组另一个，单测照过，那个旋钮从此没人核 |
| D3-甲 单测钉函数、不钉正式跑的命令 | **打中** | `main` 第 17769 行那一次调用不在单测里；一行改坏 `&selection.families` → `&[Family::LegalHistory]`，单测过，15 条正式跑里 9 条对照集变了、54 对 (驱动, 判据) 判了没对照；`run_profile` → `RunProfile::R1` 同样单测过、12 条变、51 对 |
| D3 判据表与对照循环这一侧 | 没打中 | 171 种改坏里改了某条正式跑对照集的 165 种，改后单测 0 种放过；缺口不在这一侧 |
| D3-乙 照产物钉把错的钉成对的 | 没打中（除已排除的 r3） | 14 个单元里 9 个「判了没对照」，8 个由同档同奇偶的合法一族单元罩住；罩不住的只有 r3 对抗，正是第二轮判决交用户的 C1-乙 |
| D3 `--only`、杀伤界模式、线程数 | 没打中 | 三样都不进对照集：`--only` 与界不进那次调用，线程数只在两道 `continue` 之后用，杀伤界跑的是冻结根 |
| D4 两行锚点、红在哪、第二轮下放过 | 没打中 | 第 3498、3499 行原文各命中 1、替换文 0；改后单测分别红在「r1 合法」「两份表都取的 r2 对抗」的集合断言；第二轮单测下都过 |
| D4 挂钟 | 没打中 | 按正式跑的对照计时，改后 23.053 秒、第二轮 22.917 秒（7 线程），比值 1.006；主 agent 的 `e165.log` 里这条单测 10.95 秒 |
| D4-甲 变异表罩不住 D3-甲 | 打中（随 D3-甲） | 表里没有一行锚在第 17769 行；照今天的单测，补这一行是活的 |

## D1　14 个单元与 14 份集合

### 单元是不是恰好是正式跑的全部（没打中）

`scripts/replay.sh` 第 699–880 行里 `cargo run … --bin e165_replay_bounded_enumeration` 直接起装置的 18 行，去掉 `--family root_choice`（第 766 行）、`root_choice_faults`（第 791、870 行）三条（它们走 `run_the_root_choice_family` / `run_the_root_choice_faults_family`，不调 `positive_control_readings`），剩 15 条。模型按命令行解出 (族, 档) 再与单测的单元比（原样）：

```text
== D1：14 个单元对 replay.sh 直接起装置的驱动（root_choice 两族除外）
  驱动 15 条，单元 14 个
  没有单元的驱动：无
  今天的源码上改后单测：过
```

每个单元注释写的驱动行与模型找到的驱动行逐个「对」（第 703 与 706 两行解出同一个 (合法, r1)，界不进对照那次调用，见 D3）。

经杀伤界驱动起的那几条不是今天装置的正式跑：r2 模式调装置是「DEVICE_PROFILE_ARGUMENTS = ["--profile", "r2"]」（`scripts/e165-replay-mutation-kill-bounds.sh:278`）加「            return ["--bounds", "segment2"]」（`scripts/e165-replay-mutation-kill-bounds.sh:557`），不带 `--family`，所以是合法、对抗两族一起（「    let mut families = vec![Family::LegalHistory, Family::Adversarial];」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17388`）。这个 (r2 直接建表, 两族) 组合单测里没有。但它编的是 `--root` 那棵树：「    copy_the_tree(setup, tree)」（`scripts/e165-replay-mutation-kill-bounds.sh:1020`）从「        source = os.path.join(setup.root, entry)」（`scripts/e165-replay-mutation-kill-bounds.sh:997`）拷，两个 r2 模式驱动的根是「  local frozen_root="${E165_R2_FROZEN_ROOT:-/tmp/claude-1000/e165-r2-segment4/root}"」（`scripts/replay.sh:722`）与「  local root="${E165_R5_SEGMENT1_ROOT:-/tmp/claude-1000/e165-r5-segment1/root}"」（`scripts/replay.sh:820`）。今天的装置改坏到不了那几跑，单测管不着它们也不缺。r3、r4 两个模式跑的是 root_choice 两族。

我先按这一单元打过一行坏法（J7 那条臂 `run_profile.judges_j7()` 换成 `run_profile.compares_the_two_record_tables()`：14 个单元里带 J7 的只有两份表都取的两个合法单元，都取两份表，单测照过；(r2 直接建表, 两族) 那一跑少五条 J7 对照），查到它编冻结根之后撤回：今天的正式跑上这一行是等价的。

### 14 份集合对点名产物（没打中；r1 三条注释点的产物与登记行不是同一份文件）

核一 15 条全相同（`expected-output.txt` 第 2–16 行）。逐单元（原样，节选）：

```text
  r1 合法：驱动行 [703, 706]（注释写 [703, 706]，对）；写死 11 条 = 注释点的产物 是 = 登记行对的产物 是；注释点的产物不是登记行对的那一份（登记 e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out、e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out；与 2026-10-04 那一份（轻档对轻档）逐字节相同）
  r1 对抗：驱动行 [710]（注释写 [710]，对）；写死 7 条 = 注释点的产物 是 = 登记行对的产物 是；注释点的产物不是登记行对的那一份（登记 e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out；与 2026-10-04 那一份（轻档对轻档）逐字节相同）
  r3 对抗：驱动行 [763]（注释写 [763]，对）；写死 8 条 = 注释点的产物 是 = 登记行对的产物 是；注释点的产物就是登记行对的那一份
```

另 11 个单元同第三行。r1 那三条：登记行是「E165|@driver_e165||e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out|exact」（`scripts/replay.sh:331`）与「E165|@driver_e165_adversarial||e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out|exact」（`scripts/replay.sh:333`），单测注释点的是 `…-2026-10-04-r5-segment1-r1-*.out`。两份逐字节相同（第 5 次第一段 PV3 就是拿它们比的），所以钉值不受影响；只是单测文档注释那句「每个单元跑出对照的判据恰好是那条驱动产物里 `name=control` 的那一份」对 r1 三条指的不是 replay.sh 拿来比的那一份。只报，不算打中。

**推翻条件。** replay.sh 第 699–880 行之外另有一处用今天的源码起 `e165_replay_bounded_enumeration` 的普通跑法（不是 `--pinned-cases`、`--replay-state`、`--list-states`）；或杀伤界驱动不经 `--root` 编装置。

### 2026-10-04 产物与今天的装置（单测这一侧没打中）

- 今天的非测试代码是哪一份：本机各轮冻结副本里这份装置的 sha256 按时间排是 `8006b251…`（第 36 批冻结副本，与 kb 记的 5-甲 装置 `8006b251c1011d5f…` 同一份，`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md` 第 917 行）→ `9b915056…` → `827a4fff…`（第一轮冻结副本）→ `c85942a3…`（第二轮）→ `a62469c7…`（第三轮）。逐份 `diff`，每一处改动都在这条单测的函数里（第 18268 行起）。今天的非测试代码就是 5-甲 那一份。HEAD 里这份装置是 `d6937857…`，即第 5 次第一段的根（同一份 kb 第 1033 行：第三段经 compile-then-swap 从 `d6937857…` 改成 `f72fc1a4…`；`git show HEAD:` 这份文件的 sha256 前 12 位 `d6937857c661`）。
- 14 个单元的产物出自两棵根：9 个（r1 两个、r2 六个、r3 对抗）是第一段根 `d6937857…` 编的；5 个 r5 单元是第三段 `f72fc1a4…` 编的。
- 之后在新编的二进制上复跑过的：第三段二进制上「name=pv3 product=r2-legal-family identical=True rerun_lines=1466 segment1_lines=1466 sha256=5f876ca2ba7173d6」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-profile-check.out:25`）这一组（第 22–29 行：r1 合法、r1 轻档、r1 对抗、r2 直接建表合法与对抗、r3 对抗、r3 择根、r4），5-甲 二进制上「name=pv13 product=row14_r3_adversarial identical_whole=True identical_without_config=True rerun_lines=380 earlier_lines=380」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-profile-check.out:3`）这一组（第 2–6 行：r1 合法、r3 对抗、r3 择根、r4、r5 合法）。14 个单元里 r2 多盘、两份表都取的 r2 合法/对抗/多盘、反盘次序的 r2 合法/多盘这 6 个，与 r5 对抗、r5 多盘、两个奇数版本 r5 这 4 个，产物之后一次都没在新二进制上复跑过。
- 对单测：它把 14 份集合写死成产物里的，主 agent 在今天的源码上跑出 `1 passed`，就是「今天跑出的集合 = 产物的集合」这件事本身，集合一级不会被「照它钉」藏住；哪天今天的装置跑出的集合变了，单测红在那个单元，不是照过。所以这一格单测一侧没打中。
- 余项（不是单测的事）：上面那 10 份产物在判定行一级与今天的装置是否逐字节相同，没有记录；`replay.sh` 第 335–340、350–353 行对它们写的是 `exact`。复跑命令见「要主 agent 跑的命令」第 3 条。

### D1-乙（打中，弱，两步）：同组单元写死的集合相同，手写的旋钮没人核

模型（原样）：

```text
  写死的集合相同的单元（单元里手写的档、奇偶、建表法、盘次序写成同组另一个的，集合不变、单测照过）：
    r2 多盘 ≡ r5 多盘
    两份表都取的 r2 合法 ≡ 两份表都取、盘次序反报的 r2 合法
    两份表都取的 r2 多盘 ≡ 两份表都取、盘次序反报的 r2 多盘
    r3 对抗 ≡ r5 对抗 ≡ 奇数版本的 r5 对抗
    r5 合法 ≡ 奇数版本的 r5 合法
```

单元的档是单测里手写的（例：「            let readings = positive_control_readings(&[family], run_profile, thread_count);」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18547`，`run_profile` 取自单元元组），不经命令行解析。「奇数版本的 r5 合法」那个单元把 `RunProfile::r5(odd)` 写成 `RunProfile::r5(even)`，集合不变、单测照过，从此 odd 那一跑在单测里没有单元（第二轮 C3-乙 的那类臂上看奇偶的坏法又回到看不见）。今天 14 个单元的手写档与驱动解出来的逐个相同（模型里「驱动行…对」就是按 (族, 档) 对上的），所以这是两步：先写错单测、再有一处看旋钮的坏法。记弱。

## D3　逐单元的单测还能放过什么

### D3-甲（打中）：单测钉的是 `positive_control_readings`，不是正式跑那条命令；`main` 第 17769 行不在单测里

单测说它核的是正式跑：「    /// 一种建表法、一种盘次序、一种标记奇偶；每个单元跑出对照的判据恰好是那条驱动产物里 `name=control` 的那一份（写死成字面，」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18272`）。它实际调的是函数，族与档是单元元组里手写的：「            let readings = positive_control_readings(&[family], run_profile, thread_count);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18547`）。正式跑打出 `name=control` 的那一次调用在 `main` 里：「        positive_control_readings(&selection.families, run_profile, thread_count);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17769`）。`grep -n 'positive_control_readings(' ` 在这份文件里只命中定义（第 7751 行）、这两处；`crates/` 下别的文件 0 处；这个 bin 没有集成测试。所以从「驱动的命令行」到「这一次调用的族与档」这一段（`parse_the_command_line`、`split_off_the_profile_options` 与第 17769 行本身），单测一处都不经过。档那一半的解析有启动自检 `check_the_profile_options`（「    ("profile_options", check_the_profile_options),」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:9274`）罩着；族那一半与第 17769 行没有。

**具体坏法，一行，单测放过。** 第 17769 行 `&selection.families` 换成 `&[Family::LegalHistory]`（原文命中 1 次；类型 `&[Family; 1]` 照样转成 `&[Family]`，编得过）：

1. 单测：14 个单元直接调函数，集合不变，罩全表不变，过。
2. 正式跑：对抗一族与多盘一族的 9 条（第 710、738、744、747、753、763、858、861、867 行）全换成合法一族的对照；对抗一族那几跑照判 A 域（`emit_the_verdict_of_a_domain` 里 `Family::Adversarial` 那一支，判 A0、AS 三条、A-互、A-结构、A-限），一条 A 族对照都没有；多盘那几跑照判 M、M6、MC，多盘四条对照全没有。V2 只看跑了的读数红不红，跑出来的合法一族对照都红，进程退 0（「    if !findings.void_conditions.is_empty() || !findings.halt_conditions.is_empty() {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17914`）。
3. 谁还抓得到：只有 replay.sh 逐字比正式产物（重型的实验复跑）；变异表里没有一行锚在第 17769 行（`crates/mutations.tsv` 里 `selection.families` 命中 0 次）。

模型（原样，节选；「判定行照有」数的是那份产物里带这条判据判定字段的 `name=verdict` 行，与模型的判据表不共用代码）：

```text
  族写死成合法一族（`&selection.families` → `&[Family::LegalHistory]`）：单测 过；对照集变了的正式跑 9 / 15 条；少了的 (驱动, 判据) 里这一跑判定行照有的 54 对
    第 710 行 driver_e165_adversarial：少 7 条 [A0, AS_consecutive_gap_continues, AS_consecutive_gap_stops, AS_whole_table, A_limit, A_mutual, A_structure]；多 [IA1, IA2, J0, J1, J2, J3, J4, J5a, J5a_vs_J5b, J5b, J6]；少的每条在这一跑判定行里的行数 3–3
  族换成空（`&selection.families` → `&[]`）：单测 过；对照集变了的正式跑 15 / 15 条；少了的 (驱动, 判据) 里这一跑判定行照有的 139 对
  档写死成 r1（`run_profile, thread_count` → `RunProfile::R1, thread_count`）：单测 过；对照集变了的正式跑 12 / 15 条；少了的 (驱动, 判据) 里这一跑判定行照有的 51 对
    第 741 行 driver_e165_r2_segment4_table_both_legal：少 8 条 [J6Qn, J7a_allocation_record_tree_root, J7a_filesystem_identifier, J7a_instance_table, J7b, J7c, T_row, T_table]；多 无；少的每条在这一跑判定行里的行数 11–11
```

「档写死成 r1」那一行是第二个一词坏法：正式跑 r2、r3、r5 的 12 条照判 J6Qn、J7、J8、拆行、T 与多盘四条（判定行由 `run_profile.judges_*()` 打，不经第 17769 行），对照一条都没有。

拿登记对一下这一格不是我造的口径：原登记的变异表把「对照那段循环」交给这条单测（「| 阳性对照循环只跑判据表第一行 | 单测「判据表每一行都有一条对照且都红」红 |」，`research/prompts/e165-preregistration.md:857`）。循环里的坏法今天的单测都抓（下一节 165 / 165）；循环外面、`main` 把哪一族哪一档交给循环的那一行，单测抓不到，而单测文档注释说的是「那条驱动产物里」。

**四问。** ① 分不分辨臂：分辨「钉函数的输出」与「钉驱动那条命令打出的对照集」两种钉法；不分辨第二轮与第三轮单测（两版都直接调函数）。② 被判的东西当时看不看得到：看得到，命令行就在 replay.sh 里，`split_off_the_profile_options` 与 `parse_the_command_line` 都是纯函数，单测可以拿驱动的参数原样解出单元再调。③ 判据字面：正文 D3「单元之间、单元内部还有没有一维（`--only` 选的域、杀伤界驱动起的模式、线程数）让一条判据在正式跑少了对照而单测不红」（`research/prompts/_m3-formal-code-b37-r3-body.md:30`）——这一维是「命令行 → 那一次调用的族与档」，括号里点的三样之外的第四样；单测自己那句（第 18272 行）是它字面上许下的。④ 跑前条款给的改法：正文第五节没给改法，不适用；我提的改法见文末，只在我的模型上推过。

**推翻条件。** 有一条非重型的测试经 `main` 或经与 `main` 共用的函数，从命令行字面起单元（`grep -n 'positive_control_readings('` 多出一处这样的调用）；或启动自检里有一项比对「这一跑的对照集」与按 (族, 档) 现算的集合。

### D3 判据表与对照循环这一侧（没打中）

为了确认缺口只在循环外面，模型把表这一侧能一词改坏的几类都扫了（判据挪族 70 种、`is_judged_in_every_family` 每条加减 35 种、`is_judged_under` 每组臂换成另一个谓词或取反 66 种）：

```text
  改坏 171 种里让某条正式跑的对照集变了的 165 种，其中改后单测放过的 0 种
```

第二轮打中的族这一维（48 + 3）与盘次序、奇偶那一类（臂上看旋钮）在逐单元的写法下都红，因为 14 个单元与 15 条直接起装置的驱动按 (族, 档) 一一对上。变异表里锚在这条单测上的 7 行（第 1462、1625、3383、3407、3496、3498、3499 行）也都在循环与表这一侧。

### D3-乙　照产物钉，会不会把错的钉成对的（没打中，r3 那一条已排除）

拿每个单元那份产物的判定行（不是模型推的）数「判了、这一跑没有对照」的判据（原样，节选）：

```text
  r1 对抗：判了没对照 1 条 [J6]；同档同奇偶、一个单元就罩住它们的：r1 合法
  r3 对抗：判了没对照 7 条 [J6, J6Qn, J7a_allocation_record_tree_root, J7a_filesystem_identifier, J7a_instance_table, J7b, J7c]；同档同奇偶、一个单元就罩住它们的：无
  r5 多盘：判了没对照 16 条 [IA1, IA2, J0, J1, J2, J3, J4, J5a, J5a_vs_J5b, J5b, J6, J6Qn, J8a, J8a_unmount_marker, J8b, J8c]；同档同奇偶、一个单元就罩住它们的：r5 合法
```

9 个单元有「判了没对照」，8 个由同档同奇偶的合法一族单元整份罩住（对照只跑选中那一族是登记的写法：「/// 阳性对照那段循环：遍历判据表的每一行（只跑选中的那一族、这一档判的判据）。」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7750`）。罩不住的只有 r3 对抗，就是第二轮判决交用户的那一条（r3 同档无对照，派发提示写明不再攻）。r2 多盘（直接建表）同档的合法单元都取两份表，对照自己把建表法退回直接建表（「/// 一条假结局跑在哪个档上：T-表、T-行两条对照拿 `--table both`，别的对照一律拿直接建表、盘次序照常（对照证的是判据分得出差别，」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6298`），第二轮攻方已经按 `--profile` 并过，不新。产物里没找到一份「登记要的对照没有、或登记不要的对照多了」的集合。

### D3　`--only`、杀伤界驱动起的模式、线程数（没打中）

- `--only`、`--bounds`、`--max-n`：第 17769 行那次调用只带族、档、线程数三个参数，函数签名也只有这三个（「fn positive_control_readings(」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7751`）。第 750 行 `--only R`、第 753 行 `--only M,MC` 只少判别的域，对照照整族跑；第 706 行轻档与第 703 行的对照集相同（两份产物都是 11 条）。
- 线程数：函数里线程数只出现在两道 `continue` 之后那一行（「        readings.extend(control_readings_of(*criterion, run_profile, thread_count));」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7764`）；单测取本机 `available_parallelism`（第 18278 行），正式跑取 `E165_THREADS`（8 或 10），判据一级的集合与线程数无关。模型那一行原样：「== D3 线程数：positive_control_readings 里 thread_count 只出现在 1 行，都在两道 continue 之后」。
- 杀伤界驱动起的模式：编的是 `--root` 那棵冻结根（D1 第一节），今天的装置改坏到不了。

## D4　变异两行与挂钟

### 两行锚点、红在哪、第二轮单测下放过（没打中）

模型把 `crates/mutations.tsv` 每行第 3、4 列（`\n` 换回换行）施加到冻结副本、整份重读判据表与对照循环再判（原样，节选）：

```text
  第 3498 行 E165 对抗一族八条挪进合法一族（对抗一族的正式跑没有…：原文命中 1、替换文命中 0；改后单测 红在 r1 合法；第二轮单测 过；对照集变了的正式跑 [703, 706, 710, 741, 744, 750, 763, 855, 858, 864, 867]
  第 3499 行 E165 T_table、T_row 只在合法一族跑对照…：原文命中 1、替换文命中 0；改后单测 红在 两份表都取的 r2 对抗；第二轮单测 过；对照集变了的正式跑 [744, 747, 753]
  第二轮单测在今天的源码上：过
```

两行第 5 列都是 `-p singlefs-checker-tier --bin e165_replay_bounded_enumeration`、第 6 列点这条单测；「红在哪个单元」与主 agent 的 `prove-red-logs/004.log`、`005.log` 里「r1 合法 那个单元…」「两份表都取的 r2 对抗 那个单元…」相同，同一张表上第 3383、3407、3496 行也与 `001–003.log` 相同（模型「第 3383 行…改后单测 红在 r5 合法」等）。`Criterion::family` 与 `is_judged_in_every_family` 在这份文件里只在第 7758 行用，别的单测碰不到这两行，所以「第二轮单测下放过」不靠别的单测补。

### 挂钟（没打中）

按每个单元那条正式跑的对照计时（`-timing.out` 的 `scope=positive_controls`，7 线程）：改后 14 个单元合计 23.053 秒，其中对抗一族 5 个占 22.780 秒；第二轮单测 5 个调用按几族计时相加 22.917 秒（r3 调用的合法、多盘两族没有 r3 产物，借 r2 直接建表的计时，各在 0.12 秒以内）；比值 1.006。主 agent 在本机跑这条单测是 `finished in 10.95s`（`~/.local/share/singlefs/work/m3-formal-37c-run/e165.log`）；变异门跑整个 bin 的单测，基线 190.35 秒、五行 117.97–177.75 秒（同目录 `prove-red-logs/`），在门禁 `checker-tier-crates-mutation-replay` 默认每行 1 800 秒的限时（`.claude/gate.d/checker-tier-crates-mutation-replay.sh` 第 48 行）之内。

### D4-甲（打中，随 D3-甲）：变异表罩不住第 17769 行

表里锚在这条单测上的 7 行都在判据表与对照循环那一侧；第 17769 行一行都没有。照今天的单测补一行「`&selection.families` → `&[Family::LegalHistory]`」是活的（D3-甲 第 1 条），所以要先改单测（改法甲）再补这一行。

## 改法与各修哪一格

都是我提的，只在我的模型上推过或量过，被攻过零轮。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲：单元改用驱动的命令行字面（从 replay.sh 抄，例 `["--bounds", "segment1", "--family", "adversarial", "--profile", "r3"]`）经 `split_off_the_profile_options`、`parse_the_command_line` 解出 (族, 档)；第 17768–17769 行抽成一个 `main` 与单测共用的函数（入参是解出来的 `RunSelection` 与档），单测调它 | D3-甲、D1-乙 | 推的：没实现、没跑。模型只量了今天的单测放过那五种（「单测 过」）；按代码推，甲下第 17769 行的族、档两种改坏都会在第一个对抗或多盘单元红 |
| 乙：甲落地之后补一行变异 `        positive_control_readings(&selection.families, run_profile, thread_count);` → `        positive_control_readings(&[Family::LegalHistory], run_profile, thread_count);`（原文命中 1 次是数过的） | D4-甲 | 推的；甲落地之前它是活的 |
| 丙：r1 三个单元的注释改点 replay.sh 第 331–333 行登记行对的 2026-10-01 产物（或写明两份逐字节相同） | D1 文字 | 推的；只动注释 |
| 丁：单元列表与 replay.sh 绑起来（单测里 `include_str!` replay.sh，抽出直接起装置的驱动参数，与单元列表逐条比） | D1 单元列表以后跟不跟得上 replay.sh | 推的；把 crate 的单测绑到 research/ 下的脚本上，要不要这样绑由主 agent 定 |

## 要主 agent 跑的命令

都是 checker 档或重型复跑，这条腿不跑。工作目录取一份仓副本（不是冻结副本、不是主工作区）；内存上限取 `scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值 8G，线程 15。

1. 坐实 D3-甲（族）：副本里把 e165 装置第 17769 行「        positive_control_readings(&selection.families, run_profile, thread_count);」换成 `        positive_control_readings(&[Family::LegalHistory], run_profile, thread_count);`，跑
   `SINGLEFS_HEAVY_TESTS=user-request bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 15 cargo test --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- every_criterion_has_a_positive_control_and_every_control_is_red`
   看 `test result:`：`1 passed` 即坐实单测放过。同一副本 `SINGLEFS_HEAVY_TESTS=user-request E165_THREADS=15 bash scripts/run-with-memory-cap.sh 8G cargo run --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --max-n 1 --family adversarial --profile r3`，看：`grep -c 'name=control criterion=A'` 为 0；有 `name=verdict domain=A ` 开头、带 `a0_violations=` 的行；`name=verdict scope=run` 那一行 `controls_not_red=0 void_conditions=0`；退出码 0。
2. 坐实 D3-甲（档）：另一份副本把同一行的 `run_profile, thread_count` 换成 `RunProfile::R1, thread_count`，同一条单测 `1 passed`；`cargo run … -- --bounds segment1 --max-n 1 --family legal --profile r2 --table both` 输出里 `grep -c 'name=control criterion=J6Qn '` 为 0、`grep -c 'criteria=r2 j6qn_premise_holds_violations='` 大于 0。
3. D1 余项（不挡这一批）：在主工作区按 replay.sh 第 738、741、744、747、750、753、858、861、864、867 行原样起那 10 条，各自 `| grep '^E7RESULT '` 后与第 335–340、350–353 行登记的产物 `cmp`（产物里没有 `E7RESULT ` 之外的行）。不同就是「2026-10-04 产物之后装置变了」在判定行一级的观测，集合一级照单测 `1 passed` 不变。
4. 复核模型：把模型目录拷到草稿目录，`bash tools/rerun.sh`，应报「✓ 模型输出与 expected-output.txt 逐字相同（113 行）」。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| D1 | 正式跑里有没有单元之外的 (族, 档) | replay.sh 第 699–880 行直接起装置的 18 行逐条解参数；杀伤界三个模式的调法（第 278、557 行与 r3、r4 两套）与编法（第 997、1020 行）；门禁与 scripts 里别处起这个装置的（只有 `--pinned-cases` 一条路，在对照之前就返回） |
| D1 | 14 份集合对点名产物、对登记行的产物 | 14 个单元 × 两份产物；r1 三条两份产物逐字节比 |
| D1 | 产物之后装置变没变 | 本机 5 个版本的冻结副本逐份 diff；两份 profile-check 产物列的复跑对象 |
| D1 | (r2 直接建表, 两族) 那一跑少 J7 对照（J7 臂换成 `compares_the_two_record_tables()`） | 先当打中，查到杀伤界编冻结根后撤回 |
| D3 | 判据表、`is_judged_in_every_family`、`is_judged_under` 各臂的一词改坏 | 171 种，改了正式跑对照集的 165 种全红 |
| D3 | 照产物钉、产物本来就缺 | 14 个单元 × 35 条判据，按产物判定行字段数 |
| D3 | `--only`、界、线程数进不进对照集 | 第 17769 行的三个参数、函数签名与第 7764 行 |
| D3 | 命令行解析的档那一半 | 启动自检 `check_the_profile_options` 罩 r1 三条、r2 奇偶、r3、r5 奇偶；`--table`、`--disk-order` 由 `RECORD_TABLE_SELF_CHECKS` 里那一项罩（第 9485–9540 行一带）。族那一半解析没人罩，但族改错时判与对照一起换族，不出「判了没对照」，并进 D3-甲 改法甲一起修 |
| D4 | 锚点多命中、替换文已在、红在别处、第二轮单测下也红 | 第 3498、3499 行与同表锚这条单测的另 5 行 |
| D4 | 挂钟 | 14 条正式跑的对照计时、主 agent 的两份日志、门禁默认限时 |

## 这条腿自己的限度

- 一次抽样；「没打中」各格不支撑「没有」。
- 模型只管判据一级；读数一级（第二轮判决记账）不在攻击面里。
- D3-甲 的「正式跑少了对照」是按 `main` 第 17768–17790 行读出来的、再拿产物的判定行核「这一跑判了」，没跑装置；单测放过是读码判的（单测不经 `main`），没跑单测。
- D1 里「今天的非测试代码就是 5-甲 那一份」靠本机各轮冻结副本逐份 diff；那几份副本在 `~/.local/share/singlefs/work/` 下，不入库，主 agent 要核可按「要主 agent 跑的命令」第 3 条直接复跑。
- 改法甲以后只被我推过，被攻过零轮；按正文第五节，第三轮新冒出来的零轮形态进交用户表。

## 没做什么

- 没编 Rust、没跑任何 cargo 与 checker 档测试，也没跑 harness 档。
- 没判辩方那几格（D2）；没替主 agent 采纳；没碰 `crates/` 与 replay.sh。
- 草稿目录 `~/.local/share/singlefs/work/m3b37r3-opus/`：模型草稿 `model/`、两次输出 `run1.txt`、`run2.txt`、replay.sh 与装置相对 HEAD 的 diff、各轮装置副本清单 `e165-copies.txt`、证红那一次的输出 `selfred.txt`（证红用的假根只有一份改过一行的 e165 装置源码与一份 `crates/mutations.tsv`，跑完已删；重做照 `tools/rerun.sh` 文件头那一句）。
