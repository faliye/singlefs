# 对接上游测试脚本规范这一批：三方第二轮正文（2026-10-01）

<!-- doc-lint:not-numbers U1 U2 U3 U4 T1 T2 T3 T4 T2-1 T2-2 T2-3 T2-4 T2-5 T2-6 T3-1 T3-2 T3-3 T3-4 T3-5 T3-6 T3-7 T3-8 T3-9 T1-1 T1-2 T1-3 T1-5 -->

## 一、这一轮要判什么

第二轮。第一轮判决 `research/prompts/test-script-align-r1-main-verification.md` 整份进材料，第一轮两条腿的报告与模型目录（`research/prompts/test-script-align-r1-sonnet-output.md`、`…-opus-output.md`、`…-opus-model/`）可读。这一轮只攻第一轮判决采的改法与第一轮站住的形态，不重复第一轮攻过的角度（`.claude/rules/three-way-inference.md`「多轮」）。调度与各件交回在 `records/2026-10-01-对接上游测试脚本规范.md`「第 2 批第二波与三方」一节之后（腿只读）。

第一轮之后改过的文件（同文件里别的会话没提交的改动不判；归属认不出的照调度记录各件交回那几行）：

- 钩子：`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/selftest-registry.tsv`（T2-1、T2-3、T2-4、T2-6 收紧，再加 `stage-selftest.sh --item` 放行；报告 `/tmp/claude-1000/tooling-writer-hook-tighten/report.md`、`/tmp/claude-1000/tooling-writer-hook-item/report.md`）。
- 门禁：`.claude/gate.d/checker-tier-layer0-replay.sh` 与 `.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/`（样本档核假工具，`/tmp/claude-1000/tooling-writer-gate54-exec/report.md`）。
- 研究脚本：`research/scripts/crash-amplification.sh`、`crash-amplification-item.sh`（`/tmp/claude-1000/tooling-writer-crash-amp-r1fix/report.md`）；`research/scripts/mutate.sh`（`/tmp/claude-1000/tooling-writer-mutate-r1fix/report.md`、`/tmp/claude-1000/tooling-writer-mutate-parallel-reason/report.md`）。
- 规则与定义：`.claude/rules/verification.md`（`*_parallel_line_one_*`）、`.claude/agent-common.md`、`CLAUDE.md`、`.claude/agents/tooling-writer.md`（light / heavy / heavy-if-set / 没登记的统一说法）、`.claude/main-agent.md`（「禁止」第一条：里程碑收尾……先弹窗请用户跑一趟全量）。
- kb：`.claude/kb/checks-owed.md` 的 C20、C409、C604–C613、C620、C622–C627；`.claude/kb/decisions/13-验证路线.md` 已定项 9（「合并前」改成两个时点、依据加用户定案）与 `.claude/kb/decisions-history.md` 的 D13（验证路线） 2026-10-01 块（`/tmp/claude-1000/kb-scribe-r1-debts/report.md`）。

