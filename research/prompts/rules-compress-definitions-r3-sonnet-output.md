# rules-compress-definitions-r3 正推腿报告

日期：2026-11-16。轮名 rules-compress-definitions-r3，第三轮。立场：正推（从原件、钩子文件头、用户定案推出「应该是什么」，再与第三版逐格比）。
被判对象：`research/prompts/rules-compress-r3-merged/diff-from-r2.patch` 里落在 dot-claude/agent-common.md 与 dot-claude/agents/ 下 14 份定义的每一处改动。下文「第三版」指合稿树 `research/prompts/rules-compress-r3-merged/dot-claude/`，「原件」指仓里 `.claude/`。

## 一句话结论

- 格一（第二轮修补对不对）：打中 1 处（定案第 25 条落的那一句不是原话），其余逐处核对都对（放回原件原话的是逐字一致，改写的与钩子头一致）。
- 格二（修补引出的新问题）：有。第三版 agent-common.md 第 39、96 行把批号写法裸写在「」里，被判据库判中 6 处，写闸、Bash 检出钩子 ⑰ 与门禁 doc-text 的 batch-codes 格都会拒（我自己写这份报告时，第一次 heredoc 就因为同样的字面被 ⑰ 拒了）。
- 格三（能不能换上）：这一组 diff 范围里，除上面几处，没找到矛盾总表与定案之外的做法差别。
- 这一组能不能换上：不能原样换上；「换上之前必须改」三条（见末尾），都是一行级别的改动，改完可以换上。

## 怎么核的（只读）

- diff 里每个 `+` 行，先与原件整行比对（我自写的草稿脚本 `restore.py`、`ldiff.py`，在 `~/.local/share/singlefs/work/rules-compress-definitions-r3-sonnet/`）：agent-common 第 17 行、three-way-attack 第 30 行、three-way-defense 第 33 行与原件整行逐字一致；其余是在原件基础上改写的句子，逐句对依据。
- 第三版这一组全部文字过一遍判据库：把仓里 `.claude/gate.d/lib-forbidden-notations.py` 拷进草稿目录（sha256 与仓里一致），用 `python3 -I` 调它的 `all_hits`，对 15 份文件判 batch-codes、clock-times、session-names、prime-marks，路径按 `.claude/...` 传入（不在任何排除前缀下）。同一脚本判原件 agent-common.md：四形态全 0 处，说明判法没有误伤。
- 指针：自写脚本 `ptr.py`、`ptr2.py`、`conv.py` 把这一组里所有「`文件`「节」」与「共用约束「节」」对到合稿树（规则）与仓里（上游 `.claude/singlefs-ai-sop/`、kb）的标题行。
- 钩子与脚本的依据一律引文件头的段；行号是此刻工作区版本的行（与 HEAD 差得多，只当此刻的定位）。

## 逐处核对表：agent-common.md

记法：第三版行指 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md` 的行；原件行指 `.claude/agent-common.md` 的行。

### C1 规则怎么读 · 第一条

- 改前（第二版）：两句版，「定义「开工先读：」一行点名的，与共用约束点名的……开工整份读」加「步骤里写「照 <规则>「<节>」」的，做那一步之前读那一节」。
- 改后：「一律整份读，不受读入大小的限制」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:17`）。
- 依据：原件同一句「一律整份读，不受读入大小的限制」（`.claude/agent-common.md:22`）；整行逐字比对 orig[22] == third[17] 为真。主 agent 处置「改回原件原话」（`records/2026-11-16-三方豁免取消与存量补轮.md:222`）。
- 对不对：成立。放回的是原件原话，做法回到原件。

### C2 每个定义都照守的三处

- 改前：「command-safety.md（退不回去的操作，动手前先想一遍；`pkill -f` / `killall` 一律禁用）」，括号写法。
- 改后：「`pkill -f` / `killall` 一律禁用」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:20`），三个指针都写成「节」形。
- 依据：原件「跑命令照」（`.claude/agent-common.md:25`）与「给人看的文字照」（`.claude/agent-common.md:26`）同样是「…」形；上游标题行在：`.claude/singlefs-ai-sop/rules/command-safety.md:7`、`:48`，`.claude/singlefs-ai-sop/rules/writing-discipline.md:6`。
- 对不对：成立。指针与原件相同；删掉的「两节」「一节」只是量词，没有改做法。

### C3 批号加后缀（新增半句）

- 改前：只写「批次字母代号（`5a`、`9g`……）」。
- 改后：「这类批号加后缀的写法」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:39`），前面举的例子是 `第十五批乙二`、`批 10 甲`（我在这里加反引号；第三版那一行没加）。
- 依据：写闸文件头七「同一道另判批次代号」（`.claude/hooks/write-guard.sh:41`）；判据库「batch-code-stem：天干编号的批号」（`.claude/gate.d/lib-forbidden-notations.py:49`）。原件同一条「进仓的文字不写会话名与批次代号」（`.claude/agent-common.md:54`）没有这半句：这是新补，不是放回。
- 对不对：内容成立（钩子今天就拒这种写法）；写法不成立：这一行自己把两个例子裸写在「」里，被判据库判中（见「会被拒的行」）。

### C4 D、E 编号的指针

- 改前：`kb-discipline.md`（5. 编号只能做索引，不能做称呼），括号写法。
- 改后：「5. 编号只能做索引，不能做称呼」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:42`）。
- 依据：原件同一指针「5. 编号只能做索引，不能做称呼」（`.claude/agent-common.md:57`）；上游标题 `.claude/singlefs-ai-sop/rules/kb-discipline.md:49` 是同名的 `## 5.` 节。
- 对不对：成立。同一行里原件的「（kb 之外的也算）」「不只写编号」第二版就没了（不在本轮 diff）：`.md` 本来含 kb，「写成「编号（简称）」」已含「不只写编号」，没找到做法差。

