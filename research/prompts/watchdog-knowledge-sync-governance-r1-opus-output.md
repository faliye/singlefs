# 看门狗「知识同步提醒」治理文件轮第一轮：攻方腿报告（watchdog-knowledge-sync-governance-r1，2026-11-18）

攻击面：背景材料「要腿判的」第 5 到 11 问。被判的六份文件读主工作区此刻的那一版，sha256 与开工快照 `research/prompts/watchdog-knowledge-sync-governance-r1-snapshot/judged-sha256.txt` 里六行逐行相同（`sha256sum` 现算、并排比过）。这一轮不动 `crates/`，探针不调崩溃枚举，崩溃状态数为 0（第 3b 步的崩溃状态那一条在这一轮没有对象）。

## 复跑命令与模型文件

模型目录 `research/prompts/watchdog-knowledge-sync-governance-r1-opus-model/`：

| 文件 | sha256 | 行数 |
|---|---|---|
| probe/knowledge_sync_probe.py | 5746192f06be2d26efc8ab536af8308a2b2b590af169a4aab3476a11ed4301b7 | 593 |
| tools/rerun.sh | a19f2db3eeee002e227408d0d4262dbca7b98b8b24416ba9bab0beb9dff3a2e4 | 179 |
| tools/rerun-from-a-clean-work-directory.sh | 6eda0c62832e27e63fc918d14eb00d5221e211f8dca7b0108c00258cbf4592f0 | 24 |

复跑（整条交服务，命令原样在末尾「要主 agent 跑的命令」一节）：`bash research/prompts/watchdog-knowledge-sync-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录，工作根下、以 -run 收尾> <模型目录>`。这一轮没有冻结副本目录，`<冻结副本>` 填仓根：探针第一行 `PROBE name=watchdog` 打出它导入的 `agent-watch.py` 的 sha256，主 agent 对 `judged-sha256.txt` 那一行（`1c46829b…`）核一遍再用读数。探针只在运行目录底下建小仓，git 一律带 `GIT_CEILING_DIRECTORIES=<运行目录>`、不读用户与系统配置（三个配置场景故意给自造的全局配置）。

