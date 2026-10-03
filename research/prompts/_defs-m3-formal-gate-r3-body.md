# 第五道重型门禁写进规则与定义：定义三方第三轮正文（2026-10-01）

<!-- doc-lint:not-numbers G9 G10 G11 J1 J2 J8 Y1 Y2 Y3 R1 R2 R3 R4 L1 L2 L3 L4 -->

## 一、这一轮要判什么

第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停）。第二轮判决 `research/prompts/defs-m3-formal-gate-r2-main-verification.md` 整份进材料；它第二节采的改法 G9、G10、G11、J8 与 E4 几处已由工具实现员 3-12 落进文件（报告 `/tmp/claude-1000/m3-formal-defs-fix-3-12/report.md`，改前改后原文逐处列了），被攻过零轮。这一轮只攻这些改后的字面与代码，不重复前两轮的角度（第一轮：谁跑、复用、并行与内存、清单一致、规则写法；第二轮：判法脚本干活与只读的分界、F1 与 F2、包装结局、写实句与清单）。第三轮里新冒出来的改法不再为它开一轮，写进判决的交用户表、标「零轮」。

被判的文件（相对第二轮开工快照的改动，3-12 报告逐处列了；同文件里别的会话没提交的改动不判）：`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/main-agent.md`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`research/scripts/verus-proof-check.py`、`research/scripts/e165-pin-red-examples.py`、`research/scripts/admission.py`（只注释）、`.claude/gate.d/checker-tier-formal-verification.sh`（只文件头）。开工快照 `research/prompts/defs-m3-formal-gate-r3-snapshot/sha256sums.txt`。

| 格 | 被攻的 | 问题 |
|---|---|---|
| L1 | G9：先认出是两份判法脚本、再一律算重型，只放字面写明的只读用法 | 「只读用法」的名单里有没有一种其实干活；认出判法脚本的条件（名字、位置、PYTHONPATH、符号链接、`python -c` 点名）会不会误拒别的研究脚本（名字相近、同目录） |
| L2 | G10：`--tool-versions` 哈希安装目录顶层全部文件、`VERUS_Z3_PATH` 分未设与空串、裸名照 PATH 找 | 顶层之外（子目录）有没有 Verus 会读的；整体摘要里混进了会变的东西（日志、缓存、时间戳）没有——那样永远复用不上 |
| L3 | G11：一格里每条 ✗ 都是没跑成才记没跑成；判法混合格另打「按红记」；带出包装的 ✗ 与 →；252 两种成因 | 崩溃验证员、门禁分诊照改后的字面，在「全是没跑成」「混合」「全是真红」三种格上各记什么、下一步做什么，有没有一种走不通；`gate.sh` 汇总里这一道退 1 时的说法与定义对得上吗 |
| L4 | 前两轮「已定」的几格撑不撑得住；第二轮记账的几笔（J2、QEMU 两处、第 1 步停下的频率）够不够；第二轮报出的数能不能核 | 照 `.claude/rules/three-way-inference.md` 第三轮的分工 |

**共用问句**：照改后的字面干活，哪一步会做错或做不了；举出具体的派发情形或命令。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- 3-12 跑过的：攻方第二轮 `e1-hook-cases.py`（J1 那 10 种写法两个身份 20 条全判拒、只读 8 条全放，输出与攻方 `e1-G9-output.txt` 逐行相同）、`e2-reuse.sh`（Y1–Y3 不复用、对照照旧）、`e3-mixed.py`（混合格多出「按红记」一行）；`--selftest`、lint、真 Verus 都没跑（归提交时）。
- 第二轮两条腿的模型 `research/prompts/defs-m3-formal-gate-r2-opus-model/`、`…-sonnet-model/` 可以照用。
- `crates/` 这一轮不判。

## 三、条款（材料员整段抄进附录）

- 第二轮判决整份；
- 被判文件里 3-12 改过的段落（以 3-12 报告的改前改后原文为准，腿到主仓现读）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | L3、L4 | 复核第二轮判决：采的改法是不是同样打中了被留下的形态；照改后的字面在三种格上走一遍崩溃验证员与门禁分诊 |
| 云端攻方（Opus） | L1、L2、L3 | 造写法与安装状态攻改后的代码：喂钩子只喂 JSON、看退出码；L2 在草稿目录的仓拷贝与安装拷贝上跑 `--tool-versions` 与 `admission.py` 的判定函数，不跑门禁 |

## 五、交付

- 腿的报告写 `research/prompts/defs-m3-formal-gate-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/defs-m3-formal-gate-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`（复跑挂钟写在 `rerun.sh` 文件头，超过 20 分钟的写明哪一步慢）；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引文件写那份文件自己的行号，行号去被引文件里现查，不从背景材料里数。
- 不跑重型测试，不跑 `--selftest`；只停自己起的进程。