### C5 提交之外跑不指定范围的门禁

- 改前：「提交之外要跑照 `verification.md`「提交之外」那一条先弹窗」。
- 改后：「提交之外要跑照」后接「那一条确认一次」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:59`）。
- 依据：用户定案第 24 条（`records/2026-11-16-三方豁免取消与存量补轮.md:212`）：带 `--full-gate` 或 `--whole-package-confirmed` 的命令由钩子弹的那一次就算确认，不带的主 agent 自己弹窗问一次。被指到的一条在第三版 `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`，写的正是这两种；钩子头「放宽检查的提交标识」（`.claude/hooks/run-scope-guard.sh:33`）一致。原件是「提交之外要跑先弹窗问用户」（`.claude/agent-common.md:68`）。
- 对不对：成立。共用约束这一句只指过去、不另抄判法，没有多住一处。

### C6 「样本进程放进它自己的进程组」那一条

- 改前（第二版）：「不在循环里逐个发」，「命令行里停进程照旧一次点名一个（⑥）」。
- 改后：「不在 shell 循环里逐个发」，结尾「命令行上与脚本里同一个判法（「执行前拒绝的写法」⑥），别的停法一次点名一个自己起的进程号」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:67`）。
- 依据：用户定案第 20 条「命令行上与脚本里都只许对自己用新会话起的那一个子进程的进程组发信号」（`records/2026-11-16-三方豁免取消与存量补轮.md:194`）；钩子判法：shell 里在循环里发一律报「在 for / while / until 循环里逐个发」（`.claude/hooks/bash-command-detector.sh:4009`），所以「shell 循环」这个限定词与钩子相符。原件是「对整组发信号」「命令行里停进程照旧一次点名一个」（`.claude/agent-common.md:72`），与第 20 条相比已被定案改写。
- 对不对：成立。「shell」二字是第三版新加的限定，使它与同一页 ⑥ 行放行 Python 循环里 `os.killpg` 不打架（见 C7）。

### C7 执行前拒绝 ⑥

- 改前：放行「对自己用新会话起的那一个子进程的进程组发信号」。
- 改后：同一句后加「Python 里 `os.killpg` 在循环里每一轮停自己起的那一组也放行」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:80`）。
- 依据：主 agent 处置「钩子放行的「每一轮停自己起的那一组」不算」（`records/2026-11-16-三方豁免取消与存量补轮.md:226`），用户定案第 20 条要的是「维持钩子现在的判法」；钩子代码注释「os.killpg(<进程组号>, …) 与 os.kill(-<进程组号>, …) 放行，在循环里调也放行」（`.claude/hooks/bash-command-detector.sh:4114`）。原件 ⑥ 只放行「点名一个」（`.claude/agent-common.md:90`），没有进程组一项。
- 对不对：成立，但依据在钩子的代码注释里；文件头 ⑥ 那一段只在「认法与照旧拒的写法……写在代码里」（`.claude/hooks/bash-command-detector.sh:187`）指过去，并且头里另有一句「在循环或推导式里调 os.kill」（`.claude/hooks/bash-command-detector.sh:182`）。读者只看头会读到相反的意思；见「半句清单」。

### C8 项目代码闸的出路

- 改前：「出路是写成展开之后的字面命令、经 submit 交」，名单写「`MAIN_AGENT_TOOLS_RUN_DIRECTLY`」。
- 改后：「子 agent 把那一条写进报告」，并列「`proc.py`、`remind.sh`、`watch.sh`、`agent-watch.py`、`dispatch-prompt.py`，名单是……`MAIN_AGENT_TOOLS_RUN_DIRECTLY`」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:95`）。
- 依据：钩子头「子 agent 那一份的出路是把同一条」写进报告「要主 agent 跑的命令」一节（`.claude/hooks/project-code-guard.sh:193`）；名单表此刻 `MAIN_AGENT_TOOLS_RUN_DIRECTLY = {`（`.claude/hooks/lib_project_code_runs.py:1099`）。原件项目代码闸（`.claude/agent-common.md:100`）同样分主 agent、子 agent 两条出路。
- 对不对：成立。两处差别：第三版把原件列的 `task-server/control` 的 install / start / stop / watch 去掉了；表里实际还有 `lib_session_registry.py`、`who-touched.py`、`queue-health.py`、`prune-server/control` 等，第三版只举 5 个并说「名单是」某表，没有写成「只有这 5 个」，名单以表为准；不构成做法差。

### C9 写闸那一行加批次代号

- 改前：「内容里有撇号类角标、钟点或会话号」。
- 改后：「批次代号」加四个例子，「这类拿来当名字用的」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:96`）。
- 依据：写闸文件头七「同一道另判批次代号（拒绝时另列一道「写了批次代号」）」（`.claude/hooks/write-guard.sh:41`）；原件写闸那一行没有批次代号（`.claude/agent-common.md:101`）。
- 对不对：内容成立；写法不成立（四个例子裸写在「」里，被判中，见「会被拒的行」）。另：钩子头写明「库按后缀只判 .md、.tsv、.txt，不认放行标记，仓外不判」（`.claude/hooks/write-guard.sh:42`），第三版这一句没带这个后缀限定，对 `.sh`、`.py`、`.rs` 目标会读成也拒（只在「写闸」一句的射程上多说了，不影响做事）。

### C10 定时闸

- 改前：「除 `bash scripts/agents/remind.sh` 之外的定时办法」。
- 改后：「主 agent 与子 agent 一样拒」「`remind.sh` 只给主 agent 直接跑（项目代码闸），子 agent 要等什么写进报告交主 agent」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:102`）。
- 依据：钩子头「主 agent 与子 agent 一样拒」（`.claude/hooks/timer-only-remind-guard.sh:5`）与「本工程设定时只许 scripts/agents/remind.sh 这一个办法」（`.claude/hooks/timer-only-remind-guard.sh:4`）；`remind.sh` 在只给主 agent 的名单里（`.claude/hooks/lib_project_code_runs.py:1101`）。原件定时闸（`.claude/agent-common.md:107`）写了挂载的工具名，第三版没写（头里有）。
- 对不对：成立。「子 agent 要等什么写进报告」是从名单推出的做法，不是原件原话，也没有与钩子冲突。

