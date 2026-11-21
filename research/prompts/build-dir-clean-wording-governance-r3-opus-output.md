# build-dir-clean-wording-governance-r3 攻方腿报告（2026-11-21）

被判对象：正文 `research/prompts/_build-dir-clean-wording-governance-r3-body.md`「范围」的 33 份（`research/prompts/build-dir-clean-wording-governance-r3-snapshot/judged-sha256.txt`，开工时 `sha256sum -c` 逐份对上，读的都是主工作区此刻那一版）。攻击面：正文「要腿判的」第 1 到 4 问。前两轮攻过的角度（两份定义 `bash -c` 那一形与 `mkdir -p` 过不过闸、输出目录对照、滞后表与搬迁表删行、派发闸那一处 `history-kept` 的类别、改法 4 选修判法）不重复。

判定用词：成立＝修补站得住、没打中；不成立＝打中、修补说的或做的不对；推不出＝依据推不出结论（算打中）。

这条腿没写带 `rerun.sh` 的模型（没有崩溃枚举、没有要编的原型），只写了两份探针，都在草稿目录，复跑命令在末尾「要主 agent 跑的命令」：

| 探针 | 文件 | sha256 |
|---|---|---|
| 完工清理闸列表条件变异（仓副本里只改一行） | `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-opus/copy/.claude/hooks/completion_cleanup_guard_decisions.py` | `3f6885f07b3f3c18c1b8413a0d543b5366e856a2bd7418a7de43328f2a697b2d`（主工作区原件 `de391d2e7007c71e5f25c0f5b95727e7ee2d534f7fbadc13ef50b6e8ec016eab`） |
| 注释历史判法探针根 | `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-opus/comment-probe-root/scripts/probe.sh` | `f57200d4f3ad1209baca72ddae167299482870796f4e0da57ce9c65b832c8cd7` |

## 各格判定一览

| 问 | 判定 | 一句话 |
|---|---|---|
| 1 完工清理闸接任务表 | 推不出（打中，两处） | 会话认法对（分片子任务、人手 `--manual` 都对上，换会话号核不动）；四条新开关红在名字说的那件事。打中：（甲）「两样都占写 clean」与「没取走的也可以 clean」照字面只删现场、不取结局，`task clean` 不记已取，之后每次交任务都打「严厉警告」；（乙）自证没有「设了留现场、结局已取走」的样本，列表条件丢掉「设了留现场」的变异 116 格照绿（推的，待主 agent 跑探针） |
| 2 注释历史判法带冒号才放过 | 成立（另报一处存量打中） | 判法做到它说的；带冒号照样能藏来历，文件头「管不到的」已写明；仓里现有一处真实的：检出钩子文件头三处引的格名里带「正推腿乙 Z8」，判法放过。全角冒号只认 ASCII，与文件头一致、判得对 |
| 3 服务清扫旧命名目录 | 成立 | 别的库标签、第二台、在排要接着用的、组里登了而树换路径的，都删不到；回读、告警、次序没找到会误删的缺口 |
| 4 改过的说明与出路 | 不成立（打中） | `task-server/README` 三语第 224 行仍写「取消都不删组里的编译目录」，正是第二轮第 4 条坐实的那句假话，改法 4 漏了这一行；同一份 `task_server_lib.py` 新登的三条开关写「没复跑」，与正文「开关复跑四条逐格相同」对不上；`experiment-runner.md` 新加的「`--cwd` 不是仓根时命令位置上的脚本写绝对路径」与同一份第 41 行的写法相反 |

## 第 1 问：完工清理闸接任务表那一判

### 会话认得对不对（成立；换会话号核不动）

