# e203-b8-criterion-r2：判决（2026-11-13）

<!-- doc-lint:not-numbers B1 B8 F5 F10 Q0 Q10 Q11 W1 W2 W5 W7 K100 K1000 -->

正文 `research/prompts/_e203-b8-criterion-r2-body.md`，开工快照 `research/prompts/e203-b8-criterion-r2-snapshot/`。这一题的第二轮；第一轮判决 `research/prompts/e203-b8-criterion-r1-main-verification.md`。第一轮定下的新登记形态落成了 E207（严格判据下的扫类步数），尖峰成因另立了 E209（尖峰成因）；这一轮判两份实验量出了什么。用户 2026-11-13 定的那一格（E207 复量：已记的翻面不动、`unstable`、留开着、不再加点）这一轮照题面不重判。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端攻方（Opus） | `research/prompts/e203-b8-criterion-r2-opus-output.md` | 岔路单三行都记「开着」；量不出的那一格「三条出路都判不出这一格」；题面前提「登记里计数的扫类步数不数这几处」打中，「E203、E206、E207 的 B8 口径没有少数它们」；E209 三处口径「已判的五行都不动」；撤回第一轮本方的「收严四」「收严五」 |
| 云端辩方（Sonnet） | `research/prompts/e203-b8-criterion-r2-sonnet-output.md` | 「第一轮判决：站住，没有读数说明判错」；B8 那一行拆开读，典型的后来发布守得住、挂载后第一次发布守不住、后来发布里最坏的一次读不稳；三条出路里「改最大点：放宽」；整份拷贝与比较「条款罩，依据更硬」；D8 已定项 16 两页都写「备料」，不升 |

核查命令与它们判了什么：

