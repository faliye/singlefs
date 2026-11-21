# 看门狗定时回报加「知识同步提醒」：治理文件轮第一轮（watchdog-knowledge-sync-governance-r1，2026-11-18）

这一轮判的是看门狗 `scripts/agents/agent-watch.py` 定时回报里新加的「知识同步提醒」一节与它带动的五份文件，都已换上主工作区。用户要的是：主对话每小时同步一次项目最新变动与状态，免得做久了知识不同步；同步的内容是最新的 `git status`；只加在看门狗里，没派过子 agent 的对话不罩。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 换上的实现做的是不是文件头「知识同步提醒」那一段与技能那一段说的事（正推腿）。
2. 哪一格会错：该提醒而不提醒、点名漏了变过的知识类文件、点名了没变的、把看门狗拖慢或弄挂、文档与代码对不上（攻方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；任务号后面的读数是任务日志里的原句）：

- 这一轮没有改 `crates/` 下的代码，新加的代码也不读 `crates/` 下的任何文件：`grep -n "crates/" scripts/agents/agent-watch.py` 命中的行都在这一轮之前就有的段里，「知识同步提醒」那一段（文件头第 120 到 139 行、函数第 1463 到 1734 行）零命中。
- **哪里打**：`scripts/agents/agent-watch.py:2159` 与 `:2658` 两处（`--agents` 与 `--processes-only` 两路）只在 `watch` 模式起来时调 `take_repository_baseline`（`:1577`）记一份基线；到点的定时回报（退 4）调 `print_repository_section`（`:1732`）。告警退 3、全部交回退 0、`report` 那几路不打。
- **基线记什么**：`repository_snapshot`（`:1559`）记分支、HEAD、`git status --short` 的 {路径: 两位状态码}（`parse_status_short`，`:1483`，取每行第 4 个字符起当路径，改名行「旧 -> 新」整段当键），与落在知识类文件范围里的各路径的内容摘要（`content_digest`，`:1537`）。
- **怎么判变过**：`knowledge_files_changed`（`:1602`）取两处并集：起来之后的新提交动到的路径（`new_commits_since`，`:1592`，`git diff --name-only --no-renames <起来时的 HEAD> HEAD`）里落在范围内的；此刻 `git status --short` 里相对基线新出现、状态码变了、状态码没变而内容摘要变了的。基线里有、此刻没有的另标「已不在」。范围外的路径只比状态码（`paths_changed_since`，`:1641`）。
- **范围与次序**：两张表 `KNOWLEDGE_SCOPE`、`KNOWLEDGE_PRIORITY`（`:535`、`:538`），以 `/` 结尾的条目按前缀认、别的整条相等（`matches_scope_entry`，`:1498`）。
- **git 怎么起**：`run_git`（`:1463`）带 `-c core.quotepath=off`、`GIT_OPTIONAL_LOCKS=0`，时限默认 30 秒，起不来、超时、退非 0 都抛 `RepositoryUnreadable`，那一节改写「没查成」与手动看的命令，照旧退 4。读哪个仓：`repository_root_of`（`:1571`）。
- **清单上限**：`--repository-report-lines`（默认 20，`watch.conf` 同名键），每张清单最多列这么多条、余下报条数；写 0 不记基线、不打这一节。`scripts/agents/watch.sh:76` 的 `KNOWN_CONFIG_KEYS` 加了这个键，`:216` 自检用的配置写 0。
- **技能**：`.claude/skills/dispatch/SKILL.md:39` 写读到之后怎么做（逐份重读点名的文件、照末尾「→」核手上的活、再起下一只看门狗）。
- **两张登记表**：`.claude/hooks/selftest-registry.tsv` 里 `agent-watch.py` 那一行的说明补了自证造临时小仓、起 git、拿只睡的 sh 顶替 git；`.claude/gate.d/break-switch-replay-files.tsv` 里 `agent-watch.py` 那一行改成一百条开关（四族）。
- **验过的**：换上任务 19904、20080 退 0 且换上后自证绿；`watch.sh` 自证任务 19905 绿；任务 20085（`bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/agents/agent-watch.py`）退 0，成功句「✓ 脚本与钩子的弄坏开关复跑：复跑了 100 条开关（1 份），自证红且点名的格与实际红的格逐格相同 100 条」。真仓上到点的输出见过三次，形态是提醒句、逐份点名、「仓的现状」、末尾「→」。
- **主 agent 已知、没改的两处**（交腿判该不该算缺口）：这一节只在「一轮 `--max-minutes` 里没有告警」的定时回报里打，看门狗按告警退 3 的那一轮不打，基线随那一只看门狗一起没了，下一只从它起来那一刻重新记；没派过子 agent 的对话没有看门狗，收不到（用户定的范围）。

## 范围

这一轮被判的文件（都读换上之后的那一版）：

