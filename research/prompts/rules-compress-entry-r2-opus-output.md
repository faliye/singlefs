# rules-compress-entry-r2 攻方腿报告

日期：2026-11-16。轮名 rules-compress-entry-r2，第二轮；前几轮判决 `research/prompts/rules-compress-entry-r1-main-verification.md`。

没写模型，没有模型目录与 `rerun.sh`；全部结论来自读文字与只读命令（`grep -nF`、`sha256sum`、`awk`），没有交任何任务。草稿目录 `~/.local/share/singlefs/work/rules-compress-entry-r2-opus` 已认领，里面没有建仓副本与编译目录。

开工核过（量的，`sha256sum -c` 现跑）：两份原件与 `research/prompts/rules-compress-entry-r2-snapshot/judged-sha256.txt` 逐份相同；合稿树全部文件与 `merged-tree-sha256.txt` 逐份相同（`grep -v ': OK$'` 无输出）。

被判对象只看第二版合稿树里四份：`CLAUDE-merged.md`、`dot-claude/main-agent.md`、`dot-claude/skills/merge-verify/SKILL.md`、`dot-claude/skills/dispatch/SKILL.md`；第二版相对第一版在这四份上的改动取自 `diff-from-r1.patch` 第 1-12、444-505、1063-1111 行（量的，`grep -n '^diff '` 定位）。第一轮两份腿报告与判决读过，第一轮已打中、已判的不再报。

## 各格判定一览

| 格 | 判定 | 打中几条 | 哪几条 |
|---|---|---|---|
| 一 修补对不对 | 打中 | 2 | 一-1（「只有两类」没照判决去掉，改成限定在 `crates/` 与 kb，与共用约束「简单任务自己做」相抵、做法变窄）；一-2（压缩后重读只罩两件事，「派出去之后」「交回怎么读」两节没改，判决写的是每次做那一步之前） |
| 二 修补引出的新问题 | 打中 | 4 组 | 二-1（整批重交定义新加的「第 1 步自己红了、退 2 的」读得出指令，撞第 1 步退 1、退 2 两支）；二-2（调查员例外写进写段第 2 条，写段第 6 条「主工作区里没有打了而没验过的补丁」不再成立）；二-3（拒算第 1 条「主 agent 也不往哪一棵树里手写」与技能「主 agent 在主工作区直接删」相抵，且与同节开头重复）；二-4（同一条判据住两处，五处） |
| 三 第一轮没看到的 | 打中 | 2（都轻） | 三-1（版本戳红了「先读再跑」丢了，只剩不常驻的上游 README）；三-2（「本项目只接这一份」先行词变成 `-en`） |
| 四 不采纳的那几条 | 不归攻方 | — | 分工表攻方那一行没给这一格 |

另记两条观察（不算打中，理由在各自一节）：观察-1（派发表新指到参考件，`CLAUDE-merged.md` 读参考件的时机只列两份规则）；观察-2（「提交之外」那一条的一次还是两次弹窗，修稿一方自己挂着没定）。

## 格一 修补对不对

### 一-1 「只有两类」没有照判决去掉，改成限定在 `crates/` 与 kb，做法比原件窄，又与共用约束那一条相抵