- `sha256sum -c research/prompts/e203-b8-criterion-r2-snapshot/judged-sha256.txt research/prompts/e203-b8-criterion-r2-snapshot/kb-sha256.txt`：28 份里 27 份 OK，`.claude/kb/checks-owed.md` 对不上（腿跑着的时候被别的会话改过）。两份腿报告里 `grep -c 'checks-owed'` 都是 0，腿引的行没有一处落在这份文件上，所以没有倒推快照树（`scripts/agents/snapshot-tree.py` 没跑）。
- `python3 scripts/agents/cite-check.py research/prompts/e203-b8-criterion-r2-opus-output.md research/prompts/e203-b8-criterion-r2-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_e203-b8-criterion-r2-background.md`：末行「✓ 核了 123 处引文、59 处裸行号，对上 123 处，没判 59 处（其中裸行号没有引文可比 59 处；逐处列在上面）」。判决承重的裸行号主 agent 逐处现读：`crates/singlefs-core/src/transaction.rs` 第 13213、14031、14076、14085、14105、14106 行；`.claude/kb/decisions/08-核心索引结构.md` 第 517 行（射程）与第 28 行（索引行）；`.claude/rules/fs-design.md` 第 23 行；`research/prompts/e203-preregistration.md` 第 130、153、159、256、273、279 行；`scripts/experiment-apparatus/e207_strict_scan_steps.rs` 第 188、269、295 行；`research/prompts/e207-preregistration.md` 第 393、409 行。都与腿写的相符。
- 判决引的读数由主 agent 经任务服务跑出或直接从留存产物读出：
  - 辩方的两份脚本拷到主 agent 自己的目录复跑：任务 16036 末行 `operations=742 registered_flagged=52 floor_only_flagged=54 predicted_narrow=54 registered_vs_predicted_mismatches=2 floor_only_vs_predicted_mismatches=0`（两处不符是 `files_100_units_100__head__old_mount_old` 与 `files_75_units_100__head__first_mount_first` 各自挂载里的第 1 个操作）；任务 16037 六行 0.3327、0.5024、40486、2386、55796、43458，与它报告里写的相同。
  - 攻方的模型拷到主 agent 的目录复跑：任务 16079 退 1，`name=rerun_done rc=1 checks=7 failed=0`，崩在读 E209 那一节（`b8_r2_readings.py` 第 155 行按 `__head__` 拆项名，plain 那一项拆不出）。崩之前的读数重现了：`name=window_asymmetry … alpha_recorded=0.502 … alpha_common_two_points=0.333`、两端量窗 `operations=80` 与 `operations=401`、`name=c1_extrapolated_summary high_records=2154880 cells=6 not_red=2 which=remounted/later_median,remounted/later_peak source=extrapolated`。崩掉的那一节要证的（换掉退化的旗线之后窄规则 0 处不符）由辩方脚本那一条罩住。
  - 攻方「今天的实现准入之前的步数」那张表模型没跑到，主 agent 照它报告里的求和命令直接读两份产物：`files_100_units_100 leaf=216688 round_copy=8673290 entry_copy=4336645 total_before_admission=16629097`、`files_1000_units_100 leaf=2155132 round_copy=9153640 entry_copy=4576820 total_before_admission=56918290`，与表里 K100 一行逐个相同；K1000 一行没核。
  - 第四段较小两点的合并报告（任务 16080，输出 `~/.local/share/singlefs/work/e203-b8-criterion-r2-main-checks/report-fourth-two-points.out`，没进 `research/results/`）：第 760 行 `name=verdict quantity=q10 reading=big_points_against_the_total segment=big tier=user_units_equal_declared cells=12 cells_unread=12 …`；阳性对照在低端到中点这一对上的 α：第 699 行 `… pick=first filter=all arm=c1 … lower_pair_alpha=0.313`、第 723 行 `… state=remounted … pick=first filter=all arm=c1 … lower_pair_alpha=0.438`。
  - 复量各取法的判语主 agent 现读 `research/results/e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out`（公共窗、被测臂、单个单元改动档）：同一次挂载 `later_median` 的 `steady` β=0.177 `stable_does_not_grow`、`first` 的 `all` β=1.058 `stable_grows`、`first` 的 `steady` β=0.187 `stable_does_not_grow`、`later_peak` 的 `steady` β=0.345 `unstable`；重挂载 `later_median` 0.220 `stable_does_not_grow`、`later_peak` 0.448 `unstable`、`first` 0.401 `unstable`。
- 核不动与只抽一次样的：攻方只抽了一次样，它写「没打中」的几格（E209 行五的可比操作那一处）只当线索；E209 行一至行五的全部读数两条腿都只核了汇总行与涉及三处口径的行，主 agent 读过的是正文第三节抄的那几行。

正文的一处错：第二节与第四节第 3 问写「登记里计数的扫类步数不数这几处的拷贝与比较」。两条腿都指出不对，主 agent 现读坐实：这几处就是 E203 登记行表的 s1 `whole_allocator_copy_before_the_judgement`、s24 `whole_allocator_copy_in_a_settling_round`、s30 `records_of_each_leaf`（`research/prompts/e203-preregistration.md` 第 130、153、159 行），产物里有它们的行。腿照登记判的，主 agent 写错了。

## 二、判决

