# 定义收尾第三轮判决（defs-m2-closeout-r3，2026-09-26）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 K1 K2 K3 R1 R2 R3 R4 R5 R6 R7 R8 V1 V2 V3 V4 V5 V6 V7 -->

## 一、这一轮

- 正文 `research/prompts/_defs-m2-closeout-r3-body.md`，背景材料 `_defs-m2-closeout-r3-background.md`，附录二 `_defs-m2-closeout-r3-diff.md`，开工快照 `research/prompts/defs-m2-closeout-r3-snapshot/sha256sums.txt`（派核查员之前主 agent `sha256sum -c` 27 个全 OK）。
- 腿：云端正推 `defs-m2-closeout-r3-sonnet-output.md`（探针 `defs-m2-closeout-r3-sonnet-model/`）；云端攻方 `defs-m2-closeout-r3-opus-output.md`（探针 `defs-m2-closeout-r3-opus-model/`）；本地攻方 `defs-m2-closeout-r3-local-attack-output-s1.md`、`-s2.md` 两份干净（`-void1.md` 被损坏闸判红作废：模型引了带 `**` 的通配路径，闸认成星号落单）。
- 核查员 `defs-m2-closeout-r3-verifier-output.md`：79 处 ✓75 ✗2 分不清 2 核不动 1。两处 ✗ 都在正推腿：`.claude/gate.d/40-results-cited.sh:19` 应为第 33 行（它自己的 `rerun.sh` 输出就是 33），引第二轮判决第 22 行漏抄一个「号」字；都不改它的判定。分不清两处：正推引修定义的 agent 报告第 639 行（全文 495 行，原句在第 222 行，推的是抄错）；攻方引 `research/scripts/memory-peaks.tsv:910`，那一行今天在第 919 行，文件在攻方交回之后被别的会话追加过。两条云端腿的 `rerun.sh` 复跑都逐字节一致。

## 二、逐格判

| 攻方编号 | 被攻的改法 | 判 | 改法 |
|---|---|---|---|
| R1 | G1（15、74 号在阶段里经内存包装） | **打中（量过）**：上限写成 `12GiB`、`16GB`、`8g` 时包装退 2（用法错），而包装文件头把 0–249 算作命令自己的退出码（`research/scripts/run-with-memory-cap.sh` 第 18 行与第 27 行重叠），15 号报「cargo test 退出码 2」还吞掉包装的报错，74 号报「测试二进制判红」 | V1 |
| R2 | G1 × G9（「给一个数」） | **打中（量过）**：照 `.claude/main-agent.md` 的字面写不带单位的 `16`，包装的写法正则单位可选，按 16 字节起 scope 退 251，两处出路都不指向真原因 | V1 |
| R3 | G1 × 分诊员第 3 步 | 打中（推的）：15、74 号撞上限退 250 或排不上队退 252 时 ✗ 行点名包装文件，分诊员按「点名的文件在不在暂存区 diff 里」判归属，没有一格对应包装自己的结局 | V2 |
| R4 | G2 × 40 号 | **打中（量过）**：新文件名写成日期、`-r2`、`.r12` 的 40 号判红要实验页点名；写成 `.r2` 的 40 号当中间件静默跳过，而 `replay.sh` 正指着它；逐字节一致又不写页时没有一步能让 40 号变绿。正推腿独立报了同一格 | V3 |
| R5 | G6 × 检出 hook 的出路 | **打中（量过）**：照 `.claude/hooks/bash-command-detector.sh` ④ 的出路改成 `a & b & wait`，hook 放行，两件分别退 3、退 0 时 `$?` 为 0 | V4 |
| R6 | G9 | 小（推的）：峰值表里 54、57 号 0 行、没有线程数一列；54 号 `--full` 的键带临时路径 | V5 |
| R7 | G1 | 小：15、74 号出路漏了退出码 253 | V6 |
| R8 | G3 | 小：「这一轮全部原型跑的全量」没说由谁合计，接手的新腿没有账可接 | V7 |
| K2 正推 | G2、G5、G7 | G2、G5 改后字面逐字对上第二轮判决；G5 在今天 26 条判决行上没有落错边的（攻方 355 处取值独立核过，同判）；G7 十道有依据、没漏（正推用同一条 `awk` 现算 14 道全集逐一核过）。两处做过头：十道里 27、69、88 是宽扫描，新实验页不在时只少扫、不判红，修定义的 agent 报告用「空判」一句概括七道不准确；「读实验页」扩成「读实验页或实验索引」超出第二轮原文。判：两处都留，十道挪到第 7 步之后不改变结局，扩到索引的依据（索引与正文同在第 7 步才写）站得住 | — |
| K2 本地 | G7 的字面依据 | 两份样本一致：十道各自点的依据行都只是定义路径、列目录或判文件在不在，没有一行本身在读实验页；十道都登记给了执行员。判：修定义的 agent 给的证据行偏薄，正推腿往下核到了真读的那几行（84 号 `84-verdict-false-named.sh:73`、86 号 `86-experiment-orphans.sh:13` 等），结论不受影响 | — |
| — | G3、G4、G5、G7 挪后剩下的第 6 步、G8、重型闸对上限变量的 11 种写法、15 号的复用 | 没打中（攻方一次抽样；照「一条腿只抽一次样不算一次观测」只记一次，第三轮之后不再开轮） | — |
| — | 15 号的 8G 够不够 | 没量：`cargo test --no-run` 被重型闸拒，攻方没换写法绕 | 记进第四节 |