- 取法：「JOIN tasks ON attempts.task_id = tasks.id」（`.claude/hooks/completion_cleanup_guard_decisions.py:1305`），任务按 `tasks.session = ?` 取，`?` 是钩子输入的 `session_id`（`.claude/hooks/completion_cleanup_guard_decisions.py:1618`）。
- 任务表的会话由服务照进程树取：「各祖先 /proc 环境里的 CLAUDE_CODE_SESSION_ID 取最外层的那一个」（`task-server/task_server_lib_commands.py:543`）。这一台上核过：本 Bash 进程环境里有这个变量，往上的 `claude` 进程与更上层都没有（`/proc/<pid>/environ` 逐层读），所以最外层就是 Claude 给 Bash 的那一份，与钩子的 `session_id` 同源。
- 分片子任务：建子任务时「"session": parent.get("session", "")」（`task-server/task_server_scheduler_building.py:490`），子任务与父任务同会话，接任务表那一判取得到子任务的尝试；闸把子任务挂到父任务号、按父任务判结局与留现场，与 `scene_due` 先换成父任务（`task-server/task_server_scheduler_building.py:351-352`）同一口径。成立。
- 人手 `--manual`：会话列是空串，闸不列；`scene_due` 对没有会话的按结局时刻删（`task-server/task_server_scheduler_building.py:358`），不用谁取。会话里带 `--manual` 交的，服务当场拒（`task-server/task_server_scheduler_building.py:1225`）。成立。
- 换过会话号：`/clear`、`--resume` 之后钩子输入的 `session_id` 与之后起的 Bash 环境里的变量是否同时换，这一轮查不到（Claude Code 的行为，仓里没有记录，本会话不能做这个动作），核不动。两头不一致时的后果是推的：换号之前交的任务闸不列；而 `task clean` 只许同一会话（`task-server/task_server_scheduler_starting.py:728`），列了也删不动，所以不列不算漏判。

### 四条新开关红在名字说的那件事（成立）

逐条按代码读（`.claude/hooks/completion_cleanup_guard_decisions.py:1302-1306`、`:1341`、`:1411`，自证 `.claude/hooks/completion_cleanup_guard_selftest.py:424-445`）：

- `kept-scenes-uncollected-unchecked`：`uncollected` 恒假，样本 16（没设留现场、没取走）不列，`uncollected-scene-left` 红。对。
- `uncollected-scene-removal-by-clean`：`by_wait` 恒假，样本 16 写成 `clean 16`，`uncollected-scene-left` 断言要 `wait 16`、不许 `clean 16`，红。对。
- `kept-scene-removal-uncollected-first`：两样都占的样本 7 写成 `wait 7`，`main-kept-scene-left` 断言要 `clean 7`、不许 `wait 7`，红。对。
- `kept-scene-attempts-by-session-column`：样本库尝试行会话列照服务写空串，按尝试表会话列取不到，两格都红。对。

### 打中（甲）：「两样都占写 clean」「没取走的也可以 clean」照字面只删现场、不取结局

被判对象里许可这一步的句子：

- 「两样都占的照这一样写」（`.claude/hooks/completion-cleanup-guard.sh:38`），即写 `task clean <号>`；
- 「不再要读现场的也可以 task clean <号>」（`.claude/hooks/completion-cleanup-guard.sh:37`），拦下的单子里同一句是「（不再要读现场的直接 `{TASK_SERVER_CLIENT} clean <号>`）」（`.claude/hooks/completion_cleanup_guard_decisions.py:1527`）；
- 单子上的删法行只有一条命令：「by_wait = uncollected and (not kept or broken("kept-scene-removal-uncollected-first")) and not broken("uncollected-scene-removal-by-clean")」（`.claude/hooks/completion_cleanup_guard_decisions.py:1411`）为假时只打 `clean <号>`。

服务那一侧：`task clean` 只判会话与有没有结局、删现场（`task-server/task_server_scheduler_starting.py:716` 起的 `clean`，整个函数里没有调 `mark_collected`）；记已取只在两处：`wait` 里「if valid and session == task.get("session"):」（`task-server/task_server.py:1268`）之后，与结局推送送到之后（`task-server/task_server_scheduler_notifying.py:110`）。结局没取走的父任务，服务在之后每次交任务时照它拼警告：「结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）还没交给它（collected 为假：服务发的结局消息没送到，也没起 wait 取），记进新任务的 unclaimed_at_submit，」（`task-server/task_server.py:418`），并每隔一段重推「结局没人取」（`task-server/task_server_scheduler_notifying.py:118`）。

可达的历史（推的，没跑；每一步指到上面许可它的那一句）：

