# rules-compress-definitions-r1 攻方腿报告

日期：2026-11-16。轮名 rules-compress-definitions-r1，第一轮，前几轮判决：无。这条腿没写模型，没有复跑脚本；对照用的两份输出在草稿目录 `~/.local/share/singlefs/work/rules-compress-definitions-r1-opus/`（`pointer-check.txt`、`sentence-diff.txt`），由只读文件的内联 python 生成。

被判文件对快照：`sha256sum -c research/prompts/rules-compress-definitions-r1-snapshot/judged-sha256.txt` 15 行、对不上 0 行（量的）；合稿树在 `research/prompts/rules-compress-r1-merged/` 下对 `merged-tree-sha256.txt` 对不上 0 行（量的）。

## 各格判定一览

| 编号 | 格 | 打在哪 | 判定 | 轻重 |
|---|---|---|---|---|
| 攻-1 | 用户定案第 6 条的落法（攻击面一：留的合规写法够不够照做） | 共用约束「样本进程逐个按进程号停」与规则同名节叫人写的停法，正是钩子 ⑥ 与门禁 research-gate-lint 今天判红的写法 | 打中（读钩子文件头与自证样例得出，没跑；命令在「要主 agent 跑的命令」） | 重 |
| 攻-2 | 二 二义，连带四 | 「规则怎么读」第一条改成「点到就整份读」，谁点到算数读得出两种 | 打中 | 中 |
| 攻-3 | 四 子 agent 读得到 | 三条三方腿的样本进程停法删成指共用约束，共用约束再指一份三条腿都没被点名的规则，细则只在那里 | 打中（取决于攻-2 取哪种读法） | 中 |
| 攻-4 | 一 丢失 | 「找不到历史数据去 git log」那一条：拿旧数支撑新结论之前先重跑，落点指到的那一条写的是另一句 | 打中（弱：隔两跳找得到） | 轻 |
| 攻-5 | 一 丢失 | 完工清理闸那一项丢了「写成交接的副本子 agent 交回时不列」 | 打中 | 轻到中 |
| 攻-6 | 一 丢失 | ⑬ 的出路丢了；派发闸那一项的三种「…已判」行与另几条合规行全树找不到 | 打中（主 agent 一侧，拒绝说明会补） | 轻 |
| 攻-7 | 二 二义 | 实现员第 1d 步「照说明给的写」的「说明」失了所指；共用约束「`scripts/kb/` 下的定点写入脚本」没列是哪几份 | 弱打中（两处都偏向被拒，不偏向做错） | 轻 |
| — | 三 常驻够不够 | 这一组文件本身就是常驻件，没有挪进技能的细则 | 没找到反例 | — |
| — | 五 悄悄改了做法 | 共用约束逐节、五份定义逐句、其余九份逐句差分 | 没找到矛盾总表与用户定案之外的改动 | — |

对过的范围：共用约束原件 128 行对合稿 122 行逐节逐条（「执行前拒绝的写法」各项逐项）；14 份定义全部做了按句切分的差分（`sentence-diff.txt`）。implementation-writer、experiment-runner 两份把原件与合稿整份并排逐句读了一遍；experiment-designer、tooling-writer、kb-scribe 与其余九份读的是差分出的全部句子（差分之外逐字相同的句子没再读）。

机械核过的两件（量的，命令是内联 python，只读文件）：
- 合稿共用约束与 14 份定义里带完整路径的「文件 + 小节」指针，文件在合稿树（或上游、仓里）都找得到，小节标题除两处外都逐字命中：`agent-common.md` 第 24 行的「旧证据是参考，不是依据」是那一节里的一条、不是标题；investigator 第 16 行的「开工先读：」是实现员定义里的一行、不是标题。两处都指得到。
- 14 份合稿定义里「共用约束「…」」引的条目名，逐个在合稿共用约束里按字面找，全部命中。

## 攻击面一：下沉之后留的合规写法够不够照做

### 攻-1 样本进程的停法：合稿叫人写的，正是钩子 ⑥ 与门禁今天判红的（重）