### C11 交回正文闸的主语

- 改前：「书记员与执行员报了没数或 0 项的绿行」。
- 改后：「写回员（`kb-scribe`）与执行员（`experiment-runner`）」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:103`）。
- 依据：钩子头 ③「kb-scribe / experiment-runner」（`.claude/hooks/handback-guard.sh:31`）；头第 3 行写「书记员与执行员的绿行不报数」（`.claude/hooks/handback-guard.sh:3`）。仓里还有 `clerk`（事务员），「书记员」单独写可读成它；第三版点名到 agent 名，消歧。
- 对不对：成立。同一行没改的「只判项目子 agent」与头 ④「全部子 agent」（`.claude/hooks/handback-guard.sh:12`）不一致：后台任务那一项对内置通用类型也判。第二版就是这样，不在本轮 diff，记在「半句清单」。

### C12 弹窗闸

- 改前：「问句或选项里……（不可能、一定、必然、从来不、永远不、恒为这类）」。
- 改后：「问句或选项的说明（description）里……词表只认这九个」，九个词逐个列出（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:104`）。
- 依据：钩子头判法「扫 tool_input.questions 里每一问的 question 与每个 options[].description」（`.claude/hooks/ask-user-claim-guard.sh:16`）与「找断言词：不可能、造不出、从来不、从来没有、一定、必然、永远不、绝不会、恒为」（`.claude/hooks/ask-user-claim-guard.sh:18`）；原件同一项（`.claude/agent-common.md:104`）已列这九个。
- 对不对：成立。放回原件词表，与钩子一致；出处写「反引号里的路径或命令」也与钩子头一致。

### C13 派发闸清单里去掉重复的「写者锁持有者还在跑」

- 改前：句首有「写者锁持有者还在跑又没有一行「写者锁已判」」，清单后部又有一项无条件的「写者锁持有者还在跑」。
- 改后：只留带条件的一项（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:104`）。
- 依据：原件「又没写「写者锁已判：<理由>」」只出现带条件的一项（`.claude/agent-common.md:105`）；钩子头「提示里没有「写者锁已判：<理由>」一行就拒」（`.claude/hooks/runner-dispatch-guard.sh:102`）。
- 对不对：成立。去掉的是第二版多写的无条件那一项，回到原件。

### C14 命名检查交主 agent 跑（调查员那一支）

- 改前：括号里写「调查员「查到就修」那一支是例外，照旧在主工作区改」，紧接着「`<根>` 写 `<快照>`」，读起来像 `<快照>` 也管调查员。
- 改后：先写完实现员（`<根>` 写 `<快照>`），分号之后「调查员「查到就修」那一支是例外，照旧在主工作区改，它的 `<根>` 写仓根、证红的副本改完再建（`.claude/agents/investigator.md` 第 6 步）」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:109`）。
- 依据：用户定案第 19 条（`records/2026-11-16-三方豁免取消与存量补轮.md:171`）与主 agent 处置「调查员「查到就修」那一支的验证命令写在它改的那棵树上」（`records/2026-11-16-三方豁免取消与存量补轮.md:225`）。
- 对不对：成立；与调查员定义第 6 步一致（见「调查员那一支的命令落点表」）。残留二义见该表 R6。

### C15 用户定案第 25 条：共用约束那一处「原话照留」

定案原文：「25. 主 agent 自己可以写什么：「三处原话各留，不合并」，不写「只有」」（`records/2026-11-16-三方豁免取消与存量补轮.md:213`）。矛盾总表第三版记的落点（`research/prompts/rules-compress-r3-merged/contradictions.md:164`）：共用约束一处，原件 `.claude/agent-common.md:8`，第三版 `.claude/agent-common.md:11`，标签「只改措辞」。

逐字并排：

| | 句子 |
|---|---|
| 原件 `.claude/agent-common.md:8` | 「**简单任务不派发，自己直接改**，例如修改一下、查询一下、计划一下等，任务单一流程简单的任务。」 |
| 第三版 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:11` | 「主 agent 派发：简单任务（修改一下、查询一下、计划一下这类单一、流程简单的）自己直接做，不派；派之前先规划……」 |

- 第三版里「简单任务不派发，自己直接改」0 命中（`grep -nF` 在 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md` 上零行），「自己直接改」整个合稿树（除总表等说明件外）0 命中；「不派发」改成「不派」，「自己直接改」改成「自己直接做」，「任务单一流程简单的任务」改成括号里的「这类单一、流程简单的」，并与相邻的「派发任务要规划……」（`.claude/agent-common.md:9`）并成一行。
- 对不对：不成立（字面）。第 25 条要这一处「原话照留」；第三版是改写，修稿一方（`research/prompts/rules-compress-r3-merged/dispositions-r2.md:29`）和总表都写成「各留原话」，与文件对不上。做法上没看出差别：「改」与「做」在这里指同一类小活（原件自己的例子就有查询、计划），所以这是定案字面没落到，不是做法被改。
- 多出来或少掉的半句：少了「任务单一流程简单的任务」（原件把「简单」再释一遍）；多了「派之前先规划，同类任务一起派」等并进同一行的半句（原件是独立的下一行「派发任务要规划，尽量同类任务一起做，最大程度避免串行」）。