- 判决怎么写的：`research/prompts/rules-compress-entry-r1-main-verification.md:18`：「两条腿都点到，主 agent 现查合稿第 9 行坐实，去掉」。
- 原件：`.claude/main-agent.md:7`：「探索性的 `crates/` 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写」，没有「只有」；原件共用约束 `.claude/agent-common.md:8`：「**简单任务不派发，自己直接改**」，不分目录。
- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:9`：「`crates/` 与 kb 里，主 agent 可以自己写的只有两类」，同一行接着写「别的文件的简单任务照」共用约束「派发」一节自己做。第二版共用约束 `research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:11`：「简单任务（修改一下、查询一下、计划一下这类单一、流程简单的）自己直接做，不派」，同样不分目录。
- 修稿一方的说法：`research/prompts/rules-compress-r2-merged/contradictions.md:14` 登成「改了判据（已收窄）」，同一行另写「与派发表同义」。两句互相对不上：同义就没收窄，收窄了就与原件不同义。
- 为什么算打中（格一字面：「放回的话与原件不同而做法因此不同」）：判决要的是去掉「只有」回到原件；修补没去掉，把「只有」留在 `crates/` 与 kb 两处。照原件，`crates/` 与 kb 里的简单任务（一处已定条款下的笔误、一行注释、一处 kb 错字）按共用约束「简单任务不派发」由主 agent 自己改；照第二版 `main-agent.md:9`，`crates/` 与 kb 里不属于那两类的，即使简单也不能自己写。
- 二义（同时满足格二）：第二版共用约束 `agent-common.md:11` 不分目录，`main-agent.md:9` 分目录。读法甲：入口那一句是对共用约束的收窄，`crates/` 与 kb 里的简单任务照样派；读法乙：共用约束那一条管全部简单任务，入口那一句只列「不简单却可以自己写」的两类。两种读法做法不同。
- 照第二版做会做成什么：读法甲下，一处条款已定的 `crates/` 注释错字要走 `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:91` 那一行「改 `crates/`：条款已定、验收标准写得清、改动有界」→ 派 `implementation-writer`、副本里改、交补丁、整套验证段；一处与定案无关的 kb 错字要派 `kb-scribe`。原件下两样主 agent 自己改，`crates/` 那一处照旧并进代码轮三方。
- 四句：不涉及臂；主 agent 下判断那一刻看得到两份文字（入口常驻，共用约束要另读，入口指过去了）；满足格一「放回的话与原件不同而做法因此不同」，同时满足格二「读得出两种意思」；没有跑前改法。
- 我提的改法（只在我的阅读上核过、被攻过零轮）：照判决去掉「只有」，写成「`crates/` 与 kb 里另外许主 agent 自己写的两类」；或者在第二版共用约束那一条写明 `crates/` 与 kb 除外，二者择一并登进 `contradictions.md` 入口-4 那一行，把「与派发表同义」删掉。

### 一-2 压缩上下文之后的重读只罩「验证段」「派发」两件事，「派出去之后」「交回怎么读」两节的入口没改

- 判决怎么写的：`research/prompts/rules-compress-entry-r1-main-verification.md:17`：「改成每次做那一步之前正文不在上下文就重读」。
- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:5`：「技能正文不在此刻的上下文里就整份重读」，同一行紧接「压缩过上下文之后再做这两件事之前同样重读」；「这两件事」指同一行前半句的「每次开始验证段之前」「每次派发之前」。技能自己的描述 `research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:3`：「主 agent 每次派发子 agent 之前读它」。
- 没改的入口：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:74`：「细则（看门狗怎么起、告警两组与处置、叫醒之后、坏卡与服务健康、自己交的任务怎么看与停、预计超过一小时的活）：先读」；`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:78`：「细则（交接摘要、harness 通知的三种读法、停掉子 agent 之后的任务号）：先读」。两处只写「先读」，不写哪一步之前；第一轮攻方 `research/prompts/rules-compress-entry-r1-opus-output.md:45` 点过这三节的写法，修补只动了第 5 行。
- 为什么算打中：读告警、读 harness 通知这两件事发生在叫醒时，不在派发之前、也不在验证段之前；压缩上下文的那一句只管「这两件事」。主 agent 压缩之后能不能看出「技能正文还在不在上下文里」，第一轮已经判过看不出（判决第 2 条采的就是这一点），所以第二版另加了压缩那一句；这一句的射程比判决写的「每次做那一步之前」窄。
- 照第二版做会做成什么：长会话（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:35`：「会话过 40 万上下文就压缩上下文接着做」）派完一批之后压缩一次，接着收到一条 harness 的 completed 通知、没有交回消息。三种读法只在技能里（`research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:51`：「它起了后台任务，越出」；`research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:52`：「它结束本轮时手里没有还在跑的后台任务，不会自己醒」）。压缩那一句不罩这一刻，主 agent 照摘要里的印象把它当成交回，或者等它自己醒；看门狗告警（哪几类只记不处置、卡住的在下一次回报之前处置或 `--ack`）同理。
- 二义（同时满足格二）：「技能正文不在此刻的上下文里就整份重读」可以读成管全部「先读」，也可以读成只管紧挨着的两件事；后一句「这两件事」把后一种读法坐实了一半。
- 四句：不涉及臂；主 agent 那一刻看不到「技能正文还在不在」（判决已采），只能看到规则写的触发点；满足格一「修补与判决写的改法不同而做法因此不同」；没有跑前改法。
- 我提的改法（被攻过零轮）：第 5 行压缩那一句改成「压缩过上下文之后，做任何一处写了『先读』技能的那一步之前同样重读」，第 74、78 行的「先读」改成「做这一步之前先读」，与第 1、4、8、11、15 条同一种写法。

