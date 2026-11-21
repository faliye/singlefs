# 进仓文字不写任务号、会话短号与批号：治理文件轮第三轮（no-task-numbers-governance-r3，2026-11-19）

第一轮判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md`，第二轮判决 `research/prompts/no-task-numbers-governance-r2-main-verification.md`。这是最后一轮：攻第二轮之后换上的修补，加第二轮只抽查过、没看过的地方；前两轮已攻过而没打中的角度不重复（见「已攻过的角度」）。这一轮的修补若打中，改完标「零轮」，不再开第四轮。方案按 `crates/` 今天的实现来谈。

用户定案：进仓的文字不写任务服务的任务号、会话短号与会话全名、正文里的批号（自己会话的也算），改写那件事本身；原话在 `records/2026-11-19-进仓文字不写任务号与会话短号.md` 第 5、25 行。之后又定了四件，原话在同一份记录第 33 行起的各条进展：

- 提交时门禁 doc-text 新格现在换上，`research/prompts/` 下的冻结证据不判（弹窗选「现在换上，冻结证据豁免」）。
- 进仓文字、给用户的汇报与会话之间的消息里，别的会话写 mainagent(A)、mainagent(B)，子 agent 写 subagent(A)、subagent(B)，按出场次序编字母，不再用 `<做什么>那一路的会话` 这类描述指别的会话；这是规范，不是个人偏好。
- 调度记录不另开 agent(<短号>) 的例外：任务服务自己的调度记录（仓外 `~/.local/state/singlefs/task-server/`，`task-server/task status <号>` 打出「会话 …」）已经记着哪个会话交了哪个任务。

## 问题

1. 第二轮之后的四件修补，有没有新的漏拒、误拒与能绕开的写法；出路照着改能不能过闸；规范文字与闸的判法是不是同一件事（攻方腿）。
2. 共用约束、沟通纪律与派发闸的新文字，逐句对用户四次定案的原话；第二轮判决的改法八条是不是都落了地、落得对（正推腿）。

## 实现今天的样子

主 agent 的观测（换上之后主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码；`grep -rn "task-and-session-numbers" crates/` 零命中。
- 第二轮之后换上的四件，改前改后 diff 都在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/`：
  - 第二轮改法八条：`r3-round-two-fixes.diff`，工具员报告 `~/.local/share/singlefs/work/no-task-numbers-round-two-fixes/result.md`；判据库、写闸、检出钩子 ⑰、`scripts/kb/` 五份写入工具改成整份写前写后各判一遍，判据行收紧、表格只放过表头是「批次」的列、「A/B 读数：」锚行首且只在判决里、日志路径里的号照判。
  - 门禁新格豁免 `research/prompts/`、写闸头注补点名、派发闸「A/B 读数：」那一判照判据库判：`r3-followups.diff`，报告 `~/.local/share/singlefs/work/no-task-numbers-round-two-followups/result.md`；派发闸三条开关补点名：`r3-dispatch-guard-names.diff`。
  - 别的会话改叫 mainagent(A)：`r3-agent-letters.diff`，报告 `~/.local/share/singlefs/work/no-task-numbers-agent-letters/result.md`；判据库加判据行 `session-descriptions`，共用约束 `.claude/agent-common.md` 与 `.claude/rules/writing-discipline-by-review.md`「沟通纪律」跟着改（后者 diff `r3-rules-now.diff`）。
- 读数（命令都经任务服务交，结局都是 `type=succeeded exit=0`，除注明的）：
  - 第二轮改法：换上前两条腿模型对新判据库复跑，辩方「cases=33 disagreements=0」，攻方「cases=81 disagreements=1」（不一致那一格是探针整串替换连待填提示一起换，判据库没错）；换上后写闸自证「查了 437 种情形」、检出钩子自证「查了 1506 种」；写闸开关二十条红格数与点名相同；开关复跑五份逐格相同（检出钩子 170 条、`replace-once.py` 6 条、`replace-batch.py` 11 条、`ledger-append-section.py` 12 条、`append-record.py` 10 条）。
  - 收尾三处：doc-text 样本档「110 个用例」全对、开关复跑五条逐条相等；派发闸自证「查了 539 种」，开关复跑头一次红在三条开关点名上，补点名后「复跑了 160 条开关（1 份），自证红且点名的格与实际红的格逐格相同 160 条」。
  - mainagent 那一件：doc-text 样本档「112 个用例」全对；六份自证退 0；写闸六条开关红格数与点名相同；开关复跑四份逐格相同（12、6、11、5 条），doc-text 七条逐条相等。
- 主 agent 自己撞上过的：替换脚本判整份改前改后多出来的违例行，同一行里两处只改一处，这一行仍算新违例，要一次改全；主 agent 改自己这份记录时被拒过两次。
- 门禁新格换上后，主 agent 按 HEAD 对此刻工作区量过 `research/prompts/` 之外还会被报的行，按文件发给了各会话认领；写的是别的会话还没提交的改动，不在这一轮的射程。

## 已攻过的角度（不重复）

第一轮：用户点名写法的近似变体漏拒、kb 与判决里正常行的误拒、几处放行口子、出路照改。第二轮：第一轮修补带进来的漏拒误拒（日志文件名、单个同字、ASCII 冒号、攒批、量词表、中文数字第几批、括号短号按目录分、表格任一列）、先写进围栏再拆围栏、「A/B 读数：」一行装号、派发闸与判据库认法、第一轮判决够不够得着。这一轮只攻这些角度上的修补与新冒出来的面。

## 分工

| 腿 | 攻什么 |
|---|---|
| 攻方（`three-way-attack`） | 问题 1：整份写前写后各判一遍的接法（写闸 Edit / MultiEdit、⑰ 追加、五份写入工具、doc-text 新格）、按表头认「批次」列、doc-text 新格豁免 `research/prompts/` 的边界、派发闸照判据库判那一行、`session-descriptions` 判据行的漏拒误拒；同一行几处要一次改全会不会把正常改记录的写法挡死 |
| 正推（`three-way-forward`） | 问题 2：`.claude/agent-common.md`「写」一节那一条、`.claude/rules/writing-discipline-by-review.md`「沟通纪律」那一条、派发闸拒绝说明的新文字，逐句对用户四次定案的原话；第二轮判决「三、改法」八条逐条核落地与否 |
