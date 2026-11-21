# 编译目录归树旧说法收口治理文件轮第一轮：正推腿报告（build-dir-clean-wording-governance-r1，2026-11-21）

三问各自的结论：第 5 问（甲组逐条核）成立，附 3 处边界；第 6 问（乙组逐条核）6.1 至 6.7 成立，6.8 登记表备注数对、一句现状话对不上、一行推不出；第 7 问（全仓还剩的旧说法）有，6 处在被判的 28 份之外、1 处是被判文件里故意留的错样本。逐条在下面分节；腿没写模型、没起任何脚本。

## 读到的范围

- 被判的 28 份全读了主工作区此刻那一版：`sha256sum -c research/prompts/build-dir-clean-wording-governance-r1-snapshot/judged-sha256.txt`，不是 OK 的行数 0（命令是 `… | grep -vc ': OK'`，输出 0）。
- 改前原样：用正文点名的各草稿目录 `base/` 与 `git show HEAD:<路径>` 对着 `diff -u` 读（`completion_cleanup_guard_decisions.py` 155 行 diff、`completion-cleanup-guard.sh` 54 行、`completion_cleanup_guard_selftest.py` 101 行等，逐份读完）。HEAD 里没有的新文件（七份钩子的部分文件、`lib_split_family_loaded.py`、`lib_lag_split_family.py`）只对 `base/` 看，没有 HEAD 可比。
- 没用背景材料里的行号：下面每个 文件:行 都是 `grep -nF` 现取的。

## 判定写法

「成立」＝今天的文件就是正文那样写的，引的服务原文在被引的那一行、说的是同一件事；「不成立」＝文件与正文对不上；「推不出」＝要跑才知道或读文件判不了。

## 第 5 问：甲组逐条核

### 5.1 服务现行说法（正文第一条）：成立

四句引文都在被引的行、与正文抄的一字不差：

- 「命令拿到的是 CARGO_TARGET_DIR（组里那一个的 target）：入口照它编、不删」（`task-server/task_server_lib.py:281`）
- 「任务记下结局、task clean <号>、取消都不删组里的」（`task-server/task_server_lib.py:286`）；同一行下文「clean 只删那一条任务自己的现场」「取消的照旧删它自己的现场，编译目录放回组里。」（`task-server/task_server_lib.py:286`）
- 「wait 末行不带 build=（编译目录归树、不归任务）。」（`task-server/task_server_lib.py:287`）
- 「任务记下结局、`task clean`、取消都不删组里的编译目录。」（`task-server/README.md:224`）

补查的相邻行（本腿加的，正文没引）：「树删了编译目录跟着删」（`task-server/task_server_lib.py:283`）、「命令里命令位置上有会编的 cargo」（`task-server/task_server_lib.py:282`，正文问 2 要用）、「TASK_SERVER_BUILD_DIRECTORY」（`task-server/task_server_lib.py:38`）。

推翻它的现象：`task_server_lib.py:281`、`:286`、`:287` 或 `task-server/README.md:224` 任一行被改回「跑完留给提交它的 agent 删」。这一轮读到的是现在的一版。

### 5.2 完工清理闸删「服务编译目录」一支：成立（数与删法都对上），自证结局待主 agent 跑

- 文件头新段在 `.claude/hooks/completion-cleanup-guard.sh:27`：「服务给的编译目录（TASK_SERVER_BUILD_DIRECTORY 下按树分的组）谁都不判：它归树、不归任务，树删了（副本删了）服务跟着删它那一整组」，下一行 `:28`：「主 agent 收工时该删的是自己建的副本，归上面认领目录与临时草稿目录那几处判。」它引的 `task-server/README.md「编译目录」`（README 标题在 `task-server/README.md:218`）与 `task_server_lib.py` 文件头「编译目录」（段首 `task-server/task_server_lib.py:262`）都在。
- 开关 81 改 76：`grep -c '^#   COMPLETION_CLEANUP_BREAK='` 改前 `base/` 81、今天 76（本腿现数）。五条开关在 `completion_cleanup_guard_decisions.py` 的开关对照表里整五行不见了，今天那张表的相邻两行是 `.claude/hooks/completion_cleanup_guard_decisions.py:45`「("transcript-unreadable-unnoted", "transcript-unreadable-skip-reported"),」与 `:46`「("guard-off", "break-switch-refused-outside-selftest"),」。
- 四格自证：`completion_cleanup_guard_selftest.py` 里 `main-service-build-left`、`running-service-build-not-listed`、`other-session-service-build-ignored`、`subagent-service-builds-not-listed` 四个 `check(` 连同 `service_task` 造样本库的方法整段删了（`diff -u` 的删除块；五条开关对四格是因为 `service-builds-ignored` 与 `service-build-removal-by-rm` 都红在 `main-service-build-left`）。删掉的块原先夹在 `.claude/hooks/completion_cleanup_guard_selftest.py:388`（`subagent-ignores-sibling-runs` 那格）与 `:389`「# 别的会话认领的目录不判」之间，今天这两行相邻。
- 清单里不再有服务编译目录：`.claude/hooks/completion_cleanup_guard_decisions.py:1563`「leftovers = sorted(dict(not_in_use(found_in_roots + own_leftovers, found_in_runs) + snapshots).items())」，合入后验证快照 `snapshots` 仍在清单里。
- 同时删了为这一支才有的件：`import … sqlite3 …urllib.parse`、`KIND_SERVICE_BUILD`、`TASK_SERVER_DATABASE_VARIABLE`、`task_server_database`、`service_builds`、`removal_command` 里的服务编译目录分支。现文件里 `grep -nE 'sqlite3|urllib|task_server_lib|TASK_SERVER|task clean|build='` 在这三份里只剩文件头 `:27`、`:28` 两行（新写的那段）。
- 「退役判据」一句也跟着改：`.claude/hooks/completion-cleanup-guard.sh:128`「退役判据：工作根与崩溃放量的运行目录不再放副本与库拷贝（驱动与脚本都改成每趟自己的目录、跑完自己删），连续四周这道闸一次都没拦；」（原句后面有「服务给的编译目录改回由服务跑完就删」，已无）。
- 「不认交主 agent 删那一行」里去掉服务编译目录：`.claude/hooks/completion-cleanup-guard.sh:108`「库拷贝、旧文件库、库备份与迁库遗留不认这种写法」。
- 「不改成判任务现场」的两处依据：`task-server/task_server_lib.py:296`「什么时候删：提交它的会话 task wait 取走结局（collected_at）之后过 SCENE_KEEP_AFTER_COLLECT_SECONDS」，常量 `task-server/task_server_constants.py:297` 是 `3600.0`（一小时）；`task-server/task_server_lib.py:297`「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删。」两处说的就是正文说的那件事（现场由服务一小时后删、留现场的由 clean 删）。
- 待主 agent 跑（本腿不能起）：「各格都对（判了 110 格）；76 个弄坏开关各红在点名的格」与开关复跑 76 条逐格相同。命令在末尾「要主 agent 跑的命令」一节。