`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：导入冻结副本里的 `agent-watch.py`，直接调 `take_repository_baseline` 与 `repository_status_section`（看门狗起来时、到点时调的同一对函数），每个场景中间照场景做 git 操作；末行完整性闸（`PROBE name=done emitted=N` 的 N 要等于前面的 PROBE 行数，对不上退 3）。估时（推的，没跑）：32 个场景（打 33 行 PROBE，另加末行）都是几条 git 的小仓，加上 2 GiB 稀疏镜像读一遍两次、命名管道那一格等满 20 秒，整趟估一分钟以内；内存只在读镜像时每次 1 MiB 一块。下文凡是「探针 <场景名>」的读数都没跑出来，一律标「推的，待主 agent 跑」。

## 各格判定一览

| 问 | 判定 | 打中（能照喂的输入或 文件:行） | 量的 / 推的 |
|---|---|---|---|
| 5 漏点名 | 推不出（打中两条） | 漏点名-上游规则：范围条目 `.claude/singlefs-ai-sop/rules/` 落在 `.gitignore:8` 整个忽略的目录里，那几份规则怎么改都不点名；漏点名-带空格的路径：工作区里路径带空格（或引号、反斜杠）的知识类文件，`git status --short` 给它加引号，前缀对不上，不点名 | 忽略那一条是量的（`git ls-files`、`git status --ignored` 现查）；看门狗那一节的读法推的，待探针 ignored-parent、space-untracked-new、space-tracked-rewritten、quote-committed |
| 6 误点名与噪声 | 推不出（打中一条） | 内容一字没变也点名：只暂存、只改执行位、`git reset --soft HEAD~1`、未跟踪知识类目录里只有被忽略的文件变了；行数封得住（每张清单 20 条时最多 70 行），行宽封不住 | 推的，待探针 stage-only、chmod-only、reset-soft、ignored-inside-untracked、line-bound |
| 7 拖慢与弄挂 | 推不出（打中一条） | 算内容摘要不在 30 秒时限里：未跟踪的知识类目录底下有命名管道，记基线就卡死，看门狗既不退 3 也不退 4；有被忽略的大镜像、编译目录，按大小线性拖慢；git 超时与起不来照写「没查成」退 4（成立）；`RepositoryUnreadable` 之外抛不出别的异常（推的） | 推的，待探针 fifo-in-untracked-directory、sparse-ignored-image、git-count、index-lock |
| 8 该提醒而不提醒 | 不成立（没达到「每小时一次」） | 只有同一只看门狗连着 60 分钟没有告警才打这一节；按告警退 3 一次，基线就随它没了；连着按告警退的那段时间里主对话一次都收不到，上界不存在 | 推的（读码）；告警频率没量 |
| 9 多机与环境 | 成立（读哪个仓照文件头），另有一处推不出 | 三级取法照文件头；`run_git` 不关用户与系统的 git 配置（自证关了），`status.branch`、`status.showUntrackedFiles`、`color.status` 这类全局配置改得了这一节的读数 | 推的，待探针 root-precedence、worktree-inherited-variables、global-config-* |
| 10 自证与开关 | 推不出（打中两条） | 开关分辨不开：`knowledgetagsource` 与 `knowledgeworktreeblind` 点名的格逐字相同；没格盯：读哪个仓的三级取法、去掉 `GIT_DIR` 那三个变量、未跟踪目录的摘要、知识类文件的改名都没有格，自证的小仓又没有 `.gitignore`，漏点名-上游规则那一条照样绿 | 两行相同是量的（`cmp` 退 0）；没格那几处是读码 |
| 11 技能那一段的出路 | 推不出（打中两条） | 手动命令跑不起来：给的两条命令中间是全角分号，照原样交给 bash 跑不起来，也没带 `core.quotepath`；核那一步没命令：「核手上正在改的文件」没给命令；「最后起下一只」与看门狗自己的出路句在没有子 agent 时对不上 | 推的，待探针 manual-commands-verbatim、remainder-command-verbatim；出路句那一条读码 |

## 第 5 问：漏点名

### 漏点名-上游规则：`.claude/singlefs-ai-sop/rules/` 在真仓里看不见

- 范围表写着它：「".claude/kb/", ".claude/singlefs-ai-sop/rules/")」（`scripts/agents/agent-watch.py:536`）；技能那一段也把它列进知识类文件（`.claude/skills/dispatch/SKILL.md:39`）。
- 真仓把整个目录忽略：「/.claude/singlefs-ai-sop/」（`.gitignore:8`）。现查（量的，主工作区，只读）：`git ls-files -s .claude/singlefs-ai-sop | wc -l` 打 `0`；`git -c core.quotepath=off status --short --ignored -- .claude/singlefs-ai-sop` 打 `!! .claude/singlefs-ai-sop/`；`git ls-files -o -i --exclude-standard --directory` 在八个范围条目上只列出这一个目录；目录里没有 `.git`（不是子模块）；`ls .claude/singlefs-ai-sop/rules | wc -l` 打 `11`。
- 两处来源都读不到它：`git status --short` 不带 `--ignored` 不列被忽略的路径，`git diff --name-only <起来时> HEAD` 只列跟踪的路径。CLAUDE.md 用 `@` 引的七份上游规则（`CLAUDE.md` 第 25 到 31 行，`grep -n '^@'` 现查）都在这个目录里，换包、改规则，这一节一个字都不写；能看见的只有仓根被跟踪的 `.singlefs-ai-sop-version`（不在范围里，只会落进「别的路径」那张清单）。
- 能照喂的输入：在真仓里改 `.claude/singlefs-ai-sop/rules/sop-first.md` 一个字；探针 ignored-parent 在小仓里照 `.gitignore:8` 造同一形（推的，待主 agent 跑；预期 `named_count=0 other_count=0`）。

### 漏点名-带空格的路径：工作区那一半认不出加了引号的路径

- 本机 git 2.43.0（`git --version` 现查）的 `git-status` 手册（`/usr/share/man/man1/git-status.1.gz` 解压后，转述）写：短格式里文件名含空白或不可打印字符的，那一栏按 C 字符串加双引号。`-c core.quotepath=off`（`scripts/agents/agent-watch.py:1471`）只管非 ASCII 字节，管不到空格、双引号、反斜杠、制表符。
- 认法：「return {row[3:]: row[:2] for row in text.split("\n") if len(row) >= 4}」（`scripts/agents/agent-watch.py:1485`）原样把 `".claude/kb/a b.md"`（带引号）当键；「return path.startswith(entry) if entry.endswith("/") else path == entry」（`scripts/agents/agent-watch.py:1500`）拿带引号的串比前缀，对不上，算范围外。
- 结果分三形（推的，待探针）：
  - space-untracked-new：起来之后新建 `.claude/kb/a b.md`。不点名，只在「别的路径」里出现一行 `新出现 [??] ".claude/kb/a b.md"`；提醒句写「知识类文件没变，没有要重读的」。
  - space-tracked-rewritten：起来时已改着、之后又改内容（状态码不变）。范围外只比状态码，哪里都不出现，完全漏掉。
  - quote-committed：新提交加一份 `.claude/rules/say"hi".md`。`git diff --name-only` 同样给双引号加引号（这一条是推的，没核手册），新提交那一半也漏。对照：space-committed（新提交加 `.claude/kb/c d.md`）预期照常点名——`diff --name-only` 不给空格加引号（推的）。
- 仓里已有路径带空格的文件：`git ls-files .claude | grep ' '` 打出一份 `.claude/gate.d/fixtures/doc-decisions.sh/kb-shape-red/.claude/kb/空格 文件.md`（量的；在范围外，只说明本仓会造这种路径）。

### 试过、没打中的形状（第 5 问）

| 形状 | 读法 | 依据 |
|---|---|---|
| 改名，一头在范围里一头在范围外（rename-in-to-out、rename-out-to-in） | 「旧 -> 新」两头拆开，在范围里的那一头点名「工作区改了没提交」；范围外那一头不再进「别的路径」 | 读码 `status_paths_of`、`scripts/agents/agent-watch.py:1618`；推的，待探针 |
| 整个未跟踪目录写成一行、范围条目在它底下 | 本仓不可达：八个范围条目的上层目录都是跟踪的（`.claude/` 跟踪，`CLAUDE.md` 在仓根）；唯一的上层目录是被忽略的那一个，已归漏点名-上游规则。范围里新出现的未跟踪目录（untracked-new-directory）点名的是目录 `.claude/skills/new/`、不是底下那份 `SKILL.md`；起来时就未跟踪的目录里又多一份（untracked-directory-grows）照摘要点名目录 | 读码 `scripts/agents/agent-watch.py:1545`；推的，待探针 |
| 起来时改着、之后提交了又接着改（modified-commit-modify） | 「两样都有」 | 推的，待探针 |
| `git stash`（stash-only）／`git stash` 再 `pop`（stash-then-pop） | 前者标「起来时未提交的改动已不在 git status 里」；后者工作区回到起来时的内容、不点名（不算漏：主对话读到的文件没变） | 推的，待探针 |
| `git reset --soft`（reset-soft） | 点名，但内容没变，归第 6 问 | 推的，待探针 |
| 换分支再换回来（branch-roundtrip） | HEAD 回到起来时，`start_snapshot.head_hash == now_snapshot.head_hash`（`scripts/agents/agent-watch.py:1595`），不点名，不算漏 | 推的，待探针 |
| 换到分叉的分支不回来（branch-switch-away） | 那条分支上起来之前就有的提交被列成「看门狗起来之后的新提交：1 条」，它动过的规则标「新提交动到」：点名了，标签与「新提交」的说法不对 | 推的，待探针 |
| 子模块 | 本仓没有子模块，这一形在本仓不可达 | 量的：`git ls-files -s` 里模式 160000 的行现数为 0，仓根没有 `.gitmodules` |
| 新提交删掉一份规则（commit-deletes） | 点名、标「新提交动到」，出路叫主对话「重读」一份已经不在的文件，标签没说它删了；不算漏 | 推的，待探针 |
| 未跟踪的知识类目录里指到仓外的链接，仓外那一头改了（symlink-outside） | 不点名：`os.walk` 不跟目录链接，文件链接只记链接指向的路径；只在有人把知识类文件做成链接时才碰得到 | 读码 `scripts/agents/agent-watch.py:1545`；推的，待探针 |

## 第 6 问：误点名与噪声

### 内容一字没变也点名

判法是「状态码变了，或状态码没变而摘要变了」：「if scoped_parts and (start.status_by_path.get(key) != code or digest_changed):」（`scripts/agents/agent-watch.py:1618`）。状态码只要变就点名，不看内容；新提交那一半按 HEAD 之间的差点名，不看工作区。能照喂的四形（都推的，待探针）：

| 场景 | git 操作 | 工作区内容 | 预期读数 |
|---|---|---|---|
| stage-only | 起来时 ` M .claude/kb/k.md`，别的会话 `git add` 它 | 没变 | 点名「工作区改了没提交」（状态码 ` M` → `M `） |
| chmod-only | `chmod 755 .claude/kb/k.md` | 没变 | 点名（新仓 `core.fileMode` 缺省开，状态出 ` M`） |
| reset-soft | `git reset --soft HEAD~1`，那一条提交动过 `.claude/kb/k.md` | 没变 | 点名「两样都有」，同时「没有新提交（HEAD 从 … 移到了 …，不是往前走）」 |
| ignored-inside-untracked | 起来时未跟踪的 `.claude/kb/drafts/` 里有 `notes.md` 与一份被 `*.img` 忽略的 `cache.img`，只改 `cache.img` | git 看得见的内容没变 | 点名目录 `.claude/kb/drafts/`：摘要走 `os.walk`，不看 `.gitignore`（`scripts/agents/agent-watch.py:1545`） |

真仓此刻正是这一类会常出现的样子：`git status --short` 534 行，其中已暂存 `A ` 323 行、` M` 201 行、`MM` 4 行（量的，主工作区，`cut -c1-2 | sort | uniq -c`）。几个会话用 `stage-mine.py` 暂存、取消暂存，到点正赶在中间的，只换了暂存状态的知识类文件都会被点名。

另两类噪声（不是「没变而点名」，记着）：主对话自己改的、自己提交的知识类文件照样被点名、叫它重读（判法不分是谁改的）；`--discover` 与 `--processes` 两只看门狗同时在跑时各有各的基线，同一小时打两节、条目对不上。

### 一节最多多少行

固定行：提醒句、`仓的现状（…）：`、分支与 HEAD、新提交那一行、计数那一行、别的路径那一行、末尾「→」，共 7 行；三张清单各最多 `limit` 行加一行「…余 N …没列」。合计最多 3 × limit + 10 行，`watch.conf` 的缺省 20（`scripts/agents/watch.conf:27`）下是 70 行：每张清单各自封顶之后合计封得住（成立，读码；探针 line-bound 造三张各 30 条的清单量 `lines=`，推的，待主 agent 跑）。行宽封不住：提交标题与路径原样打、不截，line-bound 用 210 字的标题量 `widest=`。没查成那一支只有三行。

## 第 7 问：拖慢与弄挂

### git 的条数与时限（成立）

- 读码：记基线三条（`log -1`、`branch --show-current`、`status --short`），到点 HEAD 没动三条、动了五条（多 `log <起来时>..HEAD` 与 `diff --name-only`）。探针 git-count 把 `run_git` 包一层数条数（推的，待主 agent 跑）。
- 时限：「GIT_TIMEOUT_SECONDS_DEFAULT = 30」（`scripts/agents/agent-watch.py:531`）每条各算；一条超时就抛 `RepositoryUnreadable`、那一趟不再起后面的，所以一次快照最坏是前几条各差一点不到 30 秒、再加一条 30 秒，到点那一趟上界约 150 秒，基线约 90 秒。超时、起不来、退非 0 都走「没查成」照旧退 4（`scripts/agents/agent-watch.py:1688` 之后那两行）。
- 别的会话提交时占着 `index.lock`：带 `GIT_OPTIONAL_LOCKS=0` 的 `git status` 不去拿锁，照常读出（探针 index-lock，推的）。
- 真仓此刻的 `git status --short`（量的，本机，有别的会话在跑）：`real 0m0.556s`，534 行；范围里的 58 个路径（`du -cb` 合计 22448149 字节）每次快照各算一遍 sha256，按 1 GB/s 估 0.02 秒（推的）。

### 打中：算摘要没有时限，命名管道能把看门狗卡死

- 摘要那一步不在 git 的时限里：目录走 `os.walk` 逐个文件读（`scripts/agents/agent-watch.py:1545`），文件「with open(path, "rb") as handle:」（`scripts/agents/agent-watch.py:1529`）阻塞读，没有超时、不判文件类型。
- 能照喂的输入（探针 fifo-in-untracked-directory，推的，待主 agent 跑）：
  ```
  mkdir -p .claude/kb/pipes && echo 草稿 > .claude/kb/pipes/notes.md && mkfifo .claude/kb/pipes/channel
  ```
  git 不列命名管道，但目录里有一份普通文件，`git status --short` 就写 `?? .claude/kb/pipes/`；落在范围里要记摘要，`os.walk` 把 `channel` 当文件交给 `open(…, "rb")`，没有写端就一直等。看门狗卡在 `take_repository_baseline`（`run()` 进循环之前，`scripts/agents/agent-watch.py:2658`）或到点那一趟：不整查、不报告警、不退 3 也不退 4，主对话不会被叫醒。探针给子进程 20 秒，预期 `hung=yes`。
- 同一处的慢形（探针 sparse-ignored-image，推的）：未跟踪的 `.claude/kb/drafts/` 里放 `notes.md` 与一份被 `*.img` 忽略的 2 GiB 稀疏镜像，基线与到点各把它读一遍；耗时按读速线性涨，读一份 100 GiB 的稀疏镜像按 2 GiB/s 估 50 秒一趟（推的）。本仓的装置常造稀疏镜像，`.gitignore` 第 6 行忽略 `*.img`；被忽略的 `target/`、`__pycache__/` 落在未跟踪的知识类目录里同样会被走一遍。

### `RepositoryUnreadable` 之外的异常（成立，推的）

逐条看过 `repository_status_section` 调到的每一步：git 输出用 `errors="replace"` 解码；`file_sha256` 接住 `OSError`；`os.walk` 不给 `onerror`，读不了的目录静默跳过；字典取值前都先判了在不在（`key in start.digest_by_path`、`path in start_status`）；计数用 `Counter`。读码没找到抛得出来的 `OSError`、解码错或 `KeyError`。唯一一处会抛的是自证专用变量 `AGENT_WATCH_GIT_TIMEOUT_SECONDS` 写成非数字时 `float()` 的 `ValueError`（`scripts/agents/agent-watch.py:1467`），平时不设。真抛了别的异常会出 `main()`，看门狗带 traceback 退 1，这一趟没有定时回报。

## 第 8 问：该提醒而不提醒（只判，不改）

### 判定：不成立——「一个小时同步一次」只在一只看门狗连着 60 分钟没有告警时才兑现

- 基线跟着一只看门狗走：每只起来时记一次（「repository_baseline = take_repository_baseline(arguments) if arguments.mode == "watch" else None」，`scripts/agents/agent-watch.py:2658`，`--processes-only` 那一路同一句在 `scripts/agents/agent-watch.py:2159`），不落盘。
- 只有到点那一支打：`--agents` 一路 `if time.time() >= report_deadline:`（`scripts/agents/agent-watch.py:2703`）之后才调 `print_repository_section`；告警那一支只在弄坏开关下打（「if BROKEN_DETECTION == "repositoryonalert":」，`scripts/agents/agent-watch.py:2692`）。检出等不到整查、读到就退 3 的那一支（`scripts/agents/agent-watch.py:2732`）同样不打。
- 技能规定的起法是 `bash scripts/agents/watch.sh` 不带 agent 号，走 `--discover`。这一路找出来的只有还没结束的子 agent（「if not ended:」，`scripts/agents/agent-watch.py:2603`），全部结束之后表是空的，而「bool(done_flags) and all(done_flags)」（`scripts/agents/agent-watch.py:2304`）在空表上是假：`--discover` 的看门狗不走「全部交回退 0」，只按告警退 3 或到点退 4。所以这一路丢提醒的只有两处：按告警退 3，与两只看门狗之间的空档。
- 连着几只都按告警退：每一只的 60 分钟从它起来那一刻重新算，前一只记下的基线与它到退出为止看到的变动一起丢掉。主对话收不到的时长没有上界：只要告警间隔一直短于 `max-minutes`（60），这一节一次都不出现；那段时间里别的会话提交、改的知识类文件，下一只起来时已经算进它的基线，之后永远不点名。
- 能照喂的一段历史（推的，按读码）：派一个子 agent，它每 40 分钟有一次工具调用超过 8 分钟（「工具调用过长」，叫醒、退 3）；主对话照技能读完告警、`--ack` 再起。四个小时里退 3 六次，退 4 零次，知识同步提醒零次；其间别的会话改了 `.claude/rules/` 里一份规则并提交，它从头到尾没被点名。
- 这一形在今天的会话里多常见：没量。要量就数一个会话的主会话记录里看门狗每次退出的退出码（3 与 4 各几次、相邻两次退 4 隔多久）；这一格没写探针，主 agent 要的话另立一件。

### 改法与代价（各修哪一格；都是推的，没实现、没跑，被攻过零轮）

| 改法 | 修的是哪一格 | 代价 |
|---|---|---|
| 告警退 3 也带这一节 | 按告警退出丢提醒 | 每次告警都多打最多 70 行，告警叫醒常是几分钟一次，变成「每次告警一次」而不是每小时一次，噪声换掉了节拍；告警那一刻多起三到五条 git，最坏把告警推迟一个 git 时限（30 秒）再加算摘要的时间（第 7 问那条没有时限的路也搬到告警上）；基线照旧随看门狗丢，空档那一格不修 |
| 基线落盘、跨看门狗接着比（放进 `--discover` 已有的会话状态目录，与 `watch-acks.tsv` 同处），再记一笔「上一次打这一节在什么时候」，任何一种退出满 60 分钟没打过就打一次、打完换基线 | 按告警退出丢提醒、两只之间的空档 | 新加一份跨进程的状态：几只看门狗（`--discover` 与 `--processes`）同时读写要加锁；起来时的 HEAD 被改写历史之后 `git log <旧>..HEAD` 会失败（走「没查成」）；落盘的是快照不是判定，不算新判定库，但要过「不新建判定库」那一条的审；状态目录要在两台都读得到的共享根下；自证要多造「上一只按告警退、下一只到点」的两段历史；一只卡死（第 7 问）或被停的看门狗留下的基线时刻过旧，第一次打会点名一大片 |
| 只把告警退 3 那一支改成「离这只起来满 60 分钟才带这一节」 | 按告警退出丢提醒的一部分（告警恰在满 60 分钟之后才来的那几次） | 代码改动最小；告警频繁时仍然一次都不打，第 8 问的主形不修 |
| 不动看门狗，改技能：读到告警之后起下一只之前先自己跑一遍 `git status --short` 与 `git log` | 都不修（靠主对话自觉） | 不是机检，与用户「只加在看门狗里」的范围对不上 |

## 第 9 问：多机与环境

### 读哪个仓（成立）

取法一行：「return arguments.repository_root or session_directories.source_repository() or REPOSITORY_ROOT」（`scripts/agents/agent-watch.py:1574`）。统一任务服务给的值是「交提交方那一份仓的根（服务所在那棵仓的绝对路径）」（`task-server/task_server_lib.py:134`），不是 `--cwd`。各处读哪个仓（推的，待探针 root-precedence 核三级的先后与相对路径那一支）：

| 在哪起 | 读哪个仓 | 对不对 |
|---|---|---|
| 主对话的 Bash 里起 `watch.sh`（技能的起法，本机） | 没有服务变量，取脚本所在的仓，即主工作区 | 对 |
| 第二台 | 看门狗不在第二台起（只在主对话的 Bash 里起）；自证经服务派到第二台时 `--repository-root` 指小仓。服务变量与脚本所在的仓在共享根上是同一绝对路径 | 对 |
| 经服务在仓副本里起（`--cwd <副本>`，跑副本里那份脚本） | 服务变量指主工作区，先于脚本所在的副本：读的是主工作区，不是副本 | 照文件头；若要的是副本就错，没有格盯这一级先后 |
| `TASK_SERVER_SOURCE_REPOSITORY` 是相对路径 | 抛 `SessionDirectoryError`，基线记成脚本所在的仓、`snapshot=None`，到点写「没查成」、手动命令指脚本所在的仓 | 对 |
| git worktree 里起，环境里继承了指到别的仓的 `GIT_DIR` | 去掉了（`scripts/agents/agent-watch.py:1468`），`git -C <工作树>` 读工作树自己的分支与索引 | 对（探针 worktree-inherited-variables 核，推的） |
| 同上，继承的是 `GIT_COMMON_DIR` | 没去掉（文件头只承诺「GIT_DIR、GIT_WORK_TREE、GIT_INDEX_FILE 不继承」，`scripts/agents/agent-watch.py:137`）：refs 与对象到别的仓里找，预期「没查成」 | 推不出，只在 git 钩子之类里起看门狗才碰得到 |

### 推不出：git 配置是本机独有状态，自证关了、真跑没关

- 真跑：`run_git` 只去掉三个变量，别的环境原样带（「environment = {name: value for name, value in os.environ.items() if name not in GIT_ENVIRONMENT_DROPPED}」，`scripts/agents/agent-watch.py:1468`），读各台自己家目录下的用户级 git 配置与系统配置。
- 自证：「SELFTEST_GIT_ENVIRONMENT = {"GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}」（`scripts/agents/agent-watch.py:3658`），「environment = dict(os.environ, **SELFTEST_GIT_ENVIRONMENT)」（`scripts/agents/agent-watch.py:3778`）交给起的看门狗子进程，`run_git` 继承下来，所以自证里这一节从不读用户配置。
- 共用约束要「不读会改变判定结局的本机独有状态」（`.claude/agent-common.md:30`）。本机用户级 git 配置此刻只有 `user`、`sendemail`、`patatt`、`b4` 四节（量的，读文件），没有一项碰得到这一节；第二台的没读到。能照喂的三份配置（探针 global-config-status-branch、global-config-untracked-hidden、global-config-color-always，推的）：
  - `status.branch = true`：`--short` 多一行 `## main`，按「第 4 个字符起是路径」读成路径 `main`、状态码 `##`，已暂存与未暂存各多数 1。
  - `status.showUntrackedFiles = no`：起来之后新建的未跟踪规则不点名。
  - `color.status = always`：状态行带颜色转义，前两位不是状态码，路径前面粘着转义串，改了的知识页不点名。
