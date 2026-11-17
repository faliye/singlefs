# 判决：规则压缩复核第三轮（residents）

日期：2026-11-17。轮名 `rules-compress-residents-r3`，最后一轮。开工快照 `research/prompts/rules-compress-residents-r3-snapshot/`。正文 `research/prompts/_rules-compress-residents-r3-body.md`。前两轮判决 `research/prompts/rules-compress-residents-r1-main-verification.md`、`research/prompts/rules-compress-residents-r2-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-residents-r3-opus-output.md`，正推 `research/prompts/rules-compress-residents-r3-sonnet-output.md`。
- 被判的原件这一轮没被改过：写判决前 `sha256sum -c research/prompts/rules-compress-residents-r3-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份。
- 腿报告的引文：两条腿交回前各自跑过 `python3 scripts/agents/cite-check.py`，末行都没有对不上的；列成「没判」的裸行号由腿用 grep 手核。
- 腿的说法：两条腿都判这一组可以换上、换上之前必须改 1 处。
- 下面每条写明主 agent 现读了第四版的哪一处。

## 二、判决

第三版上还有要改的，改在第四版 `research/prompts/rules-compress-r4-merged/`。逐条：

1. 证红那一句的「入库装置」没有定义：两条腿各自独立读出。主 agent 现读第四版，「入库装置」0 命中，那一句写明 crates/ 下的装置进 crates/mutations.tsv、scripts/experiment-apparatus/ 下的进 scripts/mutation/tables/ 下它那张表。
2. Python 里每一轮按组停自己起的那一组放不放行：用户定案第 30 条定放行。
3. 指到任务服务库「错误类型表的 stalled 一行」的三处在工作区版本里指不到（原件同样）：第四版照今天的文件头改。
4. 被并掉的八份文件不带 .md 的旧名引用五处：补进引用改法清单。
5. 上游副本在第三轮之后同步到了 0.0.74、节标题变了：主 agent 对着新副本把第四版 67 处指上游的指针逐处核，66 处是那份文件今天的标题且只有一个，余 1 处是总表里举例的字面。

腿报告里其余的打中与观察逐条进了去向表 `research/prompts/rules-compress-r4-merged/dispositions-r3.md`（修稿一方数的：97 条并成 59 行，成立 44、不成立 12、部分 3）。

## 三、改法

改法落在第四版合稿树，相对第三版的差在同目录 `diff-from-r3.patch`。这是第三轮，这些修补没有再被攻过：照用户定案第 32 条，它们是规则、定义与技能的文字，不进产品代码，不做 A/B。第四版换进仓里，安装表在同目录 `install-map.tsv`。

## 交用户的

用户已定的在调度记录 `records/2026-11-17-三方豁免取消与存量补轮.md` 里编号 27 到 33。还留着的三件，换上不等它们：证红命令模板在副本里起 `prove-red.sh` 会不会被它当主工作区拒（要探针）；门禁 `crash-point-replay.sh` 文件头格名表与 feature-gated-tests 那一段写法不一（交管那道门禁的会话）；用户定案第 9 条 RefFS 依赖格收严的欠账号还没登。

## 这一轮被判过的文件

- .claude/rules/artifact-line-pointer.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/changelog-format.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/code-discipline-by-review.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/command-safety-by-review.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/communication-discipline-by-review.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/design-doc-discipline-by-review.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/directory-layout.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/engineering-philosophy.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/evidence-discipline.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/exhaustive-interleaving.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/format-evolution.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/fs-design.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/implementation-first.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/long-run-split.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/machine-first.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/mutation-sampling.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/path-moves.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/pushback-discipline-by-review.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/service-stop-readback.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/session-wrapup.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/show-me-test-by-review.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/test-discipline.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/test-script-discipline.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/timer-only-remind.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/verify-before-claiming.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/writing-discipline-by-review.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/crash-test/commit-stages.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/crash-test/SKILL.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/decide/SKILL.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/gate/SKILL.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/gate/staged-gate.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/halt/SKILL.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/queue-health/SKILL.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/remind/SKILL.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/skills/trilingual/SKILL.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。

## 判决之后

换进仓里的是第四版，另加判决之后的几处改动，都是换上那一步门禁点出来或用户补定的：用户定案第 31 到 33 条（删主 agent 自己加的三句、第三轮的修补进产品代码先做 A/B、一题只属一类）；门禁 doc-registries 的 governance-refs 格点出 kb 逐字引用的五句规则被改了几个字，恢复成原件原话（`.claude/rules/evidence-discipline.md` 两句、`.claude/rules/test-discipline.md` 三句）；`.claude/rules/verification-reference.md` 一处测试文件路径补成全路径；用户定案第 34 条给三份定义与共用约束补了规则点名。被并掉的八份规则已删。

`research/prompts/rules-compress-residents-r3-snapshot/installed-sha256.txt` 是换上并做完这些改动之后补记的快照（事后补的，不是派腿之前记的）：记的是这一组此刻在仓里的每一份的 sha256。这几处判决之后的改动没有再被攻过。