不攻用户定案第 6 条该不该，攻它落下来子 agent 能不能照做。

原件（压缩前）给的写法是按组停，钩子放行它：
- `.claude/agent-common.md:72`：「超时与结束时在脚本里对整组发信号」
- `.claude/hooks/bash-command-detector.sh:184`：「对自己用新会话起的那一个子进程的进程组发信号（上游」

合稿改成逐个按进程号停，并要连子孙一起停：
- `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`：「超时与结束时逐个按进程号发信号，连同它再起的子孙进程，不对进程组发、不按名字或 cgroup 挑」
- `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:9`：「从最底层起逐个先 TERM、过宽限再 KILL」

一条样本起了几个子孙事先不知道，「先列出整棵、再逐个发」只能写成循环（shell 的 `for`，python 的循环里 `os.kill`），而且列进程要读 `/proc` 或 `ps`。钩子文件头今天把这两样都列在拒的写法里，同一份判定还被门禁拿去扫脚本目录：
- `.claude/hooks/bash-command-detector.sh:170`：「在 for / while / until 循环里逐个发（」
- `.claude/hooks/bash-command-detector.sh:182`：「在循环或推导式里调 os.kill、同一个函数里读 cgroup.procs 或 /proc 再 os.kill」
- `.claude/hooks/bash-command-detector.sh:192`：「同一份判定由门禁 code-tooling 的 research-gate-lint 格拿去扫」
- `.claude/hooks/bash-command-detector.sh:4009`：「循环里逐个发：这是批量停，不是点名停一个」
- `.claude/hooks/bash-command-detector.sh:5927`：「循环里 kill 报在第 3 行」

最后一行是钩子自证里扫脚本那一路的期望：样例 `loop.sh`（`for pid in …; do kill -TERM "$pid"; done`）与 `bad.py`（循环里 `os.kill`）是要报红的样例（`.claude/hooks/bash-command-detector.sh:5902`、`.claude/hooks/bash-command-detector.sh:5914`）。

子 agent 照合稿做会做成什么：工具员照合稿 `tooling-writer.md` 第 5 步与共用约束这一条，在 `scripts/`、`.claude/gate.d/`、`.claude/hooks/`、`.claude/scripts/` 下写自证或装置的停法，写出来的就是循环里逐个 `kill`；提交时门禁 code-tooling 的 research-gate-lint 格报红。能过那一格的多进程停法今天只有按组停（文件头 :184 放行的那一种）与 own-scope 标注（只给自己开的内存上限 scope），前一种合稿禁了，后一种不适用。两头都走不通，留下的「合规写法」在脚本目录里照做不了。三方腿的模型目录在 `research/prompts/` 下，不在那一格扫的四个目录里，不受这一条影响（按 :192 列的目录推的）。

合稿这一侧登了什么：`research/prompts/rules-compress-r1-merged/references.md:218`：「新判据下这一放行要撤」与 `research/prompts/rules-compress-r1-merged/merge-result.md:161`：「换上之后规则与代码、钩子不一致」，两处说的都是钩子还放行按组停（钩子比规则松）这一半；钩子把规则要的写法判红（钩子比规则严）这一半，矛盾总表「用户定-6」、`references.md`「四」与 `merge-result.md`「⑥」第 1 条里都没提（`grep -n 'research-gate-lint\|scan-scripts\|循环里逐个' ` 这三份，命中 0 行，量的）。

答四句：
- 分不分辨臂：分辨。原件的写法（按组停）过门禁，合稿的写法（循环逐个停）不过；只有「一条样本只起一个进程」的脚本两种都过。
- 被判的系统当时看不看得到：看得到，钩子文件头 :170、:182、:192 就是合稿自己说的「逐条以它的文件头为准」的那份文件。
- 满足判据字面的哪一句：正文「共用约束里说钩子拒什么的句子以各钩子文件头今天的原文为准」与攻方分工「留的合规写法够不够子 agent 照做」。
- 没有跑前条款给的改法。