1. **岔路单三行：都记「开着」，两条腿一致，主 agent 采。**
   - 第 9 行（B8 站住）：「改动至少 1000 个单元、改动远小于总量」那一半在跑得到的规模上判不出，见第 2 条；单个单元改动档 `row_one_holds=false`，不靠 0.502 那一格（辩方：`steady` 格里另有 4 个「介于」）。
   - 第 10 行（B8 翻面）：用户已定留开的那一格之外，复量把这一行拆成三样读数——典型的后来发布不随总量涨（`later_median` 两种状态 β 0.177、0.220，`stable_does_not_grow`）；挂载后第一次发布随总量涨（同一次挂载 `first` 的 `all` 格 β 1.058，`stable_grows`；剔掉旗上操作的 `steady` 格 0.187）；后来发布里最坏的一次读不稳（`later_peak` 0.345、0.448，`unstable`）。`only_the_first_grows=true` 是对各组取「或」得来的（辩方引 `scripts/experiment-apparatus/e207_strict_scan_steps.rs:1534`），不读成「只有第一次随」。
   - 第 11 行（尖峰另算）：E209 把尖峰的载体定到重挂载的写行那一步（五个复现点 share 0.949–0.981），够判条件的前一半「定到发布里的具体一步」有了；后一半「写清它重现的条件」没有——行一四个读法都判不成立，而窄读法的两处不符正是旗线退化造的（任务 16036：换成只用下限 28 的旗，742 个操作 0 处不符）。这条「窄规则其实对得上」是腿在跑后算出来的，登记不许回头改判据（辩方引 `research/prompts/e209-preregistration.md:344`、`:366`），不当机制证据，要用就另立一轮登记。E209 没到大几何（两条腿一致）。
2. **量不出的那一格：三条出路都判不了它；按登记自己的回落记。** 两条腿一致。多单元改动档的扫描阳性对照判别力不够：前三段 54 格只红 12 格；第四段低端到中点（总量十倍）实测 α 0.313、0.438（任务 16080），没到 0.5；外推到一百倍 6 格里 2 格仍不红（攻方模型，推的）。所以最大点就算跑出来，这一档多半再作废一次。整条跑过四小时不放宽判据，但约 4 万到 5.6 万秒独占一台（任务 16037 的两种外推，推的）且进不了任务服务的四小时上限；在 `crates/` 加存读入口不放宽，代价是改产品代码，买到的同上；改登记里的最大点是放宽（登记已定项 1 写死总量跨度 100 倍，F10 已打中）。登记写了回落：F10（`research/prompts/e207-preregistration.md:393`）「大点记「本次未跑」，另两个小点照报，第一至三段结论不受影响」，作废条款 7（`:409`）「只剩低端和中点时 Q10 只报 {低端, 中点}（总量十倍），不当作跨 100 倍」。这一格归 `.claude/rules/evidence-discipline.md`「判据自己也会写错」那张表的「打中不分辨臂」：病根在阳性对照这把尺子，不在被测臂。
3. **每次发布按全部分配记录做一遍的那几处：登记数了它们；今天的实现在这几行上随总量涨；条款罩它们。** 两条腿一致，主 agent 现读坐实。今天的实现是登记里的 B1 那一臂：同一种一单元覆盖写，`records_of_each_leaf` 从 216688 步涨到 2155132 步（记录数 24050 到 239268，每条记录约 9 遍），准入之前合计从 16629097 涨到 56918290。B8 那一臂是登记写死的差量做法（s1「撤回日志：失败时照日志倒回，不整份拷」、s24「差量视图叠在分配器上」、s30「只分被动过的记录」，`research/prompts/e203-preregistration.md` 第 256、273、279 行），模型把这三行定价成 0 或按改动量（`e207_strict_scan_steps.rs` 第 188、269、295 行），`crates/` 里没有实现。`.claude/rules/fs-design.md:23`「且代价不许随盘容量增长」与 D8（核心索引结构） 已定项 16 射程里「准入之前的步代价不随盘上容器总数、记录总数涨」（`.claude/kb/decisions/08-核心索引结构.md:517`）都罩这几处。这不是这一轮新量出来的：E203 的 B1 早已够判。没被看过的两件（攻方）：B8 的差量做法本身从没实现、没量过真步数；重挂载写行那一次发布的步数 E207 不量（那一刻暂停读数）。
4. **E209 三处登记口径：都动不了已判的五行。** 旗线在操作很少的挂载窗里退化：两条腿都归「判别子观测不到」，行一的「不成立」不靠它（宽读法训练失败 27 处是另一个原因）。行五的可比操作不限挂载次序：辩方按上界算术判改不动行五，攻方没打中（一次采样）；不采「限定挂载次序」这一改法。不复现的点：攻方归「打中归错了判据」（Q0 不在登记第八节点名的量里，该记「K 方向没扫过」），辩方归「打中不分辨臂」；两种归法的处置相同——那一点照作废条款 1 记「装置走不到」，几何敏感性那一行写明不稳的是 K 与 F 反向的那一个点。
5. **对已定条款的关系：E207、E209 对 D8（核心索引结构） 已定项 16 都维持「备料」；D28（挂载期承诺量） 已定项 1 写「不影响」。** 两条腿一致。E203 选臂照证据动不了：岔路单第 9、10 行的够判条件（阳性对照都判红）在多单元改动档没满足。
6. **第一轮判决：站住。** 辩方复核没有读数说明第一轮判错。第一轮没写到的两处同样打中所有替代形态，记下：「后来发布里最大的一次」在两端量窗操作数不等时不可比（80 对 401，贴线格就是这样来的）；扫描阳性对照在改动 1000 单元那一档力度不够。攻方撤回第一轮本方的「收严四」「收严五」（两个数量级的跨度不够让大改动档的扫描对照红；同一种扫描对照不能两档通用），主 agent 认。
7. **不开第三轮。** 两条腿在五问上结论一致，没有要再攻的分歧；这一轮里新冒出的形态（换旗线、换阳性对照、按 B8 式子的输入逐项判）都是零轮，写进交用户表。

