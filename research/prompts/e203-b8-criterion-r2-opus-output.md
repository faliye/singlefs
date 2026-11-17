# e203-b8-criterion-r2 攻方腿报告（opus，2026-11-13）

<!-- doc-lint:not-numbers B1 B4 B8 C1 C2 C3 D8 D28 E203 E206 E207 E209 F2 F10 K100 K1000 K1000L FTB Q0 Q2 Q5 Q11 W1 W2 W4 W5 W7 -->

立场：攻方，假设执行员与第一轮修补的倾向是错的。背景 `research/prompts/_e203-b8-criterion-r2-background.md`，判的是 `crates/` 今天的实现（`transaction.rs` d6b2083d2d950d7f、`allocator.rs` eb822f0920329af4、`mount.rs` ae24940185ad0102，前 16 位，开工时现算，与背景一致）。禁读的那一份没读。

## 复跑

模型目录 `research/prompts/e203-b8-criterion-r2-opus-model/`：只读 `research/results/` 下八份留存产物（文件名与前 16 位 sha256 在输出头几行打出），不起项目代码、不建池、不枚举崩溃状态，所以第 3b 步的稀疏盘、崩溃状态数与 10⁶ 上限在这一腿不适用。每个读数旁边有 `name=check` 行钉住，任一 `ok=false` 退 1。

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/e203-b8-criterion-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-opus-run /home/fy5090/code/singlefs/research/prompts/e203-b8-criterion-r2-opus-model
```

（第二个参数是 `--frozen`：这一轮没有冻结副本，模型只读仓里的留存产物，传仓根占位；`rerun.sh` 只用它判是不是目录。）

```
e5f9a9e69fc6364519772915884842e5dad5e410f7670d2549e32adb06c4f473  research/prompts/e203-b8-criterion-r2-opus-model/b8_r2_readings.py
5ed547adbad57ea3defa6544423376e018f3790760608771ac4b525b02ff2bdd  research/prompts/e203-b8-criterion-r2-opus-model/tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  research/prompts/e203-b8-criterion-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh
```

`tools/rerun.sh` 拷自 `scripts/agents/three-way-rerun-template.sh`，只换了 `rerun_steps` 一段（核过：`diff <(sed -n '1,79p;85,164p' 模板) <(sed -n '1,79p;89,168p' rerun.sh)` 无输出）。模型我没跑（定义不许腿自己起），下文凡标「量的（产物行）」的数是我用 `grep` / `awk` 直接读留存产物、求和、做算术得来；模型把同一批数再算一遍并钉住，判决要引的数以主 agent 复跑为准。

## 各格判定一览

| 格 | 判 | 一句话依据 |
|---|---|---|
| 问题一 · 岔路单第 9 行 | 开着。执行员「没够判」站得住，但漏了一层：够判条件里「改动至少 1000 个单元」那一半在任何跑得到的规模上都等不来（推的），照现写法这一行永远关不上 | 第二节；第一段 K1000 的 `uK` 档被测臂已有 3 格「介于」（0.281、0.290、0.295），外推到一百倍跨度 6 格里 2 格 `C1` 仍不红 |
| 问题一 · 第 10 行 | 开着（用户已定那一格留开，不重判）。补一条事实：已记的那一格翻面是两端量窗不对称造出来的，「打中不分辨臂」 | 低端 80 个操作、高端 401 个（量的）；同一高端点上峰从 15167 步到 22641 步，记录数只差 3.7%（量的）；同窗两点 α = 0.333 |
| 问题一 · 第 11 行 | 开着，执行员这一判站得住；有了 E209 还差四样 | 重现条件没写出（行一窄读法其实对得上，被旗线退化遮住）；多出的步数按 B8 行分散、按固定点轮却集中在定下那一轮；K 大那一侧没扫过；尖峰在重挂载时搬进写行发布，E207 那一刻暂停读数 |
| 问题二 · 三条出路 | 打中：前两条买到的多半是又一次作废（推的），第三条必作废（推的）；第一条在服务规则下根本交不进去 | 第二节表 |
| 问题二 · 第四条路 | 有两条，都零轮：按 B8 式子的输入逐项判（不放宽，见第二节）；只在装置拷贝里给历史那一段提速（不碰 `crates/`） | 第二节 |
| 问题二 · 都站不住怎么记 | 按「打中不分辨臂」：这一档的作废同时拿掉「站住」「翻面」两臂，另立一笔账，第 9、10 行只看 `u1` 档 | `.claude/rules/evidence-discipline.md:124` |
| 问题三 | 打中题面前提：那几处不是「没被看过」——E203 登记把它们列成 s1、s24、s30 三行，B1 数了真步数，B8 换成差量做法；fs-design 那句与 D8 已定项 16 射程那句都罩；三份实验的 B8 口径没有少数它们。真正没量过的是重挂载写行那一次发布 | 第三节；量的：一单元覆盖写准入之前 `records_of_each_leaf` 每次约 9.0 × 记录数，B1 准入之前合计 α 0.536（K100）、0.510（K1000） |
| 问题四 · 旗线退化（读法 1） | 打中，「判别子观测不到」。改得动：行一窄读法两个变体的两处不符全由它造出，换不退化的旗线后三类都 0 处不符（推的，从产物行）。改不动行一的判：岔路单第一行的规则是宽读法，旗上操作的宽读新鲜度全是 0 | 第四节 |
| 问题四 · 可比操作不限挂载次序（读法 7） | 没打中：改不动行五。8 个判「分散」的旗上操作里至少 3 个的两个可比操作与它同一次挂载，「一处分散即不成立」照立 | 第四节 |
| 问题四 · 不复现的点算不算取样点 | 打中，「打中归错了判据」：第八节写死受几何敏感性管的是 Q2–Q11，没有 Q0；那一点该记「K 方向没扫过」，不是「不稳定」。改不动五行 | 第四节 |
| 问题五 | E207、E209 对 D8 已定项 16 都只能写「备料」，回看理由要改；对 D28 已定项 1 两页正文都没提，不必加行，硬要写是「不影响」；E203 选臂动不了 | 第五节 |
| 撤回 | 修正第一轮本方「收严四」：「总量差至少两个数量级」不够让大改动档的扫描对照红 | 第六节 |

## 一、问题一：岔路单第 9、10、11 行

### 结论

| 行 | 该记 | 执行员的判哪一处站不住 |
|---|---|---|
| 第 9 行 B8 站住 | 开着 | 「没够判，等第四段」里「等第四段」站不住：够判条件写的「阳性对照……都判红」在 `uK` 档上大点也多半等不来（推的，第二节）；`u1` 那一半就算把已记的翻面拿掉，也有 4 格「介于」，照登记行一要「全部不随」，所以这一行在现写法下没有一条可跑的路能关上 |
| 第 10 行 B8 翻面 | 开着（复量那一格用户 2026-11-13 已定留开，这里不重判它的去留） | 不改状态；补一条：`u1` 档唯一一格翻面的读数分不开两臂（下面），`uK` 档在作废之前的被测臂读数是 0.098–0.295，没有一格 ≥ 0.5 |
| 第 11 行 尖峰另算 | 开着 | 「仍开」站得住；E209 之后还差四样（下面），其中一样是 E209 自己的读数里已有、被装置遮住的 |

### 第 9 行的依据

- 够判条件原文：「改动至少 1000 个单元、改动远小于总量」（`research/prompts/e203-b8-criterion-r1-forks.md:9`）。
- `u1` 档里不靠那一格也翻不成「站住」的四格（窗都对称：重挂载两端各 80 个操作，量的，第 10 行依据里的命令）：「alpha=0.484 band=between」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4568`，K100 重挂载 `first`）；FTB 重挂载 `later_median`「alpha=0.321 band=between」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4776`）。登记行一的写法：全部 `u1` 格都「不随」才站住（`research/prompts/e207-preregistration.md:264`）。
- `uK` 档：「scan_control_red=12」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:5250`），整档作废。作废之前的被测臂已经有「介于」：K1000 同一次挂载 `later_median`「alpha=0.290 band=between」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4700`）。大点能不能救这一档见第二节：照 K1000 各格的 α 外推，6 格里 2 格 `C1` 仍不红（推的）。

### 第 10 行的依据：那一格是量窗大小造出来的

- 已记的那一格：「alpha=0.502 band=grows」（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4552`），两端 `ends=7167/1/25346,22641/1/250190`。
- 两端各自的量窗操作数（量的，命令与原样输出，数的是产物里同一次挂载的 `name=declared` 行）：