1. 主 agent 交 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 task-server/task submit -- <会判红的命令>`，任务 N 有了结局；结局推送没送到（收件地址那一刻读不到），主 agent 也没起 `wait`。
2. 主 agent 收工，完工清理闸列 N 的现场，删法行只写 `task-server/task clean N`（两样都占，`:38`、`:1411`）。
3. 主 agent 照单子跑 `task clean N`：现场删了、回读不在，闸下一回不再列（`os.path.isdir` 为假，`:1345`），收工放行。
4. N 的 `collected` 仍为假。30 分钟后主 agent 再交任何一条，回话头一行是「头一行「✗ 严厉警告：会话 <号> 有 <N> 条任务跑完了没人取（<号>、…）：…」」（`task-server/task_server.py:419`）那一形的警告（转述：点名 N），服务照旧重推「结局没人取」，直到有人 `task wait N`。没设留现场、只是没取走的任务，主 agent 照 `:37` 选 `clean` 的，同样落到第 4 步。

用户动作放开扫：拦下之后主 agent 可以只跑 `clean`、只跑 `wait`、先 `wait` 再 `clean`、先 `clean` 再 `wait`。只跑 `wait` 而设了留现场的，现场不删、闸照拦（设计如此，单子写的是 clean）；另两种两条都跑的，现场删了、结局也取了，不中。中的只有「照单子上那一条字面跑」这一支，而这一支正是单子给的写法。

四句：

- 分不分辨臂：分辨。同一历史上「单子写 `wait N` 再 `clean N`」那一臂第 4 步不出警告。
- 闸当时看不看得到判别它的东西：看得到。闸手里已有 `(kept, uncollected)` 两个布尔（`:1346` 存进单子），两样都占的就是 `kept and uncollected`。
- 满足的是判据哪一个分句：正文第 1 问「两样都占写 clean 对不对」；也落在第 4 问「照字面做走不走得通」：闸这一关走得通，服务那一关留下一个没人取的结局。
- 跑前条款给的改法在这一格上中不中：第二轮判决「三、改法」第 1 条写「出路写取走结局或 `task clean <号>`」，「或」正是这一格，照它改的仍中。

建议（只在代码阅读上推过、被攻过零轮，没实现没跑）：`uncollected` 为真的，删法行先写 `task-server/task wait <号>`；两样都占的再加一行 `task-server/task clean <号>`；删掉「不再要读现场的也可以 task clean」这一支，或在它后面补「clean 不记取走，结局照样要 wait 取」。严重度：低。结局没取走只在推送没送到又没人 `wait` 时出现，而这正是闸这一支要管的那一类。

### 打中（乙）：自证没有「设了留现场、结局已取走」的样本，丢掉「设了留现场」那一判的变异活着

列不列的那一判：「if not (kept or uncollected) and not broken("kept-scenes-keep-unchecked"):」（`.claude/hooks/completion_cleanup_guard_decisions.py:1342`）。

自证里全部任务现场样本（`grep -n 'service_task(' .claude/hooks/completion_cleanup_guard_selftest.py`，九处，逐个对过参数）：样本函数缺省是「def service_task(self, task_id, state, session=None, parent_id=None, keep=True, scene=True, collected=False):」（`.claude/hooks/completion_cleanup_guard_selftest.py:152`）；已有结局的几个是：

| 样本 | 设了留现场 | 结局取走了 | 断言 |
|---|---|---|---|
| 7、8（分片子任务）、9（盘上没有）`:424-426` | 是 | 否 | 列、写 `clean 7` |
| 10 `:434` | 否 | 是 | 不列 |
| 16 `:440` | 否 | 否 | 列、写 `wait 16` |
| 13（别的会话）、14（子 agent 交回） | 是 | 否 | 不列 |

没有「设了留现场、结局已取走」那一格：`keep=True` 的样本 `collected` 全是缺省的 `False`，「设了留现场」那一判在每个列出的样本上都被「结局没取走」那一判盖住。把 `:1342` 改成 `if not uncollected and not broken("kept-scenes-keep-unchecked"):`（只看结局取没取走），推的读数：样本 7、8 仍列（没取走），删法行照 `:1411` 读的是另一份 `kept`、仍写 `clean 7`；10 仍不列；16 仍列、写 `wait 16`；别的格不碰任务现场。86 条开关里走这一行的有两条：`kept-scenes-keep-unchecked` 让 10 被列出，照样红在 `unkept-scene-not-listed`；`kept-scenes-uncollected-unchecked` 让 `uncollected` 恒假，变异后 7、8、16 都不列，点名的 `uncollected-scene-left` 照红，另多红一格 `main-kept-scene-left`。自证自己的开关轮只核点名的格红了没有（`.claude/hooks/completion_cleanup_guard_selftest.py:1394`），多红的不算错，所以变异后自证预计仍报「判了 116 格；86 个弄坏开关各红在点名的格」（推的，待主 agent 跑，命令在末尾）；能看出变异的只有逐格比对多红了哪一格的那种复跑。

这一格在真库上最常见：设了留现场的任务，主 agent 照规矩后台 `task wait` 取走结局之后（`task-server/task_server.py:1268`），就是「设了留现场、结局已取走」，服务只由 `task clean` 删（`task-server/task_server_scheduler_building.py:360`）。这正是第一轮把任务现场接进这道闸时要管的那一种；现在代码判得对，但没有一格钉住它。

四句：

- 分不分辨臂：分辨。原件与变异在「设了留现场、结局已取走」的输入上一个列、一个不列。
- 系统看不看得到：看得到，库里两个字段都在。
- 满足的是哪一个分句：第 1 问「四条新开关是不是红在名字说的那件事」的延伸：开关红得对，但「设了留现场」那一判没有自己的样本与开关。
- 改法碰不碰得到：第二轮判决「三、改法」第 1 条只写「`unkept-scene-not-listed` 那一格的样本与断言照两个条件改」，没写补「设了留现场、取走了」一格，照它改完的就是今天这一版，碰不到。

建议（被攻过零轮，没实现没跑）：加样本 `service_task(<号>, "finished", collected=True)`、断言列、写 `clean <号>`、不写 `wait <号>`；配一条开关（例 `kept-scenes-kept-unchecked`：不看留现场，设了留现场、结局已取走的不列），红在这一格。

## 第 2 问：注释历史判法「带冒号才放过」

判法：「if CELL_NAME_SEPARATOR not in piece[start:end] and "quoted-role-names-colon-unchecked" not in switches:」（`scripts/gate/comment-history-check.py:513`），分隔符是「CELL_NAME_SEPARATOR = ":"」（`scripts/gate/comment-history-check.py:250`）。一对「」括着的整段里有一个 ASCII 冒号，这一段里的三方腿角色名就换成占位符；判的只是角色名，括着的别的词（第N轮、日期、「此前」这类）照判。

### 带了冒号也能藏来历（能；文件头已写明管不到，修补本身成立）

- 正文举的「「第二轮:攻方腿」打中的」藏不住：角色名换掉了，「第二轮」照 `第[一二三四五六七八九十百\d]+轮` 判中。
- 藏得住的形状：来历只靠角色名说、「」里随便放一个冒号。例：`# 这一段是「三方:攻方腿」打中的那一处的修补`；或套一层，外层带冒号：`# 「这一段是「攻方腿」打中的修补，见 a:b」`（文件头「冒号算在哪一层就看那一层括着的整段」，外层有冒号、里面的角色名一并放过）。词表里没有「打中」，「」外剩下的字一个都不中。
- 这一形文件头自己写进了「管不到的」：「带冒号的「」里写的其实是来历而用的词正是三方腿角色名的」（`scripts/gate/comment-history-check.py:181`），连「x:攻方腿」与「正推腿乙 Z8」两个例子一起写了，交 review。判法说的与做的一致，修补成立。
- 判别子观测不到：判法只看注释文字，一个真格名与一个随手加了冒号的来历句在字面上分不出。要判，得另拿射程里测试代码的格名标签去比（引的串在不在某份自证的标签里），那是新判法，不是这一处修补的缺口。

