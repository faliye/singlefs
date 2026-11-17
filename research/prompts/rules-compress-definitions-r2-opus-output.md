# rules-compress-definitions-r2 攻方腿报告

日期：2026-11-16。轮名 rules-compress-definitions-r2，第二轮；前几轮判决 `research/prompts/rules-compress-definitions-r1-main-verification.md`。这条腿没写模型、没有复跑脚本；草稿目录 `~/.local/share/singlefs/work/rules-compress-definitions-r2-opus/` 里只有两份只读文件的辅助脚本（`ptr.py` 核指针、`sdiff.py` 按句差分）与它们的输出。

开工核对（量的）：`sha256sum -c research/prompts/rules-compress-definitions-r2-snapshot/judged-sha256.txt` 15 行全对；在合稿树目录里对 `merged-tree-sha256.txt` 全对；第一版合稿树 `research/prompts/rules-compress-r1-merged/` 对第一轮的 `merged-tree-sha256.txt` 也全对（拿它重算了第一版到第二版的差）。

## 各格判定一览

| 编号 | 格 | 打在哪 | 判定 | 轻重 |
|---|---|---|---|---|
| 攻-A | 二（修补引出）、三 | 调查员「查到就修」那一支：改在主工作区，定义却叫它把验证命令的 `--cwd`、`<根>` 填「副本」；第二版实现员第 1 步把「改动所在的树」改成「你的副本」，把调查员借用的那一路也带偏了 | 打中 | 中 |
| 攻-B | 一、二 | 「规则怎么读」第一条的修补：原件「一律整份读」被换成「步骤里点名的做那一步之前读那一节」；新句「读那一节，读全」读得出两种；去向表把它记成「用户定的」，矛盾总表记的是主 agent 定 | 打中 | 中 |
| 攻-C | 一（用户定案第 20 条落没落对）、二 | 工具员第 5 步：定案要「照这个写清」，第 5 步没写循环逐个发、先挑再发；「自证里只对进程号发」与「脚本里只对那一个组发」两句在自证脚本上重叠，各给一套写法；还把只给主 agent 的 `proc.py` 写进自证 | 打中 | 轻到中 |
| 攻-D | 二 | 派发闸那一项「写者锁持有者还在跑」写了两遍，一遍带「写者锁已判」放行行，一遍不带 | 打中 | 轻（主 agent 一侧） |
| 攻-E | 一（用户定案第 1 条）、三 | 实现员第 3 步仍写「在草稿目录建副本」：一律交补丁之后实现员手里已经有改动所在的副本，照共用约束建副本的默认是从主工作区建，建出来的没有这次的改动 | 打中 | 轻到中 |
| 攻-F | 三（弹窗闸） | 合稿把出处写成「路径、命令」，钩子只认反引号里的路径与命令；断言词写成开放的「这类」，钩子只认九个；「选项里」比钩子判的 description 宽 | 打中（照合稿写的合规句会被拒） | 轻（主 agent 一侧） |
| 攻-G | 三（定时闸、项目代码闸） | 定时闸只留「除 remind.sh 之外」，项目代码闸把只给主 agent 的工具名单删成代码指针；子 agent 读不出 remind.sh 自己起不了；项目代码闸的出路只剩「经 submit 交」，子 agent 那一支没了 | 打中 | 轻 |
| 攻-H | 三（写闸） | 写闸一项没列钩子第七道里的批次代号；共用约束只把字母代号算批次代号，钩子另拒汉字数字加天干那一类写法（本报告第一次落盘就被检出钩子 ⑰ 照这一条拒了，见攻-H） | 打中 | 轻 |
| 攻-I | 三（交回正文闸） | 「书记员」在第二版合稿树里只剩共用约束一处，哪一类是书记员全树找不到（钩子里它是 kb-scribe） | 打中 | 轻 |
| 材料 | — | `diff-from-r1.patch` 不是第二版相对第一版的全部改动：用户定案第 22 条那一处（「或卡住」）不在补丁里 | 材料不全，不算打中被判文件 | — |

没打中的范围与逐句对过的范围在末尾「没打中的形状」。

## 格一、格二：修补对不对、修补引出的新问题

### 攻-A 调查员「查到就修」的验证命令落到副本上（中）

