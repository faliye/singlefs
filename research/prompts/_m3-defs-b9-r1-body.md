# 工具与治理批的零轮形态（第二题）：第一轮正文（2026-10-13）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 Z7 M1 M2 M3 M4 M5 M6 M7 N1 N2 N3 N4 N5 F-Y3 Y1 Y3 -->

## 一、这一轮要判什么

上一题（`m3-defs-b6`）三轮跑完不开第四轮，第三轮判决 `research/prompts/m3-defs-b6-r3-main-verification.md`「交用户表」里被攻过零轮的形态，用户 2026-10-13 定「再开一题三轮」：另立这一题，从头走三轮三方。M1–M7 来自上一题第二轮判决 `research/prompts/m3-defs-b6-r2-main-verification.md`「改法」表（批 8），N1、N2、N3、N5、F-Y3 来自上一题第三轮判决「交用户表」（批 9；N4 只改判决措辞，不在这一题）。

| 格 | 零轮形态 | 在哪 |
|---|---|---|
| Z1 | M1、N1、N3：⑮「线程上限：」只认一个整数，子 agent 只照这一个整数起重活、提示里别处写的数一概不照；三个反引号围起来的代码块里的行 ⑮ ⑭ 都不判，认围栏照 CommonMark（围栏行至多缩进 3 格、反引号围栏的信息串里不许有反引号，行首的行内三反引号不开块）；只有「线程上限已判」行、没有「线程上限：」行时子 agent 不起重活、要起就停下交回；`dispatch-prompt.py` 改叫 clerk | `.claude/hooks/runner-dispatch-guard.sh` ⑮ ⑭ 与 `without_code_blocks`、`scripts/agents/dispatch-prompt.py`、`.claude/agent-common.md`、`.claude/main-agent.md`、九份定义「输入」 |
| Z2 | M2：heavy-thread-defaults 排除表按被判行匹配（锚串落在那一条被判行里才放）；`\|\| X=$var`、`\|\| X=$((…))` 认成钉数；`printenv NAME` 认成读；宽判法带回的误判逐行判、不读环境的进排除表 | `.claude/gate.d/code-tooling.sh` heavy-thread-defaults 格、`.claude/heavy-thread-exempt` |
| Z3 | M3：重型测试闸仓副本那一支的拒绝说明不写「带前缀跑」，指到 `.claude/rules/implementation-workflow.md` 重型测试那一条 | `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh` |
| Z4 | M4、N2：「别的轮名」= 仓里认得的轮名全集 ∪ 形如 `<名>-r<数字>` 的写法；叫停登记的出处写成 `records/<文件>.md 第 N 行〔<原话>〕`，按原话在那份文件里找、恰好一次才收，行号只当提示；免出处只给库里写死的旧行名单 | `.claude/hooks/lib_abandoned_rounds.py`、`runner-dispatch-guard.sh` ⑥、`.claude/gate.d/doc-process-records.sh` abandoned-rounds 格、`.claude/rules/three-way-inference.md`、`.claude/agent-common.md`、`.claude/main-agent.md`、`research/prompts/abandoned-rounds.tsv` |
| Z5 | M5、F-Y3：doc-text 各格打印路径都经 private-values 那一份打码；private-values 先判符号链接（比路径与链接文字、不跟进去）；读不了的文件列成红行并打码、不崩；「取不到 … 相对基准」那一支同样打码；兄弟三格读同一份读不了的文件不崩 | `.claude/gate.d/doc-text.sh` |
| Z6 | M6：⑭「多机：」一行与脚本文件头 `# long-run-hosts:` 逐字等于 `lib_long_run.py` 的 `CANONICAL_HOSTS_SENTENCE`（可带句末句号），后面不接任何说明，说明另起一行；重复标签拒 | `.claude/hooks/lib_long_run.py`、`runner-dispatch-guard.sh` ⑭、`scripts/hosts/detached-run.sh`、`scripts/hosts/host-dispatch.py`、`scripts/agents/dispatch-prompt.py`、定义与 `.claude/main-agent.md` |
| Z7 | M7、N5：长活声明豁免表的只缩不涨按（路径、sha256）成对比，基准那一版里同一路径的 sha256 变了也算涨；E171 下一次修订的新入口不进表；`stage-owners.tsv` long-run-header-exempt 那一行改成「红了删行；改被钉的脚本要重钉 sha256，按只缩不涨判红，交用户」 | `.claude/hooks/lib_long_run.py`、`.claude/gate.d/code-tooling.sh` long-run-header-exempt 格、`.claude/long-run-live-hosts-exempt`、`.claude/gate.d/stage-owners.tsv` |

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b9-r1/repo/`（集成副本标签 `after-b9-fixup`，没有 `.git`）；这两批零轮形态与上一题动过的 34 份文件的 sha256 在 `research/prompts/m3-defs-b9-r1-snapshot/frozen-copy-sha256.txt`。M 形态之前那一版是 `~/.local/share/singlefs/work/m3-defs-b6-r2/repo/`（`after-b7-fixup`），N 形态之前那一版是 `~/.local/share/singlefs/work/m3-defs-b6-r3/repo/`（`after-b8-fixup`）。腿只读这三份冻结副本，不读主工作区。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

## 二、实现今天的样子（主 agent 的观测，2026-10-13 现查）

- `crates/` 这一题不改。
- 集成副本复核：批 9 打上之后 `~/.local/share/singlefs/work/m3-tooling-b2-integration/verify12.summary`「派 210 项、收 210 项、判错 6 项」：4 项是复核脚本直接设库开关被门禁格覆盖（同名门禁开关都红），stage-owners 缺 session-names 一行与 abandoned-rounds 认不出三轮发过腿两格在 `after-b8-fixup` 上同样红；后一格的三行（c196-r1、c364-r3、d16-item1-r2，材料已归档）在 `after-b9-fixup` 里删了，之后那一格绿。库里的旧行名单与规则、共用约束文字仍写那四个名字。
- 上一题三轮判决的另立账不在这一题射程里，不再交，除非零轮形态让它变得更糟。

## 三、要答的

每格都问三件：这一形态有没有过度外推或越位（管了不该管的、判了判不了的）；它与已有条款、钩子、门禁之间有没有内部矛盾或循环依赖（照一处做违反另一处，一处收一处拒）；七格里哪一条最脆弱（换一个最自然的写法就绕过或误伤）。另各格的专问：

- Z1：CommonMark 认围栏之外的写法（波浪线围栏、缩进 4 格的代码块、列表里的围栏、没闭合的围栏）各怎么判；只有已判行时子 agent「不起重活」，钩子与定义谁判得到。
- Z2：按被判行匹配之后，同一锚串在一份文件里出现几次、行改了几个字会怎样；排除表 89 行里有没有锚串已经落不到被判行上。
- Z4：原话命中「恰好一次」在记录被别的会话追加、改写之后还稳不稳；旧行名单的四个名字里三个已无行，名单与表的关系。
- Z5：链接、读不了的文件之外，还有没有不经打码的输出通道。
- Z6：说明另起一行之后，现算的说法挪到下一行照样送到子 agent 手里（上一题攻方顺手记的，推的、没喂）。
- Z7：（路径、sha256）成对比在基准那一版没有这张表时做什么。

## 四、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 逐条核「定义与规则写的就是钩子、脚本、门禁做的」 | Z1、Z3、Z4、Z6、Z7 | 每一句零轮新写或改写的定义与规则文字，对着冻结副本里的钩子与脚本源码（文件:行）判「一致 / 不一致 / 文字没写到」，不一致的给出实际行为与能复现的喂法 |
| **云端攻方（Opus）** | 造工作流历史攻零轮形态 | Z1、Z2、Z4、Z5、Z6、Z7 | 每条历史写成可复现的步骤（派发提示原文、钩子喂法、命令序列），在冻结副本拷贝里真跑钩子与脚本的判定；交「照规则做、结果错」的那一步与错在哪；每条打中在那一形态之前那一版冻结副本上也喂一遍（M 形态喂 `m3-defs-b6-r2`，N 形态喂 `m3-defs-b6-r3`），写明之前中不中；上一题三轮腿报告里有的历史不再交，除非改后又中 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留 | 两条腿在那一格都给不出照定义做却违反用户定案（默认 31、固定不看忙闲、只核自己的且豁免表不登记别人没审过的、私有信息不进仓、在跑的不动、仓副本自证收回放宽）或定义说拒而钩子不拒、钩子拒而定义没写的可达历史 |
| 一格判「站不住」⇒ 改法进下一批，改完开第二轮 | 给出那样一条可达历史 |
| 打中也打中那一形态之前那一版 ⇒ 另立一笔账 | 同一段历史在之前那一版冻结副本上给出同样的错 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b9-r1-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b9-r1-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b9-r1-<腿名>-model/`（含 `tools/rerun.sh`，从 `scripts/agents/three-way-rerun-template.sh` 拷来、只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh` 时上限给 2G；要的环境变量写进 rerun.sh 的默认值，不靠复跑的人记；`--work` 的草稿目录要在 `~/.local/share/singlefs/work/` 底下）。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数；引主工作区 `research/prompts/` 下的判决与产物写仓内相对路径。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」；造的小树要齐（例：叫停轮那一格要有腿文件，不然门禁格退 77 无对象）。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
