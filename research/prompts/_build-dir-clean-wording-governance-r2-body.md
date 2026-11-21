# 编译目录归树之后的旧说法收口：治理文件轮第二轮，攻第一轮判决的修补（build-dir-clean-wording-governance-r2，2026-11-21）

这一轮攻的是第一轮判决（`research/prompts/build-dir-clean-wording-governance-r1-main-verification.md`）「三、改法」六条已换上主工作区的修补，并派辩方腿复核这份判决。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码；与 `crates/` 有交点的只有两份 agent 定义与复跑模板里编 `crates/` 下装置、调枚举函数原型时二进制取哪个目录的写法。第一轮攻过而没打中的角度不重复：两份定义新写的 `bash -c` 那一形交给服务跑不跑得起、过不过得了项目代码闸与范围声明闸（主 agent 喂六条都放行）、滞后表与搬迁表删行该不该删、派发闸那一处 `history-kept` 的类别。

## 问题

1. 每一处修补做的是不是判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 第一轮判决够不够得着：判成坐实而采的几条，依据撑不撑得住；判成不成立的几条（攻方第 2、3 问的主体、正推第 5 问「依据写得比证据宽」不改文件），判得对不对，替被判出局的一方辩护（辩方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 542–544 行）：

- 改法 1：完工清理闸另判「本会话（主 agent）已有结局、提交时设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`、现场目录还在」的任务，种类「任务现场」，出路 `task-server/task clean <号>`；只读打开服务库按会话取任务与尝试（照删掉那一支的读法）；子 agent 不判；分片按父任务算。六条新开关（`kept-scenes-ignored`、`kept-scene-removal-by-rm`、`kept-scenes-keep-unchecked`、`kept-scenes-session-unchecked`、`kept-scenes-unfinished-listed`、`subagent-kept-scenes-judged`）、五格新自证。`.claude/skills/dispatch/SKILL.md:18` 改成逐样列出闸拦哪几样。换上后自证「各格都对（判了 115 格）；82 个弄坏开关各红在点名的格」，开关复跑 82 条逐格相同。
- 改法 2：`.claude/agents/three-way-attack.md` 第 3b、8、9 步与 `.claude/agents/experiment-runner.md:29`、`:44` 写清两种交法：`bash -c` 那一形照旧；脚本那一形靠文件头 `# task-server-compiles: yes`（`task-server/task_server_lib.py:278`），第 9 步外层脚本也写这一行、交法先 `mkdir -p <运行目录>` 再 `task submit --cwd <运行目录>`；输出目录写清 `--release`、`--profile`、`--target`、`--example` 各对应哪一层。复跑模板 `scripts/agents/three-way-rerun-template.sh` 第 2 行加会编声明，一格新自证、新开关 `compiles-declaration-dropped`；那条开关起先往 `$copy` 另写一遍、让算键库折不出 `bash "$copy"`（开关复跑开跑前拒算 4 条），改成只改文件头摘录之后，新版算键退 0、旧版退 4，自证 7 格过、开关只红点名那一格、开关复跑 3 条逐格相同、拒算 0。rules-lint 对两份定义与技能退 0。
- 改法 3：完工清理闸文件头、项目代码闸、共用库、`lib_workspace_cargo_lock.py`、`prune-server/control`、dispatch 技能里「树」照 `task-server/task_server_lib.py:263`「树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层（build_tree_root；哪一层都没有就是解开后的 --cwd 本身）」写全，出路另补「`--cwd` 往上没有 `.git` 的，删那个 `--cwd` 目录」；两份自证与「绕开：读签名密钥」那一格钉了新句。主 agent 另把 kb `tooling` 三语、`task-server/README` 三语第 224 行与 `task-server/task_server_constants.py:289` 的「树删了（副本删了）」照同一句写全。三份钩子开关复跑「复跑了 319 条开关（3 份）…逐格相同 319 条…拒算 0」。
- 改法 4：注释历史判法 `scripts/gate/comment-history-check.py` 改成成对「」里的攻方腿、辩方腿、正推腿不算历史词（套着的「」按层配对，括着的日期与别的历史词照判，没收尾的「照判），四条新开关；检出钩子拿掉七处 `history-kept`（判决点名的四处加同一成因的 `:582`、`:726`、`:1133`）。判法开关复跑 31 条逐格相同；`comment-history-check.py --list` 检出钩子 0 行；`.claude/gate.d/doc-text.sh:698`「另五条」改「另十七条」。
- 改法 5：`.claude/gate.d/break-switch-replay-files.tsv` 第 10、27、33、70 行条数与结局照复跑改。
- 改法 6：`task-server/task` cancel、clean 的说明与输出、`:231`，`task-server/attempt_runner.py:27`、`:36`，`multi-host.env.example:114`，`gpu-verification-server/control:392` 照 `task_server_lib.py:263`、`:286` 改成现状。换上时 `task_server_lib.py` 自证点名三格退 0；两条开关单格证红各只红点名那一格（`clean-failure-explained`、`clean-keeps-build-pool`）；GPU 控制命令自证退 0。`.claude/gate.d/comment-history-lag.tsv` 里 `prune-server/control` 照现数改小到 1。
- 改前原样：各草稿目录的 `base/`（`~/.local/share/singlefs/work/build-dir-governance-r1-fix-a/base/`、`build-dir-governance-r1-fix-b/base/`、`build-dir-governance-r1-fix-template/base/`、`build-dir-governance-r1-fix-a/constants-tree/base/`、`build-dir-governance-r1-fix-b/doc-text-count/base/`），第一轮开工快照 `research/prompts/build-dir-clean-wording-governance-r1-snapshot/judged-sha256.txt`；不经草稿的对着 `git diff HEAD -- <路径>` 看。
- 已知开着、各有会话在修、这一轮不重复报的：门禁 doc-text comment-history 格还红在算键库两处命中（mainagent(A) 改算键库）与 `scripts/gate/large-file-git-hook.py` 一行滞后行（mainagent(B) 写的脚本）；`.claude/agent-common.md:37` 同一旧说法由 mainagent(C) 改；kb `tooling.md` 第 42 行后半句的领卡写法登在 `records/2026-11-19-未做完的任务登记.md` 第 35 行；派发闸 `.claude/hooks/runner-dispatch-guard.sh` 四处同类 `history-kept`（判法换上后用不上、留着不红）与检出钩子引的一个带出处的格名（`bash_command_detector_selftest_waits_and_scope.py:1901`），这一轮记了去向、没改。