推翻它的现象：自证不是 110 格、或某条开关红的格与文件头点名的对不上；或今天的 `grep -c` 不是 76。

### 5.3 项目代码闸、共用库、`lib_workspace_cargo_lock.py`：成立，复跑读数待主 agent

- 拒绝说明：`.claude/hooks/lib_project_code_runs_base.py:1126`「编译目录归树、不归任务，不用自己删：」（下一行 `:1127` 接「树删了（自己建的仓副本删了）服务跟着删它那一整组）」）；python 删法那一条 `:1157`「python 代码里删 task-server 的编译目录（编译目录归树、不归任务，不用自己删：树删了（自己建的仓副本删了）服务跟着删它那一整组）」。
- `TASK_SERVER_BYPASS_POLICY`：`.claude/hooks/project_code_guard_decisions.py:1210`「服务给的编译目录归树、不归任务，只由服务删」，出路 `:1213`「服务给的编译目录不用自己删：要它没，删掉自己建的那份仓副本」。这两句与 `task-server/task_server_lib.py:283`（扫到树不是目录了就删那一组）一致；「主工作树那一组留给之后的任务接着编」与 `:264` 起「同一棵树在同一台上先后交的会编任务接着用」一致。
- `lib_workspace_cargo_lock.py`：文件头 `.claude/hooks/lib_workspace_cargo_lock.py:14`「服务按这棵树给一个仓外的编译目录、经 CARGO_TARGET_DIR 交给命令」；拒绝说明 `:228`；自证钉的串 `.claude/hooks/lib_workspace_cargo_lock.py:306`「"经 CARGO_TARGET_DIR 交给命令" in message」，项目代码闸自证那一份 `.claude/hooks/project_code_guard_selftest.py:1463`「"经 CARGO_TARGET_DIR 交给命令" in bare_main.stderr」；「task-server/task clean <号>」的钉法在两份里都没了。
- `project-code-guard.sh:212`：「出路：不用自己删，删掉自己建的仓副本，服务扫到树不在了就删它那一整组」，同一行前半仍写「lib_project_code_runs.build_directory_removal」。
- 共用库 `lib_project_code_runs_base.py:842` 注释「两处都只由服务删，谁都不直接删」、`:1038` 字段注释同改。三条开关补的 5 格：`.claude/hooks/project-code-guard.sh:587`（mark-entry-only，`:589` 补两格）、`:590`（family-by-directory，`:592` 补两格）、`:593`（family-by-name，`:595` 补一格）。五格的名字都在 `.claude/hooks/lib_project_code_runs_selftest.py`：`:262`（入口注释里点名……不在族里）、`:263`（importlib.import_module 载入）、`:266`（门禁阶段入口给绝对路径）；该文件 `:238`–`:239`、`:251`–`:252`、`:257`–`:258` 的注释写各开关下哪几格红，与头注释新补的对法一致（by-name 补「注释里点名」不补 import_module，by-directory 两格都补，mark-entry-only 补 import_module 与门禁阶段那格）。
- `refusal-submit-dropped`：`.claude/hooks/project-code-guard.sh:288` 今天写「红的格（实测）」，括号里「拒绝说明不写经 task-server 提交的出路与编译目录归树的说明」；开关控制的 `if not switch_is_set("refusal-submit-dropped"):`（`.claude/hooks/lib_workspace_cargo_lock.py:225`）那一块同时含提交出路与归树句，所以「出路与归树」两样一起去掉与文件头 `lib_workspace_cargo_lock.py:36` 的说法一致。「实测」二字背后的那次读数（只红一格）本腿没复现，待主 agent 跑。
- 待主 agent 跑：项目代码闸「查了 1492 种，其中该拒 332 种」、共用库「查了 467 种」、`lib_workspace_cargo_lock.py`「查了 32 格」、三份开关复跑（155、82、对应条数）、`WORKSPACE_CARGO_LOCK_BREAK=refusal-submit-dropped` 只红一格。

