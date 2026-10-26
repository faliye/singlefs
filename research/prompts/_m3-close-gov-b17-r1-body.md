# m3-close-gov-b17-r1：批 17 治理文件改动，做法写得对不对、与已定条款和脚本对不对得上（推论轮第一轮，治理轮）正文（2026-10-23）

## 一、题面

这一批治理文件改动按 `.claude/main-agent.md`「一轮怎么开、怎么收」第 4 条一批走一轮三方，用户 2026-10-23 回「文件治理可以提交做吧」（调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`）。被判的改动两份 diff，都在开工快照目录 `research/prompts/m3-close-gov-b17-r1-snapshot/`：

1. `governance.diff`（批 17 工具员，已打进主工作区）：
   - `.claude/rules/implementation-workflow.md`「复用重型门禁的判定」那一节、`.claude/rules/verification.md`「两档：轻型与重型」重型那一行，补上 D13（验证路线） 已定项 17 的两种排除（文件级 `NOT_JUDGING_FILES`、边级只打提示不改结局的子进程边）与「模块级语句随模块进闭包」；implementation-workflow 那一处另写新加边级排除要配的自证。
   - `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」第 3 步：判决复跑的副本留到这一题的下一轮判决写完、或不再开下一轮时再删。
   - `.claude/skills/crash-test/commit-stages.md` 第 2b 步：广谱变异驱动「GPU 那一趟」的做法与 ④ 要抄的四个 gpu 数。
2. `ji8-hooks-and-verification.diff`（己八工具批，已在主工作区）：`.claude/hooks/lib_heavy_tests.py` 与 `.claude/hooks/heavy-test-guard.sh` 让 `scripts/experiment-apparatus/replay.sh --list-items` 不算重型；`.claude/rules/verification.md` one-scenario 那一行从「harness 档」扩到「harness 档与 checker 档」。

要回答：每一处写的做法是不是照它压着的条款（D13 已定项 17 定案、`.claude/main-agent.md` 第 11 条、驱动与钩子的现行为）；有没有写进理由与来历（`.claude/singlefs-ai-sop/rules/rules-discipline.md` 第 1、2 条）；有没有与别的规则、定义读来矛盾；钩子的放行会不会放过真重型。

## 二、实现今天的样子（主 agent 的观测，2026-10-23，行号按主工作区现查）

- 复用键的算法住在 `crates/singlefs-unit-graph`（Rust 与 WGSL）与 `scripts/gate/lib_function_units.py`（python）；判定库的共用读写在 `scripts/crash/lib_verdict_library.py`，`NOT_JUDGING_FILES` 在那里定义。边级排除那一条边是 `scripts/hosts/run-with-memory-cap.sh` 里 `HostLookup.query` 起 `scripts/hosts/host-resources.py`。
- 广谱变异驱动 `scripts/mutation/broad-mutation.py` 文件头「GPU 那一趟」一段是 commit-stages 第 2b 步复述的对象。
- 重型闸的判法在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`；`scripts/experiment-apparatus/replay.sh` 的 `--list-items` 只列项。
- `.claude/rules/verification.md` one-scenario 那一行对着 `.claude/gate.d/harness-model-differential-and-scenarios.sh` 文件头与 `scripts/crash/crash-case-check.py` 的 one-scenario 那一样。
- 快照 `judged-sha256.txt` 里 `scripts/hosts/run-with-memory-cap.sh`、`scripts/mutation/broad-mutation.py` 两份，别的会话这一两个小时会换新版（改的是自证与截断写法），对不上的照 `.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」倒推。

## 三、格

| 格 | 问什么 |
|---|---|
| G1 复述对不对 | 两处 D13 已定项 17 复述与定案逐项比：多了什么、少了什么、有没有把只在一处写的自证要求写成两处不一致；「现在只有那一条」这类现状句该不该进规则 |
| G2 复跑副本保留 | 新加那一句与 `.claude/main-agent.md` 第 11 条、`.claude/hooks/completion-cleanup-guard.sh` 的收工闸读来矛不矛盾；收工闸会不会照旧拦下留着的副本 |
| G3 GPU 那一趟 | commit-stages 第 2b 步写的每一样（经 `gpu-quiesce-run.sh`、`--test-threads 1`、`gpu-verdicts.tsv`、`--skip-gpu`、四个 gpu 数、NO_CONCLUSION 的几种理由与出路）在驱动源码里逐项对不对得上 |
| G4 重型闸放行 | `replay.sh --list-items` 放行的判法：会不会放过带别的参数、经包装、改了名的真重型；verification.md one-scenario 那一行扩到 checker 档与判法实际读的目录对不对得上 |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | G1–G4 | 从条款与脚本源码正推：每一处写的做法是不是条款说的、脚本做的 |
| 云端攻方（Opus） | G1–G4 | 找反例：照规则字面做会做错的情形、与别的规则或定义打架的句子、钩子放过的真重型命令 |

方案按 `crates/` 与脚本今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
