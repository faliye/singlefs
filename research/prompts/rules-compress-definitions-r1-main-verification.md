# 判决：规则压缩复核（共用约束与 agent 定义）第一轮

日期：2026-11-16。轮名 `rules-compress-definitions-r1`。开工快照 `research/prompts/rules-compress-definitions-r1-snapshot/`（被判的原件 `judged-sha256.txt`，合稿树第一版 `merged-tree-sha256.txt`）。正文 `research/prompts/_rules-compress-definitions-r1-body.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-definitions-r1-opus-output.md`，正推 `research/prompts/rules-compress-definitions-r1-sonnet-output.md`。两条腿交回时各自跑过 `scripts/agents/cite-check.py`，末行都是「对上」、没有对不上的引文；主 agent 这一轮没有另跑 `snapshot-tree.py`（任务服务排着 91 条、小任务起不来），改成对腿点名的行逐处用 grep 与 sed 现读，现读的结果记在 `records/2026-11-16-三方豁免取消与存量补轮.md`「主 agent 现查坐实的」一段。
- 腿的说法：攻方打中 7 条，正推 15 份逐条过完（582 行落点）、报丢 1 处边界与 1 处弱、二义 2 处。
- 被判的原件在腿跑着的时候没有被这一轮改过；腿中途收到的一条消息只是合稿树里两处路径改名，内容没变（两条腿都重核了 sha256）。
- 两条腿都没有写模型，没有要主 agent 复跑的数。

## 二、判决

「压缩之后规则一条没丢、没有二义、没有悄悄改做法」在第一版合稿上不成立。逐条：

1. 逐个按进程号停的写法正是钩子今天判红的写法：主 agent 读 .claude/hooks/bash-command-detector.sh 文件头第 166 到 196 行坐实，交用户，用户定案第 20 条改定只许对自己起的组发，那一节照原件恢复。
2. 「点到就整份读」两种读法：写成两句。
3. 「想拿旧数据支撑新结论，先复跑」只剩两跳指针：两条腿都报，判据句放回共用约束。
4. 丢了「写成交接的副本，子 agent 交回时不列」：放回。
5. 实现员命名检查的 <根> 两处写法不一：照原件分开写。
6. 调查员「查到就修」那一支：用户定案第 19 条，写明是例外。
7. 执行员例外只写了吃紧：用户定案第 22 条，补卡住。

其余打中的与腿的观察共同进了修稿的逐条去向表 `research/prompts/rules-compress-r2-merged/dispositions.md`（五题合计 129 条：成立 110、不成立 11、部分成立或只涉及脚本一侧 8）；主 agent 读了其中不采纳的 14 行，理由是那句话在脚本文件头而技能写了以文件头为准、或是不带判据的说明句，照收。

## 三、改法

改法都落在第二版合稿树 `research/prompts/rules-compress-r2-merged/`，相对第一版的差在同目录 `diff-from-r1.patch`。腿提的改法只当线索，丢的句子照原件原话放回。这些修补被攻过零轮，第二轮只攻它们。

## 交用户的

这一题里交用户、用户已定的条目在调度记录里编号 17 到 22；没有留着没定的。

## 这一轮被判过的文件

- .claude/agent-common.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/clerk.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/experiment-designer.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/experiment-runner.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/implementation-writer.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/investigator.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/kb-scribe.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/mutation-triage.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/prior-art.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/sweep.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/three-way-attack.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/three-way-defense.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/three-way-forward.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/three-way-materials.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agents/tooling-writer.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。
