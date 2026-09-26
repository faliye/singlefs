<!-- knowledge-sync -->
# governance-defs-r1 阶段同步

触发文件：.claude/agent-common.md、.claude/agents/crash-verifier.md、.claude/agents/experiment-designer.md、.claude/agents/experiment-runner.md、.claude/agents/gate-triage.md、.claude/agents/implementation-writer.md、.claude/agents/kb-scribe.md、.claude/agents/mutation-triage.md、.claude/agents/sweep.md、.claude/agents/three-way-attack.md、.claude/agents/three-way-local-attack.md、.claude/agents/three-way-local-defense.md、.claude/agents/three-way-materials.md、.claude/agents/three-way-verifier.md、.claude/gate.d/stage-owners.tsv、.claude/hooks/bash-command-detector.sh、.claude/hooks/heavy-test-guard.sh、.claude/rules/implementation-workflow.md、.claude/rules/mutation-sampling.md；另改了 .claude/main-agent.md（不在触发登记里，一并记）

这一阶段做成的事：第 3 轮的十五份定义改动走完一轮三方（判决 research/prompts/governance-defs-r1-main-verification.md），攻方打中的 A1–A10 与正推的开放点逐条现查后写回，写回处被攻过零轮。

## 搜索

- `sha256sum -c research/prompts/governance-defs-r1-snapshot/sha256sums.txt` → 腿交齐时 31 行全部对得上
- `grep -rn "crash-verifier.*54\|54.*crash-verifier\|跑 54 号快档"` 治理文档、钩子、规则 → 改后只剩 heavy-test-guard.sh 的放行集合与它的自证用例（判法不动，交用户）
- 合成 PreToolUse JSON 喂 heavy-test-guard.sh（experiment-runner）：`bash research/scripts/replay.sh E158` 退 2，经 `run-with-memory-cap.sh 4G` 退 0
- 合成 JSON 喂 bash-command-detector.sh（run_in_background）：`a & b & wait` 退 0 不记检出；两个 `&` 后逐个 `wait "$pid"` 退 0 但记一条「后面没有 wait 的 &」；`{ a; echo $? > a.rc; } & { b; echo $? > b.rc; } & wait; cat …` 退 0 不记检出
- `bash .claude/hooks/heavy-test-guard.sh --selftest` → 562 种通过；`bash .claude/hooks/bash-command-detector.sh --selftest` → 430 种通过；62 号、72 号绿

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/main-agent.md:14 | （第 3 轮改动，未提交） | 改了：见 research/prompts/governance-rot-r3-sync.md 对应行；这一轮三方判「站得住」照留 |
| .claude/main-agent.md:59 | 派 crash-verifier 跑 54 号快档；抄一份 --full 命令；触发只看登记输入 | 改了：A1、A2、正推开放点 |
| .claude/main-agent.md:61 | 33 号红 → experiment-runner 只修锚点 | 改了：A9 |
| .claude/agent-common.md:46 | 只写了退出码 250 | 改了：A4 |
| .claude/agent-common.md:55 | 逐个 wait "$pid" | 改了：A10 |
| .claude/agent-common.md:46 | crash-verifier 只跑 54、55、57、59 | 改了：A1 |
| .claude/agents/crash-verifier.md:24 | 54 号只跑快档 | 改了：A1 |
| .claude/agents/experiment-runner.md:32 | bash research/scripts/replay.sh E<号> | 改了：A3 |
| .claude/agents/experiment-runner.md:43 | 变异三个数 | 改了：A7 |
| .claude/agents/mutation-triage.md:28 | 照每条的 ✅ [抓到] 等标记数 | 改了：A7 |
| .claude/agents/three-way-local-attack.md:27 | （取号没写）；共用约束开着 noclobber | 改了：A5、A6 |
| .claude/agents/three-way-verifier.md:27 | （缺这一支） | 改了：A8 |
| .claude/agents/experiment-designer.md:28 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/agents/gate-triage.md:26 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/agents/implementation-writer.md:28 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/agents/kb-scribe.md:32 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/agents/sweep.md:22 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/agents/three-way-attack.md:37 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/agents/three-way-local-defense.md:11 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/agents/three-way-materials.md:24 | （第 3 轮改动） | 改了：第 3 轮，这一轮三方未打中，照留 |
| .claude/gate.d/stage-owners.tsv:37 | 54-layer0-replay.sh 归 crash-verifier | 改了：A1，62 号绿 |
| .claude/hooks/heavy-test-guard.sh:519 | 拒绝出路：crash-verifier 只跑 54、55、57、59 | 改了：A1；判法不动（放行比职责宽，交用户），--selftest 562 种通过 |
| .claude/hooks/bash-command-detector.sh:2366 | 拒绝出路：a & b & wait | 改了：A10；--selftest 430 种通过；逐个 wait "$pid" 被记成没等齐属判法，交用户 |
| .claude/rules/implementation-workflow.md:55 | 崩溃验证员跑 54 号快档与 55、57、59 号 | 改了：A1 |
| .claude/rules/mutation-sampling.md:82 | 照每条的 ✅ [抓到] 等标记数 | 改了：A7 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| governance-defs-r1-sonnet-output.md | research/prompts/governance-defs-r1-sonnet-output.md | 259 |
| governance-defs-r1-opus-output.md | research/prompts/governance-defs-r1-opus-output.md | 364 |
| governance-defs-r1-verifier-output.md | research/prompts/governance-defs-r1-verifier-output.md | 153 |
| run-all.out | research/prompts/governance-defs-r1-opus-model/run-all.out | 138 |
