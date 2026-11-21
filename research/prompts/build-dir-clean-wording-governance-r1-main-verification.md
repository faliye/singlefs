# build-dir-clean-wording-governance-r1：判决（2026-11-21）

正文 `research/prompts/_build-dir-clean-wording-governance-r1-body.md`，开工快照 `research/prompts/build-dir-clean-wording-governance-r1-snapshot/`。这是这一题的第一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/build-dir-clean-wording-governance-r1-opus-output.md` | 四问都「推不出（打中）」：删支之后设了留现场变量的任务现场收工时没人提；定义里的 `"${CARGO_TARGET_DIR:?}"` 搬进 `rerun.sh` 跑不起、`<debug 或 release>` 漏三种写法；新出路没写 `--cwd` 往上没有 `.git` 的树；检出钩子四处 `history-kept` 理由在「特殊缘由只三类」之外 |
| 正推（Sonnet） | `research/prompts/build-dir-clean-wording-governance-r1-sonnet-output.md` | 第 5 问成立（两份定义依据写得比证据宽）；第 6 问 6.1–6.7 成立、6.8 登记表两行备注对不上；第 7 问被判文件之外还剩 6 处旧说法 |

核查：

- 倒推开工快照的树：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-main-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py build-dir-clean-wording-governance-r1 --out /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-main-rerun/build-dir-clean-wording-governance-r1-tree --repository /home/fy5090/code/singlefs`，结局 type=succeeded exit=0；28 份都是「工作区此刻那一份 sha256 与快照相同」，快照之后这几份没被改过。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/build-dir-clean-wording-governance-r1-opus-output.md research/prompts/build-dir-clean-wording-governance-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-main-rerun/build-dir-clean-wording-governance-r1-tree --background research/prompts/_build-dir-clean-wording-governance-r1-body.md`，末行「✓ 核了 87 处引文、81 处裸行号，对上 43 处，没判 125 处」，没有判红的；没判的里，判决要用的几处主 agent 逐处现读：`task-server/task_server_lib.py:281`、`:286`、`:287`、`:297`，`.claude/agent-common.md:47`（「特殊缘由只三类：日期本身是数据…、引外部文档或源码写读到的那一天、绕开外部工具缺陷写从哪个版本起与修好后删哪一段」）、`:79`（「是机制或门禁判错的，修那道机制或门禁、或登进它的常设豁免表，不单次放宽」），`.claude/gate.d/break-switch-replay-files.tsv:10`、`:70`，`.claude/agents/three-way-attack.md:46`–`:47`（第 8、9 步 `rerun.sh`），正推腿第 7 问点的六处原文，都与腿引的相同。
- 主 agent 复跑（腿的输入拷进复跑目录、sha256 与腿报告相同之后在副本里交）：喂项目代码闸与范围声明闸各三条（experiment-runner、three-way-attack 两份定义新写的命令与 `prune-server/control` 的新提示），六条都 type=succeeded exit=0、日志里 `✗` 0 行、`permissionDecision` 0 行，两道闸都放行；前提探针两条：`task-server/task submit --cwd <复跑目录> -- bash <复跑目录>/probe-rerun-shape.sh` 打「name=rerun_shape_probe shape=script-body cargo_target_dir=unset」，`task-server/task submit --cwd <复跑目录> -- bash -c 'cargo build --help > /dev/null && echo "name=rerun_shape_probe shape=bash-c cargo_target_dir=${CARGO_TARGET_DIR-unset}"'` 打「name=rerun_shape_probe shape=bash-c cargo_target_dir=/home/fy5090/.local/share/singlefs/task-server-builds/tree-a01caf33-b6cca7/target」。
- 现数：`~/.local/share/singlefs/work/task-server-scenes/` 下 7 个现场目录，修改日期最早 2026-11-04（攻方数的是 13 个，此刻少的是已到期被扫掉的）。`python3 scripts/agents/who-touched.py task-server/task task-server/attempt_runner.py gpu-verification-server/control multi-host.env.example --since-minutes 1440` 这几份一天内没有会话改过。

## 二、判决

1. 攻方第 1 问（完工清理闸删支之后漏拦）：坐实而采。`task-server/task_server_lib.py:297` 原文「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删」，删掉的那一支是收工时唯一叫主 agent 起 `task clean <号>` 的地方；这类现场此刻还有长期留着的（最早 2026-11-04）。删支本身对（编译目录 task clean 删不掉，留着那一支只会永远拦同一批），缺的是另判「本会话已有结局、设了留现场、现场目录还在」的任务。同一问的低一级打中（新文件头把「树」等同「副本」，`--cwd` 往上没有 `.git` 时树是 `--cwd` 本身）坐实：探针那一条的 `--cwd` 是不带 `.git` 的复跑目录，服务给了一组 `tree-a01caf33-…`。
2. 攻方第 2 问（两份定义新写的命令）：主体不成立（照字面交给服务跑得起：两道闸都放行、`bash -c` 那一形拿得到 `CARGO_TARGET_DIR`）；打中的那一条坐实而采：`.claude/agents/three-way-attack.md:35` 的取法搬进第 8、9 步要的 `rerun.sh`（整份以 `bash` 起交）时，探针坐实服务不给 `CARGO_TARGET_DIR`，`:?` 直接退 1。`<debug 或 release>` 漏写 `--profile`、`--example`、`--target <三元组>` 三种目录形态，采（cargo 的输出目录规则，改法里写清，不另做实验）。
3. 攻方第 3 问（新出路走不走得通）：主体不成立（主工作树与仓副本照出路走得通）；`--cwd` 往上没有 `.git` 的树出路没写，采，与第 1 条低一级那一处同一件，一起改措辞。
4. 攻方第 4 问（乙组标记与删行）：滞后表删四行、搬迁表删十四行成立（基准里没有这些行，旧连字符名仓里零引用）。检出钩子四处 `history-kept` 坐实而采：理由「自证格名里的三方腿角色名，不是来历」不在 `.claude/agent-common.md:47` 的三类里；这是门禁词表把现行角色名「正推腿」当历史词，照 `.claude/agent-common.md:79` 该修那一格的判法或登常设豁免，不单行放过。派发闸那一处（被判的标题写法示例、日期与重跑次数本身是数据）在第一类里，成立。
5. 正推第 5 问：成立。它报的「依据写得比证据宽」只在正文的转述里（`.claude/rules/verification-reference.md:70` 只说手跑的 GPU 测试、`task-server/README.md:172` 只撑不取仓根 `target/`），被判的定义里没有引这两处，不改文件。
6. 正推第 6 问：6.1–6.7 成立；6.8 坐实而采：`.claude/gate.d/break-switch-replay-files.tsv:10` 写「换上之后还没复跑」而这一版已复跑「对上 76 条」，`:70` 写七十九条而这一版复跑「开关 82 条、对上 82 条」。
7. 正推第 7 问：坐实而采，被判文件之外还剩 6 处旧说法：`task-server/task:65`–`:66`、`:68`–`:69`、`:231`，`task-server/attempt_runner.py:27`，`multi-host.env.example:114`，`gpu-verification-server/control:392`。

## 三、改法

1. 完工清理闸另判本会话已有结局、设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`、现场目录还在的任务，删法 `task-server/task clean <号>`，配会红的样本与弄坏开关；文件头写明判什么，与 `task-server/task_server_lib.py` 文件头「现场」一段对上；`.claude/skills/dispatch/SKILL.md:18` 那一句跟着说清这道闸拦哪几样。派工具员。
2. `.claude/agents/three-way-attack.md` 第 3b、8、9 步：交给服务时命令位置上是脚本的（`rerun.sh` 与外面包的那一层），照 `task-server/task_server_lib.py` 文件头「入口声明」那一段（`# task-server-compiles: yes`）让服务给编译目录，或改成 `bash -c` 那一形；二进制路径写清 `--release`、`--profile <名>`、`--target <三元组>`、`--example` 各对应哪一层目录。`.claude/agents/experiment-runner.md` 同一写法照改。派工具员。
3. 完工清理闸、项目代码闸、共用库、`lib_workspace_cargo_lock.py`、`prune-server/control`、两份技能里「树删了（副本删了）」的说法写全「树」：`--cwd` 往上第一个 `.git` 所在的那一层，没有就是 `--cwd` 本身，照 `task-server/task_server_lib.py` 文件头「编译目录」认树的那一句原文写。派工具员。
4. 检出钩子四处 `history-kept`：照 `.claude/agent-common.md:79` 修 `scripts/gate/comment-history-check.py` 的判法（「」里的自证格名、角色名不当历史词），或登常设豁免；拿掉那四处单行标记。派工具员（判法改动配样本与弄坏开关）。
5. `.claude/gate.d/break-switch-replay-files.tsv:10`、`:70` 备注改成现数与现状：主 agent 自己改。
6. 正推第 7 问六处：照服务现行说法改，`task-server/task`、`task-server/attempt_runner.py` 照服务的换法、`gpu-verification-server/control` 照脚本规格换，`multi-host.env.example` 定点改。派工具员。