我提的改法（只在纸面上推过、被攻过零轮）：两条里选一条，由主 agent 定。其一，换上这一节与改钩子 ⑥、research-gate-lint 的循环判法放在同一批（给「对自己起的样本列出的子孙逐个发」一种认得出的放行形态）；其二，换上之前在共用约束这一条写明「钩子 ⑥ 改完之前，脚本目录里的停法照旧按组停」。

### 攻-5 完工清理闸：写成「交接」的副本不列，这一句丢了（轻到中）

- 原件 `.claude/agent-common.md:103`：「只认主 agent 的声明、子 agent 写了照列」，同一个括号里接着写子 agent 另不列写成交接的。
- 钩子文件头 `.claude/hooks/completion-cleanup-guard.sh:100`：「子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:100`：「只认主 agent 的声明；编译目录声明了也照列」，不列的只写了报告、补丁、备份三种。

合稿第 36 行（「用完就删」那一条）仍叫子 agent 给交接副本写 `.deliverables` 一行，但「写了这一行闸就不拦」只在原件那半句里。照合稿读：交回之前要删掉自己建的副本、删不掉的写「交主 agent 删」；交接副本既不该删、合稿又没说它不列，子 agent 会在「删掉它」（主 agent 接着跑的命令没树可跑）与「写一行交主 agent 删」（那一行只许删不掉时用）之间选一个，两样都不对。实现员今后一律交补丁、每次都有交接副本（证红命令的 `--cwd <副本>`），这一句每次都用得到。

### 攻-6 出路与合规行（轻，多在主 agent 一侧）

- ⑬ 的出路：原件 `.claude/agent-common.md:95`：「要另起一个 agent 的活写进报告交回主 agent」；合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:87` 只留拒什么（「项目子 agent 在 Bash 里起 `claude` 会话，经包装」那一行），没有出路；同文件第 8 行也只写不许。用户定案第 7 条留的是合规写法，这一条的合规写法没留。
- 派发闸一项：原件 `.claude/agent-common.md:105` 里主 agent 写派发提示要用的几种行，「一次性的活派 clerk」「准入已判：<理由>」「写者锁已判：<理由>」，合稿那一项（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:104`）压成一句；`grep -rlF` 在整棵合稿树里找「通用已判」「准入已判」「写者锁已判」各 0 份（量的），「自证在哪跑」只剩共用约束「门禁」一节子 agent 那一半（主 agent 那一半「不许点工作根」找不到）。合稿第 73 行还留着拿「…已判」举例的那半句（「只放过这一次」），例子指的东西在合稿树里已经没有定义。主 agent 被闸拒时拒绝说明会给写法，不至于做错，只是多拒一次。

## 攻击面二：删成「照共用约束〈条目〉」的地方，条目里是不是逐条写着

逐个对过的（定义里删掉的句子 → 共用约束合稿里的落点，都找到了）：范围声明写在 submit 前面、不套包装、不写内存上限与线程（第 56 行）；主 agent 跑完新派同类 agent 带「承接：」、读 `task log` 接着做（第 55、57 行）；一行一条、不串接、不写 `echo $? >`（第 57 行）；上下文多大都不交回（第 68 行）；全文写进报告文件、交回只写结论与路径与 `sha256sum`、不超过 4000 字（第 121 行）；排他新建（第 32 行）；分段不超过 150 行（第 33 行）；建仓副本（第 35 行）；并行件收退出码的写法（第 78 行 ④）；同目录临时文件再 `mv`（第 81 行 ⑦）；点名测试目标的范围声明模板（第 60 行）；「编号（简称）」（第 42 行）。实现员第 1d 步删掉的缩域变量怎么挑、对着哪棵树判、全域理由怎么写，在第 61 到 63 行逐条都在。

### 攻-3 三条三方腿的样本进程停法：两跳之后落在没被点名的规则里（中）

- 原件三条腿各自写全了做法，例 `.claude/agents/three-way-attack.md:39`：「在脚本里把每条样本放进它自己的进程组」
- 合稿三条腿都只剩一句指针：`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:39`、`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:31`、`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:30`：「起样本进程照共用约束」
- 共用约束那一条给了判据与禁什么，怎么列整棵、从哪头停、先发什么后发什么再指出去：`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`：「写法照 `.claude/rules/command-safety-by-review.md`」

`command-safety-by-review.md` 在 14 份合稿定义里只有 tooling-writer 与 experiment-runner 提到（`grep -c`，量的；执行员点的是另两节），三条腿的「开工先读：」都没有它。腿读不读这份规则取决于攻-2 的读法：照「只有开工先读点名的才读」，腿拿到的只有共用约束那一句，「连同它再起的子孙进程」怎么列、孙进程拿着管道写端这类坑（规则第 9 行）读不到；照「正文里点到就读」，读得到。这一处原件是自足的，压缩之后变成依赖一句有两种读法的话。

### 第四格的另一处（弱，写明不算打中的理由）

共用约束合稿第 60 行把原件内联的四条带 `--item` 的整条命令删成指针：原件 `.claude/agent-common.md:68`：「带 `--item` 的四条照 `.claude/agents/implementation-writer.md` 第 1d 步」后面跟着四条命令原文；合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:60`：「整条照 `.claude/agents/implementation-writer.md`」。实现员定义不是规则文件，别的类型没被点名去读它。但 14 份合稿定义里写到 `--item` 的只有 implementation-writer（13 行）与 investigator（2 行，它自己点了实现员第 1d 步），别的类型的步骤里没有要写这四条的地方（`grep -c -- '--item'`，量的）。所以今天没有哪一类会因为这一处做错；哪天给别的类型加带 `--item` 的步骤，这个指针就成了第四格的洞。

