# m3-crash-enumeration-merge-r2 攻方腿（Opus）报告（2026-10-10）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13 G14 L1 L2 L3 L4 L5 L6 K1 K2 K3 K4 K5 K6 K7 F5 W -->

攻击面：正文第四节 L1、L2、L3、L6，只攻第三节 G1–G14 那一种形态，不攻第一节「用户已定」。前几轮判决：`research/prompts/m3-crash-enumeration-merge-r1-main-verification.md`；第一轮攻过的格（K1-1 … K7-2）不重攻，只攻它们的修补（G4、G6、G7、G8、G10）。

## 复跑与模型

- 模型目录 `research/prompts/m3-crash-enumeration-merge-r2-opus-model/`，复跑 `bash research/prompts/m3-crash-enumeration-merge-r2-opus-model/rerun.sh`：先 `sha256sum -c` 三份文件，再跑两份纯算术模型，两份末行都是 `MODEL_SELFCHECK … mismatched=0 []`。不读仓、不起 cargo、不上卡。
- `SHA256SUMS` 原样：
  - `ca991e16fb6107a58fb58615fe9f43d68510dc35d099391ab940a507630685dd  decode_alias.py`
  - `41a8f2c043f3f75fc374646f7b7843529137fe4b4ba9763e688f8c13f68c4fb4  count_model.py`
  - `b33550670142269fa249bcbb205b8a74a31fc61b080097b7141d1828beaecabb  rerun.sh`
- `decode_alias.py`：把 GPU 判器内核与 CPU 事实表「状态序号 → 持久掩码」的拆法逐行抄成算术（行号写在文件头），比两边在同一序号上拆出的掩码；带一格判别力自证（把 GPU 那一份换成 64 位余数，33 写那一格必须翻成相同，`MUTANT … gpu_equals_cpu=true`）。
- `count_model.py`：L2 挂钟模型，输入逐个写出处（records 的行与用例里钉的闭式）。
- 树：`git rev-parse HEAD` = `31e47115471ba5502a4531d25dffcc921be36899` 加工作区。开工 `sha256sum -c --quiet` 两份快照清单（crates 91 份、kb 5 份）都退 0；收工结果在末节。
- 中途收到「线程上限改成 6」：在读码阶段收到，之后没有起任何编译或测试，没有要改的命令。
- 这一轮没有编译任何 Rust，没有建仓副本；所有「推的」都是读码。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| L1-1 | 打中 | G8「这一项测试文件出发的闭包」进判法版本：按项算就断了前缀共享（多判 1 426 456 802 态）；按节点取并集就是改一项的测试文件、整段共享前缀跟着重判；放量的每一项都 `mod` 同两份公共测试模块，改其一整域重判 | L1 |
| L1-2 | 打中 | G4 观察者代码进计划哈希、G8 引擎整份进判法版本：只在 CPU 上跑的代码一改，GPU 判镜像那一截的块键与版本跟着换，重判一遍判不出新东西（改删除流观察者约 1.36 小时 GPU）；引擎整份进版本与用户 2026-09-29「派活和判法拆出来」相反 | L1 |
| L1-3 | 打中（定义缺口） | G4「挂在节点上、对共享这个节点的每一项都跑」与 G9「点名单跑一项与整域判定相同」：节点上挂哪几个观察者取决于这一趟跑了哪几项，单跑多记录流时删除流的挂载观察者在不在没定义 | L1 |
| L1-4 | 打中 | G7「逐条不变量的判到 / 不适用」GPU 结论字里没有（只有违例位）；G9 提交只等 GPU，改到判法的那次提交逐条计数给不出，「每一趟都判」做不到 | L1 |
| L1-5 | 打中（定义缺口） | G9「与已判等价的才剪」：引擎里今天没有任何等价剪枝，等价关系没定义；按态等价剪时，按块存的计数向量拆不出被剪那一态的计数 | L1 |
| L1-6 | 打中（小） | G6 比的四个恢复字只有看 journal 那一遍落到的根；不看 journal 那一遍落到的根 GPU 不出、不比，第一轮 K1-3 那一形在这一遍上照样比不出 | L1 |
| L1-7 | 打中 | G10：引擎只有点级 ignore，没有段级；`states_skipped_by_ignore` 按流自己的展开算，整域下在 322 写那一段 panic，跳过的态数 2^322 − 1 装不进 u64、也装不进 u128；G10 的界取 u64，而 GPU 判器的界是每段 2^32 态，33–64 写的段 G10 不登记、GPU 判器拆错序号 | L1 |
| L1-8 | 打中（定义缺口，推的） | G6 × G9：GPU 与 CPU 不在同一趟判，后到的那一边要读先到那一边每个状态的恢复字；库今天一态一个字节、全库至多 256 种判定向量，G6 没写恢复字存哪 | L1 |
| L2-1 | 打中 | 第一轮 GPU 速率拿多记录流那一项乘全域；到 E 那一项量过 9.9 万态/秒（多记录流的 0.29 倍），每项另有约 600 秒固定开销；按量过的整项挂钟串跑 9.8 小时（照 records 的共享）到 10.8 小时（照今天代码的共享），只算判、零开销、两台均衡的下界 4.9–7.2 小时，第一轮的 1.5–3.1 小时没有量过的数撑 | L2 |
| L2-2 | 打中 | 第一轮两条 CPU 速率差 5.8 倍，差的是流与流里的位置（层 0 那条是第二条流开头一分钟的短跑，放量那条是 24 写段占多数的到 D），不是「层 0 路 / 放量路」 | L2 |
| L2-3 | 给数（推的） | CPU 那一截跟上一次判法版本（每个状态 CPU 判 + 删除流整条逐状态可写挂载）：两台 31 线程 73–175 小时（照 records 的共享）、80–220 小时（照今天代码的共享）；第二条流一项 9.6–55.3 小时 |  L2 |
| L2-6 | 打中 | 第一轮两腿与判决用的「删除流与多记录流共享 1 358 954 604 态」是按段形推的；代码里 B、C 两步各在自己的测试文件里设点，身份带设点的文件，块键不同，今天只共享取号暖机与 A（16 777 259 态，恰是 2026-10-08 多记录流那一项的 reused）；去重态数 37.9 亿 → 51.3 亿，第一轮 K1-2 的 92.8% 在今天的代码上不成立 | L2 |
| L2-4 | 提醒（交主 agent，触 L5） | 「提交只等 GPU」时，整域上判镜像的是 WGSL 写的恢复与 checker；只改 `recovery.rs` 或 Rust checker 的那次提交，整域上没有一个状态经过改了的 Rust 代码（快档除外），GPU 那一截的重判结论与改前逐位相同 | L2 |
| L2-5 | 给数 | E171 冻结前 CPU 必须跟上 ⇒ 判法版本要连续 73–220 小时不变；2026-10-03 以来工作区 `crates/` 215 份文件有改动 | L2 |
| L3-1 | 打中 | 两套「跑在哪棵树」的核法：层 0 `--full` 在 HEAD + 暂存区的 worktree 跑、开跑与跑完各算一次指纹；放量在主工作区跑、判法版本在测试二进制起来时从盘上现算。2026-10-08 一趟全量两项判在两个判法版本下照报全绿；编完到现算之间闭包里的文件被改，判定就按新版本存、却是旧代码判的。G2、G11 没写取哪一种 | L3 |
| L3-2 | 打中 | G2 合表之后谁核「每条崩溃枚举用例都登记了」：`crash-case-check.py` placement 按 `enumerate_layer0` 名字认用例，第二批删掉层 0 全量用例、第四批改名之后它认不出任何一项；放量项不标 ignore，第 ③ 条也不管它们 | L3 |
| L3-3 | 打中（小） | modules 格的 `TIER_MODULES` 写着 `layer0_progress`、`crash_amplification`，19 份测试文件首行声明它们；feature-gated-tests 格把没卡记「本次未跑」，G11 要退 78，两种语义在同一道门禁里怎么分没写 | L3 |
| L3-4 | 打中（小） | 判法版本里的工具链只有 rustc、cargo、target，没有 GPU 驱动与卡型；G9 下 GPU 结论是提交那一刻的依据 | L3 |
| L3-5 | 没打中 | 旧库、旧全绿标记会不会被新形态误复用：版本定义换了、用例名换了，都自然失效 | L3 |
| L6-1 | 打中 | G10 排在第一批（不碰 `crates/`），它要的段级不展开、跳过态数的表示都在引擎里 | L6 |
| L6-2 | 打中 | G9 快档的甲二那一半上不了 GPU：判器内核与事实表只按每次写的两 / 三态拆序号，不认甲二；模型 5 写段甲二 7 态，GPU 拆出的 7 个掩码里 3–6 个甲二没有 | L6 |
| L6-3 | 打中 | G5 随机历史两项上 GPU 要在 `crash_injection.rs` 里建判表，这份文件不在 G14 任何一批 | L6 |
| L6-4 | 打中 | 第四批改英文标识符跨到第一批的文件：`capped.sh` 设 `SINGLEFS_LAYER0_THREADS`、`admission.py` 115 处 `LAYER0`、门禁样本 7 份、`crash-case-check.py` 的正则与模块表；不同批改，线程上限静默失效、判据认不出 | L6 |
| L6-5 | 打中（定义缺口） | G9 从哪一批起生效没写：随 G12 在第一批生效，第二、三、四批每次提交都换判法版本、各要一趟整域（GPU 各约 10 小时、CPU 各从头来）；「之后整域一趟」只在 G9 推到第四批之后才成立 | L6 |
| L6-6 | 结论 | 快档与整域第一次整套跑通：最早在第三批之后，而且随机历史那一截没有批次，照 G14 字面没有这一批 | L6 |

