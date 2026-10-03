# 对接上游测试脚本规范这一批：三方第三轮正文（2026-10-01）

<!-- doc-lint:not-numbers V1 V2 V3 U1 U2 U3 U4 U1-1 U1-2 T1-3 T2-6 -->

## 一、这一轮要判什么

第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停；这一轮里新冒出来的零轮形态不再开第四轮，写进判决的交用户表、标「零轮」）。前两轮判决 `research/prompts/test-script-align-r1-main-verification.md`、`research/prompts/test-script-align-r2-main-verification.md` 整份进材料；前两轮四条腿的报告与模型目录可读（`research/prompts/test-script-align-r{1,2}-{sonnet,opus}-output.md`、`…-model/`）。不重复前两轮攻过的角度。调度与各件交回在 `records/2026-10-01-对接上游测试脚本规范.md`（腿只读）。

第二轮之后改过的（同文件里别的会话没提交的改动不判）：

- 钩子：`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/selftest-registry.tsv`（命令词通配按当前目录展开逐份判；花括号与参数通配展开后再判；不带 `--item` 的 stage-selftest.sh 也核本仓那一份；登记表头与 C613 对齐。报告 `/tmp/claude-1000/tooling-writer-hook-glob-swap/report.md`、`/tmp/claude-1000/tooling-writer-hook-brace/report.md`）。
- 新脚本 `research/scripts/hook-swap.py`（先验后换、换上后再验、反向定点替换回滚）与它挂进 `.claude/gate.d/code-tooling.sh` 的 `SELFTEST_RUNNERS` 那一行。
- 定义：`.claude/main-agent.md`「禁止」第一条（全量两样、路径清单、跑红或不跑照样可进并记一行）、`.claude/agents/tooling-writer.md` 第 3 步（改钩子只经 hook-swap.py 换上）。
- kb：`.claude/kb/decisions/13-验证路线.md` 已定项 9（全量两个所指、路径判法、上一次全量的记法、「准入条件」与「异步」合成一句、「欠」挂 C635）与变更史 D13（验证路线） 2026-10-01 块；`.claude/kb/checks-owed.md` 的 C20、C409、C604–C613、C620、C622–C627、C635（`/tmp/claude-1000/kb-scribe-r2-debts/report.md`）。

| 格 | 被攻的 | 问题 |
|---|---|---|
| V1 | 前两轮站住、已写成定案或已落地的：D13（验证路线） 已定项 9 的现文、`.claude/main-agent.md` 那一条、拦重型测试的钩子现在的放行面（通配、花括号、参数通配、`--item`、样本档、登记表按真实路径认）、`hook-swap.py` 的先验后换与回滚 | 照现文干，哪一步做不了、做错或被绕过；钩子里还有哪种 bash 会展开成重型、钩子却放行的写法（文件头「罩不住的」以外）；hook-swap.py 回滚失败或换上一半时会落到什么状态 |
| V2 | 未定项与欠账清单的完整性：C604–C613、C620、C622–C627、C635 以及写回员留成「未定」的三处（C623 棘轮靠常量还是现读、C625 文件头写法取哪种、C622 只给样本自证设的标记由谁设） | 这一批该记而没记的缺口（对照上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md` 逐节、对照本仓入口与门禁）；三处「未定」各自的候选、什么观测会让选择翻面；有没有两笔重了或互相矛盾 |
| V3 | 前两轮报出的数能不能核：两份判决与各件交回里写的自证格数、开关数、探针退出码、样本判对数、欠账里写的计数（例 C610 的 89 行 / 66 份、70 行 / 52 份） | 逐个拿命令复现；对不上的列出原数、复现数与命令 |

**共用问句**：照现文干活，哪一步会做错、做不了，或者让一个该红的东西绿着过去；举出具体的命令、输入或派发情形。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- `crates/` 这一轮不判：第 2 批四份 `crates/` 补丁仍在 `/tmp/claude-1000/impl-*/patch/`，等别的会话几轮代码三方判完再合入。
- 主 agent 2026-10-01 跑过：用 `research/prompts/test-script-align-r2-opus-model/hook-probe.py` 喂本仓钩子，`bash research/scripts/mutate.{sh,x} crates/mutations.tsv`、`bash .claude/gate.d/checker-tier-layer0-replay.{sh,} <根>`、`bash research/scripts/mutate.sh crates/mutation?.tsv` 以 implementation-writer 身份退 2，`ls research/scripts/{a,b}.sh`、`cat crates/*.toml` 退 0；第二轮攻方 `rerun.sh` 的通配六条退 2；`lib_heavy_tests.py --selftest` 464 种、`heavy-test-guard.sh --selftest` 980 种、`hook-swap.py --selftest` 9 种退 0。
- 已知、不归本批：`.claude/gate.d/code-tooling.sh` 第 383 行 `NOT_RUN_HERE` 里 `crash-amplification-two-hosts.sh` 那一行 HEAD 里就有，code-tooling 的 `green`、`coverage-claims-green` 两份样本因此判错；`.claude/gate.d/fixtures/` 下 7 个空目录（上游 0.0.63「脚本执行位」会判红）；`research/scripts/admission.py --selftest` 1 格红（暂存区别的改动新加的那一格）。上游 0.0.63 已发版、副本已同步。

## 三、条款（材料员整段抄进附录）

- 前两轮判决整份；
- 上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md` 全文；
- `.claude/kb/checks-owed.md` 的 C20、C409、C604–C613、C620、C622–C627、C635；
- `.claude/kb/decisions/13-验证路线.md` 已定项 9、15 两节。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | V2、V3 | 从上游规则逐节推本仓该有的检查，对照欠账清单找漏记；三处「未定」各写候选与翻面观测；逐个复现前两轮报出的数 |
| 云端攻方（Opus） | V1 | 造输入攻已落地的形态：喂钩子只喂 JSON、看退出码；hook-swap.py 在草稿目录的假钩子上造「换上一半」「回滚冲突」；照 D13 已定项 9 与 main-agent.md 的字面走一遍里程碑收尾，找做不了的一步 |

## 五、交付

- 腿的报告写 `research/prompts/test-script-align-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/test-script-align-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引文件写那份文件自己的行号，行号去被引文件里现查，不从背景材料里数。
- 不跑重型测试；登记为 light 的 `--selftest` 只在本仓路径上跑；不改主工作区；只停自己起的进程。
