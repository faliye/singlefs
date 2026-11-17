# rules-compress-entry-r1 攻方腿报告

日期：2026-11-16。轮名 rules-compress-entry-r1，第一轮，前几轮判决：无。

没写模型，所以没有 `rerun.sh` 与模型目录；草稿目录里只有一份核指针的辅助脚本 `pointer_check.py`（只读，直接跑的，输出见「没打中的形状」）与一份 `CLAUDE.md` 的 diff。

合稿树的路径照派发后改的新名写：`dot-claude/` 与 `CLAUDE-merged.md`。四份被判文件的 sha256 与 `research/prompts/rules-compress-entry-r1-snapshot/merged-tree-sha256.txt` 逐份相同（量的，`sha256sum` 现跑）；两份原件与 `judged-sha256.txt` 相同（量的）。

## 各格判定一览

| 格 | 判定 | 打中几条 | 哪几条 |
|---|---|---|---|
| 一 丢失 | 打中 | 1 | 丢-1（主 agent 收事实表的验收句） |
| 二 二义 | 打中 | 4 | 义-1（读一次与压缩上下文）、义-2（谁不替谁登记）、义-3（整批重交的统一定义套到三种场合）、义-4（「只有两类」） |
| 三 常驻够不够 | 打中 | 3 | 驻-1（同义-1）、驻-2（拒算尾段只在技能末尾、审 diff 在写段）、驻-3（不派发只交任务的会话读不到 needs-attention 那几句） |
| 四 子 agent 读得到 | 没打中（只抽样一次，照规则不算「没打中」的结论） | 0 | 范围见「没打中的形状」 |
| 五 悄悄改了做法 | 打中 | 1 | 改-1（挪进技能的流程细则出了工具员写范围与四道检查的射程） |

攻击面四问各自的结论：

- 验证段六步与拒算三条各支结局的出路：没找到少一支的。第 1 步五支、第 2 步退 79 与四小时两种、第 3 步退 2 的五种原因与「不是 2」那一支、第 5 步三种修补、拒算三条逐支对过，合稿都有落点（常驻稿、技能或脚本文件头）。打中的是出路的措辞（义-2、义-3），不是少支。
- 15 条压成骨架后丢的限定：没找到丢了限定的；第 3、4 条的做法变化在矛盾总表里登了（入口-1、用户定-1、合稿-2）。
- 派发表各行尾巴抽走后读不全的：「一批阶段任务结束」一行（丢-1）、「改门禁、钩子…」一行（改-1）。
- 不读两份技能会做错的步骤：驻-1、驻-2、驻-3。

## 格一 丢失

### 丢-1 主 agent 收事实表时「过了 --check-facts」这一句没有落点

- 原件：`.claude/main-agent.md:121`：「交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts`」
- 合稿里找过的地方：常驻稿那一行 `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:95`：「`sweep` 写事实表 → 候选表按组切段」；技能同名条目 `research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:75`：「事实表那一趟，派发提示的输入给改动范围与做成的事」，这一条目共五项（75 到 79 行），没有 `--check-facts`。
- 全合稿树 `grep -rnF 'check-facts'`（量的）只命中 `dot-claude/agents/sweep.md` 三行，都是回扫员自己跑：`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:44`：「写完跑 `stale-candidates.py --check-facts 事实表 --base 基准」
- 为什么算丢：原件那一句写在主 agent 的派发表里，是主 agent 往下切候选表之前的条件；合稿里只剩子 agent 定义里的自查。第二段的 `--check-report` 在技能里留着（`dispatch/SKILL.md:77`），第一段的没留，两段不对称。
- 照合稿做会做成什么：回扫员交回说「过了」，主 agent 没有一句叫它核，直接切候选表；与常驻稿「agent 的结论句一律当线索」对不上，但那一句不点名这一步。
- 四句：不涉及臂；主 agent 当时看得到（报告里有 `--check-facts` 的输出末行）；满足格一「找不到落点」；没有跑前改法。
- 我提的改法（只在我的阅读上核过、被攻过零轮）：技能那一条目第一项补半句「交回的事实表过了 `stale-candidates.py --check-facts` 才切候选表」。

## 格二 二义

### 义-1 「整份读一次」与「压缩上下文接着做」放在一起，读得出两种做法