### 另报一处存量打中：仓里现有一处格名本身带来历，判法放过

`.claude/hooks/bash-command-detector.sh:1133`（同一句另在 `:582`、`:726`）是弄坏开关说明里引的格名：

「rsync-inplace-unseen  「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（正推腿乙 Z8；rsync-inplace-unseen 下红）」」（`.claude/hooks/bash-command-detector.sh:1133`）

格名标签出自 `.claude/hooks/bash_command_detector_selftest_waits_and_scope.py:1901`。「正推腿乙 Z8」是哪一条腿哪一项打中的，正是共用约束「不写日期、谁定的与哪一轮打中的」（`.claude/agent-common.md:47`）那一类；那一条的射程含 `.claude/hooks/` 下的「弄坏开关说明」。判法照冒号放过它（`:1133` 那一对「」里有「就地改脚本:」的冒号），文件头说这一类「靠 review」，而 review 至今没改它。第二轮判决「二、判决」第 3 条说「派发闸与检出钩子现有被放过的格名都是「带冒号的格名」那一形」，没分出这一个格名本身带来历。

四句：不分辨这一轮的哪条改法（三处引文照格名原样抄，病根在格名标签）；判法看不到判别它的东西（只读注释、不知道标签的来历）；落在共用约束第 47 行「哪一轮打中的」那一分句；第二轮判决第 3 条的改法在这一格上照样放过。出路（推的，零轮）：格名标签去掉「（正推腿乙 Z8；…）」那一段，三处引文同改（`grep -rlF` 在 `.claude/gate.d/` 下找这几个字，零命中）。`bash-command-detector.sh` 不在这一轮 33 份里，交主 agent 定归哪一题。

