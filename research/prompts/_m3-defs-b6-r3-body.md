# 工具与治理批的零轮形态：第三轮（最终轮）正文（2026-10-13）

<!-- doc-lint:not-numbers Y1 Y2 Y3 Y4 Y5 Y6 V2 V4 X5 L1 L2 L3 L4 L5 L6 L7 M1 M2 M3 M4 M5 M6 M7 -->

## 一、这一轮要判什么

同一题（零轮形态）的第三轮，也是最终轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后不开第四轮）。第一轮判决 `research/prompts/m3-defs-b6-r1-main-verification.md`（改法 L1–L7），第二轮判决 `research/prompts/m3-defs-b6-r2-main-verification.md`（改法 M1–M7）；两轮改法都已落进冻结副本（批 7、批 7 收尾、批 8、批 8 收尾，调度记录 `records/2026-10-12-里程碑三工具与治理批.md`）。

第三轮分工：「已定」项撑不撑得住 / 未定项与另立账清单的完整性 / 两轮报出的数字能不能核。**只攻前两轮站住的形态**：第一轮判「站得住」的 V2（让出 CPU 的两句）、V4（`--test-threads 1` 的文字），第二轮判「站得住」的 X5（private-values 整串打码），以及第二轮辩方判「修到」的条目（`research/prompts/m3-defs-b6-r2-sonnet-output.md`）。第二轮改法 M1–M7 是被攻过零轮的新形态，这一轮不为它们开攻，判决里列进交用户表标「零轮」；腿顺手看到 M 改法本身的问题可以记一行，主 agent 只记不判。

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b6-r3/repo/`（集成副本标签 `after-b8-fixup`，没有 `.git`）；零轮形态起动过的 33 份文件的 sha256 在 `research/prompts/m3-defs-b6-r3-snapshot/frozen-copy-sha256.txt`。前两轮冻结副本在 `~/.local/share/singlefs/work/m3-defs-b6-r1/repo/`、`~/.local/share/singlefs/work/m3-defs-b6-r2/repo/`，零轮之前那一版在 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`。腿只读这四份冻结副本，不读主工作区。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

| 格 | 判什么 | 对象 |
|---|---|---|
| Y1 | 第二轮辩方判「修到」的 ⑮、⑭、⑥ 条目（第一轮正推与攻方 V1、V6、V8 打中的那些历史），在第三轮冻结副本里还修着没有；有没有可达工作流让「默认 31、固定不看忙闲」实际落空而规则让人以为没有 | `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/lib_long_run.py`、`.claude/hooks/lib_abandoned_rounds.py`、`scripts/agents/dispatch-prompt.py`、`.claude/main-agent.md`、`.claude/agent-common.md`、各定义「输入」 |
| Y2 | 第二轮辩方判「修到」的重型测试闸条目（`env -iC`、`env -S`、48 例 env 矩阵、仓副本拒绝说明）与 heavy-thread-defaults 第一轮 V3 九种写法 | `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/lib_shell_words.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/gate.d/code-tooling.sh` |
| Y3 | X5：private-values 整串打码（第二轮站得住的形态） | `.claude/gate.d/doc-text.sh` private-values 格 |
| Y4 | V2（让出 CPU 的两句）、V4（`--test-threads 1` 的文字）两处第一轮站得住的文字，在第三轮冻结副本里与钩子、脚本还对不对得上 | `.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/verification.md`、`scripts/crash/crash-point-replay.sh`、`.claude/gate.d/checker-tier-layer0-replay.sh` |
| Y5 | 未定项与另立账清单完整性：这一题两轮判决的「另立一笔账」与改法表，加四条腿报告里打中的每一条，合起来是不是全部开着的线；有没有一条打中两轮都没写去向 | 两轮判决、四条腿报告 |
| Y6 | 两轮报出的数字能不能核：两轮判决、正文与调度记录里这一题引的计数（例：verify8 184 项、verify10 191 项、排除表行数、扫 832 份、可疑写法数、各样本点名的份数），按引的命令或产物现算，对不对得上、口径写全没有 | 两轮判决、正文、调度记录、`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify*.summary` 与 `verify*-out/` |

## 二、实现今天的样子（主 agent 的观测，2026-10-13 现查）

- `crates/` 这一题不改。
- 集成副本复核：批 8（`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify10.summary`，191 项判错 5 项：1 项真红是三处自证样本缺排除行，4 项是复核脚本直接设库开关被门禁格覆盖）；批 8 收尾之后（`verify11-out/`）派发闸、detached-run、host-dispatch、dispatch-prompt 四份自证绿，两个新开关各红 7 格、3 格，heavy-thread-defaults、long-run-header-exempt、gate-structure 三格在集成真树上绿（扫 832 份），code-tooling 样本档绿。
- 上一题三轮与这一题前两轮的另立账不在射程里，不再交，除非 M1–M7 让它变得更糟。

## 三、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 逐条核「两轮判决与正文说的数和清单，就是产物与冻结副本里的样子」 | Y4、Y5、Y6 | Y6 每个数一行：引处（文件:行）、现算命令、现算结果、对不对得上、口径缺什么；Y5 每条开着的线一行：出处、在不在两轮的另立账 / 改法表里，漏列的给出出处与打中的历史；Y4 逐句核文字与钩子 |
| **云端攻方（Opus）** | 造工作流历史攻前两轮站住的形态 | Y1、Y2、Y3 | 每条历史写成可复现的步骤，在冻结副本拷贝里真跑钩子与脚本的判定；交「照规则做、结果错」的那一步与错在哪；前两轮与上一题腿报告里有的历史不再交，除非改后又中 |

## 四、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留，这一题收 | 两条腿在那一格都给不出照定义做却违反用户定案、或定义说拒而钩子不拒的可达历史；Y6 每个数都对得上 |
| 一格判「站不住」⇒ 不开第四轮，改法写进判决的交用户表、标「零轮」，由用户定 | 给出那样一条可达历史；Y6 有数对不上 |
| 打中也打中零轮之前那一版 ⇒ 另立一笔账 | 同一段历史在 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/` 上给出同样的错 |
| Y5 漏列 ⇒ 补进判决的另立账或交用户表 | 一条开着的线两轮判决都没写去向 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 五、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b6-r3-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b6-r3-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b6-r3-<腿名>-model/`（含 `tools/rerun.sh`；复跑经 `scripts/hosts/run-with-memory-cap.sh` 时上限给 2G；要的环境变量写进 rerun.sh 的默认值，不靠复跑的人记）。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数；引主工作区 `research/prompts/` 下的判决与产物写仓内相对路径。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」；造的小树要齐（例：叫停轮那一格要有腿文件，不然门禁格退 77 无对象）。
