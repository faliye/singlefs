# 里程碑二收尾：定义改动一轮三方，第二轮正文（2026-09-26）

<!-- doc-lint:not-numbers D1 D2 D3 D4 O1 O2 O3 O4 O5 O6 O7 O8 O9 O10 O11 O12 O13 O14 O15 O16 O17 O18 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 E1 E2 E3 -->

## 一、这一轮要判什么

第二轮。第一轮判决 `research/prompts/defs-m2-closeout-r1-main-verification.md` 整份进材料；它第三节的改法 F1–F16 已由修定义的 agent 改进文件（改法本身被攻过零轮），这一轮攻改后的字面，攻击面不重复第一轮（第一轮攻的是改之前那一版的矛盾与漏写）。

| 格 | 被攻的 | 问题 |
|---|---|---|
| E1 | **F1–F16 的改后字面**（附录二的 diff，比的是改之前的备份） | 每一条照字面做：会不会做错、做不了、与另一处打架；尤其是 F1 的并行收退出码写法在前台与 run_in_background 里各是什么结局，F4「原型里调枚举函数跑小流不算重型」的界能不能被照字面越过，F7「与旧的逐字节一致就删掉这一次的新文件」会不会删到不该删的，F9 `gate.sh` 整条经内存包装在 scope 里有没有别的影响，F13 弹窗闸放宽之后会不会放过一个没有出处的断言，F14 那张清单与 `.claude/hooks/lib_heavy_tests.py` 的 `classify`、`cargo_use` 逐类对不对得上 |
| E2 | **修定义的 agent 自己报的线索**（都是推的） | 40、86 号也登记给执行员，第 6 步对还没写实验页的新实验会不会与 84 号同样过早判红；F8 给 55 号的下限只算了客机内存，没算 qemu 进程与宿主侧；74 号写死的 8G 没实跑过；本地辩方样本文件名写法与攻方不同 |
| E3 | **第一轮判决复核**（辩方） | 第一轮判「部分」「小」的几格（O3、O8、O9、O11、O12）判得够不够；第四节延后的两件（D4 重型闸静态分支与版本号白名单、84 号只认 false 不认违例计数）延后之后，里程碑二提交时有没有哪一步因此做不了或被拒 |

**共用问句**：照改后的字面干活，哪一步会做错或做不了；举出具体的派发情形或命令。

## 二、实现今天的样子（主 agent 的观测，2026-09-26）

- 被判的改动：修定义的 agent 报告 `research/prompts/defs-closeout-r1-fixes-tmp-evidence/report.md` 第一节逐条表，与它的 `my-changes-final.diff`（287 行，比的是开工时 `cp -p` 的备份）；材料员把 diff 原样放进附录二，把报告第一至六节抄进附录。开工快照 `research/prompts/defs-m2-closeout-r2-snapshot/sha256sums.txt`。
- 这一轮新被改的文件，门禁 72 号管的：`.claude/agents/three-way-attack.md`、`three-way-local-defense.md`、`experiment-runner.md`、`crash-verifier.md`、`gate-triage.md`、`implementation-writer.md`、`mutation-triage.md`，`.claude/agent-common.md`，`.claude/main-agent.md`；另外改了 `.claude/rules/implementation-workflow.md`（重型清单）、`.claude/hooks/ask-user-claim-guard.sh`（F13，自证 23 → 30 种）、`.claude/gate.d/74-model-differential.sh`（F10，只改建议命令）。
- 探针：修定义的 agent 的探针用例与输出在 `/tmp/claude-1000/defs-closeout-r1-fixes/probe/`；第一轮攻方的 `research/prompts/defs-m2-closeout-r1-opus-model/probe.py` 可以接着用。
- 这一轮不碰 `crates/`：被判的都是定义、规则、hook 与门禁的建议命令。

## 三、条款（材料员整段抄进附录）

- 第一轮背景材料 `research/prompts/_defs-m2-closeout-r1-background.md` 的条款照旧；
- 第一轮判决整份；
- `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」改后那一节整节。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | E3 | 替第一轮判「部分」「小」与延后的几格辩：该不该升成打中、延后的去向够不够得着；并指出第一轮判决里哪一句够不着它引的证据 |
| 云端攻方（Opus） | E1、E2 | 造派发情形与命令攻改后的字面；要喂 hook 的只喂 JSON、只看退出码，被判的命令一条都不执行 |
| 本地攻方 | E1 的 F14 | 按事实表逐格填：`implementation-workflow.md` 改后重型清单的每一类，在 `.claude/hooks/lib_heavy_tests.py` 的 `classify` / `cargo_use` 里是哪一个判法、修定义的 agent 的探针 K1–K29 里哪一格喂过、退出码是几；每行写来源文件与行号，每张表不超过 6 行 |

两条攻方腿不重叠：Opus 造情形与命令攻语义，本地只逐格对清单与判法的字面。

## 五、交付

- 腿的报告写 `research/prompts/defs-m2-closeout-r2-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 探针放 `research/prompts/defs-m2-closeout-r2-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引文件写那份文件自己的行号，引的行是那句原文自己所在的行，不是它上面的标题行。
- 不跑重型测试；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知，不用 `true` 或 `sleep` 空转；交回之前后台不许留着跑的东西。
