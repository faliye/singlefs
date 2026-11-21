# ledger-topdown-governance-r1：判决（2026-11-18）

正文 `research/prompts/_ledger-topdown-governance-r1-body.md`，开工快照 `research/prompts/ledger-topdown-governance-r1-snapshot/`。这是这一题的第一轮，没有前几轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方腿（three-way-attack） | `research/prompts/ledger-topdown-governance-r1-opus-output.md`（sha256 以 4bc3ef6b 起头） | 正文第 1 到 6 问与第 9 问都回推不出，打中 17 条，要主 agent 跑 15 条命令 |
| 正推腿（three-way-forward） | `research/prompts/ledger-topdown-governance-r1-sonnet-output.md`（sha256 以 10717ece 起头） | 52 行成立 51、不成立 1；第 7 问八句打中 2、推不出 1；第 8 问路径类漏换 0、坏指针 2、读错风险 3 行；第 10 问成立；要跑 2 条命令 |

倒推开工快照（任务 19988，退 1）：

`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/governance-r1-verify -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py ledger-topdown-governance-r1 --out /home/fy5090/.local/share/singlefs/work/governance-r1-verify/ledger-topdown-governance-r1-tree --repository /home/fy5090/code/singlefs`

「✗ 快照 ledger-topdown-governance-r1：清单点名 61 份，倒推出 60 份、sha256 对上 60 份；倒推不出 1 份、对不上 0 份」。倒推不出的是 `.claude/gate.d/code-tooling.sh`：腿跑着的时候别的会话经换上工具换过它（它的任务 19965），只在文件头第 658、659 行加了两行；它留底的换上前那一版（`~/.local/share/singlefs/work/batch103-switch-replay-baseline-trip-gets-the-threads/originals/.claude/gate.d/code-tooling.sh`）sha256 以 b53f3c46 起头，与快照相同。这一轮判它的三处（runner 表两行与账本路径一行）在开工那一版里是第 2205、2228、9595 行，此刻各往下挪 2 行；两条腿引到这份文件的行都不落在加的那两行上。

引文核对（退 0）：

`python3 scripts/agents/cite-check.py research/prompts/ledger-topdown-governance-r1-opus-output.md research/prompts/ledger-topdown-governance-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/governance-r1-verify/ledger-topdown-governance-r1-tree --background research/prompts/_ledger-topdown-governance-r1-background.md`

「✓ 核了 137 处引文、132 处裸行号，对上 91 处，没判 178 处（其中裸行号没有引文可比 92 处；逐处列在上面）」，判错 0 处。没判的另 86 处是引到快照之外的文件（账本、调度记录、封存账本、`crates/`、别的脚本）与倒推不出的那一份；判决采的每一条，引文由主 agent 在此刻的文件里现查，查过的写在第二节各条里，没查的写明「按线索记」。

两条腿要跑的 17 条命令逐条交了，日志摘在 `~/.local/share/singlefs/work/governance-r1-verify/`：

