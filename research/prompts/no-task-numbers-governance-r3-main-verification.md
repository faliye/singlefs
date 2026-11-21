# no-task-numbers-governance-r3：判决（2026-11-19）

正文 `research/prompts/_no-task-numbers-governance-r3-body.md`，开工快照 `research/prompts/no-task-numbers-governance-r3-snapshot/`。这是这一题的第三轮，也是最后一轮（治理文件轮，`.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」最多三轮）；前两轮判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md`、`research/prompts/no-task-numbers-governance-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方 opus | `research/prompts/no-task-numbers-governance-r3-opus-output.md` | 成立：1a、1b、1c、1e、2a、2b、2e、2f、3b；不成立：1d、2c、2d、2g、3a；3c 归 1c |
| 正推 sonnet | `research/prompts/no-task-numbers-governance-r3-sonnet-output.md` | 关 1 成立三处、推不出四处、轻四处；关 2 八条都落地；关 3 轻三处 |

核查命令与结局：

- 开工快照：`sha256sum -c research/prompts/no-task-numbers-governance-r3-snapshot/judged-sha256.txt` 只有 `.claude/hooks/write-guard.sh` 对不上，开工之后 mainagent(A) 经 `scripts/gate/hook-swap.py` 换过一次；攻方模型的 `rerun.sh` 开跑先核它量的那几份 sha256，复跑报「name=judged_check changed=0」，腿量的就是换过之后那一份。其余 15 份与开工时相同。
- `python3 scripts/agents/cite-check.py research/prompts/no-task-numbers-governance-r3-opus-output.md research/prompts/no-task-numbers-governance-r3-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_no-task-numbers-governance-r3-body.md`：「核了 77 处引文、50 处裸行号，对上 77 处」。
- 攻方模型拷到 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/r3-opus-model-copy` 复跑，结局 `type=succeeded exit=0`，「name=case_summary cases=34 disagreements=0」「name=scan_session_descriptions files=1080 hit_lines=92」「name=scan_description_variants_without_named_word occurrences=373」。复跑后主 agent 现查，仓里模型目录只剩 `tools/`，没有探针留下。攻方真写进仓的 11 份探针现场在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-opus/live-moved-out-of-repo/`。
- 正推探针 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-sonnet/probe_items.py` 经服务交，结局 `type=command-failed exit=1`，「name=probe_summary cases=49 disagreements=33」。主 agent 读逐条行：33 条里 32 条是探针把判据行键名（`task-numbers`）与判据库返回的显示名（「任务服务的任务号」）当成两件事比，拒放结局与预期相同；只按拒放比，真正对不上的 1 条是「描述:不带那一路的描述（规则文字说这类描述）」，预期拒、判据库放，与腿报告预测的那一条相同。

## 二、判决

两条腿都打中，逐条坐实（攻方复跑一致，正推那一格按拒放现查一致）：

1. **⑰ 认不出拆开写的号**（攻方 1a）：坐实。`printf` 格式串与参数分开、同一条命令里先赋值的变量、`echo -e` 转义，三种都真写进过模型目录。与攻方 2b（门禁新格不判 `research/prompts/`）合起来，往判决里拆开写的号三道都不判。
2. **近似写法漏拒**（攻方 1c、3b，正推关 1 第三处）：坐实。括号里换成两位短号的 `mainagent(<短号>)`、用户已撤回的 `agent(<短号>)`、「会话」后接短号、中文数字的批号（截止批次为三、批三）、「任务编号 <号>」「<号> 号任务」；「那一路」后接回信、报、认领、转达这类动作词，现存记录与 kb 里 92 行（复跑量的）。规则写「这类描述」都不写，判据只认「那一路」加会话词，两边不是同一件事。
3. **门禁新格对搬家与改名的文件把基准取成空串**（攻方 1e）：坐实为推的，代码里取基准只按此刻路径，主 agent 现读门禁 doc-text 那一格的 `git cat-file -p <gate 基准>:<路径>` 取法确认。
4. **表格里转义的竖线**（攻方 2a）：坐实，量的。
5. **列表里的引用块**（攻方 2f）：坐实，量的；规则写引用块里的不算。
6. **沟通纪律把「任务号与批号也不写」扩到对话**（正推关 1 第一处）：坐实。用户定案的射程是进仓文字，对话那一半只定了叫法；且与共用约束要派发提示照写批次、任务号的那几条相撞。
7. **两份规范说法不一**（正推关 1 第二处）：坐实，「同一份文字里」与「同一段话里」、`那一路的 agent` 与「不换别的说法」只在一份里。
8. **规范文字带了解释**（正推关 3）：坐实，「带着任务号」「A/B 两臂」两处是解释，规则只写怎么做。
9. **相抵只看行文**（攻方 1b）：坐实但不改：旧行出围栏、离开批次列不拒，带不进新的号文字；写进判据库文件头「管不到的」。
10. **两臂分两行**（攻方 2e）：坐实但不改：规则要一行写；第二行单起照规则写就放过，写进拒绝说明的出路。
11. **门禁新格豁免整个 `research/prompts/`**（攻方 2b、正推关 1 推不出第四处）：维持，用户弹窗选了「现在换上，冻结证据豁免」；被它放过的那条路靠第 1 条补 ⑰ 关上。
12. **正推推不出与轻的几处**：「不拿描述指东西」比原话宽、「不换别的说法」与 `agent(A)` 两组叫法的关系、「会话之间的消息」只见于转述——三处照第 6、7 条一起改，规范只写用户定的叫法；派发闸拒绝说明只写「任务 <号>」而判法另认 `task <号>`，照判法补写；「长跑拆分」与「批次列」的射程写成 `records/` 下；`.claude/skills/dispatch/SKILL.md` 第 20 行「认会话看变异行与记录里的名字」改指 `ListAgents` 与任务服务的调度记录。
13. **攻方 1d、2c、2d、2g、3a**：不成立，不采。