- 主对话只在本机起看门狗，所以这一条今天只在有人改本机 git 配置时才碰得到；读到它的出路是 `run_git` 也设 `GIT_CONFIG_NOSYSTEM=1` 与 `GIT_CONFIG_GLOBAL=/dev/null`，或改用 `git status --porcelain`（不受 `status.branch`、颜色配置影响，推的，没核）。

## 第 10 问：自证与开关

### 守卫盯的那几样（成立，推的）

`scripts/gate/selftest-process-guard.py` 文件头判红的是真 cargo、rustc、herd7、verus、qemu-system-*、真清场与私有清场脚本。这一格起的是 `git`、只睡的 `sh`（`exec sleep 30`，超时被 `subprocess.run` 杀掉，同一个进程号、不留孙进程）与看门狗自己，`git commit` 用 `core.hooksPath=/dev/null` 不跑钩子，小仓对象少、`gc --auto` 不该起后台（推的）。任务 20085 的开关复跑退 0 是背景材料的转述，这一条腿没读它的日志。

### 打中：两条开关点名的格逐字相同

文件头开关表里 `knowledgetagsource` 下一行（`scripts/agents/agent-watch.py:271`）与 `knowledgeworktreeblind` 下一行（`scripts/agents/agent-watch.py:273`）逐字相同（量的：两行各写进草稿目录的文件，`cmp` 退 0），都点名「工作区改了没提交」「起来时已改内容又变」两格。复跑那一格比的是点名的格与实际红的格，两条都过，只看格名分不出是哪一条坏了；分得开要读那几行「实际点名 {…}」里的标签（前者全写「新提交动到」，后者那几份根本不在表里）。其余十五条按点名的格集合两两不同（读码比的，推的）。

