# no-task-numbers-governance-r1：判决（2026-11-19）

正文 `research/prompts/_no-task-numbers-governance-r1-body.md`，开工快照 `research/prompts/no-task-numbers-governance-r1-snapshot/`。这是这一题的第一轮，治理文件轮，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」收。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方 opus | `research/prompts/no-task-numbers-governance-r1-opus-output.md` | 「关 1：漏拒……成立」「关 2：误拒……成立」「关 3：放行口子被借来写号 成立」「关 3 之二：拒绝说明的出路照着改能不能过闸 能过」 |
| 正推 sonnet | `research/prompts/no-task-numbers-governance-r1-sonnet-output.md` | 第 1 条两处成立（表格行放行宽过定案、反引号与围栏放行没写进规则）、一处推不出（结局行原文自带号）；第 2 条去向表各行核对、没找到新的要求写号的句子；第 3 条三处成立（「A/B 读数」规则写法与派发闸、判据库三者认法不一） |

核查命令与结局：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py no-task-numbers-governance-r1 --out /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/no-task-numbers-governance-r1-tree --repository /home/fy5090/code/singlefs`：退 1，「清单点名 12 份，倒推出 10 份、sha256 对上 10 份；倒推不出 2 份」。倒推不出的两份：`.claude/agent-common.md`（快照之后「梳理 subagent」那一路会话改过别的条目；这一轮写进的第 41–45 行主 agent 现读还在、逐字未变）与 `.claude/gate.d/break-switch-replay-files.tsv`（快照之后主 agent 自己把检出钩子那一行的复跑结论改了）。两条腿引这两份的行逐处人工核过：正推腿引 `.claude/agent-common.md` 的几处行号比此刻多出 4 行，内容对得上。
- `python3 scripts/agents/cite-check.py research/prompts/no-task-numbers-governance-r1-opus-output.md research/prompts/no-task-numbers-governance-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/no-task-numbers-governance-r1-tree --background research/prompts/_no-task-numbers-governance-r1-body.md`：「核了 17 处引文、50 处裸行号，对上 15 处，没判 52 处」；没对上的两处与「路径认不出」的各处都落在倒推不出的两份与树外的规则文件上（树只铺被判文件），主 agent 在主工作区逐处 `grep -nF` 现查，原文都在。
- 攻方腿的模型复跑：模型目录拷到 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/opus-model-copy`，交 `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-main-run <副本>`，结局 `type=succeeded exit=0`；日志行「name=probe_summary cases=67 disagreements=0」：67 条探针（48 条造的、19 条现存真行）判据库的判定与腿用 `append-record.py --dry-run` 量到的逐条相同。误拒各类的计数行：kb 里 `class=chinese-ordinal hits=88`、`class=batch-size hits=31`、`class=bare-short-code hits=22`；`records` 里 `class=chinese-ordinal hits=378`、`class=bare-short-code hits=116`；`research/prompts` 里 `class=chinese-ordinal hits=299`、`class=ab-without-colon hits=27`。这些是腿自己的启发式分类，计数里混着真批号与真短号，只当量级。
- 正推腿第 3 条主 agent 现读：`.claude/hooks/runner-dispatch-guard.sh` 的 `ZERO_ROUND_TASK_NUMBER` 要「任务 / 任务号 / task」后跟数字，`ZERO_ROUND_READING_LINE` 认行首、`A/B` 与「读数」之间任意空白、可夹 `*`；`.claude/gate.d/lib-forbidden-notations.py:221` 的 `RESERVED_TASK_NUMBER_LINE` 只认 `A/B 读数\s*[:：]`、不锚行首；`.claude/rules/three-way-inference.md:38` 写的是「A/B 读数：<任务号>」。三处认法不一，坐实。
- 主 agent 另查到一处两条腿都没列的：`scripts/agents/verdict-skeleton.py:127`、`:130` 骨架的待填提示要求判决里写任务号，照骨架写判决就会把号写进仓。

## 二、判决

两条腿都打中，逐条坐实：