第二版里调查员那一支改在主工作区，这是用户定案第 19 条：
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:23`：「不在副本里改、不交补丁」

同一份定义第 1 步把所有命令的工作目录写死成副本，第 6 步（修的那一步）叫验证命令照第 1 步写：
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:28`：「--cwd 填副本」
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:33`：「证红（prove-red.sh）、clippy、build、动到的测试目标、命名检查与第 5 步那几条带 --item 的命令照第 1 步写进报告」

调查员手里叫「副本」的只有第 2 步建的那一份，建在修之前：
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:29`：「在 <草稿目录>/copy 建仓副本」

第 6 步让它照实现员第 1d 步写带 `--item` 的命令；第二版实现员第 1 步这一次改成了「填你的副本」，第 1d 步自己又写「填你改动所在的树」，两句在调查员身上不是同一棵树：
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:31`：「带 --item 的命令里的 <根> 与 --copy 填你的副本」
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:34`：「<根> 填你改动所在的树」

原件这里是对的：
- `.claude/agents/investigator.md:28`：「写成 task-server/task submit [--cwd <副本>] -- <命令> 的整条」（方括号，可不给）
- `.claude/agents/implementation-writer.md:31`：「<根> 与 --copy 填你改动所在的树」
- `.claude/agent-common.md:114`：「填改动所在的那棵树（主工作区写仓根」

第二版共用约束第 109 行照旧写主工作区填仓根，但定义写的是副本，而共用约束开头就定了冲突时听定义：
- `research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:3`：「与定义冲突时以定义为准」

子 agent 照第二版做会做成什么：调查员在主工作区改完代码、补了测试，交回的 clippy、build、测试目标与命名检查都写成 `--cwd <草稿目录>/copy`（命名检查的 `<根>` 同样），主 agent 照抄去跑，跑的是修之前的树：新测试不在那棵树里，旧测试照绿，命名检查查不到新名字；这一次修没有一条命令真在改过的树上跑过，主 agent 读到绿会当成修过了。改动已经在主工作区，没有合入后验证兜底。

矛盾总表里调查员只有改步号那一行（`research/prompts/rules-compress-r2-merged/contradictions.md:75`：「原第 6 步删掉后改号」），这处做法变化没登。第一轮两条腿都没报（两份报告 `grep -n '填副本'` 0 处，量的）。

四句：分辨（原件与第二版对同一个调查员给出两棵不同的树）；被判对象看得到（第 23、28、33 行在同一份定义里）；满足第二格「新写的句子读不读得出两种意思」与第三格「没登而做法变了」；没有跑前条款给的改法。

我提的改法（只在纸面上推过、被攻过零轮）：调查员第 1 步写「复现与二分在副本里起，`--cwd` 填副本；第 6 步那一支的命令 `--cwd` 与 `<根>` 填仓根」；实现员第 1 步改回「填你改动所在的树（就是你的副本）」。

什么现象会推翻：第二版别处写明调查员第 6 步的命令在主工作区起（我在合稿树里 `grep -n '仓根' dot-claude/agents/investigator.md` 0 处，量的）。

### 攻-B 「规则怎么读」第一条的修补（中）

原件只有一句，说的是读规则文件的读法：
- `.claude/agent-common.md:22`：「一律整份读，不受读入大小的限制」，同一句后半「照样读全那份文件」

第二版写成两句：
- `research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:17`：「开工整份读，不受读入大小的限制」（对象限成定义「开工先读：」点名的与共用约束点名的）
- 同一行：「定义与共用约束的步骤里写」……「做那一步之前读那一节，读全」

三处不对：

1. 做法变了（第一格「放回的话与原件不同而做法因此不同」）。原件对任何一份规则文件都整份读；第二版对步骤里点名的规则只读点名的那一节。第二版这一轮新加的指针就落在这一类里：三条三方腿第 3c、4、3 步「照 `.claude/rules/command-safety-by-review.md`「装置起的样本进程，放进它自己的进程组」」，照第二版只读这一节，照原件读整份。
2. 新句读得出两种（第二格）。「读那一节，读全」：读法甲是那一节读完整；读法乙是读那一节、并把整份读全。原件同一条里「读全」指的就是整份文件（「照样读全那份文件」），顺着原件读会落到乙，顺着第二版的前后文会落到甲；两种读法读的东西不同。另外「共用约束的步骤」没有所指：共用约束全是条目，没有编号的步骤，它第 22、24、42、59、67、117、118 行那些「照 <规则>「<节>」」算不算步骤，第二版没说；不算的话，这几处指针既不在开工先读、也不在步骤里，第二版没给任何读的指令。
3. 去向表把这一条记成用户定的，矛盾总表记的是主 agent 定：
   - `research/prompts/rules-compress-r2-merged/dispositions.md:83`：「用户定的两句」
   - `research/prompts/rules-compress-r2-merged/contradictions.md:102`：「改了判据（主 agent 定）」
   调度记录里用户说「两句管两件事」的是三方开轮那一条，不是规则读法：
   - `records/2026-11-16-三方豁免取消与存量补轮.md:119`：「两句管两件事，都留」
   所以这一条不在「用户定案不攻该不该」的保护里；改了判据也只经主 agent，被攻过零轮。