## 三、改法

1. E207 三语页「影响的决策」D8（核心索引结构） 已定项 16 那一格的回看句里引 0.502 的那半句换成复量的读数（量过：主 agent 现读复量报告第 2636 行）。
2. E207、E209 三语页把这一轮判的写进去：岔路单三行都「开着」与各自还差什么；第四段照 F10 与作废条款 7 记——大点「本次未跑」、Q10 只报低端与中点（十倍）、不当作跨 100 倍；多单元改动档记「打中不分辨臂：阳性对照判别力不够」；E209 不复现的点写「K 方向没扫过」（量过：任务 16080、16036、16037 与第一节现读的行）。
3. 岔路单 `research/prompts/e203-b8-criterion-r1-forks.md` 三行的状态照旧是「开着」，不改字；每行还差什么写在页里。
4. 腿提的零轮改法不进装置与登记，只列进交用户表：只用下限的旗线重判行一；多单元改动档换一种阳性对照；按 B8 式子的输入逐项判；在装置拷贝里提速或加存读。它们都要另立登记，登记之前不动。这一轮没有要写进产品代码的改法，没有 A/B 要做。
5. 正文那一处错在第一节写明，正文不改（冻结证据）。

## 交用户的

| 项 | 量到的 | 状态 |
|---|---|---|
| 第四段最大点怎么办 | 登记的回落是记「本次未跑」、只报十倍跨度（`research/prompts/e207-preregistration.md:393`、`:409`）；硬跑约 4 万到 5.6 万秒独占一台（推的），且阳性对照在十倍跨度上实测 0.313、0.438 不红（任务 16080），跑出来这一档多半仍作废 | 主 agent 照登记的回落记；要硬跑或改 `crates/` 加存读入口，要用户另定 |
| D8（核心索引结构） 已定项 16 的索引行写「进度：已实现」，而射程里「准入之前的步代价不随盘上容器总数、记录总数涨」今天的实现不满足 | `.claude/kb/decisions/08-核心索引结构.md:28`、`:517`；今天的实现一单元覆盖写准入之前 16629097 步对 56918290 步（记录数差十倍） | 攻方提的，主 agent 现读坐实；进度怎么写、要不要记欠账，要用户定 |
| E203 选臂 | 多单元改动档的够判条件没满足，证据上动不了；B8 的差量做法没实现、没量过真步数 | 等用户：选臂是设计决定，证据只到「B1 留不住、B8 的模型在单个单元改动档上典型发布不随」 |
| E209 行一要不要用只看下限的旗线另立一轮 | 换旗线后窄规则 742 个操作 0 处不符（任务 16036，跑后算的） | 零轮；不立也不影响已判的五行 |
| 多单元改动档要不要换阳性对照另立登记 | 现有扫描对照在这一档判别力不够（54 格红 12 格；十倍跨度 0.313、0.438） | 零轮；不立的话岔路单第 9 行那一半一直开着 |