| 命令 | 任务 | 结局 | 读数 |
|---|---|---|---|
| 攻 1 喂检出钩子 | 19987 | 退 1 | 7 条里 6 条与预期相同：工具员用追加、`replace-once.py`、`cp` 改 `prune-server/src/protocol.rs` 三条都「退 0（应当 0）」，两条对照「退 2（应当 2）」；攻方腿身份 `cp` 盖 `.claude/settings.json` 的对照「退 0（应当 2）」 |
| 攻 2 喂项目代码闸 | 19989 | 退 0 | 带 `SINGLEFS_WORK_ROOT=<仓根>` 起规格工具「退 0（应当 0）」，对照「退 2（应当 2）」 |
| 攻 3 喂写闸 | 19990 | 退 1 | 工具员 Edit `prune-server/Cargo.toml`、`gpu-verification-server/Cargo.toml` 两条「退 0（应当 0）」；`prune-server/tests/` 下已存在的 `.rs`「退 2（应当 0）」；对照 `src/protocol.rs`「退 2（应当 2）」 |
| 攻 4 按键认库探针 | 19992 | 退 0 | 「2 份调用方文件里的 1 处引用，缺 0 处，本次未判 0 处」 |
| 攻 5 它的对照 | 19993 | 退 1 | 「scripts/gate/caller_by_attribute.py:5 引了 project_preflight.shared_preflight_directory，可 .claude/scripts/project_preflight.py 里没有这个名字」 |
| 攻 6 规格工具沙箱 | 19994 | 退 0 | 「✓ 写出 …/q3/fakework/out/settings.json：1 份文件 1 处」 |
| 攻 7 它的对照 | 19995 | 退 3 | 「不在工作根下（认的根：…/q3/otherroot），规格没写」 |
| 攻 8 注释历史探针 | 19996 | 退 0 | 「逐行列出：1 行命中」，只列 `scripts/probe.sh:6` |
| 攻 9 单线程的记号行 | 19997 | 退 0 | 日志里有一行以 `test a_file_read_through_the_session_…` 起头、同一行带 `NOT_RUN_NO_GPU_ADAPTER`；`task-server/task status 19997` 里「本次未跑」0 处 |
| 攻 10 开关 `neutral-base-skipped` | 20003 | 退 1 | 红 2 格，与 `scripts/gate/hook-swap.py:160` 点名的两格逐格相同 |
| 攻 11 开关 `running-swap-not-checked` | 20006 | 退 1 | 红 1 格，与第 156 行点名的相同 |
| 攻 12 开关 `swap-process-pid-only` | 20007 | 退 1 | 红 2 格，与第 158 行点名的相同 |
| 攻 13 开关 `perl-bare-options-take-next-word` | 20009 | 退 1 | 红 3 格，与 `.claude/hooks/project-code-guard.sh:507` 点名的相同 |
| 攻 14 开关 `active-sessions-unlisted` | 20012 | 退 1 | 红 3 格，与第 509、510 行点名的相同 |
| 攻 15 只用 `from … import` 的探针 | 20021 | 退 0 | 「2 份调用方文件里的 0 处引用，缺 0 处，本次未判 0 处」 |
| 正 1 仓副本的条目 | 20022 | 退 0 | 有 `REPOSITORY_COPY entry gpu-verification-server` 与 `REPOSITORY_COPY entry prune-server` 两行 |
| 正 2 换上工具的闭包 | 20023 | 退 4 | 「refused	inline-script-not-literal	scripts/gate/hook-swap.py:1421」 |

## 二、判决

两条腿都有打中的，这一轮不算「没打中」。逐条现查之后分四类：坐实要改的、认下记账的、撤回或不采的、核不动的。

### 坐实、这一轮改