```
cd research/results && for p in files_100_units_100 files_1000_units_100; do F=e207-strict-scan-steps-$p-head-2026-11-13.out; for w in W1 W2 W3 W4 W5 W6 W7; do printf "%s %s same_mount=%s remounted=%s\n" $p $w "$(grep -c "^E207RESULT name=declared item=${p}__head phase=window state=same_mount operation=$w " $F)" "$(grep -c "^E207RESULT name=declared item=${p}__head phase=window state=remounted operation=$w " $F)"; done; done
files_100_units_100 W1 same_mount=32 remounted=32
files_100_units_100 W2 same_mount=32 remounted=32
files_100_units_100 W3 same_mount=8 remounted=8
files_100_units_100 W4 same_mount=8 remounted=8
files_100_units_100 W5 same_mount=0 remounted=0
files_100_units_100 W6 same_mount=0 remounted=0
files_100_units_100 W7 same_mount=0 remounted=0
files_1000_units_100 W1 same_mount=32 remounted=32
files_1000_units_100 W2 same_mount=32 remounted=32
files_1000_units_100 W3 same_mount=8 remounted=8
files_1000_units_100 W4 same_mount=8 remounted=8
files_1000_units_100 W5 same_mount=200 remounted=0
files_1000_units_100 W6 same_mount=110 remounted=0
files_1000_units_100 W7 same_mount=11 remounted=0
```

  低端 80 个操作里取最大，高端 401 个里取最大。登记读法 7 只写「`later_peak` = 其余里步数最大的一次」（`research/prompts/e207-preregistration.md:37`），没要求两端同一个操作集合。
- 同一个高端点、记录数几乎不动，峰只因多读了 321 个操作就涨了一半（量的，产物行算术）：W1–W4 的峰 15167 步在 241260 条记录上（「15167/1/241260」，`research/results/e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out:2636`），W7 第 1 次 22641 步在 250190 条上（`research/results/e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out:2640`）：步数比 1.493，记录数比 1.037。两端同取 W1–W4：α = ln(15167 ÷ 7167) ÷ ln(241260 ÷ 25346) = 0.333（算的，介于）。
- 四句：① 不分辨臂：「步数不随总量」那条臂下，401 个操作里的最大值同样大于 80 个里的最大值，这一格两臂都预言「高端更大」；② 不是被判系统做决定的事，「看不看得到」不适用；③ 满足的是登记行二「任一 later_peak 或 later_median 的 steady 格」α ≥ 0.5 且对照红那一句（`research/prompts/e207-preregistration.md:264`）的字面；④ 跑前给的改法里，复量段的公共窗（登记设计员修订 2）在这一格上起作用（六点主格判 `unstable`），第一轮第三节那六条修补没有一条碰得到它。
- 归类：「打中不分辨臂」（`.claude/rules/evidence-discipline.md:124`）。用户已定这一格照留、不加点，这里只把归类交主 agent 写进判决；第一轮的新登记形态缺「两端取同一个操作集合」这一条，算第一轮修补本身漏的一格。

### 第 11 行：有了 E209 之后还差的四样

