# m2-agentdef-r1 云端攻方（Opus）报告：A1 清理改没改指令、A2 门禁 71 号的漏判与误判

写于 2026-09-18 05:20 UTC（14:20 JST）。攻击面按正文第四节分工表：A1、A2；A2 里附录三特征表那一半归本地攻方，这里不做。
开工快照 `research/prompts/m2-agentdef-r1-start-snapshot.sha256` 里的 18 份定义与 71 号脚本，探针开头逐个核了 sha256：19 个文件，0 处不符（`probe_gate71.out` 第 1 行）。

## 复跑命令与模型文件

```bash
cd /home/fy5090/code/singlefs
# 门禁 71 号：9 格误判、26 格漏判、改法 R1 与 R4 的量；只写 <工作目录>，仓里一个字不改。输出应与 probe_gate71.out 逐字节相同
nice -n 19 python3 research/prompts/m2-agentdef-r1-opus-model/probe_gate71.py . <空的工作目录>
# 改法 R2（② 不看反引号里的日期）；输出应与 probe_r2.out 逐字节相同
nice -n 19 python3 research/prompts/m2-agentdef-r1-opus-model/probe_r2.py . <空的工作目录>
# implementation-writer 第 3 步删掉的前提：三行的玩具 crate，要 cargo；输出应与 mtime_toy.out 逐字节相同
bash research/prompts/m2-agentdef-r1-opus-model/mtime_toy.sh <工作目录>
# 规则、SOP、kb 里哪些真实小节名套进「开工先读：`文件`「小节」」会被 ④ 判红；输出应与 heading_probe.out 逐字节相同
nice -n 19 python3 research/prompts/m2-agentdef-r1-opus-model/heading_probe.py .
```

两份探针各跑了两次（一次进草稿目录、一次落盘），`cmp` 判逐字节一致；`mtime_toy.sh` 与 `heading_probe.py` 同样各跑了两次、逐字节一致；玩具 crate 此前还在草稿目录里手敲过一次同样步骤，判定相同。

| 文件 | sha256 |
|---|---|
| `research/prompts/m2-agentdef-r1-opus-model/probe_gate71.py` | `f58298c1f148e7b376a75c90fe7ea8964c73a563447e99c877975b1cb67258d9` |
| `research/prompts/m2-agentdef-r1-opus-model/probe_gate71.out` | `b8c4583bfd881222c75b86c2c0a7a40a2956a029cb3bd5d1add6fc26898b079a` |
| `research/prompts/m2-agentdef-r1-opus-model/probe_r2.py` | `e0ebdbb705ab68d23f616a1bde1aa8c0be554dd56d1e6ef5a22fc0334766a7ec` |
| `research/prompts/m2-agentdef-r1-opus-model/probe_r2.out` | `2429723dc92d2d54de7a5befc52fc8f089586b083afeaf2aaae0e7ce09f3633e` |
| `research/prompts/m2-agentdef-r1-opus-model/mtime_toy.sh` | `e00f5535a144c15552f9036a406ffc79684a7ba21922f90e1284125784cc5131` |
| `research/prompts/m2-agentdef-r1-opus-model/mtime_toy.out` | `922d94dc2109103e84366d05112b7e39e8bc2ffb406d5e227ccc29e2e5768e26` |
| `research/prompts/m2-agentdef-r1-opus-model/heading_probe.py` | `7b800acad9ae284b5dd4b8d51e6a8fd2c2c7f0797cd44976e52d279a1636c3d3` |
| `research/prompts/m2-agentdef-r1-opus-model/heading_probe.out` | `e0024068e85c2cf3963afae92420e77b54210495f492d3c3e96335ae78e95c38` |

这份报告里的数全部出自副本（工作目录里拷的 `.claude/agents/*.md` 与 71 号脚本）或玩具 crate，属「腿在副本上量出的数」，不算入库装置上的数。

## 各格判定一览

| 格 | 判定 | 强弱 | 量过 / 推的 |
|---|---|---|---|
| A1-H1 `implementation-writer` 第 3 步删掉「`rsync -a` 保留旧修改时间，cargo 跑上一份二进制」 | 打中 | 中 | 机制量过（玩具 crate：内容与原件相同的副本照样判 FAILED）；实现员会不会走「同一份副本用 `rsync -a` 还原」这条路是推的 |
| A1-H2 `three-way-verifier` 输入删掉「腿交回之后主 agent 往往已经改了主树，对主树核会报假 ✗」 | 打中 | 弱 | 推的 |
| A1 `gate-triage` 第 4 步（清理执行员点名的一处） | 指令的条件改了，但没改坏：没有一格是「旧定义对、新定义错」 | — | 推的（按 gate SKILL 写的两种成因逐格比） |
| A1 清理执行员点名要审的其余 6 处 | 没打中 | — | 逐处读新旧原文与各自「开工先读」点名的小节 |
| A1「依据：」→「开工先读：」16 行 | 没打中：14 行冒号后一字不差，2 行见 A1 第四节 | — | 量过（逐行比，命令与输出在正文） |
| A1 处置表其余各行 | 没打中 | — | 改动差里的每一块都对得上处置表的某一行，没有表外改动 |
| A2 误判：9 行造出来的纯指令 | 9/9 在预期那条判据上判红 | FP1、FP2 强：正是 71 号自己给的改法形态 | 量过（副本） |
| A2 漏判：今天文件里 71 号判绿的说明句 | 27 处（M01–M27），其中 25 处此前没人点名（M24、M25 已在 71 号文件头第 27 行或拆分提案第二十八节「没做的」里点名） | M01–M04 强：记的是实测数、经过与现状，属共用约束第 9 行点名不许写的那几类 | M01–M26 量过（探针）；M27 在表格行里，按 71 号文件头第 25 行的设计判绿，读脚本得出 |
| A2 跑前反向接受条款 | 只给了漏判的改法，误判那几格没有改法可选 | — | 读条款 |

