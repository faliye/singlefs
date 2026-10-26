# landing-audit-r3：子 agent 落地审计这一批改动第三轮正文（2026-10-23）

<!-- doc-lint:not-numbers K1 K2 K3 K4 -->

## 一、题面

前两轮判决：`research/prompts/landing-audit-r1-main-verification.md`、`research/prompts/landing-audit-r2-main-verification.md`。第七批三个工具员照第二轮判决节「三、判决」改完（调度记录 `records/2026-10-22-子agent结果落地审计.md` 节「第二轮判决与第七批」逐条写了改了什么、主 agent 怎么验的）。这是第三轮、也是最后一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）：第三轮之后不再开轮，这一轮新冒出来的零轮形态写进判决的交用户表。分工照那一节第三轮：

1. 「已定」项撑不撑得住：前两轮判了采、改完了的形态（第五批、第七批的改法，连同调度记录里经两轮加了限定的结论），今天还站不站得住。
2. 未定项清单的完整性：这次审计记下的延后项（`records/2026-09-16-subagent拆分提案.md` 节「九十、子 agent 落地审计里立了做法、还拦不住的四条与一条没想清的」① 到 ⑲、`.claude/kb/checks-owed.md` 的 C879、C880）是不是全了：前两轮判决、六份工具员交回报告、两份腿报告里点到「没做」「没查」「留给下一批」的，每一条有没有去向。
3. 报出的数字能不能核：调度记录与两份判决里引的数（结局账、返工字样、门槛换值、各条自证的种数、样本读数），照写明的命令能不能复出来。

不重复前两轮已攻过的角度：第一轮（改法做没做到缺口、合并与重合、新旧矛盾、记录里的数能不能复出来）、第二轮（第一轮的修补本身、产生结论的方法、两轮都没看过的地方、第一轮判决够不够得着）。

被判的文件：

| 类 | 文件 |
|---|---|
| 第七批改的 | `.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/doc-registries.sh`、`scripts/kb/quote-kb.py`、`scripts/kb/stale-candidates.py`、`.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`、`scripts/agents/agent-cost.py`、`.claude/agents/experiment-designer.md`、`.claude/agent-common.md`；新样本 `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-rekey-red/`、`.claude/gate.d/fixtures/doc-experiments.sh/decision-links-pending-base-unreadable-red/`、`.claude/gate.d/fixtures/doc-decisions.sh/evidence-ledger-base-unreadable-red/` |
| 两条腿都没读过的 | `.claude/agents/implementation-writer.md`（第五批第 4 步「命名检查除外」） |
| 前两轮站住的形态（照两份判决「这一轮被判过的文件」） | 第五批改的定义、钩子、门禁与脚本，今天的样子；`scripts/agents/subagent_transcript.py` |
| 记录与清单 | `records/2026-10-22-子agent结果落地审计.md`；`records/2026-09-16-subagent拆分提案.md` 第九十节；`.claude/kb/checks-owed.md` 的 C879、C880；归档目录 `research/prompts/subagent-landing-audit-2026-10-22/`（六份工具员交回报告、闭窗口结局账） |

## 二、实现今天的样子（主 agent 的观测，2026-10-23）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一批没有改 `crates/` 下的任何文件：`git diff --name-only` 列出的 `crates/` 文件是别的会话的改动，不在被判之列；门禁 doc-experiments 的 replay-comparison-units 格读 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 里 `formal_run_units` 数组的字符串字面量，只读不改。

- 第七批交回报告原样在 `research/prompts/subagent-landing-audit-2026-10-22/`：`r2fix-gates.md`、`r2fix-hooks.md`、`r2fix-definitions.md`。
- 主 agent 在主工作区重跑过的读数：`lib-lag-baseline.py --selftest` 9 项、`quote-kb.py --selftest`、`stale-candidates.py --selftest` 退 0；派发闸自证 460 种、交回闸 23 种、`agent-cost.py` 18 项、`dispatch-prompt.py` 90 项退 0；第一轮攻方腿与第五批钩子工具员两份真派发提示 `--check`：前一份退 0，后一份退 3 只剩认领行一处。
- 闭窗口结局账照东京日期切天重出：`research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-2026-10-01-to-2026-10-21.txt`（2125 个，三方腿 463 个全部报告在仓且同轮有判决、新桶 0 个）。
- 样本档 `stage-selftest.sh` 与弄坏开关的复跑格要样本与暂存区相同，这一批没暂存，在阶段里都没跑过；两个工具员在草稿里逐份核过。
- 第七批派发外的改动（主 agent 认下、被攻过零轮）：`lib-lag-baseline.py` 的 `git show` 改按当前目录取路径；`.claude/kb-directories` 登记写成 `.` 也拒；派发闸报告标签在句号、问号、叹号、分号之后的句首也认；报告行里紧跟否定词的文件名不判；`agent-cost.py --since` 只认 `YYYY-MM-DD`。
- 工作区里另有别的会话的未暂存改动（`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/hooks/lib_run_scope.py`、`.claude/hooks/run-scope-guard.sh` 上的几处），不在被判之列。

开工快照：`research/prompts/landing-audit-r3-snapshot/judged-sha256.txt`、`research/prompts/landing-audit-r3-snapshot/judged-vs-index.diff`。

## 三、格

### K1　未定项清单的完整性（辩方）

- 前两轮判决、六份工具员交回报告（`r1fix-*.md`、`r2fix-*.md`）、四份腿报告里每一处「没做」「没查」「留给下一批」「交主 agent」：逐条找它的去向（九十节第几行、C 号、调度记录哪一行、判决哪一条），找不到的列出来。
- 九十节 ① 到 ⑲ 每一行：形态写没写到能直接派（判据、样本、出路），有没有两行说的是同一件事、该并。
- 第二轮判决节「三、判决」交给主 agent 自己改的几处（调度记录六处限定、九十节 ⑥ 与 ⑪），是不是都改了、改得对不对得上判决的话。

### K2　「已定」项撑不撑得住（攻方）

- 第七批每一条改法：做没做到第二轮判决那一条要的；派发外的五处改动有没有带出新的误拦、误放（例：句首也认之后，正文里「。报告：」一类的转述会不会又被当报告行；`--since` 只认 `YYYY-MM-DD` 之后已有调用会不会断）。
- 前两轮站住的形态今天还站不站得住：第七批改了 `lib-lag-baseline.py`、派发闸、`agent-cost.py` 之后，第五批立的判法（三方腿报告形态、续做闸与看门狗的交回定义、replay-comparison-units 的形状判）有没有被改坏。
- `.claude/agents/implementation-writer.md` 第 4 步那一句与共用约束、钩子有没有矛盾。

### K3　报出的数能不能核（攻方）

- 调度记录节「结果」「第三批交回」「第一轮判决与第五批」「第二轮判决与第七批」与两份判决引的数：照写明的命令在副本上复出来，或指出哪一个复不出来、为什么。
- 重点：闭窗口结局账两版（系统时钟日期一版 2144 个、东京日期一版 2125 个）的差能不能由切天方式与三方桶、词表的改动解释全；门槛换值表能不能在新版上复出来。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 复核两份判决与清单：每一条延后有没有去向、交主 agent 的改没改 |
| 云端攻方（Opus） | K2、K3 | 攻已定的形态今天站不站得住，复核报出的数 |