### 打中：几处代码没有格盯，漏点名-上游规则在自证里照样绿

| 代码 | 没有格的依据 | 弄坏了什么也不红（推的） |
|---|---|---|
| 读哪个仓的三级取法（`scripts/agents/agent-watch.py:1574`） | 自证起的每只看门狗都带 `--repository-root`（「"--repository-root", repository, *extra_arguments]」，`scripts/agents/agent-watch.py:3798`），没给的那几只走 `AGENT_WATCH_REPOSITORY_ROOT`（`scripts/agents/agent-watch.py:3999`）；`selftest_repository_report` 里搜不到 `TASK_SERVER_SOURCE_REPOSITORY` | 后两级对调 |
| 去掉 `GIT_DIR` 那三个变量、`GIT_OPTIONAL_LOCKS=0`（`scripts/agents/agent-watch.py:1468` 起两行） | 自证里没有一格设这几个变量 | 不去掉、不设 |
| 未跟踪目录的摘要（`scripts/agents/agent-watch.py:1545`） | 自证里未跟踪的样本是一份文件（`.claude/kb/already-untracked.md`），不是目录 | 目录一律记「不在」 |
| 知识类文件的改名 | `parse_status_short` 那一格只核改名行读成一个键，没有一格让知识类文件改名 | 改名行不拆两头 |
| HEAD 往回挪的那句「不是往前走」 | 没有一格让 HEAD 往回挪 | 那句写错 |
| 范围里被 `.gitignore` 忽略的条目 | 自证的小仓没有 `.gitignore`，样本「".claude/singlefs-ai-sop/rules/sample-sop-rule.md"」（`scripts/agents/agent-watch.py:3664`）在小仓里看得见 | 漏点名-上游规则那一条：真仓里永不点名，自证照样绿 |