边界一条：`.claude/hooks/lib_project_code_runs.py` 文件头现有 82 行 `LIB_PROJECT_CODE_RUNS_BREAK=`（`grep -cE '^ +LIB_PROJECT_CODE_RUNS_BREAK=' .claude/hooks/lib_project_code_runs.py` 输出 82），与正文「共用库 82 条」一致；而 `.claude/gate.d/break-switch-replay-files.tsv:70` 的备注写「七十九条开关」，见 6.6。

推翻它的现象：自证格数与上面不符、或 `refusal-submit-dropped` 打开后红的不止「编译目录：拒绝说明给出经 task-server 提交的出路」一格。

### 5.4 技能与定义（dispatch、three-way、experiment-runner、three-way-attack、e176 注释）：成立，依据有两处只撑一半

今天的文件：

- `.claude/skills/dispatch/SKILL.md:18`：「`task-server/task clean <号>` 删的是那一条任务自己的现场」（括号里「分片父任务连同它的测试二进制」），前一句「编译目录归树、不归任务，不归主 agent 删，也不自己 `rm`：树删了（副本删了）服务跟着删它那一整组」。对 `task_server_lib.py:286`（现场与分片父任务的测试二进制）说的是同一件事。
- `.claude/skills/dispatch/SKILL.md:56`：「服务当场删掉那一条的现场、编译目录放回它那棵树的组里」，对 `task_server_lib.py:286` 的「取消的照旧删它自己的现场，编译目录放回组里。」；同行后半「有了结局的逐个 `task-server/task clean <号>`」仍对（clean 仍删现场）。
- `.claude/skills/three-way/SKILL.md:49`：「编译目录归树、不归任务，不自己删：树删了服务跟着删它那一整组」。
- `.claude/agents/experiment-runner.md:29`：「编与跑写在同一条交给服务的命令里」，给两种写法 `cargo run --release --bin <bin 名> -- <参数…>` 与 `bash -c 'cargo build --release --bin <bin 名> && "${CARGO_TARGET_DIR:?}/release/<bin 名>" <参数…>'`；`:44`：「生成产物用的二进制在产物命令那一条里照第 1b 步的写法现编现跑」。
- `.claude/agents/three-way-attack.md:35`：「调枚举函数的原型编与跑写成一条交」，后接 `task-server/task submit --cwd <副本> -- bash -c 'cargo build <参数…> && "${CARGO_TARGET_DIR:?}/<debug 或 release>/<bin 名>" <参数…>'`。三份定义里 `grep -rn 'build=' .claude/agents/*.md` 零命中（本腿跑的）。
- `scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml:9`：「编译目录由服务按树给、经 CARGO_TARGET_DIR 交给命令；归树、不归任务，不用自己删」；`:10`：「编的那一条与跑的那一条不一定派到同一台」。

引的依据逐条核：

| 依据 | 在不在那一行 | 说的是不是这件事 |
|---|---|---|
| `task_server_lib.py:281`「命令拿到的是 CARGO_TARGET_DIR（组里那一个的 target）：入口照它编、不删」 | 在 | 是：二进制取命令里的 `CARGO_TARGET_DIR` |
| `task_server_lib.py:287`「wait 末行不带 build=（编译目录归树、不归任务）。」 | 在 | 是：不再有 `build=` 目录可取 |
| `task_server_lib.py:282`「命令里命令位置上有会编的 cargo」（正文没引，本腿补） | 在 | 是：`bash -c '…cargo build…'` 能被认成会编，`lib_workspace_cargo_lock.py:7` 写「bash -c、cd 跟着走」，`:58` 的 `BUILDING_SUBCOMMANDS` 含 `build`、`run` |
| `.claude/rules/verification-reference.md:70`「手跑的 GPU 测试编与跑写成一条交，不先另交 `--no-run` 只编的那一条」 | 在 | 只撑一半：那一行说的是手跑的 GPU 测试这一类，不是实验装置二进制与枚举原型；写法同类、不是同一条规矩 |
| `task-server/README.md:172`「服务给的编译目录、调用方给的 `CARGO_TARGET_DIR`、草稿与临时目录下的 `target/`、注释、弄坏开关那一支与自证那一支……不算」 | 在 | 只撑一半：撑「不取仓根 `target/` 下的」（那一行判的是取仓根 `target/` 的警告），不撑「编与跑写进同一条」 |
| `prune-server/README.md:28` 同一写法（`bash -c 'cargo build … && cp "${CARGO_TARGET_DIR:?}/release/prune-server" …'`） | 在 | 是同一写法，只是它末尾是 `cp`、不是起二进制 |

所以这几句改法的依据站得住的是 `:281`、`:282`、`:287`；「编与跑要写成同一条」这一半，今天只有 `.claude/rules/verification-reference.md:70`（限 GPU 测试）与 `scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml:10` 的第二台理由（编译目录在第二台本地盘、`task_server_lib.py:276`「两台的组互不相干。」）能撑，`task-server/README.md:172` 撑不到。这不影响文件与服务一致，只影响正文「依据」那个括号写得比证据宽。

`<debug 或 release>` 与 cargo 的 profile 目录名是否对：本腿只在仓里找到同一约定的两处（`prune-server/README.md:28` 写 `release/`、实验员定义 `.claude/agents/experiment-runner.md:29` 写 `release/`），没有在仓里读到 cargo 自己的目录规则，这一点算外部知识、没在本项目现查，留给攻方腿问 2。rules-lint 对三份退 0：待主 agent 跑。

推翻它的现象：服务对 `bash -c 'cargo build … && "${CARGO_TARGET_DIR:?}/…"'` 不给编译目录（`:282` 起那段对 `bash -c` 里的 cargo 认不出），或第二台上 `CARGO_TARGET_DIR` 没交到命令手里。