## 格二 修补引出的新问题

### 二-1 「整批重交」的新定义里「第 1 步自己红了、退 2 的」读得出一条指令，撞第 1 步退 1、退 2 两支

- 第二版定义：`research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:14`：「第 1 步自己红了、退 2 的，第 2 步还没做过，只重交第 1 步」；同一行末尾写各步各条说到这四个字都指这一套。
- 同一份技能第 1 步两支：退 1 那一支 `research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:23`：「主 agent 照原样交，修补的补丁目录照第 5 步接在最后」（交的是脚本打出的只重跑红的那一条，带 `--item` 与 `--fingerprint-of`），同一行「那一条退 2、说那一趟开跑以来主工作区变了的，照它的出路整批重交」；退 2 那一支 `research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:24`：「退 2（打补丁核不过，脚本整批停）：照它出路里段树那一句办」，新段树照写段第 2 条建，不是重交。
- 脚本文件头今天的原文：退 2 有几种意思，`scripts/gate/merge-and-verify.sh:208`：「2 补丁核不过（apply 退 2）、用法错」，下一行接着是 `--fingerprint-of` 那一趟主工作区变了与 `merge-patch` 已经合过；只有 `--fingerprint-of` 那一种的出路是整批重交，`scripts/gate/merge-and-verify.sh:181`：「出路写整批重交（去掉 --item 与 --fingerprint-of）」。
- 两种读法：甲（修稿一方的本意，照 `dispositions.md:12`）：那半句只说「整批重交」发生在第 1 步阶段时不带第 2 步；乙（照字面）：「第 1 步红了（退 1）或退 2 的，只重交第 1 步」是一条指令。乙读法下，退 1 那一趟不交脚本打的只重跑红的那一条，改成不带 `--item` 的整批重交；退 2 打补丁核不过那一趟，不照段树那一句重建，原样把同一批补丁再交一遍（核不过的照样核不过）。
- 原件这一处没有统一定义，各支各写各的出路（`.claude/main-agent.md:57` 同一句「那一条退 2、说那一趟开跑以来主工作区变了的，照它的出路整批重交」只挂在只重跑红的那一条后面）；乙读法是修补那半句新带进来的。
- 四句：不涉及臂；主 agent 看得到退出码与脚本的「→ 怎么办：」那一行，但退 2 的几种意思要读输出才分得开，规则那半句没写是哪一种；满足格二「新写的句子读得出两种意思」；没有跑前改法。
- 我提的改法（被攻过零轮）：那半句改成「只重跑红的那一条退 2、说主工作区变了的，整批重交时第 2 步还没做过，只重交第 1 步」，把「自己红了」去掉，点名是哪一种退 2。

### 二-2 调查员「查到就修」的例外写进写段第 2 条之后，写段第 6 条的前提不再成立

- 用户定案第 19 条（不攻该不该，只攻落没落对）：`records/2026-11-16-三方豁免取消与存量补轮.md:171`：「照旧在主工作区改」。
- 落点：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:47`：「写段的补丁不打主工作区」，同一句接着写调查员那一支「那一支是例外，照旧在主工作区改」（行里带书名号的那半句不整句抄）。调查员定义的落点 `research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:23`：「这时在主工作区改（这一支是实现员」。
- 没跟着改的那一条：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:51`：「主工作区里没有打了而没验过的补丁，直接回那个会话」（全句是别的会话通知要起全量门禁时，直接回「已收尾」）。原件 `.claude/main-agent.md:51` 同一句，原件写段第 2 条没有例外，这一句的前提在原件里由第 2 条担保。
- 为什么算打中（格二「修补之后…新写的句子读不读得出两种意思」）：修补把例外写进了写段第 2 条，第 6 条仍以「主工作区里没有打了而没验过的补丁」为既成事实。读法甲：第 6 条照旧无条件回「已收尾」；读法乙：第 6 条的前提只在没有调查员「查到就修」在跑、或它的改动已验过时成立，否则不回。第二版没写乙读法下怎么办。
- 照第二版做会做成什么：写段里派了一个「查到就修」的调查员，它在主工作区改了 `crates/`、命令写进报告还没交；别的会话来问要不要起提交前的整轮门禁，主 agent 照第 6 条回「已收尾」，那一趟整轮判的是带着没验过改动的主工作区，红了算到对方头上（这一段是推的，没有跑）。
- 四句：不涉及臂；主 agent 那一刻看得到自己派过「查到就修」的调查员（调度记录与派发提示），但第 6 条没叫它看；满足格二；没有跑前改法。
- 我提的改法（被攻过零轮）：第 6 条加半句「有调查员『查到就修』改了主工作区而还没验过的，先回在改、等它的命令跑完再回」。