子 agent 照第二版做会做成什么：三方腿按读法甲只读样本进程那一节；按读法乙读整份 `command-safety-by-review.md`。共用约束第 22、42、59 行指的 `path-moves.md`、`kb-discipline.md`、`verification.md`「提交之外」那一条，没在开工先读里点名的类型照第二版可以一节都不读。

四句：分辨（甲乙两种读法读的范围不同）；被判对象看得到（同一行）；满足第一格与第二格；没有跑前条款给的改法。

我提的改法（纸面、零轮）：第二句写成「……做那一步之前读那一节，读到下一个同级标题为止」或直接恢复原件「凡读规则文件一律整份读」；另写一句共用约束条目里的「照 <规则>「<节>」」按哪一句办。

什么现象会推翻：调度记录或矛盾总表「用户定」一节里另有一条用户就规则读法说的原话（我在调度记录里 `grep -n '整份读\|做那一步之前'` 只命中第 179 行，那一行说的是主 agent 读技能，量的）。

### 攻-C 用户定案第 20 条在工具员第 5 步上落得不全（轻到中）

定案原话：
- `records/2026-11-16-三方豁免取消与存量补轮.md:194`：「命令行上与脚本里都只许对自己用新会话起的那一个子进程的进程组发信号，别的组、循环逐个发、先挑再发一律拒；工具员定义第 5 步照这个写清」

第二版第 5 步：
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/tooling-writer.md:33`：「自证里只对自己起的进程号发信号」，后接「不按名字或 cgroup 挑；不对别的进程组发信号；自己写的装置与脚本里只对自己用新会话起的那一个子进程的组发」

1. 「循环逐个发」「先挑再发」两样，第 5 步一个字没写（`grep -c '循环' dot-claude/agents/tooling-writer.md` 0，量的）。共用约束那一条写了循环、没写先挑再发；规则那一节两样都写了：
   - `research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:67`：「不对别的进程组发、不在循环里逐个发」
   - `research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:9`：「不对别的进程组发信号，不在循环里逐个发，不先挑进程再发」
   定案点名的是第 5 步写清，第 5 步只靠两跳指针。
2. 两句在自证脚本上重叠，给两套写法。自证也是脚本：读成「自证里只对进程号发」，起了 `setsid` 样本的自证不能对它的组发，孙进程留下；读成「脚本里只对那一个组发」，`kill "$!"` 这种点名进程号的也不算合规。钩子两样都放行：
   - `.claude/hooks/bash-command-detector.sh:184`：「对自己用新会话起的那一个子进程的进程组发信号（上游」
   第 5 步的「只对」两处各排掉了对方那一种。原件第 5 步只有一句，没有这个重叠（`.claude/agents/tooling-writer.md:33`：「不按名字、cgroup 或进程组发」）。
3. 第 5 步把 `proc.py stop <pid>` 写成自证里的停法；第二版共用约束删掉了原件的限定语，只剩不带条件的「只给主 agent」：
   - `research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:66`：「proc.py 只给主 agent」
   - `.claude/agent-common.md:71`：「子 agent 直接跑由项目代码闸拒」
   原件说的是子 agent 不直接起它；第二版读得出「脚本里也不许用」，与第 5 步打架。

子 agent 照第二版做会做成什么：工具员写自证时，起样本用 `setsid` 的那一支按第 5 步第一句只敢 `kill "$!"`，孙进程留下（规则那一节第 11 行说的管道写端那种）；或者按第三句改成组发而把点名进程号的放行写法当违规改掉。

四句：分辨（两句对同一份自证给出两种合规写法）；看得到（同一行）；满足第一格「放回的话与定案不同」与第二格「新写的句子读得出两种意思」；没有跑前条款给的改法。

我提的改法（纸面、零轮）：第 5 步照定案原话写一句「自证、装置与脚本里：点名一个自己起的进程号，或对自己用新会话起的那一个子进程的组发；别的组、循环逐个发、先挑再发一律不写」，`proc.py` 那半句删掉。

### 攻-D 派发闸那一项的写者锁写了两遍（轻）

- `research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:104`：「写者锁持有者还在跑又没有一行」，同一行后面又列「在跑的子 agent 满 5 个、写者锁持有者还在跑、报告行的路径或文件名不合」
- 钩子：`.claude/hooks/runner-dispatch-guard.sh:93`：「持锁的已交回 / 失效 / 从没起来」那一句接着写了写「写者锁已判」行新锁接手

第一处是这一轮补回的（攻-6 的修补），第二处是第一版留下的；修补之后同一条判据住了两处，一处带放行行、一处不带，读后一处的主 agent 会以为写了放行行也照拒。第二格「同一条判据又住了两处」。改法：删掉后一处的「写者锁持有者还在跑」。

### 材料：补丁不全（不算打中被判文件）

正文说补丁是全部改动：`research/prompts/_rules-compress-definitions-r2-body.md:17`：「第二版相对第一版的全部改动」。我拿第一版合稿树对第二版重算了 15 份的差（`diff -u`，草稿目录 `ac-r1-r2.diff` 与 `def-*.diff`，量的）：14 份定义与补丁一致；共用约束多出一处补丁里没有的改动，就是用户定案第 22 条那一句。补丁里还是旧句：
- `research/prompts/rules-compress-r2-merged/diff-from-r1.patch:238`：「上下文吃紧时照它定义第 2b 步停在最后一个写完整的点交回」
- 冻结的第二版：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:68`：「上下文吃紧或卡住（同一步反复做不过去）时」