「打中」只按这一轮跑前写死的触发观测判；四句（分不分辨、看不看得到、满足哪个分句、改法中不中）在各格正文里逐条答。

## A1 清理改没改指令

### 一、H1：`implementation-writer` 第 3 步——删掉的是一条比留下的指令宽的前提（打中，中）

清理前 `research/prompts/m2-agent-def-cleanup/before/agents/implementation-writer.md:27` 整行：

> 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录：`rsync -a` 保留源码的旧修改时间，共用 target 时 cargo 会直接跑上一份副本编出的二进制，2026-09-17 实测源码与原件相同的副本报 8 过 2 红），先跑一份不改动的副本：已经红的测试记成基线红集（多半是别的会话在制的改动），变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。

今天 `.claude/agents/implementation-writer.md:27` 整行：

> 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录），先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。

删掉的那句讲的是机制：`rsync -a` 带回旧修改时间，cargo 按修改时间判新旧，于是跑的是上一次编出来的二进制。留下的指令只管这个机制的一个后果（几份副本共用 target）。同一个机制还管另一条路：**同一份副本、它自己的 target，改坏之后用 `rsync -a` 从原件还原**。今天的定义里没有任何一句挡这条路；第 3 步自己点名的拷贝命令就是 `rsync -a`，重用一份已经编过的副本又比每条变异另起一份省一整次编译，这条路是顺手的走法。

玩具 crate 上的原样输出（`research/prompts/m2-agentdef-r1-opus-model/mtime_toy.out`，cargo 1.98.0）：

```text
path A：同一份副本、自己的 target，改坏后用 rsync -a 从原件还原
  A1 基线（未改）            → pass
  A2 改坏一处（sed -i）       → FAILED
  A3 rsync -a 还原：内容与原件 same，mtime 2026-09-01 00:00:00 → FAILED
path B：同一份副本、自己的 target，改坏后原地改回（Edit 的效果：新修改时间）
  B1 基线（未改）            → pass
  B2 改坏一处（sed -i）       → FAILED
  B3 原地改回                 → pass
path C：改坏后不还原，另起一份新副本（自己的 target）
  C1 新副本                   → pass
cargo: cargo 1.98.0 (797e8a9bc 2026-08-05)
```

A3 就是旧定义那句「源码与原件相同的副本报 … 红」的症状，只是换了一条路走到。照今天的定义走到 A3 之后会发生的事（推的，没在实现员身上试）：

1. 还原之后接着改坏另一个 crate 里的一处去证下一条测试：前一个 crate 仍是改坏的二进制，下一条测试可能被上一处改坏证成「会红」，「同时红了哪些测试」一栏也跟着错。
2. 还原之后先跑一遍确认干净（顺手的卫生步骤）：看到那条测试红，今天第 3 步里对得上的只有「已经红的测试记成基线红集」与「你要证明的那条在基线红集里就停下交回」——照它办就交回一个不存在的基线红。
旧定义下同样两步，实现员手里有「`rsync -a` 带回旧修改时间、cargo 跑旧二进制」这条机制、还有「源码与原件相同的副本报红」这个症状，才对得上号；新定义里两样都没有。

**四句**：
- 分不分辨：新旧两版在「同一份副本用 `rsync -a` 还原」这条路上，字面指令一样（两版都没明写禁止），差在旧版给了机制与症状、新版没有。所以这一格的分辨靠的是前提本身，这正是 A1 判据问的那一类（「删掉的「解释」里是否夹着条件、例外、判据或前提（执行者据以分支的东西）」）。
- 看不看得到：看得到。实现员能 `stat` 修改时间，`cargo test -v` 会打「Fresh」还是「Compiling」（在 A3 那份副本上现跑：`Fresh mtime_toy v0.1.0`，随后 `test value_is_one ... FAILED`），「内容与原件相同的副本红了」本身就是可见的症状。
- 满足哪个分句（判据与条款出自 `research/prompts/_m2-agentdef-r1-background.md` 第 40、45 行）：A1 判据「删掉的「解释」里是否夹着……前提」；触发观测「同一个输入下照新定义做出的动作与照旧定义不同，而照旧定义做才对」——动作不同这一半是推的（上面第 1、2 条），机制那一半量过。
- 改法中不中：跑前条款的改法是「把那条条件或前提写回定义（写成指令，不写经过）」。我提的写法（只在我的玩具上量过、被攻过零轮）：「副本里改坏的那一处用 Edit 改回，或者另起一份新副本；不从原件 `rsync -a`、`cp -a`、`cp -p` 拷回同一份副本。还原之后先跑一遍，那条测试回到基线的判定再往下。」玩具上 B3（原地改回）与 C1（新副本）都 pass，量过；「还原后先跑一遍」在 A3 那一格会看到 FAILED，量过；它能不能挡住实现员真走 A 路径，推的。

**什么会推翻它**：cargo 改成按内容哈希判新旧（那时 A3 会 pass）；或实现员的做法被别处写死成「每条变异另起一份副本」（那时这条路走不到）——我在 18 份文件与共用约束里没找到这样的句子。

### 二、H2：`three-way-verifier` 输入——删掉的前提管所有轮，留下的指令只管代码轮（打中，弱）

清理前 `research/prompts/m2-agent-def-cleanup/before/agents/three-way-verifier.md:21` 整行：