### 二-3 拒算第 1 条「主 agent 也不往哪一棵树里手写」读得出连删行也不许，撞技能第 2 条「主 agent 在主工作区直接删」

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:68`：「主 agent 也不往哪一棵树里手写，收的行只走拒算补丁目录」；技能 `research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:55`：「主 agent 在主工作区直接删」（这一批之前就有的「已不拒算」行）。
- 原件：`.claude/main-agent.md:68`：「实现员不登记，也不替它登记」；`.claude/main-agent.md:69`：「主 agent 在主工作区直接删」。「登记」说的是加行，与直接删行不相抵。
- 两种读法：甲，「手写」只指往拒算表加行；乙，「手写」宾语空着，指任何一棵树里的拒算表一律不手改，删行也算。乙读法下，这一批之前就有的「已不拒算」行要么也做成拒算补丁目录，要么不删；照技能则在主工作区直接删。做法不同。
- 同一条判据又住了两处：同一节开头 `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:66`：「不另往哪一棵树里手写这一行」，第 68 行修补后又写一遍（第 66 行带宾语「这一行」，第 68 行不带），两处措辞不一，是乙读法的来源。
- 四句：不涉及臂；主 agent 看得到两份文字；满足格二「新写的句子读得出两种意思」与「同一条判据又住了两处」；没有跑前改法。
- 我提的改法（被攻过零轮）：第 68 行那半句删掉，留第 66 行一处；或写成「主 agent 不往哪一棵树里手写要收的行」。

### 二-4 同一条判据修补之后住了两处（五处）

格二字面「修补之后同一条判据是不是又住了两处」，有一处就算打中。下面五处都是第二版新放回或新加的句子，另一处在第一版就已经有：

| 判据 | 第二版入口 | 第二版技能 | 两处措辞差在哪 |
|---|---|---|---|
| `task wait` 以 `needs-attention` 退出不是结局 | `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:74`：「以 `needs-attention` 退出时任务还在、不是结局」 | `research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:41`：「时任务还在、不是结局」 | 技能另写退出码以 `task_server_lib.py` 文件头为准，入口没有 |
| 服务健康处置后 `control check` 回读 | `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:74`：「回读，退 0 才算健康」 | `research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:39`：「回读，退 0 才算服务健康」 | 技能另写缺一样退 78、出路在输出里 |
| 事实表过了 `--check-facts` | `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:95`：「--check-facts` 才往下」 | `research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:75`：「--check-facts` 才切候选表」 | 「才往下」与「才切候选表」 |
| 拒算清单罩不到的三类审 diff 时看 | `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:68`：「的清单罩不到挂特性的测试文件、产品 GPU 包的测试包与别的包里的新测试」 | `research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:58`：「挂特性的测试文件、产品 GPU 包的测试包」 | 入口写「罩不到」，技能写「只取验证包不带特性那一趟」 |
| 调查员「查到就修」是交补丁的例外 | `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:47`：「那一支是例外，照旧在主工作区改」 | `research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:14`：「那一支除外，在主工作区改；各在副本里改」 | 技能那一处插在括号中间，「各在副本里改」的主语读起来接着调查员 |

- 修稿一方是有意的：`research/prompts/rules-compress-r2-merged/dispositions.md:15` 写「各留一句」。打中的是格二这一格的字面，做法上眼下两处一致，风险是以后只改一处（第一轮的拒算尾段就是这样只剩技能一处、被打中的）。
- 第五行另有一点：技能那一处的括号把例外插在「实现员一律交补丁」与「各在副本里改」之间，读得出「调查员除外，在主工作区改；［调查员］各在副本里改」这种自相矛盾的读法，只是做法上读的人会自己纠正，记轻。
- 四句：不涉及臂；看得到；满足格二「同一条判据又住了两处」；没有跑前改法。
- 我提的改法（被攻过零轮）：入口每处只留一句判据加「细则在技能」，技能那一处写「判据在入口第几行」，不两处各写一遍；第五行技能的括号拆成两句。

### 观察-1（不算打中）：派发表新指到参考件，读参考件的时机只列两份规则

- 第二版派发表「暂存之后、提交之前跑门禁」一行改成指参考件：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:96`：「各道判什么照 `.claude/rules/verification-reference.md`」（引文到书名号前为止，后面接的节名是门禁管哪一半）。参考件常驻不常驻那一句 `research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:66`：「到 `.claude/rules/verification.md` 与 `.claude/rules/implementation-workflow.md` 点名它的那一步之前读点名的那一节」，没列 `main-agent.md`。
- 节名两处都现查在：参考件第 140 行是「门禁管哪一半」、第 14 行是「门禁 crash-point-replay 怎么跑」（`grep -n '^#'` 现取）。
- 不算打中的理由：「照 X 某节」本身就叫人去读，字面上只多一种「时机列表没列入口」的读法，做法差别小；而且第一版里别的文件（提交阶段技能、`fs-design.md`、工具员定义）早就从这两份之外指参考件（`grep -rlF` 量的，9 份文件含参考件自己），不是这一版新带进来的问题，只是又多了一处。

