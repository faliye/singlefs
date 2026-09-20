<!-- knowledge-sync -->
# decision-links-2026-09-19 阶段同步

触发文件：.claude/gate.d/43-owed-table-shape.sh、.claude/gate.d/75-decision-experiment-links.sh、.claude/gate.d/stage-owners.tsv、.claude/rules/format-evolution.md、research/scripts/decision-slim-check.py

这一阶段做成的事：欠账表进门禁（43 号加判「欠着那张里写着已还的行」「表被空行断开」，C273（小节清单不认一级标题） 挪进已还清，九行开头的「已还」改写，六个空行删掉、九行分隔符修好）；决策瘦身形态与决策—实验双向登记写进规则，新立门禁 75 号与待回填清单 `.claude/decision-links-pending`；D28（挂载期承诺量） 按新形态重写（试点），E19（defer 窗口下的假性 ENOSPC）、E82（准入的在飞合成）、E141（切换预留的挂载准入自证）、E150（回退复用被抛弃的根引用的单元） 写全影响的决策表，E148（提交固定点按两棵记录树重算） 写了 D28（挂载期承诺量） 那几行；`research/scripts/decision-slim-check.py` 核瘦身时内容没丢。

## 搜索

- `grep -rn "43-owed-table-shape\|门禁 43 号" .claude records research/scripts --include=*.md --include=*.sh --include=*.py --include=*.tsv | wc -l` → 7（门禁自己、样本、归属表与下面表里的三处；归属表那一行说的「欠账表行形状」仍成立）
- `grep -rln "实验页" .claude/agents .claude/agent-common.md .claude/main-agent.md .claude/kb/experiments.md` → experiment-designer.md、experiment-runner.md、agent-common.md（全仓 `grep -rn "实验页" … | wc -l` → 79，其余是具体实验的叙述）
- `grep -rln "决策正文" .claude/agents .claude/agent-common.md .claude/kb/decisions.md` → kb-scribe.md、decisions.md
- `grep -rn "欠检查 [0-9]\+ 条" .claude records research/scripts --include=*.md | wc -l` → 3（共享 SOP 一处实测、checks-owed.md 历史一处、records 一处）

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/tooling.md:601 | 43 号（欠账表形状） \| 新的六列欠账行接在文件末尾，而那里是「已还清」那张 4 列表 | 补了：会红的形态加「欠着那张里一格开头写已还或销账」「欠着那张中间夹空行」两种，写法加挪表与改写开头、不留空行 |
| .claude/kb/decisions.md:5 | （新增） | 补了：正文形态（四块、依据只写指针、有实验才有决策）指到 format-evolution.md 那一节与门禁 75 号 |
| .claude/kb/experiments.md:6 | （新增） | 补了：每个实验页有「### 影响的决策」一节、出了新结论要重新回看，指到规则与门禁 75 号 |
| .claude/kb/checks-owed.md:359 | （新增） | 补了：欠着的那张表里不留还清的行、表中间不许有空行，门禁 43 号判红 |
| .claude/agents/experiment-runner.md:34 | 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的 | 不改：该补「影响的决策」表与回看，但改 agent 定义要先走一轮三方（`.claude/rules/implementation-workflow.md`，门禁 72 号），交用户定要不要这一轮开；在那之前派发提示里写上，75 号的出路句也指到规则 |
| .claude/agents/kb-scribe.md:3 | 书记员：定案之后照主 agent 给的逐条规格写回 kb（决策正文、变更史、分项状态、欠账表）并跑 kb 门禁阶段 | 不改：同上一行，决策正文的新形态由派发时的逐条规格带过去，定义改不改交用户 |
| .claude/agent-common.md:60 | 定义点名的产出文件（跑前登记、腿的 output、样本、运行记录、实验页、代码与 kb 的改动）照定义写进文件 | 不改：只说产物要照定义落盘，不涉及实验页的写法 |
| records/2026-09-11-D19项6第二轮.md:12 | 同时发现欠账表 31 行混在「已还清」那张四列表里 | 不改：说的是那一次发生的事 |
| .claude/singlefs-ai-sop/rules/show-me-test.md:133 | 实测（2026-09-16，singlefs 的 kb 腐化阶段）：它报「欠检查 326 条、已还清 0 条」 | 不改：共享 SOP 里记的那一次实测，本仓不就地改 |
| records/2026-08-29-加密与日志定案.md:17 | 不变量 43 条；欠检查 34 条。 | 不改：那一天的数，是事件 |
| .claude/kb/checks-owed.md:351 | （新增） | 补了：C385（首次写要不要读回，两条已定项说反话） 与下一行 C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号），D4（校验和位置） 瘦身时查出 |
| .claude/kb/checks-owed.md:222 | D4（校验和位置）`:94` 与 `:250` | 改了：改指 D4（校验和位置） 已定项 1 射程与已定项 7 依据，瘦身后行号全变 |
| .claude/kb/checks-owed.md:82 | 无压缩实现（D4（校验和位置） 字段表压缩算法 7 位是 day-1 预留） | 改了：改指 D19（块指针的结构与宽度预算） 已定项 11 的压缩两段，字段表已挪出 D4（校验和位置） |
| .claude/kb/experiments/148-提交固定点按两棵记录树重算.md:4 | D28（挂载期承诺量） 已定项 4 逐字「它的上界押在 C83（提交固定点没人回答） 上」 | 改了：改成「定案前把上界押在 C83（提交固定点没人回答） 上」，那句已挪进变更史；decision-slim-check.py 的引文核对查出 |
| records/2026-09-19-决策瘦身与双向登记.md:1 | （新增） | 补了：两个试点的数、还没改的实验页腐烂、下一批派发提示要改的八条 |