> - 腿开工那一刻的快照路径（`crates/` 与 `.claude/kb/` 至少这两样；代码轮必给）。腿交回之后主 agent 往往已经改了主树，行号对主树核会报一批假 ✗——2026-09-17 步 6 checker 那一轮，一格「24 条」判 ✗、还建议查材料，实为快照晚于主 agent 的改正；之后两轮给了快照，没有再出假 ✗。没给快照的代码轮，停下要，不对主树核。

今天 `.claude/agents/three-way-verifier.md:21` 整行：

> - 腿开工那一刻的快照路径（`crates/` 与 `.claude/kb/` 至少这两样；代码轮必给）。没给快照的代码轮，停下要，不对主树核。

两版第 31 行（第 6 步）一字不差，都有「分不清：文件在腿交回之后被改过」这一类，但只写了一个触发（「报告文件现在的 sha256 与交回里给的对不上，整份记后一种」），没写被引的 kb、规则文件在腿交回后被改了怎么认。开工快照的规则 `.claude/rules/implementation-workflow.md` 第 24 行起那一节只要求代码轮记快照，所以设计轮派核查员时可以不给快照。

输入（推的，没派核查员试）：一轮设计轮，没给快照；一条腿引 `.claude/kb/某份.md:120` 那一行，腿读的时候是对的；腿交回之后、核查员开工之前，主 agent 在第 100 行插了一行（开工快照那一节第 28 行写的正是这件事在代码轮里发生过）。
- 今天的核查员：第 2 步对主树取第 120 行，对不上；再找原文实际在第 121 行；不是背景材料的行号；记 ✗、写实际位置。没有一句让它先查被引文件交回后改没改。
- 旧定义下的核查员：输入那一条告诉它「主 agent 往往已经改了主树，行号对主树核会报一批假 ✗」，有据去查被引文件的修改时间，把这一格记进第 6 步的「分不清」。

**四句**：分不分辨——字面指令两版相同（都只对代码轮说「停下要」），差在前提的覆盖面，与 H1 同类；看不看得到——看得到（`stat -c %y` 比腿报告的修改时间，`git log -1 -- 文件`）；满足哪个分句——A1 判据的「前提」一项，触发观测里「动作不同」那一半是推的；改法中不中——写回成指令（推的，没实现）：「没给快照、对主树核的：对不上的那一处先查被引文件在腿报告落盘之后改没改，改过的记「分不清：文件在腿交回之后被改过」，不记 ✗。」
**为什么判弱**：第 2 步的括注「（输入给了快照就一律对快照核，别对主树）」还在，细心的核查员读得出「主树不可靠」；它只是不再有「往往」「假 ✗」这两个信号。
**什么会推翻它**：规则改成每一轮都记快照；或第 6 步补上对被引文件的触发。

### 三、`gate-triage` 第 4 步：条件改了，没改坏

清理前 `research/prompts/m2-agent-def-cleanup/before/agents/gate-triage.md:27` 整行：

> 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。跑全量工作区时「工作区跑的过程中没变」那一条在别的会话同时改仓时多半红（2026-09-17 试跑整轮 2 小时 13 分，工作区从 128 项变到 224 项）：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。

今天 `.claude/agents/gate-triage.md:27` 整行：

> 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。

旧版的动作挂在「全量 + 那一条红」上，「别的会话同时改仓」是预测；新版把它变成了前提。这是改了指令，不只是删了说明。逐格比（gate-triage 开工先读的 `.claude/singlefs-ai-sop/skills/gate/SKILL.md` 第 33 行给了这一条红的两种成因：「自己还在改，或者别的会话在改」；那一条红时只打两个指纹、不点名文件，`.claude/singlefs-ai-sop/scripts/gate.sh` 第 369–388 行）：

| 格 | 旧定义 | 新定义 | 对的是 |
|---|---|---|---|
| 全量、红、别的会话确实在改、分诊看得到 | 不是这一轮 | 不是这一轮 | 两版一样 |
| 全量、红、没有别的会话，是主 agent 自己边跑边改 | 不是这一轮（错） | 前提不成立，落回第 3 步，那一条不点名文件 ⇒「分不清」 | 新版 |
| 全量、红、别的会话跑到一半停了，分诊开工后看不出 | 不是这一轮（对，但没有依据） | 前提认不出 ⇒「分不清」 | 新版诚实、旧版碰巧对 |
| 全量、红、自己与别的会话都在改 | 不是这一轮（一半错） | 不是这一轮（一半错） | 两版同错，不分辨 |

没有一格是「照旧定义做才对、照新定义做错」，按触发观测不算打中。交主 agent 的只有一句：处置表第 31 行把这一处记成「换成」一句条件句，它实际改了判归属的条件，判决里值得写明。

### 四、其余各处：没打中

