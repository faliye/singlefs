<!-- knowledge-sync -->
# governance-defs-r2 阶段同步

触发文件：.claude/agents/crash-verifier.md、.claude/agents/experiment-designer.md、.claude/agents/gate-triage.md、.claude/agents/mutation-triage.md、.claude/agents/three-way-verifier.md、.claude/gate.d/54-layer0-replay.sh、.claude/rules/implementation-workflow.md、.claude/rules/mutation-sampling.md；另改了 .claude/main-agent.md（不在触发登记里，一并记）

这一阶段做成的事：严查第一轮之后的十份定义改动走完一轮三方（判决 research/prompts/governance-defs-r2-main-verification.md），攻方打中的 B1–B7 与正推 G3 逐条现查后写回，写回处被攻过零轮。

## 搜索

- `sha256sum -c research/prompts/governance-defs-r2-snapshot/sha256sums.txt` → 腿交齐时 42 行全部 OK
- `grep -c 'stage-must-run.sh' .claude/gate.d/{54,55,57,59,74,87}-*.sh` → 57 号 0，其余都调用；复用名单补 87
- `grep -rnF '那一行各栏都有' .claude research/scripts`（改后）→ 0
- `grep -rnF '54、55、59、74 号只在' .claude`（改后）→ 0
- 54 号打印的三行拿桩在临时仓里放进同一次 `bash -c` 跑：`--full` 退 0 / 1 / 250 / 251 时整次调用退同一个数，临时目录 0、worktree 只剩主树
- 已有登记里「英文名」：`grep -l '英文名' research/prompts/e*-preregistration.md | wc -l` → 0（B6 属实）

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/gate.d/54-layer0-replay.sh:233 | --full …; git worktree remove --force … | 改了：B4：保留 --full 的退出码、清理临时目录 |
| .claude/gate.d/54-layer0-replay.sh:230 | 把下面三行命令放进同一次 Bash 调用（…） | 补了：这次调用的退出码就是 --full 的 |
| .claude/main-agent.md:59 | 全量判绿、全绿标记写好之后才往下；54、55、59、74 号 | 改了：B4 看这次调用退 0；B5 补 87 |
| .claude/rules/implementation-workflow.md:55 | 54、55、59、74 号只在这一趟里能复用 | 改了：B5 |
| .claude/rules/implementation-workflow.md:26 | 记一份 sha256sum 快照，交核查员当输入 | 补了：B7 连同派腿时刻 |
| .claude/agents/gate-triage.md:26 | 54、55、59（还有 74） | 改了：B5 |
| .claude/rules/mutation-sampling.md:82 | 门禁 59 号照它自己收尾的「计数：」行，那一行各栏都有 | 改了：B1 |
| .claude/agents/mutation-triage.md:28 | 那一行各栏都有；`💥` 不判失败 | 改了：B1 |
| .claude/agents/crash-verifier.md:24 | git diff --quiet -- crates Cargo.toml Cargo.lock litmus research/scripts research/results | 改了：B2、B3：每道取自己的输入，未跟踪文件要没有，停下之后的出路 |
| .claude/agents/experiment-designer.md:28 | 重跑登记照抄原登记的英文名 | 补了：B6 原登记没有的从装置文件名取 |
| .claude/agents/three-way-verifier.md:22 | 没给就不查那一支，照实写没查 | 改了：B7 没给时落「分不清」 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 正推腿报告 | research/prompts/governance-defs-r2-sonnet-output.md | 108 |
| 攻方腿报告 | research/prompts/governance-defs-r2-opus-output.md | 334 |
| 核查员报告 | research/prompts/governance-defs-r2-verifier-output.md | 117 |