## 逐处核对表：14 份定义

记法：第三版行指 `research/prompts/rules-compress-r3-merged/dot-claude/agents/<名>.md` 的行；原件行指 `.claude/agents/<名>.md` 的行。「开工先读」七处改动的共同点：第二版把上游规则的「节」写成括号里的说明，第三版放回「…」形，与原件一致。

| 编号 | 位置 | 改前 → 改后 | 依据 | 对不对 |
|---|---|---|---|---|
| A1 | clerk 开工先读 | 括号 → 「1. 正文只写当前现状，历史一律放到文末」 | 原件同样写成「…」（`.claude/agents/clerk.md:16`）；上游标题 `.claude/singlefs-ai-sop/rules/design-doc-discipline.md:7` | 成立 |
| A2 | experiment-runner 开工先读 | 括号 → 「测试镜像一律放临时目录」 | 原件同（`.claude/agents/experiment-runner.md:16`）；上游标题 `.claude/singlefs-ai-sop/rules/command-safety.md:86` | 成立 |
| A3 | experiment-runner 第 3b 步 | 「在跑产物之前照原文真跑」后补「（命令照第 1 步写进报告，主 agent 跑）」 | 原件「在跑产物之前做，照原文真跑（命令照第 1 步写进报告，主 agent 跑）」（`.claude/agents/experiment-runner.md:36`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/agents/experiment-runner.md:42`）只少「做，」二字 | 成立：括号是原件原话；第二版少了「写进报告，主 agent 跑」是丢，已补回 |
| A4 | implementation-writer 开工先读 | 括号 → 「测试镜像一律放临时目录」「子 agent 交回之前，删掉自己建的编译目录与仓副本」 | 原件把路径写两遍各挂一节，第三版并成一个路径挂两节；两节标题 `.claude/singlefs-ai-sop/rules/command-safety.md:86`、`:119` 都在 | 成立 |
| A5 | implementation-writer 第 1 步 | 括号内加「实现员是自己的副本；调查员「查到就修」填的树见 `.claude/agents/investigator.md` 第 6 步」 | 定案第 19 条（`records/2026-11-16-三方豁免取消与存量补轮.md:171`）与处置（同文件 :225）；落点 `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33` 存在且唯一 | 成立；残留一处读法，见 R7 与半句清单 |
| A6 | implementation-writer 第 3 步 | 「照共用约束「建仓副本」在草稿目录建副本」→「在你改动所在的副本里证红（副本带着这次的实现与新测试；……调查员「查到就修」改在主工作区，改完另建一份带改动的副本，见 `.claude/agents/investigator.md` 第 6 步）」 | 同 A5；「建仓副本」是共用约束里唯一的加粗条（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:35`） | 成立 |
| A7 | implementation-writer 产出·补丁目录 | 「登记行跟着走，第 7a 步」后加「合入后验证认这几张登记表、不当判不出测试目标的路径」 | 原件已有同一半句（`.claude/agents/implementation-writer.md:59`）；脚本头 `REGISTRATION_TABLES`（`scripts/gate/merge-and-verify.sh:89`）认这几张表 | 成立：放回原件原话（句序略调，没少字） |
| A8 | investigator 开工先读 | 括号 → 「子 agent 交回之前，删掉自己建的编译目录与仓副本」 | 原件同（`.claude/agents/investigator.md:16`）；标题 `.claude/singlefs-ai-sop/rules/command-safety.md:119` | 成立 |
| A9 | investigator 第 1 步 | 「`--cwd` 填副本」→「复现与二分的命令 `--cwd` 填副本；写了「查到就修」的，验证改动的命令照第 6 步填它改的那棵树，不填修之前建的副本」 | 处置（`records/2026-11-16-三方豁免取消与存量补轮.md:225`）；第 6 步（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33`） | 成立：与第 6 步一致；一个小处：「它改的那棵树」对证红是副本、对其余是仓根，靠「照第 6 步」才分得开 |
| A10 | investigator 第 6 步 | 加「这几条命令的 `--cwd` 与命令里的 `<根>` 写主工作区仓根……证红的 `prove-red.sh` 不认主工作区，……填改完之后……新建的副本（带着改动）」 | 定案第 19 条；`prove-red.sh` 头「不是这份脚本所在的主工作区」（`scripts/mutation/prove-red.sh:171`） | 成立；变异验证命令那一条有二义，见 R6 |
| A11 | prior-art 开工先读 | 括号 → 「2. 每条带出处与状态」 | 原件同（`.claude/agents/prior-art.md:16`）；标题 `.claude/singlefs-ai-sop/rules/kb-discipline.md:32` | 成立 |
| A12 | three-way-attack 开工先读 | 括号 → 「子 agent 交回之前，删掉自己建的编译目录与仓副本」 | 原件同（`.claude/agents/three-way-attack.md:17`） | 成立 |
| A13 | three-way-attack 第 3 步 | 「（`three-way-inference.md`「判决由主 agent 做，不由投票做」）」→ 加「一节「攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的」那一条办」 | 整行与原件逐字相同（原件 `.claude/agents/three-way-attack.md:30`）；落点 `research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:39` 在「判决由主 agent 做，不由投票做」一节（该节标题 :30、下一节 :41）里 | 成立：放回原件整行；指针指得到且唯一 |
| A14 | three-way-defense 第 5 步 | 加「，对不上就拒交回」 | 整行与原件逐字相同（原件 `.claude/agents/three-way-defense.md:33`）；钩子头 ②「有对不上的拒」（`.claude/hooks/handback-guard.sh:26`） | 成立：放回原件整行 |
| A15 | tooling-writer 开工先读 | 括号 → 「…」形（sop-first、command-safety、show-me-test、code-discipline、rules-discipline 共 5 份） | 原件同样是「…」形（`.claude/agents/tooling-writer.md:16`）；标题逐条在：`.claude/singlefs-ai-sop/rules/sop-first.md:18`、`:50`、`:40`，`.claude/singlefs-ai-sop/rules/show-me-test.md:27`，`.claude/singlefs-ai-sop/rules/code-discipline.md:139`，`.claude/singlefs-ai-sop/rules/rules-discipline.md:9`、`:27`、`:32`、`:40`、`:48` | 成立 |
| A16 | tooling-writer 第 5 步 | 「自证里只对自己起的进程号发信号……不对别的进程组发信号」→ 许／不许两张清单（见下面 S1 逐条） | 定案第 20 条（`records/2026-11-16-三方豁免取消与存量补轮.md:194`）：「工具员定义第 5 步照这个写清」；钩子头 ⑥ | 成立，依据有一半在钩子代码注释里，见 S1 |

### S1 tooling-writer 第 5 步逐句对钩子头 ⑥ 与定案第 20 条

第三版：`research/prompts/rules-compress-r3-merged/dot-claude/agents/tooling-writer.md:33`。

- 「许的是点名一个自己起的进程号、一次一个（`kill "$!"`、`proc.py stop <pid>`）」：钩子头放行行「`kill "$!"`、`kill -TERM "$pid"`、`kill %1`、`kill %%` 这样点名一个的」（`.claude/hooks/bash-command-detector.sh:183`）。一致。
- 「与对自己用新会话起的那一个子进程的进程组发信号」：头「对自己用新会话起的那一个子进程的进程组发信号」（`.claude/hooks/bash-command-detector.sh:184`），定案第 20 条同。一致。
- 「不许的：对别的进程组发（目标带负号而不是自己起的那一组、是 0 或 `$PPID`），一次给几个目标，在 for / while / until 循环里逐个发 `kill` 或 `proc.py stop`，按名字、cgroup 或 `/proc` 挑进程再发」：头「kill 的目标带负号（进程组；-1 是能发到的全部）」（`.claude/hooks/bash-command-detector.sh:169`）起的几项；shell 里的循环判法 `.claude/hooks/bash-command-detector.sh:4009`。一致（第三版没抄头里 fuser、cgroup.kill、systemctl、loginctl 等不属「发信号」的项，不影响这一步）。
- 「Python 里 `os.killpg` 在循环里每一轮停自己起的那一组钩子也放行」：依据是钩子代码注释（`.claude/hooks/bash-command-detector.sh:4114`），不是文件头。定案第 20 条字面是「循环逐个发……一律拒」，但同一条先说「维持钩子现在的判法」，钩子今天放行这一种，所以第三版跟钩子；主 agent 处置同（`records/2026-11-16-三方豁免取消与存量补轮.md:226`）。
- 判：成立。多出来的半句只有最后一句（出自钩子，不出自定案字面）。

## 会被写闸与检出钩子拒的行

判法：写闸文件头七「库按后缀只判 .md、.tsv、.txt，不认放行标记，仓外不判」（`.claude/hooks/write-guard.sh:42`）；检出钩子 ⑰「往仓里的文件写字面内容，内容里有会话号、钟点或批次代号」（`.claude/hooks/bash-command-detector.sh:295`）读同一份库；门禁 doc-text 的 batch-codes 格全仓逐行判。判据库的上下文规则只放过反引号代码段、产物字段、算式（`.claude/gate.d/lib-forbidden-notations.py:54`），「」里的不算。合稿树不在任何排除前缀下（排除的只有上游副本、门禁与钩子样本目录、变异表）。

对 15 份文件（agent-common 加 14 份定义）判 batch-codes、clock-times、session-names、prime-marks，合起来命中 6 处，全在 agent-common.md，别的 14 份定义与原件 agent-common.md 都是 0 处：

- 第三版 `agent-common.md` 第 39 行（批号加后缀那一条）：第 107 列 `第十五批乙二`（判据行 batch-code-stem）；第 115 列 `批 10 甲`（batch-code-stem）。
- 第三版 `agent-common.md` 第 96 行（写闸那一行）：第 147 列 `24c-1`（batch-code-suffixed）；第 154 列 `批次 24c`（batch-code-labeled）；第 162 列 `第十五批乙二`（batch-code-stem）；第 170 列 `批 10 甲`（batch-code-stem）。
- 钟点、会话号、撇号类角标：这一组 0 处。
- 复现：合稿树 agent-common.md 上这 6 处，与我写这份报告时第一次 heredoc 被 ⑰ 拒的列表同形（第 41、43 行的 `第十五批乙二`、`批 10 甲`）：钩子今天就拒这种写法。
- 出路：判据库的出路说明「代码字面放进反引号」（`.claude/gate.d/lib-forbidden-notations.py:512`）；把这两行里的例子逐个套反引号即可，不改意思。

门禁 doc-text 另有 private-values、out-of-project-names 两格（不在这份判据库里），这一组新写句子里没有主机名、外部目录名，我没逐格跑它们的脚本。

## 调查员「查到就修」那一支：命令落点表

走的链：调查员定义第 1、5、6 步（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:28`、`:32`、`:33`）→ 实现员定义第 1、1b、1d、3、3a、4 步（`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:31`、`:46`）→ 共用约束「建仓副本」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:35`）与「命名检查交主 agent 跑」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:109`）。「填得出」指照文字能唯一填成一个值。

