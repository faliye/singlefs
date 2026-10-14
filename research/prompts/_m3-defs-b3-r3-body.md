# 工具与治理批的定义与规则改动：第三轮正文（2026-10-13）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 R1 R2 R3 R4 R5 R6 R7 R8 R9 G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 H1 H2 H3 H4 K1 K2 K3 K4 K5 K6 K7 K8 K9 K10 K11 -->

## 一、这一轮要判什么

这是同一题的第三轮，也是最终轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后不开第四轮）。第一轮判决 `research/prompts/m3-defs-b3-r1-main-verification.md`（改法 G1–G11，岔路 H1–H3），第二轮判决 `research/prompts/m3-defs-b3-r2-main-verification.md`（改法 K1–K11，岔路 H4）；两轮的改法都已落进冻结副本（批 4、批 4 收尾两件、批 5 四件与批 5 收尾，调度记录 `records/2026-10-12-里程碑三工具与治理批.md`）。

第三轮分工：「已定」项撑不撑得住 / 未定项与另立账清单的完整性 / 两轮报出的数字能不能核。**只攻前两轮站住的形态**：第二轮判「站得住」的 R4、R6、R8，以及各格里辩方第二轮判「修到」的那几条（`research/prompts/m3-defs-b3-r2-sonnet-output.md` 第 1 节）。第二轮的改法 K1–K11 是被攻过零轮的新形态，这一轮不为它们开攻，判决里列进交用户表标「零轮」；腿顺手看到 K 改法本身的问题可以记，主 agent 只记不判。

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/clerk.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-defense.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b3-r3/repo/`（集成副本标签 `after-b5-fixup` 那一版，没有 `.git`）；被判文件与它们压着的钩子、脚本、门禁共 35 份的 sha256 在 `research/prompts/m3-defs-b3-r3-snapshot/frozen-copy-sha256.txt`。前两轮的冻结副本在 `~/.local/share/singlefs/work/m3-defs-b3-r1/repo/`、`~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`。腿只读冻结副本，不读主工作区。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

| 格 | 判什么 | 对象 |
|---|---|---|
| T1 | 线程数「默认 31、可输入参数调整、固定不看忙闲、双机各 31」这一套已定项，在冻结副本里还有没有可达的工作流让实际线程数不是登记值而规则让人以为是（第一、二轮已列的代码批、变异门禁、E171、双机驱动、GPU 对拍 CPU 一侧不再交） | `.claude/heavy-thread-default`、`scripts/hosts/capped.sh`、`scripts/hosts/heavy-threads.sh`、`scripts/hosts/heavy_threads.py`、`.claude/main-agent.md`「派发提示怎么写」、`.claude/agent-common.md` 线程那条、各定义「输入」 |
| T2 | 写者锁（R6 站住的形态）与看门狗「被停的同样算」 | `.claude/hooks/lib_writer_lock.py`、`.claude/hooks/write-guard.sh`、`.claude/agent-common.md` 写者锁那条、`.claude/main-agent.md`「派出去之后」 |
| T3 | ⑭ 长活三行与多机线程（R8 站住的形态） | `.claude/hooks/lib_long_run.py`、`.claude/hooks/runner-dispatch-guard.sh` ⑭、`scripts/agents/dispatch-prompt.py`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md` |
| T4 | `verification.md` 两句（R4 站住的形态）与 `--test-threads 1` 修到的单机路径 | `.claude/rules/verification.md`、`.claude/gate.d/checker-tier-layer0-replay.sh`、`scripts/crash/crash-point-replay.sh` |
| T5 | 未定项与另立账清单完整性：两轮判决里「另立一笔账」「交用户」「核不动的」三类，合起来是不是这一批改动留下的全部开着的线；有没有一条打中两轮都没列、也没修 | 两轮判决、两轮岔路单、四条腿报告 |
| T6 | 两轮报出的数字能不能核：判决与正文里引的每个计数（例：可疑写法 94 处、排除表 66 行、扫 831 份、verify3 133 项、verify4 148 项、records 能当出处的 504 行、replay.sh 22 行、19835 行新增命中），按引的命令或产物现算，对不对得上、口径写全没有 | 两轮判决、第二轮正文、调度记录里引的产物（`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify*.summary` 与 `verify*-out/`、各批报告） |

## 二、实现今天的样子（主 agent 的观测，2026-10-13 现查）

- `crates/` 这一轮不改；`crates/` 里自己定并行度默认值的写法照旧登记在冻结副本 `.claude/heavy-thread-exempt` 的「代码批」行。
- 批 5 在集成副本上复核（`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify4.summary`）：派 148 项、收 148 项、判错 1 项（派发闸一格期望过时，批 5 收尾改了）；批 5 收尾之后主 agent 复跑派发闸、重型测试闸与它的库、骨架脚本四份自证绿，`thread-limit-any-refused`、`selftest-work-copy-unrecognized`（两份）、`types-before-nine` 四个开关各自红（`verify5-out/`），九种类型骨架都打「线程上限：31」，十种都打「预计挂钟」。
- 还没改、等别的会话提交落地后改的：双机驱动 `scripts/crash/crash-amplification-two-hosts.sh` 的现算线程与 `--test-threads`、`scripts/gate/admission.py` 拼的 crash-case 命令；E171 `rows_orchestrator.py`；变异门禁 `worker_count_for` 等。

## 三、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 逐条核「两轮判决与正文说的数和清单，就是产物与冻结副本里的样子」 | T5、T6 | T6 每个数一行：引处（文件:行）、现算命令、现算结果、对不对得上、口径缺什么；T5 每条开着的线一行：出处、在不在两轮的另立账 / 交用户 / 核不动三类里，漏列的给出打中的历史与出处 |
| **云端攻方（Opus）** | 造工作流历史攻前两轮站住的形态 | T1、T2、T3、T4 | 每条历史写成可复现的步骤（派发提示原文、钩子喂法、命令序列），在冻结副本拷贝里真跑钩子与脚本的判定；交「照规则做、结果错」的那一步与错在哪；前两轮报告里有的历史不再交，除非改后又中 |

## 四、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留，这一题收 | 两条腿在那一格都给不出照定义做却违反用户定案、或定义说拒而钩子不拒的可达历史；T6 每个数都对得上 |
| 一格判「站不住」⇒ 不开第四轮，改法写进判决的交用户表、标「零轮」，由用户定或另立一题 | 给出那样一条可达历史；T6 有数对不上 |
| 打中也打中改前的文字 ⇒ 另立一笔账 | 同一段历史在第一轮冻结副本上给出同样的错 |
| T5 漏列 ⇒ 补进判决的另立账或交用户表 | 一条开着的线两轮判决都没写去向 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 五、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b3-r3-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b3-r3-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b3-r3-<腿名>-model/`。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」，主 agent 写判决时复跑。