## 三、改法

这是最后一轮，下面的改法都被攻过零轮，改完照常换上、自证、开关证红与开关复跑，不再开第四轮。这几处是治理文件，不写进产品代码，不做 A/B。

1. ⑰ 照拆开的写法判：`printf` 格式串代入同一条命令里的字面参数后判、`echo -e` 展开转义后判、同一条命令里先赋值的字面变量代入后判。零轮。
2. 判据行：`mainagent(...)`、`subagent(...)` 括号里只放大写字母；`agent(<短号>)`、「会话」后接短号照判；`records/` 下中文数字的批号照判；「任务编号 <号>」「<号> 号任务」照判；`session-descriptions` 另认「那一路」后接回信、报、认领、转达、在改、说这类动作词，说跑法的「按块那一路」「变异那一路」照放。零轮。
3. 门禁 doc-text 新格取基准时认搬家与改名（`git diff -M` 的旧路径）。零轮。
4. 表格转义的竖线不算列分隔；列表里的引用块照引用块算。零轮。
5. 规范文字：`.claude/rules/writing-discipline-by-review.md`「沟通纪律」那一条只写叫法（别的会话写 mainagent(A)、子 agent 写 subagent(A)，按出场次序编字母），删掉对话里任务号与批号那一句；两份规范说法照 `.claude/agent-common.md` 统一；删「带着任务号」「A/B 两臂」两处解释；「长跑拆分」「批次列」写成 `records/` 下；派发闸拒绝说明补写 `task <号>`；`.claude/skills/dispatch/SKILL.md` 第 20 行改指 `ListAgents` 与任务服务的调度记录。零轮。
6. 判据库文件头「管不到的」写进第 9、10 条两处与「那个会话」这类泛称不判。零轮。

## 交用户的

| 项 | 主 agent 的判法 | 依据 |
|---|---|---|
| 对话里写不写任务号与批号 | 不在规范里管，规范对对话只管叫法 | 用户定的射程是进仓文字；对话只定了叫法（「这不是偏好 而是 规范」那一句说的是叫法） |
| 「那个会话」这类泛称 | 判据库不判，写进「管不到的」 | 现存 373 处，与正常行文分不开 |
| 门禁新格豁免整个 `research/prompts/` | 维持 | 用户弹窗选「现在换上，冻结证据豁免」 |

## 这一轮被判过的文件

- research/prompts/_no-task-numbers-governance-r3-body.md：正文，判了它列的四件修补与已攻过的角度是这一轮的射程。
- research/prompts/no-task-numbers-governance-r2-main-verification.md：正推核了「三、改法」八条，都落地。
- .claude/gate.d/lib-forbidden-notations.py：判了近似写法漏拒、表格转义竖线、列表里的引用块、相抵只看行文，改法第 2、4、6 条。
- .claude/hooks/write-guard.sh：判了整份写前写后接法（攻方复跑一致，没新打中），开工之后被别的会话换过一次、腿量的是换后那一份。
- .claude/hooks/bash-command-detector.sh：判了 ⑰ 认不出拆开写的号，改法第 1 条。
- .claude/hooks/runner-dispatch-guard.sh：判了与判据库对同一行逐条结局相同、两臂分两行、拒绝说明少写一种认法，改法第 5 条。
- scripts/kb/lib_notation_check.py：判了经它接上的几份写入工具的写前写后接法，没新打中。
- scripts/kb/append-record.py：判了追加的写前写后接法，没新打中。
- scripts/kb/replace-once.py：判了同一行几处要一次改全会不会挡死正常改法，有出路。
- scripts/kb/insert-row.py：判了插行的写前写后接法，没新打中。
- scripts/kb/replace-batch.py：判了按文件写前写后接法，没新打中。
- scripts/kb/ledger-append-section.py：判了账本追加的写前写后接法，没新打中。
- scripts/agents/verdict-skeleton.py：判了骨架那一行与派发闸、判据库认法，没新打中。
- .claude/gate.d/doc-text.sh：判了新格豁免 `research/prompts/` 的边界与搬家改名的基准，改法第 3 条。
- .claude/agent-common.md：判了新文字对用户定案，两份规范说法不一与两处解释，改法第 5 条。
- .claude/rules/writing-discipline-by-review.md：判了「沟通纪律」那一条把任务号与批号扩到对话，改法第 5 条。

## 回看决策

不涉及决策：这一轮只改协作工具与共用约束，`.claude/kb/decisions/` 下的决策一条都不碰；改的是进仓文字的写法与判据库，不动任何决策的定案、射程与依据。
