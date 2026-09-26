<!-- knowledge-sync -->
# governance-defs-r3 阶段同步

触发文件：.claude/agent-common.md、.claude/agents/crash-verifier.md、.claude/agents/experiment-runner.md、.claude/agents/mutation-triage.md、.claude/agents/three-way-verifier.md、.claude/rules/mutation-sampling.md；另改了 .claude/main-agent.md（不在触发登记里，一并记）

这一阶段做成的事：严查第二轮之后的七份定义改动走完一轮三方（判决 research/prompts/governance-defs-r3-main-verification.md），攻方打中的 D1、D3–D7 与正推 G2、G3 逐条现查后写回，D1、D2 依赖的 cargo 输出形状由主 agent 实测坐实；写回处被攻过零轮。

## 搜索

- `sha256sum -c research/prompts/governance-defs-r3-snapshot/sha256sums.txt` → 41 行全部 OK
- 草稿目录小 crate（`named_before` 通过、`zz_abort_after` 调 `std::process::abort()`）`cargo test --offline --test t -- --test-threads=1` → 退 101；stderr 有 `error: test failed, to rerun pass \`--test t\`` 与 `process didn't exit successfully: … (signal: 6, SIGABRT: process abort signal)`；按 59 号 `judge_one` 判：zz_abort_after 没红、named_before 没红、never_reached 无效
- `grep -rnF '输出里有一行 \`error:\` 开头（测试二进制被信号杀时' .claude`（改后）→ 0
- `git diff 73ba4a4 -- crates/mutations.tsv | wc -l` → 0（带 `$` 的 11 行不是这个分支带进来的）

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/rules/mutation-sampling.md:82 | 测试进程没跑完的，输出里有一行 `error:` 开头…记「无效」 | 改了：D1、D2：照 59 号的判档次序 |
| .claude/agents/mutation-triage.md:28 | 同上 | 改了：D1、D2 |
| .claude/agents/mutation-triage.md:27 | 编不过的与测试进程被杀的在「无效」一栏 | 改了：D1 |
| .claude/agents/mutation-triage.md:19 | 提交时那一次门禁 59 号的输出路径 | 改了：正推 G2：指到崩溃验证员报告里的日志路径 |
| .claude/agents/crash-verifier.md:24 | 未跟踪文件三道共用一组路径 | 改了：D3、正推 G2 |
| .claude/agents/three-way-verifier.md:27 | 判别力自证按第 2 步核 | 改了：D5、正推 G3 |
| .claude/agents/three-way-verifier.md:28 | git log / git status 现查 | 补了：D4：被 git 忽略的文件记分不清 |
| .claude/main-agent.md:54 | 派 `three-way-verifier` | 补了：D5：给派腿时刻与快照 |
| .claude/agents/experiment-runner.md:32 | 另写一个 `driver_e<号>` 驱动函数 | 补了：D6：先 grep 现查、同实验加后缀 |
| .claude/agent-common.md:46 | 2 是包装的用法写错 | 改了：D7 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 正推腿报告 | research/prompts/governance-defs-r3-sonnet-output.md | 168 |
| 攻方腿报告 | research/prompts/governance-defs-r3-opus-output.md | 368 |
| 核查员报告 | research/prompts/governance-defs-r3-verifier-output.md | 115 |