### 全角冒号（判得对）

只认 ASCII，与文件头「那一对「」里又有 ASCII 冒号（CELL_NAME_SEPARATOR，格名「<族>:<格>」的分隔）时不算」（`scripts/gate/comment-history-check.py:45`）一致：本项目格名的族与格之间用 ASCII 冒号，失败行「✗ 自检：」「✗ 自证 <格>：」里的全角冒号是句读，不当格名分隔。若把全角冒号也算上，`「✗ 自检：<随便什么>」` 一形的引文都会放过角色名，宽得多。

副作用（推的）：照原文引的失败行、而标签本身没有 ASCII 冒号又带角色名的（例：自证标签「攻方腿引文对上」，`.claude/hooks/handback-guard.sh:1538`），引成 `「✗ 自检：攻方腿引文对上」` 会被判中。射程里此刻没有这样的引文（`grep -rnE` 在七个射程目录的 `.py`、`.sh`、`.rs`、`README.md` 里找「」括着角色名的，只命中 `comment-history-check.py`、`bash-command-detector.sh`、`runner-dispatch-guard.sh` 三份，后两份的都带 ASCII 冒号）。出路是改标签，判法不用改。

探针（推的读数，待主 agent 跑）：`/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-opus/comment-probe-root/scripts/probe.sh` 七行注释，预计 `--list` 只列第 2、4、6、7 行：第 1 行（「三方:攻方腿」）、第 3 行（外层带冒号）、第 5 行（检出钩子那一句原样）放过；第 2 行没冒号、第 4 行与第 7 行只有全角冒号照判，第 6 行中「第二轮」。

## 第 3 问：服务清扫旧命名目录（成立）

被判的一支：`task-server/task_server_scheduler_building.py:195` 起的 `sweep_unpooled_attempt_label_directories`，由组那一扫收尾调「self.sweep_unpooled_attempt_label_directories()」（`task-server/task_server_scheduler_building.py:171`）。删的条件四样都要：名字整串是「label_name = re.compile(rf"task-{re.escape(self.store.tag)}-\d+-\d+")」（`task-server/task_server_scheduler_building.py:208`）、不是符号链接、是目录、真路径不在「组里每一行（`select("build_slots")` 不带条件、核不过的行也算，`:211`）与在起、在跑尝试的 `build_directory`（`:212`）」的真路径里；删法照本机那一支，有进程开着里面的文件就不删、回读不在才算（`task-server/task_server_machines.py:361-363`）。

逐个试正文点名的四种：

| 目录 | 会不会被删 | 依据 |
|---|---|---|
| 别的库标签的（自证临时服务、另一份库） | 不会 | 名字里要是这一个库的标签（`:208`）；库标签是建库时随机的 8 位十六进制（`task-server/task_server_store.py:91`） |
| 第二台的 | 不会 | 只列本机编译根（`:203` 取 `self.configuration.build_directory`）；第二台的编译根照字面落在共享根下就拒（`scripts/hosts/multi-host-run.sh:61`），两台的目录不会落到同一个路径。本机编译根 `~/.local/share/singlefs/task-server-builds` 在 `.claude/hooks/lib_work_root.py` 登成「按台」（`PER_HOST_DATA_ENTRIES`），第二台的家目录是另一个 |
| 在排、还没起的尝试要接着用的 | 不会 | 起尝试拿编译目录只从组里的行拿、或照 `build_directory_name` 新起 `tree-…`（`:79-101`，`task-server/task_server_machines.py:356`）；照标签起名、又不在组里的目录，没有哪一条路会再拿到它。撞上限重排接着编的拿的是上一次尝试记的组里那一行（`:61-68`），那一行在组里、受保护 |
| 组里登了、树刚换过路径的 | 不归这一支删 | 那一行仍在组里，保护到组那一扫把它当「树不在了」删掉、回读不在、删了那一行之后（`:153-171`）；组那一扫删不成的那一行留着，照旧受保护。树换路径就是另一棵树，整组跟着删是「编译目录」一段的设计（`task-server/task_server_lib.py:283`），不是这一支的误删 |

另试过、没打中的：

