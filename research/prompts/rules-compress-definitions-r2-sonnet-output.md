# rules-compress-definitions-r2 辩方腿报告

日期：2026-11-16。轮名 rules-compress-definitions-r2，第二轮，立场辩方。这条腿没写模型，没往任务服务交任务；草稿目录 `~/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet/`（里面只有 `cite-r1.txt` 与 `scan-sample/` 三个样本脚本，sha256 在末尾「要主 agent 跑的命令」一节）。

被判件对快照：`sha256sum -c research/prompts/rules-compress-definitions-r2-snapshot/judged-sha256.txt` 对不上 0 行（量的）；第二版合稿树按 `merged-tree-sha256.txt` 对不上 0 行（量的，把清单里 `./` 去掉后在 `rules-compress-r2-merged/` 里核）。仓里 `.claude/agent-common.md` 此刻与快照同一份，所以下面引原件都直接引仓里的行。

写法：原件 = 仓里 `.claude/`；第一版 = `research/prompts/rules-compress-r1-merged/dot-claude/`；第二版 = `research/prompts/rules-compress-r2-merged/dot-claude/`。

## 一、先说结论

- 判决「二、判决」七条：七条打中都站得住，没有一条能判成「其实不成立」。其中第 5 条要补一句：原件两处本来就不一致（不是压缩带来的），但第一版照原件定义那一处写的 `<根>`，交回闸会拒，所以判它成立、第二版的改法是对的。
- 第二版的修补：七条都够得着。没有「照改反而把对的改错」的；留下四处小的读法余地，都是偏向被拒或多读，不偏向做错（见第三节「修补之后还剩的」）。
- 第 2 条「点到就整份读」写成两句：比原件窄一处（步骤指针只读那一节），宽一处（共用约束点名的规则开工就整份读）；判据没丢。逐项在 3.2。
- 不采纳的行：这一题 5 行里，4 行理由站得住（其中 1 行写的理由是「攻方自己判弱」，那不算理由，真正站得住的理由换成指到原文），1 行（格五「推不出」）不是不采纳而是没核完，留着；另有 1 行（矛盾题里执行员「卡住」）写的是「不改」，已被用户定案第 22 条推翻，去向表没跟着改，属记录过时，树是对的。
- 核查方式：缺三样（第五节）。最要紧的一样：主 agent 没跑 `cite-check.py` 的理由（服务排队）对它不成立，它是直接跑的文本工具；我直接跑了，1283 处引文全对上、7 处裸行号没判。

## 二、判决逐条复核

「站得住」= 判决说的那件事在原件、第一版、第二版里现读成立。