## 这一轮被判过的文件

- research/prompts/e203-b8-criterion-r1-forks.md：判了三行各自的够判条件满足到哪一步、状态该记什么
- research/prompts/e203-b8-criterion-r1-main-verification.md：辩方复核它第二节各格与第三节新登记形态站不站得住
- research/prompts/e207-preregistration.md：判了已定项 1、F10 与作废条款 7 对第四段跑不出的回落，三条出路哪一条是放宽
- research/prompts/e209-preregistration.md：判了三处口径各归「判据自己也会写错」那张表的哪一种、改不改得动五行
- research/prompts/e207-spike-cause-questions.md：判了五行的判定在三处口径下动不动
- crates/singlefs-core/src/transaction.rs：判了发布入口与固定点每轮的整份拷贝、两次全量比较随什么涨，条款罩不罩
- crates/singlefs-core/src/allocator.rs：判了被整份拷的集合随单元区与记录数怎么涨
- crates/singlefs-core/src/mount.rs：判了重挂载写行那一次发布是尖峰的载体、E207 在那一刻暂停读数
- scripts/experiment-apparatus/e207_strict_scan_steps.rs：判了 B8 那一臂把三行定价成 0 或按改动量、`only_the_first_grows` 的取法、阳性对照在两档的判别力
- scripts/experiment-apparatus/e209_spike_cause.rs：判了旗线在小窗里的退化与可比操作的取法对五行的影响
- scripts/experiment-apparatus/e203_admission_scan_cost_counters.py：判了它数的行里有整份拷贝与按记录分叶这三行
- research/results/e207-strict-scan-steps-report-first-2026-11-13-r3.out：判了单个单元改动档总判、多单元改动档作废、贴线格两端量窗不对称
- research/results/e207-strict-scan-steps-report-first-2026-11-13-r4.out：判了它是上一份的留存形态、读数相同
- research/results/e207-strict-scan-steps-report-second-2026-11-13.out：判了三种重现假设都不成立对第 11 行的含义
- research/results/e207-strict-scan-steps-report-third-2026-11-13.out：判了更大总量下扩展窗口里旗上操作为 0 对第 11 行补了哪一格
- research/results/e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out：判了复量各取法的判语把第 10 行拆成哪三样读数
- research/results/e207-strict-scan-steps-report-reconciliation-split-2026-11-13.out：判了 `tree_changed` 只说树变了、对第 9、10 行起不了翻面作用
- research/results/e207-strict-scan-steps-report-reconciliation-split-slots-placed-undercounted-2026-11-13.out：判了阳性对照 ① 红在放置读数的负值上
- research/results/e207-strict-scan-steps-report-reconciliation-split-image-witness-flipped-2026-11-13.out：判了阳性对照 ② 红在计数版对不插计数版的分叉上
- research/results/e209-spike-cause-report-third-2026-11-13.out：判了五行的汇总读数、不复现的点、几何敏感性那一行不稳的是哪一点
- research/results/e209-spike-cause-report-third-2026-11-13-raw.out：判了它是上一份去掉前缀的原始形态、辩方脚本读的 742 个操作行出自同一份读数

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 备料 | 2026-11-13 不受影响：两份实验量的是 B8 模型的步数与尖峰成因，分项字面不动；射程那句今天的实现不满足、索引行写已实现，这一处交用户 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-13 不受影响：这一轮没有读数碰到承诺量的定义；E209 的写行载体涉及已定项 4 的保留池上界，辩方写明未核 |