| 格 | 被攻的 | 问题 |
|---|---|---|
| U1 | 钩子收紧之后的放行面：样本档（标记逐字节核、样本目录与 HEAD 加暂存区逐字节相同）、登记表按真实路径认、`--item` 只判点名的几道、命令词带通配的不当自证（T2-6 的改法）、带 `--item` 时命令词须是本仓那一份 stage-selftest.sh | 收紧之后还有哪条子 agent 会写的命令能跑起真重活（尤其通配写法 `bash research/scripts/heavy*.sh --selftest` 一类、不带 `--item` 的 stage-selftest.sh 只按名字认）；有没有该放的被误拒（改钩子的人自己的自证、样本刚改没暂存时） |
| U2 | 入口与脚本的改后判法：崩溃放量入口（红优先于 78、驱动退 78 而行里报红记红、ignore 两个字段与成功句、缺 `red=` 判红、零态判红的例外、只供测试的变量平常跑退 2、缺值退 2）；`mutate.sh`（252「排不上」/ 254「被总上限挤掉」判失败、只供自证的变量平常跑退 2、平常跑给自己的工作进程设并行理由、基线撞 252 报排不上）；54 号样本档核假工具 | 改后哪一格又会假绿、假「本次未跑」、假红；新退出码与文件头表、与调用方（门禁 checker-tier-crates-mutation-replay、`crates-mutation-rows.py`、实验执行员）的读法对不对得上；平常跑给自己设并行理由会不会让「日常一条」失去意义 |
| U3 | 第一轮判决本身（辩方复核）：采的每一条改法是不是打中了被攻的那个形态、有没有同样打中别的写法；T2-5、T3-10、T3-11 只抽一次不作「没打中」用，判得对不对；T1-3 交用户之后写回的两个时点（`.claude/kb/decisions/13-验证路线.md` 已定项 9、`.claude/main-agent.md`「禁止」第一条、C20）照字面能不能执行 | 判决里哪一条去向判错了；写回的字面照着干会不会做错（例：「动了崩溃点重放或判器」怎么认、谁记「上一次全量」） |
| U4 | 欠账改写与新记（C604–C613、C620、C622–C627、C20、C409）与改钩子的流程（不开草稿副本的自证口子：草稿里语法检查加只调判定函数的探针，换上后立即跑自证，红了马上换回原样） | 每一笔改写之后的「怎么拦」能不能落成会红的检查；新记几笔有没有和已有的重了；改钩子那条流程会不会让一次坏的换上把整个会话的派发都拦死、有没有更稳的做法 |

**共用问句**：照改后的字面干活，哪一步会做错、做不了，或者让一个该红的东西绿着过去；举出具体的命令、输入或派发情形。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- `crates/` 这一轮不判：第 2 批四份 `crates/` 补丁仍在 `/tmp/claude-1000/impl-*/patch/`，等别的会话几轮代码三方判决写完再合入。驱动行新字段按 `/tmp/claude-1000/impl-crash-amp-lines/report.md` 第 1 条的约定判。
- 主 agent 2026-10-01 跑过：`lib_heavy_tests.py --selftest`（393 种）、`heavy-test-guard.sh --selftest`（932 种，该拒 175 种）退 0；`stage-selftest.sh "$PWD/.claude/gate.d" --item checker-tier-layer0-replay.sh`「点名的 1 道阶段判得都对（8 个样本）」；doc-lint 对本批改的欠账行与 D13 零命中；doc-decisions 17 格里三格红在早有的地方（kb-shape、item-ref-status、history-loss），不是本批的。各件自证由各自的 tooling-writer 跑（报告里有原样输出）。
- 第一轮的攻方模型 `research/prompts/test-script-align-r1-opus-model/` 可以照用（`rerun.sh`）。

## 三、条款（材料员整段抄进附录）

- 第一轮判决整份；
- 上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md` 全文；
- `.claude/kb/checks-owed.md` 的 C20、C409、C604–C613、C620、C622–C627；
- `.claude/kb/decisions/13-验证路线.md` 已定项 9、15 两节。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | U3、U4 | 复核第一轮判决：采的改法是不是同样打中了被留下的形态，去向判得对不对；照写回后的字面走一遍主 agent 在里程碑收尾时该做什么、欠账每一笔怎么落成检查 |
| 云端攻方（Opus） | U1、U2 | 造输入攻改后的钩子与脚本：喂钩子只喂 JSON、看退出码；脚本在草稿目录的仓拷贝上用假驱动、假包装、假 cargo、假时钟跑；不重复第一轮的 T2、T3 探针，找收紧之后新开的口子与新的假绿 |

## 五、交付

- 腿的报告写 `research/prompts/test-script-align-r2-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/test-script-align-r2-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引文件写那份文件自己的行号，行号去被引文件里现查，不从背景材料里数。
- 不跑重型测试；登记为 light 的 `--selftest` 只在本仓路径上跑（钩子只认本仓那一份），不改主工作区；只停自己起的进程。