### 观察-2（不算打中）：「提交之外」那一条的一次还是两次弹窗，修稿一方自己挂着

- 第二版入口第 3 条 `research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:26` 改成指 `verification.md` 那一条：`research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:38`：「提交之外要跑（包括用户要求的那一趟），一律先弹窗问用户，同意了才跑」，同一行另写「提交时那一趟的确认就是带 `--full-gate` 的那一次弹窗」，提交之外那一趟的确认算不算 `--full-gate` 那一次没写。
- 修稿一方自己记了没定：`research/prompts/rules-compress-r2-merged/contradictions.md:147`：「两次弹窗还是一次，交主 agent 或用户定」。句子住在 `verification.md`，不在这一组四份里，所以只记观察；落用户定案第 18 条时这一处要一起定。

## 格三 第一轮没看到的

### 三-1 版本戳红了「先读再跑」丢了，只剩不常驻、不点名去读的上游 README 一处

- 原件：`./CLAUDE.md:43`：「对不上就红——那是在提醒」，同一格接着是 `./CLAUDE.md:43`：「规矩变过了，先读再跑」。
- 第二版：`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:42`：「门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红」，后面没有。同一张表下一格只有刷版本戳的做法：`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:43`：「发版之后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳」。
- 找过的地方：合稿树 `CLAUDE-merged.md` 与 `dot-claude/` 整棵 `grep -rn '先读再跑\|规矩变过了'` 0 命中（量的）；仓里只剩上游副本 `.claude/singlefs-ai-sop/README.md:27`：「规矩变过了，先读一遍再跑」，这份 README 不 `@` 常驻，入口只把它挂成链接，没有哪一句叫主 agent 读它。
- 第一轮正推腿把这半句归成「门禁红了是什么意思」的说明（`research/prompts/rules-compress-entry-r1-sonnet-output.md:397`），没算做法。我认为它是一句做法：「先读」是红了之后先做什么，「再跑」是读完才接着跑。
- 照第二版做会做成什么：门禁第一阶段报版本戳对不上，主 agent 手里只有「对不上就红」与「跑 install.sh 刷版本戳」，照后一句刷了戳、门禁转绿，接着干活；规则副本在会话中途变过，而常驻的那一份是会话开头载入的，变了的条款没读（推的，没有量会话里常驻规则会不会自己刷新）。
- 四句：不涉及臂；主 agent 那一刻看得到红的那一格，看不到「该先读」；满足格三「找到丢的」；没有跑前改法。
- 强度：轻。版本戳红多半出在主 agent 自己同步副本之后，那时它多半刚读过改动；但同步是别的会话做的那一种就没人读。
- 我提的改法（被攻过零轮）：`CLAUDE-merged.md:42` 那一格补回半句「对不上就红：规矩变过了，先读变了的规则再跑」。