### 5.5 判定库服务（`prune-server/control`、`prune-server/README.md`）：成立

- `prune-server/control:163`：「编法的提示：编译目录归树、不归任务（任务记下结局、task clean、取消都不删它，树删了才跟着删）」；`:165` 起的 `BUILD_COMMAND` 做法不变（`cargo build … && cp "${CARGO_TARGET_DIR:?}/release/prune-server" <共享根下的目录>/prune-server.new && mv …`），`:169`「task wait 的末行不点名编译目录：编译目录归树、在派到的那台自己的编译根下，别到那里取）」。依据两处：`task_server_lib.py:287`（末行不带 `build=`）与 `:271` 起「两台同一段（Machine.build_directory）：本机那一个在 <TASK_SERVER_BUILD_DIRECTORY>/tree-…/target」、`:231` 起「编译目录在第二台本地盘上」，都在、说的是这件事。
- 自证格 `prune-server/control:1391`：`expect(problems, "编译目录归树" in error and "末行不点名编译目录" in error,`，与 `:169` 两个短语对得上（`BUILD_COMMAND` 里有这两个词，故意写错的那份没有）。弄坏开关说明 `prune-server/control:103` 与文档串 `:236`、`:1383` 同改。
- 故意写错的那份提示 `prune-server/control:171`–`:172`（`BUILD_COMMAND_LEAVING_THE_BUILD_DIRECTORY`）仍写「task wait <号> 的末行 build=<目录>」：这是弄坏开关 `build-hint-leaves-build-directory` 的样本、注释 `:170`「弄坏开关 build-hint-leaves-build-directory 下的提示：只让取编译目录里的二进制、没拷到共享根」说明了它是错样本，不算旧说法没改。
- `prune-server/README.md:27`「编译目录归树、不归任务（任务记下结局、`task-server/task clean`、取消都不删，树删了才跟着删）」，`:31`「编译目录不用自己删：它归这棵树，树删了服务跟着删它那一整组」。`git diff HEAD -- prune-server/README.md` 还有「放认领」「列认领」「测试与自证起的临时服务」几段别的会话的改动，这一轮只核了 `:27`、`:31`。
- 待主 agent 跑：`prune-server/control --selftest` 退 0、`PRUNE_SERVER_CONTROL_BREAK=build-hint-leaves-build-directory` 只红 `install-build-hint-saves-binary`。

### 5.6 kb `.claude/kb/tooling.md:42` 三语：成立；三语六格退 0 待主 agent 跑

- 中文 `.claude/kb/tooling.md:42`：「编译目录归树、不归任务，记下结局、`clean`、取消都不删它，树删了（副本删了）服务跟着删它那一整组」，同句前面「`clean <号>` 删这条任务自己的现场（分片父任务连同它的测试二进制）」；对 `task_server_lib.py:286` 一致。
- 英文 `.claude/kb/tooling.en.md:42`：「build directories belong to the tree, not to the task」；日文 `.claude/kb/tooling.ja.md:42`：「ビルドディレクトリは木に属しタスクには属さず」。三语同一句、同样有「删现场」与「树删了服务删整组」两层。
- 原句「第二台那一趟的编译目录由 `multi-host-run.sh` 跑完自己删」今天三份里都没有（`git diff HEAD -- .claude/kb/tooling*.md` 里是删除行）；它不真的依据是 `task-server/task_server.py:603`「都带 --keep-build-directory 跑完留着（服务不设 CARGO_TARGET_DIR，task clean 不管它）」与 `task_server_lib.py:275`（跑完打了 `PEER_BUILD_KEPT_MARK` 的组里记成可接着用）。
- 同一行后半句的领卡写法没改：今天 `.claude/kb/tooling.md:42` 仍有「任务自己在跑的那一次尝试里按需领卡、还卡用 `lease take`、`lease return`」，登记在 `records/2026-11-19-未做完的任务登记.md:35`（该行点名了 `tooling.md` 第 42 行三语后半句），这一轮不改。`git diff HEAD` 里同一段还有别的会话改的「服务只调配 CPU 与内存，不分卡、不领卡、不清场」，与那半句并存，已登记、不重复报。

## 第 6 问：乙组逐条核

### 6.1 检出钩子开关表四行行尾加 history-kept：成立（文件如正文所写）；理由站不站得住归攻方问 3 与问 4

- 四行是 `.claude/hooks/bash-command-detector.sh:636`、`:804`、`:965`、`:1027`，行尾都是「history-kept: 自证格名里的三方腿角色名，不是来历」（`diff -u` 对 `comment-history-hooks/base/` 只多这四处行尾，别处没动）。
- 理由字数 17（`python3 -I -c 'print(len(…))'` 数的），门禁下限在 `.claude/gate.d/doc-text.sh:899`「特殊缘由：那一行的注释里写 history-kept 标记加至少 8 个字的理由，那一行放过；理由不够长的判红并点名。」；四行都是以 `#` 起头的注释行，标记在行内注释里，形态合。
- 被标的那几句是自证格名，名字确是代码里拼出来的：`.claude/hooks/bash_command_detector_selftest_waits_and_scope.py:1053`「第二次同立场的攻方写第一次那条腿的报告，拒」，`:1063` 拼成 `Bash写范围:三方腿:{label}`。所以「是自证格名」属实。
- 本腿留给主 agent 判的一点（事实、不是裁决）：`.claude/agent-common.md:47` 写「特殊缘由只三类：日期本身是数据」，三类是日期本身是数据、引外部文档、绕开外部工具缺陷；这四行的理由「自证格名里的三方腿角色名，不是来历」字面上不在三类里，门禁（`.claude/gate.d/doc-text.sh:899`）只看 8 个字下限、不看类。