### 时序：按就绪文件等之后仍有一段固定窗口（推的，没量）

要改仓的五只看门狗回报周期是「REPOSITORY_WATCH_MINUTES = "0.4"」（`scripts/agents/agent-watch.py:3659`），从各自 `watch_started` 起 24 秒；就绪文件在第一次整查之后建，而到点的判断在建就绪文件之前（`scripts/agents/agent-watch.py:2703` 先于 `:2714`）。所以「记基线 + 第一次整查 + 自证对五个小仓做完全部改动（七条提交，每条 add、commit、log 三条 git，共二十一条）」只要超过 24 秒，那一只在改动做完之前就到点退 4，相应的格偶发红。机器忙时（开关复跑一百条、按每台默认重活线程数并行，每趟各起十几只看门狗）整查要扫 `/proc` 下全部进程开着的文件，这一段多长没量；`--max-minutes` 写小数本身不是问题（`type=float`）。

## 第 11 问：技能那一段的出路

### 打中：「没查成」与「余下没列」给的命令照原样跑不起来

- 「→ 自己看：git -C {root} status --short；git -C {root} log -5」（`scripts/agents/agent-watch.py:1692`）与「git -C {root} diff --name-only {start.head_short_hash} HEAD；git -C {root} status --short」（`scripts/agents/agent-watch.py:1700`）两条命令中间是全角分号「；」。bash 不认它，整串成了一条：`status` 收到参数 `--short；git`，预期 git 报不认的选项、退 129（推的，待探针 manual-commands-verbatim、remainder-command-verbatim 照原样交给 `bash -c` 量退出码与标准错误第一行）。技能写「写着「没查成」的，照它给的命令自己看仓」（`.claude/skills/dispatch/SKILL.md:39`），照原样跑就跑不成，要主对话自己拆。
- 项目里管 Bash 的几道闸（检出钩子、项目代码闸、范围声明闸、定时闸）不拦 `git -C <仓> status`、`git log`：它们都是只读 git，主 agent 照跑。拦不拦不是问题，问题是命令本身。
- 两条都没带 `-c core.quotepath=off`，与看门狗自己起 git 的写法不同；kb 写着不带的后果：「中文路径默认被转义成八进制」（`.claude/kb/tooling.md:215`），本仓的记录与 kb 路径多是中文名，照着看会漏。