| 判决条 | 辩方判定 | 证据 |
|---|---|---|
| 1 逐个按进程号停，正是钩子判红的写法 | 站得住 | 第一版共用约束：「超时与结束时逐个按进程号发信号，连同它再起的子孙进程，不对进程组发、不按名字或 cgroup 挑」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`）；第一版工具员：「不对进程组发、不按名字或 cgroup 挑」（`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:33`）。钩子今天的文件头：「终止进程只许点名一个自己起的进程号或任务号，一次一个」（`.claude/hooks/bash-command-detector.sh:165`），拒「在 for / while / until 循环里逐个发」（`.claude/hooks/bash-command-detector.sh:170`），放行「对自己用新会话起的那一个子进程的进程组发信号」（`.claude/hooks/bash-command-detector.sh:184`）。脚本一侧由门禁扫：「同一份判定由门禁 code-tooling 的 research-gate-lint 格拿去扫」（`.claude/hooks/bash-command-detector.sh:192`）。钩子自证里就有一份循环逐个 kill 的样本、期望报红：「"loop.sh":」（`.claude/hooks/bash-command-detector.sh:5902`）、「循环里 kill 报在第 3 行」（`.claude/hooks/bash-command-detector.sh:5927`）。第一版要脚本里写的，就是这份样本。 |
| 1 补一句（辩） | 站得住，并且不是压缩的错 | 第一版是照用户定案第 6 条落的；第 6 条已撤回：「维持钩子现在的判法」（`records/2026-11-16-三方豁免取消与存量补轮.md:194`）。原件自己就有两处不一：共用约束「超时与结束时在脚本里对整组发信号，跑完核没有留下进程」（`.claude/agent-common.md:72`）对工具员定义「不按名字、cgroup 或进程组发」（`.claude/agents/tooling-writer.md:33`）。所以判它成立靠的是钩子文件头，不是靠「原件不是这样」。 |
| 1 的修补够不够得着 | 够得着 | 第二版共用约束：「超时与结束时在脚本里只对自己这样起的那一个子进程的组发信号，不对别的进程组发、不在循环里逐个发」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:67`）；执行前拒绝的 ⑥ 补了放行：「与对自己用新会话起的那一个子进程的进程组发信号」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:80`）；规则一节：「组号只取自己用新会话起的那一个子进程的」（`research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:9`）；工具员：「自己写的装置与脚本里只对自己用新会话起的那一个子进程的组发」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/tooling-writer.md:33`）；三条三方腿都指到共用约束与规则同名节（`research/prompts/rules-compress-r2-merged/dot-claude/agents/three-way-defense.md:31`、`research/prompts/rules-compress-r2-merged/dot-claude/agents/three-way-attack.md:39`）。五处与钩子 :184 说的是同一件事。原件 ⑥ 只写「放行的是点名一个」（`.claude/agent-common.md:90`），没把按组停写进放行，第二版补上是对的。 |
| 2 「点到就整份读」两种读法 | 站得住 | 第一版：「点到就整份读，不受读入大小的限制」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`）。「点到」可以是定义「开工先读：」那一行点到，也可以是共用约束与定义正文任何一处点到；两种读法读的文件不同（攻方数过：正文里点到的规则文件 9 份）。另一处证据：第一版「规则只读三样，别的不读」（`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:16`）与后一种读法撞。 |
| 3 旧数据复跑那句只剩两跳 | 站得住 | 原件那一条的触发句是「拿它支撑新结论之前先重新跑一遍」，出处是上游：「想拿它支撑新结论，先复跑一遍，确认今天还成立」（`.claude/singlefs-ai-sop/rules/kb-discipline.md:39`）。第一版只留「怎么用照 `.claude/rules/evidence-discipline.md`」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:24`），那一条的触发句是另一个：「对早先的结论有疑问时，重新验证它」（`.claude/rules/evidence-discipline.md:35`）。触发条件换了（有疑问 → 拿来支撑新结论），要再跳到上游才找到原话。 |
| 4 丢了「写成交接的副本不列」 | 站得住 | 原件：「子 agent 另不列写成「交接」的」（`.claude/agent-common.md:103`）；钩子文件头：「子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删」（`.claude/hooks/completion-cleanup-guard.sh:100`）；第一版共用约束里没有「写成「交接」的副本」这几个字（`grep -nF` 命中 0 行，量的）。 |
| 5 命名检查 `<根>` 两处写法不一 | 站得住（要补一句） | 第一版共用约束：「`<根>` 写 `<快照>`」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:109`），第一版实现员：「`<根>` 与 `--copy` 填你改动所在的树」（`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:31`）。原件也是这两处：「在副本里改、交补丁的，`<根>` 写 `<快照>`」（`.claude/agent-common.md:114`）对「`<根>` 与 `--copy` 填你改动所在的树」（`.claude/agents/implementation-writer.md:31`），原件靠「与定义冲突时以定义为准」解，不是「分开写」。判它成立的硬证据在交回闸：「根还可以写占位 `<快照>`」（`.claude/hooks/handback-guard.sh:46`）、「副本根与别的路径照拒」（`.claude/hooks/handback-guard.sh:48`）；实现员的副本是 minimal-copy 建的，不是 git worktree，按实现员定义写副本根会被拒。 |
| 6 调查员「查到就修」写明是例外 | 站得住 | 原件实现员：「没写的在主工作区改」（`.claude/agents/implementation-writer.md:15`）；第一版实现员改成一律交补丁（`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:15`），而调查员那一支要在主工作区改，第一版两处没连起来。用户定案第 19 条。 |
| 7 执行员例外只写吃紧 | 站得住 | 原件执行员定义写的是两样：「上下文吃紧或卡住时，停在最后一个写完整的点」（`.claude/agents/experiment-runner.md:33`）；第一版共用约束只留一样：「例外只有 `experiment-runner`：上下文吃紧时」（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`）。用户定案第 22 条：「上下文吃紧与卡住都可以交回、新派承接」（`records/2026-11-16-三方豁免取消与存量补轮.md:200`）。 |