1. **漏拒**（攻方关 1）：坐实。现存记录里同形、整行原样放行的写法：「任务」与数字之间夹全角括号、冒号、`#`；数字后接 `/`；单个与「与 / 和 / /」连的「同 <号>」；括号里短号后面跟话；`批 #N`、`第 N–M 批`、`第 N、M 批`；`task=<号>`。判据是派发提示关 1「造出一条放行的、在现存记录里出现过同形的写法」，编号 a 到 m 是现存真行原样放行，字面满足。
2. **误拒**（攻方关 2）：坐实。fsync 攒批参数（`seq 批 1`、`攒批 64`）、泛指次序（`第一批字节`）、括号里的步号（`（a6）`）、任务个数（`任务 1180 个`）、判决旧写法（`A/B 读数任务 12400`）原样喂库都拒。主 agent 写这份判决时，写闸也把正文里一处泛指次序的引例拒了一次，放进反引号后放行。
3. **放行口子被借**（攻方关 3、正推第 1 条第一二处）：坐实。「A/B 读数：」在行内任何位置、任何文件都放过整行；「长跑拆分：」起头的行任何文件都放过；表格行在任何文件、任何一列放过批号，宽过定案的「依赖表批次那一列」；反引号三条判据行都放过，而共用约束「号一律不写」没写这个例外；以收尾围栏开头的新内容其后整段放过（攻方实写进仓一次，现场 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-opus/probes/heredoc-half-fence.after.md`）。
4. **出路自己带号**（攻方关 3 之二 R1、R2，正推第 1 条第三处）：坐实。判据库 remedy 与共用约束都叫人写「结局行原文」，而结局行带 `task=<号>`；remedy 又叫人把命令「放进反引号」，`task-server/task log <号>` 放进反引号照过。主 agent 判：用户定案是号一律不写，结局行只写 `type=` 与 `exit=` 那一段，不抄 `task=`；反引号不是任务号的放行口子。
5. **保留两处认法不一**（正推第 3 条）：坐实，见「一、腿与核查」第四条。
6. **骨架要求写号**：坐实，主 agent 现查。
7. 正推腿第 2 条（全仓还要求写号的句子）：工具员去向表各行核对成立；腿没找到新的，主 agent 补了第 6 条那一处。正推腿「原话在第 22 行，实际在第 25 行」坐实，正文行号写错，不影响判决。

## 三、改法

全部并进下一件工具员（这一轮的改法都是腿提的或主 agent 推的，被攻过零轮，改完开第二轮攻）。这几处是治理文件，不写进产品代码，不做 A/B。

1. 任务号那一条：「任务」与数字之间另认全角括号、冒号、`#`；数字后接 `/`、`-` 再接四位以上数字的照判；「同」那一支另认「与」「和」「/」连的号；`task=<四位以上数字>` 照判；`task-server/task <子命令> <号>` 照判；后面跟「个」「条」的放过；任务号这一条在反引号里照判。零轮。
2. 会话短号：括号里两位短号后面跟「在修 / 在改 / 接了 / 已接 / 推的 / 量的」这类词也判；不挨会话词、数字字母混写的括号短号只在 `records/` 下判，kb 与 `research/prompts/` 里放过（那里是步号）。零轮。
3. 批号：`第 N–M 批`、`第 N、M 批`、`批 #N`、`批次号 N` 另认；中文数字的「第几批」不判（泛指次序）；前面紧挨 `seq`、`攒`、`追加`、`覆写`、`写入` 的放过；量词表去掉「只」。零轮。
4. 放行口子收窄：「A/B 读数：」锚行首、认法与派发闸 `ZERO_ROUND_READING_LINE` 相同、只在 `research/prompts/*-main-verification.md` 放过；「长跑拆分：」起头的行只在 `records/` 下放过；表格行放过批号只在 `records/` 下；写闸 Edit、⑰ 追加与定点写入工具判围栏时先读目标文件，算出插入点前围栏开没开着。零轮。
5. 出路与规则文字：判据库 remedy 与共用约束「结局行原文」改成「结局行的 `type=` 与 `exit=` 那一段（不抄 `task=`）」，「放进反引号」只用于不是任务号的代码字面；共用约束写明反引号与围栏只放过批号与短号形状的代码字面、任务号在里面照判；两份规则的保留写法改成「A/B 读数：任务 <号>」。零轮。
6. `scripts/agents/verdict-skeleton.py:127`、`:130` 骨架提示改成写命令与结局行的 `type=`、`exit=`，零轮改法那一条写「A/B 读数：任务 <号>」一行。零轮。
7. 正文 `research/prompts/_no-task-numbers-governance-r1-body.md` 的行号写错是冻结证据，不改，判决里记一笔。

## 交用户的

| 项 | 主 agent 的判法 | 依据 |
|---|---|---|
| 结局行原文带 `task=<号>` | 只写 `type=` 与 `exit=` 那一段，号不进仓 | 用户原话「这些任务编号 对之后的任务追溯几乎 起不到什么作用」 |
| 反引号里的任务号 | 照判，反引号只放过不是任务号的代码字面 | 同上，用户在弹窗里选的是「号一律不写」 |
| 表格行里的批号 | 只放过 `records/` 下的表格行，不细到哪一列（闸认不出是不是依赖表） | 定案只留依赖表批次那一列；列认不出，`records/` 是依赖表唯一住处 |

## 这一轮被判过的文件

- research/prompts/_no-task-numbers-governance-r1-body.md：正文，判了它引的用户原话行号写错一处（冻结、不改）。
- .claude/agent-common.md：判了「进仓的文字只写本项目的东西」那一条的新文字对用户定案：表格行放行写得宽过定案、反引号与围栏的例外没写、「结局行原文」会把号带进仓，改法第 5 条。
- .claude/rules/long-run-split.md：判了第 6 步「长跑拆分：」起头的写法与判据库放行认法一致，只在 `records/` 放过照改法第 4 条收窄。
- .claude/rules/three-way-inference.md：判了「A/B 读数：<任务号>」与派发闸要的「任务 <号>」不一致，改法第 5 条。
- .claude/gate.d/lib-forbidden-notations.py：判了三条判据行的漏拒与误拒、上下文放行的口子，改法第 1 到 4 条。
- .claude/hooks/write-guard.sh：判了第七道接法与围栏只看新内容一处漏拒，改法第 4 条。
- .claude/hooks/bash-command-detector.sh：判了 ⑰ 接法与半截围栏漏拒（攻方实写进仓一次），改法第 4 条。
- scripts/kb/append-record.py：判了只判追加文本的接法与出路，攻方用它的 `--dry-run` 量了全部探针。
- scripts/kb/lib_notation_check.py：判了 `session_name_refusal` 接法，同判据库的漏拒与误拒。
- scripts/kb/ledger-append-section.py：判了开关头注改成只点名失败行，复跑逐格相同。
- scripts/kb/replace-batch.py：判了开关头注改成只点名失败行，复跑逐格相同。
- .claude/gate.d/break-switch-replay-files.tsv：判了五行理由照复跑结论改写、不带任务号。

## 回看决策

不涉及决策：这一轮只改协作工具与共用约束，`.claude/kb/decisions/` 下的决策一条都不碰；改的是进仓文字的写法与判据库，不动任何决策的定案、射程与依据。