## 攻击面三：每份定义的输入、步骤、交付少了哪一条

### 攻-2 「规则怎么读」第一条：谁点到算数（中）

- 原件 `.claude/agent-common.md:22`：「一律整份读，不受读入大小的限制」，后半句限定的是开工先读点名小节时照样读全。说的是怎么读，没说什么时候读。
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`：「点到就整份读，不受读入大小的限制」

「点到」读得出两种：
- 读法甲：定义「开工先读：」一行点到的。那么共用约束正文里「照某规则某节」的指针（第 24、52、67、118 行这类）不算叫人去读，攻-3、攻-4 那几处的细则子 agent 读不到。
- 读法乙：共用约束与定义正文里任何一处点到的。那么每个子 agent 开工都要整份读共用约束点到的 9 份规则文件（`evidence-discipline.md`、`verification.md`、`three-way-inference.md`、`implementation-workflow.md`、`path-moves.md`、`command-safety-by-review.md` 与上游三份，`grep -o … | sort | uniq -c` 数的，量的），与 clerk 合稿第 16 行「规则只读三样，别的不读」撞（clerk 的定义写了这一句，靠「与定义冲突时以定义为准」解掉；别的定义没写「别的不读」）。

两种读法做法不同（只读定义「开工先读：」点的那几份，还是另加共用约束点到的 9 份），满足第二格「给得出两种读法且两种读法做法不同」。同一条里合稿第 28、29 行用的是「开工另读」，那两处没有二义；第 24、52、67 行用的是「照」「写法照」，有。

### 攻-4 旧数怎么用（轻）

- 原件 `.claude/agent-common.md:35`：「拿它支撑新结论之前先重新跑一遍」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:24`：「读到的是当时的数、不是结论，怎么用照」后面指 `evidence-discipline.md` 的一条。
- 指到的那一条 `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:61`：「对早先的结论有疑问，重新验证」，并写「不去翻旧证据」。
- 原句的意思要再跳一次才有：`.claude/singlefs-ai-sop/rules/kb-discipline.md:39`：「想拿它支撑新结论，先复跑一遍」