| 编号 | 要交的命令 | 落在哪棵树（`--cwd`） | 占位符填什么 | 填得出吗 |
|---|---|---|---|---|
| R1 | 原样复现（第 2 步） | 副本 `<草稿目录>/copy`（修之前建的） | 输入给的命令改成在副本里起 | 填得出 |
| R2 | 二分、推翻现象（第 3、4 步） | 各自的副本（修之前建的，几份由调查员定） | 路径调查员自取 | 填得出（命名没规定，自取不冲突） |
| R3 | `cargo clippy`、`cargo build`、动到的测试目标（实现员第 4 步，调查员第 6 步点名） | 主工作区仓根 | 没有 `<根>`；`--cwd` 写仓根（调查员定义第 6 步「这几条命令的 `--cwd` 与命令里的 `<根>` 写主工作区仓根」） | 填得出 |
| R4 | 带 `--item` 的四条（形式化、崩溃枚举用例、崩溃放量、变异；实现员 1d，调查员第 5 步点名） | 主工作区仓根 | `--cwd <根>` 与命令里 `<根>` 都写仓根；`--item` 点这一次改到的项，项在主工作区里（改动已在那里） | 填得出 |
| R5 | 证红 `prove-red.sh`（实现员 3a） | 改完之后从主工作区新建的副本 | `--cwd <副本>` 与 `--copy <副本>` 都填这份新副本；`prove-red.sh` 头「不是这份脚本所在的主工作区」（`scripts/mutation/prove-red.sh:171`）故不能填仓根 | 填得出，两处未规定：新副本放在草稿目录哪个名字下、它在不在 `.deliverables` 写成「交接」（共用约束「用完就删」那一条要求写，调查员定义没点名；不写则完工清理闸列它） |
| R6 | 新变异行的验证段那一条：`crates-mutation-replay.sh --item '<行名>'`（实现员 1d 末段） | 实现员定义写 `--cwd <快照>` 与结尾 `<快照>`「原样写这个占位、主 agent 照第 1 步打出的快照路径填」 | 调查员定义第 6 步说这类命令的 `<根>` 写仓根；两处各说一个值 | 填不出：调查员这一支没有写段补丁，也没有「第 1 步留下的快照」可填；读调查员定义写仓根，读实现员 1d 原文写 `<快照>` |
| R7 | 命名检查两条 | 主工作区仓根 | 共用约束「命名检查交主 agent 跑」写「它的 `<根>` 写仓根」（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:109`）；实现员第 1 步同一句里「命名检查的 `<根>` 写 `<快照>`」紧挨着调查员的括号（`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:31`） | 填得出（以共用约束与调查员第 6 步为准）；实现员第 1 步那一句调查员若读到，会读出两种 |
| R8 | 拒算登记表核对 `refused-test-registry.py list <根>`（实现员 3c） | 改动所在的树 | 调查员第 6 步点名的步骤是 1d、2、3、3a、3b、4，不含 3c（原件同样不含） | 没规定：调查员补测试要不要写这一条，两版都没说 |
| R9 | 崩溃枚举用例登记行与 `crash-case-check.py`（实现员 7a） | 改动所在的树 | 第 6 步没点名 7a，调查员写范围也不含 `.claude/gate.d`（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:37`） | 不适用（调查员那一支不新写崩溃枚举用例） |