补充第 3 条的原件行：原件那一条写在「拿它支撑新结论之前先重新跑一遍」（`.claude/agent-common.md:35`），是共用约束自己的一句，后面才括注指到上游。

## 三、修补够不够得着（第二版）

### 3.1 逐条

| 判决条 | 第二版落点 | 判定 | 说明 |
|---|---|---|---|
| 1 | 见上一节 1 的修补 | 够得着 | 五处与钩子文件头同一件事。留两处小的读法余地，见 3.3 |
| 2 | 「做那一步之前读那一节，读全」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:17`） | 够得着，没丢判据，宽窄各一处 | 单独核在 3.2 |
| 3 | 「拿它支撑新结论之前先重新跑一遍」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:24`） | 够得着 | 半句放回共用约束自己的一条里，触发条件与原件一致。丢了「确认今天还成立」（上游 `.claude/singlefs-ai-sop/rules/kb-discipline.md:39`），不改做法：重跑就是为了确认。「要推翻早先的结论，照三方规则重走一轮」原件在同一条里写成一句，第二版靠「别的用法照」指到那一节：「要推翻早先的结论，照 `.claude/rules/three-way-inference.md` 重走一轮」（`research/prompts/rules-compress-r2-merged/dot-claude/rules/evidence-discipline.md:61`），多一跳，句子在 |
| 4 | 「子 agent 交回、收工时不列（主 agent 结束一轮时照列、用完就删）」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:100`） | 够得着 | 与钩子文件头同一件事（`.claude/hooks/completion-cleanup-guard.sh:100`）。「基线」那半句：原件写「子 agent 写了照列」，第二版写「只认主 agent 的声明」，同义 |
| 5 | 「命名检查的 `<根>` 写 `<快照>`」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:31`）；「带 `--item` 的命令里的 `<根>` 与 `--copy` 填你的副本」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:31`） | 够得着，与交回闸一致 | 交回闸认 `<快照>`、拒副本根（`.claude/hooks/handback-guard.sh:46`、`.claude/hooks/handback-guard.sh:48`）。「`--cwd`、带 `--item` 的命令」一支指副本：与同定义 1d 的「`<根>` 填你改动所在的树」同一处（改动在副本里），不冲突 |
| 6 | 「不在副本里改、不交补丁」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:23`）；「那一支是例外，照旧在主工作区改」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:109`）；实现员一侧也写了例外（`research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:15`） | 够得着 | 三处互相指，主 agent 一侧的两份也写了（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:47`） |
| 7 | 「上下文吃紧或卡住（同一步反复做不过去）时」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:68`） | 够得着 | 与执行员定义自己的两样一致：「上下文吃紧或卡住时，停在最后一个写完整的点」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/experiment-runner.md:39`）。括号里「同一步反复做不过去」是主 agent 补的释义，原件没有（来历：`records/2026-11-16-三方豁免取消与存量补轮.md:200`）；它把「卡住」限在执行员自己的进度上，与共用约束另一处的「任务卡住由服务判」不撞，不是放宽也不是收严 |

### 3.2 判决第 2 条：原件那一条的原话，第二版两句比原件宽还是窄

原件（读规则文件）：
- 「一律整份读，不受读入大小的限制」（`.claude/agent-common.md:22`）。
- 「照样读全那份文件，点名的小节是这件活要照做的那几条」（`.claude/agent-common.md:22`）。
- 「只读点名的那几个小节」说的是规则类文件以外「开工先读：」点名的别的文件（`.claude/agent-common.md:23`）。
- 那一条只说怎么读（整份、不受大小限制），没说什么时候读；什么时候读由各定义的「开工先读：」一行定。