### 三-2 「本项目只接这一份」的先行词落在 `-en` 上（轻）

- 原件：`./CLAUDE.md:41`：「本项目只接其中一份，不需要知道别的」。
- 第二版：`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:40`：「多语言版本**对外以 `-en` 为准**，本项目只接这一份」。紧挨着「这一份」的名词是 `-en`；原件「其中一份」不指哪一份，前文是兄弟目录 `../singlefs-ai-sop-zh`。
- 两种读法：甲，本项目只接 `-zh`（前文的兄弟目录）；乙，本项目只接 `-en`。做法上，下一格同步命令写死了 `-zh`（`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:41`：「rsync -a --exclude」那一条），照命令做不会做错；会做错的是「从本仓的会话改上游只往文件里填内容」时改哪一份仓，乙读法会去改 `-en`（推的）。
- 四句：不涉及臂；看得到；满足格三「二义」；没有跑前改法。
- 我提的改法（被攻过零轮）：「本项目只接 `-zh` 这一份」。

## 没打中的形状

这些都只读了一遍，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，一次不算「没打中」的结论。

格一，修补逐处对过、没找到问题的：

- `CLAUDE-merged.md` 冻结层那一行：放回的三条与原件 `./CLAUDE.md:90` 逐字相同；新指针指的 kb 那一节在（`.claude/kb/freeze-layer-membership.md` 第 109 行是「门禁判哪三条」）；门禁今天的判法与它一致：`.claude/gate.d/doc-decisions.sh:1851`：「每棵树在 key 编码层与索引节点内部布局层各至少一行」，同一行接着写每个单元类码至少一行。
- 入口第 3 条改成指 `verification.md`：「一个门禁：两种跑法」在那份第 29 行、「提交之外」那一条在第 38 行，内容与用户定案第 18 条一致（「用户要求的那一趟」也先弹窗）。
- 派发表放回的 `--check-facts`：与原件 `.claude/main-agent.md:121` 那半句同义，只多了「才往下」。
- 拒算第 1 条放回的三类：与脚本文件头一致，`scripts/gate/refused-test-registry.py:22`：「覆盖：只有验证包不带特性那一趟的测试函数」。
- 派发表「暂存之后」那一行新指的两节都在参考件里（第 140、14 行）；`--full-gate` 写法与弹窗、次序在第二版提交阶段技能第 20、22 行。
- 整批重交的新定义，在二-1 那半句之外的六处用法（技能第 35、37、38、40、41、55 行）逐处对原件 `.claude/main-agent.md:58`、`:59`、`:69`：做法相同；与脚本文件头第 163、181 行「去掉 --item 与 --fingerprint-of」一致。
- 两份技能 frontmatter 描述的改动与入口第 5 行同义。

格二，指针：四份被判文件里点到 `.claude/rules/`、`.claude/skills/`、`.claude/agents/` 下的每一份文件，在合稿树里都在（逐个 `[ -f ]`，量的，无缺）；第二版新加或改过的节名指针逐个现查在：共用约束「派发」（第二版第 6 行）、「本机常有别的会话同仓干活」（第 52 行）、「崩溃点测试准确率和正确率优先，其次再衡量时间成本」（第 54 行）、三方技能「用户叫停、不写判决的轮」（第 31 行）、实现员定义第 3c 步（第 49 行）。`CLAUDE-merged.md` 一句话版本里改成指 `.claude/gate.d/counted-claims.tsv` 的两个数，表里有 `CLAUDE.md` 的两行（114 条不变量、崩溃放量项数），命令与原件里写的数法相同。

格三，`CLAUDE.md` 第 1 到 122 行逐行对过第二版，除三-1、三-2 外，下面这些差别判成没丢做法：

