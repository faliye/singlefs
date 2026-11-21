# build-dir-clean-wording-governance-r2：判决（2026-11-21）

正文 `research/prompts/_build-dir-clean-wording-governance-r2-body.md`，开工快照 `research/prompts/build-dir-clean-wording-governance-r2-snapshot/`。这是这一题的第二轮，攻第一轮判决 `research/prompts/build-dir-clean-wording-governance-r1-main-verification.md` 的修补并复核它。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/build-dir-clean-wording-governance-r2-opus-output.md` | 四问都「推不出（打中）」：完工清理闸按尝试表的会话列取尝试，服务写的是空串，真库上一个现场都列不出；第 9 步 `--cwd <运行目录>` 配相对的模型目录找不到脚本；「」里只放角色名、来历写在「」外的句子被放过；「取消都不删它」对派到第二台、中途被停的那一趟是假话 |
| 辩方（Sonnet） | `research/prompts/build-dir-clean-wording-governance-r2-sonnet-output.md` | 第一轮七条结论都成立，四处依据有缺口（第 1 条撑结论的现场选错、第 2 条输出目录没量、第 4 条「登常设豁免」那一支不可用、`lib_workspace_cargo_lock.py:225` 还有一句同类话）；改法 4 选修判法成立，改法 1 对「判全部现场」推不出（闸只抄了 `scene_due` 两个不到期条件里的一个）；另报旧命名编译目录 12 个、81G 没人删 |

核查：

- 倒推开工快照的树：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-main-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py build-dir-clean-wording-governance-r2 --out /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-main-rerun/build-dir-clean-wording-governance-r2-tree --repository /home/fy5090/code/singlefs`，type=succeeded exit=0，`✓` 31 行、`✗` 0 行。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/build-dir-clean-wording-governance-r2-opus-output.md research/prompts/build-dir-clean-wording-governance-r2-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-main-rerun/build-dir-clean-wording-governance-r2-tree --background research/prompts/_build-dir-clean-wording-governance-r2-body.md`，末行「✓ 核了 68 处引文、30 处裸行号，对上 35 处，没判 63 处」，没有判红的；判决要用的几处主 agent 现读：`task-server/task_server_scheduler_starting.py:140`（尝试记录写 `"session": ""`）、`task-server/task_server_store.py:268`（库照记录写会话列）、`.claude/hooks/completion_cleanup_guard_decisions.py:1294`、`:1299`（`WHERE session = ?` 取任务与尝试）、`.claude/hooks/completion_cleanup_guard_selftest.py:170`（自证样本库把尝试的会话列填成会话号）、`.claude/hooks/completion-cleanup-guard.sh:35`、`task-server/task_server_scheduler_building.py` 的 `scene_due`（有会话、没取走的不到期；设了留现场的只由 task clean 删）、`task-server/task_server_lib.py:276`（第二台中途失败、被停的那一趟「⑥ 连它一起删了」）与 `:286`、`.claude/hooks/lib_workspace_cargo_lock.py:225`。
- 主 agent 复跑（腿的模型目录与探针拷进 `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-main-rerun/`，sha256 与腿报告相同之后在副本里交）：
  - 攻方整份复跑 `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/bdcw-r2-main-abs-run -- bash <副本>/model/tools/rerun-from-a-clean-work-directory.sh …`，type=succeeded exit=0：「name=kept_scene_store_shape arm=selftest-shape attempt_session_column=s-kept-scene-store-shape hook_exit=0 blocked=true scene_listed=true clean_named=true guard_query_rows=1 joined_query_rows=1」「name=kept_scene_store_shape arm=store-shape attempt_session_column=empty hook_exit=0 blocked=false scene_listed=false clean_named=false guard_query_rows=0 joined_query_rows=1」；「name=compiles_declaration cwd=…/bdcw-r2-main-abs-run cargo_target_dir=…/task-server-builds/tree-c227e2fc-a5c04d/target expected_local_stem=tree-c227e2fc」；「name=quoted_role_names arm=current checker_exit=0 listed_lines=5,7」「name=quoted_role_names arm=quoted-role-names-judged checker_exit=0 listed_lines=2,3,4,5,6,7」。
  - 照字面那一形（`--cwd <运行目录>`、模型目录写相对路径）type=command-failed exit=127。
  - 喂项目代码闸、范围声明闸各一条第 9 步的 submit、项目代码闸一条 `mkdir -p`，三条都 type=succeeded exit=0、`✗` 0 行。
  - 真库：`task-server/task status <整份复跑那一条> --json` 里那一次尝试的 `session` 是 `''`。
  - 辩方注释探针 `python3 scripts/gate/comment-history-check.py --list <副本>/comment-probe-root` 列出第 3、4、5 行，没列第 2 行「# 这一段是「攻方腿」打中的那一处的修补」。
  - 辩方输出目录探针（`--cwd <副本>/profile-probe`，七种构建形态各编一个二进制再 find），落点：`custom/bin_profile_custom`、`debug/bin_dev`、`debug/examples/ex_one`、`release/bin_profile_bench`、`release/bin_profile_release`、`release/bin_release_flag`、`x86_64-unknown-linux-gnu/debug/bin_target_triple`，与两份定义写的对照逐项相同。
  - 现数：`~/.local/share/singlefs/task-server-builds/` 下旧命名 `task-*` 12 个、`du -sch` 合计 81G，`tree-*` 13 个。

## 二、判决

1. 攻方第 1 问：坐实而采。完工清理闸新判的任务现场在真库上一个都列不出：服务起尝试时会话列写空串（真库那一条尝试的 `session` 是 `''`），闸按 `session = ?` 取尝试；自证样本库填了会话号，看不见这一处。复跑两臂分辨得出（同一闸，`selftest-shape` 臂 `blocked=true`，`store-shape` 臂 `blocked=false`，按任务号接任务表查 `joined_query_rows=1`）。
2. 攻方第 2 问：坐实而采。第 9 步与模板用法写 `--cwd <运行目录>` 而模型目录给相对路径，照字面交退 127；会编声明那一形本身成立（`cargo_target_dir` 落在 `--cwd` 那棵树的组 `tree-c227e2fc`），两道闸放行，输出目录对照经探针逐项坐实。对照表漏 `-r` 与 `--profile=<名>` 两种写法，采，低。
3. 攻方第 3 问：坐实而采。「」里只放角色名、来历写在「」外的句子被新判法放过（探针第 2 行没列出，开关打开那一臂列出）；派发闸与检出钩子现有被放过的格名都是「带冒号的格名」那一形，改成「」里带冒号才放过角色名，不再放过第 2 行那种。
4. 攻方第 4 问：坐实而采。「取消都不删它」对派到第二台、中途失败或被停的那一趟不成立（`task-server/task_server_lib.py:276`）；这几处照抄的是同一份文件头第 286 行，第 286 行本身与第 276 行对不上，一起改。三处「树」漏「解开符号链接后」，采，低。
5. 辩方第 5 问：第一轮七条结论维持。依据补正：第 1 条撑「留现场的现场长期留着」应引 2026-11-12 那一个（辩方量的，设了留现场、已取走），最早两个是另一种（没设留现场、结局没取走），见第 6 条；第 2 条输出目录这一轮经探针量过；第 4 条「或登常设豁免」那一支不可用（`.claude/gate.d/comment-history-exempt.tsv` 只收主题就是历史词表的整份文件，滞后表新写的不许加），这一轮选的修判法不受影响；第 7 条 `.claude/hooks/lib_workspace_cargo_lock.py:225`「编译目录用完就删」与紧跟着的「编译目录归树、不归任务：不用自己删」矛盾，坐实而采。
6. 辩方第 6 问：改法 4 选修判法成立。改法 1 只判设了留现场的，对「判全部现场」推不出那一句坐实而采：闸文件头写与 `scene_due` 同一口径，`scene_due` 的不到期条件有两个（有会话、结局没取走；设了留现场），闸只判了后一个；自证 `unkept-scene-not-listed` 那一格的断言写「服务在取走结局之后到时自己删」，样本却是没取走的。改成两个条件都判。
7. 辩方另报：旧命名编译目录 12 个、81G 没有一条路删（服务按组清扫、`task clean` 不删编译目录、完工清理闸不判），坐实而采：改成服务清扫时一并删编译根下不在任何组里、没有进程开着、名字是旧命名的目录。

## 三、改法

1. 完工清理闸取尝试改成按任务号接任务表、用任务表的会话判（复跑探针 `joined_query_rows=1` 那一形）；自证样本库的尝试行照服务真写的形状把会话列写空串，加会红的弄坏开关。另判第二个不到期条件（本会话已有结局、结局没取走、现场还在），出路写取走结局或 `task clean <号>`；`unkept-scene-not-listed` 那一格的样本与断言照两个条件改。
2. `.claude/agents/three-way-attack.md` 第 9 步与复跑模板用法里的模型目录写成绝对路径（`<仓根>/research/prompts/…`），并写明 `--cwd` 换了之后脚本路径照它解；对照表补 `-r`、`--profile=<名>`。`.claude/agents/experiment-runner.md` 同一处照改。
3. 注释历史判法：「」里带冒号的才放过角色名；配会红的样本（「攻方腿」只带角色名、来历写在外面那一形）与弄坏开关。
4. 「取消都不删它」几处（`task-server/task`、`task-server/attempt_runner.py`、`gpu-verification-server/control`、`prune-server/control`，与 `task-server/task_server_lib.py:286`、`remove_cancelled_artifacts` 的文档串）照第 276 行写全：本机组里的不删，派到第二台、中途失败或被停的那一趟连它一起删；三处「树」补「解开符号链接后」。
5. `.claude/hooks/lib_workspace_cargo_lock.py:225` 那一句去掉「编译目录用完就删」，只留这一道堵的是什么。
6. 任务服务清扫加一支：编译根下不在任何组里、没有进程开着的旧命名目录一并删，配自证格与弄坏开关；换上后照服务的换法重启。
7. 这几条改完开第三轮（最后一轮），攻这一轮的修补；第三轮新冒出来的形态标零轮，不再开第四轮。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/agents/experiment-runner.md：判了两种交法与输出目录对照（探针逐项坐实），漏 `-r`、`--profile=<名>`（改法 2）
- .claude/agents/three-way-attack.md：判了第 9 步 `--cwd <运行目录>` 配相对模型目录照字面退 127（坐实，改法 2）；会编声明那一形成立
- .claude/gate.d/break-switch-replay-files.tsv：判了第 10、27、33、70 行条数与复跑结局一致，成立
- .claude/gate.d/comment-history-lag.tsv：判了 `prune-server/control` 那一行改小到现数，成立
- .claude/gate.d/doc-text.sh：判了第 698 行「另十七条」与判法开关现数一致，成立
- .claude/hooks/bash-command-detector.sh：判了拿掉七处 `history-kept` 之后判法不再点名它，成立；格名都带冒号，改法 3 之后照旧放过
- .claude/hooks/completion_cleanup_guard_decisions.py：判了按 `session = ?` 取尝试在真库上取不到（坐实，改法 1），只判一个不到期条件（坐实，改法 1）
- .claude/hooks/completion_cleanup_guard_selftest.py：判了样本库尝试的会话列填了会话号、看不见真库形状，`unkept-scene-not-listed` 断言文字与样本不符（坐实，改法 1）
- .claude/hooks/completion-cleanup-guard.sh：判了文件头「与 scene_due 同一口径」只抄了一个条件（坐实，改法 1）
- .claude/hooks/lib_project_code_runs_base.py：判了写全的树与出路照字面走得通，成立；树漏「解开符号链接后」的一处同改法 4
- .claude/hooks/lib_workspace_cargo_lock.py：判了第 225 行「编译目录用完就删」与出路矛盾（坐实，改法 5）
- .claude/hooks/project_code_guard_decisions.py：判了出路与 `TASK_SERVER_BYPASS_POLICY` 写全的树，成立
- .claude/hooks/project_code_guard_selftest.py：判了钉的新句，成立
- .claude/hooks/project-code-guard.sh：判了出路写全的树，成立
- .claude/kb/tooling.en.md：判了第 42 行写全的树与中文一致，成立；「取消都不删」同改法 4
- .claude/kb/tooling.ja.md：判了第 42 行写全的树与中文一致，成立；「取消都不删」同改法 4
- .claude/kb/tooling.md：判了第 42 行写全的树，成立；「取消都不删」同改法 4
- .claude/skills/dispatch/SKILL.md：判了第 18 行逐样列出闸拦哪几样，成立；任务现场那一样随改法 1
- gpu-verification-server/control：判了第 392 行「取消都不删它」对第二台被停那一趟不成立（坐实，改法 4）
- multi-host.env.example：判了第 114 行编译目录归树的说明，成立
- prune-server/control：判了第 163 行「取消都不删它」同改法 4
- research/prompts/build-dir-clean-wording-governance-r1-main-verification.md：辩方复核七条结论维持，第 1、2、4、7 条依据补正写在「二、判决」第 5 条
- scripts/agents/three-way-rerun-template.sh：判了会编声明与开关只改文件头摘录，成立；用法里相对模型目录同改法 2
- scripts/gate/comment-history-check.py：判了「」里只放角色名、来历写在外面的句子被放过（坐实，改法 3）
- task-server/attempt_runner.py：判了第 27 行「取消都不删它」同改法 4
- task-server/README.en.md：判了第 224 行写全的树，成立
- task-server/README.ja.md：判了第 224 行写全的树，成立
- task-server/README.md：判了第 224 行写全的树，成立
- task-server/task：判了 clean 说明与输出改成删现场，成立；cancel 说明「取消都不删它」同改法 4
- task-server/task_server_constants.py：判了第 289 行写全的树，成立

## 回看决策

不涉及决策：这一轮判的是完工清理闸、两份定义与复跑模板、注释历史判法、任务服务与几个控制命令的说明，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