## L1　第一轮修补本身

### L1-1　G8 判法版本带「这一项测试文件出发的闭包」：按项算断前缀共享，按节点并起来就是改一项重判一片

**许可它的那一句**：`research/prompts/_m3-crash-enumeration-merge-r2-body.md:30`：「判法版本 = 判器闭包（crash.rs、GPU 判器）∪ 这一项测试文件出发的闭包 ∪ 引擎」；同一份 G3 要「身份与前缀共享」（同文件第 25 行）。

**今天共享靠什么**：块键与判法版本都相同才复用，判法版本今天是全局一份（`crates/singlefs-checker-tier/src/crash_identity.rs:840`：「"crates/singlefs-checker-tier/src/crash.rs",」，入口只有两份判器文件，同一段文档第 836–837 行写明设崩溃点的测试文件与流水线都不在闭包里）。两项之间判法版本一变，前缀就一态都不复用，记录里有先例：`records/2026-10-01-删除与并发事务.md:1292`：「判法版本 c7c1e096 → 93e1e4e3、键对不上」。

**造出来的历史（两种读法各一条，推的）**：
- 读法甲（每项一个版本）：同一棵树上跑整域。多记录流那一项把共享节点的块存在（块键，V多记录）下，删除流那一项去查（块键，V删除）：两项的测试文件不同，V 不同，一块都查不到。照 G3 共享的前缀全部重判：到 D ⊂ 到 E 那 67 502 198 态，加上 G3 若把多记录流 B、C 两步挪进公共模块要共享的 1 358 954 604 态。模型原样：`MODEL sharing_lost_if_versions_are_per_item states=1426456802 gpu_extra_hours=1.17 cpu_extra_hours=8.2..47.0`。
- 读法乙（共享节点取几项版本的并）：第 k 次提交只给删除流的测试文件加一行观察者。V删除变，共享节点的并也变，多记录流那一项挂在这些节点上的块一起作废、重判。「只改一项测试文件只重判那一项」不成立。
- 两种读法共有的：放量的每一项都经 `mod` 引同两份公共测试模块，例 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:24`：「#[path = "../../singlefs-harness/tests/common/mod.rs"]」、同文件第 27 行 `mod common_crash_points;`（多记录流第 25、28 行，到 E 第 10、12 行同形）；闭包按 `mod` 收（`crates/singlefs-checker-tier/src/crash_identity.rs:698`：「测试目标里的 mod 与 #[path]」）。harness 的 `tests/common/mod.rs` 2026-09-20 以来进了 9 次提交（`git log --since=2026-09-20 --format=%h -- crates/singlefs-harness/tests/common/mod.rs | wc -l` 输出 `9`），每一次都是整域重判。

**四句**：① 不分臂（两种读法都中，打的是 G8 字面与 G3 的组合）。② 复用那一刻系统只比（块键，判法版本），看得到；问题不在观测不到，在键的定义把「这一项」塞进了被几项共用的节点。③ 满足 L1「只改一项测试文件是不是只重判那一项」那一句。④ 形态没给改法。我提的：甲，判法版本按节点算，取「设这个点的文件出发的闭包 ∪ 判器闭包」（点今天就记着设它的文件 `code_file_in_repository`），节点上的观察者另按观察者版本存（见 L1-2 甲）；乙，照 G8 字面、另把公共测试模块拆成「只造流」与「只判」两份、只有后者进版本。甲在两种读法上都不中；乙只缓解第三条、读法甲照中。只在我的模型上量过、被攻过零轮。

### L1-2　只在 CPU 上跑的代码进了 GPU 结论的键

**许可它的那几句**：G4「观察者的代码进节点的计划哈希」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:26`）；计划哈希进块键（`crates/singlefs-checker-tier/src/crash_amplification.rs:253`：「enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),」）。G8 引擎整份进判法版本（同上第 30 行「引擎（crash_amplification.rs 与它的依赖）」）。

**GPU 结论跟它们无关**：GPU 判器判一个状态只读判表与内核（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:2`：「每个调用判一个崩溃状态」），观察者与可写挂载按 G5 只在 CPU 上跑。派活那一份今天特意不进闭包：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:19`：「那一份不在判法闭包里」，下一行「改它不换判法版本；这一份不许引它。」。引擎 `crash_amplification.rs` 里大半是派活、显存账、两台分工（文件头第 16–55 行），2026-10-03 以来工作区里这一份改了 4556 行（`git diff --stat HEAD -- crates/singlefs-checker-tier/src/crash_amplification.rs` 末行「1 file changed, 3990 insertions(+), 566 deletions(-)」），照 G8 每一次都整域重判。

**造出来的历史（推的）**：只改删除流测试文件里「停在中间读回上一版」那个观察者的一个断言消息。G4 下删除流节点的计划哈希变、块键变；G8 下判法版本也变。下一次提交 GPU 那一截把这些节点全部重判，结论字与上一次逐位相同（判表、内核一个字没变）。模型：`MODEL observer_edit_gpu_rejudge_hours=1.30..2.40`（只删除流的节点 … 加上照 G3 共享给多记录流的节点；按 2026-10-08 量过的两项整项挂钟）。