- 主工作区现状：`ls ~/.local/share/singlefs/task-server-builds` 此刻有 `task-` 起头的 11 个，标签都是同一个（`task-b0f6b921-*`）；正文记重启后那一扫「删了 1 个、没删成 0 个」。没删成为 0、又剩 11 个，按代码只能是这 11 个都在组里或在跑尝试里（不在组里又没进程开着的会被删），与正文「其余 9 个属哪棵树」那一句不矛盾。服务库这一轮没读（读库归主 agent），推的。
- 组那一扫抛错：同一个 `try` 里，抛错就跳过这一回的旧命名那一扫（`task-server/task_server_scheduler_building.py:172`），往不删的一边错，不会误删。
- 组里某一行的 `directory` 不是字符串（改过库）：`os.path.realpath` 抛错，这一回整扫跳过，同样往不删的一边错。
- 在起、在跑尝试的保护那一半没有自证格钉（`task-server/task_server_selftest_lifecycle.py:864` 那一格只摆了组里记着的一行）。本机尝试拿的编译目录在它跑着时一定在组里（占着的行不删），所以这一半今天是冗余的保护；删掉它不会误删，不算打中。
- 告警：没删成的每一扫写一条 `build-directory-left`，`alert` 不去重（`task-server/task_server_scheduler_lifecycle.py:36-42`），有进程长期开着的一个目录每 10 分钟一条。与组那一扫同一做法，不是误删，记一笔。

## 第 4 问：改法 2、4、5 改过的说明与出路（不成立）

对照的两段：`task-server/task_server_lib.py` 文件头「编译目录」（第 262 行起）与「现场」（第 297 行起），逐句读过；改前原样取 `~/.local/share/singlefs/work/build-dir-governance-r2-fix-c/base/` 与 `build-dir-governance-r2-fix-d/base/`，逐份 `diff` 过。

### 打中（丙）：`task-server/README` 三语第 224 行仍写「取消都不删组里的编译目录」

- 「任务记下结局、`task clean`、取消都不删组里的编译目录。」（`task-server/README.md:224`）
- 「Recording an outcome, `task clean` and a cancel do not remove the pool's build directories.」（`task-server/README.en.md:224`）
- 「結末の記録、`task clean`、取り消しでは組のビルドディレクトリを消さない。」（`task-server/README.ja.md:224`）

对不上的那一句：「派到第二台、中途失败或被停的那一趟（没等到退出码），multi-host-run.sh 的 ⑥ 连它用的组里那一个一起删」（`task-server/task_server_lib.py:292`），第二台那一侧是「没等到退出码的（中途失败、被信号停）照 ⑥ 删。」（`scripts/hosts/multi-host-run.sh:12`）。第 224 行写的「组里的」不分台，对派到第二台、中途被取消的那一趟是假话，正是第二轮判决「二、判决」第 4 条坐实的那一句；改法 4 改了同一份 README 的第 163 行（cancel 那一行）与 `prune-server/README.md:27`，第 224 行这一句没改。这一轮改法 6 在同一行后半补了旧命名那一扫，句子就在被改的那一行里。

可达的历史：主 agent 交一条会编的任务，服务把它派到第二台、接着用组里那一个（`--reuse-build-directory`）；跑到一半主 agent `task-server/task cancel <号>`；⑥ 没等到退出码，连这一趟的本地目录（就是组里那一个）一起删，收尾时组里删掉那一行（`task-server/task_server_scheduler_building.py:114-120` 的 `settle_build_slot`，`kept` 为假删行）。读 README 第 224 行的人以为组里那一个还在、下一条接着编，实际第二台那一组少了一个、下一条从零编。

四句：分辨（改成「本机组里的」那一臂在这段历史上说的是真话）；读的人看不到判别它的东西（README 是给人读的，读者照字面信）；落在正文第 4 问「有没有一处改出了新的假话」与第二轮判决第 4 条同一分句；第二轮判决「三、改法」第 4 条点名的是「这几处」，没点 README 第 224 行，照它改的碰不到这一格。出路：照第 163 行与 `prune-server/README.md:27` 的写法改成「本机组里的不删；派到第二台、中途失败或被停的那一趟连它一起删」，三语同改。

同一行另有一处现状句写成了旧做法：「每次尝试一个目录那种旧命名」（`task-server/README.md:224`），英文「the old one-directory-per-attempt naming」、日文「試行ごとに一つの旧い名付け」。`task-server/` 下的 `README.md` 在共用约束「代码与脚本的注释、说明只写现状，不写历史」的射程里，那一条不许写「旧值与旧做法」；词表里没有「旧命名」，判法判不到。出路：只写这种名字是什么（`task-<库标签>-<号>-<次>`，组里新开的一律是 `tree-…`），不写「旧」。