这一轮有打中、坐实的，改完开第二轮：攻这一轮的修补，加一条辩方腿复核这份判决。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/hooks/completion-cleanup-guard.sh：判了删掉服务编译目录那一支之后漏拦设了留现场的任务现场（坐实，改法 1）、文件头把树等同副本（坐实，改法 3）
- .claude/hooks/completion_cleanup_guard_decisions.py：判了删支之后的判法与收工单子，同改法 1
- .claude/hooks/completion_cleanup_guard_selftest.py：判了删掉的四格自证与余下 110 格，改法 1 要补的样本落在这里
- .claude/hooks/project_code_guard_decisions.py：判了拒绝说明与 `TASK_SERVER_BYPASS_POLICY` 的新出路照做走得通（喂三条都放行），树的说法同改法 3
- .claude/hooks/project_code_guard_selftest.py：判了钉串改成「经 CARGO_TARGET_DIR 交给命令」与删掉的那个只赋值的变量，成立
- .claude/hooks/lib_project_code_runs_base.py：判了三处出路与注释，成立；树的说法同改法 3
- .claude/hooks/project-code-guard.sh：判了第 212 行出路、三条共用库开关补的点名与「（实测）」，成立；树的说法同改法 3
- .claude/hooks/lib_workspace_cargo_lock.py：判了拒绝说明与文件头的新说法，成立；树的说法同改法 3
- .claude/skills/dispatch/SKILL.md：判了第 11 条与第 56 行与服务现行说法一致，成立；第 18 行完工清理闸拦哪几样同改法 1
- .claude/skills/three-way/SKILL.md：判了第 49 行编译目录不归主 agent 删，成立
- .claude/agents/experiment-runner.md：判了第 29、44 行编与跑写成一条、二进制取 `CARGO_TARGET_DIR`，跑得起；目录形态同改法 2
- .claude/agents/three-way-attack.md：判了第 35 行的取法搬进第 8、9 步 `rerun.sh` 跑不起（坐实，改法 2）
- scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml：判了第 9、10 行注释与服务现行说法一致，成立
- prune-server/control：判了编法提示的理由与说明、自证格加的断言，成立；树的说法同改法 3
- prune-server/README.md：判了「装」第 1、4 条与服务现行说法一致，成立
- .claude/kb/tooling.md：判了第 42 行 clean 那一句与服务现行说法一致，成立
- .claude/kb/tooling.en.md：判了第 42 行同一句的英译与中文一致，成立
- .claude/kb/tooling.ja.md：判了第 42 行同一句的日译与中文一致，成立
- .claude/gate.d/break-switch-replay-files.tsv：判了第 10、70 行备注与复跑结局对不上（坐实，改法 5）
- .claude/hooks/bash-command-detector.sh：判了四处 `history-kept` 的理由不在三类特殊缘由里（坐实，改法 4）
- .claude/hooks/runner_dispatch_guard_locks_and_verdicts.py：判了第 672 行 `history-kept` 落在「日期本身是数据」一类，成立
- .claude/hooks/write-guard.sh：判了第 107 行改写成现状，成立
- .claude/hooks/write_guard_selftest.py：判了第 602 行删掉来历，成立
- .claude/gate.d/comment-history-lag.tsv：判了删的四行基准里没有、改净后删，成立
- .claude/gate.d/path-moves.tsv：判了删的十四行旧前缀在基准里没有文件，成立
- scripts/gate/lib_split_family_loaded.py：判了第 17 行措辞改写，成立
- scripts/gate/lib_lag_split_family.py：判了文件头例子挪到 Python 拆法那一行，成立
- .claude/hooks/selftest-registry.tsv：判了 make-swap-spec.py 那一行备注改成现状，成立

## 回看决策

不涉及决策：这一轮判的是钩子、技能、定义、判定库服务控制命令、kb 工具说明与几张登记表的改动，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