### 打中：「核手上正在改的文件」那一步没给命令

末尾「→」写「checking = "核手上正在改或要改的文件有没有被别的会话改过、提交过」（`scripts/agents/agent-watch.py:1720`），技能那一段照抄成「再照末尾「→」那一行核手上正在改或要改的文件」，两处都没给命令。规则写「提醒里的每一种处置都给能照跑的命令」（`.claude/rules/command-safety-by-review.md:19`）。仓里现成的是 `python3 scripts/agents/who-touched.py <路径或符号>…`，技能同一份文件第 15 条那一段已经点名它（「查谁改过某个路径或谁提到过某个符号」，`.claude/skills/dispatch/SKILL.md:20`）。

### 「最后起下一只看门狗」在没有子 agent 在跑时还该不该起：该起，但看门狗自己不说

- 该起：`--discover` 那一只是这一节唯一的载体（第 8 问），不起就没有下一次提醒；`.claude/skills/dispatch/SKILL.md:26` 本来就写「它退出了（按告警退出、定时回报、全部交回）才再起一只」。
- 对不上的地方：`--discover` 找不到在跑的子 agent 时照样到点退 4（第 8 问那一条），这时出路句不打——「if not active_ids and not unstarted_ids:」（`scripts/agents/agent-watch.py:2437`）直接返回，输出里没有「接着盯就用 run_in_background 再起一个看门狗」那一行；文件头还写「4 定时回报（没有告警、子 agent 还在跑」（`scripts/agents/agent-watch.py:23`），这一回报里没有在跑的子 agent。主对话照输出不起、照技能起，两份说法不一致（推不出）。
- 次序：技能写先逐份重读、核完手上的活，「最后起下一只看门狗」。上一只到点那一刻到下一只起来记基线之间的变动永远不会被点名，重读花得越久这段越长；先起下一只再重读，这段缩到起一只看门狗的工夫（推的）。