- 合稿：`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:5`：「到那一步之前整份读一次」，同一行另写「会话里第一次派发之前先整份读」；`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:35`：「会话过 40 万上下文就压缩上下文接着做」
- 原件里这些细则全在 `.claude/main-agent.md`，由 `CLAUDE.md` 的 `@` 常驻，压缩上下文之后还在；合稿把它们挪进按需读的技能，压缩之后技能正文不保证还在上下文里（推的，没量过压缩后留下什么）。
- 两种读法：甲，一个会话读一次就够，压缩之后「第一次派发」已经过去，不再读；乙，各条末尾的「做这一步之前先读」每到那一步都读。两种读法做法不同：甲在压缩之后照骨架做验证段第 3 步、读告警、读 harness 通知，手里没有各支出路。
- 各节的入口写法也不一致：第 1、4、8、11、15 条与验证段各步写「做这一步之前先读」，「派出去之后」「交回怎么读」「派发提示怎么写」三节只写「先读」，没说哪一步之前。
- 照合稿做会做成什么：长会话（用户说了可以一直跑的那种）压缩一次之后，第二批的验证段照常驻稿第 3 步那一句合进主工作区，退 2 的五种原因分不开。
- 四句：不涉及臂；主 agent 看不到「技能正文还在不在上下文里」这件事本身，只能靠规则写死；满足格二「两种读法且做法不同」，同时满足格三；没有跑前改法。
- 我提的改法（被攻过零轮）：第 5 行改成「每到那一步之前，技能正文不在上下文里就重读」，或在第 12 条补「压缩之后两份技能当没读过」。

### 义-2 「主 agent 也不替它登记」与同一节「主 agent 收的那几行做成拒算补丁目录」读起来相抵

- 原件：`.claude/main-agent.md:68`：「实现员不登记，也不替它登记」，主语没写。
- 合稿补了主语：`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:68`：「实现员不登记，主 agent 也不替它登记」；同一节上面 `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:66`：「主 agent 收的那几行做成一份只含拒算表的补丁目录」
- 两种读法：甲，主 agent 不往拒算表里登任何行（那收的行谁登）；乙，主 agent 不替实现员往哪一棵树里手写，登记只走拒算补丁目录。原件主语空着时这一句接在「不收，退给实现员」与「才收」之后，还读得出第三种：不收的那几条谁都不登。合稿把主语写死成主 agent、又紧接在「才收」后面，甲读法变得更顺。
- 矛盾总表里没有这一条（`grep -n '也不替' contradictions.md` 零命中，量的）。
- 照合稿做会做成什么：照甲读，收了的行不做拒算补丁目录，验证段第 2 步 `refused-test-registry.py list` 一直报「未登记」。
- 我提的改法（被攻过零轮）：改成「实现员不写拒算表，主 agent 也不往哪一棵树里手写，收的行只走拒算补丁目录」。

### 义-3 「整批重交」的统一定义套到三种场合，有两处对不上

- 合稿在技能开头立了一条统一定义：`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:14`：「第 1 步连同修补的补丁目录整批重交…第 2 步那几条在新留下的快照上重交，再回到原来那一步」，同一行写明各步各条说到这四个字都指这一套。
- 用到它的场合之一，第 1 步红支：`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:23`：「照它的出路整批重交」。原件同一句是 `.claude/main-agent.md:57`：「照它的出路整批重交」，那时没有统一定义，出路是脚本自己打的。套上统一定义之后读得出两种：甲，照脚本出路只重交第 1 步；乙，照定义还要把第 2 步那几条在新快照上重交、再回到第 1 步，而这时第 2 步还没做过。
- 场合之二，第 3 步核的那几条退非 0：`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:37`：「退非 0 不合，整批重交」。原件是 `.claude/main-agent.md:59`：「第 1 步整批重交（不带 `--item` 与 `--fingerprint-of`，键相同的照复用），第 2 步在新留下的快照上重验，再回到这一步」，没有「修补的补丁目录」。套上定义之后读得出：要先有一份修补才重交，还是原样那一批重交。
- 做法差别：第二处照「要有修补」读，主 agent 会为一次主工作区被别人改动去派实现员；第一处照乙读，会在第 1 步还红着的时候去交第 2 步。
- 四句：不涉及臂；主 agent 看得到自己在哪一步；满足格二；没有跑前改法。
- 我提的改法（被攻过零轮）：定义拆成两句，「有修补的把修补的补丁目录接在最后」「做过第 2 步的才在新快照上重交第 2 步」。