**四句**：① 不分臂。② 看得到：键里装着 CPU 侧代码的摘要。③ L1「G4 观察者进计划哈希，会不会让本来能共享的前缀不再共享」那一句的后半——不共享之外还多一层：GPU 重判。④ 改法：甲，一个块存三样、三把键：GPU 结论按（块键，GPU 判器版本），CPU 结论按（块键，CPU 判器版本），观察者结果按（块键，观察者版本）；复用各认各的，对拍（G6）在两边都在时比；乙，照 G8，但把引擎里改结论的那几个函数（`gpu_verdict_items`、`recovery_words_of`、`gpu_recovery_words`）挪进一份小文件、只它进版本，派活与显存账照今天不进。甲在这一格不中；乙只修引擎那一半、观察者那一半照中。被攻过零轮。

### L1-3　节点上挂哪几个观察者，取决于这一趟跑了哪几项

**许可它的那几句**：G4「对共享这个节点的每一项、每一个状态都跑」；G9「点名单跑一项与它在整域里跑判定相同」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:31`）。

**造出来的历史（推的）**：库是空的，先 `--item multi-record-publish` 单跑。照 G3 共享的节点上挂不挂删除流的挂载观察者？挂：单跑一项要先知道表里别的项会经过这些节点，得把别的项的流也录一遍（录制在 2026-09-30 那一趟单线程、每条约 480–520 秒，`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35`：「每条流录制约 480–520 秒」）。不挂：这一趟存下的块没有挂载观察者的结果，之后整域跑删除流时这批块要么照 G4 补跑观察者（与「单跑判定相同」不符），要么复用而缺一截。G4、G9 都没写取哪一边。

**四句**：① 不分臂。② 单跑那一刻系统看不到表里别的项会不会经过这个节点（除非读整张表、录别的流），属「判别子观测不到」。③ G9「点名单跑一项与它在整域里跑判定相同」那一分句。④ 改法：观察者挂在「标准前缀」这个登记了名字的节点上，由登记表静态列出（哪个节点挂哪几个观察者），不由这一趟跑了哪几项推出来；单跑一项照表把它经过的节点上登记的观察者都跑。被攻过零轮。

### L1-4　G7 的逐条不变量计数，GPU 给不出

**许可它的那几句**：G7「逐条不变量的判到 / 违例 / 不适用 … 这些断言每一趟（含剪枝那一趟）都判」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:29`）；G9「提交只等 GPU 那一截跑完」。

**今天**：CPU 一侧的计数里有不适用（`crates/singlefs-checker-tier/src/crash.rs:775`：「pub checker_not_applicable_states: BTreeMap<&'static str, u64>,」），第一条流的空真守门就靠它（`assert_checker_counts`，第一轮 K1-1）。GPU 一个状态的结论字只有违例位：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:10`：「第 0 字是红位」，第 8..12 字池级 checker 的违例位、第 12 字「走到第几步」（同段第 14 行），没有逐条的判到 / 不适用。

**造出来的历史（推的）**：第 k 次提交改了 Rust 池级 checker 的一条判法（判法版本换，整域重判）。提交只等 GPU：GPU 那一截新判的块没有逐条判到 / 不适用，计数向量只有一半；G7 要这一趟判的逐条计数断言判不了。要么照 G9 报「CPU 对拍本次未跑完」（那 G7 的「每一趟都判」就不成立），要么等 CPU（那 G9 的「只等 GPU」就不成立）。

**四句**：① 不分臂。② 系统在提交那一刻看不到逐条判到 / 不适用，属「判别子观测不到」。③ G7「这些断言每一趟（含剪枝那一趟）都判」。④ 改法：甲，GPU 结论字加逐条「判到」位（再 4 个字，128 位），G6 对拍连这几位一起比；乙，G7 写明逐条判到 / 不适用两列只随 CPU 那一截走，提交那一趟报「本次未核」，违例与闭式、按发布分的态数照判（这几样 GPU 给得出：违例位、序号）。甲要动内核（第三批）、换判法版本；乙只改条款，但这一格它照中（只是报出来）。被攻过零轮。

### L1-5　G9「与已判等价的才剪」：等价剪枝今天不存在

`grep -n '等价' crates/singlefs-checker-tier/src/crash_amplification.rs` 只命中第 4620 行一处测试注释，引擎里没有任何按等价剪的代码，G9 也没写等价关系是什么。若照 E161 那种「同一张镜像的状态只判一次」按态剪，被剪的那一态的逐条计数要从它的代表态拿；G7 按块存的是整块的合计（一块 65 536 态，`crates/singlefs-checker-tier/src/crash_amplification.rs:129`：「pub const STATES_PER_BLOCK: u64 = 1 << 16;」），拆不出单个代表态的那一份，闭式之外的计数就合不出来。四句：① 不分臂；② 系统看得到被剪态与代表态的对应，但看不到代表态单独的计数；③ G7「复用时与判定一起读回」；④ 改法：G9 写明等价剪枝只许以整块为单位（两块的写表、起步镜像与计划逐项相同才算等价），或这一版不做等价剪枝、删掉那半句。被攻过零轮。

### L1-6　G6 的四个恢复字比不到不看 journal 那一遍落到的根（小）

**许可它的那一句**：G6「与 GPU 逐状态比红项与恢复字（recovery_words_of 四个字）」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:28`）。

**逐位同义那一问**：四个字两边同口径（推的，读码）：GPU 取 `crates/singlefs-checker-tier/src/crash_amplification.rs:1817`：「[words[2], words[3], words[4], words[6]]」，内核写的是看 journal 那一遍落到的实例与 txg 低 32 位、两遍的结局（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:643`：「verdicts[out + 4u] = consulted.outcome;」，第 645 行 `ignored.outcome`）；结局编号两边都是 0 没有文件、1 读出、2 失败（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:85`：「const OUTCOME_FAILED: u32 = 2u;」）。

**比不到的**：不看 journal 那一遍落到哪条根——CPU 有（`RecoveryReport.effective_root`，`crates/singlefs-core/src/recovery.rs:344`：「pub effective_root: Option<(InstanceGeneration, CheckpointTxg)>,」，`Layer0StateJudgement` 两遍各一份），GPU 结论字里没有这个字（第 6 / 7 字只写这一遍的结局与失败点，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:13`：「第 4 / 5 字它的走读结局与失败点；第 6 / 7 字不看那一遍的；」）。两遍的失败点（第 5、7 字）GPU 有、G6 不比。

**造出来的历史（推的）**：GPU 在不看 journal 那一遍择错了根，落到一个也在版本表里的旧版本；两边 oracle 都判绿、结局都是「读出」，四个字相同、红项相同，G6 判两边一致。这正是 `crates/singlefs-checker-tier/src/crash_amplification.rs:1786`：「红项相同而落到的根不同也是判错」那一形，第一轮 K1-3 只把它在看 journal 那一遍补上了。

**四句**：① 不分臂。② CPU 看得到，GPU 没有输出这个字，属「判别子观测不到」（对 GPU 那一侧）。③ G6「比红项与恢复字」。④ 改法：GPU 结论字加不看那一遍的实例与 txg 低 32 位两个字，G6 改成六个字、连两遍的失败点一起比（八个字）。要动内核，排进第三批。被攻过零轮。

### L1-7　G10 按段登记：引擎没有段级的口子，跳过的数装不进，界取错了