1. **重现条件没写出来，但读数里有一条被遮住的候选。** 行一按登记的宽读法确实不成立：四个训练旗上操作的宽读新鲜度都是 0（产物行 `fresh_mount_wide=0`，见第四节），规则在训练集上就定不起来。窄读法（「带出」= 被释放链拉进来）在训练集 0 处失败、检验集各 1 处不符，那两处都是旗线在一两个操作的窗里退化（第四节）；换掉退化的旗线，窄读法在 742 个操作行上 0 处不符（推的，产物行逐行核，第四节）。也就是「这次释放链拉进来的、挂载以来没被拉进过的节点数 ≥ 18 ⇒ 旗上」这条规则，E209 的读数没有反例。它是看过产物才写的，被攻过零轮，要另立一次登记才能进第 11 行。
2. **「具体一步」取决于怎么切。** 行五按 B8 的 28 行切：8 个分辨够的旗上操作前三行只占 0.717–0.777，判「分散」。同一批行里按固定点的轮切，多出来的步数几乎全在定下那一轮：「2:settled:1.022」（`research/results/e209-spike-cause-report-third-2026-11-13.out:851`），两个 U 点是「3:settled:0.912」（同文件第 902 行）与 0.946（第 922 行）；F = 200 的五个分在两轮，「2:placement_not_covered:0.497,3:settled:0.497」（同文件第 940 行），其余四个 0.399–0.509 对 0.491–0.603（第 942–952 行）。也就是 F ≤ 100 时集中在定下那一轮，F = 200 时平分在落点并入那一轮与定下那一轮，切法换了，「一步」的答案跟着几何变。E207 Q7 与 E209 行四给的站点都是第一次并入时顺释放链拉进来的那一处（`release_not_covered`）。第 11 行的够判条件「定到发布里的具体一步」按哪种切法算，岔路单没写。
3. **K 大那一侧没扫过。** E209 的取样点全在 F ≤ 200，「F = 1000 的点（E207 复现尖峰的那个）不进本实验」（`research/prompts/e209-preregistration.md:200`）；K 方向唯一的点 `files_10_units_1000` 不复现（「verdict=not_reproduced」，`research/results/e209-spike-cause-report-third-2026-11-13.out:86`）。E207 在 K 大的两点上复现过尖峰（`files_100_units_1000` 与 `files_10_units_10000` 的 W1 第 1 次在旗上，E207 页第一段现状 F7 那一条），E209 一个都没放。
4. **尖峰没有消失，重挂载时搬进了写行那一次发布，而那一次没人量过步数。** 行四：写行那一步带出「carried_by_the_mount=51 share=0.962」（`research/results/e209-spike-cause-report-third-2026-11-13.out:849`），F = 200 时「carried_by_the_mount=103」（同文件第 936 行），F = 75 时「carried_by_the_mount=37」（同文件第 958 行）：带出的节点数对 F 的 α = ln(103 ÷ 37) ÷ ln(200 ÷ 75) = 1.04（算的，量的是节点数不是步数）。E207 在重挂载那一刻暂停读数、把这一段的读数丢掉：「pool.crash_and_mount_writable().expect("崩溃后可写挂载");」（`scripts/experiment-apparatus/e207_strict_scan_steps_histories.rs:1001`），「drop(readings::take_settle_lines(&item, "phase=remount", false」（`scripts/experiment-apparatus/e207_strict_scan_steps_histories.rs:1006`）。所以 E207 重挂载那一半「不旗」读的是写行已经替它付过账之后的操作；写行那一次发布的 B8 步数随不随总量涨，E207、E209 都没量（第三节第 4 条）。

## 二、问题二：多单元改动档量不出

### 结论

打中：三条出路里没有一条能在不放宽判据的前提下让这一格判得出来（推的）。前两条不放宽也不换对象，但买到的多半是又一次作废；第三条必作废；第一条在今天的服务规则下交不进去。第四条路有，但要么同样买不到判，要么换了阳性对照的形态、只能另立一次登记。都站不住时，这一格按「打中不分辨臂」记，另立一笔账。

### 为什么大点也救不了：扫描对照的判别力取决于「记录数 ÷ 这次改动的步数」

`C1` 的定义：「每次操作的 B8 步数 + 操作之前的」`allocation_records`（`research/prompts/e207-preregistration.md:232`）；核过：K1000 `first` 格对照的两端正好是被测臂加记录数（218083 + 22420 = 240503，272551 + 216376 = 488927，`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4684`、`:4685`）。改 1000 个单元的一次发布自己就有约 2.1–3.2 × 10⁵ 步，多扫一遍 2 × 10⁴ 条记录只占一成，`C1` 红不红取决于低端的记录数比这次发布的步数大多少，跨度只是次要的。登记自己写过：「贴线，不保证红」（`research/prompts/e207-preregistration.md:56`）。

把 K1000 那 6 格被测臂的 α 当作在一百倍跨度上不变（推的；低端每单元步数取 K1000 低端的量的值，低端记录数取 K1000 低端，高端记录数取 2.0 × 10⁶ 与 2.5 × 10⁶；第四段中点量到 `allocation_records=215488`，十倍是 2.15 × 10⁶），`C1` 的 α：

| 格 | 被测臂 α（K1000，量的） | 每单元步数低端（量的） | 外推高端（推的） | `C1` α，R高 = 2.0 × 10⁶ | 2.5 × 10⁶ |
|---|---|---|---|---|---|
| 同一次挂载 `first` | 0.098 | 218.1 | 339–347 | 0.507 | 0.524 |
| 同一次挂载 `later_peak` | 0.281 | 218.4 | 772–822 | 0.544 | 0.557 |
| 同一次挂载 `later_median` | 0.290 | 213.8 | 784–836 | 0.550 | 0.563 |
| 重挂载 `first` | 0.295 | 214.0 | 797–851 | 0.554 | 0.566 |
| 重挂载 `later_peak` | 0.236 | 318.6 | 910–960 | **0.481** | **0.495** |
| 重挂载 `later_median` | 0.118 | 311.3 | 527–542 | **0.454** | **0.472** |

登记要「应判的格全部判随」，否则那一（段 × 档）作废。照这张表，第四段 `uK` 档多半再作废一次（2 / 6 格不红，推的）。即使对照全红，K1000 上被测臂已有 3 格 0.281–0.295「介于」，跨度变大 α 不变，大点给的多半也是「介于」，第 9 行站不住、第 10 行翻不了（推的）。这张表由模型第二节重算（`name=c1_extrapolated`），换成实测的第四段两点读数要跑「要主 agent 跑的命令」第 2 条。

### 三条出路各自的代价与买到的东西

| 出路 | 代价 | 放不放宽判据、换不换被判对象 | 买到的东西 |
|---|---|---|---|
| 整条跑过四小时 | 约 56000 秒独占一台（执行员推的）；服务规则里「确认放不过四小时硬上限」（`.claude/agent-common.md:69`），交不进去，要先改服务规则（用户的事） | 不放宽、不换对象 | 推的：`uK` 档多半再作废（上表），对照全红时多半是「介于」 |
| `crates/` 加会话与读数状态的存读入口 | 产品代码为实验加一层面（要决策与实现员）；总 CPU 时间不变，只是切片 | 用 serde 存读整份会话不换对象；用 `crash_and_mount_writable` 接回会多一次写行与暖机发布，换了历史（执行员已指出） | 同上一行 |
| 改登记里的最大点 | 新登记 | 规模变小、跨度变小：不放宽字面，但看过前三段才改点，属新登记 | 推的：跨度不到一百倍时每格 `C1` 都更低（30 倍跨度 `first` 格约 0.41），必作废 |

### 第四条路

- **只在装置拷贝里给历史那一段提速或加存读，不碰 `crates/`。** 慢的是被测实现自己的 B1 代码，不是计数：E207 在历史那一段暂停读数（`scripts/experiment-apparatus/e207_strict_scan_steps_histories.rs:955` 起），而今天的实现每次一单元覆盖写准入之前要按全部记录分叶约 9.0 遍（第三节）。计数本来就插在拷出来的树里，同样可以在拷贝里把历史那一段的全量找与全量比换成结果逐字节相同的做法，或给拷贝加存读；钉法是已量完的较小点上每次发布 `final_set_hash`、历史末镜像逐个相同。不放宽、不换对象，但买到的与第一条一样（推的：多半作废）。零轮。
- **按 B8 式子的输入逐项判。** B8 的步数全由 `by_delta_rows` 的式子吃量到的输入算出（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:179` 起：轮数、每轮变了的叶、`m_round`、`dirty` 等），一次发布的 B8 步数要随总量涨，只能经某个输入随总量涨或对数因子。逐个输入判「随不随」，每个输入的阳性对照是往那个输入里加一份记录数，α ≈ 1，十倍跨度就红，K1000L 的低端与中点（产物已在，合并报告没出）就够用。代价与限度：对「站住」那条臂，它把旧判据下作废的格变成可判，照 `.claude/rules/evidence-discipline.md`「只许收严」算不上收严，只能当「另立一笔账修共用前提」的新登记；还要另加「B8 合计照原判据判」，免得对数因子攒起来漏过。零轮，只在这份报告里推过，没跑。

### 都站不住时这一格怎么记

四句：① 不分辨臂：作废同时拿掉「站住」「翻面」两臂的判，病根是两臂共用的阳性对照在可达几何上判别力不够；② 不是被判系统做决定的事，「看不看得到」不适用；③ 满足的是作废条款 1（`uK` 档 `C1` 不红即该档作废）的字面，登记已定项 1 早预告过；④ 改法里，大点（已定项 1 给的改法）在这几格上照样中（推的，上表），前三条出路都碰不到它。

按「打中不分辨臂」那一行办：「不拿它判臂：把那个共用前提另立一笔账先修，判臂只看分辨得出臂的那几格」（`.claude/rules/evidence-discipline.md:124`）。落法：`uK` 档记「作废，判别力在可达几何上不够（推的）」，不进第 9、10 行；第 9 行的够判条件里「改动至少 1000 个单元」那一半要么由用户删掉，要么等那笔账（换阳性对照形态的新登记）判完再回来；第 9、10 行只按 `u1` 档判。

## 三、问题三：每次发布按全部分配记录做一遍的那几处

### 结论

打中题面的前提：「登记里计数的扫类步数不数这几处」不对。E203 登记把它们列成 s1、s24、s30 三行，B1 那一臂数了今天的真步数，B8 那一臂按登记写死的差量做法换掉；E206、E207 的「B8 步数」吃的是同一份换法。所以三份实验的 B8 口径没有少数它们。fs-design 那一句与 D8 已定项 16 射程那一句都罩它们：几处都在「式子那一判」之前，拷贝随单元区（盘容量）涨，分叶随记录数涨。今天的实现因此在这两句上是红的，这一点 E203 B1 早已够判。真正没被看过的是两件：B8 那几样差量做法从没实现过；重挂载写行那一次发布的步数没人量过。

### 依据：代码里的次序与每一处随什么涨

- 发布入口先整份拷分配器：「let allocator_before_this_publish = allocator.clone();」（`crates/singlefs-core/src/transaction.rs:13213`），上一行注释「第一版每次发布约 450 KiB 的拷贝」（`crates/singlefs-core/src/transaction.rs:13212`）。被拷的是「pub devices: Vec<DeviceFreeMap>,」（`crates/singlefs-core/src/allocator.rs:2285`，每盘几张按槽的位图，随单元区涨）与「records: Vec<AllocationRecord>,」（`crates/singlefs-core/src/allocator.rs:2294`，全部分配记录）。
- 两道准入的位置：落点那一判「admit_the_units_landing_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13422`）；然后走固定点「let settled = settle_the_allocation_record_tree(」（`crates/singlefs-core/src/transaction.rs:13435`）；式子那一判「admit_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13525`）。固定点每一轮「let mut rehearsal = allocator.clone();」（`crates/singlefs-core/src/transaction.rs:14076`），两处「nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());」（`crates/singlefs-core/src/transaction.rs:14085`、`:14105`），两处装着记录的节点（`:14093` 那一支与「let holding_records = nodes_holding_records(&geometry, rehearsal.records());」`crates/singlefs-core/src/transaction.rs:14106`）。都在式子那一判之前。
- 比较按两份全量记录各分一次叶：「let before = records_of_each_leaf(geometry, previous_records);」（`crates/singlefs-core/src/allocation_record_tree.rs:373`），`:374` 是 `after`；装着记录的节点同样整份分叶：「for leaf in records_of_each_leaf(geometry, records).into_keys() {」（`crates/singlefs-core/src/allocation_record_tree.rs:344`）。
- 同形的另一处 `settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`（`crates/singlefs-core/src/transaction.rs:1975` 起，拷贝在 `:1930`、`:2038`）只走在没有文件的那一版上，记录数小，不影响下面的结论；题面没列它，补上。

### 依据：量到的增长（量的，留存产物里 `name=row` 行的 `steps` 按段求和，只取准入之前的 `caller`、`before_landing`、`between` 三段）

一次一单元覆盖写（W1 第 3 次，两端 `user_units=1`），K100 两点与 K1000 两点：

| 集合 | 记录数 | `records_of_each_leaf`（s30） | 两份固定点拷贝（s24） | 入口拷贝（s1） | B1 准入之前合计 |
|---|---|---|---|---|---|
| K100 低 → 高 | 24050 → 239268 | 216688 → 2155132，α 1.000，每条记录 9.01 遍 | 8673290 → 9153640 | 4336645 → 4576820 | 16629097 → 56918290，α 0.536 |
| K1000 低 → 高 | 22250 → 216070 | 200464 → 1946250，α 1.000 | 8666326 → 9062140 | 4333163 → 4531070 | 15873508 → 50629318，α 0.510 |

同一格 B8 的步数在几千步（第一段 `u1` 档两端 3730–22641，产物行），B1 准入之前是它的两千倍以上。拷贝两行在记录数轴上不怎么涨，是因为两点单元区相同（2¹⁹），它们随单元区涨，E203 的 U 轴已量过：「row=whole_allocator_copy_before_the_judgement low_end=101959 high_end=103286290 alpha=0.998444」（E203 合并报告第一段，见 E203 页「三段结果」）。求和命令（原样，`research/results/` 下起）：

```
for p in files_100_units_100 files_1000_units_100 files_10_units_1000 files_100_units_1000; do F=e207-strict-scan-steps-$p-head-2026-11-13.out; grep "publish=same_mount/W1/3/0 " $F | grep 'name=row ' | awk -v p=$p '{for(i=1;i<=NF;i++){if($i~/^row=/)r=substr($i,5); if($i~/^steps=/)st=substr($i,7); if($i~/^segment=/)sg=substr($i,9)}; if(sg=="after") next; if(r=="records_of_each_leaf")a+=st; if(r=="whole_allocator_copy_in_a_settling_round")b+=st; if(r=="whole_allocator_copy_before_the_judgement")c+=st; tot+=st} END{print p, "s30="a, "s24="b, "s1="c, "before_admission_rows="tot}'; done
```

模型第五节重算这张表并核两端 `user_units` 相同（`name=before_admission_b1`）。

### 依据：谁数了它们、B8 拿什么换掉

- E203 登记的行表：s1「whole_allocator_copy_before_the_judgement」（`research/prompts/e203-preregistration.md:130`），s24「whole_allocator_copy_in_a_settling_round」（`research/prompts/e203-preregistration.md:153`），s30「records_of_each_leaf」（`research/prompts/e203-preregistration.md:159`）。
- B8 的做法：s1「撤回日志：失败时照日志倒回，不整份拷」（`research/prompts/e203-preregistration.md:256`）；s24「差量视图叠在分配器上，每轮从同一个底子重放」（`research/prompts/e203-preregistration.md:273`）；s30「只分被动过的记录」（`research/prompts/e203-preregistration.md:279`）。E207 模型照抄：「("whole_allocator_copy_before_the_judgement", 0),」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:188`），「input.changed_slots_and_records_per_round.iter().sum(),」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:269`），「("records_of_each_leaf", input.dirty_per_call.iter().sum()),」（`scripts/experiment-apparatus/e207_strict_scan_steps.rs:295`）。

### 罩不罩

- fs-design：运行时决策路径「且代价不许随盘容量增长」（`.claude/rules/fs-design.md:23`），被禁的两件之一是「在需要答案的那一刻还没有答案」（`.claude/rules/fs-design.md:34`）：这次哪些节点变了，不是增量记着的，是要用的那一刻把两份全量记录各分一遍叶比出来的，落在这一句里；s24 拷的位图随单元区涨，正是盘容量。罩。
- D8 已定项 16 射程：「准入之前的步代价不随盘上容器总数、记录总数涨」（`.claude/kb/decisions/08-核心索引结构.md:517`）。s30 随记录数一次涨（上表）。罩。
- B8 那一行：它说的是换成差量做法之后的步数，今天的代码不是 B8，它不替今天的代码作保。E203 已判 B1「够判：留不住」（`.claude/kb/experiments/203-准入之前扫类的步代价.md:54`），D8 已定项 16 的索引行却写着「进度：已实现。」（`.claude/kb/decisions/08-核心索引结构.md:28`），射程那一句在今天的实现上不成立。这是已知的，不是这一轮新打中的；记在这里是因为「没被看过」的说法会让人以为它们不在账上。

### 没被看过的两件

1. B8 的三样差量做法（撤回日志、差量视图、只分被动过的记录）在 `crates/` 里一样都没有，E207 的「B8 步数」是把今天固定点的输出（轮数、重写的节点、每轮动了的槽）喂进式子算的。这要求差量视图上走出来的固定点与整份拷贝上走出来的逐个相同，登记写的是「认：每一行取的是平衡有序映射或分层位图这类通用做法」，没有原型，被攻过零轮。
2. 重挂载写行那一次发布：E209 行四说尖峰的节点在它里面带出（第一节第 11 行第 4 条），它不判式子（「一个普通分配都没有的发布（空发布、写行、暖机、抬 F）不判」，`crates/singlefs-core/src/transaction.rs:13490`），但固定点照走、落点照取；E207 在这一刻暂停读数，它的 B8 步数随不随总量涨没有读数。节点数随 F 的 α 1.04（量的节点数，推的步数）。

## 四、问题四：E209 三处没写死的口径

| 口径 | 改不改得动已判的五行 | 归哪一种 | 要不要另立登记 |
|---|---|---|---|
| 挂载窗里只有一两个操作时旗线退化（读法 1） | 改不动行一的判（行一的规则是宽读法）；改得动行一窄读法两个变体那两行，从「不成立」变成三类都 0 处不符（推的，产物行逐行核）；行二至行五不动 | 判别子观测不到 | 要：窄读法那条规则只能在新登记、新取样上判 |
| 行五可比操作不限挂载次序（读法 7） | 改不动：至少 3 个判「分散」的旗上操作的两个可比操作都与它同一次挂载 | 不算打中，四种都不归；只动分母（26 个分辨不够、15 个没有可比） | 判行五不要；想把分母补齐才要 |
| 不复现的点算不算几何敏感性的取样点 | 改不动 | 打中归错了判据 | 判五行不要；要量 K 那个旋钮才要 |

### 读法 1：旗线在小窗里退化

- 写死的窗：「同点同状态量窗」= 一个项里同一次挂载的全部脚本操作（`research/prompts/e209-preregistration.md:24`）。窗里一个操作时下中位数就是它自己，旗线 4 倍于它，永远旗不上；两个操作时下中位数是小的那个。
- 两个物理上一样的事件，一个旗上一个不旗：训练点 `first_mount_first` 第 0 次挂载第 1 个操作「final_nodes=53 flagged=true」（`research/results/e209-spike-cause-report-third-2026-11-13.out:89`）；`old_mount_old` 第 0 次挂载第 1 个操作「final_nodes=53 flagged=false」（`research/results/e209-spike-cause-report-third-2026-11-13.out:374`）。两行同一份历史、同在历史之后第一个操作、种子 11 个、拉进 42 个、链深 23，窄读新鲜度都是「fresh_mount_narrow=42」。差的只是这个操作之后窗里还有几个操作（2 个对 1 个）。F = 75 那一处同理：「final_nodes=39 flagged=false」（`research/results/e209-spike-cause-report-third-2026-11-13.out:391`），窗里另一个是「final_nodes=11」（`research/results/e209-spike-cause-report-third-2026-11-13.out:392`），旗线 44。
- 窄读法的两处不符正是这两个：「script_mismatch_labels=files_100_units_100__head__old_mount_old#1:fresh=42:observed_flag=0」与「geometry_mismatch_labels=files_75_units_100__head__first_mount_first#1:fresh=28:observed_flag=0」（`research/results/e209-spike-cause-report-third-2026-11-13.out:832`）。
- 小窗里到底有哪些操作过 28 个节点（量的，命令与原样输出；列的是操作不超过三个、且有操作 ≥ 28 个节点的窗）：

```
cd research/results && F=e209-spike-cause-report-third-2026-11-13.out; grep 'name=operation_row ' $F | awk '{for(i=1;i<=NF;i++){split($i,a,"="); v[a[1]]=a[2]}; k=v["item"]"|"v["mount_epoch"]; n[k]++; row[k]=row[k] sprintf(" [%s fn=%s fl=%s fmn=%s fmw=%s]", v["operation"], v["final_nodes"], v["flagged"], v["fresh_mount_narrow"], v["fresh_mount_wide"])} END{for(k in n) if(n[k]<=3) print n[k], k, row[k]}' | sort -k2 | awk '{ if ($0 ~ /fn=([3-9][0-9]|2[89]|[1-9][0-9][0-9])/ ) print }'
2 files_100_units_100__head__first_mount_first|0  [overwrite_first fn=53 fl=true fmn=42 fmw=0] [overwrite_old fn=13 fl=false fmn=0 fmw=0]
1 files_100_units_100__head__old_mount_old|0  [overwrite_old fn=53 fl=false fmn=42 fmw=0]
2 files_100_units_100_unit_area_2_pow_17__head__first_mount_first|0  [overwrite_first fn=53 fl=true fmn=42 fmw=0] [overwrite_old fn=13 fl=false fmn=0 fmw=0]
2 files_100_units_100_unit_area_2_pow_20__head__first_mount_first|0  [overwrite_first fn=53 fl=true fmn=42 fmw=0] [overwrite_old fn=13 fl=false fmn=0 fmw=0]
2 files_200_units_100__head__first_mount_first|0  [overwrite_first fn=107 fl=true fmn=90 fmw=0] [overwrite_old fn=11 fl=false fmn=0 fmw=0]
2 files_75_units_100__head__first_mount_first|0  [overwrite_first fn=39 fl=false fmn=28 fmw=0] [overwrite_old fn=11 fl=false fmn=0 fmw=0]
```

  不足三个操作的窗只有这六个有操作过 28；把这种窗的旗线换成只看下限 28，旗变的只有 `old_mount_old#1` 与 F = 75 那一个，两者窄读新鲜度 42、28 都 ≥ 门槛 18，于是窄读法两个变体在训练、脚本检验、几何检验三类上都是 0 处不符，两类检验的旗上操作 18、32 个都 ≥ 3：照 Q2 写法是「成立」（推的，模型第三节 `name=freshness_recheck` 钉 0、0、0）。
- 四句：① 它不是让各臂一起出局的那种，宽读法照样不成立，只有窄读法被它拉下；② 判据要规则区分 `first_mount_first#1` 与 `old_mount_old#1`，而任何由这次操作之前的历史算出的量在两行上逐条相同（上面），旗的差来自之后的操作，系统在那一刻看不到；③ 满足的是 Q2「任一类出现一处不符 ⇒ 不成立」的字面；④ 跑前写的唯一护栏「一个项里旗上操作超过半数时」另标 `median_inflated`（`research/prompts/e209-preregistration.md:24`）在这两格上没起作用，两行都是 `median_inflated=false`（旗上 0 / 1、1 / 2，都不超过半数）。
- 归「判别子观测不到」：「改判据，改完是新一轮」（`.claude/rules/evidence-discipline.md:125`）。行一的判不动：岔路单第一行的规则是宽读法（「岔路单第一行的规则是」`C_mount` 宽读那一份，`research/prompts/e209-preregistration.md:260`），四个训练旗上操作的宽读新鲜度都是 0，「variant=mount_wide threshold=0」（`research/results/e209-spike-cause-report-third-2026-11-13.out:831`），规则在训练集上就立不起来，这与旗线无关。行三池化分错至少 20 个，两个旗换掉动不了；行四的窗 5 个操作，不退化；行五多两个旗上操作可判，「一处分散即不成立」照立。

### 读法 7：可比操作不限挂载次序

写死的是「按项名字典序取前两个」（`research/prompts/e209-preregistration.md:30`），没限挂载次序。8 个判「分散」的旗上操作的可比操作（量的，模型第四节逐个查挂载次序）：训练点那一个的两个可比操作是「comparable_first=files_100_units_100__head__create_then_first#3」与 `first_repeated#2`（`research/results/e209-spike-cause-report-third-2026-11-13.out:851`），两份脚本里没有重挂载，都在第 0 次挂载，与旗上操作同一次；两个 U 点那两个（第 902、922 行）取 `filler_create_4#7`、`filler_young_4#7`，同样没有重挂载。F = 200 的五个才取到重挂载之后的操作，如「comparable_second=files_200_units_100__head__mount_then_first#2」（`research/results/e209-spike-cause-report-third-2026-11-13.out:942`），噪声 658–3216。Q11 写「任一「分散」⇒ 不成立」，限定挂载次序之后训练点与两个 U 点那三个仍是分散（0.771、0.717、0.717），行五照判不成立。

### 不复现的点

- 第八节写死受几何敏感性管的量：「每个带判决的量（Q2、Q3、Q4、Q5、Q6、Q7、Q8、Q9、Q11）」（`research/prompts/e209-preregistration.md:304`），没有 Q0；F10 的触发是「同一行同一变体在两个取样点上判定不同」（`research/prompts/e209-preregistration.md:353`）；Q0 不复现时「该点所有行的判都记」装置走不到（`research/prompts/e209-preregistration.md:258`），作废条款 1 同句「该点上行一至行四的判记」装置走不到（`research/prompts/e209-preregistration.md:361`）。
- E209 页把它写成「字面按 F10 复现闸一量不稳定」（`.claude/kb/experiments/209-尖峰成因.md:805`）。按条款字面，那一点上发生的是 Q0 的「不复现」，归作废条款 1，不归 F10；行二、行四在那一点上没有判定，谈不上「判定不同」。
- 四句：① 不涉及臂；② 不涉及被判系统；③ 满足的是 Q0「两者都为 0」那一分句，不是 F10；④ 页里并列的「第二种读法」（把行二、行四记不稳）照条款不成立。
- 该怎么记：第八节自己写了「扫不到的（F = 1000 估计超 4 小时，不跑）写」没扫过（`research/prompts/e209-preregistration.md:315`）。K 与 F 反向那个旋钮唯一的取样点不复现，该记「K 方向没扫过」，不是「不稳定」。E207 在 `files_100_units_1000`、`files_10_units_10000` 上复现过尖峰（第一节），要扫 K 方向就从这两点里挑，另立登记。

## 五、问题五：对已定条款的关系

### 结论

| 实验 | D8 已定项 16 | D28 已定项 1 |
|---|---|---|
| E207 | 备料（不升支撑，也不能写推翻）；回看理由要改：把 0.502 那一格当射程证据的那半句换掉 | 页里正文没提 D28，不必加行；要写就是「不影响」 |
| E209 | 备料，回看照留，补一句写行发布 | 同上，「不影响」 |

E203 选臂：第一轮交用户的两项前提都已有了结果（判据形态 2026-11-09 采用；整份覆盖写「整份覆盖写用户定登欠账」，`records/2026-10-13-里程碑四预开发.md:1587`），但证据选不出臂：B1 留不住，B8 在 `u1` 档有「介于」格、第一次发布的尖峰随总量一次涨、第 11 行没关、`uK` 档判不出；B4 本来就只能当改条款交。动得了的只有用户直接定：选 B4 改射程，或定「尖峰另算」的豁免怎么写。照证据不动。

### 依据

- 规则：关系四选一；「实验必须对应决策」（`.claude/rules/format-evolution.md:38`），「支撑、推翻要写到分项」，且那条分项的依据要引回这个实验（`.claude/rules/format-evolution.md:39`）。D8 已定项 16 的依据里没有 E203、E206、E207、E209 任一个（依据列表在 kb 第 521–533 行，`awk` 取第 519–536 行数 `E20[3679]` 得 0）。
- 被判那一句：「准入之前的步代价不随盘上容器总数、记录总数涨」（`.claude/kb/decisions/08-核心索引结构.md:517`）。它是用户 2026-10-31 定的读法，是要求，不是对代码的观测；实验能说的是今天的代码与候选做法满不满足它，说不了这条要求本身错。
- E207 对这一句实际说了什么（量的，产物行）：一是 B8 自己的式子在第一次发布上随总量涨，K100「first filter=all」那一格 α 1.180（`research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out:4540`），K1000 1.005（同文件第 4636 行），FTB 0.608（第 4732 行），这正是第 11 行要「另算」的那一块；二是今天的实现（产物里的 B1 行）一单元覆盖写准入之前合计 α 0.536、0.510（第三节）。两样都说这一句在今天的代码与 B8 上不成立，要成立得靠「尖峰另算」的豁免或改射程。不写推翻：条款的定案（第一版 unlink 怎么删）没被它碰到，射程那一句也只是没被满足；要写推翻得先由用户改条款、在依据里引回实验。
- E207 页的回看现写「在 `one_user_unit` 档新点上一格过线（α = 0.502，贴线，复量六点不稳）」（`.claude/kb/experiments/207-严格判据下的扫类步数.md:394`）。照第一节，那一格是量窗大小造出来的，作为「射程那一句过线」的证据不站；该换成上一条的两样。
- 顺带一格（交主 agent，我没判）：D8 已定项 16 的索引行写「进度：已实现。」（`.claude/kb/decisions/08-核心索引结构.md:28`），而 `.claude/kb/decisions.md` 的判法里「未实现」含「有而不照它做」（第 27 行）。射程那一句今天不被满足（E203 B1 够判），算不算「有而不照它做」，取决于射程句算不算这一条分项的内容，规则没写，我没往下推。
- E209：登记自己写「本实验不判这两句」（`research/prompts/e209-preregistration.md:49`）。它的读数（重挂载的写行那一步带出尖峰那批节点，第一节第 11 行第 4 条）是写「尖峰另算」豁免时要用的料：豁免要写清尖峰在同一次挂载里落在第一次覆盖写、在重挂载时落在写行，两处都要。所以「备料」；页上回看「不受影响」的理由照留，补一句「写行那一次发布不判式子、E207 没量它的步数」。
- D28 已定项 1：两页正文都没提 D28（`grep -o 'D[0-9]\+（' 两页` 只有 D8，命令在开工时跑过），照「实验正文……提到的每条决策都要有一行」（`.claude/rules/format-evolution.md:37`）不用加行。硬要写：式子的需求取「这次新写的全部角色（普通分配加固定点）」，写行不判式子；E209 量到写行带出 37–103 个节点，这份空间由已定项 4 的保留池罩，保留池按分配记录树整棵算（「拿得准的上界只有整棵树」，`.claude/kb/decisions/28-挂载期承诺量.md:119`），所以不影响。我核过的只有这两处字面，没算过「整棵树」与 103 个节点在小盘上的大小关系。

## 六、撤回与修正第一轮本方提过的方案

- **修正「收严四」。** 第一轮本方写：「大改动档要有「改动相同、总量差至少两个数量级」的取样，没有就记没判，不记不随」（`research/prompts/e203-b8-criterion-r1-opus-output.md:155`）。登记照这句加了 K1000L 一百倍跨度。第二节的外推说这还不够：大改动档扫描对照红不红，主要看低端记录数比这次改动的步数大多少，一百倍跨度下 6 格里仍有 2 格不红（推的）。「两个数量级」不是够判条件，我撤回「有了这样的取样就判得出」的含义；够不够要按「低端记录数 ÷（改动单元数 × 每单元步数）」现算。
- **修正「收严五」在大改动档上的适用。** 第一轮本方写：「阳性对照：B1 真步数、合成扫描臂在判据上必须红，不红判据作废」（`research/prompts/e203-b8-criterion-r1-opus-output.md:156`）。对 `u1` 档它站得住（42 / 42 红）；对 `uK` 档，在可达规模上它让这一档永远作废（第二节）。我不撤回「必须红」，撤回的是「同一种合成扫描臂两档通用」：大改动档要另配判别力够的对照（第二节第四条路，零轮），配不出就照「打中不分辨臂」另立一笔账，不拿这一档判臂。

## 要主 agent 跑的命令

1. 模型整份复跑（在新 shell 里交，工作目录随意，写的只有运行目录）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/e203-b8-criterion-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-opus-run /home/fy5090/code/singlefs/research/prompts/e203-b8-criterion-r2-opus-model
```

   - 在哪起：仓根。运行目录 `/home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-opus-run`，包装脚本先删它再交给 `tools/rerun.sh`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `name=rerun_done rc=0 checks=… failed=0` 与 `name=done checks_failed=0`。这份报告引的数对应：`name=window_asymmetry … alpha_common_two_points=0.333`；`name=freshness_recheck variant=fresh_mount_narrow flags=flag_floor_in_small_windows … training_failures=0 script_mismatches=0 geometry_mismatches=0`；`name=c1_extrapolated_summary … not_red=2`（三种高端记录数各一行）；`name=before_admission_b1` 三行（K100 的 W1#3、W2#3 与 K1000 的 W1#3；K1000 两点 W2 第 3 次改动大小 1 对 1000，不比）；`name=spread_operation … same_mount=yes` 至少 3 行；`name=remount_carried_growth … alpha_against_files=1.044`。任一 `ok=false`（退 1）就是我这份报告的读法或数错了，照那一行改判，不是装置的毛病。内存与时间：读 8 份产物、最大一份 62.8 MB，逐行读，没量过，推的几十秒、几百 MB 以内。
2. 第四段较小两点的合并报告（把第二节的外推换成两点实测的每单元步数；E207 页说「合并报告没出」）：

```
task-server/task submit --cwd /home/fy5090/code/singlefs/scripts/experiment-apparatus -- cargo run --release --bin e207-strict-scan-steps -- report --segment fourth /home/fy5090/.local/share/singlefs/work/e207-b8-strict-runner-22/run-fourth/shard-raw/files_10_units_1000_unit_area_2_pow_21__head-0-of-1-1.out /home/fy5090/code/singlefs/research/results/e207-strict-scan-steps-files_100_units_1000_unit_area_2_pow_21-head-2026-11-13.out
```

   - 在哪起：`scripts/experiment-apparatus`（同 E209 页复跑那一条的写法）。
   - 低端产物没进 `research/results/`，在执行员的运行目录里有两份：`e207-b8-strict-runner-22/…`（前 16 位 24418365fd5ff13c，Nov 13）与 `e207-b8-strict-runner-19/…`（191aa9610995bc7f，Nov 12），两份都有 1 行 `name=item_done`。哪一份是判定库里那一份我核不动，交主 agent 定；命令里写的是较新那一份。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=`（读数器有判决行为假时退 1，不等于跑坏，读日志）；日志里 `name=cell segment=… set=K1000L` 的 `uK` 档被测臂与 `arm=c1` 两行的 `ends=`，各格低端、中点的每单元步数。拿它们代替第二节表里的「每单元步数低端」与「被测臂 α」，用模型第二节同一个式子把 `C1` 推到大点。登记作废条款 7 说只剩两点时只报两点，不当一百倍跨度判，这一条只用来量判别力，不判第 9、10 行。

## 没打中的形状

- 第一轮修补里「x 取量到的总量」：那一格 x 取 `mapped_units` 是 0.453、取记录数是 0.502，主读是跑前写死的记录数，不算打中。「改动不变读原步数」「记一次输」在 `u1` 档照做了，没找到反例。
- 多单元改动档「只红 12 格就整档作废」：按格判（只留对照红的 K100 那 12 格）试过，那几格被测臂最大 0.309（介于），而且 K100 的改动是 100 个单元，不进第 9 行「至少 1000 个单元」那一半，改不动任何一行。
- `tree_changed` 不分是哪次提交：`files_1000` 本来不进任何 K、D、FTB 集合，重挂载一侧的格两端都在今天的树上，找不到经它改动第 9–11 行的路。
- E209 行二、行三、行四、行五在旗线换掉之后：行二靠五个复现点的「跟操作走」，行三池化分错至少 20，行四的窗 5 个操作，行五一处分散即不成立，都不动（第四节）。
- E209 行四与 D28：写行带出的节点由保留池按整棵树罩，没找到不够的取样点；没在小盘上算具体大小。
- 准入之前还有没有 E203 行表外的全量步：试了 `previous_nodes.clone()`、`rewritten.intersection(&nodes_after)`、`holding_records == nodes_after`，都按节点数走（约记录数 ÷ 400），s15 记了前一样，后两样没核是不是记了；量级比 s30 小两个数量级，不改第三节的结论。
- 取样范围：E207 第一段合并报告的全部 `u1` 格与 K1000 `uK` 格、复量段两行、四个点的 W1#3 与 W2#3、E209 第三段合并报告的全部 742 个操作行与 8 个分散操作；没读 E206 的产物。

## 这条腿自己的限度

- 只抽了一次；「没打中」的几格要第二份同立场样本才算数。
- 第二节的外推假设 K1000 各格被测臂的 α 在一百倍跨度上不变、低端记录数与 K1000 低端相同；第四段两点的产物在手，B8 步数要模型算，我没跑，所以那张表全是推的。
- 第四节「窄读法换掉退化旗线后成立」是看过 742 行之后才说的，换法（小窗只看下限 28）也是我挑的，被攻过零轮，只是线索。
- 第三节的 B1 求和只取了两个操作（W1#3、W2#3）、四个点；没量第四段最大点慢在哪几行，「慢的是 B1 代码不是计数」是从历史段暂停读数推的，计数在暂停时还做不做别的事我没读。
- 第二节第四条路两样都只在这份报告里推过：没有原型，没跑，没被攻过。

## 没做什么

- 没往统一任务服务交任务，模型没跑；判决要引的数由主 agent 复跑（上一节两条）。
- 没建仓副本、没编译，草稿目录 `/home/fy5090/.local/share/singlefs/work/e203-b8-criterion-r2-opus` 里只有认领、锁与引文核对的输出 `cite.txt`，没有要删的副本或编译目录。
- 不判正推、辩方那几格，不替主 agent 采纳；复量那一格的去留照用户 2026-11-13 定的不重判。
- 没读禁读的那一份；执行员 `result.md` 只在背景里转述的范围内当线索，没去读原件。
