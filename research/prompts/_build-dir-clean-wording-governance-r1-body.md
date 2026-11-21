# 编译目录归树之后的旧说法与几处注释、登记表的收口：治理文件轮第一轮（build-dir-clean-wording-governance-r1，2026-11-20）

这一轮攻的是两组已换上主工作区的治理文件改动。甲组：任务服务把编译目录改成按树给、`task clean` 不删之后，钩子、技能、agent 定义、装置注释、判定库服务的控制命令与 README、kb 里还写着「编译目录用完 `task clean <号>` 删」「`task wait` 末行 `build=`」的地方，照服务现行说法改掉；完工清理闸照旧做法判的那一支整支删掉。乙组：同一段里为几道门禁格的红改的注释与登记表（四份钩子 7 处注释改成现状或加 `history-kept` 标记、注释历史滞后表删四行与改两行路径、搬迁登记表删 14 行、认族共用库文件头一处措辞、两张登记表的备注）。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码；与 `crates/` 有交点的只有 `scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml` 的注释（编这份装置的命令怎么写）与两份 agent 定义里编 `crates/` 下实验包装置、调枚举函数原型的命令写法。

## 问题

1. 甲组每一处改法说的是不是服务今天真做的事；完工清理闸删掉那一支之后，有没有一样本该拦的东西不再被拦（攻方腿）。
2. 两份定义里新写的「编和跑写进同一条 submit、二进制取那一条里的 `CARGO_TARGET_DIR`」交给服务之后，跑不跑得起、派到第二台对不对、过不过得了项目代码闸与范围声明闸（攻方腿）。
3. 乙组每一处：加的 `history-kept` 标记理由站不站得住（是不是把真在讲来历的句子豁免掉了）；滞后表、搬迁登记表删掉的行是不是该删、删了有没有漏判（攻方腿）。
4. 甲组、乙组每一处改动逐条核：今天的文件是不是「实现今天的样子」写的那样，依据引的服务原文是不是在那一行、说的是不是那件事（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 535–539 行）：

