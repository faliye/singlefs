# 判决：规则压缩复核（矛盾与不明晰处的修正）第一轮

日期：2026-11-16。轮名 `rules-compress-contradictions-r1`。开工快照 `research/prompts/rules-compress-contradictions-r1-snapshot/`（被判的原件 `judged-sha256.txt`，合稿树第一版 `merged-tree-sha256.txt`）。正文 `research/prompts/_rules-compress-contradictions-r1-body.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-contradictions-r1-opus-output.md`，正推 `research/prompts/rules-compress-contradictions-r1-sonnet-output.md`。两条腿交回时各自跑过 `scripts/agents/cite-check.py`，末行都是「对上」、没有对不上的引文；主 agent 这一轮没有另跑 `snapshot-tree.py`（任务服务排着 91 条、小任务起不来），改成对腿点名的行逐处用 grep 与 sed 现读，现读的结果记在 `records/2026-11-16-三方豁免取消与存量补轮.md`「主 agent 现查坐实的」一段。
- 腿的说法：攻方判 85 条里改得对 82、其实改了判据 3、改错 0；正推判改得对 84、改错 1；两条腿都没有核不动的。
- 被判的原件在腿跑着的时候没有被这一轮改过；腿中途收到的一条消息只是合稿树里两处路径改名，内容没变（两条腿都重核了 sha256）。
- 两条腿都没有写模型，没有要主 agent 复跑的数。

## 二、判决

「压缩之后规则一条没丢、没有二义、没有悄悄改做法」在第一版合稿上不成立。逐条：

1. 验证-9 改错：主 agent 对 scripts/gate/admission.py 文件头现查坐实，照文件头改。
2. 入口-4「只有两类」：去掉「只有」。
3. 常驻-8「分不清是谁的红」那条路被删：用户定案第 18 条，路留着、先弹窗。
4. 合稿-1 代码轮照三轮多数：用户定案第 17 条，改回。
5. 常驻-7、常驻-11、共用-10 标成只改措辞而动了做法：标签更正；常驻-11 交用户，用户定案第 21 条。
6. 总表写偏的行号与依据行：逐条改；照脚本改了做法而没登的补登合稿-4 到合稿-12。
7. 其余 81 条两条腿都判改得对，照收；这是两次独立抽样的同一结论。

其余打中的与腿的观察共同进了修稿的逐条去向表 `research/prompts/rules-compress-r2-merged/dispositions.md`（五题合计 129 条：成立 110、不成立 11、部分成立或只涉及脚本一侧 8）；主 agent 读了其中不采纳的 14 行，理由是那句话在脚本文件头而技能写了以文件头为准、或是不带判据的说明句，照收。

## 三、改法

改法都落在第二版合稿树 `research/prompts/rules-compress-r2-merged/`，相对第一版的差在同目录 `diff-from-r1.patch`。腿提的改法只当线索，丢的句子照原件原话放回。这些修补被攻过零轮，第二轮只攻它们。

## 交用户的

这一题里交用户、用户已定的条目在调度记录里编号 17 到 22；没有留着没定的。

## 这一轮被判过的文件

- CLAUDE.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/main-agent.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/agent-common.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
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
- .claude/rules/implementation-workflow.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
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
- .claude/rules/three-way-inference.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/timer-only-remind.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/verification.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/verify-before-claiming.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/writing-discipline-by-review.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
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
