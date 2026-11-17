# 判决：规则压缩复核（其余项目规则与技能）第一轮

日期：2026-11-16。轮名 `rules-compress-residents-r1`。开工快照 `research/prompts/rules-compress-residents-r1-snapshot/`（被判的原件 `judged-sha256.txt`，合稿树第一版 `merged-tree-sha256.txt`）。正文 `research/prompts/_rules-compress-residents-r1-body.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-residents-r1-opus-output.md`，正推 `research/prompts/rules-compress-residents-r1-sonnet-output.md`。两条腿交回时各自跑过 `scripts/agents/cite-check.py`，末行都是「对上」、没有对不上的引文；主 agent 这一轮没有另跑 `snapshot-tree.py`（任务服务排着 91 条、小任务起不来），改成对腿点名的行逐处用 grep 与 sed 现读，现读的结果记在 `records/2026-11-16-三方豁免取消与存量补轮.md`「主 agent 现查坐实的」一段。
- 腿的说法：攻方打中 6 处，正推 35 份原件逐条过完、落点表 1065 行、报丢 7 处小的、二义 1 处、改做法没登 2 处。
- 被判的原件在腿跑着的时候没有被这一轮改过；腿中途收到的一条消息只是合稿树里两处路径改名，内容没变（两条腿都重核了 sha256）。
- 两条腿都没有写模型，没有要主 agent 复跑的数。

## 二、判决

「压缩之后规则一条没丢、没有二义、没有悄悄改做法」在第一版合稿上不成立。逐条：

1. 指到验证规则「一个门禁：两种跑法」的指针落空：两条腿都报，主 agent 现查那一节 lkmm、litmus、领卡、核账 0 命中，坐实，改指参考件里真有那句话的节。
2. 证红的变异清单写死成 crates/mutations.tsv：两条腿都报，主 agent 现查坐实，两处各写明。
3. 新加的「两处说同一件事以上游为准」：主 agent 现查原件 0 命中，坐实是多出来的规则，删。
4. 四处半句没有落点（等改动停下再跑、引的时候引那一行、即值 − 1、先问它是哪个参数的函数）：从原件放回。
5. 并行交错那一句被处置成「规则里没有对应的节」：攻方指出实现流程规则里有，主 agent 的处置错了，改成指对地方。
6. 检出工具可自行终止那一支被删而标成只改措辞：标签更正；放不放回交用户，用户定案第 21 条定只报不停。
7. 指到上游节名的指针：上游副本此刻 grep 那个节名 0 命中，改成指到文件并把判据句留在项目一侧。

其余打中的与腿的观察共同进了修稿的逐条去向表 `research/prompts/rules-compress-r2-merged/dispositions.md`（五题合计 129 条：成立 110、不成立 11、部分成立或只涉及脚本一侧 8）；主 agent 读了其中不采纳的 14 行，理由是那句话在脚本文件头而技能写了以文件头为准、或是不带判据的说明句，照收。

## 三、改法

改法都落在第二版合稿树 `research/prompts/rules-compress-r2-merged/`，相对第一版的差在同目录 `diff-from-r1.patch`。腿提的改法只当线索，丢的句子照原件原话放回。这些修补被攻过零轮，第二轮只攻它们。

## 交用户的

这一题里交用户、用户已定的条目在调度记录里编号 17 到 22；没有留着没定的。

## 这一轮被判过的文件

- .claude/rules/artifact-line-pointer.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/changelog-format.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/code-discipline-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/command-safety-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/communication-discipline-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/design-doc-discipline-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/directory-layout.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/engineering-philosophy.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/evidence-discipline.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/exhaustive-interleaving.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/format-evolution.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/fs-design.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/implementation-first.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/long-run-split.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/machine-first.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/mutation-sampling.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/path-moves.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/pushback-discipline-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/service-stop-readback.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/session-wrapup.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/show-me-test-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/test-discipline.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/test-script-discipline.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/timer-only-remind.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/verify-before-claiming.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/writing-discipline-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/crash-test/commit-stages.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/crash-test/SKILL.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/decide/SKILL.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/gate/SKILL.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/gate/staged-gate.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/halt/SKILL.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/queue-health/SKILL.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/remind/SKILL.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/skills/trilingual/SKILL.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。