## 打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的是哪一问的哪一个分句 | 背景材料给的改法在打中的格上还中不中 |
|---|---|---|---|---|
| 漏点名-上游规则 | 这一轮没有臂；它打的是「范围八项」这一条现状句本身 | 看不到：git 两处来源都不列被忽略的路径，看门狗做决定时手上没有这几份文件的任何信息（判别子观测不到） | 第 5 问「子模块或被忽略的文件」；也碰第 10 问 | 第 8 问的两个改法都不修它 |
| 漏点名-带空格的路径 | 同上 | 看得到：原始行里有引号，只是没去引号 | 第 5 问「路径带空格或引号」 | 不修 |
| 内容一字没变也点名 | 同上 | 看得到：内容摘要就在手上，判法却先看状态码 | 第 6 问「造得出没变而被点名的吗」（别的会话暂存那一形） | 不修 |
| 命名管道卡死 | 同上 | 看得到：`os.walk` 给的条目能判文件类型，没判 | 第 7 问「读的时候……」与「结局是没查成还是看门狗退别的码」：结局是不退 | 「告警退 3 也带这一节」会把同一条卡死的路搬到告警上；基线落盘那一条不碰它 |
| 告警退出丢提醒 | 同上 | 看得到：到告警那一刻基线还在内存里 | 第 8 问「连着几只看门狗都按告警退出时」 | 「告警也带这一节」修按告警丢、不修空档；「基线落盘」两格都修；「满 60 分钟才带」只修一部分 |
| 两条开关点名的格相同 | 同上 | 复跑那一格看得到失败行原文，只比了格名 | 第 10 问「有没有两条弄坏同一处、或点名的格其实靠别的开关也会红」 | 不涉及 |
| 几处代码没有格 | 同上 | — | 第 10 问 | 不涉及 |
| 手动命令跑不起来 | 同上 | 看得到：命令是它自己拼的 | 第 11 问「给的命令过不过项目里管 Bash 的那几道闸」：过得了闸，过不了 bash | 不涉及 |
| 核那一步没命令 | 同上 | — | 第 11 问（同上一句的射程，推广到「→」那一行） | 不涉及 |

## 自己提的改法（只在读码上推的，没在任何模型上量过，被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 范围表删掉 `.claude/singlefs-ai-sop/rules/`，或这一条改成按内容摘要直接比目录（不经 git），另把仓根 `.singlefs-ai-sop-version` 算进知识类文件 | 漏点名-上游规则 | 推的，待主 agent 跑（探针 ignored-parent 是这一格的对照） |
| `git status` 换成 `--porcelain -z`（NUL 分隔、不加引号），`diff --name-only` 加 `-z` | 漏点名-带空格的路径；连带去掉第 9 问 `status.branch`、颜色那两形（推的） | 推的，待主 agent 跑 |
| 状态码变了而摘要没变的不点名（只在摘要也变了，或摘要记不了时才看状态码） | 内容一字没变也点名的只暂存、执行位两形；`reset --soft` 那一形还要新提交那一半也看工作区摘要 | 推的，待主 agent 跑 |
| 算摘要只读普通文件（`stat.S_ISREG`）、跳过被 `.gitignore` 忽略的条目（目录摘要改成 `git ls-files -o --exclude-standard -- <目录>` 列出来的文件），并给整次摘要设时限，超时走「没查成」 | 命名管道卡死、被忽略的大镜像拖慢 | 推的，待主 agent 跑 |
| `run_git` 也设 `GIT_CONFIG_NOSYSTEM=1`、`GIT_CONFIG_GLOBAL=/dev/null` | 第 9 问的配置三形 | 推的，待主 agent 跑 |
| 两条命令拆成两行各一条，写成 `git -c core.quotepath=off -C <仓> …`；「核」那一步给 `python3 scripts/agents/who-touched.py <路径>…` | 第 11 问的两条 | 推的 |
| 开关 `knowledgetagsource` 或 `knowledgeworktreeblind` 改点名一格只有它会红的（例：给「两样都有」「已不在」各开一格） | 两条开关点名的格相同 | 推的 |