推翻它的现象：门禁 comment-history 格单跑时这四行仍被点名；或四行里有一处说的是改动经过（不是格名）。门禁结局待主 agent 跑。

### 6.2 派发闸 `runner_dispatch_guard_locks_and_verdicts.py:672`：成立

今天 `.claude/hooks/runner_dispatch_guard_locks_and_verdicts.py:672` 行尾是「history-kept: 被判的标题写法示例，日期与重跑次数本身是数据」（理由 22 个字）；同一行前半「### 2026-10-20：…（第五次重跑…」是被判的标题写法例子，属 `.claude/agent-common.md:47` 里的第一类「日期本身是数据」。

### 6.3 写闸 `write-guard.sh:107`：成立

`.claude/hooks/write-guard.sh:107`：「被改那几份样本此刻的样子拷进 <草稿目录>/fixtures-base/」，原句「改前的样本拷进」已不在（`diff -u` 的删除行）。新句说的是出路里要拷什么，与同一段 `.claude/hooks/write-guard.sh:105`–`:107`（已有门禁的样本档目录那一条与它的出路）一致。

### 6.4 写闸自证 `write_guard_selftest.py:602`：成立

`.claude/hooks/write_guard_selftest.py:602`：「派发闸收、值里另带别的写法的号的那四条与两臂、单臂各一条」；原来的「攻方第二轮 3f 那四条」今天 `grep -nF '攻方第二轮' .claude/hooks/write_guard_selftest.py` 零命中。

### 6.5 注释历史滞后表删四行与「改两行路径」：成立

- 今天 `.claude/gate.d/comment-history-lag.tsv` 里没有以 `.claude/hooks/` 起头的行（`grep -c '^\.claude/hooks/'` 输出 0）；暂存区那一版 `git show :.claude/gate.d/comment-history-lag.tsv` 的第 17–20 行是 `bash-command-detector.sh`（4）、`runner_dispatch_guard_locks_and_verdicts.py`（1）、`write_guard_selftest.py`（1）、`write-guard.sh`（1）；HEAD 那一版一行 `.claude/hooks/` 都没有（`git show HEAD:… | grep -c` 输出 0）。所以这四行「基准里没有」属实：它们是这一轮自己在暂存区加的，按 `.claude/gate.d/comment-history-lag.tsv:6`「只缩不涨：与 gate 基准那一版按路径比，基准里没有的行判红、数比基准大判红」算涨，删掉合表头；表头 `:8`「改净一份文件就删它那一行」。「改两行路径」指的是上面第 18、19 行两份下划线名（推的，依据 `records/2026-11-17-subagent账本清欠调度记录.md:525`「两行改成新路径（改名后各仍命中 1 行）」，暂存区那一版里这两行各为 1；旧连字符名在 HEAD 的 `git ls-tree` 与暂存区的 `git ls-files` 里都没有），之后与另两行一起删，所以终态是四行全无。
- 表里每个路径都在盘上：`awk -F'\t' '!/^#/ && NF>=2 {print $1}' … | while read p; do [ -e "$p" ] || echo MISSING $p; done` 没有输出。
- 与这条相连、但属别的会话的：同一文件里崩溃点重放、doc-process-records 一族的行数变动不是这一轮的，不报。
- 删完后这四份钩子是否真的零命中：本腿没法读判（词表在 `scripts/gate/comment-history-check.py`，跑它要交任务）。命令在末尾「要主 agent 跑的命令」一节。

推翻它的现象：`comment-history-check.py --list` 的输出里还点名这四个文件之一；或 `doc-text.sh --check comment-history` 因这四份红。

### 6.6 搬迁登记表删 14 行：成立

- 今天 `.claude/gate.d/path-moves.tsv` 745 行（`wc -l`）。被删的 14 行的原文：本腿在工作根下另一个会话的集成副本里读到（该副本 `path-moves.tsv` 共 759 行，第 746–759 行就是这 14 行，头一行 `.claude/hooks/write-guard-decisions.py` → `.claude/hooks/write_guard_decisions.py`，末一行 `.claude/hooks/bash-command-detector-selftest-entries-and-main.py` → `.claude/hooks/bash_command_detector_selftest_entries_and_main.py`），14 个旧前缀都是连字符名的钩子部分文件。
- 判据在表头：`.claude/gate.d/path-moves.tsv:5`「这张表写得对不对、此刻生效的行串没串、有没有搬迁实据（基准里旧前缀下的每份文件此刻在新前缀下都在且被 git 跟踪，新前缀在基准里没有文件，旧前缀此刻不在磁盘上；出处在）由门禁 code-tooling 的 change-range-single-source 格判。」基准里这 14 个旧前缀下没有文件：`git ls-tree -r HEAD --name-only -- .claude/hooks` 里与之相关的只有 `.claude/hooks/lib_project_code_runs.py`，七份钩子的部分文件（下划线名）今天在暂存区是新增（`git status` 的 `A`），HEAD 里没有一份。所以「旧前缀在基准里没有文件、不算搬迁」属实。
- 删后今天表里没有 `.claude/hooks/` 下连字符名的行（`grep -nE '^\.claude/hooks/'` 只有第 8、9 行两条 2026-11-05 的旧搬迁）。
- 删了有没有漏判：本腿读判不了，`change-range-single-source` 格单跑待主 agent 跑。

