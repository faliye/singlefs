# 改法处置：m2-agentdef-r1 判决第二节第 1–7 行（2026-09-18）

规格 `research/prompts/m2-agentdef-r1-fix-spec.md`；判决 `research/prompts/m2-agentdef-r1-main-verification.md` 第二节；攻方报告 `research/prompts/m2-agentdef-r1-opus-output.md`（H1、H2、FP1–FP9、M01–M27、改法 R1、R4）。
改前的 18 份定义以开工快照 `research/prompts/m2-agentdef-r1-start-snapshot.sha256` 为准：把这一次做的 27 处替换倒着施加回去，18 份的 sha256 与快照逐份相同，处置表每一行的原句在改前那一行里逐字命中、改成的串在现在那一行里逐字命中（`research/prompts/m2-agentdef-r1-fix-evidence/disposition-check.out`）。

## 一、定义：逐处处置（规格第 1–4 条）

表由 `research/prompts/m2-agentdef-r1-fix-evidence/tools/disposition.py --table` 生成；实际做的 27 处替换（旧串、新串整段）在同目录 `definition-edits.json`。每处替换都在一行之内，改前改后行号相同。

| 标签 | 位置 | 原句那一截（整抄） | 改成什么 | 类别 |
|---|---|---|---|---|
| 规格 1 | `.claude/agents/implementation-writer.md:27` | 「（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录），先跑一份不改动的副本」 | 改成「（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本」 | 加指令（H1） |
| 规格 2 | `.claude/agents/three-way-verifier.md:21` | 「没给快照的代码轮，停下要，不对主树核。」 | 原句留着，后面加「没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」（与第 6 步那一类一样单列），不记 ✗。」 | 加指令（H2） |
| 规格 3 | `.claude/main-agent.md:24` | 「盯住，不强制结束：」 | 改成「盯住，不能一直干等，也不强制结束：」 | 补回（A3） |
| M01 | `.claude/agents/crash-verifier.md:24` | 「（54 号在本机负载下实测约 47 分钟，55、57、59 号各在一分钟内）」 | 删掉；那一处现在是「结束后读输出。别的会话」 | 删说明 |
| M01 同一行 | `.claude/agents/crash-verifier.md:24` | 「，后台起它的那条命令自己的 `$?` 只说明起没起来」 | 改成「；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`）」 | 前提改写成指令 |
| M02 | `.claude/agents/kb-scribe.md:26` | 「（agent-defs-r2 攻方模型：1/3 对 0/3）」 | 删掉；那一处现在是「用它，不用逐条 Edit。」 | 删说明 |
| M03 | `.claude/agents/mutation-triage.md:25` | 「（今天 4 张）」 | 删掉；那一处现在是「表头写着「被测装置是 shell 探针」的，照表头」 | 删说明 |
| M04 | `.claude/agents/mutation-triage.md:25` | 「：共用的会让开头几行链接上一轮最后一条变异，计划第十八节」 | 删掉；那一处现在是「不用它默认那个跨轮共用的）。」 | 删说明 |
| M05 | `.claude/agents/experiment-runner.md:26` | 「，免得读数被抢」 | 删掉；那一处现在是「`gate.sh` 也要等。」 | 删说明 |
| M06 | `.claude/agents/experiment-designer.md:28` | 「，免得命中行把结论带进来」 | 删掉；那一处现在是「--exclude-dir=prompts`。」 | 删说明 |
| M07 | `.claude/agents/experiment-designer.md:29` | 「，防等价变异」 | 删掉；那一处现在是「写明它在哪个取样点上改变输出。计时类」 | 删说明 |
| M08 | `.claude/agents/crash-verifier.md:13` | 「，派你是为了在提交前的整轮门禁之前先拿到它们的读数」 | 删掉；那一处现在是「里最重的那几道。」 | 删说明 |
| M09 | `.claude/agents/crash-verifier.md:23` | 「（两份层 0 全量同时跑各要一个多钟头）」 | 删掉；那一处现在是「不起 54 号，等它结束。」 | 删说明 |
| M10 | `.claude/agents/gate-triage.md:24` | 「（两道门禁同时跑，重阶段互相拖、工作区指纹互相干扰）」 | 删掉；那一处现在是「在跑就等它结束。」 | 删说明 |
| M11 | `.claude/agents/experiment-runner.md:27` | 「（后面的门禁阶段都不查命名）」 | 改成「新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。」 | 前提改写成指令 |
| M12 | `.claude/agents/experiment-runner.md:30` | 「（`replay.sh` 按这个相对路径找）」 | 改成「生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；」 | 前提改写成指令 |
| M13 | `.claude/agents/kb-scribe.md:28` | 「（变异表锚点里的分项标签不跟着改，会腐化）」 | 删掉；那一处现在是「加跑 33 号，并把」 | 删说明 |
| M14 | `.claude/agents/kb-scribe.md:28` | 「（入库产物里印着旧标签的，复跑会对不上）」 | 删掉；那一处现在是「逐个列给主 agent；预演」 | 删说明 |
| M15 | `.claude/agents/kb-scribe.md:27` | 「（`--write` 会先往那些条目里写「（待补）」）」 | 改成「就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；」 | 前提改写成指令 |
| M16 | `.claude/agents/implementation-writer.md:46` | 「（56 号门禁要的判决文件由主 agent 的那一轮产出）」 | 删掉；那一处现在是「- 没走三方对抗；层 0」 | 删说明 |
| M17 | `.claude/agents/mutation-triage.md:25` | 「；在仓根下跑会被报成「基线就是红的」」 | 改成「（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）」 | 前提改写成指令 |
| M18 | `.claude/agents/kb-scribe.md:26` | 「用它不用逐条 Edit：两阶段写在有一处不中时一个文件都不写，逐条 Edit 中途被打断或锚点被别人改掉会留下半套」 | 删掉；那一处现在是「用它，不用逐条 Edit。」 | 删说明 |
| M19 | `.claude/agent-common.md:64` | 「：`/tmp` 下的草稿目录会话一重启就没了，那一跑等于白干」 | 删掉；那一处现在是「再往下做。停机条款」 | 删说明 |
| M20 | `.claude/agent-common.md:43` | 「，那类读数被抢了 CPU 就不作数」 | 删掉；那一处现在是「停下；只有别的」 | 删说明 |
| M21 | `.claude/main-agent.md:26` | 「临时派 general-purpose 干带长等待的活时，派发提示里写上这一条：它不读共用约束。」 | 改成「临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上这一条。」 | 前提改写成指令 |
| M22 | `.claude/agents/kb-scribe.md:19` | 「标题决定条目挂在哪条决策下、条目里裸写的分项归谁，所以也由主 agent 给；」 | 删掉；那一处现在是「不由你概括；标题里「（其N）」那一段」 | 删说明 |
| M23 | `.claude/agents/experiment-designer.md:34` | 「`experiment-runner` 照这些节名找东西，以后门禁也按它们查。」 | 删掉；那一处现在是「写一句为什么不适用，不删节。」 | 删说明 |
| M24 | `.claude/main-agent.md:20` | 「发散是探索该有的样子，开口子不是问题，开出来没人收才是。」 | 删掉；那一处现在是「一条都不许无声消失。」 | 删说明 |
| M25 | `.claude/agent-common.md:43` | 「这个仓里别的会话几乎一直在跑 cargo，见到就停等于开不了工。」 | 删掉；那一处现在是「等锁等了多久。定义另有更严要求的照定义。」 | 删说明 |
| M26 | `.claude/main-agent.md:19` | 「延迟不是丢掉，」 | 删掉；那一处现在是「逐条写去向。不做也要写明依据。」 | 删说明 |
| M27 | `.claude/agents/experiment-designer.md:39` | 「（它只会整份写出口文件，出口写成登记本身会盖掉占号时写的文件头，而且退出码照样是 0）」 | 改成「；出口不指向登记本身，追加完回读登记，占号时写的文件头在不在看文件，不看退出码 \|」 | 前提改写成指令 |