**许可它的那一句**：G10「按段登记（不是整条流）… 报 states_skipped_by_ignore … 同一条流里装得进的段照判」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:32`）。

**引擎今天只有点级**：一段不展开有两条路——它归的那个点标了 ignore（`crates/singlefs-checker-tier/src/crash_amplification.rs:274`：「if self.points[self.owner_of_segment[segment_index]].ignored {」），或流自己的展开方式给它 `NotExpanded`（按展开上限，`crates/singlefs-checker-tier/src/crash_amplification.rs:172`：「if segment.len() <= self.0 {」）。前一条一并丢掉那个点管的别的段（今天 ignore 的单位是点）；后一条不进 `states_skipped_by_ignore`：那个数是按流自己的展开方式该有的态数减去这一份计划的态数（`crates/singlefs-checker-tier/src/crash_amplification.rs:6704`：「let without_ignore =」，下一行照 `flow.expansion` 算），流自己不展开的段算出来是 0，且 `exhaustive` 只比段长（`crates/singlefs-checker-tier/src/crash_amplification.rs:182`：「segments.iter().all(|segment| segment.len() <= self.0)」），整域那一趟照入口的规矩判红。

**跳过的数装不进**：整域照 G9 每段 `EveryProperSubset`，算 `without_ignore` 那一步在第三条按位置寻址流 322 写那一段上 panic（`crates/singlefs-checker-tier/src/crash.rs:2011`：「.expect("一段的状态数装得进 u64")」；用例自己写过 `crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs:406`：「第三条流的单元写段 322 写，debug 与 release（工作区开着溢出检查）都会 panic」）。就算不 panic，2^322 − 1 是 322 位的数：模型原样 `SKIPPED segment=322 states_skipped=322-bit fits_u64=false fits_u128=false`；字段是 u64（`crates/singlefs-checker-tier/src/crash_amplification.rs:6655`：「pub states_skipped_by_ignore: u64,」），入口按十进制整数累加。G10 要报的恰是装不进的那个数。

**界取错了**：G10 只登记装不进 u64 的段，装得进的「照判」、按 G5 上 GPU。GPU 判器内核拆段内序号只用低 32 位：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3528`：「var remaining = ordinal_lo - tab(T_SEGMENTS, s * 6u + 2u);」，CPU 事实表用 u64（`crates/singlefs-checker-tier/src/crash_facts.rs:556`：「let mut remaining = ordinal - segment.states.start;」）。判表打包不核段长（`crates/singlefs-checker-tier/src/crash_judge_tables.rs:1265`：「let (start_lo, start_hi) = split(segment.states.start);」前后没有判断）；只有第 ② 段核对器核（`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:304`：「segment.states.end - segment.states.start < u64::from(u32::MAX),」，它的内核注释写着「CPU 保证每段的状态数装得进 u32」，同文件第 116 行），G5、G6 没提第 ② 段留不留。模型逐行抄两边的拆法，原样：

```
DECODE writes=30 tearable=0 states=1073741823 fits_u32=true
DECODE writes=32 tearable=0 states=4294967295 fits_u32=true
DECODE writes=33 tearable=0 states=8589934591 fits_u32=false local_ordinal=4294967301 gpu_equals_cpu=false gpu_equals_cpu_of_local_mod_2^32=true
DECODE writes=64 tearable=0 states=18446744073709551615 fits_u32=false local_ordinal=4294967301 gpu_equals_cpu=false gpu_equals_cpu_of_local_mod_2^32=true
MUTANT gpu_remainder_64_bit writes=33 gpu_equals_cpu=true
```

读法：一段 33 写（或三态写多到 3^m·2^(n−m) > 2^32），段内第 2^32 + 5 个状态 GPU 判的是第 5 个状态，后面 2^32 个状态 GPU 一个都没判、判的都是前面判过的。今天最长的段 30 写（`scripts/crash/crash-amplification.sh:219`：「FULL_EXPAND_UP_TO=30」），没有一条流落在 33–64 写之间：潜在的，第一条长出这种段的流上出现。G9 下提交只等 GPU，CPU 对拍跟上之前这一截是假绿；第 ② 段若照今天留着，它在这里 panic（假红而不是假绿）。

**四句**：① 不分臂。② GPU 内核判的那一刻看不到高 32 位，属「判别子观测不到」；CPU 看得到。③ G10「同一条流里装得进的段照判」与 G5「能走 GPU 的必须走 GPU」合起来那一分句。④ 改法：甲，G10 的界改成「GPU 判器装得下的段」（今天每段 < 2^32 态），判表打包照第 ② 段那样核、不满足就登记成那一段 GPU 判不了（G6 那一条判红）；乙，内核按 lo / hi 两个字算满 64 位余数（模型里 `MUTANT` 那一格就是这个样子）。段级登记另要引擎加「按段不展开、计入跳过」的口子，跳过的数按「段号 + 写数」报、不报十进制态数。只在我的模型上量过、被攻过零轮。

### L1-8　G6 × G9：两边不在同一趟判，先到那一边的恢复字存哪没写（推的）

G9 让 GPU 在提交时判、CPU 之后补；G6 的比对就要后到的那一边读先到那一边每个状态的红项与恢复字。今天库里一个块存「每个状态一个字节：判定向量编号」（`crates/singlefs-checker-tier/src/verdict_store.rs:16`：「块里每个状态一个字节：判定向量编号」），编号表全库共用、跨判法版本累计，满 256 种就拒写（`crates/singlefs-checker-tier/src/verdict_store.rs:608`：「return Err(VerdictStoreError::MoreThan256VerdictVectorsNotDecidedInTheFirstVersion);」）。恢复字进判定向量，种数 = 落到的根 × 两遍结局的组合 × 红项组合；我没数今天各流有多少条不同的根，会不会过 256 是推的。G6 没写存法。四句：① 不分臂；② 看得到；③ G6「与 GPU 逐状态比」；④ 改法：恢复字另开一张按（块键，版本）存的表，每态 16 字节，不进判定向量。被攻过零轮。

## L2　产生结论的方法：代价数的速率口径

模型 `count_model.py` 原样输出（`bash research/prompts/m3-crash-enumeration-merge-r2-opus-model/rerun.sh`）：

```
MODEL sharing=records_assumption unlink_shared=1358954604 distinct_states=3791610606 unlink_item_wall=946s gpu_serial_item_walls_hours=9.8 gpu_judging_only_floor_hours=4.9..6.0 cpu_catch_up_hours=73..175
MODEL sharing=code_today unlink_shared=16777259 distinct_states=5133787951 unlink_item_wall=4680s gpu_serial_item_walls_hours=10.8 gpu_judging_only_floor_hours=6.2..7.2 cpu_catch_up_hours=80..220
MODEL cpu_second_stream_alone_hours=9.6..55.3
MODEL cpu_unlink_mounts_alone_hours=50.8
MODEL r1_reference distinct_states=3774833347 gpu_only_hours=1.5..3.1 cpu_every_state_hours=21.6..124.4
MODEL sharing_lost_if_versions_are_per_item states=1426456802 gpu_extra_hours=1.17 cpu_extra_hours=8.2..47.0
MODEL observer_edit_gpu_rejudge_hours=1.30..2.40 (only the unlink nodes .. unlink plus the shared multi-record nodes)
MODEL_SELFCHECK compared=11 mismatched=0 []
```

### L2-1　GPU 那一截：第一轮拿一项的速率乘全域