推翻它的现象：该格单跑仍红在 `path-moves.tsv`；或基准里那 14 个连字符名路径其实有文件。

### 6.7 认族共用库两处措辞：成立

- `scripts/gate/lib_split_family_loaded.py:17`：「点号前面那一段模块名加 .py（缩进里、函数里的也算）」；`grep -n '以前' scripts/gate/lib_split_family_loaded.py` 零命中。
- `scripts/gate/lib_lag_split_family.py` 文件头：Python 拆法一段 `:11`–`:12` 例子「.claude/hooks/write-guard.sh 加 write_guard_decisions.py」，钩子拆法一段 `:13`–`:14` 例子只剩「scripts/gate/merge-and-verify.sh 加 merge-and-verify-steps-lib.sh」。例里点名的文件核了 `merge-and-verify.sh`、`merge-and-verify-steps-lib.sh`、`broad-mutation.py`、`broad_mutation_base.py`、`prove-red.sh`、`prove_red_core.py`、`code-tooling-source-scan-cells.sh`、`broad-mutation-samples.py`、`write_guard_decisions.py`，都在盘上（`[ -e … ]` 逐个核，没有 MISSING）。`:11`「Python 拆法（入口是 .py 或 .sh）」写明模块名「-」换成「_」，所以 `write-guard.sh` 的部分文件是下划线名属 Python 拆法、不属钩子拆法，例子挪对了地方。

### 6.8 两张登记表的备注：数对，一处现状句对不上，一处推不出

| 行 | 今天写的 | 核对 | 判定 |
|---|---|---|---|
| `.claude/gate.d/break-switch-replay-files.tsv:10` 完工清理闸 | 「七十六条开关（文件头 COMPLETION_CLEANUP_BREAK= 现数：服务编译目录那一支连同它的五条开关去掉的那一版，换上之后还没复跑；」 | 76 与文件头 `grep -c` 一致 | 数成立；现状句「换上之后还没复跑」与正文第 17 行「开关复跑 76 条逐格相同」对不上（先后次序本腿读不出：谁在后）。同一行下文「60 ×（1 + 79 ÷ 31 向上取整）= 240」仍写 79：76 与 79 向上取整都是 3、结果 240 不变，但字面还是 79 条那一版的算式 |
| `.claude/gate.d/break-switch-replay-files.tsv:95` make-swap-spec | 「十五条开关（文件头 MAKE_SWAP_SPEC_BREAK= 现数，照 switch_entries 的读法）」与「造小仓经 lib_repository_copy 起 rsync 拷文件」 | `scripts/gate/make-swap-spec.py` 文件头 `grep -c '^  MAKE_SWAP_SPEC_BREAK='` 输出 15；`scripts/gate/make-swap-spec.py:32` 写「出规格不起子进程（自证搭假仓经 scripts/hosts/lib_repository_copy.py 的 copy_entries 起 rsync，所以运行条件要有 rsync）」 | 成立；「自证十六格」那个数本腿没数 |
| `.claude/hooks/selftest-registry.tsv:138` make-swap-spec | 「取工作根与仓副本的几格造小仓经 lib_repository_copy 起 rsync 拷文件」 | 同上 `make-swap-spec.py:32` | 成立 |
| `.claude/gate.d/break-switch-replay-files.tsv:70` lib_project_code_runs | 「七十九条开关」 | 文件头 `grep -cE '^ +LIB_PROJECT_CODE_RUNS_BREAK='` 输出 82；该行记「照 switch_entries 的读法」 | 推不出：本腿没跑 switch_entries 的读法；若读法同 `grep`，备注该是 82（正文也写「共用库 82 条」） |

这一行 `:70` 不在这一轮的未暂存改动里（`git diff -- .claude/gate.d/break-switch-replay-files.tsv` 的五处变动里没有它），归不归这一轮由主 agent 定。

## 第 7 问：全仓还剩的旧说法

范围：`git grep -n --untracked` 对全仓，排除 `research/`、`records/`、`*/target/*`，另在个别查法里排除 `scripts/experiment-apparatus/`（`build=` 在那里是产物行字段）与任务服务自证文件；查法写在每条后面。变更史（`.claude/kb/decisions-history.md`、`experiments-history.md`）与已知开着的几处（`.claude/agent-common.md:37` 现已是新说法、`scripts/mutation/crates_mutation_rows_stage.py:745` 现已是新说法）不单列。

「`task wait` 末行 `build=`」的旧说法：`git grep -n --untracked -E 'build=' -- . ':!research/' ':!records/' ':!*.lock' ':!*/target/*'` 109 行，逐文件看：除 `prune-server/control:172` 外都是别的意思（产物行 `E203RESULT … build=$build`、`build=("${@…}")` 数组、`task_server_lib.py:287` 与 `task-server/README*.md:53` 的「不带 `build=`」新说法）。`prune-server/control:172` 是故意写错的弄坏开关样本（见 5.5），不算。

「编译目录用完 `task clean` 删」的旧说法，今天还在的：