计数：M01–M27 共 28 处（M01 一行两处），删说明 21 处，前提改写成指令 7 处（M01 同一行、M11、M12、M15、M17、M21、M27）；另加规格第 1–3 条各一处。

几处要写明的判断：

- M01「实测约 47 分钟」照规格删。它也是「54 号要后台跑」的依据；删掉之后定义里还剩第 2 步「单个阶段超过 Bash 单次上限就后台跑」与共用约束 `.claude/agent-common.md` 第 46 行「预计单段要等 15 分钟以上的（层 0 全量、E152（按里程碑对比六家文件系统的文件性能） 那类），不起计时器」。没另加「54 号一律后台跑」：规格把这一截列在删说明里，要不要加交主 agent。
- M12 改写的依据是 `research/scripts/replay.sh` 第 15 行 `cd "$(dirname "$0")/.."`、第 391 行 `cargo build --release --manifest-path e7-index-bench/Cargo.toml`、第 410 行 `./target/release/"$bin" $args`：设了 `CARGO_TARGET_DIR` 时它编到别处，跑的却是 `research/target/release/` 里原有的二进制。
- M15 改成「不让 `--write` 往那些条目里写「（待补）」」：原句说的是跑 `--write` 的后果，改成禁止那个后果；同一行后半「新增的行里有这一轮之外的条目就停下交回」照旧管已经写进去的情形。
- M21 把条件从名字（general-purpose）换成那条前提（不读共用约束的 agent），general-purpose 作为例子留在括注里。
- M22 删掉的是「为什么标题由主 agent 给」；「标题由主 agent 给」在同一行开头「标题整行、日期、改前、改后、依据各一句」里。
- M27 在表格行里：「只会整份写出口文件」「出口写成登记本身会盖掉文件头」「退出码照样是 0」三截各改成一条动作（出口不指向登记本身；追加完回读登记；文件头在不在看文件、不看退出码）。
- 规格第 2 条加在 `.claude/agents/three-way-verifier.md` 第 21 行（输入里讲快照那一条）。同一份第 27 行第 2 步「若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」」对没给快照的设计轮同样适用，两条碰上时先照哪条，定义没写；这一轮没改第 2 步。

