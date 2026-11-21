# 进仓文字不写任务号、会话短号与批号：治理文件轮第二轮（no-task-numbers-governance-r2，2026-11-19）

第一轮判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md`：两条腿都打中，改法六条（「三、改法」）全被攻过零轮。这一轮攻第一轮的修补，加第一轮只抽查过、没看过的地方；第一轮已攻过而没打中的角度不重复（见「已攻过的角度」）。另派一条辩方腿复核第一轮判决。方案按 `crates/` 今天的实现来谈。

用户定案：进仓的文字（`records/`、kb、判决、commit message）不写任务服务的任务号、会话短号与会话全名、正文里的批号（别的会话的与自己会话的都算），改写那件事本身。原话在 `records/2026-11-19-进仓文字不写任务号与会话短号.md` 第 5 行与第 25 行。

## 问题

1. 第一轮的修补本身（攻方腿）：改法第 1 到 6 条换上之后，新判据行、新上下文规则（「A/B 读数：」锚行首且只在判决里、「长跑拆分：」与表格行只在 `records/`、攒批放宽到同一分句、括号短号按目录分）、围栏读插入点前文的接法，有没有新的漏拒、误拒与绕开；出路照着改能不能过闸。
2. 第一轮判决够不够得着（辩方腿）：第一轮判的每一条打中，是不是同样打中被它取代的写法；主 agent 在「交用户的」表里定的三处判法（结局行只写 `type=` 与 `exit=`、反引号里的任务号照判、表格行批号只在 `records/` 放过）有没有站不住的；被判出局的判法（第一轮之前的放行口子）有没有被冤枉的。

## 实现今天的样子

主 agent 的观测（换上之后主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码；`grep -rn "task-and-session-numbers" crates/` 零命中。
- 第一轮改法的工具员报告 `~/.local/share/singlefs/work/no-task-numbers-round-one-fixes/result.md`（每条改法的判法、样本、开关与工具员自己写明的取舍在「每条改法：判法、样本与开关」与「判决与验收之外要主 agent 知道的」两节）。改前改后 diff：脚本与钩子 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/round-one-fixes.diff`，三份规则 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/round-one-rules.diff`。改前原样在 `~/.local/share/singlefs/work/no-task-numbers-round-one-fixes/originals/`。
- 被改的：`.claude/gate.d/lib-forbidden-notations.py`、`.claude/hooks/write-guard.sh`、`.claude/hooks/bash-command-detector.sh`、`scripts/kb/lib_notation_check.py`、`scripts/kb/append-record.py`、`scripts/kb/replace-once.py`、`scripts/kb/insert-row.py`、`scripts/kb/replace-batch.py`、`scripts/kb/ledger-append-section.py`、`scripts/agents/verdict-skeleton.py`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/long-run-split.md`、`.claude/gate.d/break-switch-replay-files.tsv`。
- 读数（日志在 `/home/fy5090/.local/state/singlefs/task-server/logs/`）：换上前草稿新判据库验收「name=acceptance_summary cases=53 disagreements=0」；第一轮攻方 67 条探针对新库与换上后的主工作区各复跑一次「name=probe_summary cases=67 disagreements=0 rule_vs_library_differs=15」（探针一律当 `records/probe.md` 判，15 条与第一轮攻方写的「规则」列不同是照新判法：表格行与「长跑拆分：」在记录里放过、括号步号在记录里照拒、「A/B 读数」不在行首或不在判决里照拒、几条判决没要求收的仍放过）；换上后写闸自证「查了 410 种情形」、检出钩子自证都退 0；开关逐条与开关复跑格六份都逐格相同。新库上攻方模型的误拒计数：kb `class=batch-size hits=2`（第一轮 31）、kb 不再有 `chinese-ordinal` 行；`records` 下 `class=bare-short-code hits=131`。
- 工具员自己写明的、没被攻过的：攒批那一条从「前面紧挨」放宽到「同一分句里前面有」；`records/` 下不挨会话词的括号短号照判（记录里写步号要放反引号）；⑰ 只在追加写法读目标原文，同一条命令里先 `>` 再 `>>` 写同一份的读的是起跑前那一份；四格缩进的代码块仍当正文判；Edit 的 `replace_all` 每处各取一段前文。

## 已攻过的角度（不重复）

第一轮攻方攻过：用户点名四种写法的近似变体漏拒、kb 与判决里正常行的误拒、「A/B 读数：」与表格行与反引号与半截围栏几处放行口子、出路照改能不能过闸。第一轮正推核过：共用约束新文字对用户原话、全仓要求写号的句子、保留两处写法与闸的认法。这一轮只攻这些角度上的修补与新冒出来的面。

## 分工

| 腿 | 攻什么 |
|---|---|
| 攻方（`three-way-attack`） | 问题 1：改法第 1 到 6 条换上之后的新判据行、新上下文规则与围栏前文接法；第一轮只抽查过的地方（写闸 Write / Edit 第一轮攻方没实测、`insert-row.py` 与 `ledger-append-section.py` 的接法、骨架 `scripts/agents/verdict-skeleton.py`、`.claude/hooks/runner-dispatch-guard.sh` 与判据库对「A/B 读数」一行认法是不是真同形） |
| 辩方（`three-way-defense`） | 问题 2：复核第一轮判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md` 每一条够不够得着、是不是同样打中替代方案；「交用户的」表三处判法站不站得住 |