被判文件这一处本身落对了（执行员定义第 2b 步原件就写「上下文吃紧或卡住时」，两处一致）。只看补丁的腿会漏攻这一句。

## 格三：第一轮没看到的

### 攻-E 用户定案第 1 条之后，实现员第 3 步的「建副本」（轻到中）

- `research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:15`：「一律在草稿目录的副本里改」，同一行「没给的从主工作区建」
- `research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:46`：「在草稿目录建副本」（第 3 步证红，前面是「照共用约束」建仓副本）

原件第 3 步同样写建副本（`.claude/agents/implementation-writer.md:36`：「在草稿目录建仓副本」），那时实现员默认在主工作区改，证红要另建一份副本。定案第 1 条之后实现员本来就在副本里改，第 3 步这一句没跟着改：照字面再建一份，`<仓根>` 默认取主工作区，新副本里没有这次的实现与新测试；第 1 步又说 `--copy` 填「你的副本」，手里有两份，哪一份没写。照新建的那份写 `prove-red.sh --copy`，变异行的原文在那棵树里找不到、或新测试不存在，主 agent 跑出来是拒或证不红，承接的实现员去改一个本来没错的变异行。调查员借用这一步时它是对的（调查员改在主工作区，新建副本才带着修），所以这一句该分两支写，不是删。第一轮攻方逐句读过这份定义，没报。

### 攻-F 弹窗闸：照合稿写的出处会被拒（轻，主 agent 一侧）