填不出的两项：R6 的 `<快照>`（二义，建议在调查员第 6 步加一句「新变异行的验证段那一条也写仓根」，或在实现员 1d 那一段加「调查员写仓根」）；R5 的交接登记（共用约束已写，调查员定义没点到）。R8 是两版相同的空白，不是这一轮引入。

## 子 agent 起步读不到的文件：指针清单

子 agent 起步不读 CLAUDE.md 与规则。这一组被改的句子里，指到它起步读不到的文件的指针：

| 位置 | 指到 | 子 agent 要不要读 |
|---|---|---|
| `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:59` | `.claude/skills/crash-test/commit-stages.md`、`.claude/skills/gate/staged-gate.md` | 不要：写的是主 agent 提交时怎么跑 |
| `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:104` | `.claude/skills/three-way/SKILL.md`「用户叫停、不写判决的轮」 | 不要：派发闸只管主 agent 用 |
| `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:109` | `.claude/main-agent.md`「写与验分两段」验证段 | 不要：写的是主 agent 在验证段怎么填 `<快照>` |
| `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:67` 与 `:80` | `.claude/rules/command-safety-by-review.md`「装置起的样本进程，放进它自己的进程组」 | 要。三方腿的定义（three-way-attack 第 39 行、three-way-defense 第 31 行）与我自己的定义（three-way-forward 第 30 行）开工先读里都没点名；该文件 76 行，共用约束的读法（`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:19`「没被「开工先读：」点名的别的文件」整读一次；规则类「一律整份读」）读得到，节标题 `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:4` 存在且唯一 |
| `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:59` | `.claude/rules/verification.md`「提交之外」 | 要看的是主 agent 做法，子 agent 只需知道「不指定范围的提交之外不由它起」；`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38` 是加粗条而不是标题，指针写「那一条」，指得到 |

全组「`文件`「节」」指针：用脚本逐个对标题，共解析 ptr.py 报告的全部项，指不到的 0 处；标题不是标题行、而是加粗条或正文的 3 处（`.claude/rules/verification.md`「提交之外」、`.claude/rules/evidence-discipline.md`「旧证据是参考，不是依据」、`.claude/agents/implementation-writer.md`「开工先读：」那一行）：都按「那一条」写、不按标题写，可指到。五处只写文件名不写目录的指针（`test-script-discipline.md`、`evidence-discipline.md`）在 experiment-designer、experiment-runner、sweep 三份定义里（这一轮没改到那几行），不在本轮 diff；文件名在规则目录里唯一，读得到。

## 多出来或少掉的半句清单

