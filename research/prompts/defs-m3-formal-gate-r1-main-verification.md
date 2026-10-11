# 第五道重型门禁写进规则与定义：定义三方第一轮判决（2026-10-01）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 A1 A2 A3 A4 A5 B1 B2 B3 C1 C2 C3 C4 F1 F2 H1 H2 H3 H4 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 被判的 hunk | `research/prompts/_defs-m3-formal-gate-r1-body.md`、`_defs-m3-formal-gate-r1-background.md`、`_defs-m3-formal-gate-r1-diff.md`（11 份文件 25 个 hunk） | — |
| 开工快照 | `research/prompts/defs-m3-formal-gate-r1-snapshot/sha256sums.txt`（17 份） | 腿交齐之后 `sha256sum -c` 全 OK |
| 云端正推（Sonnet） | `research/prompts/defs-m3-formal-gate-r1-sonnet-output.md`（217 行，sha256 01c9d2da…a79b）；模型 `…-sonnet-model/` | 探针只喂 JSON 给钩子、看退出码；没复跑（攻方 A1 与它的 D1-19 到 D1-22 是两份互不共享代码的探针，结论同向） |
| 云端攻方（Opus） | `research/prompts/defs-m3-formal-gate-r1-opus-output.md`（253 行，sha256 9d6785cc…3269）；模型 `…-opus-model/` | 主 agent 在 `/tmp/claude-1000/defs-m3-formal-gate-r1-main-check/` 照 `rerun.sh` 复跑：`d1-output.txt`、`d2-summary.txt` sha256 与入库那份相同；`d1b-output.txt`、`d2-*-output.txt` 只差时间戳与文件数（562 个文件那一列），去掉十六进制与路径之后 diff 只剩时间戳行 |
| 引文 | 两份腿报告 | `python3 scripts/agents/cite-check.py <两份报告> --root . --background research/prompts/_defs-m3-formal-gate-r1-background.md`：「✓ 核了 109 处引文，对上 109 处，没判 0 处」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| D1 谁跑 | 25 格探针 0 格与字面不符；`verus-stage` 与 `lib_heavy_tests.py:132、152` 对得上；缺口六处：整轮 `gate.sh` 里复用不作数时真跑（`main-agent.md:66`「只核标记」不成立）、`crash-verifier.md:31` 第 2 步后台模板没带两个上限、第 6 步指纹不含这一道的输入、判法脚本直接起闸全放行、没有 required-inputs、`gate-triage.md:28` 措辞 | A1 打中：`verus-proof-check.py`、`e165-pin-red-examples.py --replay` 直接起闸全放行（量过）；A2 打中：`research/results/` 有未跟踪文件时第 1 步两条核对过、写的标记在 `--staged` worktree 里对不上，整轮真跑（量过，与 QEMU 那一道同形）；A3 一次观测、复现不出；A4 `--tool-versions` 只打版本也被当重型拒；A5 没打中 | **打中，两条腿同向**（判法脚本绕闸两条腿各自量到）。改：`lib_heavy_tests.py` 把这两份判法脚本的干活用法判成 `verus-stage`、`--tool-versions` 不算重型；`main-agent.md:66` 那句改成写实（复用不作数时整轮里真跑），崩溃验证员第 1 步对 `research/results/` 的未跟踪文件怎么办写明；`crash-verifier.md` 第 2 步模板补两个上限、第 6 步指纹补这一道的输入。A3 单次观测，不采、记账 |
| D2 复用 | 只记不判 | B1 打中：全绿标记作数时换了 Verus（version、commit、sha256 全变）三条问法都复用（量过，把 4-5 推的那句坐实）；B2：按流程 `environment=` 一次都不会被记下；B3：就算比，只哈希 `verus` 外壳，换 z3 或 `VERUS_Z3_PATH` 照样复用 | **打中，主 agent 复跑坐实**（`d2-summary.txt` sha256 相同，「H1 换版本 … 三条路里复用 3/3」）。改法取攻方 F1（`environment=` 进全绿标记的指纹）加 F2（`--tool-versions` 加哈希 `rust_verify`、z3、vstd 与 `VERUS_Z3_PATH`）：它在草稿拷贝上量到换 Verus 的各臂 0/3 复用、没换的仍复用。**零轮**；改的是 `scripts/gate/admission.py` 与门禁的 `--tool-versions`，不在被判的 11 份里 |
| D3 并行与内存 | 没做 | C1 打中：内存包装在格里退 250–254 时被报成这一格红、出路指向改变异行或修实现，崩溃验证员与门禁分诊的定义都没接住；C2、C3 没打中（推的）；C4 轻：装置上限写不成「量过的」 | **C1 采**：判法里把包装自己的 250–254 判成「没跑成」，出路照包装文件头；两份定义写明怎么接。C2、C3 推的、没打中，留着。C4 记账 |
| D4 清单一致 | 漏 5 处：`agent-common.md:54`、`gate-triage.md:27、29` 的内存包装分类、`heavy-test-guard.sh:1286` 与 `:606`、文件外 `implementation-writer.md:58`、`crash-test/SKILL.md:17` 的「四样」表等 | 没判 | **打中**，逐处补上。文件外的 `admission.py:70、:1124`、`stage-must-run.sh:17`、`code-tooling.sh:669`、`README.md:126` 一并核 |
| D5 规则写法 | `implementation-workflow.md:74` 放进了「测试与崩溃检测优先多线程」小节、与 :32、:53、`verification.md:98` 重复；两处「在今天的实现上」是相对时间词 | 没判 | **打中**，挪回该在的小节、删重复、去掉「今天」 |

## 三、下一步

1. 改法并成一件工具活（第 4 批 3-11，tooling-writer）：第二节「判决」列 D1、D2、D3、D4、D5 的改法全部，文件按格列；与别的会话正在改的钩子与共用库先对过。
2. 改完开第二轮，只攻这一轮的改法本身（F1、F2 与新写的句子都是零轮）。

## 回看决策

不涉及决策：这一轮只判规则、定义、钩子与门禁复用的写法，没有一条推翻或确立决策分项。