1. **实现员定义里缩域证红那条命令照写会退 2**（正推腿 A、B）。`.claude/agents/implementation-writer.md` 第 3a 步后半句仍写「缩域写成 `task-server/task submit --cwd <副本> -- env <缩域变量>=<n> bash scripts/mutation/prove-red.sh --copy <副本> …`」，而证红脚本对「`--copy` 指的就是起它的那棵树」退 2（`scripts/mutation/prove-red.sh:962-963`「✗ --copy 指的是主工作区」）。这一轮只把同一步前半句与第 1 步改成了从仓根起，漏了这一句。同一个毛病在调查员定义里也有（`.claude/agents/investigator.md:33` 让 `--cwd` 写副本，按线索记、改之前现读）。
2. **工具员在 Bash 里改两个服务 `src/` 下已存在的文件没有闸拦**（攻方腿 L1，任务 19987 三条都放行）。写闸第十一道只挂 Write / Edit，检出钩子 ⑦ 只认脚本、放行追加，⑧ 只认就地改的那几种写法；定义却写「同样不在主工作区改（写闸第十一道执行前拒）」（`.claude/agents/tooling-writer.md:31`）。闸与定义说的不是一回事。
3. **两个服务的 `Cargo.toml`、`Cargo.lock` 不在第十一道的范围里**（攻方腿 L2，任务 19990 两条放行）。理由与 `src/` 相同：分几次改到一半服务编不过。`tests/` 下的 `.rs` 这一条攻方腿推错了：写闸今天就拒（同一任务第三条「退 2」）。
4. **写闸拒绝说明与工具员定义给的 `--verify` 出路，对要临时判定库服务的自证用不了**（攻方腿 O1，正推腿 B 第二条）。这一轮自己就撞过：带验证命令真换 19767 在本机退 5 回滚，之后 `replay.sh` 是不带验证换上的（调度记录第 257 行、现行账本第二百六十一节）。出路写了一条照做会失败的命令，不合「出路本身要过得了别的闸」。
5. **换上工具按键认库有缺口**（攻方腿 K1，任务 19992 与对照 19993 结局不同）：用字符串键从 `sys.modules` 取库、而本文件里没按键存过时，这一处既不核也不列「本次未判」（`scripts/gate/hook-swap.py:864-865`）。与文件头「认不出去向的写法逐处列成本次未判，不当成查过」对不上。
6. **只用 `from 库 import 名字` 引的调用方整份不核**（攻方腿 K3，任务 20021「0 处引用，缺 0 处」）。提早退出那一句早于这一轮，可这一轮的按键认库建在它上面，文件头又承诺「含 from lib import 名字」（`scripts/gate/hook-swap.py:84`）。并进同一批改。
7. **规格工具的 `--out` 能写到哪只看命令前的环境变量**（攻方腿 S1，任务 19989、19994、19995）：项目代码闸放行带 `SINGLEFS_WORK_ROOT=<任意目录>` 的那一条，脚本就把那个目录认成工作根。名单登它的理由「只写工作根下的 `--out`」（`.claude/hooks/lib_project_code_runs.py:1107`、`scripts/gate/README.md:21`）因此不成立。检出钩子那一层的读数不采：那条喂法的对照没红（见「核不动的」第 1 条），分不出是 ⑫ 认不出 `--out` 还是这个身份本来就不在它判的范围里。
8. **规格工具的临时文件跟随符号链接**（攻方腿 S2，现读 `scripts/gate/make-swap-spec.py:201-202`：名字可预测、`open(…, "w")`）。要猜进程号，比上一条难用；与上一条同一份文件，一起改。
9. **任务服务认「本次未跑」记号行的两个缺口**（攻方腿 D1、M1）：三份文件里 `nocapture` 各 0 处（`grep -c` 现数），没有一处写明「测试不带 `--nocapture` 时这一行被测试框架收走、服务读不到」；单线程时记号接在测试框架结果行起头后面，服务只认行首就漏认（任务 19997：日志里有那一行、`task status` 里没有本次未跑那一段），而仓里门禁读同一个记号的库认这一形（`scripts/gate/lib_libtest_result_lines.py:104`，按线索记）。
10. **注释历史判法「面」的例外放得太宽**（攻方腿问 6，任务 19996 只列出对照那一行）：「这一段以前面临内存撞上限，所以按块读。」「以前面向旧布局的那一支已删掉，现在只认新布局。」两句真讲过去，都被放过。`scripts/gate/comment-history-check.py:158` 的例外该只在「以前」前面紧挨「所」时成立。
11. **规格工具几种拒绝分支没有开关也没有自证格**（攻方腿 W1：同一份点两次、绝对路径或带 `..`、`--max-context` 不是正整数、不是 UTF-8、`--out` 是目录或父目录不在、取不到工作根）。新建的脚本，照「每一处会判红的分支配一个弄坏开关或一条变异」补。
12. **两处坏指针与三行读错风险**（正推腿 C）：`.claude/gate.d/crash-point-replay.sh:294` 引 2026-10-23 那份的「第一百四十三节」「第一百三十七节」，`scripts/hosts/run-with-memory-cap.sh:73` 一处，腿查到那份里没有这几个节号（按线索记，改之前现查它们今天在哪一份）；`.claude/gate.d/doc-process-records.sh:602,616,619` 的样本说明把 2026-10-29 那份叫「现行账本」，现读属实，而那一份已封存。

### 认下、记账不在这一轮改