第二版两句（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:17`）：
- 第一句：「开工整份读，不受读入大小的限制」，对象是定义「开工先读：」点名的，与共用约束点名的（「每个定义都照守」那一条，与「写」一节里写「开工另读」的）。
- 第二句：「做那一步之前读那一节，读全」（步骤里写「照 <规则>「<节>」」的）；「做那一步之前读那一份」（只写「照 <规则>」没写节的）。

逐项比（量的：原件、第二版各按上面的行读出）：

| 项 | 原件 | 第二版 | 比原件 |
|---|---|---|---|
| 开工先读点名的规则文件 | 整份读，点名小节照做 | 同，原话保留 | 相同 |
| 共用约束「每个定义都照守」那三份（命令安全、文风、陈述外部状态） | 没写开工读；用到时按「一律整份读」整份读 | 开工整份读（三份：`.claude/singlefs-ai-sop/rules/command-safety.md` 127 行，`.claude/singlefs-ai-sop/rules/writing-discipline.md` 54 行，第二版合稿树里的 `evidence-discipline.md` 166 行；`wc -l` 量的。原件那一份是 `.claude/rules/verify-before-claiming.md` 54 行，三份合计 235 行，第二版合计 347 行） | 宽：时间提前，读的行数也多（235 行变 347 行，多出来的是合稿把陈述外部状态那份并进了证据纪律） |
| 共用约束「写」一节两处「开工另读」 | 原件没有这两个字，只有指针括注：「（`.claude/rules/verification.md`「判定库只有一份」」（`.claude/agent-common.md:40`） | 「写测试、门禁、入口脚本、实验装置的，开工另读」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:28`）、「写这几类的，开工另读」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:29`） | 宽：由「用到才读」变成「写这几类就开工读」（第一版已加，第二版沿用，三方判决没提） |
| 步骤里「照 <规则>「<节>」」 | 无这一类读取义务；真读了，按「一律整份读」读整份 | 读那一节，读全，不读整份 | 窄一处：读的范围只到那一节。宽一处：多了「做之前必须读」的义务 |
| 步骤里只写「照 <规则>」没写节 | 同上 | 读那一份 | 同整份；义务是新的 |
| 「不受读入大小的限制」 | 在那一句里 | 只在第一句；第二句「读那一份」没带这半句 | 窄：若将来哪份规则超过工具读入上限，第二句读法没明说照样读全。今天最大的规则文件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md` 36784 字节，在上限之内（`wc -c` 量的），不触发 |

判：两种读法的二义消掉了；第二版没有丢原件的判据，宽的一面（开工多读三份、写这几类多读判定库与多机两节）是显式写出来的，窄的一面（节指针不读整份）是本轮判决要的写法。需要主 agent 知道的只有两点：一是「开工整份读」的集合比原件大；二是节指针读那一节不再带原件「一律整份」的上下文，指到的节若依赖同文件别的节，要在那一节里自足。我没有逐处核第二版里每一处节指针指到的节是否自含，没找到反例只说明没抽到，这一点没核。

### 3.3 修补之后还剩的（都偏向被拒或多读，不偏向做错）