## 二、M01–M27 之外、逐行读时认出的同类（没改）

规格点名的是 M01–M27；下面几处在改过的文件里，也是说明或前提，这一轮没动，列在这里：

| 位置 | 原句那一截（整抄） | 形态 |
|---|---|---|
| `.claude/agent-common.md:43` | 「（cargo 自己排队拿文件锁）」 | 括注里的前提（「照常跑」凭它） |
| `.claude/agent-common.md:64` | 「，主 agent 才知道它在哪、还来不来得及拷」 | 逗号引出的为什么 |
| `.claude/agents/kb-scribe.md:20` | 「，缺了门禁必红而你不能自己补」 | 逗号引出的为什么 |
| `.claude/agents/kb-scribe.md:20` | 「写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉」 | 前提 |
| `.claude/main-agent.md:26` | 「子 agent 的提示缓存是 5 分钟档」 | 前提（攻方报告第 279 行提到它还留着） |
| `.claude/agents/mutation-triage.md:17` | 「（多是连字符，源文件名是下划线）」 | 括注里的前提 |

## 三、门禁（规格第 5–8 条）

| 条 | 改了什么 | 文件 |
|---|---|---|
| 5 | ④ 在剥掉「」之后的行上判（攻方改法 R1），上下文片段也取剥掉之后的行；绿样本加一行「开工先读：`.claude/kb/tooling.md`「提示里的 `**粗体**:` 会诱发损坏（2026-08-29 实测 3/3 对 3/3）」」（FP1 那一行，小节名里的「实测」同时带数字，一并测 ⑤ 也剥「」） | `.claude/gate.d/71-agent-def-flow-only.sh`；`.claude/gate.d/fixtures/71-agent-def-flow-only.sh/green/setup.sh`、`green/expect` |
| 6 | 新判据 ⑤ 词法说明：攻方改法 R4 的五条正则原样，在剥掉「」、跳过围栏与表格行之后的行上找；命中一处记一处，报法照 ①–④（✗ 汇总、逐条明细、→ 下一步）；成功行报「词法说明判了 N 行（围栏与表格行不判），命中 0」。红样本在共用约束样本末尾加五行、每行只犯 ⑤ 的一种；绿样本加一行表格行「今天 4 张，所以只放数」（表格行不判 ⑤） | 同上，另 `red/setup.sh`、`red/expect` |
| 7 | 文件头：「判四条」改「判五条」、④ 写明先剥「」、加 ⑤ 的五种与正则、「都不判的」加 ⑤、「判不到的」改写成五类（④ ⑤ 的词不在标点后或在顿号后、「实测」「今天」后面跟的不是阿拉伯数字；日期的替身；表格行；括注、冒号、逗号引出的为什么；格言），另加「会误判的」一行写 FP3–FP9 那一类（说明与指令字面相同）；判别力一行跟着改 | `.claude/gate.d/71-agent-def-flow-only.sh` |
| 8 | 判决只认这一次改动新写的：`git diff --name-only --diff-filter=A` 相对基准、`--cached --diff-filter=A`、未跟踪三路取并集；被改过的旧判决不算点名。✗ 行与成功行写明「新写的」，→ 下一步加「点名写进新的判决，不往旧判决里补」 | `.claude/gate.d/72-agent-def-adversarial-review.sh`、`.claude/gate.d/56-crates-adversarial-review.sh` |
| 8 样本 | 89 号只跑每个阶段的 `red`、`green` 两个样本目录（`.claude/gate.d/89-stage-selftest.sh` 第 34 行 `for kind in red green`），「各加一份红样本」并进了原红样本：多一个被管文件（72 号是 `main-agent.md`，56 号是 `crates/demo/src/other.rs`），唯一点名它的是一份基准里就有、这次只补了一句的旧判决；期望缺失从 1 份变 2 份。56 号绿样本的新判决改成 `git add` 进暂存区（走暂存区新增那一路；72 号绿样本走未跟踪那一路） | 两道的 `red/setup.sh`、`red/expect`、`green/expect`，56 号另 `green/setup.sh` |