- **清理执行员点名的其余 6 处**（背景材料第 33 行）：
  - `three-way-local-attack` 第 1 步「（本地模型读不到附录）」、第 5 步「闸抓不到（…别的形态照样可能漏）」：它开工先读的 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节里有「本地腿读不到中文附录」（第 281 行）与「闸判绿之后照样通读，闸只认得登记过的形态」（第 291 行），前提没丢。我试了一个既不缺头、也不是尾部复读、两截都在词表里的粘连词：草稿里一句含 `commitroot` 的英文，`python3 research/scripts/oov-check.py` 原样输出「绿 …  生词=1 拼接=0」「生词: commitroot」，闸放过、进生词表；新第 5 步「拿不准是新造词还是粘连的，一律记带损坏」与那一节第 291 行照样把它判带损坏。
  - `implementation-writer` 第 3 步「（多半是别的会话在制的改动）」：实现员在主工作区改，副本拷的是含它自己改动的主树，基线红也可能是它自己弄红的；删掉这句让判断变中性，不是变坏。「要证明的那条在基线红集里就停下」两版相同。
  - `kb-scribe` 第 4 步 doc-lint 那段：「另跑一次 doc-lint 贴末行；红在这一轮写的句子上，停下交主 agent 改规格」两版相同，删掉的是「为什么另跑」与常见红法。
  - `crash-verifier` 第 5 步「（`env.sh` 不查这三样）」与 herd7 不在 PATH 的说明：我按「57 号红 + 验证员自己的 shell 里 `command -v herd7` 退出码 1」这一格读了脚本（没跑 57 号）。`.claude/scripts/lkmm.sh` 第 283 行找不到 herd7 时先加载 `opam env`，第 288 行真缺时自己打「herd7 缺失」，第 320 行找到时先打 herd7 版本的 ✓ 行；「以阶段自己的报错为准」「这三条命令不单独下判断」两句都在，新定义下照样不会把退出码 1 当成缺 herd7。
  - `crash-verifier` 第 6 步「别的会话常在同时改 `crates/`：」：留下的指纹指令本来就不带条件。
  - `experiment-runner` 第 3 步两句诊断合成一句：新句「报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错」把旧句的两个成因都变成了先查的项，删掉的「脚本的目录检查照样放过」在「在 `research/` 下跑」这条指令下不影响动作。
- **「依据：」→「开工先读：」16 行**：逐行把标签后面的部分与清理前比（命令在草稿里现跑）：14 行一字不差；`three-way-defense` 删了括注「（辩方腿推翻命中靠的是去查同类先例，不是重新论证利弊）」，做法在它第 26 行（「2. 找得到同类先例的，贴出处；找不到就写找不到，不用重新论证利弊顶替。」），它点名的那一节里也有；`three-way-local-defense` 从「同 … 的「依据」一行」改成「… 里「开工先读：」那一行点名的小节」，指向同一行。点名的文件与小节一个没少。顺带：`experiment-designer` 第 14 行点名的「交岔路时写岔路单，派实验时带上它」在 `.claude/rules/three-way-inference.md` 第 224 行是粗体段首、不是标题，照共用约束第 21 行「先 `grep -n '^#' 文件` 找到小节」找不到它；清理前原样，不是这一轮的改动。
- **处置表其余各行**：改动差（`diff -r research/prompts/m2-agent-def-cleanup/before/agents .claude/agents` 188 行、共用约束 36 行、主 agent 入口 10 行）里每一块我都对上了处置表的某一行，没有表外改动（两份 frontmatter 是搬迁、不在清理之列）。删掉的都是日期、经过、实测数与只说「为什么」的尾巴；逐条看它删掉之后同一个输入下动作变不变，除上面三节外没找到会变的。

## A2 门禁 71 号：误判（纯指令判红）

探针在工作目录里拷一份今天的 18 份文件与 71 号脚本，每格只改一处、把一句纯指令写进去，跑原版 71 号。基线（不改）：

```text
baseline	files=18	exit=0	✓ 定义只写怎么做（18 份文件 824 行；4 行带日期：2 行的日期只在「」引的小节名里，2 行同行指路；记录小节 0、解释性段落 0、解释性半句 0）
```

九格，改的那一处与 71 号的原样判定（`probe_gate71.out` 第 5–13 行；行号是副本里的行号）：

| 格 | 改成什么（新串原样） | 这是什么指令 | 71 号判定 |
|---|---|---|---|
| FP1 | `three-way-local-attack.md` 开工先读一行末尾加「；`.claude/kb/tooling.md`「提示里的 `**粗体**:` 会诱发损坏（2026-08-29 实测 3/3 对 3/3）」」 | 读取指令，点名 `.claude/kb/tooling.md` 第 198 行那个真实存在的小节 | 红，④ 1 处：`.claude/agents/three-way-local-attack.md:14（「实测」）` |
| FP2 | `gate-triage.md` 开工先读一行末尾加「；`.claude/kb/tooling.md`「并发：两个会话在同一个仓里同时工作过（2026-08-30 实测）」」 | 同上，`tooling.md` 第 561 行的小节 | 红，④ 1 处：`.claude/agents/gate-triage.md:14（「实测」）` |
| FP3 | `kb-scribe.md` 第 5 步后加小节「### 决策变更史的条目」与一条「- 同一天多条的「（其N）」照当月文件现取，其余逐字照规格。」 | 书记员的本职就是写变更史，给它单开一节 | 红，① 1 处：`.claude/agents/kb-scribe.md:32：### 决策变更史的条目` |
| FP4 | `prior-art.md` 第 25 行「出处、日期、实测还是读文档、口径」改成「出处，日期，实测还是读文档，口径」 | 交付字段清单，只把顿号换成逗号 | 红，④ 1 处（「实测」） |
| FP5 | `kb-scribe.md` 第 26 行「；再 `--dry-run` 核全部恰好命中一次」改成「；试跑 `--dry-run`，核全部恰好命中一次」 | 「试跑」当动词 | 红，④ 1 处（「试跑」） |
| FP6 | `kb-scribe.md` 输入第一项拆成子列表，第二个子项「- 依据：判决或用户定案的出处。」 | 输入项，书记员要的是每条改动的出处 | 红，③ 1 处：`kb-scribe.md:20（以「依据」起头，1 行）` |
| FP7 | `agent-common.md` 第 25 行末尾加「（写成 `2026-09-18 14:08 JST` 这种形式）」 | 时刻格式的例子 | 红，② 1 行 |
| FP8 | `three-way-materials.md` 第 30 行「每一行与理由」改成「每一行（理由：照清单原样抄）」 | 交付物的一栏 | 红，④ 1 处（「理由」） |
| FP9 | `mutation-triage.md` 第 36 行「（变异名 / 类别 / 依据）」改成「（变异名，类别，依据：分类引的 `mutation-sampling.md` 哪一类）」 | 交付表的列名 | 红，④ 1 处（「依据」） |