第一轮两条 GPU 速率都取自多记录流那一项（`research/prompts/m3-crash-enumeration-merge-r1-opus-model/count_model.py` 的 `gpu-effective` 与 `gpu-balanced`）。各项量过的整项挂钟差得很远：
- 到 E 那一项（全域最大的一项）：`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35`：「到 E 那一项平均每秒约 9.9 万态，低于 2026-09-29 那一趟约 13 万」——多记录流那一项的 0.29–0.38 倍。
- 每项有固定开销：同一行「抬 F 8 223 态 579 秒」、「σ 65 548 态 0 红 607 秒」，态数可忽略的两项各占约 600 秒；G2 合表之后多出六项（第一条流、io_min 8192、清扫两项、随机历史两项），照这个数各 600 秒（推的）。
- 录制：同一行「每条流录制约 480–520 秒」（树分裂、按位置寻址两项「判只占几秒到几分钟」）。
- 存判定占挂钟：`records/2026-10-01-删除与并发事务.md:1292`：「存判定占本机多记录那一趟 1045.6 / 2774.7 秒」——第一轮「含存判定」那条速率含它；两台不均：`records/2026-10-01-删除与并发事务.md:1288`：「第二台 409 862 144 态 1121.2 秒 3 卡 380 989 态/秒」，本机同一项 2774.7 秒，第一轮 `gpu-balanced` 把两台各自的速率相加，等于假设两台同时收工。

按量过的整项挂钟逐项串跑（今天入口逐项串行，`scripts/crash/crash-amplification.sh:226`：「次序就是全量的跑序」）：9.8 小时（照 records 的共享）到 10.8 小时（照今天代码的共享，L2-6）。只算判、零固定开销、两台完全均衡的下界：4.9–6.0 / 6.2–7.2 小时。第一轮报给用户弹窗的「GPU 单判 1.5–15 小时」区间下半截（1.5–3.1）没有量过的数撑；量过的整项挂钟落在 10 小时上下。都是 2026-09-30 与 10-08 那两天的树与内核（之后删除线往内核里加了判法，今天快慢没量）。

四句：① 不分臂（这一格只给数）；② 不适用；③ L2「哪几条速率的口径与 G5、G9 的跑法不一样」；④ 推翻它的现象：今天的树上照「要主 agent 跑的命令」第 1 条量出到 E 那一项高于 34 万态/秒。

### L2-2　CPU 两条速率的差在流与段，不在「层 0 路 / 放量路」

第一轮两条 CPU 速率差 5.8 倍，攻方当成路的差（「合成后 CPU 参照走的是放量那条路…更该取后一个」）。两条量在不同的地方：
- 层 0 那条：`records/2026-09-24-里程碑二收尾调度.md:318`：「第二条流速率（量过）：本机 23,700 个/秒、第二台 24,800 个/秒」，同一行前文是续跑预演「跑一分钟再停」——第二条流开头那几段的短跑。
- 放量那条：`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:55`：「前 9 分钟小段 17.5 k，24 写段掉到约 2 k」，整趟平均 4.1–4.3 k 态/秒/台，到 D 全域、24 写段占多数。
- 放量路自己在小段上就是 17.5 k/台，与层 0 路开头的 2.4 万/台同一量级；第一条流全量（主要是 A 那一段 2^24）层 0 路两台约 5.1 万/秒（`records/2026-09-24-里程碑二收尾调度.md:315`：「第二台约 27,100 个状态/秒，本机约 23,700 个/秒」），那是一个小池。

