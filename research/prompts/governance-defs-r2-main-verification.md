# 严查第一轮之后的十份定义改动：第一轮判决（2026-09-26）

正文 `research/prompts/_governance-defs-r2-body.md`；背景材料 `research/prompts/_governance-defs-r2-background.md`；附录二 `research/prompts/_governance-defs-r2-diff.md`；开工快照 `research/prompts/governance-defs-r2-snapshot/sha256sums.txt`（42 行，腿交齐后 `sha256sum -c` 全部 OK，腿跑着的时候没有文件被改）。

被判的 10 份：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-verifier.md`。

## 一、这一轮交了什么

| 腿 | 报告 | sha256 |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/governance-defs-r2-sonnet-output.md`（108 行） | 6d720dba38e19b077289134bfd12d7014748290f92ca959b6569ada226fa48e8 |
| 云端攻方（Opus） | `research/prompts/governance-defs-r2-opus-output.md`（334 行），模型 `research/prompts/governance-defs-r2-opus-model/`，复跑 `bash run-all.sh <草稿目录>` | 523511e074c5117cc023ce19ae06f30adbae5b02170591fd260a01ef362d33cb |
| 核查员 | `research/prompts/governance-defs-r2-verifier-output.md`（117 行） | ea115aff04e561246362d964d4eb30c53bab307f361706ba9a035e858ac4138f |
| 本地攻方 / 本地辩方 | **缺席**：这个容器里没有 `~/code/ai-center`；用户选「走一轮三方（缺本地腿）」 | — |

核查员：攻方 24 处 22 ✓、0 ✗、核不动 1、部分核 1；模型复跑退 0，归一化临时路径后与 `run-all.out` 逐字节相同。正推 38 处 35 ✓、3 ✗，3 处都是引文跨两行只标了一个行号，内容无误，不影响判定方向。

## 二、跑前条款，各触发没触发

| 条款 | 触发没触发 |
|---|---|
| 一格判「站得住」⇒ 照留 | G4 的三处标签统一与四种情形分支、G5 的大部分改动照留 |
| 一格判「站不住」⇒ 改那一处，判决标「被攻过零轮」 | 触发：B1、B2、B4、B6 与正推 G3 同一格；B3、B5、B7 改前改后都中或分辨一部分，改法不改变行为，一并写回 |
| 删改丢了判据 ⇒ 补回 | 没触发：正推逐处看了 diff 删掉的半句，没有丢今天仍成立的判据 |

## 三、逐条判决（主 agent 逐条现查过）

| 编号 | 格 | 打中了什么 | 现查 | 判决与改法 |
|---|---|---|---|---|
| B1 / 正推 G3 | G3 | 59 号「计数：」行没有 💥、⚠️ 两栏；进程没跑完的落「无效」或「没红」，都判红；改后文字说「那一行各栏都有」「💥 不判失败」对 59 号不成立 | `59-crates-mutation-replay.sh` 的 `judge_one` 属实：编译错误记 invalid，否则记 failure（「没跑到」） | 站不住。mutation-sampling.md 与 mutation-triage.md 写明 59 号没有这两栏、两种落法都判红；「💥 不判失败」限定为 `mutate.sh` |
| B2 | G2 | 崩溃验证员的六条路径对 57、59 太宽，别的会话动了 59 号不读的 `research/scripts/` 文件也会停；停下之后 main-agent 没有出路 | 属实（`stage-inputs.tsv` 59 号只登记 `research/scripts/run-with-memory-cap.sh`） | 站不住。改成每道取自己的输入（55、59 取登记行，57 取 `litmus crates .claude/scripts/lkmm.sh`，都加阶段脚本本身）；停下之后主 agent 照第 9 条协商 |
| B3 | G2 | 漏了 `lkmm.sh`、阶段脚本、未跟踪的 `src/bin`、`tests` | 属实 | 同 B2 一并改：路径补上，未跟踪文件要没有输出 |
| B4 | G1 | 三行放进同一次调用后退出码永远是 `git worktree remove` 的，「判绿之后才往下」无退出码可依；每次留一个临时目录 | 主 agent 用桩复现：改后第三行在 0 / 1 / 250 / 251 四种结局下整次调用退同一个数，临时目录与 worktree 都不留 | 站不住。54 号打印的第三行改成保留 `--full` 的退出码、清理临时目录；main-agent 写明看这次调用退 0 |
| B5 | G1 | 复用名单漏了 87 号 | `87-replay.sh` 调 `stage-must-run.sh` 属实 | 改：三处补 87 |
| B6 | G5 | 重跑登记「照抄原登记的英文名」，已有登记没有这一行 | 属实 | 改：原登记没有的从装置文件名取 |
| B7 | G4 | 没人要求主 agent 记腿开工时刻；没给时间时清单外的文件落 ✗ | 属实 | 改：implementation-workflow.md 快照那一节要求连同派腿时刻交核查员；核查员没拿到时刻时落「分不清」 |
| N1 | G1 | main-agent 那一句抄进派发提示会被派发闸拒 | 改前原句一样被拒，不分辨 | 不拿它判这一批；派发提示照 `main-agent.md`「派发提示怎么写」只写对方要的东西，不抄那一整句 |
| O1 / 正推 G1 | G1 | 崩溃验证员在主工作区跑的 55、57、59，gate-staged.sh 在暂存树上不复用、再跑一遍 | 属实（`stage-must-run.sh` 只比 `refs/sop/staged-green`） | 设计判断，交用户，不在这一轮改 |

写回之后：54 号出路打印的桩复现如上；10、20、62、63、68、72、73、75、90 号与 doc-lint、rules-lint 在提交前现跑。**写回的每一处都被攻过零轮**，由第二轮严查接着攻。

## 四、交用户的

| 项 | 为什么交用户 |
|---|---|
| 崩溃验证员的 55、57、59 与 gate-staged.sh 里的同几道在一次提交里各跑一次 | 要么撤掉崩溃验证员这一步、要么让它的绿能被复用，都是流程设计，改动越出这一轮的描述修正 |
| `.claude/term-rename-exempt` 里 `.claude/warnings/` 整目录豁免，与 path-moves.md「不整个目录豁免」相反（正推附带发现） | 在 71f0cbc 之前就存在，这一轮没碰那份文件 |

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、两份规则与 54 号出路的文字，没有动任何一条决策分项的定案、射程或依据，也没有新增或撤回实验结论。