13. **换上工具没有锁**（攻方腿 R1，`grep -c flock scripts/gate/hook-swap.py` 现数 0）：两趟并发时后一趟读不到前一趟还在草稿里验，结局文件会互盖。腿没造出确定性的探针，结局是推的；同类的打补丁工具有锁。认下这是缺口，改法（起手到写完成之间持一把锁）进下一批，改之前先造一个两趟并发的假驱动样本证红。
14. **退 8 把坏的旧钩子诊断成喂法失效**（攻方腿 N1，推的）：有出路（自写喂法），只算误诊。记账。
15. **换上工具的 22 条开关登不进开关复跑表**（攻方腿 W3）：今天仍然成立（任务 20023「refused inline-script-not-literal scripts/gate/hook-swap.py:1421」）。这一轮补跑了三条（任务 20003、20006、20007），红的格与文件头点名的逐格相同；剩下的 19 条在按键认库那一版之后没复跑。记账：改它的下一批把 22 条整份逐条复跑一次。
16. **项目代码闸两条新开关的点名**（攻方腿 W2）：这一轮量了（任务 20009、20012），红的格与文件头点名的各 3 格逐格相同，文件头那两处「推的，没跑」可以改成量过；登记表那一行的理由列没提这两条，随下一次改这份钩子时补。库那一侧写的 `-x`、`-C`、`-d`、`-D` 四种选项没有格，记账。
17. **两台内存上限不同时计时行的「判了」与「本次不判」可以不同**（攻方腿 P1，推的，没量过两台的负载）：不改库里的判定，改的是那一趟的结局行。记账，要量再量。
18. **正文表里的四处错**（正推腿 A）：`comment-history-lag.tsv` 的 `prune-server` 几行是这一轮照门禁报的数改小的（调度记录第 119 行现读属实），表里写成了别的会话的；`task-server/control` 的改动描述、`write-guard.sh` 的行号差一、`replay.sh` 的 `python3 -B` 已在 HEAD，按线索记。正文是冻结证据不回改，更正写在这里。
19. **工具员定义与登记表文件头的次序对不上**（正推腿 B 的推不出）：`.claude/agents/tooling-writer.md:35` 让把要加的那一行写进报告、由门禁那一格去复跑，`.claude/gate.d/break-switch-replay-files.tsv:7` 写「加一行之前先把那份脚本的每一条开关设上跑一遍」。两句各说一半，没有闸拒；随第 1、4 条改定义时把次序写成一句。
20. **正推腿点的三处少改**：`task-server/README.md` 没写 `--nocapture` 缺口（并进第 9 条）；`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py:416` 的帮助串还写「改之前的名单」（现读属实，并进下一批注释存量）；账本侧三处过时句（第 449 行、第 268 行尾巴、第 183 行的数）由主 agent 改账本去向行。

### 撤回或不采

21. 攻方腿说调度记录里「`swap-on-draft-red` 19596 红 9 格」是数错了、日志是 12 行 7 格：点名对得上这一半采，记录那一句的数在调度记录里另写一行更正，不回改原行。
22. 攻方腿的 `tests/` 下 `.rs` 漏放：不成立，写闸今天就拒（任务 19990）。

### 核不动的

1. 任务 19987 里攻方腿身份 `cp` 盖 `.claude/settings.json` 的对照没红。现读检出钩子文件头：「没有写范围的不判：主 agent（输入里没有 agent_type）、表里没有它那几行的定义」，而写范围表 `.claude/hooks/agent-write-scope.tsv` 里没有 `three-way-attack` 的行（`grep` 零命中）。所以只有 Read 与 Bash 的那几类定义（三方腿、回扫员、调研员这类）在 Bash 里往仓里写，⑫ 一律不管。这早于这一轮、也不在被判的 52 份里，是这一轮顺带量到的；它到底罩着多少类定义、Claude Code 自己的权限层对 `.claude/settings.json` 拦不拦，这一轮没量。登进现行账本另立一节。
2. 攻方腿 R1、N1、P1 三条都是推的，没有探针；照上面第 13、14、17 条记账，不当核过。
3. 正推腿没读任务日志，A 表「验证」一栏按调度记录转述；主 agent 这一轮也没有逐条回读那些任务号，只读了这一轮新交的 18 条。

### 这一轮的结局

