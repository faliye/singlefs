# 里程碑三形式化验证：第五道重型门禁写进规则与定义，一轮三方正文（2026-10-01）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 -->

## 一、这一轮要判什么

用户 2026-10-01 定：装了工具的形式化验证进提交门禁、每次提交代码都验，第五道重型门禁（`.claude/gate.d/checker-tier-formal-verification.sh`）写进规则与定义、走一轮三方（`records/2026-10-01-里程碑三形式化接入.md` 第 199 行 ②）。第一轮。被判的是第 4 批 4-5、3-8、3-9 与主 agent 对下面 11 份文件的改动，只判点名这一道门禁的那些 hunk（`research/prompts/_defs-m3-formal-gate-r1-diff.md`，相对 HEAD 抽出 25 个 hunk；同文件里别的会话没提交的改动不在这一轮）：

`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/agents/crash-verifier.md`、`.claude/main-agent.md`、`.claude/agents/gate-triage.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/investigator.md`、`.claude/skills/crash-test/SKILL.md`、`.claude/agent-common.md`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/runner-dispatch-guard.sh`。

| 格 | 问题 |
|---|---|
| D1 | 谁跑、什么时候跑：照改后的字面，提交时这一道由谁带 `SINGLEFS_HEAVY_TESTS=commit` 起、整轮 `gate.sh` 里它怎么走（只核标记与复用判定，还是真跑）、单跑一格（`--check verus-proof`）谁跑得了；`heavy-test-guard.sh` 给 crash-verifier 加的 `verus-stage` 与 `lib_heavy_tests.py` 第 132、152 行的判定对不对得上；有没有一条派发情形会被闸拒、或者绕过闸跑重型 |
| D2 | 复用：`.claude/gate.d/stage-inputs.tsv` 这一道那一行登记的输入与 `environment=`（Verus 版本、提交号、可执行文件指纹）够不够；`research/scripts/admission.py` 的 `gate_reuse` 只要全绿标记作数就不再比 `environment=`，24 小时内换了 Verus、输入没变时会复用旧判定（4-5 读代码推的，没验） |
| D3 | 并行与内存上限：改后的 `crash-verifier.md` 写这一道与 54 号、QEMU、herd7 并行、两个上限（`GATE_VERUS_MEMORY_MAX`、`GATE_E165_PINNED_MEMORY_MAX`）、阶段里面已经包了内存不在外面再包；照这道门禁实际读写的东西（Verus 直接起不经 cargo、E165 装置编在 `singlefs-e165-pinned-target`），并行会不会抢同一个编译目录或锁、600 秒限时在并行争 CPU 时撞不撞（推的） |
| D4 | 清单一致：四份定义、两份规则、一份共用约束、一份 skill、两个钩子里凡列提交时重型门禁的地方，是不是都点了这一道、点法一致；有没有一处漏掉或多出（`grep -rn` 现查，输出不截断） |
| D5 | 规则写法：改后的句子有没有论证、日期、位置指代（`.claude/singlefs-ai-sop/rules/rules-discipline.md`「1. 正文只写四样」「7. 正文不许用位置指代，也不许自称」） |

**共用问句**：照改后的字面干活，哪一步会做错或做不了；举出具体的派发情形或命令。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- 门禁本体：`.claude/gate.d/checker-tier-formal-verification.sh` 六格（verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds、pinned-red-examples-complete、pinned-red-examples-replay），判法 `research/scripts/verus-proof-check.py`、`research/scripts/e165-pin-red-examples.py`，登记 `.claude/gate.d/verus-pins.tsv`（crate `crates/singlefs-replay-prefix` 32、`rlimit	10`）。被验的核 `crates/singlefs-replay-prefix/src/lib.rs`，必红世界表 `crates/singlefs-replay-prefix/verus-mutations.tsv` 94 行。
- 主 agent 2026-10-01 在合入后的真核上现跑：Verus 0.2026.09.20 `--no-cheating`「32 verified, 0 errors」；94 行必红世界逐行套上全红、没有一行只红在 rlimit 用尽上（`/tmp/claude-1000/merge-4/verus-rows/`）；3-10 用判法本身在真仓上跑必红格，94 行全部红计入、整格退 0（`/tmp/claude-1000/m3-formal-gate-3-10/report.md`）。整道门禁没在真仓上跑过（归提交时）。
- `crates/` 这一轮不判。

## 三、条款（材料员整段抄进附录）

- 被判的 11 份文件的现行全文（快照 `research/prompts/defs-m3-formal-gate-r1-snapshot/sha256sums.txt`）；
- `.claude/hooks/lib_heavy_tests.py` 第 120–160 行（kind 表）、`.claude/gate.d/stage-inputs.tsv` 这一道那一行、`research/scripts/admission.py` 的 `gate_reuse`；
- `.claude/gate.d/checker-tier-formal-verification.sh` 文件头。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | D4、D5，兼 D1 | 逐份核改后的字面是不是用户定案与 4-5 派发出口要的：清单一致、写法合规；D1 照字面逐个角色推一遍 |
| 云端攻方（Opus） | D1、D2、D3 | 造派发情形与命令攻改后的字面：要喂 hook 的只喂 JSON、只看退出码，被判的命令一条都不执行；D2 能在草稿目录的仓副本里造一个「换了 Verus、输入没变」的复用情形就造 |

## 五、交付

- 腿的报告写 `research/prompts/defs-m3-formal-gate-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/defs-m3-formal-gate-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引文件写那份文件自己的行号，行号去被引文件里现查，不从背景材料里数。
- 不跑重型测试（这一道门禁本身也是重型），不跑 `--selftest`；只停自己起的进程。
