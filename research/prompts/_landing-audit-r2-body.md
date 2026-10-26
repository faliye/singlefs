# landing-audit-r2：子 agent 落地审计这一批改动第二轮正文（2026-10-22）

<!-- doc-lint:not-numbers K1 K2 K3 K4 -->

## 一、题面

第一轮判决 `research/prompts/landing-audit-r1-main-verification.md` 节「三、判决：要改的（交下一批 tooling-writer）」定了三份修补，第五批三个工具员照它改完，主 agent 补了几处（调度记录 `records/2026-10-22-子agent结果落地审计.md` 节「第一轮判决与第五批」逐条写了改了什么、主 agent 怎么验的、哪几格红是改前就红的）。这一轮照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮的分工判三件事，不重复第一轮已攻过的角度（第一轮的角度是：每处改法做没做到缺口、能不能合并与重合、新旧句子矛盾、记录里的数能不能复出来）：

1. 第一轮的修补本身：第五批每一处改法是不是做到了判决那一条要的、有没有改出新的重合、矛盾或漏网，判法变宽的地方有没有误拦或误放。
2. 产生结论的方法：这次审计的结论（结局账的分桶与门槛、返工字样的词表、「真浪费约 1% 到 2%、返工约 14%」这类判断、九条缺口的判定）是怎么从数据推出来的，推法本身站不站得住；第一轮判决采与不采的理由站不站得住。
3. 从未被看过的地方：两轮都没看过、而这一批改动碰得到的对象。

被判的文件（第五批改动加主 agent 补的几处；`git diff` 里同一份文件上别的会话的改动不在被判之列，见节「实现今天的样子」）：

| 类 | 文件 |
|---|---|
| 定义与共用约束 | `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/sweep.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md` |
| 钩子与登记表 | `.claude/hooks/runner-dispatch-guard.sh`（④ 与 ⑰ 共用的报告行解析、三方腿报告形态判法）、`.claude/hooks/continuation-guard.sh`、`.claude/hooks/lib_long_run.py`（`exempt-check` 改收 `--base`）、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/selftest-registry.tsv` |
| 脚本 | `scripts/agents/subagent_transcript.py`（新）、`scripts/agents/agent-cost.py`（`outcome --until`、词表、改从新库读）、`scripts/agents/agent-watch.py`、`scripts/agents/dispatch-prompt.py`、`scripts/agents/README.md`、`scripts/kb/stale-candidates.py`（读 `.claude/kb-directories`） |
| 门禁与登记 | `.claude/gate.d/lib-lag-baseline.py`（新）、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/doc-process-records.sh`、`.claude/gate.d/code-source-discipline.sh`、`.claude/gate.d/code-tooling.sh`、`.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/`、`-green/`；删掉的 `.claude/gate.d/replay-comparison-units-lag.tsv` |
| kb 与记录 | `.claude/kb/milestone/03-third-txn.md` 节「待优化：流程、门禁与装置上踩过的坑」第 3 行（三语各一份）；`records/2026-10-22-子agent结果落地审计.md`；`records/2026-09-16-subagent拆分提案.md` 节「九十、子 agent 落地审计里立了做法、还拦不住的四条与一条没想清的」；归档目录 `research/prompts/subagent-landing-audit-2026-10-22/` |

## 二、实现今天的样子（主 agent 的观测，2026-10-22）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一批没有改 `crates/` 下的任何文件：`git diff --name-only` 列出的 `crates/` 文件是别的会话的改动，不在被判之列；门禁 doc-experiments 的 replay-comparison-units 格读 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 里 `formal_run_units` 数组的字符串字面量，只读不改。

- 第五批三份交回报告原样在 `research/prompts/subagent-landing-audit-2026-10-22/`：`r1fix-gates.md`、`r1fix-definitions.md`、`r1fix-hooks.md`。
- 主 agent 在主工作区重跑过的读数（调度记录节「第一轮判决与第五批」）：`lib-lag-baseline.py --selftest` 8 项过、`LAG_BASELINE_BREAK=unreadable-as-absent` 下 3 项红；派发闸自证 454 种、续做闸 20 种、`dispatch-prompt.py` 90 项、`agent-cost.py` 14 项、`subagent_transcript.py` 12 项都退 0；真仓单跑 doc-experiments 的 verdict-false-named、multipath-registry、replay-comparison-units 绿，results-cited、quoted-result-lines、doc-process-records 两格、code-source-discipline 两格红，与改前那一版门禁对着同一棵树跑的输出逐行相同。
- 主 agent 补的：`.claude/gate.d/code-tooling.sh` 的 `SELFTEST_RUNNERS` 加新库自证一行；`.claude/kb/milestone/03-third-txn.md` 三语第 185 行改掉删掉的滞后表；归档目录从 `research/results/` 搬到 `research/prompts/`（门禁 results-cited 把它当没被实验页点名的产物）；闭窗口结局账 `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-2026-10-01-to-2026-10-21.txt`。
- 样本档 `stage-selftest.sh` 要样本与暂存区相同，这一批没暂存，样本档没在阶段里跑过（工具员在草稿里逐份核过退出码与 want）。
- 已有的同类判法（腿自己核）：`.claude/singlefs-ai-sop/scripts/gate-overlap.py --list`；`.claude/hooks/lib_*.py` 下已有的共用库；`scripts/agents/` 下已有读会话记录的脚本（`agent-handover.py`、`cite-check.py` 等）。
- 工作区里另有别的会话的未暂存改动：`.claude/hooks/lib_run_scope.py` 与 `.claude/hooks/run-scope-guard.sh` 的 `replay.sh --list-items` 几处、`.claude/agent-common.md` 与 `.claude/main-agent.md` 上别的会话的改动，不在被判之列。