- 第 12 行「主 agent 的职责…都在那一份」那张清单删了：挪进技能之后这句已经不真，第二版入口第 5 行另写了两份技能。
- 第 16 行「同一会话里派的继承会话开始时那一份」：理由句。
- 第 22 行「门禁归提交时，由提交前主 agent 照 … `gate-staged.sh` 判」：第二版 `verification.md` 第 35、45 行有；「其余几类还没有格判、由主 agent 写判决时逐份点名」：第二版 `implementation-workflow.md` 第 21 行有。
- 第 42 行「副本不带上游的 `CLAUDE.md` 与 `.git`」：同步命令的两个 `--exclude` 带着。
- 第 75 行「经过要留就留在 `records/` 与 `.claude/kb/` 里已有的那一份」换成指上游 `rules-discipline.md`：上游第 3 条「不另立落点，也不去别处补一份」与「留在已有的那一份」不相抵，只是少了「留哪」这一句许可，记没丢。
- 第 89、96、98、111 行几处删词（「一行一条」「要变成规则」「仍」「README、简报与 kb 的译文照它用词」）：不带做法。
- 第 103 行任务服务那一格删了「挑机器、内存额度、线程、GPU 与卡都由服务定，提交时不写；子 agent 不交任务」：第二版 `verification.md` 第 50 行、派发技能第 58 行、合入后验证技能第 26 行、入口第 19 行有，写了的服务当场拒。
- 第 107 行 `insert-row.py` 那一格删了「`replace-once.py` 只解决改一处…两个会话一前一后会互相覆盖」：说明句，插行用哪一个由两格各自的用途分得开。
- 第 120 行理由半句、第 122 行 2026-10-11 实测（用户定案第 11 条）与两条数法（改成指登记表）。
- `@` 常驻与 read-once 两张清单的增删归常驻规则那一题，这一题没判。

格三，派发技能「派出去之后」「交回怎么读」两节对原件 `.claude/main-agent.md` 第 76 到 100 行逐句对过：看门狗起法（第 80 行）、告警两组（第 82 行）、三类处置（第 84 行）、坏卡与草稿根两样（第 86 行）、服务健康（第 88 行）、发消息的两种时候（第 90 行）、叫醒之后与 `--ack` 三种键（第 94 行）、交回之后不发消息与交接摘要、harness 通知两种特殊读法、TaskStop 之后核任务号（第 100 行），在技能第 22 到 54 行与入口第 74、78 行都有落点、做法相同；技能第 50 行多写一种「有交回消息的照交回判」，不改做法。这两节里打中的只有触发时机（一-2），没有丢句。

## 我认为最脆弱的三处

1. 一-1：`crates/` 与 kb 里「只有两类」。判决写的是去掉，修补留着并收窄，修稿一方一行里同时写「改了判据」与「同义」，第二版共用约束那一条又不分目录，两份常读的文字直接相抵。
2. 一-2：压缩上下文之后的重读只罩两件事。第一轮判决最重的一条（技能挪出常驻之后靠什么保证读到），修补只补了一半，叫醒时用到的两节仍靠「正文还在不在」这个主 agent 看不到的判断。
3. 二-2：调查员例外写进写段第 2 条，写段第 6 条照旧把「主工作区里没有没验过的改动」当事实，别的会话起整轮门禁时会被回「已收尾」。

## 要主 agent 跑的命令

无。这条腿没写模型、没有要经服务跑的东西；引文核对 `scripts/agents/cite-check.py` 是直接跑的文本工具，已跑，末行贴在交回里。

## 这条腿自己的限度

- 全部结论来自读文字，没有一条经过执行；写了「推的」的三处（二-2 那一趟整轮红算到对方头上、三-1 会话里常驻规则会不会自己刷新、三-2 改哪一份上游仓）没量。
- 格三只做了分工表点名的 `CLAUDE.md` 与派发技能两节；入口与合入后验证技能别的段只在格一、格二的修补处读了。
- 格四不在攻方分工里，`dispositions.md` 入口题那几行只当线索读了，没判理由站不站得住。
- 别的题的合稿文件（共用约束、`verification.md`、参考件、提交阶段技能、调查员定义）只在被这一组指到、或要核落点时读了那几行。
- 一次样本，照规则「没打中」的几格不撑结论。

## 没做什么

- 不判辩方那几格；不替主 agent 采纳；没往统一任务服务交任务；没有副本上的数。
- 没建仓副本、编译目录；草稿目录 `~/.local/share/singlefs/work/rules-compress-entry-r2-opus` 里只有两份 `cite-check.py` 的输出 `cite1.txt`、`cite2.txt`，没有仓副本与编译目录要删。
- 禁读清单里的文件（辩方腿的报告、模型与草稿目录，别的四题第二轮各腿的产出与草稿）没读。