指到的那一条管的是「对旧结论起疑」，原句管的是「拿旧数撑新结论」，触发条件不同；而且共用约束这一条刚教人去 `git log` 里翻旧产物，指过去的那一条开头就是不去翻旧证据，子 agent 读到会停一下。按第一格「落点只剩指针而指到的地方没有那句话」算打中；隔一跳在上游规则里有，所以轻。改法（纸面，被攻过零轮）：合稿第 24 行把「拿它支撑新结论之前先重跑一遍」这半句留在原地。

### 攻-7 两处措辞（轻）

- 实现员第 1d 步：原件 `.claude/agents/implementation-writer.md:34`：「出路照它说明里的两条」，「它」是崩溃放量入口；合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:42`：「照说明给的写」，前面的「它说明」删了，「说明」成了没有所指的词（是入口退 79 时打的说明，还是共用约束第 64 行指的文件头用法，两处给的 `<n>` 未必同一个来源）。
- 共用约束可直接跑的写入工具：原件 `.claude/agent-common.md:66`：「与定义点名的写入工具 `scripts/kb/replace-once.py`」起逐个列名；合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:58`：「下的定点写入脚本」。`scripts/kb/` 下另有 `sweep-term.py`、`history-consolidate.py`、`doc-consolidate.py` 这类也写文件的脚本，不在放行名单里；读宽了会被项目代码闸拒一次，不会写坏东西。

### 逐句对过、没找到少一条的

- implementation-writer：输入 7 项、步骤 1 到 7a、写范围、产出、没做什么，逐项都在。删掉的只有三类：指到 `main-agent.md` 的括注（主 agent 一侧；抽查两处：「走不到的理由」二次查实在合稿 `skills/three-way/SKILL.md` 第 82 行，回扫报告「交实现员」的行怎么派在 `skills/dispatch/SKILL.md` 第 78 行，其余没逐个追）、与共用约束重复的拼法、「没写的在主工作区改」那一支（用户定案第 1 条）。第 4a 步删了主 agent 对着那一节看的半句，那是主 agent 的动作。
- experiment-runner：输入 5 项、步骤 1 到 8、写范围、产出三节逐项都在；第 2 步拆成了列表，四条只对入库装置生效的 ①②③④ 都在，② 改成进代码轮三方（矛盾总表「定义-1」登了）；第 2b 步的上下文吃紧那一句还在，与共用约束第 68 行的例外对得上。
- experiment-designer：差分出的全是表格排版、分段与标点；实义的只有「未定项：」写在哪（矛盾总表「定义-10」）与 NOT_RUN 的写法（「定义-3」）。
- tooling-writer：删掉的三句（新建排他、上下文不交回、交回只写结论）共用约束都有；第 5 步照用户定案第 6 条改写（见攻-1）；多了一句「写门禁、改算键的代码时另读」参考件，是加读、不改做法。
- kb-scribe：`changelog-format.md` 的形态括注删成指「变更史：条目、日期、变更清单三层」，那一节里同一天只一块、缩进两格、不写与标题相同的日期、点名几条决策的写法四样都在（合稿 `changelog-format.md` 第 22 到 26 行，`grep -n` 核的）；写范围里不变量清单那两条模式还在第 42 行。

## 没打中的形状

