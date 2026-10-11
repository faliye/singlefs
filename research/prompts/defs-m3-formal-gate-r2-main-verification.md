# 第五道重型门禁写进规则与定义：定义三方第二轮判决（2026-10-01）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 E1 E2 E3 E4 F1 F2 G9 G10 G11 J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 Y1 Y2 Y3 V1 V2 V3 V4 V5 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 被判的 hunk | `research/prompts/_defs-m3-formal-gate-r2-body.md`、`_defs-m3-formal-gate-r2-background.md`、`_defs-m3-formal-gate-r2-diff.md`（54 个 hunk） | — |
| 开工快照 | `research/prompts/defs-m3-formal-gate-r2-snapshot/sha256sums.txt`（22 份） | 交齐之后 18 份 OK；对不上的 4 份都是别的会话改的：`.claude/rules/verification.md` 未暂存 3 行（快档与崩溃放量那两行、「门禁管哪一半」表第一行），不在被判的行上；`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/gate.d/stage-inputs.tsv` 是别的会话暂存的改动。攻方的探针跑在与快照 22 份逐份相同的拷贝上；辩方读主仓，引文行号交回时现查 |
| 云端辩方（Sonnet） | `research/prompts/defs-m3-formal-gate-r2-sonnet-output.md`（202 行，sha256 c7287385…364d）；模型 `…-sonnet-model/` | 没复跑：它的 E3-c 与攻方 J10 是两份互不共享代码的探针，结论同向 |
| 云端攻方（Opus） | `research/prompts/defs-m3-formal-gate-r2-opus-output.md`（215 行，sha256 66a9ad84…b1c9）；模型 `…-opus-model/` | 主 agent 在 `/tmp/claude-1000/defs-m3-formal-gate-r2-main-check/` 照 `rerun.sh` 复跑：`e1-output.txt`、`e1b-output.txt`、`e1-G9-output.txt`、`e1b-G9-output.txt`、`e2-lkmm-hook-output.txt`、`e3-mixed-output.txt` 与入库的逐字节相同；`e2-reuse-output.txt`、`e2-reuse-G10-output.txt` 只差输入文件数（566 → 576，期间别的会话往仓里加了文件），`e2-verus-toy-output.txt` 只差 Verus JSON 的键序；复用结论行原样：「Y1 未设 → 空串（Verus 起不了 z3）… q1 … 退 0（0 = 复用）… q3 … 退 10（10 = 复用）」，Y2（宏库截断）、Y3（裸名 z3 换 PATH 里那一份）同形，对照「版本变」两条问法都不复用；玩具 Verus 退出码 V1 0、V2 128、V3 0、V4 0、V5 1 |
| 引文 | 两份腿报告 | `python3 scripts/cite-check.py <两份报告> --root . --background research/prompts/_defs-m3-formal-gate-r2-background.md`：「核了 106 处引文，1 处对不上」——辩方第 51 行引 `.claude/singlefs-ai-sop/scripts/gate.sh:666`，原文在第 668 行；那句原文在，行号差两行，结论照采 |

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| 第一轮 D1–D5 复核 | 五格都站得住；两处同形的别的臂没覆盖（QEMU 第 1 步不查 `research/results`；QEMU 没登记 `environment=`），记账不推翻 | — | **第一轮判决维持**。QEMU 两处记账 |
| E1 判法脚本干活与只读的分界 | — | J1 打中（量过）：`PYTHONPATH=… python3 -m`、`python3 -c runpy`、符号链接、子命令放变量或命令替换、shell 函数、for 循环逐格跑，闸照放；同样写法起广谱变异驱动闸都拒，只有这一道漏（`verus_judge_use` 认不出干活就放）。J2 打中不分辨臂：`env -C`、脚本目录放变量、`script -qc`、`xargs` 也漏，但广谱变异同样漏，病根在 `lib_shell_words.py`。J3 没打中 | **J1 打中，主 agent 复跑坐实**（`e1-output.txt` 与 `e1-G9-output.txt` 逐字节相同）。改法取攻方 G9：先认出是这两份判法脚本、再一律算重型，只放过字面写明的只读用法，认法照广谱变异那一份（攻方在副本上量到 J1 的 10 种写法全改判拒、只读用法仍放）。**零轮**。J2 不归这一道：并进共用库盲区那一批（`records/2026-09-16-subagent拆分提案.md` 第四十三节第 1 行） |
| E2 F1 与 F2 | — | J4 没打中：F1 之后指纹连算三次再在 worktree 里算一次，四次逐字相同。J5、J6、J7 打中（量过）：只截坏安装里的 `libverus_builtin_macros.so`、`VERUS_Z3_PATH` 未设与空串、裸名 `z3` 换 PATH 里那一份，三种都复用。J8 轻：F1 之后 lkmm 的标记查询在里面起 `lkmm.sh --herd7-version`，直接起这条命令闸会拒 | **J5、J6、J7 打中，主 agent 复跑坐实**（第一节最后两行）。改法取攻方 G10：`--tool-versions` 另哈希安装目录顶层的全部文件，`VERUS_Z3_PATH` 分开未设与空串，裸名照 PATH 找到再哈希。**零轮**。J8 记账：闸那一侧把 `lkmm.sh --herd7-version` 当只读放行，并进同一件 |
| E3 包装 250–254 判没跑成、253 判红 | E3-a、E3-b 站得住；E3-c 同一格里没跑成与真红可以并存（空真格 15 条 ✗ 里 8 没跑成、7 真红），`crash-verifier.md` 第 3 步按格记会盖掉真红；`gate-triage.md:29`「251 与 253 ⇒ 环境」对 Verus 253 是死文字；E3-d 外部杀进程（137 等）不在 250–254 里，E165 一侧仍写「修实现回归」；E3-e 排队等待推的 | J9、J12 没打中；J10 打中（量过）：一处包装退 252、另一处真红时判法打两条 ✗，崩溃验证员第 3 步照字面记整格没跑成、门禁分诊按 252 归并发，都盖住真红；J11 轻：Verus 一侧吞了包装自己的 ✗ 与 →，252 两种成因出路只写一种 | **两条腿独立同向打中同一处**（辩方 E3-c 与攻方 J10）。改法取攻方 G11：定义改成一格里每一条 ✗ 都是没跑成才记没跑成，有一条真红就记红。`gate-triage.md` 那一句写成对 Verus 253 按超时判红；E165 一侧外部杀进程的出路改成照包装文件头；Verus 一侧把包装的 ✗ 与 → 原样带出。**零轮** |
| E4 写实句与清单 | E4-a、E4-b、E4-c、E4-e 站得住（第 1 步会因别的会话的未跟踪产物停，不是每次都停，频率没量）；E4-d 同族清单还有四处注释没补（`scripts/admission.py` 第 228、2763、2929 行、`heavy-test-guard.sh` 提示文字）；E4-a 三处小毛病（「它不直接调」指代含糊、漏了 24 小时复用上限、`heavy-test-guard.sh` 提示还留「靠复用」） | — | **E4-d 与 E4-a 三处采**，逐处补。E4-b 记账：第 1 步因别的会话的未跟踪产物停的频率，等几次提交的实况再判 |

## 三、下一步

1. 改法并成一件工具活（第 4 批 3-12，tooling-writer）：G9、G10、G11 与 E3、E4 采的几处；J8 一并。
2. 改完开第三轮（最终轮），只攻这一轮的改法；第三轮之后不再开轮，新冒出来的写进交用户的表、标零轮。

不涉及决策：这一轮只判规则、定义、钩子与门禁复用的写法，没有一条推翻或确立决策分项。