| 编号 | 位置 | 多出来或少掉 | 出处 | 是否改做法 |
|---|---|---|---|---|
| H1 | 共用约束第 11 行 | 少：「任务单一流程简单的任务」；字面「不派发，自己直接改」改成「自己直接做，不派」；多：并进来的「派之前先规划，同类任务一起派」 | 原件 `.claude/agent-common.md:8`、`:9`；定案第 25 条要原话 | 否；字面没落到定案 |
| H2 | 共用约束 C3（第 39 行） | 多：批号加后缀的写法也不写（新补，原件没有） | 写闸头七（`.claude/hooks/write-guard.sh:41`）、判据库（`.claude/gate.d/lib-forbidden-notations.py:49`） | 否（钩子与门禁今天就拒） |
| H3 | 共用约束 ⑥（第 80 行）与 tooling-writer 第 5 步 | 多：Python 里 `os.killpg` 在循环里每一轮停自己起的那一组放行 | 钩子代码注释（`.claude/hooks/bash-command-detector.sh:4114`）；定案第 20 条「维持钩子现在的判法」；而头里只有「在循环或推导式里调 os.kill」（`.claude/hooks/bash-command-detector.sh:182`） | 否；依据在代码注释，头与句子对不上 |
| H4 | 写闸那一行（第 96 行） | 多：批次代号；少：「按后缀只判 .md、.tsv、.txt」 | `.claude/hooks/write-guard.sh:42` | 否（`.sh`、`.py`、`.rs` 目标上读成也拒，钩子不拒；只是读宽了） |
| H5 | 定时闸（第 102 行） | 多：「子 agent 要等什么写进报告交主 agent」；少：挂载的工具名 | 名单 `.claude/hooks/lib_project_code_runs.py:1101` | 否（由名单推出；总表定义-13 登记了） |
| H6 | 项目代码闸（第 95 行） | 少：原件列的 `task-server/control` 四个子命令；名单只举 5 个 | 名单表 `.claude/hooks/lib_project_code_runs.py:1099` | 否（句子写「名单是……」，以表为准） |
| H7 | 交回正文闸（第 103 行） | 没动的「只判项目子 agent」与头 ④「全部子 agent」不一致 | `.claude/hooks/handback-guard.sh:12` | 第二版起就有，不在本轮 diff；内置通用类型的子 agent 不读共用约束，影响小 |
| H8 | 实现员第 1 步（第 31 行） | 调查员括号之后紧跟「命名检查的 `<根>` 写 `<快照>`」，调查员读到会读出两种 | 共用约束 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:109` 写仓根 | 否（以共用约束与调查员第 6 步为准）；二义 |
| H9 | 调查员第 6 步 | 少：新副本写进 `.deliverables`；少：变异验证段命令的 `<根>` | 共用约束「用完就删」那一条；R5、R6 | 否；有一处二义（R6） |

## 三格结论

记法：「成立」指这一格「怎么算打中」的条件满足；「不成立」指没找到；「推不出」指现有材料不够判。

### 格一 第二轮修补对不对：成立（找到 1 处定案落偏，其余对）

- 放回原件原话的：C1、A13（three-way-attack 第 3 步）、A14（three-way-defense 第 5 步）与七处「开工先读」的指针形（A1 到 A4、A8、A11、A12、A15 的各条），逐字或逐指针与原件一致。
- 与脚本文件头是不是一回事：C5、C6、C8、C10 到 C13 的句子逐项对了钩子头，一致；C7 与 S1 的循环放行半句与代码注释一致、与头对不上（H3）。
- 用户定案落的那一句：第 24 条（C5）成立；第 19 条（C14、A5、A6、A9、A10）成立；第 20 条（C6、C7、A16）成立；**第 25 条（C15、H1）不成立**：共用约束那一处不是原话。
- 证据：C15 的逐字并排；`grep -nF '简单任务不派发'` 在第三版 agent-common.md 上 0 行。
- 什么现象会推翻：若 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md` 里另有一行逐字含「简单任务不派发，自己直接改」；或主 agent 向用户确认第 25 条的「原话」只指意思。

### 格二 修补引出的新问题：成立（找到 3 处）

1. 新写的句子被判据库判中（第 39、96 行，共 6 处），写闸、检出钩子 ⑰、门禁 doc-text 的 batch-codes 格都会拒；我写报告时 ⑰ 真拒了第一次写入。推翻它的现象：这两行换上后判据库命中 0（`scan2.py` 退出行 `total 0`），或合稿树被登进 batch-codes 的排除前缀。
2. 调查员那一支的变异验证命令 `<根>` 与 `<快照>` 两种读法（R6）。推翻它的现象：主 agent 说明调查员那一支也有「第 1 步留下的快照」，则 `<快照>` 可填，这一条消失。
3. 循环放行半句的依据只在钩子代码注释里（H3）：读者只看文件头会读成相反（头列了「在循环或推导式里调 os.kill」）。这是小问题，换上后可改。
- 没找到的：同一条判据又住两处且说法不同的（样本进程放进进程组、命名检查 `<根>`、确认一次，三处各自互相一致；进程停法住在共用约束两条与工具员第 5 步三处，措辞不同但判法相同，没有互相抵触）；指针指不到的 0 处，指不唯一的 0 处；指到起步读不到的文件的指针见上面清单，都不影响子 agent 做事。

### 格三 能不能换上：推不出「完全一样」；这一组的 diff 范围内没找到登记之外的做法差别

- 这一格要求把原件里带「不」「只」「一律」「除外」「先」「才」的句子逐个对到第三版；那是攻方腿的做法。我只对了这一轮 diff 里的改动：每一处的做法与原件或钩子头相同，或已登记在矛盾总表（定义-11 到定义-14）与用户定案里，没找到没登记的不同。diff 之外（前两轮已判、第二版就有的压缩）我没逐句对原件，所以格三我只能回「diff 范围内不成立（没找到没登记的不同）、整组推不出」。
- 什么现象会推翻：任何一句原件的约束在第三版这一组里找不到对应（攻方腿负责）。