- `scripts/agents/agent-watch.py`：文件头「知识同步提醒」一段与开关表里 `repository` 起头十一条、`knowledge` 起头六条；函数 `run_git` 到 `print_repository_section`；两处调用点；自证 `selftest_repository_report`（`:3736` 起）；长选项 `--repository-report-lines`、`--repository-root`（`:4551` 起）。这份文件里这一轮之前就有的别的检测不在这一轮判。
- `scripts/agents/watch.sh`（`:76`、`:216` 两处）
- `scripts/agents/watch.conf`（`repository-report-lines` 那两行）
- `.claude/skills/dispatch/SKILL.md`（「派出去之后」里讲「知识同步提醒」的那一段）
- `.claude/hooks/selftest-registry.tsv`（`agent-watch.py` 那一行）
- `.claude/gate.d/break-switch-replay-files.tsv`（`agent-watch.py` 那一行）

这一轮的改动相对 HEAD 的样子：`git diff HEAD -- <上面六份>`。各批工具员的报告（改了什么、没做什么）在 `~/.local/share/singlefs/work/` 下：`watchdog-git-status-reminder/result.md`、`watchdog-knowledge-sync-reminder/result.md`、`watchdog-break-switch-header-names/result.md`。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（一串 git 操作、一个路径名、一份配置）或 文件:行。

正推腿：

1. 文件头「知识同步提醒」一段的每一句现状句，逐句对函数：次序、来源标签四种、计数怎么数、上限怎么起作用、写 0、只在定时回报里打、读哪个仓、查不成时写什么，代码做的是不是这一句。
2. 开关表里这十七条：每条写的「弄坏哪一处」是不是代码里那一处，点名的自证格在 `selftest_repository_report` 里在不在。
3. 技能那一段、`watch.conf` 那两行注释、两张登记表那两行，与代码今天的行为逐句对得上没有（范围八项、标签的字面、默认值 20、开关条数与四族各几条）。
4. 用户要的三件（每小时提醒主对话同步、内容是最新的 git status、只加在看门狗里）各落在哪一行；有没有多做的（用户没要而加了行为的）。

攻方腿：

5. 漏点名：造得出「知识类文件在这一轮里变过、到点却不点名」的 git 操作序列吗？至少试这几形：路径带空格或引号（`git status --short` 仍加引号的那一类）；改名（状态行「旧 -> 新」，一头在范围里、一头在范围外）；整个未跟踪目录被写成一行而范围条目在它底下（例：未跟踪的上层目录里新出现一份规则）；起来时就改着、之后提交了又接着改；`git stash`、`git reset --soft`、换分支再换回来；子模块或被忽略的文件；新提交里删掉了一份知识类文件。
6. 误点名与噪声：造得出「没变而被点名」的吗（只有 mtime 变、行尾转换、别的会话暂存又取消暂存）；真仓此刻几百个未提交路径，那一节最长会打多少行、每张清单各自封顶之后合计封不封得住。
7. 拖慢与弄挂：基线与到点各起几条 git、对范围内的路径逐个算 sha256（`.claude/kb/` 底下未跟踪的大目录、指向仓外的符号链接、读的时候文件正被别的会话写）；git 被别的会话的提交锁住、磁盘满负荷时超 30 秒，结局是「没查成」还是看门狗退别的码；`repository_status_section` 里除 `RepositoryUnreadable` 之外抛得出别的异常吗（`OSError`、解码、`KeyError`），抛了的话定时回报还打不打、退几。
8. 该提醒而不提醒：告警退 3 的那一轮不打、基线丢掉——连着几只看门狗都按告警退出时，主对话多久收不到一次；它算不算没达到用户要的「一个小时同步一次」，改成什么样能达到（例：告警退出时也带这一节，或把基线落盘跨看门狗接着比），改法各有什么代价。只判，不改。
9. 多机与环境：`repository_root_of` 的三级取法在第二台、在仓副本里起、在 `TASK_SERVER_SOURCE_REPOSITORY` 不是这个仓时各读哪个仓；`GIT_DIR` 这类变量不继承之后，在 git worktree 里起读得对不对。
10. 自证与开关：`selftest_repository_report` 起真 git、造临时小仓，过不过 `scripts/gate/selftest-process-guard.py` 盯的那几样；按就绪文件等、`--max-minutes` 写小数这类时序在磁盘慢的时候会不会偶发红；十七条开关里有没有两条弄坏同一处、或点名的格其实靠别的开关也会红（分辨不出是哪条坏了）。
11. 技能那一段的出路：「最后起下一只看门狗」在没有子 agent 在跑时还该不该起；「写着没查成的，照它给的命令自己看仓」给的命令过不过项目里管 Bash 的那几道闸。

## 分工

正推腿做第 1 到 4 问，攻方腿做第 5 到 11 问。两条腿都读「范围」里的六份文件；只有一条腿读到的范围在报告里写明。要跑的探针（临时小仓里的 git 操作序列、喂看门狗的命令）不自己跑，照「要主 agent 跑的命令」写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条。
