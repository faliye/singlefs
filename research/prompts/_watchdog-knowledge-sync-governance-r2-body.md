# 看门狗定时回报加「知识同步提醒」：治理文件轮第二轮，攻第一轮判决的修补（watchdog-knowledge-sync-governance-r2，2026-11-18）

这一轮攻的是第一轮判决（`research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md`）坐实的 9 条与按线索记的 2 条的修补，都已换上主工作区；辩方腿复核第一轮判决。第一轮攻过而没打中的角度不重复：改名两向、起来时改着之后提交又改、`git stash` 与 `stash pop`、换分支再换回、新提交删文件、只改 mtime、行尾转换、索引锁被占、读哪个仓的三级取法本身。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 每一处修补做的是不是第一轮判决「三、改法」那一行说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 第一轮判决够不够得着：坐实的 9 条理由成不成立，按线索记的 2 条与不采的 4 条判得对不对（辩方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；任务号后面的读数是任务日志里的原句）：

- 这一轮没有改 `crates/` 下的代码，改动只在 `scripts/agents/agent-watch.py`、`scripts/agents/watch.conf`、`.claude/skills/dispatch/SKILL.md` 与两张登记表各一行；`scripts/agents/watch.sh` 没改。
- **两批修补**：第一批（工具员报告 `~/.local/share/singlefs/work/watchdog-knowledge-sync-r1-fixes/result.md`）照判决九行改法改，任务 20363 换上、换上后自证绿；第二批（`~/.local/share/singlefs/work/watchdog-knowledge-sync-r1-fixes-key/result.md`）只改自证里把给人抄的命令交给 shell 的那一格：原来 `["bash", "-c", command]`，门禁算键整份拒算（任务 20364「118 条里拒算 118 条」），改成写进临时目录里的 `manual-command.sh` 再交 `bash`，任务 20378 换上、换上后自证绿。
- **落盘与按时带这一节**：`knowledge_state_path`（`scripts/agents/agent-watch.py:1751`）取会话目录（`--session-dir`，没给取被看的子 agent 会话记录的上两层），文件名 `KNOWLEDGE_STATE_FILE_NAME = "agent-watch-knowledge-sync.json"`（`:618`）；`save_knowledge_state`（`:1767`）`mkstemp` 再 `os.replace`；`load_knowledge_state`（`:1787`）核格式与仓；`knowledge_sync_section`（`:2041`）告警退 3 与定时回报退 4 都调，离上次提醒满 `--max-minutes` 才带，带了换基线、记时刻、落盘。
- **被 git 忽略的范围条目**：`ignored_knowledge_paths`（`:1713`）用 `git ls-files -o -i --exclude-standard` 列范围里被忽略的文件，逐个算摘要，新标签「被 git 忽略、内容摘要变了」。工具员现查本仓此刻列出 11 份，全在 `.claude/singlefs-ai-sop/rules/`。
- **读状态**：改成 porcelain 加 `-z`；不继承的 git 变量补了 `GIT_COMMON_DIR` 等五个；给人抄的命令每条单独一行、带 `-c core.quotepath=off`，「核手上的文件」给 `python3 scripts/agents/who-touched.py <路径>…`；起来时有摘要、摘要没变的不点名；算摘要只读普通文件、整次带时限。
- **技能**：`.claude/skills/dispatch/SKILL.md:39` 改成五种来源标签、退 3 与退 4 满了才带、读到之后先起下一只看门狗再重读再核。
- **验过的**：第一批换上后自证（任务 20363）、第二批换上后自证（任务 20378）都退 0；`watch.sh` 自证任务 20365 退 0；任务 20379（`bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/agents/agent-watch.py`）退 0，成功句「✓ 脚本与钩子的弄坏开关复跑：复跑了 118 条开关（1 份），自证红且点名的格与实际红的格逐格相同 118 条」，「按条复用：现判 119、复用 0、拒算 0」，这一格用时 790.1 秒。
- **工具员写明没做或推的**：文件头退出码那一行「4 定时回报（没有告警、子 agent 还在跑…）」在 `--discover` 找不到在跑的子 agent 时不成立，没改；`--processes-only` 没给 `--session-dir` 时不落盘；几只看门狗同时盯一个会话时谁先满谁打，没加锁；只在基线时干净、之后只改执行位的文件照状态码点名；`reset --soft` 那一形不改。
- **主 agent 打的两行**：`.claude/hooks/selftest-registry.tsv` 里 `agent-watch.py` 那一行改了自证起的进程说明（两次定点替换）；`.claude/gate.d/break-switch-replay-files.tsv` 里 `agent-watch.py` 那一行改成一百一十八条、四族、任务 20379 复跑的读数。后一行是主 agent 用 python 整行换的（断言恰好一行以 `scripts/agents/agent-watch.py\t` 起头、第二列是 600），不是经 `scripts/kb/replace-once.py`。