## 这一组能不能换上

不能原样换上；改下面三条后可以换上。

### 换上之前必须改

1. `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md` 第 39、96 行：把 `第十五批乙二`、`批 10 甲`、`24c-1`、`批次 24c` 各套反引号（或去掉例子只留「这类批号加后缀的写法」）。不改的话换上那一步写闸与 ⑰ 当场拒，提交时 doc-text 的 batch-codes 格红。矛盾总表第 119 行（`research/prompts/rules-compress-r3-merged/contradictions.md:119`）也含同样字面，判据库在 research 路径下也判中 4 处（那一份是材料，不在这一组）。
2. 同文件第 11 行：改回定案第 25 条要的原话「简单任务不派发，自己直接改」（原件 `.claude/agent-common.md:8`），总表第 164 行的落点跟着改。一行字面，做法不变。
3. 调查员第 6 步（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33`）加一句：新变异行的验证段那一条命令，`--cwd` 与 `<根>` 也写仓根（或说明它也填快照）。不加的话调查员写出的命令主 agent 填不出。

### 换上之后可以再改

- H3：⑥ 行与工具员第 5 步里的循环放行半句，加一句「认法写在钩子代码 SESSION_CHILD_CALLS 与 OWN_GROUP_TARGET 两段注释里」。
- H4：写闸那一行加「批次代号只判 .md、.tsv、.txt」。
- H6：项目代码闸名单写成「例：……」，或补回 `task-server/control` 四个子命令。
- H7：交回正文闸「只判项目子 agent」改成「除后台任务那一项判全部子 agent」。
- H8：实现员第 1 步命名检查那一句加「（实现员）」。
- H9：调查员第 6 步加「新建的带改动副本写进 `.deliverables` 成交接」。
- R8：调查员补测试要不要写 `refused-test-registry.py list`，两版都没说，补一句。

## 要主 agent 跑的命令

我写的核对脚本都是只读的草稿脚本，已用 `python3 -I` 直接跑过一次（钩子没拦）；判决要引数时请照下面三条复跑（`libfn.py` 是仓里 `.claude/gate.d/lib-forbidden-notations.py` 的拷贝，sha256 一致 1d128fb7…e3a5）。

- 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r3-sonnet -- python3 -I scan2.py`；在哪个目录起：`/home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r3-sonnet`（即 `--cwd`）；跑完看：`TASK_SERVER_RESULT` 行 `exit=0`，`task-server/task log <号>` 里逐行列命中，末行 `total 6`，全在 `.claude/agent-common.md` 的第 39、96 行。
- 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r3-sonnet -- python3 -I scan3.py`；在哪个目录起：同上；跑完看：输出一行 `orig hits`，四个形态都是空表（原件 agent-common.md 0 处，证判法没误伤）。
- 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r3-sonnet -- python3 -I ptr.py`；在哪个目录起：同上；跑完看：末行 `total problems 8`，其中 5 行是只写文件名的指针（`FILE-MISSING`，不在本轮 diff），3 行是加粗条不是标题（`in-body-not-heading`）。

脚本 sha256：`scan2.py` aab0f11c…1b6、`scan3.py` 5430cec2…7b3b、`ptr.py` d5e75c02…f780、`libfn.py` 1d128fb7…e3a5（完整值用 `sha256sum` 在草稿目录现取）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 格一 第二轮修补对不对 | 成立 | 打中 1 处定案落偏：第 25 条共用约束那一句不是原话；其余各处与原件或文件头一致 |
| 格二 修补引出的新问题 | 成立 | 第三版第 39、96 行裸写批号被判据库判中 6 处，写闸、⑰、doc-text 会拒；调查员变异验证命令二义；循环放行半句依据在代码注释 |
| 格三 能不能换上 | 推不出（diff 范围内不成立） | 这一轮改动里没找到登记之外的做法差别；diff 之外的原件句子没逐句对（攻方腿做） |
| 这一组能不能换上 | 改三条后可以 | 换上之前必须改：批号字面套反引号、第 11 行改回原话、调查员第 6 步补变异验证命令的 `<根>` |

## 落点整句核的几处

只对上开头几个字的落点，我读完整句、再读被指到的那一段才判：

- C14、A5、A6、A9、A10 里的「`.claude/agents/investigator.md` 第 6 步」：落在 `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33` 整句，第 6 步只有这一条，唯一。
- C5「提交之外」：指到的是 `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38` 的加粗条，不是标题；整条读完，写的是定案第 24 条的两种情形，对得上。
- C14、实现员第 1 步里「`.claude/main-agent.md`「写与验分两段」验证段」：标题在 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:40`，是标题行；验证段第 2 步在快照上交命名检查，整节读过，与句子一致。
- A13 里「「判决由主 agent 做，不由投票做」一节「攻方腿的装置把用户动作写死时……」那一条」：节标题 :30、被指的那一条 `research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:39` 在同一节内（下一个标题 :41），整句读过，说的是「把用户动作放开扫」，与第 3 步一致。
- C7、A16 里「文件头 ⑥ 那一段」：头里的 ⑥ 是 `.claude/hooks/bash-command-detector.sh:165` 起那一段，另有同名的代码注释 `.claude/hooks/bash-command-detector.sh:3805`（不是头）；放行半句要往代码注释里找，见 H3。

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数没复跑（命令在上面「要主 agent 跑的命令」）。
- 没逐句对原件里带「不」「只」「一律」「除外」「先」「才」的句子（格三那一半是攻方腿的活）；没核 diff 之外第二版就有的压缩。
- 没跑门禁 doc-text 的 private-values、out-of-project-names 两格，也没跑上游检出钩子的自证。
- 没读禁读清单里的别的腿产出与别的几题第三轮的东西。