## 三、改法（第三轮里新冒出来的，被攻过零轮，不再开轮）

- **V1**（R1、R2）：包装的写法正则单位必写（正整数加 K / M / G / T），用法错改退 251（「命令一行都没跑」那一类），不再落在 0–249 里；15、74 号调包装之前先核上限写法，写错单独判红、出路指到变量名（攻方 `cap-syntax-fix.py` 在副本上量过的那一版）；`.claude/main-agent.md` 与 `.claude/agents/crash-verifier.md` 的「给一个数」写成「一个带单位的上限（例 16G）」。
- **V2**（R3）：门禁分诊员第 3 步加一格：✗ 行写着内存包装退 25x 的不按点名文件判，250 判「环境（上限）」、252 与 254 判「并发」，原样抄出路。
- **V3**（R4）：执行员第 4 步定新文件名的拼法 `<旧名去掉 .out>-<YYYY-MM-DD>.out`，同一天再跑加 `-rN`，不写 `.rN.out`；第 5 步改指 `replay.sh` 的同时，新文件要被实验页点名（逐字节一致也点一行），这一次不写页的写进报告交主 agent 派人点名。40 号跳过 `*.r[0-9].out` 之前先核它是不是 `replay.sh` 登记的产物，是就不跳。
- **V4**（R5）：检出 hook ④ 的出路改成共用约束 ④ 的收法：每件写退出码文件、文件名带批号件号、收的时候数件数。
- **V5**（R6）：崩溃验证员「输入」里「量过的」写明是同一线程数下量的；峰值表加线程数一列。
- **V6**（R7）：15、74 号出路补 253（限时：把 `RUN_WITH_MEMORY_CAP_TIME_LIMIT` 拿掉或放宽再跑）。
- **V7**（R8）：攻方第 3b 步写成「这一条腿与接手它的腿全部原型跑的全量」，接手的腿从交接摘要里取已跑的量。

**改的时候**：`research/scripts/run-with-memory-cap.sh` 与 `.claude/hooks/bash-command-detector.sh` 此刻正被在跑的腿与每一次工具调用读着，一律用 `research/scripts/replace-once.py` 换新 inode 改，不就地写（第二轮核查员坐实过一次包装脚本在跑着的时候被原地改、跑着的那一趟报语法错）。改完跑 47、62、63 号与 doc-lint、两份脚本的自证与 hook 自证。

## 四、延后与报给别处的

- 15 号的 8G 够不够没量：记进 `records/2026-09-16-subagent拆分提案.md` 第四十节，提交那一轮门禁分诊员跑 15 号时看峰值。
- 门禁 40 号今天在工作区上红 7 份（E156、E158 的产物没被实验页点名，攻方副本上的基线）：归 E156、E158 的重跑登记（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实四甲交回」那一行）。
- 第二轮延后的上游 `check.sh` 那一件照旧（提案记录第四十节第 45 行）。

## 五、交用户的

| 项 | 状态 |
|---|---|
| V1–V7 | 零轮、不再开轮，主 agent 照 CLAUDE.md「定义、共用约束、三方流程与配套脚本用着发现问题就直接改」直接改；用户要看或要撤，照这张表逐条点 |

## 六、按路径点名被判的定义与文件（门禁 72 号）

这一轮被判的：`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-local-defense.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/hooks/ask-user-claim-guard.sh`、`.claude/gate.d/74-model-differential.sh`、`.claude/gate.d/15-research-build.sh`、`.claude/gate.d/stage-inputs.tsv`。V1–V7 要改的：`research/scripts/run-with-memory-cap.sh`、`.claude/gate.d/15-research-build.sh`、`.claude/gate.d/74-model-differential.sh`、`.claude/main-agent.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/agents/experiment-runner.md`、`.claude/gate.d/40-results-cited.sh`、`.claude/hooks/bash-command-detector.sh`、`.claude/agents/three-way-attack.md`。

## 回看决策

不涉及决策：被判的是 agent 定义、共用约束、主 agent 定义、hook、门禁与研究脚本，不改 `.claude/kb/decisions/` 里任何一条决策。