### 打中（丁）：新登的三条开关写「没复跑」

「「✗ task_server_lib.py 自证：unpooled-label-directories-removed：」（新开关，推的，没复跑）」（`task-server/task_server_lib.py:670`，同一句另在 `:672`、`:674`）。正文「实现今天的样子」改法 6 写「开关复跑四条逐格相同」，而调度记录第 549 行只写「task_server_lib.py 自证 4 格都对」、没写开关复跑。两者至少一个不对：复跑过的，这三行是假话；没复跑的，正文那一句说过头。另外「新开关」只在写下那一天为真，按共用约束第 47 行的判据（「放到这段代码第一次写下的那天去读还是真话」）算不得现状，而它照抄的是同一段 `:664`、`:666`、`:668`、`:676` 已有的写法。严重度低。出路：主 agent 照复跑结局把这三行改成只写点名的格，或在正文里改「开关复跑」那一句。

### 打中（戊）：`experiment-runner.md` 新加的规矩与同一份第 41 行相反

新加的：「`--cwd` 不是仓根时，命令位置上的脚本写绝对路径」（`.claude/agents/experiment-runner.md:29`）。同一份第 41 行照旧写「bash ../mutation/mutate.sh <bin 名> <源文件> ../mutation/tables/<变异表>」（`.claude/agents/experiment-runner.md:41`），`--cwd scripts/experiment-apparatus`、脚本用相对 `--cwd` 的路径。第 41 行那一形照 `--cwd` 解是对的、跑得起；新句把「相对仓根的路径在 `--cwd` 换了之后找不到」说成了「一律写绝对路径」，比病根宽，于是与第 41 行（以及不在这一轮 33 份里的 `.claude/agents/mutation-triage.md:29`）冲突。执行员照第 29 行改第 41 行不出错，照第 41 行写又违了第 29 行，两句不能同时照做。四句：不涉及臂；读的人只看得到两句字面；落在第 4 问「照字面做走不走得通」；第二轮判决改法 2 只说「`experiment-runner.md` 同一处照改」，没碰第 41 行。出路：第 29 行那一句改成「命令里的相对路径照 `--cwd` 解：`--cwd` 不是仓根时，命令位置上的脚本写绝对路径或相对 `--cwd` 的路径，不写相对仓根的」。严重度低。

### 只读出、不算这一轮改出来的

- 「（上面「编译目录」的取消的不留产物）」（`task-server/task_server_lib.py:304`）指到「编译目录」一段，那一段里没有「取消的不留产物」这几个字（`grep -nF '不留产物' task-server/task_server_lib.py` 只命中第 304 行自己），这句话在「取消的不留产物（用户：「取消的任务要自己删掉产物 不要残留」；remove_cancelled_artifacts）」（`task-server/task_server.py:64`），即 `task_server.py` 文件头「接口」的取消。改前那一版同一句已在，这一轮没碰；指路错一处，交主 agent 定改不改。

### 对得上的（成立）

- `task-server/task:67-68`、`task-server/attempt_runner.py:27-28`、`task-server/task_server.py:66`、`gpu-verification-server/control:392-393`、`prune-server/control:163-165`、`task-server/README` 三语第 163 行、`prune-server/README.md:27`、kb `tooling` 三语第 42 行：「本机组里的不删、第二台中途失败或被停的那一趟连它一起删」与 `task_server_lib.py:291-292`、`scripts/hosts/multi-host-run.sh:10-12` 逐句对上。
- `.claude/hooks/lib_workspace_cargo_lock.py:225` 只剩「这一道堵的是什么」，紧跟着的出路写「编译目录归树、不归任务：不用自己删、不直接 rm」，不再矛盾。
- `.claude/agents/three-way-attack.md` 第 9 步与 `scripts/agents/three-way-rerun-template.sh` 第 8 至 10 行：脚本与模型目录写成 `<仓根>/research/prompts/<轮>-opus-model/…`，照字面交给服务时 `bash` 找得到；与「编译目录」段「树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层」对得上（运行目录在工作根下，工作根往上没有 `.git`）。

## 要主 agent 跑的命令

两份探针都在草稿目录；仓副本 `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-opus/copy` 在 `.deliverables` 写成「交接」，跑完由主 agent 删（117M，`du -sh` 量的）。