## 范围

这一轮被判的文件（都读换上之后的那一版）：

- `scripts/agents/agent-watch.py`：文件头「知识同步提醒」一段与开关表里这两批新加、改过点名的开关；`run_git` 到 `knowledge_sync_section` 各函数、两处调用点、`run_selftest_shell_command`（`:4100`）与自证 `selftest_repository_report`。这份文件里这一轮之前就有的别的检测不在这一轮判。
- `scripts/agents/watch.conf`（`repository-report-lines` 那几行注释）
- `.claude/skills/dispatch/SKILL.md`（「派出去之后」里讲「知识同步提醒」的那一段）
- `.claude/hooks/selftest-registry.tsv`（`agent-watch.py` 那一行）
- `.claude/gate.d/break-switch-replay-files.tsv`（`agent-watch.py` 那一行）

这两批的改动看 `diff <(git show HEAD:scripts/agents/agent-watch.py) scripts/agents/agent-watch.py`，第一轮开工快照里这份文件的 sha256 在 `research/prompts/watchdog-knowledge-sync-governance-r1-snapshot/judged-sha256.txt`。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中，打中给能照喂的输入或 文件:行）：

1. 落盘：会话目录取不到、两只看门狗同时起、同时到点、前一只被杀在 `os.replace` 前后、落盘文件被手改或截断、换了仓、换了会话，各是什么结局；第一次起（没有落盘文件）时「上次提醒的时刻」取什么，第一次提醒在起来之后多久出；连着几只按告警退出时，提醒是不是每满 `--max-minutes` 至少出一次（给一串退出时刻算一遍）。
2. 被忽略的条目：`git ls-files -o -i --exclude-standard` 在范围里列出大文件、很多文件、符号链接、命名管道时的耗时与结局；被忽略的文件删掉、新加，标签与来源对不对；整次摘要的时限到了，这一节写什么、基线换不换。
3. porcelain 加 `-z` 之后：改名与复制（`R`、`C`）的两段、带换行的路径、子模块行、未跟踪目录整行，读成的键对不对；第一轮的配置三形（`status.showUntrackedFiles`、`color.status`、`status.branch`）还改不改得掉读数；`git diff --name-only -z` 那一边同样核。
4. 「摘要没变就不点名」：会不会把真的变过又改回去、或只在索引里变了的文件漏掉（索引里改了、工作区原样），这算不算该点名；基线时没摘要的路径走状态码那一支，状态码判法这一轮有没有变。
5. 给人抄的命令：写进临时脚本交给 `bash` 这一改，自证还核不核得出「照抄能跑」（例：命令里带 `$`、反引号、单引号的路径）；新开关 `repositorymanualcommandssemicolon` 是不是只红点名的那一格；`who-touched.py` 那一行照抄过不过项目里管 Bash 的几道闸。
6. 文件头、技能段、`watch.conf` 注释与这两批改完的行为逐句对得上没有（五种标签的字面、哪些退出带、「→ 同步：」那一行、「→ 接着盯」在没有子 agent 时有没有）；开关表这两批新加、改过点名的每条写的「弄坏哪一处」是不是那一处。
7. 第一轮判决写明只按线索记的两处：命名管道卡死那一条，修补后的自证格真能造出卡死吗（第一轮的探针走不到，因为小仓里 `.claude/` 整个未跟踪）；自证时序窗口那一条，「先落盘基线、再改仓、改完才起」之后还剩不剩靠固定时间的地方。

辩方腿（逐条回「判决站得住」「判决够不着」「推不出」）：

8. 第一轮坐实的 9 条：每条的理由是不是同样打中别的写法（例：第 2 条改 porcelain 是不是唯一出路）；第 7 条「该提醒而不提醒」用户定的改法对第一轮攻方腿列的四种改法里另三种，代价比得对不对。
9. 第一轮按线索记的 2 条（命名管道、时序窗口）与核不动的 1 条（行数上界）：判「按线索记、一并改」够不够，还是该先量再改。
10. 第一轮不采的 4 条（不列此刻全部路径、「多做 6 项」、超时变量设成非数字、行宽不封顶）：替「该采」那一方写出最强的理由再判。
11. 这两批修补是不是照判决「三、改法」做的：多做、少做、做偏的逐条写；工具员写明没做的几件（上面「实现今天的样子」最后两条）判该不该算这一轮的缺口；主 agent 用 python 整行换登记表那一行，与「几个会话共写一批文件时只许经 `scripts/kb/replace-once.py` 改」那一条（`CLAUDE.md`「项目本地事实」表 `scripts/kb/replace-once.py` 那一行）对不对得上。

## 分工

攻方腿做第 1 到 7 问，辩方腿做第 8 到 11 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。要跑的探针不自己跑，照「要主 agent 跑的命令」写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条，探针只碰自己草稿目录里造的小仓，不碰真仓与真的会话目录。