- 服务现行说法：`task-server/task_server_lib.py:281`「命令拿到的是 CARGO_TARGET_DIR（组里那一个的 target）：入口照它编、不删」、`:286`「任务记下结局、task clean <号>、取消都不删组里的：clean 只删那一条任务自己的现场（下面「现场」）与分片父任务的测试二进制；取消的照旧删它自己的现场，编译目录放回组里。」、`:287`「wait 末行不带 build=（编译目录归树、不归任务）。」；`task-server/README.md:224`「「服务build之后必须删除历史」「自己用完自己删」照树算——树删了（副本删了）它的整组跟着删。…任务记下结局、`task clean`、取消都不删组里的编译目录。」。
- 完工清理闸：「服务编译目录」那一支整支删掉（读服务库、`KIND_SERVICE_BUILD`、列单子要 task clean 的那几段），五条开关（`service-builds-ignored`、`service-build-removal-by-rm`、`service-builds-session-unchecked`、`service-builds-unfinished-listed`、`subagent-service-builds-judged`）与四格自证跟着删，开关 81 改 76；文件头新写一段「服务给的编译目录…谁都不判：它归树、不归任务」。工具员判「不改成判任务现场」的依据：`task_server_lib.py:296`（wait 取走结局满一小时后服务自己删现场）、`:297`（设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1` 的是有意留的）。换上后自证「各格都对（判了 110 格）；76 个弄坏开关各红在点名的格」，开关复跑 76 条逐格相同。
- 项目代码闸与共用库：直接 rm 服务编译目录照旧拒，拒绝说明与 `TASK_SERVER_BYPASS_POLICY` 的出路改成「编译目录归树、不归任务，不用自己删：树删了（自己建的仓副本删了）服务跟着删它那一整组」；`lib_workspace_cargo_lock.py` 的拒绝说明改成「服务按这棵树给一个仓外的编译目录、经 CARGO_TARGET_DIR 交给命令；…不用自己删、不直接 rm」，两份自证钉的串改钉「经 CARGO_TARGET_DIR 交给命令」；`project-code-guard.sh:212` 出路同改。项目代码闸文件头三条共用库开关（`temporary-service-mark-entry-only`、`-family-by-directory`、`-family-by-name`）照实测补了 5 格点名，`refusal-submit-dropped` 的「（推的）」改「（实测）」。换上后自证「查了 1492 种，其中该拒 332 种」、共用库「查了 467 种」、`lib_workspace_cargo_lock.py`「查了 32 格」；`WORKSPACE_CARGO_LOCK_BREAK=refusal-submit-dropped` 只红「编译目录：拒绝说明给出经 task-server 提交的出路」一格；开关复跑项目代码闸 155 条、共用库 82 条逐格相同。
- 技能与定义：`.claude/skills/dispatch/SKILL.md` 第 11 条与第 56 行、`.claude/skills/three-way/SKILL.md:49` 改成编译目录不归主 agent 删、`task clean` 删的是任务现场；`.claude/agents/experiment-runner.md:29`、`:44` 与 `.claude/agents/three-way-attack.md:35` 不再叫去 `build=` 目录取二进制，改成「编与跑写成一条交：`task-server/task submit --cwd <副本> -- bash -c 'cargo build <参数…> && "${CARGO_TARGET_DIR:?}/<debug 或 release>/<bin 名>" <参数…>'`」（依据 `task_server_lib.py:281`、`:287`、`.claude/rules/verification-reference.md:70`、`task-server/README.md:172`，同一写法 `prune-server/README.md:28`）。rules-lint 对这三份退 0。
- 判定库服务：`prune-server/control` 编法提示的做法（编与拷到共享根写在同一条命令里）不变，理由与说明改成编译目录归树、`task wait` 末行不点名它；自证格 `install-build-hint-saves-binary` 加一条断言；换上后自证退 0，`PRUNE_SERVER_CONTROL_BREAK=build-hint-leaves-build-directory` 只红那一格。`prune-server/README.md`「装」第 1 条理由与第 4 条（原「用完 `task-server/task clean <号>` 删编译目录」）主 agent 定点改了。
- kb：`.claude/kb/tooling.md` 第 42 行三语三份 `clean <号>` 那一句改成删任务现场、编译目录归树，原句「第二台那一趟的编译目录由 `multi-host-run.sh` 跑完自己删」一起改掉；三语六格退 0。同一行后半句的领卡写法与现行借卡白名单对不上，不在这一轮改，登在 `records/2026-11-19-未做完的任务登记.md` 第 35 行。
- 乙组：`bash-command-detector.sh` 开关表四行行尾加「history-kept: 自证格名里的三方腿角色名，不是来历」、`runner_dispatch_guard_locks_and_verdicts.py:672` 加「history-kept: 被判的标题写法示例，日期与重跑次数本身是数据」、`write-guard.sh:107`「改前的样本」改「被改那几份样本此刻的样子」、`write_guard_selftest.py:602` 删「攻方第二轮 3f」；`.claude/gate.d/comment-history-lag.tsv` 删这四份那四行（基准里没有、算涨）；`.claude/gate.d/path-moves.tsv` 删 746–759 行（旧前缀在基准里没有文件）；`scripts/gate/lib_split_family_loaded.py:17`「点号以前」改「点号前面那一段」、`scripts/gate/lib_lag_split_family.py` 文件头一处例子挪行；两张登记表（`break-switch-replay-files.tsv`、`selftest-registry.tsv`）的备注改现数与现状。检出钩子开关复跑 172 条逐格相同，comment-history、change-range-single-source 两格单跑时这几处都已不在点名里。
- 改前原样：各草稿目录的 `base/`（`~/.local/share/singlefs/work/build-dir-clean-wording/base/`、`build-dir-clean-wording-rest/base/`、`build-dir-clean-wording-prune/base/`、`build-dir-clean-wording/pcg-header/base/`、`build-dir-clean-wording-rest/pcg-measured/base/`、`comment-history-hooks/base/`）；不经草稿的（kb 三份、`prune-server/README.md`、几张登记表）对着 `git show HEAD:<路径>` 与 `git diff HEAD -- <路径>` 看。
- 已知开着、各有会话在修、这一轮不重复报的：门禁 doc-text comment-history 格还红在算键库两处命中（mainagent(A) 改算键库）与 `scripts/gate/large-file-git-hook.py` 一行滞后行（mainagent(B) 写的脚本）；`.claude/agent-common.md:37` 同一旧说法由 mainagent(C) 并进它那一批改。

## 范围

这一轮被判的文件（28 份，逐个列在 `research/prompts/build-dir-clean-wording-governance-r1-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 完工清理闸删掉「服务编译目录」那一支之后：主 agent 收工时还留着、本该被拦下的东西（任务现场、设了留现场变量的现场、合入后验证的快照与槽、副本）有没有一样因此漏拦；文件头新写那一段与 `task_server_lib.py` 文件头「现场」「编译目录」两段对不对得上。
2. 两份定义新写的命令：`bash -c` 里的 `cargo build … && "${CARGO_TARGET_DIR:?}/…"` 交给服务，服务认不认得它会编、给不给编译目录（`task_server_lib.py:282` 起那一段）；派到第二台时 `CARGO_TARGET_DIR` 交没交到命令手里；项目代码闸、范围声明闸、派发闸会不会拒它；`<debug 或 release>` 与 cargo 的 profile 目录名对不对。
3. 项目代码闸、共用库、`lib_workspace_cargo_lock.py`、`prune-server/control` 新出路里点名的做法，照做之后走不走得通、过不过得了别的闸（`.claude/singlefs-ai-sop/rules/sop-first.md`「出路本身要过得了别的闸」）。
4. 乙组：加 `history-kept` 的五处，有没有一处其实是在讲来历；滞后表删的四行、搬迁登记表删的 14 行，删掉之后那几份文件与那几条搬迁还有没有被判到。

正推腿（逐条回「成立」「不成立」「推不出」）：

5. 「实现今天的样子」甲组每一条：今天的文件是不是这样写的（写出 文件:行 与那一句原文），引的服务原文是不是在那一行、说的是不是这件事。
6. 「实现今天的样子」乙组每一条，同第 5 问的核法。
7. 全仓还有没有「编译目录用完 `task clean` 删」「`task wait` 末行 `build=`」的旧说法没改（`research/prompts/`、`records/`、变更史与已知开着那几处除外），给出 文件:行。

## 分工

攻方腿做第 1 到 4 问，正推腿做第 5 到 7 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