### 义-4 「主 agent 可以自己写的只有两类」

- 原件：`.claude/main-agent.md:7`：「定案后主 agent 自己写得清的小处 kb 改动可以自己写」，没有「只有」。
- 合稿：`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`：「主 agent 可以自己写的只有两类」
- 合稿树里与「只有两类」对不上的句子：`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:21`：「修它们本身就是这一轮的出口时，直接改，不先问」（定义、共用约束、三方流程与配套脚本）；同一份常驻稿职责一节前半句列的三方正文、判决也是主 agent 自己写的文件；拒算补丁目录、岔路单、调度记录、阶段同步记录同样。
- 两种读法：甲，「两类」只说 `crates/` 与 kb 这两处的例外，别处照各节；乙，除这两类之外主 agent 一个文件都不自己改，治理文件的一行小改也派工具员。做法不同，而且乙读法撞上改-1（技能不在工具员写范围里）。
- 矛盾总表里没有这一条（`grep -cF '只有两类' contradictions.md` 得 0，量的）。
- 我提的改法（被攻过零轮）：去掉「只有」，或写成「`crates/` 与 kb 里主 agent 可以自己写的只有两类」。

## 格三 常驻够不够

### 驻-1 压缩上下文之后技能不在、常驻稿没叫重读

同义-1，不重复。它是这一组里我认为最重的一条：原件靠 `@` 常驻保证的东西，合稿改成靠「读过一次」保证，而同一份常驻稿第 12 条规定了长会话压缩上下文。

### 驻-2 拒算一节的尾段只在技能末尾，用它的时刻在写段

- 原件：`.claude/main-agent.md:72`：「这份清单只取验证包不带特性那一趟的测试函数」，后半句叫主 agent 审 diff 时照实现员定义第 3c 步看挂特性的测试文件、产品 GPU 测试包与别的包里的新测试有没有读活树。
- 合稿：只在 `research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:58`：「审 diff 时照 `.claude/agents/implementation-writer.md` 第 3c 步看有没有读活树」，是技能最后一段，不属于编号的三条。
- 常驻稿的入口：整份读的时机是「开始验证段之前」（第 5 行）；写段里的入口只有 `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:68`：「做这一步之前先读 `.claude/skills/merge-verify/SKILL.md`」后面点名的是第 1 条。审 diff 发生在写段收补丁时与派发表「主 agent 审 diff」那一步，这两处都没有指到尾段。
- 照合稿做会做成什么：写段照第 1 条读技能的第 1 条，判完报告「拒算的测试」一节就过；挂特性的测试文件里读活树、又没列进报告的新测试没人看，`refused-test-registry.py list` 的清单也不含它们，一直到提交都不红（清单不含它们这一句是原件与技能自己写的，量的；「一直不红」是推的）。
- 我提的改法（被攻过零轮）：尾段并进技能第 1 条，或常驻稿第 1 条加半句「清单罩不到的三类照技能末段在审 diff 时看」。

### 驻-3 不派子 agent、只交任务的会话，读不到自己交的任务那几句

- 合稿把这几句放在派发技能的「派出去之后」：`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:41`：「时任务还在、不是结局」（`task wait` 以 `needs-attention` 退出时怎么办），第 39 行是服务健康处置后用 `task-server/control check` 回读。
- 常驻稿留的是 `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:74`：「主 agent 自己交的任务后台起 `task-server/task wait <号>`、结束本轮等叫醒」，细则的入口挂在「派出去之后」一节，整份读的触发条件是「会话里第一次派发之前」。
- 一个会话只做提交（暂存、六道门禁、整轮门禁），一个子 agent 都不派，这个触发条件永远不成立；`needs-attention` 全合稿树只在这份技能与 `long-run-split.md` 里出现（`grep -rnF needs-attention`，量的），共用约束「长活交回主 agent」那一条里没有（原件 `.claude/agent-common.md` 第 79 到 81 行同样没有，量的）。
- 照合稿做会做成什么：`task wait` 退非 0，主 agent 当成结局，重交或记红。卡坏那一支不受影响，`verification.md` 常驻稿里另有一句。
- 强度：中偏低。常驻稿那一节有「先读」，只是挂的节名与触发句都说的是派发。
- 我提的改法（被攻过零轮）：第 5 行的触发改成「会话里第一次派发或第一次自己交任务之前」。

## 格四 子 agent 读得到