第一轮有打中的，坐实 12 条。照代码轮与治理文件轮的收法：改完再攻一轮，第二轮派辩方腿复核这一份判决，攻这一轮的修补加上面写明只按线索记、没现查的地方。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 标 |
|---|---|---|---|
| `.claude/agents/implementation-writer.md` 第 3a 步缩域那一句的 `--cwd` 改成仓根；`.claude/agents/investigator.md:33` 同改 | 第 1 条 | 正推腿 | 量过：证红从仓根起、`--copy` 指副本的写法这一轮跑过（任务 19876、19921）都退 0 |
| 检出钩子 ⑦ 另认往两个服务 `src/` 与它们的 `Cargo.toml`、`Cargo.lock` 写已存在文件的写法（追加、`cp`、定点替换工具）；写闸第十一道的源码一类加这两份清单 | 第 2、3 条 | 攻方腿 | 零轮：先造喂法（任务 19987、19990 的那几条预期翻成拒），换上后喂过才算 |
| 写闸拒绝说明与工具员定义的出路按 `.claude/hooks/selftest-temporary-service.tsv` 分两种：登记成要临时服务的写「不带 `--verify` 换上、自证另交一条」，并写明这样换没有自动回滚 | 第 4 条 | 攻方腿 | 量过：不带验证连换、自证单交的走法这一轮用过（任务 19770、19771、19773） |
| 换上工具：按键取库而本文件没存过的列「本次未判」；提早退出的条件把「有 `from 目标库 import`」算作有绑定 | 第 5、6 条 | 攻方腿 | 零轮：攻方腿的三棵探针小树（`q2/root`、`q2/control`、`q2/fromimport`）当样本，改后三条的结局都要翻 |
| 规格工具不认命令前的 `SINGLEFS_WORK_ROOT`（工作根从多机配置读）或挪出直接起的名单；临时文件排他建、不跟链接；补几种拒绝分支的自证格与开关 | 第 7、8、11 条 | 攻方腿 | 零轮：沙箱两条（任务 19994、19995）当样本 |
| 任务服务认行内的记号（行首空白、测试框架结果行起头之后都认，与门禁那一份库同一个认法）；三处文档写明不带 `--nocapture` 读不到 | 第 9 条 | 攻方腿 | 零轮：任务 19997 那一行当样本；这一批改 `task-server/` 的三份此刻别的会话也在改，先发消息对文件 |
| 注释历史判法「面」的例外只在前面紧挨「所」时成立；补两句样本 | 第 10 条 | 攻方腿 | 零轮：探针 `q6/root` 当样本，改后要列出第 2、3 行 |
| 坏指针两处改指今天的落点；样本说明三行改掉「现行账本」的叫法 | 第 12 条 | 正推腿 | 量过的是现状句；指到哪一份改之前现查 |

这一批都是治理文件，没有要写进产品代码的零轮改法，不触发探针里做 A/B 那一条；每条零轮改法的样本先证红再换上。

## 交用户的

没有要用户定的岔路。告知两件：

- 这一轮顺带量到：只有 Read 与 Bash 的那几类子 agent 在 Bash 里往仓里写文件，检出钩子照写范围判的那一道一律不管（核不动的第 1 条）。它早于这一轮，登账本另做。
- 换上工具没有锁、22 条开关没有复跑兜底这两件认下了、没在这一轮改（第 13、15 条）。

## 这一轮被判过的文件