1. 完工清理闸列表条件变异（打中乙）：副本里只有 `.claude/hooks/completion_cleanup_guard_decisions.py` 第 1342 行与主工作区不同（`diff` 只出这一行：`if not uncollected and not broken("kept-scenes-keep-unchecked"):`）。
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-opus/copy -- env SINGLEFS_MULTI_HOST_CONFIG=/home/fy5090/code/singlefs/multi-host.env bash .claude/hooks/completion-cleanup-guard.sh --selftest`
   - 在哪起：`--cwd` 已写在命令里（副本根）；从仓根交。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`，且 `task-server/task log <号>` 末尾是「判了 116 格；86 个弄坏开关各红在点名的格」，就是变异活着（打中乙坐实）；有「✗ 自证 」起头的格名行或退非 0，就是有格钉住了它（打中乙不成立）。对照臂是主工作区原件，主 agent 换上那一趟已跑出同一句（调度记录第 547 行），不用再跑。
2. 注释历史判法探针（第 2 问的推读数）：
   - 命令：`task-server/task submit -- python3 scripts/gate/comment-history-check.py --list /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-opus/comment-probe-root`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`task-server/task log <号>` 里以 `scripts/probe.sh:` 起头的行，预计恰好四行、行号 2、4、6、7（第 6 行命中词是「第二轮」，另三行是「攻方腿」）；第 1、3、5 行不出现。

两条都不碰 `crates/`、不编，不走崩溃枚举。

## 没打中的形状

- 第 1 问：会话认法换着试了分片子任务（子任务会话照父任务、闸与 `scene_due` 都挂到父任务）、人手 `--manual`（会话空串、闸不列、服务按结局时刻删；会话里带 `--manual` 当场拒）、`wait` 由别的会话起（`:1268` 不记取走，闸那一侧本来也不列别的会话的）、分片父任务没结局而子任务有（两边都跳过）、库里改过的行（闸不核签名会列，`clean` 回 409 删不动，下一回闸放行判在 `stop_hook_active` 那一支，属已知）。四条开关逐条对过样本与断言。
- 第 2 问：试了「」里放第N轮、日期、「此前」（照判）、没收尾的「（不算括着）、两对「」各自判冒号（不串）、套层时冒号在里层而角色名在外层（外层整段算，放过，已写进「管不到的」）、全角冒号。
- 第 3 问：表里四种加库行改坏、组那一扫抛错、符号链接、编译根没建（`listdir` 抛错直接返回）、在跑尝试的保护那一半删掉，都不会误删。
- 第 4 问：九处改过的「本机组里的 / 第二台连它一起删」逐句对 `task-server/task_server_lib.py:291-292` 与 `scripts/hosts/multi-host-run.sh:10-12`，对上；`.claude/hooks/lib_workspace_cargo_lock.py:225` 与出路不再矛盾；第 9 步与模板的绝对路径那一形照字面交得起（第二轮主 agent 已喂过闸，这一轮不重复）。

## 这条腿自己的限度

- 打中甲、丙、丁、戊都是按代码与文本推的，没跑；打中乙的读数是推的，待上面第 1 条。
- 换过会话号那一格核不动：Claude Code 在 `/clear`、`--resume` 之后钩子输入与 Bash 环境里的会话号怎么变，仓里没有记录、这一轮造不出。
- 服务库没读（读库归主 agent），第 3 问里那 11 个旧命名目录都在组里是照「没删成 0 个」按代码倒推的。
- 两份 README 的英文、日文只核了第 163、224 两行与 kb 第 42 行的那一段，别的行没逐句对。
- 正推腿那三问（正文第 5 到 7 问）没做；禁读清单里的另一条腿报告没读。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，两份探针的读数都待主 agent 跑。
- 写了的只有这份报告、草稿目录里的仓副本（一行变异）、探针根、`.deliverables` 与几份 `diff` 输出；没改仓里任何被判文件。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/build-dir-clean-wording-governance-r3-opus-output.md --background research/prompts/_build-dir-clean-wording-governance-r3-body.md` 末行：

「✓ 核了 25 处引文、44 处裸行号，对上 25 处，没判 44 处（其中裸行号没有引文可比 44 处；逐处列在上面）」

## 留下的

没删 /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-opus/copy：打中乙的探针副本（117M），要主 agent 在里面跑上面第 1 条，`.deliverables` 写成「交接」，跑完由主 agent 删。这一轮没建编译目录。