1. 第二版规则写的是「shell 里用 `setsid --wait` 起（不带 `-f`），同一层紧跟着 `名字=$!`」（`research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:8`），钩子认的是「先 setsid [--wait] <命令> &」（`.claude/hooks/bash-command-detector.sh:185`）：规则没写后台符号 `&`。照规则字面不加 `&` 写，`$!` 不是那个子进程，钩子不认、判红；失败方向是被拒。
2. 工具员第 5 步前一句「自证里只对自己起的进程号发信号」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/tooling-writer.md:33`）与后一句「自己写的装置与脚本里只对自己用新会话起的那一个子进程的组发」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/tooling-writer.md:33`）：自证脚本算前一句还是后一句，读得出两种。钩子两种都放行（`.claude/hooks/bash-command-detector.sh:184`），所以做法不会被判红，只是工具员可能把按组停当成自证里不许用的。
3. 同一句还点名了 `proc.py stop <pid>`，共用约束写「`proc.py` 只给主 agent」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:66`）。原件就是这样（`.claude/agents/tooling-writer.md:33` 与 `.claude/agent-common.md:71`），不是压缩带来的，也不是第二版带来的，列出来给主 agent。
4. 第二版共用约束命名检查那一句，括号「调查员「查到就修」那一支是例外，照旧在主工作区改」后面直接接「`<根>` 写 `<快照>`」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:109`）：读得出调查员也写 `<快照>`。交回闸只在改动全落在副本里时认 `<快照>`（`.claude/hooks/handback-guard.sh:47`），主工作区改了的「根仍要填那棵树」（`.claude/hooks/handback-guard.sh:48`），所以调查员写错会被拒，不会悄悄放过。

## 四、去向表里这一题写「不成立」「部分」的行（`research/prompts/rules-compress-r2-merged/dispositions.md`）

| 行 | 去向表写的 | 辩方判定 | 证据 |
|---|---|---|---|
| 91 带 `--item` 的四条整条删成指针 | 「不成立（攻方自己判弱）」（`research/prompts/rules-compress-r2-merged/dispositions.md:91`） | 不采纳站得住，但理由要换：「攻方自己判弱」不是理由 | 四条整句模板还在实现员第 1d 步：「带 `--item` 的命令照第 1 步写进报告」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:34`），共用约束指过去：「`--item` 可给几次，各条的 `--list-items` 列可点的项」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:60`）。第二版 14 份定义里写带 `--item` 命令的只有实现员与调查员（`grep -l -- '--item' agents/*.md` 命中 `investigator.md`、`implementation-writer.md`，量的），调查员那一份也指到实现员第 1d 步（`research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:33`）。没有第三类读者需要共用约束里内联四条 |
| 95 项目代码闸、写闸其余各道、弹窗闸、定时闸没逐句核 | 「部分核」（`research/prompts/rules-compress-r2-merged/dispositions.md:95`） | 这不是「不采纳」，是没核完；应该留成开着的格，不当作已判不成立 | 去向表自己写了只核了交回正文闸与范围声明闸的一段。这一轮攻方的分工格三已把它们列为重点（正文「分工」表）。我没替它核：钩子文件头逐句对共用约束这四段不在辩方这一份的范围里，没做 |
| 96 `quote-rust-items.py` 钩子库还没加 | 「成立（脚本一侧的活）」（`research/prompts/rules-compress-r2-merged/dispositions.md:96`），不改文字 | 站得住 | 直接跑名单里今天没有它：`.claude/hooks/lib_project_code_runs.py` 的 `TEXT_TOOLS_RUN_DIRECTLY` 一段列了 `cite-check.py`（`.claude/hooks/lib_project_code_runs.py:1085`）、`quote-kb.py`（`.claude/hooks/lib_project_code_runs.py:1089`），`grep -nF quote-rust-items` 命中 0 行（量的）。材料员定义原件与第二版都写了要跑它（原件 `.claude/agents/three-way-materials.md:29`），所以这是原件带来的、第二版没动的；用户定案第 10 条排了脚本一侧的批次（`records/2026-11-16-三方豁免取消与存量补轮.md:137`）。文字不改对 |
| 98 找不到落点 3 到 5（消费方说明、写范围闸括注、「不抄第二份」） | 「不成立（不带判据）」（`research/prompts/rules-compress-r2-merged/dispositions.md:98`） | 站得住 | 三类都是说明句：写范围一节在三份定义里都还在（`research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:63`、`research/prompts/rules-compress-r2-merged/dot-claude/agents/experiment-runner.md:54`、`research/prompts/rules-compress-r2-merged/dot-claude/agents/kb-scribe.md:39`），「不抄第二份」那一句的读取指令（开工先读 `format-evolution.md` 那一节整节）在执行员开工先读行里（`research/prompts/rules-compress-r2-merged/dot-claude/agents/experiment-runner.md:16`）。这几句放到原件第一次写下的那天读，删掉之后做法不变 |
| 99 指针核对表 E 类 5 个 | 「不成立（只管主 agent 的动作）」（`research/prompts/rules-compress-r2-merged/dispositions.md:99`） | 站得住 | 5 个目标是 `main-agent.md`、两份提交与门禁技能、三方技能、拆分提案记录。第二版里指到它们的句子，要么是写给主 agent 的动作（「次序照 `.claude/main-agent.md`「写与验分两段」验证段」，`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:109`），要么是工具员要改的文件名（`research/prompts/rules-compress-r2-merged/dot-claude/agents/tooling-writer.md:40`），子 agent 做的事不依赖读它们 |
| 119（矛盾题）执行员「卡住」只写吃紧 | 「只写吃紧，不改」（`research/prompts/rules-compress-r2-merged/dispositions.md:119`）；修稿报告同（`research/prompts/rules-compress-r2-merged/revise-result.md:42`） | 当时不采纳站不住，现在这一行已过时 | 攻方是对的：原件执行员写的是「吃紧或卡住」（`.claude/agents/experiment-runner.md:33`）。用户定案第 22 条把「卡住」补回，第二版树里已有（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:68`）；去向表这一行与修稿报告没跟着改。树是对的，记录两处该改：去向表第 119 行、修稿报告第 42 行 |

有没有该采而没采的：没有新的。上面 6 行里，91 与 99 理由站得住，96 与 98 站得住，95 是没核完，119 已被用户定案推翻、树里已落。另外有一条去向表里写「成立」但第二版没补的，不算不采纳、也不是我这一份要翻案的：派发闸那一项原件里主 agent 一侧的「自证在哪跑：」一行点了工作根或会话临时目录根下的路径会被拒（`.claude/agent-common.md:105`），第二版没带，去向表写进「拿不准」（`research/prompts/rules-compress-r2-merged/dispositions.md:88`「拿不准」）；子 agent 一侧那半（「子 agent 照它填命令里的路径」）第二版还在（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:108`）。