没打中，只查了一遍：合稿树里定义与共用约束指到 `.claude/main-agent.md` 的五处（`dot-claude/agent-common.md` 第 51、64、109 行，`dot-claude/agents/tooling-writer.md` 第 15、40 行）都只写「主 agent 照哪一节做」或写范围，不要子 agent 自己去读；指到两份新技能的文件只有常驻稿与两份技能自己（`grep -rl`，量的）。别的定义逐份的指针没在这一轮的攻击面里，没查。

## 格五 悄悄改了做法

### 改-1 挪进技能的流程细则，出了工具员的写范围，也出了四道检查的射程

- 原件里验证段各支出路、看门狗告警处置、派发表各行尾巴都在 `.claude/main-agent.md`。合稿把它们挪进 `.claude/skills/merge-verify/SKILL.md` 与 `.claude/skills/dispatch/SKILL.md`。
- 谁能改：合稿的工具员定义 `research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:40`：「第二种活另加 `.claude/agents/*.md`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/*.md`」，没有技能；写范围表 `.claude/hooks/agent-write-scope.tsv` 里 `grep -n skills` 零命中（量的），工具员那几行是第 33 到 45 行。
- 谁在判：`.claude/gate.d/doc-process-records.sh:2272`：「agent-common\.md|main-agent\.md」（agent-def-adversarial-review 格只认这三处，改技能时这一格退 77 无对象）；`.claude/gate.d/code-tooling.sh:4153`：「judged_files = definitions + [path for path in (".claude/agent-common.md", ".claude/main-agent.md")」（规则行抄进定义的那一格）；`.claude/gate.d/lib-lag-baseline.py:235`：「GOVERNED_FILES = (".claude/agent-common.md", ".claude/main-agent.md")」；`.claude/gate.d/doc-text.sh:1953`：「CODE_FILES = (".claude/main-agent.md", ".claude/agent-common.md")」。四处都不含技能目录。
- 合稿的派发表仍写 `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:99`：「这一批改动（门禁、钩子、脚本、定义、共用约束、规则、技能、两个服务都算）」走三方，派的是工具员。
- 为什么算改了做法：同一段文字，压缩前判决打中之后派工具员改得了、改完没走三方有一格会红；压缩后工具员写不进去（写闸拒，推的，没跑），主 agent 自己改又撞义-4 的乙读法，改完没走三方没有哪一格红。技能在这几格的射程之外是压缩前就有的现状（原件第 125 行自己写了「其余几类那一格还没判」），新的是有多少流程细则落进了这个射程之外。
- 矛盾总表、引用处清单里没登：`contradictions.md` 里 `写范围` 与技能同行的命中 0 处（量的）；`references.md` 第 245 行只说三份新技能不用登 read-once，第 261 行只列新建的四份。
- 照合稿做会做成什么：第一条打中验证段出路的判决，主 agent 照派发表派工具员改技能，被写闸拒回；出路照共用约束是「修那道机制或登进它的常设表」，即改写范围表与工具员定义，这是合稿没带的一处联动改动。
- 四句：不涉及臂；主 agent 派发之前看得到写范围表；满足格五「做法不同而两张表都没登」；没有跑前改法。
- 我提的改法（被攻过零轮，「推的」，没实现没跑）：工具员写范围第二种活加 `.claude/skills/*/SKILL.md`（定义与 `agent-write-scope.tsv` 两处同改，门禁 code-tooling 的 agent-write-scope 格要两向相等）；四道检查要不要把技能收进射程，归治理文件那一批另判。

## 没打中的形状

逐句对过的范围（原件一侧）：`.claude/main-agent.md` 全文 129 行；`CLAUDE.md` 只对了 `diff` 出的 117 行差异，没差异的行没再读。合稿一侧：`dot-claude/main-agent.md` 107 行、两份技能 58 行与 91 行全文；`CLAUDE-merged.md` 全文；`contradictions.md` 只读了表头、入口各行与用户定案一节。

- 验证段第 1 步五支：绿、退 3、退 1、退 2、退 75 都在技能第 21 到 25 行。原件括号里几句脚本行为（退 3 时别的步绿、退 75 时没有那两段、验证包目标带 `--release`、三样都没有时报本次未跑）技能里没抄，脚本文件头今天都有：`scripts/gate/merge-and-verify.sh` 第 59、168、210、212、227 行（`awk` 按行号现取，量的）。照正文「以脚本文件头为准」不算丢。
- 第 1 步之外的结局（被停、卡坏、四小时、needs-attention）的出路原件也没单列，不算压缩丢的；删快照那三种情形在（矛盾总表入口-2）。
- 第 2 步：五样要交的、退 79 与四小时两种、有红怎么修，都在技能第 27 到 35 行。
- 第 3 步：核的那几条、退 2 的五种原因、不是 2 的那一支、`--read-outcome`、连着三趟、跨轮停下，都在技能第 36 到 41 行。
- 第 5 步三种修补与改钉值那一句在技能第 43 到 47 行；第 4、6 步常驻稿全文。
- 拒算三条：第 1 条的判法、行的写法、拷三份的做法，第 2 条两种行各怎么办，第 3 条退 77，都在。
- 15 条：第 2、5、6、7、12、14 条常驻稿全文；第 13 条丢的括号「作废在本次未跑栏」在 `dot-claude/skills/crash-test/commit-stages.md` 第 53 行有；第 9 条指到的共用约束那一条在 `dot-claude/agent-common.md` 第 52 行；第 10 条「变更史条目的形态」在 `dot-claude/agents/kb-scribe.md` 第 21 行；第 10 条「合入后验证不跑门禁阶段」字面没有了，「扫仓的门禁只在提交前」在 `dot-claude/agent-common.md` 第 108 行，判为没丢做法。
- 派发表：六行「细则在技能同名条目」与技能六个条目一一对上；「暂存之后」一行抽走的 `--full-gate`、`separate-run=`、整轮全绿才前移引用，分别在 `commit-stages.md` 第 20 行、`verification-reference.md` 第 146 行、`staged-gate.md` 第 17 行有字面命中；六道之间谁并行谁串行那一句，用「串行」在合稿树里搜不到落点，没再换词搜，记「没查完」。
- 指针：四份被判文件里「`文件`「节」」形态的指针用 `pointer_check.py` 逐个核（文件在不在、节名在目标里有没有），输出只有一行「hit-but-not-heading」，是 `implementation-workflow.md` 表里「实现员交回前」那一行，指得到。写成「一节「…」那一条」的指针另用 `grep -nF` 手查了共用约束的八个条目名与实现员定义、写回员定义、三方技能、提交阶段技能的节名，都在。
- 钩子与脚本打出的「照 main-agent.md 某节某步」（`grep -rnF` 读到的几处，没数；`runner-dispatch-guard.sh`、`completion-cleanup-guard.sh`、`merge-and-verify.sh`、`doc-process-records.sh`），节名与条号在合稿里都还在；`dead-path-kept.tsv` 第 45、46 行载体要改，`references.md` 第 236、237 行已登。

这些「没找到」都只读了一遍，照规则一次不算「没打中」的结论。

## 我认为最脆弱的三处

1. 义-1 与驻-1：按需读加上压缩上下文。原件靠常驻保证的，合稿没有等价的保证。
2. 改-1：流程细则搬到了没人有写范围、没有格判的文件类里。
3. 义-3：「整批重交」的统一定义。验证段里它出现得最多，定义一处读歪，三步的出路一起歪。

## 要主 agent 跑的命令

无。这条腿没写模型、没有要经服务跑的东西。改-1 里「写闸拒」是按写范围表推的，要坐实可以在下一次真派工具员改技能时看，不必专门跑。

## 这条腿自己的限度

- 全部结论来自读文字，没有任何一条经过执行；标「推的」的三处（压缩后上下文留什么、写闸拒、读活树的测试一直不红）没量。
- 原件一侧只判了两份；别的组挪进这四份的句子（如果有）没有反向核。
- `contradictions.md` 第 7 节的 85 条没逐条读，只按关键词搜；说「没登」的四处各有一条 `grep` 计数。
- 派发提示说用户定案十六条，矛盾总表「用户定」一节是十二条加主 agent 处置六条；我照表里的读，调度记录的编号没去对。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没交任何任务。
- 派发提示点名的四份规则（`three-way-inference.md`、`evidence-discipline.md`、`pushback-discipline-by-review.md`、`command-safety.md`）仓里的原件没有整份读：读的是会话里送到的合稿树版本的前两份与 `machine-first.md` 里并进去的那几节，`command-safety.md` 没读。照定义这算没做全，照实写在这里。
- 没建仓副本、编译目录；草稿目录里只有 `pointer_check.py` 与 `claude-md.diff` 两个小文件。