汇总行原样：`summary	false_positive_red_on_expected=9/9	R1_turns_green=2/9	miss_cases_green=26/26`。

**最硬的是 FP1、FP2**：共用约束 `.claude/agent-common.md` 第 9 行要的就是这种写法，71 号自己红了以后打印的改法也是这种写法（`.claude/gate.d/71-agent-def-flow-only.sh` 第 163 行）：

> 这份与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进 `records/2026-09-16-subagent拆分提案.md` 或 `.claude/kb/`，这里不写、也不留解释的尾巴；要用到那些内容时写成一条读取指令（「开工先读：`文件`「小节」」）。门禁 71 号判这一条（规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。

>     print("               要读的规则或 kb 改成一条读取指令「照 `文件`「小节」办」。")

小节名由被读的文件定，写定义的人改不了。`.claude/rules/`、SOP 的规则与技能、`.claude/kb/` 共 224 份文件的 3365 个标题里，有 6 个套进这种读取指令会被 ④ 判红（`research/prompts/m2-agentdef-r1-opus-model/heading_probe.py`，输出 `heading_probe.out`：`.claude/kb/decisions/20-承重面单元的原子性与自包含.md:27`、`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:1`、`.claude/kb/experiments-history.md:1543`、`.claude/kb/tooling.md:198`、`:235`、`:561`）。病根在脚本里两条判据对「」的处理不一样：② 先剥掉「」再找日期（第 109 行 `            if not DATE.search(without_quoted(line)):`），④ 直接在整行上找（第 120 行 `            for match in HALF_SENTENCE.finditer(line):`）；文件头第 12 行只给 ② 写了「」里的「不认」，④ 那几行（第 19–23 行）没有。绿样本 `.claude/gate.d/fixtures/71-agent-def-flow-only.sh/green/setup.sh` 第 17、27 行引的两个小节名都不含 ④ 的触发词，测不到这一格。

**四句**：
- 分不分辨：FP1、FP2 只在 ④ 不剥「」时红；改法 R1（④ 也先剥「」）下两格都绿，其余七格照红——R1 只分辨这两格。
- 看不看得到：FP1、FP2 看得到（「」是字面记号）。FP3–FP9 看不到：同一个词出现在同一个位置（标点后的「实测」「试跑」「依据：」「理由：」、标题里的「变更史」、反引号里的日期），当说明用与当指令用在字面上逐字相同，71 号只看字面，任何词法改法都只能在「漏」与「误」之间挪。这几格属证据纪律「判别子观测不到」那一种。
- 满足哪个分句：A2 触发观测第二半「造一行纯指令、71 号判红」（背景材料第 41 行）。
- 改法中不中：跑前反向接受条款（第 45 行）只写了「漏判的那一行改掉，71 号能补判据就补、补不了写进它的「判不到的」」，对误判没有改法。照字面办，FP1–FP9 只能去改那一行纯指令迁就门禁，方向是反的；这是「改法碰不到打中的格」。建议判决给误判另写一条出路：能补的补不认（R1），补不了的在 71 号文件头与「判不到的」并列写一句「会误判的」，给出被误判时怎么办（例如在那一行里换同义词，或登记豁免）。

改法各修哪一格（都是我自己提的，只在我的模型上量过，被攻过零轮）：

| 改法 | 修的格 | 不修的格 | 对原有判别力的影响 | 量过 / 推的 |
|---|---|---|---|---|
| R1：④ 先剥「」（第 120 行 `finditer(line)` → `finditer(without_quoted(line))`） | FP1、FP2（`R1_exit=0`） | FP3–FP9 | 红样本、绿样本照过；清理前原件上 ①0 ②35 ③3 ④30，与原版逐项相同 | 量过（`probe_gate71.out` 第 5–6、14–19 行） |
| R2：② 不看反引号里的日期 | FP7（`②=0`） | 其余 | 红样本照过、清理前原件 ②35 不变；绿样本的汇总片段变了（「3 行带日期」少一行，那一行的日期在反引号里的 `records/…` 文件名里），`expect` 要跟着改；反引号包住的日期后面跟一段记录会从 ② 漏掉 | 前三项量过（`probe_r2.out`），最后一项推的 |
| 文件头加「会误判的」一节、条款加误判的出路 | FP3–FP9 | — | 不改判法 | 推的 |

## A2 门禁 71 号：漏判（今天 71 号判绿的说明句）

取样：18 份文件 806 行我整份读了一遍，另用一个词表扫了一遍 71 号词表之外的解释标记（草稿里的辅助脚本，37 处命中，逐处人判）。下面是找到的，不是普查。「是说明」是我按规则原文（`.claude/rules/implementation-workflow.md` 第 18 行「只写怎么做——步骤、约束、判据、交付」、共用约束第 9 行点名的「为什么」「实测」「经过」与带日期的记录）判的，人判。

探针对 M01–M26 逐格核了三件事（`probe_gate71.out` 第 20–45 行，每行末尾是那一行的整行原文）：片段真在所写的那一行（`present=True`，26/26）；基线输出里没有这一行的位置，即 71 号判它绿（`flagged_by_71=False`，26/26）；清理前原件里有同一片段（`same_text_before_cleanup=True`，26/26）——**清理与 71 号都没动过它们**。

### 最硬的四处：写的是实测数、经过与现状

`.claude/agents/crash-verifier.md:24`（M01，「实测」前面是「负载下」不是标点，④ 认不出）：