- 合稿：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:104`：「问句或选项里一句话带断言词（不可能、一定、必然、从来不、永远不、恒为这类）而同一句既没有出处（路径、命令、文件:行号、产物行、量过的数）」
- 钩子：`.claude/hooks/ask-user-claim-guard.sh:23`：「反引号里的路径（带 / 的，或带登记过的扩展名 PATH_EXTENSIONS 的文件名）或命令」
- 钩子：`.claude/hooks/ask-user-claim-guard.sh:43`：「词表只认上面九个」
- 钩子：`.claude/hooks/ask-user-claim-guard.sh:16`：「（header、label 不扫）」
- 原件：`.claude/agent-common.md:104`：「反引号里的路径或命令」

合稿丢了「反引号里的」：主 agent 照合稿在断言句里写一个不带反引号的路径当出处，钩子不认、照拒。断言词写成开放的「这类」，钩子是封闭的九个（「造不出」「从来没有」「绝不会」合稿没列）；「选项里」比钩子判的 options 的 description 宽。后两样只会让人多写出处，不会做错；第一样是照合稿写的合规句被拒。用户定案第 7 条许细节下沉、只留合规写法，留下的合规写法与文件头不符，按第三格算。

### 攻-G 定时闸与项目代码闸：子 agent 读不出 remind.sh 只给主 agent（轻）

- 合稿：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:102`：「除 bash scripts/agents/remind.sh 之外的定时办法」
- 合稿：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:95`：「出路是写成展开之后的字面命令、经 submit 交」
- 原件：`.claude/agent-common.md:100`：「与只给主 agent 的会话工具（proc.py、remind.sh」
- 原件同一行：「子 agent 把那一条写进报告「要主 agent 跑的命令」一节」
- 钩子：`.claude/hooks/lib_project_code_runs.py:1098`：「只给主 agent 直接跑的会话工具」，下一行起名单里有 `scripts/agents/remind.sh`
- 钩子：`.claude/hooks/project-code-guard.sh:193`：「子 agent 那一份的出路是把同一条」

合稿第 95 行把工具名换成了代码里的变量名，`remind.sh` 只在定时闸那一句出现，而那一句读起来是「定时就用它」。子 agent 照合稿要定时（或照第 95 行出路「经 submit 交」去交）都会被项目代码闸拒一次；原件两句都写明了子 agent 怎么办。修稿一方在去向表里写明这几道没逐句核（`research/prompts/rules-compress-r2-merged/dispositions.md:95`：「项目代码闸、写闸其余各道、弹窗闸、定时闸没逐句核」）。

### 攻-H 写闸：批次代号没列，共用约束的批次代号比钩子窄（轻）

- 合稿写闸一项：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:96`：「内容里有撇号类角标、钟点或会话号」
- 合稿写法规矩：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:39`：「不写批次字母代号」，同一行「调度记录依赖表的「第 N 批」批号照留」
- 钩子：`.claude/hooks/write-guard.sh:39`：「七、会话号与批次代号」
- 钩子：`.claude/hooks/write-guard.sh:41`：「拿来当名字用的批次与运行代号」，同一行的例子里除了字母代号，还有批字或汉字数字的批号后面接天干的两种写法

写闸那一项没列批次代号；共用约束只把字母代号叫批次代号，又说「第 N 批」照留，子 agent 照它写「第 N 批」后面接天干的批号，会被写闸（Write / Edit）与检出钩子 ⑰（Bash）拒。量的：本报告第一次落盘时我在攻-H 这一行引了钩子那两个例子，检出钩子 ⑰ 当场拒了，拒绝说明点名「批次代号」与那两个写法、出路是写成那一批做了什么。拒绝说明会给出路，不会写坏东西，所以轻。原件第 54、101 行同样没写（存量），修稿一方的去向表写明写闸其余各道没逐句核。

### 攻-I 交回正文闸的「书记员」在合稿树里没有所指（轻）

- `research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:103`：「书记员与执行员报了没数或 0 项的绿行」
- 钩子：`.claude/hooks/handback-guard.sh:395`：「GREEN_COUNT_CHECKED_AGENTS = {"kb-scribe", "experiment-runner"}」

钩子里书记员就是 kb-scribe；kb-scribe 的定义自称写回员，同一项后半也叫它写回员。第二版合稿树里「书记员」只剩这一处（`grep -rn '书记员' dot-claude CLAUDE-merged.md` 1 行，量的；第二版把 prior-art 的「交书记员」改成了「交写回员（kb-scribe）」，原来另一处用法没了）。kb-scribe 读这一项认不出说的是自己。共用约束第 110 行另有通用的「贴绿行时连查了多少项一起贴」，所以做法丢不了，只是这一项的主语没有所指。

## 没打中的形状

逐句对过的：
- 补丁里落在这一组的每一处改动：共用约束 15 处改动行、定义 9 份 15 处改动行（补丁里 `+` 起头的行，量的），每处对原件与（涉及钩子的）文件头。另用第一版合稿树重算了 15 份的差，找到补丁漏的那一处（见「材料」）。
- 第一轮攻方只按句差分的 12 份定义：按「。；」切句，原件里在第二版找不到原样的句子共 234 句（clerk 11、experiment-designer 47、investigator 17、kb-scribe 15、mutation-triage 25、prior-art 10、sweep 21、three-way-attack 36、three-way-defense 11、three-way-forward 12、three-way-materials 7、tooling-writer 22，`sdiff.py` 数的），逐句找了落点；调查员与实现员第 1、1d、3 步之间的互相引用按整步对读。
- 指针：15 份第二版文件里「`文件`「节」」与「`文件`（节）」共 216 处（`ptr.py` 的输出行数，量的），文件都在、节标题都逐字命中；没命中标题的两处写的是「那一条」（证据纪律第 61 行、验证规则第 38 行），条目在。上游那几处由「」改成（）的指针，节标题在上游文件里都在。
- 文件头：弹窗闸（第 1 到 47 行整段）、定时闸（第 1 到 66 行整段）、写闸（第 1 到 120 行；其后的自证开关表只按「六」检索）、检出钩子 ⑥（第 160 到 200 行）、完工清理闸「交接」那几行（第 99 到 102 行）、派发闸里三种「已判」行。

对过而没打中的修补：交接副本不列（与 `.claude/hooks/completion-cleanup-guard.sh:100`：「子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删」一致）；⑥ 的放行行与 `.claude/hooks/bash-command-detector.sh:184`：「对自己用新会话起的那一个子进程的进程组发信号」一致；旧数先重跑放回原处；⑬ 出路放回；直接跑的写入工具名单与 `lib_project_code_runs.py` 的文本工具名单一致；提交之外先弹窗（定案第 18 条）落在验证规则第 38 行；定案第 19、22 条各处的句子都在（第 19 条的做法问题见攻-A）；实现员「照入口退 79 时打的说明给的值写」与 `scripts/crash/crash-amplification.sh:18`：「算得出就写出能压到上限以内的 n」一致，估不出时那一句后面留着「留给提交时」的出路。

想打而判不中的：派发闸丢了「Explore 除外」，但钩子第 13 行说的通用类型本来就不含 Explore；丢了「退 2」，但钩子第 65 行把退 2、退 3 一起算「前提没齐」，第二版那半句盖得住。三方腿第 3c 步「`--cwd` 填副本」对只写模型、不建仓副本的腿没有所指，攻方第 51 行另写了模型的编译照第 3c 步，读得通，不算。kb-scribe、mutation-triage、sweep、prior-art、three-way-materials、experiment-designer 的改写（列表化、指针化）找到的落点都逐字在（experiment-designer 另用 14 个关键短语对两版计数：按行数 13 个相等；「implementation-first.md 第 4 条」原件两处在同一行、第二版拆成两行，数出 1 对 2，两处都在）。

只抽了、没逐句的：项目代码闸文件头 575 行里只读了第 1 到 60 行与第 185 到 230 行，其余按「fmt、出路、MAIN_AGENT、systemctl」检索；写闸第 121 到 191 行。

## 我认为最脆弱的三处

1. 攻-A：调查员修在主工作区、验证命令却落在修之前的副本上，主 agent 照抄跑出的绿不证明任何事，又没有合入后验证兜底。
2. 攻-B：「读那一节，读全」两种读法，加上这一条被记成用户定的而其实没被攻过。第二版大量细则改成「照 <规则>「<节>」」，读不读、读多少全压在这一句上。
3. 攻-C：工具员第 5 步是用户点名要写清的一步，现在两句各排掉对方的写法，又引了只给主 agent 的工具。

## 要主 agent 跑的命令

无。这一轮的打中都靠读文件与文件头现查；攻-H 有一次钩子真拒的读数（本腿写报告时的那一次检出），不需要另跑。

## 这条腿自己的限度与没做什么

- 钩子会不会拒，除攻-H 那一次是当场量到的，其余都是读文件头推的，没喂钩子（攻-F、攻-G 的「会被拒」是推的）。
- 项目代码闸文件头只读了两段（见上）；写闸自证开关表没逐行读。
- 12 份定义里两版逐字相同的句子没再读，只在攻-A、攻-E 这种跨步引用处按整步对读了实现员与调查员。
- 没判辩方那几格、没判第四格；不替主 agent 采纳；自己提的改法都只在纸面上推过、被攻过零轮；没往统一任务服务交任务；没建仓副本与编译目录，草稿目录里只有两份辅助脚本与它们的文本输出。
- 没读禁读清单里的文件。

什么现象会推翻这份报告：攻-A，第二版别处写明调查员第 6 步的命令在仓根起；攻-B，找到用户就规则读法说过的原话；攻-C，第二版工具员定义里另有写循环与先挑的句子；攻-F、攻-G、攻-H，喂钩子放行了照合稿写的那一句。