| 文件:行 | 原句 | 与今天服务对不上在哪 |
|---|---|---|
| `task-server/task:65`、`:66` | 「服务在同一次取消里删这条任务各次尝试的」「编译目录与现场目录」 | 取消只删现场、编译目录放回组里（`task-server/task_server_lib.py:286`；`task-server/README.md:163`「编译目录归树的那一组，放回、不删」） |
| `task-server/task:68`、`:69` | 「只许提交它的会话、有了结局才删：这条任务留下的编译目录（删完回读不在），打删了几份；」「没删干净退 5，逐份写明哪个编译目录、为什么没删成」 | `clean` 只删现场与分片父任务测试二进制（`task_server_lib.py:286`；`task-server/README.md:165`「编译目录归树的那一组，不删」）；自证 `task-server/task_server_selftest_lifecycle.py:871` 期望「删了 0 份」 |
| `task-server/task:231` | 「去掉它再提交：服务给每一次尝试一个仓外的编译目录、跑完删」 | 服务给的是树的组、不跑完删（`task_server_lib.py:283`、`:286`） |
| `task-server/attempt_runner.py:27` | 「子进程退了，编译目录不删（任务有了结局由服务删」 | 结局之后服务也不删，树删了才删（`task_server_lib.py:286`、`:283`） |
| `multi-host.env.example:114` | 「跑完留给提交它的会话 task-server/task clean <号> 删」 | `clean` 不删编译目录（`task_server_lib.py:286`） |
| `gpu-verification-server/control:392` | 「编法：编译目录归任务服务、任务一结束就删，所以编和拷到共享根在同一条提交命令里做完」 | 同 `prune-server/control:163` 已改的那句，编译目录归树不归任务；它后面的命令写法（`:393`–`:396`）仍对 |

查法：`git grep -n --untracked -E '编译目录.{0,80}(task clean|clean <号>|clean `?<号>)|(task clean|clean <号>).{0,80}编译目录'` 18 行；再 `git grep -n --untracked -E '(取消|cancel).{0,70}编译目录|编译目录.{0,70}(取消|cancel)|编译目录.{0,40}(任务一结束|跑完就删|用完.{0,6}删|跑完.{0,6}删|由服务删)'` 95 行；逐行读过，剔掉新说法（服务自己的文件头、README 三语、完工清理闸新段、项目代码闸、技能、kb、prune 两处）与别的工具各自的编译目录（`scripts/hosts/run-build-directory.sh`、`scripts/crash/`、`scripts/mutation/prove-red*`、`scripts/gate/gate-staged.sh` 这些「自己建、自己跑完删」的）。

只缩到上面这个查法的范围：没读 `.claude/kb/` 里其他页之外的英日译文逐句、没扫 `.claude/singlefs-ai-sop/` 里的非 `.md`；三语 README `task-server/README.en.md:224`、`task-server/README.ja.md:224` 与 `task-server/README.md:224` 同句，已核。

## 要主 agent 跑的命令