> 2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`，后台起它的那条命令自己的 `$?` 只说明起没起来），结束后读输出（54 号在本机负载下实测约 47 分钟，55、57、59 号各在一分钟内）。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。

`.claude/agents/kb-scribe.md:26`（M02 括注里的「agent-defs-r2 攻方模型：1/3 对 0/3」是上一轮量出的数，用轮名代替了日期，② 认不出；同一行 M18「用它不用逐条 Edit：……会留下半套」是冒号引出的为什么）：

> 1. 开工时先记下规格点名的文件、当月变更史文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | bash .claude/hooks/write-guard.sh`），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它不用逐条 Edit：两阶段写在有一处不中时一个文件都不写，逐条 Edit 中途被打断或锚点被别人改掉会留下半套（agent-defs-r2 攻方模型：1/3 对 0/3）。

`.claude/agents/mutation-triage.md:25`（M03「（今天 4 张）」是用「今天」代替日期的现状数，今天数出来仍是 4 张，加第 5 张就成假话；M04「共用的会让……，计划第十八节」是为什么加一条不带文件路径的经过指路；M17「在仓根下跑会被报成「基线就是红的」」是前提）：

> 3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；在仓根下跑会被报成「基线就是红的」）；表头写着「被测装置是 shell 探针」的（今天 4 张），照表头写的复跑方式逐条跑，判据用表头那句；crates 那张表直接跑 `GATE_MUTATION_TARGET_DIR=<草稿目录>/target nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh`（它自己拷副本、整张表逐条跑，从输出里取主 agent 点名的条目；target 放你的草稿目录，不用它默认那个跨轮共用的：共用的会让开头几行链接上一轮最后一条变异，计划第十八节）。

现状核对：`grep -l '被测装置是 shell 探针' research/mutations/*.tsv | wc -l` → `4`。

### 其余各处（整行见 `probe_gate71.out` 第 20–45 行；M27 不在探针里）

| 格 | 位置 | 那一行里的片段 | 形态 |
|---|---|---|---|
| M05 | `.claude/agents/experiment-runner.md:26` | 「，免得读数被抢」 | 目的状语 |
| M06 | `.claude/agents/experiment-designer.md:28` | 「，免得命中行把结论带进来」 | 目的状语 |
| M07 | `.claude/agents/experiment-designer.md:29` | 「，防等价变异」 | 目的状语 |
| M08 | `.claude/agents/crash-verifier.md:13` | 「派你是为了在提交前的整轮门禁之前先拿到它们的读数」 | 目的状语 |
| M09 | `.claude/agents/crash-verifier.md:23` | 「（两份层 0 全量同时跑各要一个多钟头）」 | 括注里的为什么 |
| M10 | `.claude/agents/gate-triage.md:24` | 「（两道门禁同时跑，重阶段互相拖、工作区指纹互相干扰）」 | 括注里的为什么 |
| M11 | `.claude/agents/experiment-runner.md:27` | 「（后面的门禁阶段都不查命名）」 | 括注里的为什么 |
| M12 | `.claude/agents/experiment-runner.md:30` | 「（`replay.sh` 按这个相对路径找）」 | 括注里的为什么 |
| M13 | `.claude/agents/kb-scribe.md:28` | 「（变异表锚点里的分项标签不跟着改，会腐化）」 | 括注里的为什么 |
| M14 | `.claude/agents/kb-scribe.md:28` | 「（入库产物里印着旧标签的，复跑会对不上）」 | 括注里的为什么 |
| M15 | `.claude/agents/kb-scribe.md:27` | 「（`--write` 会先往那些条目里写「（待补）」）」 | 括注里的为什么 |
| M16 | `.claude/agents/implementation-writer.md:46` | 「（56 号门禁要的判决文件由主 agent 的那一轮产出）」 | 括注里的为什么 |
| M17 | `.claude/agents/mutation-triage.md:25` | 「在仓根下跑会被报成「基线就是红的」」 | 括注里的前提 |
| M18 | `.claude/agents/kb-scribe.md:26` | 「用它不用逐条 Edit：两阶段写在有一处不中时一个文件都不写，…会留下半套」 | 冒号引出的为什么 |
| M19 | `.claude/agent-common.md:64` | 「：`/tmp` 下的草稿目录会话一重启就没了，那一跑等于白干」 | 冒号引出的为什么 |
| M20 | `.claude/agent-common.md:43` | 「，那类读数被抢了 CPU 就不作数」 | 逗号引出的为什么 |
| M21 | `.claude/main-agent.md:26` | 「：它不读共用约束」 | 冒号引出的为什么 |
| M22 | `.claude/agents/kb-scribe.md:19` | 「，所以也由主 agent 给」 | 「所以」 |
| M23 | `.claude/agents/experiment-designer.md:34` | 「`experiment-runner` 照这些节名找东西，以后门禁也按它们查。」 | 独立一句为什么 |
| M24 | `.claude/main-agent.md:20` | 「发散是探索该有的样子」 | 格言（已点名） |
| M25 | `.claude/agent-common.md:43` | 「这个仓里别的会话几乎一直在跑 cargo，见到就停等于开不了工」 | 格言（已点名） |
| M26 | `.claude/main-agent.md:19` | 「延迟不是丢掉」 | 格言 |
| M27 | `.claude/agents/experiment-designer.md:39`（表格行） | 「（它只会整份写出口文件，出口写成登记本身会盖掉占号时写的文件头，而且退出码照样是 0）」 | 表格行里的为什么；71 号文件头第 25 行「表格行（以 \| 起头）不判 ③ ④」，第 93、119 行按设计跳过 |

### 四句与改法

