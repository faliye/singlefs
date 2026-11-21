# 编译目录旧说法收口治理文件轮第二轮：辩方腿报告（build-dir-clean-wording-governance-r2，2026-11-21）

被复核的是第一轮判决 `research/prompts/build-dir-clean-wording-governance-r1-main-verification.md`「二、判决」七条与「三、改法」里的改法 1、改法 4。这条腿没写模型，没起任何脚本，没往统一任务服务交任务；读的是主工作区此刻的文件（被判的 30 份用 `sha256sum -c research/prompts/build-dir-clean-wording-governance-r2-snapshot/judged-sha256.txt` 对过，见下面「读到的范围」），写过一份服务状态的只读查询（`task-server/task status <号> --json`，号取自现场目录名）。没读禁读清单里的另一条腿报告。

## 两问的结论

- 第 5 问（七条）：七条的结论都站得住；只有第 1 条里「这类现场此刻还有长期留着的（最早 2026-11-04）」这一句的依据对不上它要撑的话（量的：最早两个现场不是设了留现场的任务，是已有结局、没人取走的任务），结论本身（设了留现场的现场没人提）仍成立。第 2 条里 `<debug 或 release>` 漏三种写法一句判决说「cargo 的输出目录规则，不另做实验」，本机离线文档里读不到那条规则，写「推不出」，探针放在「要主 agent 跑的命令」。
- 第 6 问（改法 1、4 选的路）：改法 4 选「修判法」同样打中的只有「「」里的来历句放过」这一种，而且判法文件头自己写了（`scripts/gate/comment-history-check.py:177`）；三个替代方案各自更宽或更重，成立。改法 1 选「只判设了留现场的」没有同样打中所有替代方案：它对「不判」成立，对「判全部现场」推不出，原因是它文件头抄的服务口径少了一半（服务认「没取走」也不删，闸只认「设了留现场」）。

## 读到的范围

- 判决、两条第一轮腿的报告、第二轮正文 `research/prompts/_build-dir-clean-wording-governance-r2-body.md` 全读。
- 现查原文并取行号的文件：`task-server/task_server_lib.py`、`task-server/task_server_scheduler_building.py`、`task-server/task_server_machines.py`、`task-server/task_server_lib_configuration.py`、`.claude/hooks/completion-cleanup-guard.sh`、`.claude/hooks/completion_cleanup_guard_decisions.py`、`.claude/hooks/completion_cleanup_guard_selftest.py`、`scripts/gate/comment-history-check.py`、`.claude/agent-common.md`、`.claude/agents/three-way-attack.md`、`scripts/agents/three-way-rerun-template.sh`、`.claude/gate.d/break-switch-replay-files.tsv`、`.claude/gate.d/comment-history-lag.tsv`、`.claude/gate.d/path-moves.tsv`、`.claude/hooks/lib_workspace_cargo_lock.py`。
- 改前原样读的是 `~/.local/share/singlefs/work/build-dir-clean-wording/base/` 下的完工清理闸判法（第一轮开工前那一版，里面有被删的那一支）与 `git show HEAD:<路径>`。
- 只有这条腿读到的范围：现场目录与服务库里的真实任务状态（`task-server/task status`），其余与攻方腿同一份。

## 判决逐条：辩方判定一览

| 判决条目 | 辩方判定 | 证据（文件:行与原文，或命令） |
|---|---|---|
| 第 1 条 攻方第 1 问（删支后漏拦；采） | 结论成立；依据句「长期留着的现场」推不出（对不上那一类）；低一级（树 = `--cwd` 本身）成立 | 结论：`task-server/task_server_lib.py:297`「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删。」；依据句的反例见第 1 条一节 |
| 第 2 条 攻方第 2 问（主体不成立；`rerun.sh` 取法坐实） | 主体成立（本机）、第二台是推的；`rerun.sh` 那一条成立（两臂都量了）；`<debug 或 release>` 漏三种写法是推的 | 探针两臂：脚本形 `cargo_target_dir=unset`、`bash -c` 形给出仓外路径（判决第一节原文）；`${CARGO_TARGET_DIR:?}` 在脚本里退 1（本腿跑了一次，见第 2 条一节） |
| 第 3 条 攻方第 3 问（主体不成立；`--cwd` 无 `.git` 采） | 成立 | 树名里的 8 位十六进制用 sha256 现算对上，见第 3 条一节 |
| 第 4 条 攻方第 4 问（滞后表、搬迁表、检出钩子四处、派发闸一处） | 成立；「或登常设豁免」那一支实际不可用 | HEAD 里滞后表与搬迁表都没有那些行；豁免表整份文件不判，`scripts/gate/comment-history-check.py:50` |
| 第 5 条 正推第 5 问（依据写得比证据宽，不改文件） | 成立 | 被判的两份定义与两份技能里没有引 `.claude/rules/verification-reference.md:70` 与 `task-server/README.md:172`（grep 零命中） |
| 第 6 条 正推第 6 问（6.1 至 6.7 成立，6.8 采） | 成立 | `.claude/gate.d/break-switch-replay-files.tsv:10` 与 `:70` 此刻写八十二条、复跑「开关 82 条、对上 82 条」；`grep -c` 开关行都是 82 |
| 第 7 条 正推第 7 问（还剩 6 处旧说法，采） | 成立；六处此刻都已改；被判文件里还剩一处同类话（推不出是不是假话） | `.claude/hooks/lib_workspace_cargo_lock.py:225`「编译目录用完就删，主工作区的 target/ 删了又长的那条路由这一道堵上」 |
| 改法 1（另判设了留现场的任务现场） | 对「不判」成立；对「判全部现场」推不出 | 第 6 问一节 |
| 改法 4（修判法，不登豁免、不单行标记） | 成立 | 第 6 问一节 |

