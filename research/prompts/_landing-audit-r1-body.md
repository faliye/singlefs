# landing-audit-r1：子 agent 落地审计这一批改动第一轮正文（2026-10-22）

<!-- doc-lint:not-numbers K1 K2 K3 K4 -->

## 一、题面

调度记录 `records/2026-10-22-子agent结果落地审计.md` 节「九条缺口的判定」判出八条要改的缺口，第三批四个工具员与主 agent 照它改了一批定义、共用约束、钩子、门禁与脚本（同一份记录节「第三批交回」逐条写了改了什么、主 agent 怎么验的）。这一轮判两件事：

1. 每一处改法是不是做到了它要关的那条缺口，有没有越位、漏改、把推的写成事实。
2. 用户点名的三面：这一批新加或改动的判法、格、子命令、登记表，能不能并进已有的那一份；与已有的脚本、钩子、门禁有没有重合（管同一件事、同一批对象、同一个触发点，或整段相同、共用逻辑没抽成库）；新写的句子、判法与已有的规则、定义、钩子有没有矛盾。

被判的文件：

| 类 | 文件 |
|---|---|
| 定义与共用约束 | `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/sweep.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/implementation-writer.md` |
| 钩子与登记表 | `.claude/hooks/runner-dispatch-guard.sh`（新判法 ⑰）、`.claude/hooks/lib_heavy_tests.py`（`classify_gate_stage` 的命名自查放行）、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/write-guard.sh`、`.claude/hooks/handback-guard.sh`、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/selftest-registry.tsv` |
| 脚本 | `scripts/agents/agent-cost.py`（新子命令 `outcome`）、`scripts/agents/dispatch-prompt.py`、`scripts/agents/agent-watch.py`、`scripts/agents/cite-check.py`、`scripts/agents/README.md`、`scripts/gate/apply-writer-patch.py`、`scripts/kb/claim-experiment.sh`、`scripts/kb/stale-candidates.py` |
| 门禁与登记 | `.claude/gate.d/doc-experiments.sh`（新格 replay-comparison-units）、`.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/replay-comparison-units-lag.tsv`、`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/`、`-green/`、`-zero/` |
| kb | `.claude/kb/checks-owed.md` 的 C879、C880 两行；`.claude/kb/milestone/03-third-txn.md` 节「待优化：流程、门禁与装置上踩过的坑」第 3 行现状（三语各一份） |

## 二、实现今天的样子（主 agent 的观测，2026-10-22）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一批没有改 `crates/` 下的任何文件：`git diff --name-only` 列出的 `crates/` 文件是别的会话的改动，不在被判之列；被判的门禁新格读 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 里 `formal_run_units` 数组的字符串字面量，只读不改。

