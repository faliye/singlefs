# 检出钩子循环变量、任务服务包着的写工具与退 10、引文核对小节标题、变异脚本分线程：治理文件轮第三轮（最后一轮），攻第二轮判决的修补（wrapped-writers-loop-paths-cite-titles-governance-r3，2026-11-21）

这一轮攻的是第二轮判决（`research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-main-verification.md`）「三、改法」五条已换上主工作区的修补；这是这一题的最后一轮，这一轮新冒出来的形态不再开第四轮，标零轮。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码，改的都是协作工具，这一题没碰 `crates/`。前两轮攻过而没打中的角度不重复：`modifies_workspace` 的判定没变、退 10 两种拒绝说明的认法与 `scripts/gate/hook-swap.py` 现行原文对得上、检出钩子按候选判的 10 格各钉住点名的那一件、变异脚本对服务设的值判得对且不分两遍、起 `--read-outcome` 的起法留在 `SchedulerLifecycle.workspace_write_judged`。

## 问题

1. 每一处修补做的是不是第二轮判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 前两轮正文或判决里只抽查过、标了「推的」「核不动」的地方，与「实现今天的样子」每一条（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 610–611 行）。改前原样与工具员报告在各草稿目录（`base/`、`new/`、`result.md`）。

- 任务服务（草稿 `~/.local/share/singlefs/work/task-server-r2-fix/`，另 `~/.local/share/singlefs/work/task-server-known-break-items/`）：
  - `task-server/task_server_scheduler_building.py` 的 `shell_text_pieces` 把花括号组记进分隔、heredoc 正文的行标上不算平铺着跑，`shell_text_runs` 把花括号组当一层、串里自己带 `env -C` 的那一条目录算不出；三形都落到「算不出、照可能写了一半报」。
  - `task-server/task_server_scheduler_lifecycle.py` 新方法 `hook_swap_read_back`：被杀的 hook-swap.py 起它自己的 `--read-outcome`（带这一条的 `--root` 与 `--scratch`），换完了、已退回的不报；没换完而输出点名的反向规格在同一条命令里另一条换钩子的 `--scratch` 下的，这一条也不报。`subprocess.run(read_back, …)` 留在 `workspace_write_judged`。行为变化一处：被杀在写开始行之前的换钩子（结局文件里还是上一趟的完成）不报了。
  - 退 10 草稿目录被占那一支的出路改成「等占着的那一趟结束再交；两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是），可以换一个 --scratch」，异常表那一行、README 三语第 174 行同改。
  - 新格 `wrapped-patch-directory-unresolvable`、`hook-swap-outcome-judged`；新开关五条（brace-group-not-layered、heredoc-body-as-commands、string-env-chdir-unread、swap-outcome-unread、swap-outcome-owner-ignored），五条名字由主 agent 补进 `task-server/task_server_lib_constants.py` 的 `KNOWN_BREAK_ITEMS`（工具员漏登，第一趟开关复跑五条报「认不出的项」）。
  - 读数：换上验证「18 格都对」，整份自证「455 格都对」，开关复跑「复跑了 10 条开关（1 份）…逐格相同 10 条」。任务服务已重启。真任务上的一次：主 agent 的换上撞上别的会话在换、退 10，服务推的提醒写「这个仓的换上正被另一趟 hook-swap.py 占着…换草稿目录没有用」，与日志拒绝说明相符。
  - 工具员写明没做的：两条换钩子用同一个 `--scratch` 时分不出是哪一条、两条都点名（偏严）；算术里的 `<<` 被当成 heredoc、之后的行都算不出（偏严）；脚本正文的路径不跟 cd。
- 引文核对（草稿 `~/.local/share/singlefs/work/cite-check-r2-fix/`、`~/.local/share/singlefs/work/cite-check-header-named-cell/`）：`scripts/agents/cite-check.py` 带反引号的位置、标题带「，」「。」而文件里没有这样整行标题的判红，不带反引号的才列进没判；不带反引号的标题后紧跟括号说明，整串命中 0 次时逐层去掉末尾括号、去掉后是恰好一处的整行标题就取它（同名几处的不取）；兜底只收小节写法，理由分括号没配上、标题超出八十个字、结尾认不出三种。新开关五条。读数：换上验证的自证过；主 agent 复测一条带反引号、标题带逗号的伪造引文，末行「核了 1 处引文、0 处裸行号，1 处对不上，0 处没判」退 1；开关复跑 17 条逐格相同（第一趟 `title-parens-unpaired` 多点名一格、头注删了那一处）。工具员写明没做的：不带反引号、位置紧挨引号的散文仍走严格那一支。
- 检出钩子：`.claude/hooks/bash-command-detector.sh` 第 224 行「同名的这几种在也照按候选判」改成「同名的这几种在时也照按候选判」，只这一处。
- 变异脚本：`scripts/mutation/mutate.sh` 文件头「并行」那一段改成「这一路一条任务里各工作进程的测试线程合计不超过线程上限；用例显式要的原样传给每个工作进程，合计是显式值 × 工作进程数，可以超过线程上限」，只这一处。
- 这一轮之后的事：算键库别的会话把超折叠上限改成了拒算（新拒算种类 fold-limit-exceeded），与这一轮被判的文件无关。

## 范围

这一轮被判的文件（14 份，逐个列在 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 任务服务算当前目录：三形改成「算不出」之后，有没有平常写法被连带降成算不出、因而正常打全了的补丁也照「可能写了一半」报（多报）；花括号组、heredoc、`env -C` 之外还有没有算出错目录的形态（子 shell `( … )`、`cd` 在函数里、`&&` 链上前一条失败、`time`/`nice` 这类前缀后的 cd）。
2. `hook_swap_read_back`：`--read-outcome` 读的是按仓只有一份的结局文件，被杀的这一条与结局文件里记的那一趟是不是同一趟靠什么认（`--scratch`、进程号、时刻），认错时是多报还是漏报；「被杀在写开始行之前不报」会不会漏掉真写了一半的（开始行写了一半、换了第一份还没记）；反向规格在另一条的 `--scratch` 下那一支的判法；两格新格与五条新开关红在名字说的那件事。
3. 退 10 出路的新句对四种组合（这一条干跑或真换 × 占着的那一趟干跑或真换）各成不成立；提醒、异常表、README 三语三处说的是不是一回事。
4. 引文核对：带反引号判红之后，有没有合法写法被误红（反引号里标题后面跟了行号、标题本身就带「，」而文件里是别的层级的标题、同名标题两处）；逐层去括号有没有取到错的那一节（去掉的括号其实是标题的一部分、去掉之后撞上另一节）；兜底不收散文之后，此前列进没判的真小节写法有没有掉出去不判也不列。

正推腿（逐条回「成立」「不成立」「推不出」）：

5. 「实现今天的样子」每一条：今天的文件是不是这样写的（写出 文件:行 与那一句原文）。
6. 第二轮判决里「推的」「核不动」的逐条现查：三形算错当前目录在改前的代码上是不是真算错（对着草稿 `base/` 读码，写出每一形算出什么）、正常结束的换钩子被点名在改前是不是真会发生；辩方腿七条补充探针没跑的，能用只读办法核的核掉。

## 分工

攻方腿做第 1 到 4 问，正推腿做第 5、6 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