## 五、核查方式：判决写「主 agent 没有另跑 snapshot-tree.py、改成逐处现读」

判决原话：「主 agent 这一轮没有另跑 `snapshot-tree.py`（任务服务排着 91 条、小任务起不来），改成对腿点名的行逐处用 grep 与 sed 现读」（`research/prompts/rules-compress-definitions-r1-main-verification.md:7`）。规则要求的做法：「先经 `task-server/task submit --cwd <草稿目录> -- python3 <仓根>/scripts/agents/snapshot-tree.py」（`.claude/rules/three-way-inference.md:128`），再自己跑 `cite-check.py`，两条命令原样写进判决。核查与判决由主 agent 做：「核查与判决由主 agent 做，不派子 agent 去核子 agent」（`.claude/rules/three-way-inference.md:126`）。

够不够：不够，缺三样。

1. 缺 `cite-check.py` 一条，这一样不是服务排队能挡的。它在直接跑名单里：「"scripts/agents/cite-check.py": "any"」（`.claude/hooks/lib_project_code_runs.py:1085`），主 agent 不用交任务。判决只写了腿自己跑过「末行都是对上」，那是被核的一方自己报的数，规则要主 agent 自己跑。我直接跑了：`python3 scripts/agents/cite-check.py` 对两份第一轮腿报告加背景材料，退出码 0，末行「核了 1283 处引文、7 处裸行号，对上 1283 处，没判 7 处」（输出在草稿目录 `cite-r1.txt`，量的）。7 处裸行号没判，全在攻方报告里：钩子自证样本行 `.claude/hooks/bash-command-detector.sh:5902`、`:5914`（第 48 行）、第一版 `agent-common.md:87`（第 72 行）、`:104`（第 73 行）、原件 `.claude/agent-common.md:105`（第 73 行）、第一版 `three-way-attack.md:39` 与 `three-way-defense.md:31`（第 82 行）。这 7 处正是攻-1、攻-3、攻-6 的支撑行。我现读过：钩子 :5902 是 `loop.sh` 样本、:5927 期望「循环里 kill 报在第 3 行」；第一版 attack:39、defense:31 的原句是「样本进程逐个按进程号停」；第一版共用约束 :87 是 ⑬ 那一行，原件 :105 是派发闸那一项。7 处都读到了，与攻方说的一致。
2. 缺 `snapshot-tree.py` 的替代理由没写全。这一轮用不着倒推：被判的原件对快照清单逐份 sha256 都对上（我开工时 `sha256sum -c` 对不上 0 行，量的），第一版合稿树也在仓里，所以原件与第一版的行号用仓里现读与倒推出来的树是同一份。判决应写「原件没有被改过，所以对着仓现读」，而不是只写服务排队；规则那一步的另一半用处（倒推出已被改过的原件）在这一轮不起作用。命令在第六节，主 agent 想补可以交。
3. 「对腿点名的行逐处现读」只覆盖打中的那几条与腿的观察，没覆盖正推腿落点表里判「一致」的那些行。正推腿 582 行落点里，有「原话基本在」的 270 行（量的：`grep` 按「原话基本在 NN%」数）靠的是按句字符串近似匹配，每行引文只截前 15 个字左右，匹配度最低 70%（`research/prompts/rules-compress-definitions-r1-sonnet-output.md` 第 823 行 at-08）。近似匹配只证明那句话前面几个字在，不证明整句意思没变。证据：同一份报告把「一律整份读」改成「点到就整份读」判成「一致或同义改写」（`research/prompts/rules-compress-definitions-r1-sonnet-output.md:279`），而这一条后来被判成成立（判决第 2 条）。所以那 270 行加 154 行「一致」都没有被任何一方逐句核过。我抽了匹配度最低的 10 行里的 9 行（at-08、df-14、er-18、er-29、inv-15、at-11、cl-04、sw-24、iw-62；at-33 是整句换成指针没逐句对），逐句对原件与第二版，看到四处删减：三方攻方「由用户决定的动作」那一句括注里「照三方规则那一节「攻方腿的装置把用户动作写死时…」那一条办」的小节指针没了（原件 `.claude/agents/three-way-attack.md:30`，第二版 `research/prompts/rules-compress-r2-merged/dot-claude/agents/three-way-attack.md:30` 只指到「判决由主 agent 做，不由投票做」整节）；三方辩方「交回闸对三方腿跑同一个脚本，对不上就拒交回」少了后半（原件 `.claude/agents/three-way-defense.md:33`，第二版 `research/prompts/rules-compress-r2-merged/dot-claude/agents/three-way-defense.md:33`）。实现员「交补丁」那一条（原件 `.claude/agents/implementation-writer.md:59`，第二版 `research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:70`）少了「合入后验证认这几张登记表、不当判不出测试目标的路径」，行为在脚本文件头；执行员「停机条款」那一步少了括注「写进报告，主 agent 跑」（原件 `.claude/agents/experiment-runner.md:36`，第二版 `research/prompts/rules-compress-r2-merged/dot-claude/agents/experiment-runner.md:42`），同一步里「命令照第 1 步」还在。四处都是说明性的，不改做法。另外 er-18 那一块有一处意思相反：入库装置原件写「不进代码轮判决的点名清单」（`.claude/agents/experiment-runner.md:32`），第二版写「进代码轮三方」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/experiment-runner.md:36`）；这一处不是悄悄改的，矛盾总表登了：「入库装置进代码轮」（`research/prompts/rules-compress-r2-merged/contradictions.md:65`）。其余几行没发现意思变了。这是 9 行抽样，不是 270 行。