## 四、跑过的门禁与判别力

原样输出都在 `research/prompts/m2-agentdef-r1-fix-evidence/`：`gate-26.out`、`gate-47.out`、`gate-56.out`、`gate-62.out`、`gate-63.out`、`gate-71.out`、`gate-72.out`、`gate-89.out`、`doc-lint.out`、`gate-lint.out`、`shell-lint.out`；第 5、6 条改回去各一次的输出 `discrimination-item5.out`、`discrimination-item6.out`；第 8 条顺带做的 `discrimination-item8.out`；新 71 号在第 4 条改之前的 18 份定义上 `71-before-item4.out`。复跑用的脚本在 `tools/`：`fixture-check.sh`（照 89 号第 34–56 行写的单阶段版本）、`make-reverted.py`（造「改回去」的副本）、`disposition.py` 与 `definition-edits.json`。

## 五、没做的

- 72、56 号的红样本没有单开第二份目录（89 号不认），并进了原红样本，见第三节「8 样本」一行。
- 第一节列的 M01「54 号后台跑」与第二节的六处同类，没改，交主 agent。
- 71 号新判法在清理前原件上的逐项数（攻方报告第 190 行「①0 ②35 ③3 ④30」）没重跑；只跑了第 4 条改之前的 18 份（⑤ 命中 7 处，与攻方报告第 270 行逐处相同）与改之后的 18 份（全绿）。
