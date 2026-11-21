# no-task-numbers-governance-r2：判决（2026-11-19）

正文 `research/prompts/_no-task-numbers-governance-r2-body.md`，开工快照 `research/prompts/no-task-numbers-governance-r2-snapshot/`。这是这一题的第二轮，治理文件轮；第一轮判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方 opus | `research/prompts/no-task-numbers-governance-r2-opus-output.md` | 关 1 五格成立（同字连号误拒一行 kb、日志与目录名里的号放过、攒批同一分句放过调度批号、量词表把「处理 / 行为 / 项目」当量词、任务号后接「条件」放过）；关 2 三格成立（先写进围栏再拆围栏、「A/B 读数：」一行装无关的号、「长跑拆分：」一行同理），一格推不出；关 3 四格成立（replace-batch 的 count 只按头一处前文判、insert-row 插围栏行、两份写入工具出路写成「会话号」、派发闸收而判据库在判决里拒的 A/B 读数四条），一格推的，一格没打中 |
| 辩方 sonnet | `research/prompts/no-task-numbers-governance-r2-sonnet-output.md` | 第一轮判决第 1、2、3 条与第 4、7 条轻度成立（改法够不着判决自己列的单个「同 <号>」与 ASCII 冒号；括号步号放过也放过了判决里的真短号；中文数字的第几批在记录里是真批号；表格行只缩到 `records/` 任何列；`log=` 文件名带号）；第 5、6 条不成立；「交用户的」表第三处成立、前两处不成立；判决无去向四项 |

核查命令与结局：