都在仓根起（`--cwd` 缺省即仓根）；本腿一条都没起。结局看各自任务末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`，下面写了「看哪几行」的再读 `task-server/task log <号> --all`。

1. 完工清理闸自证。命令：`task-server/task submit -- bash .claude/hooks/completion-cleanup-guard.sh --selftest`。看：`type=succeeded exit=0`，日志里「各格都对（判了 110 格）；76 个弄坏开关各红在点名的格」。对应 5.2。
2. 项目代码闸自证（连同共用库 `lib_project_code_runs.py` 的自检）。命令：`task-server/task submit -- bash .claude/hooks/project-code-guard.sh --selftest`。看：`exit=0`，「查了 1492 种，其中该拒 332 种」与共用库「查了 467 种」。对应 5.3。
3. `lib_workspace_cargo_lock.py` 自证。命令：`task-server/task submit -- python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest`。看：`exit=0`，「查了 32 格」。对应 5.3。
4. 证红 `refusal-submit-dropped`。命令：`task-server/task submit -- env WORKSPACE_CARGO_LOCK_BREAK=refusal-submit-dropped python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest`。看：`exit=1`，✗ 行只有「编译目录：拒绝说明给出经 task-server 提交的出路」一格（连同汇总行）。对应 5.3。
5. 判定库服务自证。命令：`task-server/task submit -- python3 prune-server/control --selftest`。看：`exit=0`；`PRUNE_SERVER_CONTROL_SELFTEST cells=20 failed=0`（今天的格数以输出为准）。对应 5.5。
6. 证红 `build-hint-leaves-build-directory`。命令：`task-server/task submit -- env PRUNE_SERVER_CONTROL_BREAK=build-hint-leaves-build-directory python3 prune-server/control --selftest`。看：`exit=1`，✗ 行只有 `install-build-hint-saves-binary`。对应 5.5。
7. 完工清理闸开关复跑。命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格里完工清理闸一份的开关; 范围=1 份 76 条开关; 为什么=核登记备注写的换上之后还没复跑与文件头 76 条开关逐格相同' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/completion-cleanup-guard.sh`。看：`exit=0`，「复跑了 76 条开关（1 份）…逐格相同 76 条」。对应 5.2、6.8。
8. 共用库开关复跑（定 6.8 里 79 对 82）。命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格里共用库 lib_project_code_runs.py 一份的开关; 范围=1 份; 为什么=核登记备注七十九条与文件头 82 行哪个是现数' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/lib_project_code_runs.py`。看：「复跑了 N 条开关（1 份）」的 N 是 79 还是 82。对应 6.8。
9. 注释历史格。命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-text.sh 的 comment-history 一格; 范围=1 格; 为什么=核乙组四份钩子的 history-kept 标记与滞后表删四行之后这一格点名的文件' task-server/task submit -- bash .claude/gate.d/doc-text.sh --check comment-history`。看：日志里不点名 `.claude/hooks/bash-command-detector.sh`、`.claude/hooks/runner_dispatch_guard_locks_and_verdicts.py`、`.claude/hooks/write-guard.sh`、`.claude/hooks/write_guard_selftest.py`；已知开着的算键库两处与 `scripts/gate/large-file-git-hook.py` 那行照旧不算这一轮。对应 6.1 至 6.5。
10. 逐行命中表。命令：`task-server/task submit -- python3 scripts/gate/comment-history-check.py --list .`。看：输出里（每行「路径:行号:命中的词:原文」）上面四个文件的命中行都带 `history-kept` 或已不在；这一条只读。对应 6.1。
11. 搬迁登记格。命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 change-range-single-source 一格; 范围=1 格; 为什么=核搬迁登记表删 14 行之后这一格' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check change-range-single-source`。看：`exit=0`，「搬迁登记表 .claude/gate.d/path-moves.tsv … 写法对、出处都在」与「change-range-single-source：绿」。对应 6.6。
12. 三语格。命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-text.sh 的三语六格; 范围=6 格; 为什么=核 kb tooling 三语第 42 行改动' task-server/task submit -- bash .claude/gate.d/doc-text.sh --check trilingual-files,trilingual-structure,trilingual-changed-together,trilingual-glossary,trilingual-untranslated,trilingual-omission`。看：`exit=0`。对应 5.6。
13. 三份定义的 rules-lint：本腿没在 `.claude/singlefs-ai-sop/scripts/rules-lint.sh` 文件头里读到点名定义文件的写法（它扫 `rules/*.md` 与 `RULES_LINT_FILES` 点名的文件），命令写不出，这一格写「复核不了：命令形态没读到，请主 agent 照正文里那次的原命令重交」。对应 5.4。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 5.1 服务现行说法四句 | 成立 | `task_server_lib.py:281`、`:286`、`:287` 与 `task-server/README.md:224` 的原文都在被引的行，说的是正文那件事 |
| 5.2 完工清理闸删支 | 成立 | 文件头新段在 `completion-cleanup-guard.sh:27`、开关 81 改 76、四格自证与五条开关都删了；110 格与 76 条复跑待主 agent 跑 |
| 5.3 项目代码闸、共用库、`lib_workspace_cargo_lock.py` | 成立 | 拒绝说明、出路、自证钉的串、三条开关补的 5 格都在，且与 `:283`、`:286` 一致；1492 种、467 种、32 格、开关 155 与 82 条待主 agent 跑 |
| 5.4 技能与两份定义、e176 注释 | 成立 | 文件如正文；依据 `:281`、`:282`、`:287` 站得住，`.claude/rules/verification-reference.md:70` 只限 GPU 测试、`task-server/README.md:172` 只撑「不取仓根 target」，括号写得比证据宽 |
| 5.5 判定库服务 | 成立 | `control:163`、`:169`、`:1391` 与 `prune-server/README.md:27`、`:31` 对得上服务；故意错的样本 `control:171`–`:172` 不算旧说法 |
| 5.6 kb 三语第 42 行 | 成立 | 三语同句；原句「第二台那一趟…跑完自己删」已无；三语六格待主 agent 跑 |
| 6.1 检出钩子四行 history-kept | 成立 | 四行在 `:636`、`:804`、`:965`、`:1027`，理由 17 字；理由是否合 `.claude/agent-common.md:47` 的三类留给主 agent、攻方问 3 |
| 6.2 派发闸 `:672` | 成立 | 标记在行尾，属日期本身是数据 |
| 6.3 写闸 `:107` | 成立 | 「改前的样本」已改成「被改那几份样本此刻的样子」 |
| 6.4 写闸自证 `:602` | 成立 | 「攻方第二轮 3f」零命中 |
| 6.5 滞后表四行 | 成立 | 今天无 `.claude/hooks/` 行；HEAD 无、暂存区有，算涨，删对；零命中待主 agent 跑 |
| 6.6 搬迁登记表 14 行 | 成立 | 14 行原文读到了（别的会话集成副本 746–759 行）；旧前缀在 HEAD 无文件；格单跑待主 agent 跑 |
| 6.7 认族共用库两处 | 成立 | `lib_split_family_loaded.py:17` 与 `lib_lag_split_family.py:12`、`:14` 如正文 |
| 6.8 两张登记表备注 | 不成立（一句）、推不出（一行） | `.claude/gate.d/break-switch-replay-files.tsv:10` 数 76 对，但「换上之后还没复跑」与已复跑对不上、算式仍写 79；`:70` 备注 79 对文件头 82 |
| 7 全仓旧说法 | 有 | `task-server/task:65`–`:69`、`:231`，`task-server/attempt_runner.py:27`，`multi-host.env.example:114`，`gpu-verification-server/control:392`，共 6 处在被判的 28 份之外；被判文件里只有 `prune-server/control:172` 一处是故意错样本 |

## 没做什么

- 不判攻方腿的问 1 至问 4（漏拦、命令能不能跑、闸会不会拒、`history-kept` 理由能不能站住）：本腿只记了与第 5、6 问相交的事实，没替主 agent 采纳或出判决。
- 没起任何脚本、没跑模型、没往统一任务服务交任务；判决要引的读数（各格数、复跑条数）都没复跑，命令在上一节。
- 被删的 14 行搬迁登记行原文不是主 agent 草稿里的改前副本，是别的会话集成副本里的同一份；两份是否逐字相同本腿没比。
- 全仓扫描限在第 7 问写的查法范围；英日译文只扫了 `task-server/README*.md` 与 `kb/tooling*.md`，没逐页扫别的译文。
- 没读攻方腿这一轮的报告（禁读清单）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-sonnet` 只放了查旧说法的 grep 输出与一份开关登记差异（共 144K，没有仓副本与编译目录）；未建模型目录。