- 分不分辨：跑前条款给漏判两条路——「71 号能补判据就补」与「补不了写进它的「判不到的」」。有词法特征、能补的只有 7 处（下表 R4），其余 20 处两条路一样，只能走后一条。
- 看不看得到：M07–M21、M23–M27 这 20 处看不到：括注里的为什么、冒号或逗号引出的为什么、格言、表格行，71 号只看字面，这些句子与同位置的指令没有字面差别（M07 的「防」是单字、M08 的「为了」不在标点后，R4 没收，收了误伤面太宽，推的）。M01–M06、M22 看得到，靠的是 71 号现在没用的特征（「实测」后面跟数、轮名跟比例、「今天」跟数、标点后的「免得 / 所以」、不带路径的「计划第 N 节」）。
- 满足哪个分句：A2 触发观测第一半「指出一行按 71 号字面判绿、而按规则「只写怎么做」是说明的」（背景材料第 41 行）。M01–M04 另外正中共用约束第 9 行点名不许写的「实测」「经过」与记录。
- 改法中不中：「漏判的那一行改掉」对 27 处都中。但其中几处是执行者据以分支的前提（M11、M12、M15、M17，还有 M01 同一行的「后台起它的那条命令自己的 `$?` 只说明起没起来」）；照 A1-H1 量出来的教训，这几处要**改写成指令**，不能直接删——删掉就是 A1 那一类改坏。例：M17 改成与 `experiment-runner` 第 3 步同形的「报「基线就是红的」时先查是不是在仓根下跑」；M15 改成「`--write` 之前先跑不带 `--write` 的一次」（第 2 步已有，括注可删）。

71 号文件头第 27 行对自己漏判范围的写法偏窄：

> # 判不到的：不带这些词、不带日期的解释句（例如「发散是探索该有的样子」）这一道认不出，靠三方审核与人看（72 号）。

M01 带着「实测」照样判绿（④ 只认紧跟在标点后面的「实测」）；M02、M03 带着日期的替身（轮名、「今天」）；表格行里的解释（M27）带不带这些词都判绿——绿样本 `.claude/gate.d/fixtures/71-agent-def-flow-only.sh/green/setup.sh` 第 31 行「| 表格行不判解释 | 数据，因为表格只放数 |」就是带着「因为」判绿的。这一句该写成「只认紧跟在标点后、或在段首的这些词，与「」外的 `20\d\d-\d\d-\d\d`；别的位置、别的写法（轮名、「今天」）、表格行里的解释都认不出」。

改法 R4（我自己提的，只在我的模型上量过，被攻过零轮）：五条词法，在去掉「」、跳过围栏与表格行之后的行上找。

| 条 | 正则 | 今天 18 份 | 清理前 18 份 | 绿样本 |
|---|---|---|---|---|
| R4a | `实测[^，。；,;（）()「」]{0,8}\d` | crash-verifier.md:24 | 同左，另加 agent-common.md:42（清理删掉的那句） | 0 |
| R4b | `[a-z][a-z0-9]*(?:-[a-z0-9]+)*-r\d+[^，。；）)]{0,12}\d+\s*/\s*\d+` | kb-scribe.md:26 | 同左 | 0 |
| R4c | `今天\s*\d` | mutation-triage.md:25 | 同左 | 0 |
| R4d | `(?:[（(，,；;。：:]\|——)\s*(?:免得\|以免\|为了\|所以)` | experiment-designer.md:28、experiment-runner.md:26、kb-scribe.md:19 | 同左 | 0 |
| R4e | `计划第[一二三四五六七八九十]+节` | mutation-triage.md:25 | 同左，另加 three-way-local-attack.md:29（清理删掉的那句） | 0 |

原样三行（`probe_gate71.out` 第 46–48 行）：

```text
R4	today	hits=7	R4a@crash-verifier.md:24=实测约 47 ; R4d@experiment-designer.md:28=，免得 ; R4d@experiment-runner.md:26=，免得 ; R4d@kb-scribe.md:19=，所以 ; R4b@kb-scribe.md:26=agent-defs-r2 攻方模型：1/3 对 0/3 ; R4c@mutation-triage.md:25=今天 4 ; R4e@mutation-triage.md:25=计划第十八节
R4	before	hits=9	R4a@agent-common.md:42=实测一段执行员 6 ; R4a@crash-verifier.md:24=实测约 47 ; R4d@experiment-designer.md:28=，免得 ; R4d@experiment-runner.md:26=，免得 ; R4d@kb-scribe.md:19=，所以 ; R4b@kb-scribe.md:26=agent-defs-r2 攻方模型：1/3 对 0/3 ; R4c@mutation-triage.md:25=今天 4 ; R4e@mutation-triage.md:25=计划第十八节 ; R4e@three-way-local-attack.md:29=计划第十一节
R4	fixture-green	hits=0
```

今天的 7 处命中我逐处读过，全是说明（人判），没有一处是指令；R4 只在探针里另算，没接进 71 号脚本，接进去之后两套样本与 89 号过不过，推的。R4 修 M01–M06 与 M22，不修其余 20 处。

## 没打中的形状

