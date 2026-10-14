# 工具与治理批的零轮形态（第二题）：第二轮正文（2026-10-13）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 Z7 P1 P2 P3 P4 P5 P6 P7 S1 S2 S3 S4 S5 S6 S7 F-Z1 F-Z4 F-Z5 -->

## 一、这一轮要判什么

同一题的第二轮。第一轮判决 `research/prompts/m3-defs-b9-r1-main-verification.md`：Z1、Z2、Z3、Z4 站不住，Z5、Z6、Z7 部分站不住，改法 P1–P7。批 10 照改法改完、主 agent 补两处文字（调度记录 `records/2026-10-12-里程碑三工具与治理批.md`）。

第二轮分工（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。这一轮派辩方腿与攻方腿：辩方复核第一轮判决够不够得着、P1–P7 是不是对第一轮两腿每一条打中都修到了、有没有改过头；攻方攻 P1–P7 本身与它们带进来的新机制。

| 格 | 第一轮改法 | 改后的形态 | 在哪 |
|---|---|---|---|
| S1 | P1 | ⑮ ⑭ 认代码块改用 markdown-it-py 的 commonmark 预设：反引号与波浪线、列表与引用块里的都认，只遮闭合了的围栏块，没闭合的照判；库载不进一行都不遮；定义写「代码块（照 CommonMark）」，⑭ 写明代码块里的行不判、同一标签第二行拒 | `.claude/hooks/runner-dispatch-guard.sh` 的 `without_code_blocks`、`.claude/agent-common.md`、`.claude/main-agent.md`、十份定义「输入」 |
| S2 | P2 | 缺行出路两种写法：要别的数的写「线程上限：N」再加已判行，单独的已判行只给确实不起重活的活；`dispatch-prompt.py` 弄坏开关说明里「书记员」改 clerk | `runner-dispatch-guard.sh` ⑮、`scripts/agents/dispatch-prompt.py` |
| S3 | P3 | heavy-thread-defaults：`\|\| X=$var` 只在同一份文件里 var 不是由登记位读法赋值时算钉数 | `.claude/gate.d/code-tooling.sh` heavy-thread-defaults 格 |
| S4 | P4 | 仓副本里正文改过的登记为 light 的自证按提交时的检查办，主 agent 在集成副本里带 `SINGLEFS_HEAVY_TESTS=commit` 跑、不弹窗；钩子这一支的说明同义 | `.claude/rules/implementation-workflow.md`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh` |
| S5 | P5 | 叫停登记出处：原话命中几处而行号提示落在其中一处取那一行；原话找不到而行号提示那一行还有这一轮的名字取那一行，往下照判；两份文件头与库同义；规则写「原话所在的那一行含这一轮的名字」「判决文件」写 `<轮>-main-verification.md` | `.claude/hooks/lib_abandoned_rounds.py`、`runner-dispatch-guard.sh` ⑥、`.claude/gate.d/doc-process-records.sh` abandoned-rounds 格、`.claude/rules/three-way-inference.md`、`.claude/agent-common.md`、`.claude/main-agent.md` |
| S6 | P6 | doc-text：date-once 红行与 term-renames 转出的 sweep-term 输出路径经打码；clock-times、prime-marks、session-names 的片段两头落在私有值里就挪到值的边上、整值进片段再打码 | `.claude/gate.d/doc-text.sh` |
| S7 | P7 | 多机只照规范句，提示里别处写的台数、线程取法一概不照；豁免表「基准那一版没有这张表时不比；重钉提交进基准之后不再红」 | `.claude/agent-common.md`、`.claude/main-agent.md`、十份定义、`.claude/gate.d/stage-owners.tsv` |

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/implementation-workflow.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b9-r2/repo/`（集成副本标签 `after-b10-fixup`，没有 `.git`）；这一题动过的文件的 sha256 在 `research/prompts/m3-defs-b9-r2-snapshot/frozen-copy-sha256.txt`。第一轮冻结副本（`after-b9-fixup`）在 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/`。腿只读这两份冻结副本，被判字面只取冻结副本；做法照主工作区的共用约束。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

## 二、实现今天的样子（主 agent 的观测，2026-10-13 现查）

- `crates/` 这一题不改。
- 本机装着 `python3-markdown-it` 3.0.0（`dpkg -l` 现查）；派发闸只在跑 Claude Code 的这一台上起。
- 集成副本复核：批 10 打上之后 `~/.local/share/singlefs/work/m3-tooling-b2-integration/verify13.summary`「派 223 项、收 223 项、判错 5 项」：`brk-long-run-exempt-needless-unchecked` 是复核脚本直接设库开关被门禁格覆盖（同 verify12）；stage-owners 缺 session-names 一行在 `after-b9-fixup` 上同样红；doc-text 在集成真树上 clock-times、term-renames 两格红，同一条命令在 `after-b9-fixup` 的树上一样红、数相同（15 行在 6 份文件、豁免表 10 条指不到），是记录内容，不是这一批；批 9 复核里两棵漂移小树（原话改了字、原话出现两次）期望红而退 0，是 P5 照设计改的（取到行号提示那一行，门禁成功句另列「照行号提示取那一行照收」）。
- 第一轮判决的另立账（Z2-甲、Z2-丙、Z7-乙、Z4-08、Z4-12）与上一题三轮的另立账不在射程里，不再交，除非 P1–P7 让它变得更糟。

## 三、要答的

- S1：markdown-it 的解析与 LLM 读派发提示的方式在哪些写法上不一致（HTML 块、链接引用定义、setext 标题、表格里的反引号、制表符缩进、`\r\n`）；库载不进的退路在哪种环境下会被触发、触发后误伤多少现有派发提示。
- S2：两种出路之间有没有一种写法两头都不落。
- S3：var 来源判法的边界（var 在函数里赋、在另一份 source 进来的文件里赋、赋两次、条件赋）。
- S4：「按提交时的检查办、不弹窗」与「重型提交之外先弹窗」两条在哪种命令上撞。
- S5：行号提示参与之后，漂移、重复、改字三种日常改动各回到什么判定；攻方第一轮写的弱点（提示那一行恰好是另一句带轮名与「」的话）在真记录里能不能造出来。
- S6：除挪片段与路径打码之外还剩哪些输出通道。
- S7：定义说「一概不照」之后，钩子判不到的那一半由谁判。

## 四、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端辩方（Sonnet）** | 复核第一轮判决：P1–P7 落到冻结副本里，是不是对第一轮两腿每一条打中都修到了（拿第一轮两腿报告 `research/prompts/m3-defs-b9-r1-sonnet-output.md`、`-opus-output.md` 与模型目录里的喂法重喂），有没有改过头（第一轮判对的行为被改坏）；第一轮哪一格判重、判轻 | S1–S7 | 逐条「修到 / 没修到 / 改过头」与喂法、改前改后退出码；门禁格起不了的写成小树与命令标「要主 agent 跑」 |
| **云端攻方（Opus）** | 造工作流历史攻 P1–P7 本身与它们带进来的新机制 | S1、S3、S5、S6、S7 | 每条历史写成可复现的步骤，在冻结副本拷贝里真跑钩子与脚本的判定；每条打中在第一轮冻结副本上也喂一遍，写明是这一批带进来的还是之前就中；第一轮已攻过的角度不再交，除非改后又中 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留 | 两条腿在那一格都给不出没修到、改过头或新的判错历史 |
| 一格判「站不住」⇒ 改法进下一批，改完开第三轮（最终轮） | 给出一条可达历史：照定义做却违反用户定案（默认 31、固定不看忙闲、只核自己的且豁免表不登记别人没审过的、私有信息不进仓、在跑的不动、仓副本自证收回放宽），或定义说拒而钩子不拒、钩子拒而定义没写 |
| 打中也打中第一轮冻结副本 ⇒ 不拿它否这批改动，另立一笔账 | 同一段历史在 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/` 上给出同样的错 |
| 辩方报「改过头」⇒ 照「站不住」办 | 第一轮判对的一段历史，改后判错 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b9-r2-<腿名>-output.md`（辩方 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b9-r2-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b9-r2-<腿名>-model/`（含 `tools/rerun.sh`，从 `scripts/agents/three-way-rerun-template.sh` 拷来、只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh` 时上限给 2G；要的环境变量写进 rerun.sh 的默认值；`--work` 的草稿目录要在 `~/.local/share/singlefs/work/` 底下；输出里的绝对路径先换占位再截断）。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数；引主工作区 `research/prompts/` 下的判决与产物写仓内相对路径。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」；造的小树要齐。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