所以一个状态 CPU 判多久是「这条流、这一段的镜像有多大」的函数（持久的单元多、checker 走得多），不是路的函数；第二条流 28 写段占多数（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:57`：「六个 28 写段」），照到 D 的 24 写段速率更近（推的）。四句：① 不分臂；② 不适用；③ L2「CPU 那一条是层 0 路还是放量路」；④ 推翻它的现象：今天的树上同一条流、同一段，放量路与层 0 路的 CPU 速率差出 3 倍以上（第 2 条命令）。

### L2-6　第一轮与判决用的「删除流共享 13.6 亿态」，今天的代码里不共享

**被用的数**：`records/2026-10-01-删除与并发事务.md:1286`：「没扣掉与多记录流共享、判定库可复用的 1 358 954 604 态」，第一轮攻方模型把它当删除流的共享前缀，K1-2（92.8% 的态丢挂载观察者）与判决第三节给用户的 37.7 亿态都建在它上面。

**代码**：节点身份带设点的文件（`crates/singlefs-checker-tier/src/crash_identity.rs:70`：「"{}::{}::{}",」，第一格是 `code_file_in_repository`），身份进块键（`crates/singlefs-checker-tier/src/crash_amplification.rs:251`：「stream_or_node_name: StreamOrNodeName::new(&self.identity)」）。公共模块设的点才是几条流的同一个节点（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:3`：「崩溃点的文件是这一份（file!()），不是调它的测试文件」），多记录流与删除流从公共模块取的只有取号暖机与 A 两个点（同文件 `record_pool_build`）；B、C 两步各在自己的测试文件里设：`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:434`：「let second = recorder.step(file!(), "sequential_write", || {」，删除流 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1100` 同一句，`file!()` 是两份不同的文件，身份不同、块键不同。

**对得上的观测**：2026-10-08 多记录流那一项 `reused=16777259`（`records/2026-10-01-删除与并发事务.md:1292`：「states=1342177375 reused=16777259」），恰是取号暖机与 A 那一截（第一条流全量 16 777 260 态减最后全持久那一个）；B、C 的 2^28 与 2^30 两段一态都没复用。那一趟删除流 `reused=0` 被归给判法版本中途变了，它本来最多也只能复用这 1 677 万态。

**后果**：去重态数 37.9 亿（照 records）→ 51.3 亿（照今天代码，模型 `distinct_states=5133787951`）；G4「挂在标准前缀上」在今天的代码上只碰到 A 之前那 1 677 万态，删除流 B、C 两段的挂载观察者本来就在删除流自己的节点上；G3 若要把 B、C 挪进公共模块去共享，是一处新改动，要另写进 G3。四句：① 不分臂；② 看得到（身份文本里就有文件路径）；③ L2「第一轮的代价数」——打中的是它的态数一侧，不是速率；④ 推翻它的现象：今天的树上同一份库先跑多记录流、再跑删除流，删除流那一行 `states_reused` 大于 1 677 万（第 3 条命令）。

### L2-3　CPU 那一截跟上要多久（两台、31 线程，推的）

每个状态 CPU 判一张镜像（G6）、删除流整条每个状态再起一次可写挂载（G4；挂载速率只有「每台约 4 千态/秒」一句，`records/2026-10-01-删除与并发事务.md:1286`，出处不明）：73–175 小时（照 records 的共享）、80–220 小时（照今天代码的共享）；其中删除流的挂载一项 50.8 小时、第二条流一项 9.6–55.3 小时。区间两端是 L2-2 那两条速率，今天的树都没量。推翻它的现象：第 2 条命令量出第二条流 28 写段上两台合计高于 4.85 万态/秒。

### L2-4　「提交只等 GPU」时整域上判镜像的不是被改的那份 Rust 代码（提醒，触 L5）

GPU 判器的恢复与 checker 是 WGSL 写的另一份（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:3`：「判法与 CPU 判器（crate::crash::judge_crash_image）逐条相同」），判表只从 core 取几样解析（`crates/singlefs-checker-tier/src/crash_judge_tables.rs:18`：「use singlefs_core::recovery::PoolReader;」那几行），不调 `recover`。判法闭包是一份（L1-1），`recovery.rs` 一改，GPU 那一截整域重判，判表与内核没变、结论逐位与改前相同；改了的 Rust 恢复在整域上只经 CPU 那一截（跟上要 L2-3 那么久）与快档。只改写路径（core 的发布、分配）的提交，写表变、GPU 判的是新镜像，那一类 GPU 那一截真测到。这一格不是打中：用户定了「提交只等 GPU」，主 agent 交用户时可以连这一句交代（G5 把「两遍恢复」归 GPU，而「要真跑 Rust 实现的只能 CPU」那一条原话也罩得到 Rust 恢复——归 L5 判）。

### L2-5　E171 冻结前要多长的冻结窗

G9「里程碑收尾与 E171 冻结之前必须跟上」⇒ 判法版本（G8 的并：判器闭包、各项测试文件闭包、引擎、观察者）连续 73–220 小时不变。`git diff --stat HEAD -- crates/` 末行「215 files changed, 124787 insertions(+), 8284 deletions(-)」（HEAD 是 2026-10-03），根环线与删除线每天往 core 与 GPU 判器里合批；冻结窗里这两条线都要停。这是计数，不是打中。

## L3　没被看过的地方

### L3-1　两套「在哪棵树上跑、版本什么时候算」的核法，G2、G11 没写取哪一种

**层 0 那一套**：`.claude/gate.d/checker-tier-layer0-replay.sh:34`：「--full：在 HEAD + 暂存区的 worktree 里跑（主工作区读得到别的会话没暂存的改动）」；指纹由门禁在测试之外算，`.claude/gate.d/checker-tier-layer0-replay.sh:37`：「开跑与跑完各算一次指纹、相同才写」，跑的过程中输入变了判红不写标记。

**放量那一套**：`scripts/crash/crash-amplification.sh:2`：「每次调都在这一刻的树上跑全量或点名的分项」——主工作区。分项驱动先只编（`scripts/crash/crash-amplification-lib.sh:161`：「cargo_build_arguments=(test --release -p singlefs-checker-tier」，带 `--no-run`），再起 `cargo test`；判法版本由测试二进制起来之后从盘上读源码现算：`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:292`：「let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))」，第 299 行 `PipelineJudgingCode::of_the_judges(&root, …)`。版本算的是此刻盘上的源码，跑的是编译那一刻的源码；入口不比各项的版本是否相同（入口文件头 `grep -n 判法版本` 只命中复用的说明）。

**已经发生过的**：`records/2026-10-01-删除与并发事务.md:1292`：「删除流这一趟复用 0，因为两项之间主树合入了根环线第九块」，同一趟全量两项判在两个判法版本下、入口照报两项全绿。

**造出来的历史（推的）**：CPU 对拍照 G9「日常剪枝跑」在主工作区跑，两台、几天。某一项的 `cargo test` 正在编（release，几分钟），别的会话在这几分钟里合进一批改 `crash.rs` 的补丁：rustc 已读的是旧 `crash.rs`，二进制起来之后读到的是新 `crash.rs`，判法版本取新的。这一项的块按新版本存进库，判它们的是旧代码。之后任何一趟在新树上编、跑，同一版本全部复用——新 `crash.rs` 在这些块上一个状态都没判过，报绿。

**四句**：① 不分臂（G2、G11 都没选一边）。② 测试二进制看不到自己是从哪一份源码编出来的，属「判别子观测不到」。③ L3「两台工具链与输入指纹的核法怎么并」。④ 改法：甲，判法版本在编译时算进二进制（`build.rs` 照同一套闭包算、用 `env!` 读回），跑时再从盘上算一遍，两边不等就整趟不写库、退 78；乙，合并后的入口一律在 HEAD + 暂存区的 worktree 里跑（门禁 `--full` 那一套），CPU 日常那一趟同样在冻结的 worktree 上跑。甲在这一格不中；乙不中，但要多一棵树的磁盘与一次全编。被攻过零轮。

### L3-2　合表之后谁核「每条崩溃枚举用例都登记了」

今天这一样由 `crash-case-check.py` placement 判，认用例的办法是按名字：`scripts/crash/crash-case-check.py:95`：「ENUMERATION_CALL = re.compile(r"\b(enumerate_layer0」，另认 `every_crash` 与循环里造 `CrashImage`；`scripts/crash/crash-case-check.py:33`：「判不了的：经 tests/<子目录>/ 里的共用函数间接调枚举函数的、宏展开出来的调用——认不出」；要求登记的只有标了 ignore 的（同文件第 23 行「③ checker 档包里标了 #[ignore] 的（全量那条）」）。

放量项走 `run_from_environment` / 公共模块的 `amplify_prepared_flow`，不标 ignore、挂特性（例 `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:425`：「#[cfg(feature = "verdict-store")]」）。G3 第二批把层 0 全量用例并掉（删掉直接调 `enumerate_layer0_*` 的那几份），G1 第四批把 `enumerate_layer0_*` 改名：之后 placement 一条合并后的项都认不出。**造出来的历史（推的）**：第五批有人照放量的写法加一条新流的用例、忘了进登记表：placement 认不出它是崩溃枚举用例，登记表里没有它，整域那一趟不跑它，门禁全绿。四句：① 不分臂；② 系统看得到测试源码，只是判据按旧名字认；③ L3「crash-case-check.py 的 placement / modules 两样在合并后判什么」；④ 改法：placement 改成按调用合并后引擎的入口认（`CrashPointRecorder` 建流再交引擎的那一类），登记表反向核：表里每一项的测试函数在、源码里每一条交引擎的测试都在表里；与 G1 改名同一批改。被攻过零轮。

### L3-3　modules 格的模块表、feature-gated-tests 格的「本次未跑」（小）

- `scripts/crash/crash-case-check.py:218`：「TIER_MODULES = ["crash", "layer0_progress",」（同一行后面有 `crash_amplification`），checker 档测试文件首行声明导入的模块，19 份声明了这两个之一（`grep -c 'checker 档模块：.*crash_amplification\|checker 档模块：.*layer0_progress' crates/singlefs-checker-tier/tests/*.rs | awk -F: '$2>0' | wc -l` 输出 `19`）。G1 改模块名，模块表与 19 份首行要同一批改，见 L6-4。
- feature-gated-tests 格：`.claude/gate.d/checker-tier-layer0-replay.sh:9`：「GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑」；G11「配置开了 GPU 而卡用不上 … 退 78」。合并后同一道门禁里，崩溃点重放的项没卡退 78、别的挂 `gpu` 特性的单测（GPU 对拍那几份）没卡记本次未跑——两种语义怎么分、哪一格管哪一种，G2、G11 没写。四句：① 不分臂；② 看得到；③ L3 那一句；④ 改法：G11 写明 78 只管登记表里的项，feature-gated-tests 格照旧管表外的挂特性单测。被攻过零轮。

### L3-4　判法版本里的「工具链」不含 GPU 驱动与卡型（小）

`crates/singlefs-checker-tier/src/layer0_progress.rs:191`：「编这个测试二进制、跑这一片的工具链」，字段只有 rustc 前三行、cargo、target triple（同文件第 195–197 行）。G9 下 GPU 结论是提交那一刻的依据；驱动升级（内核由驱动编，`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:404`：「冷编由驱动做」）或块落在另一种卡上，键不变、旧判定照复用。规则：`.claude/singlefs-ai-sop/rules/test-script-discipline.md@判定可复用：按输入存，按前缀剪`：「每一份判定按它依赖的全部输入算键存下：被测代码、判法代码、参数、工具链。」四句：① 不分臂；② 判的那一刻看得到驱动版本，键里没放；③ 规则「工具链」那一项；④ 改法：GPU 判器版本另拼驱动版本与 wgpu 后端名（不拼卡型：G6 的 CPU 对拍罩卡型差）。被攻过零轮。

### L3-5　旧库、旧全绿标记（没打中）

试过的形状：新形态会不会把旧库里的块当成新判法版本下的判定复用、会不会认旧的 `singlefs-crash-case-green.<用例>.<指纹>` 标记。都不会：G8 换了版本的定义，新版本的摘要与任何旧版本不同，旧行留作历史；标记按用例名加门禁清单指纹记，G1 改名之后新名下一个旧标记都没有。剩下的只是磁盘：库今天 272M（`du -sh ~/.local/share/singlefs/crash-amplification/library` 输出 `272M`），层 0 进度目录与旧标记留在 git common-dir，落地那一批照「旧证据归档」删或留，不影响判定。整份库对账（`unjudged_blocks`）会把旧流录过、再也不会判的块列成没判，但只有测试调它（`grep -rn 'unjudged_blocks(' crates/singlefs-checker-tier/src crates/singlefs-checker-tier/tests` 只命中 `crash_amplification_pipeline_records_checks_and_reuses.rs`），驱动用按流的那一份，没打中。

## L6　落得下来吗

### L6-1　G10 排在不碰 `crates/` 的第一批

`research/prompts/_m3-crash-enumeration-merge-r2-body.md:36`：「一、脚本、门禁、钩子、登记表、条款（G2、G9 的入口与档、G10、G11、G12），不碰 crates/」。G10 要的两样都在引擎里（L1-7）：段级「不展开、计入跳过」的口子（今天只有点级 ignore，`crash_amplification.rs:274`），跳过的态数换一种装得下的表示（今天 u64、整域下先 panic）。第一批落地之后按位置寻址那一项照 G9 整域跑：要么在 322 写那一段 panic，要么照今天按展开上限不展开、`exhaustive=false`、入口判红。四句：① 不分臂；② 不适用；③ L6「每一批的出口与验收标准写得清吗」；④ 改法：G10 挪进第二批，第一批只改登记表的形态（一段一行）。

### L6-2　快档的「甲二」那一半第三批之前上不了 GPU

G9 快档 = 甲二 ∪ 展开 6、「GPU 判镜像」。GPU 判器内核与事实表拆序号只认每次写的两 / 三态，不认段的展开方式：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3505`：「序号 → 持久掩码（与 crash_facts::persisted_writes_of_state 同一套数字次序）」，事实表那一份逐次写取 `crates/singlefs-checker-tier/src/crash_facts.rs:559`：「let choices = if is_tearable[index] { 3 } else { 2 };」；甲二是 CPU 计划里单独的一支（`crates/singlefs-checker-tier/src/crash.rs:2400`：「ordinal_within_the_segment / 2,」，余数决定单元写全落或全不落）。判表照计划的甲二序号区间建（区间短），内核按全展开的进制拆，判到的是另一批状态。模型原样（一段 2 次原地写、3 次单元写，甲二 7 态）：

```
JIA_ER segment_order=[0, 1, 2, 3, 4] in_place=[0,1] copy_on_write=[2,3,4] states=7 states_in_both=4 gpu_only=3 gpu_partial_copy_on_write=3
JIA_ER segment_order=[2, 3, 4, 0, 1] in_place=[0,1] copy_on_write=[2,3,4] states=7 states_in_both=1 gpu_only=6 gpu_partial_copy_on_write=6
```

GPU 拆出的 7 个掩码里 3–6 个是甲二没有的（单元写只落一部分的那一类），甲二要的几个状态 GPU 一次都没判；快档里 CPU 对拍照 G6 逐状态比，这一趟红（假红，不是假绿）。`grep -n 'InPlaceSubsets\|甲二' crates/singlefs-checker-tier/src/crash_judge_tables.rs crates/singlefs-checker-tier/src/crash_amplification.rs` 零命中：引擎与判表今天都没走过甲二。四句：① 不分臂；② GPU 判的那一刻不知道这一段按甲二展开（判表里没有展开方式），属「判别子观测不到」；③ G9「GPU 判镜像」那一分句；④ 改法：判表加一列段的展开方式、两个内核照它拆（第三批，换判法版本）；第三批之前的快档写明甲二那一半只 CPU 判、登记成豁免行。被攻过零轮。

### L6-3　随机历史两项上 GPU，没有批次管 `crash_injection.rs`

G5：「随机历史两项的镜像那一判同样上 GPU（从它们录下的写流建判表）」。两份用例与 `crash_injection.rs` 今天都不建判表（`grep -c 'crash_judge\|JudgeFlow' crates/singlefs-checker-tier/tests/crash_injection_campaign.rs crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs` 两行都是 `0`）；随机历史的写在 `crash_injection.rs` 里留着（第 85 行引 `RetainedWrite`），从它们建判表要改这份 8313 行的库文件。G14 第二批是「`crash_amplification.rs` 引擎口子 … 与只在合并范围里的测试文件」，第三批是「GPU 判器那几份」，`crash_injection.rs` 不在任何一批。四句：① 不分臂；② 不适用；③ L6「哪一批之后快档与整域第一次能整套跑通」；④ 改法：第二批写明含 `crash_injection.rs` 的录写与建判表那一截。

### L6-4　第四批改英文标识符，跨到第一批的文件

G1 退役 `layer0_progress`、计数行前缀 `LAYER0…` 这一族。仓里 `crates/` 之外认这些名字的（`grep -c`、`grep -l` 原样数）：
- `scripts/hosts/capped.sh:25`：「SINGLEFS_LAYER0_THREADS」——线程上限由它设进环境，crates 里读的是 `crates/singlefs-checker-tier/src/crash.rs:1733`：「pub const LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_THREADS";」。第四批只改 crates 里的变量名，`capped.sh` 设的旧名没人读，枚举照默认取全部核数：派发提示里的「线程上限」静默失效（`run-with-memory-cap.sh` 第 1699 行的自证也拿这个名字）。
- `scripts/gate/admission.py`：`grep -c 'LAYER0'` 输出 `115`（计数行前缀、判日志）；`.claude/gate.d/stage-inputs.tsv` 9 行带 `count-line=LAYER0`；门禁样本 `.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/` 下 7 份文件带 `LAYER0`；`.claude/hooks/lib_heavy_tests.py` `grep -c 'layer0\|LAYER0'` 输出 `84`。
- `crash-case-check.py` 的认法正则（第 95 行）与模块表（第 218 行），见 L3-2、L3-3。

G14 第一批「脚本、门禁、钩子、登记表」、第四批「英文标识符改名」，按文件不相交切；这几份在第一批改过一次之后，第四批要再改一次，而且必须与 crates 同一个提交，否则中间那几个提交上门禁认不出计数行、线程上限失效。四句：① 不分臂；② 不适用；③ L6「每一批的出口」；④ 改法：第四批的文件清单写成「crates 里的标识符 + 上面这几份脚本里认它们的地方」，或环境变量名（`SINGLEFS_LAYER0_*`）明写不改、只改 Rust 标识符与计数行前缀。

### L6-5　G9 从哪一批起生效没写

G12（第一批）把 D13 已定项 9 改成「每次提交跑整域」；今天的条款是 `.claude/kb/decisions/13-验证路线.md:251`：「全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快速档全绿（必要条件）。」。若第一批落地起就照 G9：第二批改引擎（G8 让引擎进版本）、第三批改 GPU 判器、第四批改 `crash.rs` 里的标识符，每一批都换判法版本，每一次提交都要一趟整域——GPU 各约 10 小时（L2-1），CPU 各从头补（L2-3）；中间态的整域是今天的两套装置（层 0 `--full` 是 CPU 枚举，按 G9「只等 GPU」它算不算 GPU 那一截也没写）。G14 写「之后整域一趟」，只在 G9 推到第四批之后才生效时成立。四句：① 不分臂；② 不适用；③ L6「每一批的出口与验收标准写得清吗」；④ 改法：G12 写明新条款在第四批落地后的第一次提交起生效，第一至第四批照今天的条款（提交前快档全绿 + 用户要求时跑全量），或照 G9 而接受四趟整域。被攻过零轮。

### L6-6　哪一批之后快档与整域第一次整套跑通

逐样对批次（推的，读码）：

| 要的 | 哪一批给 | 依据 |
|---|---|---|
| 段级不展开、跳过数的表示（G10） | 第二批（G14 写的是第一批） | L6-1 |
| 观察者挂节点、计数随块存、判法版本（G4、G7、G8） | 第二批 | G14 |
| 甲二在 GPU 上拆（G9 快档） | 第三批（G14 没写） | L6-2 |
| 第一条流、io_min 8192、清扫两项的判表建得出（第一轮 K4-3 核不动） | 第三批之后才知道 | 第一轮判决 K4-3 |
| 随机历史两项的判表（G5） | 没有批次 | L6-3 |
| 标识符改名连脚本（G1） | 第四批，要连第一批的文件 | L6-4 |

快档最早在第三批之后能按 G9 整套跑（甲二上 GPU）；整域要 G10（第二批）与新项的判表（第三批），也在第三批之后；随机历史那一截照 G14 字面没有批次，所以「整套」照字面在第四批之后也跑不通。

## 没打中的形状

- G7 计数读回与新判的块合错：合并照计划逐块读（`unjudged_blocks_of_the_flow` 那样按点、按块走），每块只读一次，闭式核得出重算；只有按驱动行逐趟相加才会重复计（`crash_amplification.rs` 第 6676–6682 行写了几趟的行会重复计），G7 没要求按行加。没打中。
- 双机按速度分块在长 CPU 趟上偏：分块是按权重平滑轮转、一块一块交错（`crates/singlefs-checker-tier/src/crash_amplification.rs:904`：「平滑加权轮转：前 k 块里每一方分到的块数与权重成比例，任何时刻偏差不超过一块。」），流里前后速度差 6–12 倍也摊得匀；偏只来自权重本身（2026-10-08 那一趟两台差 2.5 倍收工）。没打中。
- 旧库与旧标记被误复用：L3-5。
- G6 恢复字四个字两边不同义：读码同义（L1-6 前半），没打中。
- 今天的树上有没有 33–64 写的段：最长 30 写，没有（L1-7 是潜在的）。
- 取样范围：没有跑任何一段历史的崩溃状态；两份模型都是纯算术，`closed_form_state_count` 的总量是 0 态，这条腿原型跑的全量合计 0 态。

## 这条腿自己的限度

- 全部是读码与纯算术：没编一行 Rust、没跑任何 checker 档、没上卡。L1-7 的 GPU 拆错、L6-2 的甲二拆错是把内核与 CPU 那几行抄成 Python 比出来的，不是在卡上跑出来的；抄错一行，模型就跟着错（抄的行号写在 `decode_alias.py` 文件头）。
- L2 的速率全是 2026-09-24 至 10-08 记录里的数，那几天的树与内核；之后删除线往内核里加了判法，GPU 今天快了还是慢了没量。新加六项每项 600 秒是拿抬 F 与 σ 两项外推的。挂载速率「每台约 4 千态/秒」出处不明。
- L2-6 的「今天只共享到 A」是按身份的代码与 10-08 那一行 `reused=16777259` 推的；没在今天的树上跑两项看删除流的 `states_reused`（第 3 条命令）。
- L1-1 的两种读法、L1-3 的单跑语义都是我替 G4、G8 补的读法；形态作者的本意若是第三种，这几格要重判。
- 只攻 L1、L2、L3、L6；没判 L4、L5（L2-4 那一句交主 agent 归 L5）。

## 要主 agent 跑的命令

都是 checker 档（重型），子 agent 不跑；要不要跑由主 agent 定。工作目录一律仓根，库目录用新的空目录（不碰常用库）。

1. 今天的树上各项整项挂钟（核 L2-1、L2-6）：`SINGLEFS_HEAVY_TESTS=user-request bash scripts/crash/crash-amplification.sh --library ~/.local/share/singlefs/work/m3-merge-r2-rates/library-full`（多机配置照今天默认开 GPU、两台；内存上限照入口默认 `--memory-cap`、`--peer-memory-cap`）。看逐项 `CRASH_AMPLIFICATION_ITEM item=… states=… reused=… seconds=…` 与末行 `CRASH_AMPLIFICATION_TOTAL`。判法：到 E 那一项 `states / seconds` 高于 34 万 ⇒ L2-1 的「0.29 倍」不成立；删除流那一项 `reused` 大于 16777259 ⇒ L2-6 不成立。按模型推约 10–11 小时。
2. 今天的树上 CPU 判一个状态多快（核 L2-2、L2-3）：`SINGLEFS_MULTI_HOST_CONFIG=<两台、ENABLE_GPU=0 的配置> SINGLEFS_HEAVY_TESTS=user-request bash scripts/crash/crash-amplification.sh --item fixed-script-through-rollback --expand-up-to 24 --library ~/.local/share/singlefs/work/m3-merge-r2-rates/library-cpu`。看各行 `states_per_second=` 与 `checking_seconds=`；按 8 428 态/秒推约 2 小时。要比层 0 路，另起门禁 `--full --item layer0-second-stream` 跑一小时后停，读进度文件片行的态数与挂钟。
3. 只核 L2-6 的便宜做法：同一份新库先后 `… --item multi-record-publish --expand-up-to 24`、`… --item unlink-after-the-multi-record-publish --expand-up-to 24`（GPU 配置），看删除流那几行 `states_reused`：等于 A 之前那一截（展开 24 下约 1 677 万）⇒ B、C 不共享；多出 B、C 小段的态数 ⇒ 共享、L2-6 错。

## 没做什么

- 没跑任何 checker 档测试、门禁阶段、崩溃放量、GPU；没编译；没建仓副本与编译目录（交回时没有要删的）。草稿目录 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r2-opus/model/` 留着两份模型的工作拷贝（与模型目录里的逐字节相同，`sha256sum` 见开头），不是仓副本。
- 没判辩方那几格（L4、L5），没替主 agent 采纳；模型上的数不算入库装置上的数。
- 没数今天各流不同的根有多少（L1-8 的 256 上限够不够是推的）。
- 第 56 批（GPU 判器那几份）在我读的时候还没合入；L1-4、L1-6、L1-7、L6-2 引的内核与判表那几份都在开工快照的清单里，收工时逐份没变（下一节），第 56 批合入之后行号要重查。

## 收工核对

- `sha256sum -c --quiet research/prompts/m3-crash-enumeration-merge-r2-snapshot/crates-sha256.txt`：退 0（91 份全对）。
- `sha256sum -c --quiet research/prompts/m3-crash-enumeration-merge-r2-snapshot/kb-sha256.txt`：退 1，原样「.claude/kb/checks-owed.md: FAILED」「sha256sum: WARNING: 1 computed checksum did NOT match」——开工时对、收工时变了，是别的会话在这期间改的；这份报告没有引 `checks-owed.md` 的任何一行。其余 4 份 kb 对。