- A1：上面第四节逐处列的 6 处（含一次 `oov-check.py` 实跑）；「开工先读 `.claude/agent-common.md`」（第 11 行，无冒号）与「开工先读：」一行（有冒号）标签撞名，我试了「子 agent 会不会把共用约束第 21 行的只读点名小节用到共用约束自己身上」——它要先读完共用约束才知道那条规矩，Read 默认一次读完 66 行，动作不变；`three-way-local-defense` 第 24 行删掉的「五次坏三次、两次闸判绿」由它照做的本地攻方第 5、6 步覆盖；共用约束删掉的缓存 5 分钟档与计时器成本：两条指令本来就不带条件，15 分钟门槛留着，主 agent 那侧 `main-agent.md` 第 26 行保留了 5 分钟档的前提；`experiment-designer` 第 3 步删掉的 E152 经过在它现在点名的 `.claude/kb/vm-harness.md` 第 147 行那一节里。取样范围：处置表 66 行与改动差 234 行逐块看过。
- A2 漏判：② 允许「带日期的记录 + 同一行一个 `records/…` 路径」判绿，④ 不认顿号后面的词——两种都能藏记录，但今天的 18 份文件里没有实例（4 行带日期：2 行只在「」里、2 行日期只在 `records/2026-09-16-…md` 文件名里）；frontmatter 的 `description:` 行 71 号照扫，18 行里没看到解释。
- A2 误判：没找到今天 18 份文件里已经被误判的行（基线全绿），九格都是造的；造的时候只改一处、不改指令的意思。

## 这条腿自己的限度

- 「是说明」「是纯指令」两种判定都是我读原文判的，规则里没有比 71 号词表更可操作的定义；M01–M27 与 FP1–FP9 的分类，换一个读的人可能有出入（M17、M21 这类前提最容易有分歧）。
- FP1–FP9 是造的：今天的文件基线全绿，没有一行已经被误判。它们像不像真会写出来的句子，是论证不是观测；FP1、FP2 点名的是真实存在的 kb 小节（`.claude/kb/tooling.md` 第 198、561 行），其余七格的原句都取自今天的文件、只改一处。
- A1 的两格打中都只量到「机制」或只推到「动作」：H1 的机制在玩具 crate 上量过（不是 singlefs 的工作区，cargo 1.98.0），实现员会不会走到那条路是推的；H2 整格是推的。没派实现员、核查员去试。
- 改法 R1、R2、R4 与 H1、H2 的写法都是我提的，只在我的模型上量过或只是推的，被攻过零轮。R1、R2 是改副本里的 71 号脚本量的；R4 没接进脚本；89 号那一道只照它的判法（退出码加 want 片段）在探针里仿了一遍，没跑 89 号本身。
- 处置表「经过在哪」一列（删掉的经过是不是真在 `records/` 那一节里）我没逐条核；它不影响指令，归不归这一轮的判据我不判。
- A1「没打中」只覆盖我逐块读过的形状；没有对 16 份定义逐个造输入跑新旧两版，「一个输入下动作变不变」靠读。

## 没做什么

- A3、A4 与附录三特征表那一半没碰（分工表不归我）；没读禁读清单里的文件。
- 仓里除了这份报告与模型目录，一个文件没写；没做任何 git 写操作。复跑、改脚本全在工作目录的拷贝里。
- 编了一个三行的玩具 crate（`mtime_toy.sh`，`nice -n 19`，草稿目录里自己的 target）。开跑前与写报告时 `ps` 看到别的会话一条 `cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 …`（pid 1175404，05:08 UTC 时已跑 42 分 30 秒，05:25 UTC 时 58 分 55 秒），没有性能测量在跑；玩具 crate 不用仓里的 target，没等锁。
- 阶段归属表里登记给 `three-way-attack` 的阶段 0 个（共用约束「门禁」一节那条 `awk` 接 `| wc -l` 输出 `0`），所以没跑任何登记阶段；门禁全量没跑；72、63 号没跑；71 号在仓里原样跑过一次（附末尾），其余都在拷贝上跑。
- 草稿目录 `/tmp/claude-1000/m2-agentdef-r1-opus/` 里还有：清理那一步的三份改动差（`cleanup-agents.diff` 等）、词表扫描脚本与输出（`scan_markers.py`、`scan_markers.out`，37 处命中，只用来找候选、不承重）、探针的工作目录。承重的脚本与输出都已拷进模型目录；草稿里的东西不入库，会话重启就没了，不影响复跑。

## 附：A1 用到的命令与原样输出

「依据：」→「开工先读：」16 行逐行比（去掉标签后比整行）：

```text
$ for f in research/prompts/m2-agent-def-cleanup/before/agents/*.md; do b=$(basename $f); old=$(grep -m1 '^依据：' "$f" | sed 's/^依据：//'); new=$(grep -m1 '^开工先读：' .claude/agents/$b | sed 's/^开工先读：//'); if [ "$old" = "$new" ]; then echo "SAME  $b"; else echo "DIFF  $b"; fi; done | sort | uniq -c -w4 | awk '{print $1, $2}'
2 DIFF
14 SAME
$ for f in research/prompts/m2-agent-def-cleanup/before/agents/*.md; do b=$(basename $f); old=$(grep -m1 '^依据：' "$f" | sed 's/^依据：//'); new=$(grep -m1 '^开工先读：' .claude/agents/$b | sed 's/^开工先读：//'); [ "$old" = "$new" ] || echo "DIFF  $b"; done
DIFF  three-way-defense.md
DIFF  three-way-local-defense.md
```

清理那一步改动差的行数：

```text
$ diff -r research/prompts/m2-agent-def-cleanup/before/agents .claude/agents | wc -l
188
$ diff research/prompts/m2-agent-def-cleanup/before/agent-common.md .claude/agent-common.md | wc -l
36
$ diff research/prompts/m2-agent-def-cleanup/before/main-agent.md .claude/main-agent.md | wc -l
10
```

门禁 71 号在今天的文件上（仓里原样跑一次，05:08 UTC 前后）：

```text
$ nice -n 19 bash .claude/gate.d/71-agent-def-flow-only.sh; echo "exit=$?"
  ✓ 定义只写怎么做（18 份文件 824 行；4 行带日期：2 行的日期只在「」引的小节名里，2 行同行指路；记录小节 0、解释性段落 0、解释性半句 0）
exit=0
```