## 没打中的形状

- 第 5 问：改名两向、起来时改着之后提交又改、`stash` 与 `stash pop`、换分支再换回、新提交删文件、未跟踪目录整行与目录里又多一份、本仓的子模块（不可达）——表在第 5 问末尾，取样是每形一个小仓、一段固定的 git 序列；没放开扫的用户动作：改名之后再改内容、`stash` 之后再提交、几只看门狗交错起。
- 第 6 问：只改 mtime（git 按内容判，不点名，推的）、行尾转换（本仓没有 `.gitattributes`，`ls` 现查，不可达）；一节的行数（封得住）。
- 第 7 问：索引锁、git 超时与起不来、`RepositoryUnreadable` 之外的异常（读码没找到）、指到仓外的链接（`os.walk` 不跟链接，不会走进环）。
- 第 9 问：三级取法、相对路径、worktree 里继承 `GIT_DIR`。
- 第 10 问：守卫盯的那几样。

## 这条腿自己的限度

- 探针一个都没跑：子 agent 不交任务，凡标「推的，待主 agent 跑」的读数都是按读码写下的预期，主 agent 跑出来对不上的，以跑出来的为准、那一条按没打中或另判。
- 命名管道那一格预期卡死，探针给子进程 20 秒、之后停它那一组；20 秒是拍的，读数只说明「20 秒内没记完」。
- 第 8 问的告警频率没量；「连着按告警退」那段历史是造的，不是哪个会话里出现过的。
- 第 9 问只读了本机的用户级 git 配置，第二台的没读；第 10 问的时序窗口没在忙的机器上量过。
- `git diff --name-only` 给双引号加引号、不给空格加引号，这两句没核手册，是按 git 的引号规则推的；探针 quote-committed 与 space-committed 是它们的对照。
- 只读了「范围」里的六份文件与它们直接调到的库（`lib_session_directory.py`、`task-server/task_server_lib.py` 文件头一段、`scripts/gate/selftest-process-guard.py` 文件头）；正推腿的报告、模型与草稿照禁读清单没读。

## 要主 agent 跑的命令

1. 探针整趟（新 shell 里起；交之前先确认 `/home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-opus/probe-run` 不在或可删，包装脚本只删这一个以 `-run` 收尾的目录）：

task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-opus/probe-run /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r1-opus-model

   - 在哪起：`--cwd` 是这条腿的草稿目录 `/home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-opus`；第二个参数（冻结副本）填的是仓根，换成开工快照那棵树也行。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 算跑完（退非 0 的件数为 0 算过；`exit=3` 是完整性闸红、整趟作废）；`task-server/task log <号> --all` 里 `PROBE name=watchdog` 那一行的 `sha256=` 要等于 `1c46829bc927f97c4cb1a2dd0fc488bfbdd2a1c7d192325f9173f237f898719a`；`PROBE name=done emitted=33 setup_failures=0` 与 `name=rerun_done probe_exit=0 probe_lines=33`。
   - 逐行对的预期（每行 `named=`、`other=`、`reminder=` 与这份报告对应那一问写的预期比）：ignored-parent `named_count=0 other_count=0`；space-untracked-new `named_count=0`、`other` 里有带引号的 `".claude/kb/a b.md"`；space-tracked-rewritten `named_count=0 other_count=0`；quote-committed `named_count=0`；space-committed `named_count=1`；stage-only、chmod-only、reset-soft、ignored-inside-untracked 各 `named_count=1`；line-bound `lines=70 bound=70`；git-count `baseline_git=3 tick_git_head_still=3 tick_git_head_moved=5`；index-lock `reminder=changed-1`；fifo-in-untracked-directory `hung=yes group_left=no`；global-config-status-branch 计数比实际多 1、global-config-untracked-hidden 与 global-config-color-always `named_count=0`；manual-commands-verbatim 与 remainder-command-verbatim `exit=129`。对不上的那一格，这份报告里对应的打中按读数改判。
   - 跑完：读完日志删运行目录 `/home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-opus/probe-run`（里面是几十个带 `.git` 的小仓，完工清理闸会把它们当仓副本列）。

2. 没写 `tools/rerun.sh --selftest`：模板自证拿假仓起换过的 `rerun_steps`，假冻结副本里没有 `scripts/agents/agent-watch.py`，那一格预期红在「拷进与仓无关的临时目录、传参应当跑通」，不是这份模型的毛病。

## 没做什么

- 不判第 1 到 4 问（正推腿那几格）；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数都没复跑（命令在上一节）。
- 没在副本上改代码试改法，「自己提的改法」一张表全是推的。
- 第 8 问照派发只判与写代价，不改文件。
- 草稿目录里只有几份文本（真仓 `git status` 的一份只读输出、范围内路径清单、开关表两行、换 `rerun_steps` 用的文本），没有仓副本、编译目录与库拷贝，没有要删的。