- research/prompts/_ledger-topdown-governance-r1-body.md：这一轮的正文；判了它的范围表，四处写错的更正在第二节第 18 条
- .claude/agent-common.md：判了第 44、100 行两句与写闸、换上工具今天的行为是不是一回事：一致；第 100 行写闸第十一道那一句随第 2、3 条的改法补 Bash 那一半
- .claude/agents/implementation-writer.md：判了第 1b 步与证红命令的起点：第 3a 步缩域那一句漏改，照写退 2（第二节第 1 条）
- .claude/agents/tooling-writer.md：判了第 3 步与第 7 步：`--verify` 的出路对要临时服务的自证用不了（第 4 条），与登记表文件头的次序对不上（第 19 条），「在仓副本里改两个服务」成立（任务 20022）
- .claude/main-agent.md：判了派发表工具员那一行与换上工具今天的行为：一致
- .claude/hooks/write-guard.sh：判了第十一道罩两个服务 `src/` 的那一支：误拒没找到，漏放两处（第 2、3 条），拒绝说明的出路一处过不了（第 4 条）
- .claude/hooks/bash-command-detector.sh：判了文件头十处现状句与账本路径：成立；⑦ 对两个服务 `src/` 的追加与 `cp` 放行（第 2 条）
- .claude/hooks/lib_project_code_runs.py：判了账本路径与规格工具登进直接起的名单：名单的理由不成立（第 7 条）
- .claude/hooks/project-code-guard.sh：判了这一轮点到的几处与两条新开关：红的格与点名的逐格相同（任务 20009、20012），文件头的「推的，没跑」该改成量过（第 16 条）
- .claude/hooks/handback-guard.sh：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- .claude/hooks/lib_run_scope.py：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- .claude/hooks/run-scope-guard.sh：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- .claude/hooks/runner-dispatch-guard.sh：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- .claude/hooks/selftest-registry.tsv：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- .claude/gate.d/code-tooling.sh：判了 runner 表两行与账本路径一行：成立；这一份在腿跑着的时候被别的会话换过，判的三处不在它动的两行上
- .claude/gate.d/break-switch-replay-files.tsv：判了这一轮改的几行条数与文件头开关数：逐份相等；文件头第 7 行与工具员定义的次序对不上（第 19 条）
- .claude/gate.d/comment-history-lag.tsv：判了这一轮改的行：成立；正文把 `prune-server` 几行的归属写错了（第 18 条）
- .claude/gate.d/doc-text.sh：判了这一轮点到的行做的是不是账本那一节说的事：成立
- .claude/gate.d/crash-point-replay.sh：判了文件头第 314–318 行与判器登记、与滤日志行的代码是不是一回事：成立；第 294 行一处坏指针（第 12 条）
- .claude/gate.d/doc-process-records.sh：判了第 457–458 行那一句：成立；第 602、616、619 行样本说明的叫法过时（第 12 条）
- .claude/gate.d/stage-owners.tsv：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- .claude/gate.d/replay-comparison-units.tsv：判了这一轮点到的行做的是不是账本那一节说的事：成立
- .claude/gate.d/fixtures/doc-text.sh/comment-history-past-red/expect：判了这两份新样本与开关 `prefix-compounds-flagged` 对不对得上：成立
- .claude/gate.d/fixtures/doc-text.sh/comment-history-past-red/.gate-cells：判了这两份新样本与开关 `prefix-compounds-flagged` 对不对得上：成立
- .claude/gate.d/fixtures/doc-text.sh/comment-history-past-red/scripts/demo/run.sh：判了这两份新样本与开关 `prefix-compounds-flagged` 对不对得上：成立
- .claude/gate.d/fixtures/doc-text.sh/comment-history-prefix-green/crates/demo/src/lib.rs：判了这两份新样本与开关 `prefix-compounds-flagged` 对不对得上：成立
- .claude/gate.d/fixtures/doc-text.sh/comment-history-prefix-green/expect：判了这两份新样本与开关 `prefix-compounds-flagged` 对不对得上：成立
- .claude/gate.d/fixtures/doc-text.sh/comment-history-prefix-green/.gate-cells：判了这两份新样本与开关 `prefix-compounds-flagged` 对不对得上：成立
- scripts/gate/hook-swap.py：判了退 8、退 9、按键认库三处：按键认库缺口两处（第 5、6 条），没有锁（第 13 条），退 8 误诊（第 14 条）；补跑三条开关逐格相同
- scripts/gate/make-swap-spec.py：判了它能往哪里写与自证罩没罩全：`--out` 只看环境变量（第 7 条）、临时文件跟链接（第 8 条）、几种拒绝分支缺自证格（第 11 条）
- scripts/gate/comment-history-check.py：判了「以前」的四个例外：「面」放得太宽（第 10 条），「缀」「者」造不出反例
- scripts/gate/README.md：判了第 20、21 行：第 21 行「只写工作根下的 --out」随第 7 条不成立
- scripts/agents/agent-watch.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/agents/dispatch-prompt.py：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- scripts/kb/stale-candidates-record-carriers.tsv：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/replay.sh：判了按这一台的内存取并行度、报 host-cap、样本仓取法只读三处：只读那一种在其余 16 格里没有；两台上限不同可改计时行的结局（第 17 条，推的）
- scripts/experiment-apparatus/replay_row_costs.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/e184-segment-one-long-run.sh：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py：判了这一轮改的三行文档串：成立；第 416 行帮助串少改一处（第 20 条）
- scripts/experiment-apparatus/e185_r4_baseline_rejudge.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/e203-admission-scan-cost-run.sh：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/long-run-lib/binary-identity.sh：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/README.md：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/e56-vm-bench-run.sh：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/test_full_table_run.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/current_tree_row_driver.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/test_current_tree_row_driver.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- task-server/task_server.py：判了认「本次未跑」记号行的那几段：只认行首，漏认接在结果行起头后面的（第 9 条）
- task-server/task_server_lib.py：判了 `gpu-cases-not-run` 的提醒种类与异常表：成立；异常表那一行没写读不到的情形（第 9 条）
- task-server/task_server_selftest.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- task-server/README.md：判了第 149 行：没写不带 `--nocapture` 读不到（第 9 条）
- .claude/hooks/lib_shell_words.py：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- scripts/agents/active-sessions.py：判了这一轮点到的行做的是不是账本那一节说的事：成立
- scripts/agents/README.md：判了这一轮点到的行做的是不是账本那一节说的事：成立
- task-server/control：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立
- .claude/gate.d/split-proposal-ledgers.tsv：判了这一轮改的那几处（账本指针或登记行）换成现行账本之后还是不是真话：成立

## 回看决策

不涉及决策：这一轮判的是定义、钩子、门禁、脚本与任务服务的改动，没有拿依据去推翻或确立哪一条决策分项。