- 第五格：共用约束逐节、定义按句差分里凡是做法有变的句子，逐个回矛盾总表找，都登了（「共用-1」到「共用-10」「定义-1」到「定义-11」与用户定案第 1、5、6、7、10、12 条）。没找到没登的。
- 指针断链：合稿这 15 份里带完整路径的文件与小节指针全部指得到（见一览下面那两件机械核对）；被合并掉的 8 份规则（`verify-before-claiming.md`、`pushback-discipline-by-review.md`、`engineering-philosophy.md`、`design-doc-discipline-by-review.md` 等）在定义里的指针都已改指并入后的文件与节，节名逐字在。
- 「照共用约束〈条目〉」的条目名：全部按字面找得到，没有指到改了名或拆没了的条目。
- 自证套临时服务的整条写法：合稿第 56 行写出的那一条与 `.claude/hooks/selftest-temporary-service.tsv` 文件头第 5 行逐字一致。
- 用户定案第 12 条补的七类（⑨ ⑪ ⑯ ⑰ ⑱ 与写闸第六道、第十道）：钩子文件头里这几个编号都在，合稿每类一行，出路都留了。交回正文闸文件头 ① 到 ⑧ 八样，合稿那一项八样都提到。
- 三方腿「主 agent 跑完新派同立场的腿」删成共用约束的「同类 agent」：三种立场是三个类型，同类就是同立场，没有二义。
- sweep 第 6、11 步删了「只判不改」：定义开头第 15 行与写范围第 55 行还写着不改，没丢。
- mutation-triage 的结果行怎么数、九栏：删成指 `mutation-sampling.md`，那份规则第 58 行各符号与九栏都在；输入两项并成一项后「缺它不开工」还在。
- frontmatter（tools、model、effort、required-inputs、leg）：14 份原件与合稿各取这几行比过，内容相同（只比了这几种键）。

## 我认为最脆弱的三处

1. 攻-1：用户定案第 6 条换上之后，脚本目录里「一条样本起多个进程」的停法没有一种既合规则又过 research-gate-lint。规则与钩子要同批换，或换上前写明过渡期怎么办。
2. 攻-2：「点到就整份读」。合稿把大量细则换成了「照某规则某节」，这些指针算不算叫人去读，全压在这四个字上；攻-3、攻-4 与今后每一处新指针的第四格都跟着它走。
3. 攻-5：交接副本在完工清理闸里列不列。实现员一律交补丁之后每次交回都带交接副本，共用约束里说它不被拦的那半句没了。

## 要主 agent 跑的命令

只有一条，把攻-1 从「读的」变成「量的」。草稿目录里放了两份照合稿写法写的样本脚本（不执行，只给扫）：`scan-sample/scripts/stop-sample-tree.sh`（sha256 开头 9c504a66）与 `scan-sample/scripts/stop_sample_tree.py`（sha256 开头 f2371d74）。

- 命令原样（`<草稿>` 填这条腿草稿目录的绝对路径，即工作根下的 `rules-compress-definitions-r1-opus`）：
  `task-server/task submit -- bash .claude/hooks/bash-command-detector.sh --scan-scripts <草稿>/scan-sample <草稿>/scan-sample/scripts`
- 在哪个目录起：仓根。
- 跑完看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=`。`exit=1` 且日志里点名 `scripts/stop-sample-tree.sh` 的循环那一行与 `scripts/stop_sample_tree.py` 里循环 `os.kill` 那一行，攻-1 成立；`exit=0`，攻-1 在这两种写法上被推翻。范围声明闸若要声明，照它的拒绝说明补。

## 这条腿自己的限度与没做什么

- 攻-1 的「会判红」是读钩子文件头与自证样例推的，没跑；research-gate-lint 那一格此刻实际扫哪几个目录取的是文件头 :192 的说法，没读门禁脚本核。
- 主 agent 一侧挪走的句子（指到 `main-agent.md` 的括注、派发闸一项）只按关键词在合稿树里找过落点，没逐句核那几份技能；它们归别的题。
- experiment-designer、tooling-writer、kb-scribe 与其余九份定义，差分之外逐字相同的句子没再读；差分按「。；：」切句，同一句里只动了一两个字的会整句列出、已读，但跨句挪动的句子靠人眼配对，可能漏配。
- 没判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务；自己提的两处改法只在纸面上推过、被攻过零轮。
- 没读禁读清单里的文件。写报告时有一条想参考别的报告里路径写法的 `grep` 范围过宽（会扫到别的腿的产出），执行前被权限层拒了，没有读到任何内容，之后没再试。

什么现象会推翻这份报告的结论：上面那条命令退 0（攻-1）；合稿里另有一句写明正文指针算不算「点到」（攻-2、攻-3）；合稿别处写着交接副本交回时不列（攻-5，共用约束合稿里含「交接」的只有第 36、57 两行，`grep -n` 数的，都没有这一句）。