## 六、要主 agent 跑的命令

这一节的命令我都没跑（子 agent 不交任务），读数写「推的，待主 agent 跑」。样本脚本在草稿目录，sha256（`sha256sum`，量的）：

- `~/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet/scan-sample/scan-root/scripts/loop-by-pid.sh`：9f63d144b629275ce9705e647b573e92f02999615126dcdfd5f29e85517d47b1（第一版叫人写的停法：循环里逐个 `kill -TERM "$pid"`）
- 同目录 `own-group.sh`：0f7bd548c00cd3147969423a7bcf58bbbd69f01e01834c69cb36ea6f01b2138f（第二版叫人写的停法：`setsid --wait … &`、`sample=$!`、`kill -TERM -- -"$sample"`）
- 同目录 `own-group.py`：647e4ab296dde745d837a801f62d66eeac852ca635527ddc4944a14917fc44f4（`start_new_session=True` 起、`os.killpg(sample.pid, …)`）

1. 判决第 1 条与第二版修补的钩子读数（推的，待主 agent 跑：应当只报 `loop-by-pid.sh` 第 3 行，不报另两份）
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/hooks/bash-command-detector.sh --scan-scripts /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet/scan-sample/scan-root /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet/scan-sample/scan-root/scripts`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`（`--cwd` 已写）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=1`；`task-server/task log <号>` 里有 `scripts/loop-by-pid.sh:3` 一行、没有 `own-group.sh`、`own-group.py` 的行算过。若 `exit=0` 或 `own-group.*` 被报，第二版规则与共用约束里的写法就判红，推翻本报告第二节第 1 条的「够得着」。若范围声明闸要声明，种类按 `items` 写。
2. 判决第五节第 2 样：补跑倒推树，再用它当 `--root` 跑 `cite-check.py`
   - 命令一：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py rules-compress-definitions-r1 --out /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet/r1-tree --repository /home/fy5090/code/singlefs`
   - 命令二（直接跑的文本工具，不交服务，仓根起）：`python3 scripts/agents/cite-check.py research/prompts/rules-compress-definitions-r1-opus-output.md research/prompts/rules-compress-definitions-r1-sonnet-output.md --root /home/fy5090/code/singlefs --root /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet/r1-tree --background research/prompts/_rules-compress-definitions-r1-body.md`
   - 看哪几行：命令一 `TASK_SERVER_RESULT` 的 `exit=0` 且汇总说点名的都倒推出、sha256 全对上；命令二退出码 0、末行「对上」的数与我读到的相同（1283 处引文、7 处裸行号）。倒推出的树做完后用 `task-server/task clean <号>` 删，我没建它。