- 开工快照：`sha256sum -c research/prompts/no-task-numbers-governance-r2-snapshot/judged-sha256.txt` 只有 `.claude/gate.d/break-switch-replay-files.tsv` 对不上（开工之后被改过），两条腿都没引它；其余 15 份与开工时相同，所以没有倒推树，`cite-check` 直接以仓根为根。
- `python3 scripts/agents/cite-check.py research/prompts/no-task-numbers-governance-r2-opus-output.md research/prompts/no-task-numbers-governance-r2-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_no-task-numbers-governance-r2-body.md`：「核了 46 处引文、99 处裸行号，对上 46 处」。
- 辩方模型拷到 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/r2-sonnet-model-copy` 复跑，结局 `type=succeeded exit=0`，「name=probe_summary cases=33 disagreements=0」；表格行的数「name=table_header_position last=118 not_last=114」「name=table_batch_cells in_batch_column=52 in_other_columns=559」。
- 攻方模型拷到 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/r2-opus-model-copy` 复跑，结局 `type=command-failed exit=1`，「name=case_summary cases=71 disagreements=1」：不一致的一格是「骨架:照待填提示填上读数与 A/B 读数之后整份 Write」，腿写 pass、复跑 refuse，拒在写闸「账本追加这一道没判：读不到 …/fake-root/.claude/gate.d/split-proposal…」，是腿造的假仓里少一份账本登记表、写闸另一道读不到，不是号的判据；腿自己把这一格标「待主 agent 跑」。主 agent 判这一格推不出，不当打中也不当没打中；判据库对骨架那一行放不放，腿另一格（3e 照骨架填好的一行）量的是放过。其余 70 格与腿一致；「name=scan_class class=log-path-task-number-passed」三行合计 45 行（`records` 15、`.claude/kb` 1、`research` 29），「class=clause-batch-size-exempt-in-records dir=records lines=1」，「class=same-joined-refused-outside-records dir=.claude/kb lines=1」。
- 攻方 2a 真写进仓的现场在它模型目录 `research/prompts/no-task-numbers-governance-r2-opus-model/live/`，落盘后挪到草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-opus/live-moved-out-of-repo/`；主 agent 现查仓里模型目录只剩 `live-base/` 与 `tools/`。

## 二、判决

两条腿都打中，逐条坐实（攻方的复跑数与辩方的复跑数都由主 agent 在副本里跑出，见上）：

1. **日志、目录名里的任务号放过**（攻方 1b、辩方第 4 条）：坐实。第一轮改法叫人写「日志里的那一行」，引日志时连文件名一起抄，文件名带号。判：日志文件名与运行目录名里的任务号同样照判，出路改成写日志里那一行的内容、不写日志路径。
2. **先写进围栏、再拆围栏**（攻方 2a、3a、3b、3c）：坐实，五种写法真写进过仓。第一轮只看插入点之前的文字，看不到写完之后整份的样子。判：写入一侧改成整份写前、写后各判一遍，写后多出来的违例行就拒；第一轮的「插入点前文」接法由它取代。
3. **改法够不着判决自己列的形**（辩方第 1 条）：坐实。单个「同 <号>」、号后接 ASCII 冒号仍放过；「个 / 条」放宽带进任务号后接「条件」（攻方 1e），「条」在现存真行里零处用到。
4. **攒批同一分句放宽、量词表**（攻方 1c、1d）：坐实。同一分句放宽在 `records/` 里放过一条调度批号；量词表把「处理」「行为」「项目」的头一个字当量词。
5. **中文数字的第几批与括号短号按目录分**（辩方第 2 条）：坐实。中文数字的第几批在 `records/` 里是真调度批号（例 `records/2026-11-18-等GPU拆到锁外与前台等锁让路.md:59`）；括号短号在 `research/prompts/` 一律放过，也放过了判决文件里的真短号。判：中文数字的第几批在 `records/` 下照判；括号短号在判决文件（`research/prompts/*-main-verification.md`）里同 `records/` 一样照判。
6. **表格行批号只缩到 `records/` 任何一列**（辩方第 3 条、「交用户的」第三处）：坐实，`records/` 下表格里批号落在别的列的 559 个、落在「批次」列的 52 个。第一轮「闸认不出列」不成立：判写后整份时表头读得到。判：只放过表头那一列是「批次」的单元格。
7. **「A/B 读数：」一行装无关的号**（攻方 2b）与派发闸认、判据库拒的四条（攻方 3f）：坐实。判：放过的只是这一行值里、按派发闸认法认出的任务号，至多两个（A/B 两臂），引用块（`>` 起头）不放；判据库与派发闸对同一行判法逐条相同，攻方 3f 的四条进自证。
8. **「长跑拆分：」一行装无关的号**（攻方 2c）：坐实但不收窄。`.claude/rules/long-run-split.md` 第 6 步本来就要这一行写项名、任务号与拆后各项重跑的号，几个号都合规，闸分不出哪个号无关；留作已知的缝，写进判据库文件头「管不到的」。
9. **出路写成「会话号」**（攻方 3d）：坐实，`scripts/kb/replace-batch.py` 与 `scripts/kb/ledger-append-section.py` 拒任务号与批号时出路那一句照会话号写。
10. **判决里没去向的**（辩方第 3 问）：坐实四项。`#第 N 批` 锚点放过、没有提交时的格（`cp`、`mv` 与别的命令的输出绕过写闸与 ⑰）、`records/` 下 `.json` 不判、会话词表缺「范围 / 判过」。判：锚点不再放过（派发提示不进仓，仓里不该有这种行）；门禁 doc-text 加一格只判相对 diff 基准新加的行，补上绕过钩子写进仓的那一条路；`.json` 是机读台账、不是进仓文字的射程，不判，写进判据库「管不到的」；会话词表补上。
11. **第一轮判决自身的错**（辩方第 2、3、7 条）：判决旧写法（「A/B 读数」后面不带冒号直接跟任务号）不是误拒（规则要冒号，改法换上后照拒，第一轮判决把它列进误拒是错的）；第一轮把「号一律不写」写成共用约束的话，实际只在用户弹窗选项里；正推腿那一格的引号句是主 agent 的转述。这三处记在这里，第一轮判决不回改。
12. **「交用户的」表前两处**（辩方第 2 问）：结局行只写 `type=` 与 `exit=`、反引号里的任务号照判，辩方判不成立，维持。反引号仍放过批号与会话短号形状的代码字面，理由：它们与代码字面（`seq 批 1`、步号）字面上分不开，反引号是写的人声明「这是代码字面」的唯一出路；任务号没有这种同形的代码字面。
13. **攻方 2d、3e**：推不出、没打中，不采。

## 三、改法

全部并进下一件工具员，改完开第三轮（最后一轮）。这几处是治理文件，不写进产品代码，不做 A/B。

1. 任务号：日志文件名与运行目录名（`task-<十六进制>-<号>` 一类）里的号照判；单个「同 <号>」照判，后面跟量词或单位的放过；号后接 ASCII 冒号再接空白或行尾的照判；量词放宽只留「个」。零轮。
2. 批号：攒批放宽收回到「前面紧挨」，`records/` 下不放宽；量词表里会作双字词头一个字的（处、行、项……）后面接成词的不算量词；中文数字的第几批在 `records/` 下照判；`#第 N 批` 照判；表格行只放过表头那一列是「批次」的单元格。零轮。
3. 会话短号：判决文件里同 `records/` 一样照判；会话词表补「范围」「判过」。零轮。
4. 「A/B 读数：」一行：只放过值里按派发闸认法认出的任务号、至多两个，引用块不放；攻方 3f 四条进自证，两边判法相同。零轮。
5. 写入一侧：写闸、⑰、`scripts/kb/` 五份写入工具改成整份写前写后各判一遍，写后多出来的违例行就拒，取代插入点前文接法；replace-batch 的 count 每一处都在内。零轮。
6. 出路：两份写入工具的出路照任务号与批号写；判据库与共用约束的出路写明不抄日志路径。零轮。
7. 门禁 doc-text 加一格：相对 diff 基准新加的 `.md`、`.tsv`、`.txt` 行照这一形态判（同 `scripts/code-name-lint.py` 只判新加行的做法），补上 `cp`、`mv` 与命令输出写进仓的路。零轮。
8. 判据库文件头「管不到的」写进「长跑拆分：」一行装几个号、`.json` 不判两条。

## 交用户的

| 项 | 主 agent 的判法 | 依据 |
|---|---|---|
| 表格行里的批号 | 只放过表头那一列是「批次」的单元格（第一轮是 `records/` 下整行放过） | 用户定案只留依赖表批次那一列；第二轮辩方量到别的列里 559 个批号 |
| 日志文件名里的任务号 | 照判，引日志写那一行的内容、不写日志路径 | 用户原话「这些任务编号 对之后的任务追溯几乎 起不到什么作用」 |
| 提交时的门禁格 | 门禁 doc-text 加一格，只判新加的行，旧记录不回改 | 钩子绕得过（`cp`、`mv`、命令输出），漏过就留在仓里 |

## 这一轮被判过的文件

- research/prompts/_no-task-numbers-governance-r2-body.md：正文，判了它列的第一轮修补与「已攻过的角度」是不是这一轮的射程。
- research/prompts/no-task-numbers-governance-r1-main-verification.md：辩方复核，判了七条判决与「交用户的」三处，三处判决自身的错记在本判决第 11 条。
- .claude/gate.d/lib-forbidden-notations.py：判了新判据行的漏拒与误拒（日志文件名、单个同字、ASCII 冒号、攒批同一分句、量词表、中文数字第几批、括号短号按目录分、表格行任一列、A/B 读数一行装号、锚点），改法第 1 到 4、8 条。
- .claude/hooks/write-guard.sh：判了 Edit / MultiEdit 的插入点前文接法挡不住先写进围栏再拆围栏，改法第 5 条。
- .claude/hooks/bash-command-detector.sh：判了 ⑰ 一条命令两次追加先写进围栏再拆围栏，改法第 5 条。
- scripts/kb/lib_notation_check.py：判了定点写入工具共用的前文接法，改法第 5 条。
- scripts/kb/append-record.py：判了追加的前文接法，改法第 5 条。
- scripts/kb/replace-once.py：判了两步替换先写进围栏再拆围栏，改法第 5 条。
- scripts/kb/insert-row.py：判了插一行围栏让后面的行出围栏，改法第 5 条。
- scripts/kb/replace-batch.py：判了 count 只按头一处前文判与出路写成会话号，改法第 5、6 条。
- scripts/kb/ledger-append-section.py：判了出路写成会话号，改法第 6 条。
- scripts/agents/verdict-skeleton.py：判了照骨架填好的 A/B 读数一行过不过闸（放过）与整份写（攻方假仓少一份登记表，推不出）。
- .claude/agent-common.md：判了出路「日志里的那一行」把人引向日志文件名，改法第 6 条。
- .claude/rules/three-way-inference.md：判了「A/B 读数：任务 <号>」写法与派发闸、判据库三处认法，攻方 3f 四条要两边判法相同，改法第 4 条。
- .claude/rules/long-run-split.md：判了「长跑拆分：」一行能装几个号，判为合规、留作已知的缝。
- .claude/gate.d/break-switch-replay-files.tsv：判了开工之后被改过、两条腿都没引它。

## 回看决策

不涉及决策：这一轮只改协作工具与共用约束，`.claude/kb/decisions/` 下的决策一条都不碰；改的是进仓文字的写法与判据库，不动任何决策的定案、射程与依据。