开工快照：`research/prompts/landing-audit-r2-snapshot/judged-sha256.txt`（被判文件）、`research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff`（被判文件 `git diff` 对暂存区，加新建文件全文）。

## 三、格

### K1　第一轮判决够不够得着、是不是同样打中替代方案（辩方）

- 第一轮判决节「二、逐格判」每一条「采」：它采的理由是不是真打中了被改的那一处，改法是不是比别的改法更好；同一条理由放到判决没选的改法上是不是一样成立。
- 每一条「不采」（K2-7、K2-8 与正推里不采的那条）：替被否掉的一方辩护，不采的理由站不站得住。
- 判决节「五、没核动的」三条：没核动的会不会改变采与不采。
- 第五批每份交回报告「偏离与遗留」「判法变宽的地方」：工具员的偏离是不是合理，判决当初的要求是不是本来就做不到。

### K2　第一轮的修补本身（攻方）

- `.claude/gate.d/lib-lag-baseline.py` 并了七处之后：七处原来的判法与并后逐处比，有没有一处的语义被改了（基准取不到、基准里没有表、表为空、行尾空白、注释行这几种边角）；新库的自证与弄坏开关罩不罩得住这些边角。
- 删掉滞后表之后 replay-comparison-units 的「只缩不涨」：拿登记表里复跑写 `none:` 的行与基准比，换个写法能不能绕过（改登记出处、改产物名、把 none 行改成驱动名再删驱动行）。
- 派发闸 ④ 与 ⑰ 共用 `report_line_mentions` 之后：④ 现在也判裸「报告：」行，三方三种类型没写报告行就拒，会不会误拦已有的合法派发（拿 `.claude/agents/*.md` 的骨架与本会话的派发提示现试）。
- `scripts/agents/subagent_transcript.py` 的「交回」与「起点」定义：续做闸、`agent-cost.py`、看门狗改从它读之后，看门狗认交回从「结果里带 `"success":true`」改成「结果不是错误」、三个子命令改按首条记录时刻筛子 agent，这两处变宽有没有让某种情形判反。
- 第五批定义改动（`r1fix-definitions.md` 逐份列的 17 处）与这一批的钩子、门禁有没有新矛盾。

### K3　产生结论的方法（攻方）

- 结局账 `agent-cost.py outcome` 的分桶：「写了、不低于 70% 的行进了提交或工作树」「30% 到 70%」「低于 30%」这几道门槛与「三方腿」「报告被主 agent 点到」的认法，各自从什么数据推出来、推得出推不出调度记录节「结果」那几句结论（真浪费约 1% 到 2%、返工约 14%）；门槛换一个取值结论翻不翻。
- 返工字样词表（`REWORK_WORDS`）：去掉「修订」之后，词表的误报与漏报各是什么形态，「返工约 14%」作为上界成不成立。
- 调度记录节「九条缺口的判定」：每条缺口是从几个样本推出来的、样本是怎么挑的，有没有把一两次事件推成流程缺口。
- 第一轮判决的核查方法：`cite-check.py` 判了什么、没判什么；「与改前门禁对着同一棵树输出逐行相同」这种对照能证明什么、不能证明什么。

### K4　从未被看过的地方（攻方）

- 两轮都没看过、这一批碰得到的对象：例如 `lib_long_run.py exempt-check` 的调用方、`stale-candidates.py` 读登记位之后 `.claude/kb-directories` 登记了而 `quote-kb.py` 不认的形态、`agent-write-scope.tsv` 写回员那几行与 `.claude/kb-directories` 的覆盖关系、归档目录搬到 `research/prompts/` 之后有没有门禁按 `research/prompts/` 的规矩（冻结证据、三方轮名）判它。
- 每一条写「这一批里谁该管它、现在谁也没管」，给能复出来的命令。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 复核第一轮判决：采的够不够得着、不采的替它辩护、同一理由是不是同样打中替代方案 |
| 云端攻方（Opus） | K2、K3、K4 | 攻第五批修补本身、攻产生结论的方法、找两轮都没看过的地方 |
