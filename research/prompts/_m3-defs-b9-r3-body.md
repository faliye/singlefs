# 工具与治理批的零轮形态（第二题）：第三轮正文（2026-10-14）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 Z7 P1 P2 P3 P4 P5 P6 P7 S1 S2 S3 S4 S5 S6 S7 U1 U2 U3 U4 Q1 Q2 Q3 Q4 Q5 Q6 -->

## 一、这一轮要判什么

同一题的第三轮，也是最终轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后不开第四轮）。第一轮判决 `research/prompts/m3-defs-b9-r1-main-verification.md`（改法 P1–P7），第二轮判决 `research/prompts/m3-defs-b9-r2-main-verification.md`（改法 U1–U4）；两轮改法都已落进冻结副本（批 10、批 10 收尾、批 11，调度记录 `records/2026-10-12-里程碑三工具与治理批.md`）。

第三轮分工：「已定」项撑不撑得住 / 未定项与另立账清单的完整性 / 两轮报出的数字能不能核。**只攻前两轮站住的形态**：第二轮判「站得住」的 S2（P2 缺行出路两种写法、clerk 改名）、S4（P4 仓副本改过正文的自证按提交时的检查办）、S6（P6 doc-text 打码），以及第二轮辩方判「修到」的条目（`research/prompts/m3-defs-b9-r2-sonnet-output.md`「总表」）。第二轮改法 U1–U4 是被攻过零轮的新形态，这一轮不为它们开攻，判决里列进交用户表标「零轮」；腿顺手看到 U 改法本身的问题可以记一行，主 agent 只记不判。

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/implementation-workflow.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b9-r3/repo/`（集成副本标签 `after-b11`，没有 `.git`）；这一题动过的 35 份文件的 sha256 在 `research/prompts/m3-defs-b9-r3-snapshot/frozen-copy-sha256.txt`。前两轮冻结副本在 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/`（`after-b9-fixup`，P 改法之前）、`~/.local/share/singlefs/work/m3-defs-b9-r2/repo/`（`after-b10-fixup`）。腿只读这三份冻结副本取被判字面；做法照主工作区的共用约束。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

| 格 | 判什么 | 对象 |
|---|---|---|
| Q1 | S2：⑮ 缺行时的两种出路（要别的数写「线程上限：N」加已判行；单独的已判行只给确实不起重活的活）与 `dispatch-prompt.py` 的 clerk 改名，在第三轮冻结副本里撑不撑得住；有没有可达工作流让「默认 31、固定不看忙闲」实际落空而规则让人以为没有 | `.claude/hooks/runner-dispatch-guard.sh` ⑮、`scripts/agents/dispatch-prompt.py`、`.claude/main-agent.md`、`.claude/agent-common.md`、各定义「输入」 |
| Q2 | S4：仓副本里正文改过的登记为 light 的自证按提交时的检查办、主 agent 在集成副本里带 `SINGLEFS_HEAVY_TESTS=commit` 跑、不弹窗；与「重型提交之外先弹窗」「仓副本自证收回放宽」两条撑不撑得住 | `.claude/rules/implementation-workflow.md`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh` |
| Q3 | S6：doc-text 各格打印路径与片段经打码（date-once、term-renames 转出的 sweep-term 输出、clock-times、prime-marks、session-names 片段挪到私有值边上） | `.claude/gate.d/doc-text.sh` |
| Q4 | 第二轮辩方判「修到」的条目（第一轮两腿打中的 Z1 围栏四形、Z4 三种日常改动、Z5 两形、Z7 文字），在第三轮冻结副本里（批 11 动过之后）还修着没有 | 同第一轮判决逐格判决表的「在哪」一列 |
| Q5 | 未定项与另立账清单完整性：这一题两轮判决的「另立一笔账」与改法表，加四条腿报告里打中的每一条，合起来是不是全部开着的线；有没有一条打中两轮都没写去向 | 两轮判决、四条腿报告 |
| Q6 | 两轮报出的数字能不能核：两轮判决、正文与调度记录里这一题引的计数（例：verify13 223 项判错 5、verify14 244 项判错 7、三道样本档 46 / 22 / 27 例、扫 832 份、排除表 89 行、各样本点名的份数、cite-check 的引文数），按引的命令或产物现算，对不对得上、口径写全没有 | 两轮判决、正文、调度记录、`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify13.summary`、`verify14.summary` 与 `verify13-out/`、`verify14-out/`、`~/.local/share/singlefs/work/m3-defs-b9-r2-main-out/log/` |

## 二、实现今天的样子（主 agent 的观测，2026-10-14 现查）

- `crates/` 这一题不改。
- 集成副本复核：批 11 打上之后 `~/.local/share/singlefs/work/m3-tooling-b2-integration/verify14.summary`「派 244 项、收 244 项、判错 7 项」：`brk-long-run-exempt-*` 四项是复核脚本直接设库开关被门禁格覆盖（同 verify12）；stage-owners 缺 session-names 一行与 doc-text 在集成真树上 clock-times、term-renames 两格红，批 10 之前同样红；fixture-claims 格红在集成副本的 git 历史从 2026-10-12 起（122 个文件名、964 处正文的日期早于第一个提交）与 checker-tier-layer0-replay 缺红样本，批 11 加的样本一份都不在红项里。两棵漂移小树只记不判（原话改了字那棵退 1，原话出现两次那棵退 0）。
- 这一题前两轮与上一题三轮的另立账不在 Q1–Q4 射程里，不再交，除非 U1–U4 让它变得更糟；Q5 要把它们一并列进清单核完整性。

## 三、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 逐条核「两轮判决与正文说的数和清单，就是产物与冻结副本里的样子」 | Q5、Q6 | Q6 每个数一行：引处（文件:行）、现算命令、现算结果、对不对得上、口径缺什么；Q5 每条开着的线一行：出处、在不在两轮的另立账 / 改法表里，漏列的给出出处与打中的历史 |
| **云端攻方（Opus）** | 造工作流历史攻前两轮站住的形态 | Q1、Q2、Q3、Q4 | 每条历史写成可复现的步骤，在冻结副本拷贝里真跑钩子与脚本的判定；交「照规则做、结果错」的那一步与错在哪；前两轮与上一题腿报告里有的历史不再交，除非改后又中 |

## 四、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留，这一题收 | 两条腿在那一格都给不出照定义做却违反用户定案、或定义说拒而钩子不拒的可达历史；Q6 每个数都对得上 |
| 一格判「站不住」⇒ 不开第四轮，改法写进判决的交用户表、标「零轮」，由用户定 | 给出那样一条可达历史；Q6 有数对不上 |
| 打中也打中 P 改法之前那一版 ⇒ 另立一笔账 | 同一段历史在 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/` 上给出同样的错 |
| Q5 漏列 ⇒ 补进判决的另立账或交用户表 | 一条开着的线两轮判决都没写去向 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 五、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b9-r3-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b9-r3-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b9-r3-<腿名>-model/`（含 `tools/rerun.sh`，从 `scripts/agents/three-way-rerun-template.sh` 拷来、只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh` 时上限给 2G；要的环境变量写进 rerun.sh 的默认值；`--work` 的草稿目录要在 `~/.local/share/singlefs/work/` 底下；输出里的绝对路径先换占位再截断）。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数；引主工作区 `research/prompts/` 下的判决与产物写仓内相对路径。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」；造的小树要齐。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
