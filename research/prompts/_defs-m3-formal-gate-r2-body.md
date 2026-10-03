# 第五道重型门禁写进规则与定义：定义三方第二轮正文（2026-10-01）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 E1 E2 E3 E4 F1 F2 -->

## 一、这一轮要判什么

第二轮。第一轮判决 `research/prompts/defs-m3-formal-gate-r1-main-verification.md` 整份进材料；它第二节的改法已由工具实现员 3-11 落进文件（报告 `/tmp/claude-1000/m3-formal-defs-fix-3-11/report.md`，「每处改动的前后原文」一节逐处列了），主 agent 另补五处（`.claude/skills/crash-test/SKILL.md` 第 3、17 行与表里 Verus 一行，`README.md` 第 126 行，`agent-common.md` 第 52 行、`implementation-workflow.md` 第 57 行、`verification.md` 第 106 行三处重型清单加「连同直接起它的判法脚本」）。这些改法都被攻过零轮，这一轮只攻它们本身，不重复第一轮的角度（谁跑、复用、并行与内存、清单一致、规则写法这五格第一轮各自的攻法）。

被判的 hunk：`research/prompts/_defs-m3-formal-gate-r2-diff.md`（相对 HEAD 抽出 54 个 hunk；`research/scripts/verus-proof-check.py`、`e165-pin-red-examples.py`、`.claude/gate.d/checker-tier-formal-verification.sh` 在 HEAD 上没有，整份进 diff）。同文件里别的会话没提交的改动（detached-run 那一套、`agent-common.md`「长活可以等」等）不判。

| 格 | 被攻的改法 | 问题 |
|---|---|---|
| E1 | D1：`lib_heavy_tests.py` 把两份判法脚本的「干活用法」判成 `verus-stage`、只读用法与 `--tool-versions` 不算 | 干活与只读的分界照两份脚本文件头划的，有没有一种只读用法其实起了 Verus 或编了 E165 装置、或一种干活用法换个写法（参数次序、`python3 -m`、符号链接、经别的包装）就被判成只读 |
| E2 | D2：F1（`environment=` 进全绿标记指纹，`research/scripts/admission.py` 的 `stage_fingerprint`）与 F2（`--tool-versions` 另哈希 rust_verify、z3、vstd.vir、libvstd.rlib 与 `VERUS_Z3_PATH`） | F1 动了每一道登记 `environment=` 的阶段（checker-tier-lkmm 与这一道）的标记：改后旧标记全部失效之外，有没有一道因此永远复用不上（环境输出每次都变，例如带时间、路径、进程号）；F2 哈希的五样够不够（Verus 还读别的东西吗）；`VERUS_Z3_PATH` 没设与设成空串是不是同一个指纹 |
| E3 | D3：两份判法把包装 250–254 判「没跑成」并退 1；253（脚本给包装的限时）照「超时不当通过」仍判红 | 「没跑成」与「红」在 crash-verifier、gate-triage 与 `gate.sh` 汇总里各怎么记，会不会被当成通过或被复用；253 一侧判红与 250、251、252、254 判没跑成，交给分诊时分得开吗 |
| E4 | D1 的写实句与 D4、D5 的清单与挪句 | `main-agent.md` 那句写实之后，crash-verifier 第 1 步「`research/results` 有未跟踪文件就不跑、不写标记、停下报主 agent」在别的会话常有未跟踪产物的仓里会不会每次提交都停；挪句与补清单之后有没有新的不一致 |

**共用问句**：照改后的字面干活，哪一步会做错或做不了；举出具体的派发情形或命令。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- 3-11 跑过的只有攻方第一轮的 22 条钩子 JSON（改后全判对）与 `d2-reuse.sh`（改后仓拷贝上换 Verus 的三臂 0/3、不换的 3/3），另有 D3 两份函数级探针；`--selftest`、样本、lint 都没跑（归提交时）。
- 第一轮攻方的模型 `research/prompts/defs-m3-formal-gate-r1-opus-model/`（`d1-hook-cases.py`、`d2-reuse.sh`、`rerun.sh`）可以照用。
- `crates/` 这一轮不判。

## 三、条款（材料员整段抄进附录）

- 第一轮判决整份；
- 被判的 hunk（`_defs-m3-formal-gate-r2-diff.md`）；
- 3-11 报告「每处改动的前后原文」一节。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | E3、E4，兼复核第一轮判决 | 复核第一轮判决五格够不够得着、改法是不是同样打中了被留下的形态；E4 照字面在今天这个仓里走一遍提交流程会停在哪 |
| 云端攻方（Opus） | E1、E2、E3 | 造用法与历史攻改法：喂钩子只喂 JSON、看退出码；E2 在草稿拷贝上跑 `admission.py` 的判定函数，不跑门禁 |

## 五、交付

- 腿的报告写 `research/prompts/defs-m3-formal-gate-r2-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/defs-m3-formal-gate-r2-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引文件写那份文件自己的行号，行号去被引文件里现查，不从背景材料里数。
- 不跑重型测试，不跑 `--selftest`；只停自己起的进程。
