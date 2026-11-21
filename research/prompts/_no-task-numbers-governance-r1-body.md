# 进仓文字不写任务号、会话短号与批号：治理文件轮第一轮（no-task-numbers-governance-r1，2026-11-19）

用户定案（`records/2026-11-19-进仓文字不写任务号与会话短号.md` 第 5 行与第 22 行引的原话）：进仓的文字（`records/`、kb、判决、commit message）不写任务服务的任务号、会话短号与会话全名、正文里的批号（别的会话的与自己会话的都算），改写那件事本身：哪条命令、结局行原文、日志里哪一行、谁在改哪个文件。保留：调度记录依赖表的批次那一列（以 `|` 起头的表格行）、`.claude/rules/three-way-inference.md` 的「A/B 读数：<任务号>」一行、`.claude/rules/long-run-split.md`「怎么做」第 6 步那一行。已有记录里的旧号不回改。这一轮攻的是照这个定案改并已换进主工作区的治理文件。方案按 `crates/` 今天的实现来谈。

## 问题

1. 判据库新形态、两道钩子与几份写入脚本的判法：该拒的拒不拒、不该拒的会不会误拒（误判把正常写记录挡死），放行的口子会不会被用来绕开，拒绝说明给的出路能不能照着改过去（攻方腿）。
2. 共用约束与两份规则的改法说的是不是用户定的那几句；与没改的规则、定义、技能（要求把任务号写进仓的句子）有没有互相矛盾，有没有该改而没改的（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码；`grep -rn "task-and-session-numbers" crates/` 零命中。
- **共用约束与两份规则**（工具员在主工作区直接改，diff 在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/rules-and-tables.diff`，相对暂存区；`.claude/agent-common.md` 暂存区里另有别的会话的改动，不在这一轮）：
  - `.claude/agent-common.md`「写」一节「进仓的文字只写本项目的东西」那一条重写，⑰ 与写闸那两条描述跟着改；
  - `.claude/rules/long-run-split.md`「怎么做」第 6 步：调度记录那一行以「长跑拆分：」起头；
  - `.claude/rules/three-way-inference.md` 零轮改法那一条：任务号写成「A/B 读数：<任务号>」一行。
- **判据库与写入一侧**（工具员报告 `~/.local/share/singlefs/work/no-task-numbers-in-repo-text/result.md`，换上前后 diff 在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/scripts-and-hooks.diff`）：
  - `.claude/gate.d/lib-forbidden-notations.py` 加形态 `task-and-session-numbers`，三条判据行 `task-numbers`、`session-short-codes`、`batch-numbers`，上下文放行（反引号、「A/B 读数：」行、「长跑拆分：」起头的行、`|` 起头的表格行）与围栏代码块放行，只判 `.md`、`.tsv`、`.txt`；判法与「管不到的」写在库文件头「任务号、会话短号与批号」一节；
  - `.claude/hooks/write-guard.sh` 第七道接上，`.claude/hooks/bash-command-detector.sh` ⑰ 接上，`scripts/kb/lib_notation_check.py` 的 `session_name_refusal` 接上（`replace-once.py`、`insert-row.py`、`replace-batch.py`、`ledger-append-section.py` 经它接上），`scripts/kb/append-record.py` 接上、只判追加的文本；
  - 工具员自己写明的管不到：没带「任务」「task」「同」的裸任务号；不挨会话词的纯数字或纯字母括号短号；不带括号的会话短号；英文 batch 加数字；括号里像 `(a1)`、`(2c)` 的步号会被误判；Edit 的 new_string 只带半截围栏时认不出围栏。
- **开关头注**（工具员报告 `~/.local/share/singlefs/work/rawafterrename-switch-header/result.md`，规格 `~/.local/share/singlefs/work/rawafterrename-switch-header/spec.json`）：`scripts/kb/append-record.py`、`scripts/kb/ledger-append-section.py`、`scripts/kb/replace-batch.py` 三份 `ATOMIC_REPLACE_BREAK=rawafterrename` 的点名只留 ✗ 失败行；`.claude/gate.d/break-switch-replay-files.tsv` 五行理由照复跑结论改。
- 读数（日志在 `/home/fy5090/.local/state/singlefs/task-server/logs/`）：
  - 换上后自证都退 0：写闸、检出钩子、`append-record.py`、`replace-once.py`、`insert-row.py`、`replace-batch.py`、`ledger-append-section.py`、门禁 doc-text 样本档；
  - 写闸九条开关、`replace-once.py` 两条、`append-record.py` 两条逐条打开，红的格与工具员报告点名的逐格相同；
  - 开关复跑格点名写闸、`append-record.py`、`ledger-append-section.py`、`replace-batch.py`、`replace-once.py` 都绿；点名检出钩子那一趟 162 条里 ⑰ 的两条对上，另两条在 ⑫「Bash写范围:只读定义」9 格上不符，那几格是别的会话暂存着的改动，它已认领在改，不在这一轮。
- 换上之后主 agent 自己往这一轮的调度记录与这份正文写字时，写闸拦下过一次：正文里一句把调度记录依赖表的批号写进了正文，判成批号，改成说那一件做了什么之后放行。

## 分工

| 腿 | 攻什么 |
|---|---|
| 攻方（`three-way-attack`） | 问题 1：判据库三条判据行的正则与上下文放行、两道钩子与写入脚本的接法；能不能造出该拒而放行的、不该拒而拒的真实写法（拿现存 `records/`、kb、`research/prompts/` 里的行当样本），拒绝说明的出路照着改能不能过闸 |
| 正推（`three-way-forward`） | 问题 2：共用约束与两份规则的新文字逐句对用户原话；全仓规则、定义、技能里要求把任务号、批号、会话短号写进仓的句子还有没有没改的；保留的两处固定写法与判据库的放行认法是不是同一个形态 |