## 七、什么现象会推翻这份报告里的结论

- 第 1 条「站得住」：命令 1 里 `loop-by-pid.sh` 不报红；或钩子文件头 :170 的循环一句被删。
- 第 5 条「站得住（要补一句）」：交回闸 `.claude/hooks/handback-guard.sh` 改成认副本根；或实现员的副本是 git worktree 建的。
- 3.2「没丢判据」：找到一份规则文件，第二版节指针只读那一节而漏掉同文件别的节里的判据；我只说没抽到，没逐处核。
- 第四节 91 行：第二版 14 份定义里除实现员、调查员外有第三份需要写带 `--item` 的命令。
- 第五节第 3 样：对那 270 行近似匹配的落点逐句核时找到意思变了而没登记的；我抽的 9 行里没有。

## 八、没做什么

- 没替主 agent 撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务；判决要引的钩子读数（第六节命令 1）没复跑，是「推的，待主 agent 跑」。
- 没逐句核钩子文件头对共用约束项目代码闸、写闸其余各道、弹窗闸、定时闸四段（攻方分工格三）。
- 没逐处核第二版里每个节指针指到的节是否自含。
- 没读禁读清单里的文件：`research/prompts/rules-compress-definitions-r2-opus-output.md`、`research/prompts/rules-compress-definitions-r2-opus-model/`、攻方草稿目录、别的几题第二轮各腿产出与草稿目录（`grep` 与 `ls` 的范围都限在被判的原件、第二版合稿树、第一轮报告与判决、规则与钩子文件头里）。
- 草稿目录 `~/.local/share/singlefs/work/rules-compress-definitions-r2-sonnet/` 里没有仓副本、编译目录与库拷贝（`du -sh` 44K，量的：只有 `cite-r1.txt` 与 `scan-sample/`）。
