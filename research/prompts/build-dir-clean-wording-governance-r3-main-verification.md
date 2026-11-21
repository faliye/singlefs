# build-dir-clean-wording-governance-r3：判决（2026-11-21）

正文 `research/prompts/_build-dir-clean-wording-governance-r3-body.md`，开工快照 `research/prompts/build-dir-clean-wording-governance-r3-snapshot/`。这是这一题的第三轮（最后一轮），攻第二轮判决 `research/prompts/build-dir-clean-wording-governance-r2-main-verification.md` 的修补；第一轮判决 `research/prompts/build-dir-clean-wording-governance-r1-main-verification.md`。这一轮之后不开第四轮，这一轮的改法都标零轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/build-dir-clean-wording-governance-r3-opus-output.md` | 第 1 问推不出（两样都占写 clean 照做只删现场、不取结局，之后每次交任务挨警告；自证没有「设了留现场、结局已取走」的样本），第 2、3 问成立（另报检出钩子一个格名带出处「正推腿乙 Z8」被放过），第 4 问不成立（`task-server/README` 三语第 224 行取消那一句漏改、`task_server_lib.py:670`–`:674` 写「推的，没复跑」、`experiment-runner.md:29` 新句与 `:41` 范例相反） |
| 正推（Sonnet） | `research/prompts/build-dir-clean-wording-governance-r3-sonnet-output.md` | 第 5 问成立，另读出 `task_server_lib.py:670`–`:674`、`.claude/gate.d/break-switch-replay-files.tsv:77` 条数、清扫「在跑尝试用着」那一层没有自证格与开关；第 6 问剩下 9 个旧命名目录属 5 棵树、都还在盘上，登没登在组里推的；第 7 问 `task-server/README` 三语第 224 行取消那一句，另顺带 `three-way-forward.md`、`three-way-defense.md` 写复跑外层脚本「先删掉运行目录」与攻方定义这一轮改的「先清空、目录本身留着」不一致 |

核查：

- 倒推开工快照的树：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-main-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py build-dir-clean-wording-governance-r3 --out /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-main-rerun/build-dir-clean-wording-governance-r3-tree --repository /home/fy5090/code/singlefs`，type=succeeded exit=0，`✗` 0 行。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/build-dir-clean-wording-governance-r3-opus-output.md research/prompts/build-dir-clean-wording-governance-r3-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-main-rerun/build-dir-clean-wording-governance-r3-tree --background research/prompts/_build-dir-clean-wording-governance-r3-body.md`，末行「✓ 核了 28 处引文、111 处裸行号，对上 22 处，没判 117 处」，没有判红的；判决要用的几处主 agent 现读：`task-server/task_server_scheduler_starting.py` 的 `clean`（只删现场与分片父任务的测试二进制，不记已取）、`task-server/task_server.py` 文件头「提交时警告」②（会话名下结局满 30 分钟没取走，每次交任务打严厉警告）、`.claude/hooks/completion_cleanup_guard_decisions.py:1342`、`.claude/agents/experiment-runner.md:41`（`--cwd scripts/experiment-apparatus -- bash ../mutation/mutate.sh …`）、`.claude/agents/three-way-defense.md:32`（「开头先删掉那一个运行目录」）。
- 主 agent 复跑（腿的副本与探针拷进 `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-main-rerun/` 之后在副本里交）：
  - 攻方变异（第 1342 行改成只看取没取走）下完工清理闸自证：自证本身各格照绿，退 1 只因一条无关开关 `hand-way-example-oversized` 点名的格在副本里本次未跑；变异活着。
  - 注释历史判法 `--list` 探针列出第 2、4、6、7 行，与攻方预期相同；第 5 行（检出钩子引的格名「…（正推腿乙 Z8；…）」）带冒号被放过。

## 二、判决

1. 攻方第 1 问：坐实而采两处。甲：「两样都占写 clean」「没取走的也可以 clean」照做，结局仍算没取，服务之后每次收这个会话的任务都打「有结局没人取」的严厉警告；出路要先取走结局。乙：自证缺「设了留现场、结局已取走」的样本，变异活着（主 agent 复跑坐实）。会话认法与四条开关成立。
2. 攻方第 2 问：成立；带冒号照样能藏来历这一点判法文件头「管不到的」已写明。另报的检出钩子格名带出处「正推腿乙 Z8」坐实而采（格名本身写了来历，判法照规矩放过它）。
3. 攻方第 3 问：成立。正推腿另报清扫「在跑尝试用着」那一层没有自证格与开关，坐实而采。
4. 攻方第 4 问与正推第 7 问：坐实而采：`task-server/README` 三语第 224 行「取消都不删组里的编译目录」漏改，同一行主 agent 补的「旧命名」是历史写法；`task-server/task_server_lib.py:670`、`:672`、`:674`「（新开关，推的，没复跑）」与复跑结局对不上；`.claude/agents/experiment-runner.md:29` 新句与同一份 `:41` 范例相反；`.claude/gate.d/break-switch-replay-files.tsv:77` 条数 530 与现数 533 对不上；`three-way-forward.md`、`three-way-defense.md` 复跑外层脚本「先删掉运行目录」与攻方定义不一致。
5. 正推第 5、6 问：第 5 问成立。第 6 问：剩下 9 个旧命名目录分属 5 棵还在盘上的树，登没登在组里主 agent 读不了服务库、核不动；第二轮判决第 7 条「81G 没有一条路删」改正为：清扫删了 1 个不在组里的，余下的里至少 2 个近 6 小时仍在写、在用（主 agent `find` 现查），其余的去留跟着各自那棵树。

## 三、改法

这一轮的改法都标零轮（第三轮之后不再开轮），改的是治理文件，不进产品代码，不做 A/B。

1. 完工清理闸：结局没取走的出路写 `task-server/task wait <号>`；设了留现场、结局已取走的写 `task-server/task clean <号>`；两样都占的写先 `wait` 再 `clean`；自证补「设了留现场、结局已取走」样本与会红的开关。零轮。
2. 检出钩子格名「正推腿乙 Z8」改成只说判什么（`bash_command_detector_selftest_waits_and_scope.py` 与检出钩子文件头三处一起改）。零轮。
3. 任务服务清扫「在跑尝试用着」那一层补自证格与开关；`task_server_lib.py:670`–`:674` 改成实测；登记表第 77 行条数照现数改。与另派的「服务认出包着的写工作区工具」那一批同改 `task_server_lib.py`，先换上那一批再改。零轮。
4. `task-server/README` 三语第 224 行取消那一句照第 276 行写全、去掉「旧命名」；`experiment-runner.md:41` 范例改成与第 29 行一致（绝对路径）；`three-way-forward.md`、`three-way-defense.md` 复跑外层脚本那一句照攻方定义改成「先清空、目录本身留着」。零轮。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/agents/experiment-runner.md：判了第 29 行新句与第 41 行范例相反（坐实，改法 4）
- .claude/agents/three-way-attack.md：判了第 9 步绝对路径与先清空运行目录，成立；两份同类定义不一致同改法 4
- .claude/gate.d/break-switch-replay-files.tsv：判了第 77 行条数 530 对现数 533（坐实，改法 3）
- .claude/hooks/completion_cleanup_guard_decisions.py：判了接任务表的会话认法成立，两样都占写 clean 照做不取结局（坐实，改法 1）
- .claude/hooks/completion_cleanup_guard_selftest.py：判了缺「设了留现场、结局已取走」样本、变异活着（坐实，改法 1）
- .claude/hooks/completion-cleanup-guard.sh：判了文件头两条出路的说明，同改法 1
- .claude/hooks/lib_workspace_cargo_lock.py：判了这一轮的修补照字面与服务现行说法一致，成立
- .claude/hooks/project_code_guard_decisions.py：判了这一轮的修补照字面与服务现行说法一致，成立
- .claude/hooks/project_code_guard_selftest.py：判了这一轮的修补照字面与服务现行说法一致，成立
- .claude/hooks/project-code-guard.sh：判了这一轮的修补照字面与服务现行说法一致，成立
- .claude/kb/tooling.en.md：判了这一轮的修补照字面与服务现行说法一致，成立
- .claude/kb/tooling.ja.md：判了这一轮的修补照字面与服务现行说法一致，成立
- .claude/kb/tooling.md：判了这一轮的修补照字面与服务现行说法一致，成立
- .claude/skills/dispatch/SKILL.md：判了这一轮的修补照字面与服务现行说法一致，成立
- gpu-verification-server/control：判了这一轮的修补照字面与服务现行说法一致，成立
- prune-server/control：判了这一轮的修补照字面与服务现行说法一致，成立
- prune-server/README.md：判了这一轮的修补照字面与服务现行说法一致，成立
- research/prompts/build-dir-clean-wording-governance-r2-main-verification.md：判了这一轮的修补照字面与服务现行说法一致，成立
- scripts/agents/three-way-rerun-template.sh：判了这一轮的修补照字面与服务现行说法一致，成立
- scripts/gate/comment-history-check.py：判了带冒号才放过角色名、全角冒号照判，成立
- task-server/attempt_runner.py：判了这一轮的修补照字面与服务现行说法一致，成立
- task-server/README.en.md：判了第 224 行同一句，同改法 4
- task-server/README.ja.md：判了第 224 行同一句，同改法 4
- task-server/README.md：判了第 224 行取消那一句漏改与「旧命名」（坐实，改法 4）
- task-server/task：判了这一轮的修补照字面与服务现行说法一致，成立
- task-server/task_server_constants.py：判了这一轮的修补照字面与服务现行说法一致，成立
- task-server/task_server_lib_constants.py：判了这一轮的修补照字面与服务现行说法一致，成立
- task-server/task_server_lib.py：判了第 670–674 行「推的，没复跑」与复跑结局对不上（坐实，改法 3）
- task-server/task_server.py：判了这一轮的修补照字面与服务现行说法一致，成立
- task-server/task_server_scheduler_building.py：判了清扫旧命名目录不误删，成立；在跑尝试用着那一层没有格与开关（坐实，改法 3）
- task-server/task_server_selftest_lifecycle.py：判了这一轮的修补照字面与服务现行说法一致，成立
- task-server/task_server_selftest.py：判了这一轮的修补照字面与服务现行说法一致，成立
- task-server/task_server_selftest_shards.py：判了这一轮的修补照字面与服务现行说法一致，成立

## 回看决策

不涉及决策：这一轮判的是完工清理闸、注释历史判法、任务服务清扫、几份定义与 README 的改动，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