- 子 agent 用 Write 写名字为 `report.md` 的文件，被 Claude Code 自带的拦截拒（工具结果原文「Subagents should return findings as text, not write report files. Include this content in your final response instead.」）：2026-10-14 起 209 次全拒，名字里带 report 的别的文件 16 次里拒 6 次，`result.md` 这一批 9 个子 agent 写成。仓里的钩子没有这句话。
- 主 agent 在主工作区重跑过的读数（记录节「第三批交回」）：新格真仓单跑绿（14 行，10 行复跑相符，4 行在滞后表）；三份样本照 `expect` 判对；八个弄坏开关逐个判出；`claim-experiment.sh`、`stale-candidates.py`、`apply-writer-patch.py`、`agent-cost.py` 的 `--selftest` 绿、开关红；runner-dispatch-guard 自证 437 种、heavy-test-guard 1386 种、write-guard 228 种绿。
- 已有的同类判法（现查的落点，腿自己核）：`.claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 列已有门禁与钩子；doc-experiments 已有 multipath-registry 格与 `.claude/gate.d/multipath-registry-lag.tsv`；`scripts/agents/agent-watch.py`、`scripts/agents/watch.sh` 报交回与没交回；`scripts/agents/agent-cost.py` 原有 usage、timing、dispatch 三个子命令；runner-dispatch-guard 原有 ① 到 ⑯ 的派发闸判法。
- 工作区里另有别的会话的未暂存改动，`runner-dispatch-guard.sh` 里有别的会话先改的 TaskStop 13 行，不在被判之列。

开工快照：`research/prompts/landing-audit-r1-snapshot/judged-sha256.txt`（被判文件）、`research/prompts/landing-audit-r1-snapshot/judged-vs-index.diff`（被判文件 `git diff` 对暂存区，加新建文件全文）。

## 三、格

### K1　每处改法做没做到它要关的缺口（正推）

- 对着调度记录节「九条缺口的判定」与「第三批依赖表」：每条缺口的改法落在哪一份文件哪一句、哪一个判法；做到的写依据（文件:行），没做到、只做了一半、越出缺口的写哪一格。
- 每条改法照 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」第 3 步：先造一个该红的输入，现有门禁或钩子会不会红；不会红的，调度记录引的 `records/2026-09-16-subagent拆分提案.md` 节「九十、子 agent 落地审计里立了做法、还拦不住的四条与一条没想清的」有没有登到。

### K2　能不能合并、和已有脚本有没有重合（攻方）

- 新格 replay-comparison-units 与 multipath-registry 格、它们各自的滞后表：判的是不是同一类对象、读不读同一份输入、能不能并成一格或共用一段判法。
- 新判法 ⑰ 与 runner-dispatch-guard 已有的判法、与 write-guard、handback-guard 里管报告文件的判法：是不是同一件事判了两处。
- `agent-cost.py outcome` 与 `agent-watch.py`、`watch.sh` 报的交回与没交回、与 `agent-cost.py` 原有三个子命令：口径是不是重了、能不能共用读会话记录的那一段。
- `claim-experiment.sh` 查各工作树、`stale-candidates.py` 认不变量清单、`lib_heavy_tests.py` 的命名自查放行：有没有别的脚本已经在做同一件事（例：别处已有列工作树、列载体、放行轻型的判法）。
- 用 `gate-overlap.py --list` 与它的整段相同判法现查，引它的输出。

### K3　有没有矛盾（攻方）

- 新句子与已有规则、定义、钩子逐对比：例如 `.claude/agent-common.md` 节「门禁」原有「子 agent 交回前不跑门禁阶段与各类 lint」与新加的「命名检查交回前跑」；`.claude/main-agent.md` 派发表里写回员「不改 `crates/`」与 `crates/singlefs-checker/invariants/*.md` 的例外；`.claude/agents/experiment-designer.md` 第 3a 步「核可跑性与判别力」与 `.claude/singlefs-ai-sop/rules/test-discipline.md`「实验开跑之前，答案不许已经存在」；`.claude/agents/sweep.md` 的「交实现员」一节与 `.claude/main-agent.md`「一轮怎么开、怎么收」第 4 条；`.claude/main-agent.md` 第 2 条「延后的当时就登记」与第 7、8 条和 CLAUDE.md「经验与约定写在哪」（子 agent 欠账不占 C 号）。不限于这几对。
- 报告文件名改 `result.md` 之后：全仓还有没有别的定义、技能、规则、脚本仍要求或按名字认 `report.md`（引 `grep -rn` 的完整计数，不截断）。
- 钩子放行与拒绝之间：新放行的命名自查命令会不会被别的钩子（run-scope-guard、two-logs-guard、bash-command-detector）拒，定义里给的命令照抄能不能过。

### K4　数与读数能不能核（攻方）

- 调度记录节「结果」与「第三批交回」报的数（结局账 2268 个、各桶占比、返工字样 353 个 14.1%、盲判一致率 37/42 与 33/42、延后项 8/54、回扫 `crates/` 行 16/129）：照记录写的命令在副本上复出来，或指出哪一个复不出来、为什么。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1 | 从缺口推到改法：每条做到没有、依据在哪 |
| 云端攻方（Opus） | K2、K3、K4 | 找能并的、重合的、互相矛盾的，和复不出来的数 |