## 范围

这一轮被判的文件（30 份，逐个列在 `research/prompts/build-dir-clean-wording-governance-r2-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。第一轮判决与它引的主 agent 复跑读数也在辩方腿的射程里。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 完工清理闸新判的任务现场：会话认得对不对（主 agent 换过会话号、分片子任务、人手 `--manual` 交的）；「设了留现场」认法与服务认法是不是同一份；现场目录在第二台的、取不到结局的、服务库读不了的各怎么办；五格自证与六条开关是不是红在它们名字说的那件事。
2. 两份定义与复跑模板：外层脚本写了会编声明之后，服务给不给 `CARGO_TARGET_DIR`、给的是哪棵树的组；`mkdir -p` 再 `--cwd <运行目录>` 那一形交给服务、项目代码闸与范围声明闸认不认；输出目录那张对照对不对得上 cargo；模板开关改成只改文件头摘录之后，那一格还测不测得到「拷出的 rerun.sh 不带声明」这件事。
3. 注释历史判法：「」里不判角色名之后，有没有一种写法能把真在讲来历的句子藏进「」放过（例「「第二轮攻方腿」打中的」）；四条开关是不是红在点名的格；全仓因此少报的行是不是只有检出钩子与派发闸那几行。
4. 改法 3、6 写全的「树」与改过的几处出路、说明：照字面做走不走得通，与 `task-server/task_server_lib.py` 文件头「编译目录」「现场」两段逐句对不对得上；有没有一处改出了新的假话。

辩方腿（逐条回「成立」「不成立」「推不出」）：

5. 第一轮判决「二、判决」七条：判成坐实而采的，主 agent 引的复跑读数分不分辨得出（有无那一处两臂）；判成不成立、不改文件的几处，判得对不对，替被判出局的一方辩护。
6. 第一轮判决的改法 1 选「另判任务现场」、改法 4 选「修判法」而不选「登常设豁免」：是不是同样打中所有替代方案（不判、判全部现场、登豁免表、改格名），各在哪一种情形下判错。

## 分工

攻方腿做第 1 到 4 问，辩方腿做第 5、6 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