## 第 1 条：攻方第 1 问（删支后漏拦）

引文与结构，现查：

- `task-server/task_server_lib.py:297`「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删。」；`task-server/task_server_scheduler_building.py:319`「if (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1" and not lib.broken("scene-keep-ignored", self.environment):」：服务对这一类现场没有到期删。
- 「删掉的那一支是收工时唯一叫主 agent 起 `task clean <号>` 的地方」成立：改前原样 `~/.local/share/singlefs/work/build-dir-clean-wording/base/.claude/hooks/completion_cleanup_guard_decisions.py:1179`「def service_builds(role, session, notes):」整支只读服务库、列「编译目录还在盘上」的任务，删法是 `task clean <号>`；那一份里 `task clean` 只出现在这一支与它的删法行。
- 删支本身对：`task-server/task_server_lib.py:286`「任务记下结局、task clean <号>、取消都不删组里的」，留着那一支只会每次列同一批。

依据句对不上的地方（量的，辩方独有）。判决「现数」一节与「二、判决」第 1 条用「`~/.local/share/singlefs/work/task-server-scenes/` 下 7 个现场目录，最早 2026-11-04」撑「这类现场此刻还有长期留着的」。这 7 个目录我逐个用 `task-server/task status <号> --json` 查了（号取自目录名；这里不写号，写修改日期）：

| 现场目录的修改日期 | 提交时设了 KEEP | 状态 | 已取走（collected） |
|---|---|---|---|
| 2026-11-04 | 没设（environment 里没有这个键） | finished（succeeded） | 否 |
| 2026-11-06 | 没设 | finished（stalled） | 否 |
| 2026-11-12 | 设了 1 | finished（command-failed） | 是 |
| 2026-11-20 四个 | 没设 | 其中三个是同一条在跑的任务的各次尝试，另一个 finished | 在跑的否、finished 的是 |

读数：7 个目录里只有修改日期 2026-11-12 的一个是「设了留现场」的任务；判决说的「最早 2026-11-04」那一个和 2026-11-06 那一个都没设，是已有结局而没人 `wait` 取走的任务。这两类在服务里是两种不同的留法：

- `task-server/task_server_lib.py:301`「没人 wait 的任务（列在 GET /sessions/self 的「完成了没人取」里）一直留到 task clean。」；对应 `task-server/task_server_scheduler_building.py:317`「if not task.get("collected") and (task.get("session") or lib.broken("scene-sessionless-kept", self.environment)):」，有会话、没取走就不到期。
- 设了留现场的：上面 `:297` 与 `:319`。

所以「最早 2026-11-04」撑不起「设了留现场的现场长期留着」；它撑的是「没取走的现场长期留着」。设了留现场的那一类，现存读数只有一个（2026-11-12，已取走、八天没人 clean）。这一句依据推不出判决的那一类，结论（这一类服务不删、收工时原先靠删掉的那一支列）仍由 `:297`、`:319` 两处代码撑住，成立。

这也是两臂分不分辨的回答：判决引的 7 个目录读数不分辨「留着删掉的那一支」与「删掉那一支」两臂（最早两个现场不是设了留现场的，改前那一支只列有编译目录的任务，这两个任务 `compiles` 为 false、尝试里的 `build_directory` 为空，改前那一支也不会列它们）。分辨臂的是代码读法（`service_builds` 只读 `build_directory`）与判决写的那段历史（设了留现场、又会编的任务收工时被带着清掉），后者是推的，没人照它喂过。

改前那一支只覆盖「会编」的任务：设了留现场而不编的任务，改前也没有人提。所以判决「删支之后漏拦」对不编的任务是旧缺口、对会编的任务才是删支带出的新缺口；改法 1 的另判一并补上两种，不受影响。

低一级（新文件头把「树」等同「副本」）成立，且读数独立可核：

- `task-server/task_server_machines.py:110`「stem = "tree-" + hashlib.sha256(tree.encode("utf-8")).hexdigest()[:8]」，树路径取 `task-server/task_server_lib_configuration.py:606` 说的「directory 解开符号链接之后，往上找到的第一层有 .git」。
- 本腿对复跑目录路径 `printf '%s' <复跑目录> | sha256sum | cut -c1-8` 得 `a01caf33`，对它下面的 `build-dir-clean-wording-governance-r1-tree` 得 `8da5fa5e`，对仓根得 `9f6ec4dd`；服务给的那一组叫 `tree-a01caf33-…`（`ls -d ~/.local/share/singlefs/task-server-builds/tree-a01caf33*` 现查在）。树 = `--cwd` 本身，不是仓根、也不是复跑目录里拷出的那棵树。这一条读数分辨得出「树是 `--cwd`」与「树是副本」两种说法（哈希只对得上前者）。

推翻第 1 条结论的现象：设了留现场的任务有结局之后，服务某一轮把它的现场删了（`:319` 那一行被改成不拦），或 `task clean` 之外有别的路径把它删掉。

## 第 2 条：攻方第 2 问（两份定义新写的命令）

- 主体「照字面交给服务跑得起」：本机成立。判决里 `bash -c` 那一形探针打出仓外路径、脚本形打 `unset`，两臂都量了，分辨得出（判决第一节「主 agent 复跑」一条）。第二台拿得到 `CARGO_TARGET_DIR` 是读代码的结论、没人在第二台跑过：`scripts/hosts/multi-host-run.sh:1453` 的 `command_text` 里 `${build_text}` 在 `env -C … ${wrapper_text}$(printf '%q ' "$@")` 之前，`build_text` 在 `:1419` 是 `CARGO_TARGET_DIR=…` 前缀，bash 子进程与它里面的 `bash -c` 都继承；写「推的」。
- 打中的那一条（`rerun.sh` 脚本形里取不到）：成立。探针脚本 `probe-rerun-shape.sh`（复跑目录里那一份，sha256 `1f09b6be65234e407b0e192c4d9361e524b9d68cd507e7cc8acb3141dccaec68`，本腿 `sha256sum` 现算）没有文件头声明、只在正文里写 `cargo build --help`，打 `cargo_target_dir=unset`；声明那一臂由 `task-server/task_server_lib.py` 文件头「入口声明」一段读出（没有第二个探针量过）。`${CARGO_TARGET_DIR:?}` 在没设时退几：本腿起了一个只含 `echo "${CARGO_TARGET_DIR:?}/release/x"` 的脚本，`env -u CARGO_TARGET_DIR bash <脚本>` 打「parameter null or not set」、退出码 1（量的，bash 5.2.21）；用 `bash -c` 一行跑同一句退出码是 127（也是非 0）。判决说「直接退 1」对脚本形成立。
- 现行 `.claude/agents/three-way-attack.md:46`「文件头的 `# task-server-compiles: yes` 留着」与 `scripts/agents/three-way-rerun-template.sh:2`「# task-server-compiles: yes」，对应判决改法 2 的第一支（改法写成「或改成 `bash -c` 那一形」的第二支没被采）。
- `<debug 或 release>` 漏 `--profile`、`--example`、`--target`：判决写「采（cargo 的输出目录规则，改法里写清，不另做实验）」，攻方腿报告自己写「cargo 的 profile 目录规则凭印象」。本机离线文档里读不到这条规则：`~/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/share/man/man1/cargo-build.1` 只写 `--release`、`--profile <name>`、`--target-dir`，没有目录名对应表。我知道的规则（`bench` 进 `release/`、自定义 profile 进以它的名字命名的目录、`--target` 多一层三元组目录、`--example` 在 `examples/`）也是外部知识，所以写「推不出」。现行 `.claude/agents/three-way-attack.md:35` 已按这条规则写了对应表，这一条的探针（`cargo build` 各形各编一个只有空 `main` 的二进制，再 `find` 看落在哪层）放在「要主 agent 跑的命令」。

推翻第 2 条的现象：探针的脚本形打出路径（前提倒），或 `find` 的结果与第 35 行那张表不符。

## 第 3 条：攻方第 3 问（新出路走不走得通）

- 主体「主工作树与仓副本照出路走得通」：成立（读代码）。`task-server/task_server_lib.py:263`「按「哪一棵树 × 哪一台」给：树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层」，`:283` 一行起的「树删了编译目录跟着删——组里那一行的树此刻不是目录了、那一个空着的，服务删它」。仓副本经 `scripts/hosts/minimal-copy.py:28`「在副本根 git init」带 `.git`，树是副本根，删副本即树不是目录。
- 「`--cwd` 往上没有 `.git` 的树出路没写」：成立，读数同第 1 条低一级（哈希现算，树 = `--cwd`）。现行 `.claude/hooks/lib_workspace_cargo_lock.py:230`「树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层」与 `.claude/hooks/completion-cleanup-guard.sh:28` 同一句已按这一句写全。

## 第 4 条：攻方第 4 问

- 滞后表删四行、搬迁表删十四行：成立。`git show HEAD:.claude/gate.d/comment-history-lag.tsv | grep -c '^\.claude/hooks/'` 输出 0，此刻同一条 `grep -c` 也是 0；十四行里我只核了三个连字符旧名（`write-guard-decisions`、`completion-cleanup-guard-decisions`、`bash-command-detector-selftest-entries-and-main`；十四行的全名单只在别的会话的目录里，这条腿没有），它们在 HEAD 与此刻的 `.claude/gate.d/path-moves.tsv` 里各 `grep -c` 输出 0（量的，三个；十四个整批只是读判决与攻方腿报告），`git ls-tree HEAD --name-only .claude/hooks/ | grep -c -E 'write-guard-decisions|completion-cleanup-guard-decisions'` 输出 0。我用较松的子串 `bash-command-detector-selftest` 在钩子目录搜，命中的是下划线名自证文件里 `tempfile.mkdtemp(prefix="bash-command-detector-selftest-…")` 的临时目录前缀与测试输入里的 `research/bash-command-detector-selftest-literal.md` 这类字符串，不是旧路径引用。
- 检出钩子四处 `history-kept`「理由不在三类」：成立。`.claude/agent-common.md:47`「特殊缘由只三类」，三类是日期本身是数据、引外部文档、绕开外部工具缺陷。判决接着写「照 `.claude/agent-common.md:79` 该修那一格的判法或登常设豁免」，`:79`「是机制或门禁判错的，修那道机制或门禁、或登进它的常设豁免表，不单次放宽、不只豁免这一次」成立；其中「或登常设豁免」那一支实际不可用：
  - `scripts/gate/comment-history-check.py:50`「整份文件不判」：豁免表 `.claude/gate.d/comment-history-exempt.tsv` 登的是整份文件，此刻四行都是主题就是这些词的文件（判法脚本、词表、两份整理历史的脚本）；`.claude/hooks/bash-command-detector.sh` 不是（`grep -c '^ *#'` 此刻 1372 行注释行），登进去就整份不判。
  - 滞后表那一支：`.claude/agent-common.md:47`「新写的不许往里加」。
  所以「修判法」是判决列的两支里能用的那一支。
- 派发闸 `.claude/hooks/runner_dispatch_guard_locks_and_verdicts.py:672` 那一处成立：「history-kept: 被判的标题写法示例，日期与重跑次数本身是数据」，那一行「「### 2026-10-20：…」」里的日期在「」里，新判法「括着的日期与别的词照判」（`scripts/gate/comment-history-check.py:46`），所以换上新判法之后这一行的标记仍要在，与判决「第一类，成立」一致。

## 第 5 条：正推第 5 问（依据写得比证据宽，不改文件）

成立。两处原文现查在：`.claude/rules/verification-reference.md:70`「手跑的 GPU 测试编与跑写成一条交，不先另交 `--no-run` 只编的那一条」（只说手跑的 GPU 测试），`task-server/README.md:172`「提交时警告（用户：「绕过的 那个要做成提醒 谁做错了严厉警告」）」（那一行判的是取仓根 `target/` 的警告）。`grep -n 'verification-reference\|task-server/README' .claude/agents/experiment-runner.md .claude/agents/three-way-attack.md .claude/skills/dispatch/SKILL.md .claude/skills/three-way/SKILL.md` 零命中：被判的定义与技能没有引这两处，只有正文转述里有，所以不改文件对。

为被判出局的一方（「编与跑写成一条」的依据写得宽）补一条它站得住的理由：本树里撑它的是 `task-server/task_server_lib.py:276`「两台的组互不相干」（编译目录按树按台各一组，编与跑拆成两条交时可能派到两台），与 `scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml:10` 的第二台理由；这一条两份定义没写成依据，也没写错。

## 第 6 条：正推第 6 问

成立（6.8 采）。`.claude/gate.d/break-switch-replay-files.tsv:10` 此刻起头「八十二条开关（文件头 COMPLETION_CLEANUP_BREAK= 现数」，`:70` 起头「八十二条开关（2026-11-21 完工清理闸删服务编译目录那一支」；两个文件头开关数此刻 `grep -cE` 都是 82：完工清理闸 `grep -cE '^# +COMPLETION_CLEANUP_BREAK=|^#   COMPLETION_CLEANUP_BREAK=' .claude/hooks/completion-cleanup-guard.sh` 输出 82，共用库 `grep -cE '^ +LIB_PROJECT_CODE_RUNS_BREAK=' .claude/hooks/lib_project_code_runs.py` 输出 82（量的，本腿跑的）。「开关 82 条、对上 82 条」是主 agent 复跑读数，本腿没复跑，写「推的，待主 agent 跑」。

6.1 至 6.7 里我抽核了三条（现读，原文在那一行）：`.claude/hooks/write-guard.sh:107`「被改那几份样本此刻的样子拷进 <草稿目录>/fixtures-base/」、`.claude/hooks/write_guard_selftest.py:602`「派发闸收、值里另带别的写法的号的那四条与两臂、单臂各一条」（`grep -rn '攻方第二轮' .claude/hooks/write_guard_selftest.py | wc -l` 输出 0）、`scripts/gate/lib_split_family_loaded.py:17`「点号前面那一段模块名加 .py（缩进里、函数里的也算）」（`grep -c '以前'` 输出 0）；6.2、6.5、6.6 在第 4 条一节有本腿的读数；6.1 本腿只读了此刻 `grep -c 'history-kept' .claude/hooks/bash-command-detector.sh` 输出 0（标记已拿掉），标记当时的理由与行内容读的是判决与两条腿的报告。

## 第 7 条：正推第 7 问

成立。六处此刻都已按服务现行说法改完（现读）：`task-server/task` 的 cancel、clean 说明（取消那一行「编译目录不删：它归这棵树的组，尝试停了就放回」）、`task-server/attempt_runner.py:27`「子进程退了，编译目录不删：它归这棵树的组」、`multi-host.env.example:114`「TASK_SERVER_BUILD_DIRECTORY    本机的编译目录放在它下面，按树分组」、`gpu-verification-server/control:392`「# 编法：编译目录归树、不归任务」。

被判文件里还剩一处同类话，两条第一轮腿与判决都没点名：`.claude/hooks/lib_workspace_cargo_lock.py:225`「编译目录用完就删，主工作区的 target/ 删了又长的那条路由这一道堵上」，接在它下面的 `:230` 写「编译目录归树、不归任务」。是不是假话推不出：这句引的是用户那条「所有脚本自行编译，完成后自行删除」的原则，服务给的组在树删了才删；这一句与新说法放在同一条拒绝说明里相邻，读者会读到两个删的时点。放在这里交主 agent 判，不替它采。判法脚本的词表不判字符串，这一行不会红。

我用较宽的查法（`git grep -n --untracked -E '编译目录.{0,60}(任务一结束|跑完就删|跑完.{0,4}删|用完.{0,6}删|由服务删|task clean|clean <号>)|(task clean|clean <号>).{0,60}编译目录|build=<目录>|末行 build='`，排除 `research/`、`records/`、`.claude/singlefs-ai-sop/`）得 85 行，逐行看过命中最多的几份：别的工具自己建、自己跑完删的编译目录（`scripts/mutation/`、`scripts/gate/`、`scripts/hosts/run-build-directory.sh`、`replay.sh`）不算；`prune-server/control:173` 是弄坏开关的错样本；`scripts/hosts/multi-host-run.sh:1525` 的用法错说明写「跑完删」，说的是 `--peer` 直接起时的那一趟，服务起的带 `--keep-build-directory`，没改成假话但读者容易当成服务的行为，记一条交主 agent，不采。85 行我没有逐行读完，只读了命中数最多的 12 份，写成「读过一部分」。

## 第 6 问：改法 1 与改法 4 同样打中所有替代方案吗

### 改法 1：只判「本会话已有结局、设了留现场、现场目录还在」

先把两个服务里的口径并排放（原文，行号现查）：

- 服务认「不到期」：`task-server/task_server_scheduler_building.py:317`「if not task.get("collected") and (task.get("session") or lib.broken("scene-sessionless-kept", self.environment)):」与 `:319`「if (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1" and not lib.broken("scene-keep-ignored", self.environment):」两种，各自返回 False。
- 闸文件头说它与服务同一个口径：`.claude/hooks/completion-cleanup-guard.sh:35`「服务判留不留同一个口径：task-server/task_server_scheduler_building.py 的 scene_due」，却只认 `:319` 那一种；`:37`「在排、在跑的不列；没设留现场的不列（服务到时自己删）。」。
- 闸的样本把这个差别钉成了「对」：`.claude/hooks/completion_cleanup_guard_selftest.py:432`「unkept = s.service_task(10, "finished", keep=False)」，样本任务的记录是 `:163`「record = {"id": task_id, "state": state, "session": owner_session, "environment": environment}」（没有 `collected`，有 `session`），断言 `:435`「不该列（服务在取走结局之后到时自己删）」。这个样本是「有会话、没取走、没设留现场」，按 `:317` 服务不到期；断言说的「取走之后到时自己删」对它不成立。

| 替代方案 | 判错的情形 | 判对的情形 | 证据 |
|---|---|---|---|
| 不判（含只改 `.claude/skills/dispatch/SKILL.md:18` 的说法，攻方腿报告改法的第二条） | 设了留现场、已取走的现场没人 clean，服务永不删：现存一个（修改日期 2026-11-12，已取走、设了 1、八天没动） | 主 agent 一向读完就 `task clean`、从不忘 | `:297`、`:319`；现场目录读数见第 1 条 |
| 判全部现场（闸的 `kept-scenes-keep-unchecked` 开关就是这一臂） | 已有结局、已取走、没设留现场的现场：服务取走后一小时自己删（`task-server/task_server_lib.py` 文件头「现场」一段），闸要主 agent 当场 `task clean`，下一轮还要读那份现场的读不到；同一张单第二次放行，代价是多拦一次 | 已有结局、没取走、没设留现场的现场：服务永不删（`:317`、`:301`），现存两个（修改日期 2026-11-04 与 2026-11-06） | 同上 |
| 改法 1（只判设了留现场的） | 没取走、没设留现场的现场（上面「判对」那一行）：闸不列、服务不删、`:301` 说的「完成了没人取」只靠服务推结局，推的对象是提交它的会话，会话没了就没人收 | 设了留现场的（现存一个）；已取走没设的不拦 | `:35`、`:37` 与样本 `:432` |
| 没列进判决的臂：设了留现场 或 没取走（和 `scene_due` 返回 False 的两个条件逐个对齐） | 没取走的是主 agent 下一轮要 `wait` 的（收工时手里没有后台任务与子 agent，这种情形少；样本钉「在跑在排的不列」） | 两类服务都不删的现场都列 | `:317`、`:319` |

读数的限度：现存的三个老现场里，最老两个所属的会话与本会话是不是同一个，我没对（本腿不知道本会话号，也不读服务库）；闸只判本会话，所以「判全部」「没取走」两臂在这两个目录上不一定会拦。这一点不影响上面的判错情形（它们由代码读出），只影响「现存两个」是不是闸能拦到的。

结论：改法 1 对「不判」成立（`:297`、`:319` 加现存一个设了留现场的现场）；对「判全部现场」推不出：判决没有写出为什么只认设了留现场，闸文件头写的理由（`:37`「服务到时自己删」）对没取走的那一类是错的，判决引的读数（7 个目录）恰好大多落在没取走的那一类；而「判全部」自己也有判错的情形（已取走、没设的，服务一小时后自己删）。判决选的是更窄的一个，没有选错，但选的理由没撑到「同样打中所有替代方案」。推翻这一条的现象：把闸的样本 `:432` 改成取走的任务后「不该列」断言仍成立，且现存没取走的现场都不属于本会话。

### 改法 4：修 `scripts/gate/comment-history-check.py` 的判法，「」里的三方腿角色名不当历史词

判法现状（现读）：`scripts/gate/comment-history-check.py:242`「ROLE_NAME_WORDS = ("攻方腿", "辩方腿", "正推腿")」；`:495`「def judged_text(piece, switches):」只把收尾了的「」里这三个词换成占位符；`:249`「"定案", *ROLE_NAME_WORDS, r"攻方 ?v\d", r"第[一二三四五六七八九十百\d]+轮", "判决第", r"第 ?\d+[a-z]? ?批",」里别的词与日期在「」里照判。同类先例（形式上的豁免做在判法里、不做在行上）：`.claude/agent-common.md:47`「路径与文件名里的日期是名字，照写」，判法里 `path-dates-flagged` 开关钉两边；`以前` 的「除以、可以、前缀」几种形态也是判法里写的例外（`scripts/gate/comment-history-check.py` 文件头「以前」一段）。

| 替代方案 | 判错的情形 | 证据 |
|---|---|---|
| 不改判法，留单行标记（改前的做法，检出钩子 4 处加上同一成因的另 3 处） | 标记放过整行所有词：`scripts/gate/comment-history-check.py:48`「那一行放过」；检出钩子那些开关表行一行里列六七个格名，之后有人往同一行补一句「此前……」「第二轮……」，标记把它一起放过，门禁不红；理由「自证格名里的三方腿角色名，不是来历」不在 `.claude/agent-common.md:47`「特殊缘由只三类」里 | 同一行的放过范围与判法的放过范围比：标记是整行，判法只是「」里的三个词 |
| 登常设豁免（`.claude/gate.d/comment-history-exempt.tsv`） | 整份不判：`scripts/gate/comment-history-check.py:50`「整份文件不判」；`.claude/hooks/bash-command-detector.sh` 的 1372 行注释里真有来历句时也放过；表里现有四行都是主题就是这些词的文件；滞后表不能加新行（`.claude/agent-common.md:47`「新写的不许往里加」） | `ls`、`grep -v '^#' .claude/gate.d/comment-history-exempt.tsv` 四行 |
| 判法里「」整段都不判（开关 `quoted-spans-unjudged` 就是这一臂） | 「」里的日期与别的历史词也放过：`scripts/gate/comment-history-check.py:170`「收尾了的「」里整段都不判：括着的日期与别的历史词也放过」；例如 `.claude/hooks/runner_dispatch_guard_locks_and_verdicts.py:672` 那一行「「### 2026-10-20：…」」里的日期是真要判的（它靠标记放过，理由是「日期本身是数据」） | 判决选的更窄；整段不判这一臂判错的是第 672 行这一类 |
| 改格名（把自证格名里的「正推腿」「攻方腿」「辩方腿」改成不在词表里的叫法，七行注释连同自证与它钉的串同改） | 格名要同时改自证样本、开关表七八处注释行、所有按格名匹配的钉串（推的：我没数全）；改出来的名字不叫现行的角色名，读的人对不上 `three-way-forward` 等定义；对「格名里带出处」的那一种反而是对的（见下） | 检出钩子里的 `.claude/hooks/bash-command-detector.sh:582`「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（正推腿乙 Z8；rsync-inplace-unseen 下红）」 |
| 改法 4（修判法） | 一句真在讲来历、只用角色名一个词的话藏进「」：`scripts/gate/comment-history-check.py:177`「当格名放过，靠 review」；上面检出钩子的「正推腿乙 Z8」就是「带出处的格名」，新判法放过它；「第二轮攻方腿」里的「第二轮」仍判红（`:249`），所以背景材料举的那个例子打不中，打得中的是不带别的历史词的「「攻方腿」打中的……」 | 样本与读法见「要主 agent 跑的命令」第一条 |

结论：改法 4 成立。判决选的这一支在放过的范围上是五条里最小的一个，判错的情形是判法文件头自己写明的（`:177`），它不被「同样打中」；打中它的是上表最后一行，别的方案或者放得更宽（单行标记、豁免表、整段不判），或者要改的东西更多（改格名）。没有同类先例就不写先例：「改格名」这一支在仓里没有先例，我找不到。

推翻这一条的现象：样本 `hole.sh`（`comment-probe-root/scripts/hole.sh`，sha256 `23bc1089fa8c5ef93c87d03bbb19453733d887ce22bf45345037a630c76599d2`）的第 2 行一行（只有「攻方腿」在「」里）在 `--list` 里被点名（那就是判法没放过）；或第 3 行「第二轮」没被点名（那就是判法放过得比文件头写的宽）。

另：同一类理由的标记在 `.claude/hooks/runner-dispatch-guard.sh` 里还有 20 处，理由都是「history-kept: 引的是自证用例标签原文，按原文匹配」（`grep -c 'history-kept' .claude/hooks/runner-dispatch-guard.sh` 输出 20，行号如 `:252`、`:255`、`:484`、`:486`），其中至少这四行（`:252`、`:255`、`:484`、`:486`）里的历史词只是角色名，新判法下不再需要标记（推的，待主 agent 用 `--list` 核）。第二轮正文「已知开着」一节说这四处同类留着不红，本腿不重复报；放在这里是因为判决对检出钩子四处按「理由不在三类里」判出局，对这 20 处同一句理由没有判。

## 判决没覆盖、本腿现查到的两件事（交主 agent 判，不替它采）

1. 旧命名的编译目录没人删。`ls -d ~/.local/share/singlefs/task-server-builds/task-*` 有 12 个以 `task-<库标签>-<任务号>-<次数>` 起名的目录（不是 `tree-…` 的），`du -sch` 合计 81G（量的，在 `~/.local/share/singlefs/task-server-builds` 里 `du -sch task-* | tail -1` 输出 81G；`~/.local/share/singlefs/task-server-builds` 整体 117G）。我逐个查了其中四个，所属任务都已 finished 且已取走，`task-server/task status <号> --json` 里 `build_directory_removed` 为 false。改前 `task clean <号>` 会连它们一起删（转述：正推腿报告第 7 问写旧一版 `task-server/task` 的 clean 说明是删这条任务留下的编译目录；本腿没读那一版的 `task`），此刻 `task_server_lib.py:286`「任务记下结局、task clean <号>、取消都不删组里的」，完工清理闸也不再列；它们不在 `tree-…` 的组里，由谁删我没读到（组表由服务的 `sweep_vanished_tree_pools` 只扫组里的行，这一条是推的）。判决说「删支本身对」对删支成立，但这批是删支加 `clean` 改成不删之后没人收的旧命名遗留；`.claude/agent-common.md`「里程碑 17 收尾之前不兼容任何旧东西」不允许为它写迁移脚本，可以由主 agent 一次性手清或交用户。直接 `rm` 服务给的编译目录项目代码闸会拒，出路在拒绝说明里。
2. 完工清理闸文件头 `.claude/hooks/completion-cleanup-guard.sh:35` 写「服务判留不留同一个口径」，只抄了服务两个不到期条件里的一个，上面第 6 问一节已写。

## 要主 agent 跑的命令

这条腿没写模型（没有 `rerun.sh`），下面三项是探针与一项只读复核；样本与探针放在草稿目录 `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-sonnet/`，每个文件的 sha256 在下面。探针两项都很小（没有依赖的空 `main`，`cargo build` 秒级，内存按最小一档，推的，没量）。

文件 sha256（`sha256sum` 现算）：

```
23bc1089fa8c5ef93c87d03bbb19453733d887ce22bf45345037a630c76599d2  comment-probe-root/scripts/hole.sh
c872305cfdb507f746f4b90b80c81cc882bf722dd9488a6071e483c444946502  profile-probe/Cargo.toml
```

1. 注释历史判法对「」里的角色名放过多宽（对应「改法 4」一节与第 4 条）。
   - 命令：`task-server/task submit -- python3 scripts/gate/comment-history-check.py --list /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-sonnet/comment-probe-root`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`task-server/task log <号>` 里每行形如「路径:行号:命中的词:原文」。推的预期：第 3 行（词「第二轮」）、第 4 行（日期 2026-11-03）、第 5 行（词「改成了」）被列出，第 2 行（只有「攻方腿」在「」里）不被列出；第 2 行被列出就是判法没放过，第 3 行没被列出就是放过得比文件头写的宽。

2. 输出目录那张对照（对应第 2 条的 `<debug 或 release>` 一句）。每个二进制名只由一种构建形态编出，`find` 的结果直接读出各形态落在哪一层。
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-sonnet/profile-probe -- bash -c 'cargo build --bin bin_dev && cargo build --release --bin bin_release_flag && cargo build --profile release --bin bin_profile_release && cargo build --profile bench --bin bin_profile_bench && cargo build --profile custom --bin bin_profile_custom && cargo build --example ex_one && cargo build --target x86_64-unknown-linux-gnu --bin bin_target_triple && find "${CARGO_TARGET_DIR:?}" -path "*/deps" -prune -o -type f -perm -u+x \( -name "bin_*" -o -name "ex_one" \) -print'`
   - 在哪起：`--cwd` 就是上面那个草稿子目录（往上没有 `.git`，树是它本身，服务给它一组编译目录）。
   - 看哪几行：`exit=0`，日志末尾 `find` 打的路径。推的预期：`…/debug/bin_dev`、`…/release/bin_release_flag`、`…/release/bin_profile_release`、`…/release/bin_profile_bench`、`…/custom/bin_profile_custom`、`…/debug/examples/ex_one`、`…/x86_64-unknown-linux-gnu/debug/bin_target_triple`；任何一个落在别处，`.claude/agents/three-way-attack.md:35` 那张表就是错的。
   - 跑完删草稿目录里的 `profile-probe`（`rm -r` 点名这一个具体目录），服务按树把它的编译目录组一并扫掉。

3. 现场现状（对应第 1 条与第 6 问改法 1 一节，只读，主 agent 可直接跑）。
   - 命令一：`ls -ld --time-style=+%F /home/fy5090/.local/share/singlefs/work/task-server-scenes/*`；命令二：对每个目录名里的任务号 `task-server/task status <号> --json`，读 `task` 里的 `environment`（有没有 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES`）、`collected`、`state`、`session`，与本会话号对。
   - 在哪起：仓根。
   - 看哪几行：本腿读到最老两个是没设留现场且 `collected` 为 false、修改日期 2026-11-12 的一个是设了 1 且已取走（量的）；本腿没有对「这些任务是不是本会话的」，这一项只有主 agent 有会话号。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，上面三项都没跑；判决要引的数（复跑读数「82 条、对上 82 条」、自证格数）没复跑，这里写的都标了量的或推的。
- 没复跑任何门禁、自证、开关复跑；`comment-history-check.py --list` 没跑，所以「检出钩子 0 行」「少报的行」没复核；只用 `grep -rnE` 粗扫了射程目录里「」里带角色名的行（不分注释与字符串，14 行：`.claude/hooks/bash-command-detector.sh` 7 行、`.claude/hooks/runner-dispatch-guard.sh` 4 行、`scripts/gate/comment-history-check.py` 3 行，输出在草稿目录 `quoted-role-lines.txt`），这个数是粗扫的上界、不是判法的少报数。
- 没读第二台上的任何东西；第二台拿得到 `CARGO_TARGET_DIR` 是读代码的结论。
- 没验 cargo 的 profile 目录规则（只在本机离线文档里找了，没有）；没读 `.claude/agents/experiment-runner.md` 第 29 行以外的改动。
- 没读禁读清单里的另一条腿报告；没建仓副本、没建编译目录、没起样本进程；草稿目录里是两个探针样本、`quoted-role-lines.txt`、`old-wording-grep.txt`、一份 `colon-question-probe.sh`（本腿量 `:?` 退出码用的），没有要删的副本或编译目录。
