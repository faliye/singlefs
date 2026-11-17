# 判决：规则压缩复核第二轮（residents）

日期：2026-11-16。轮名 `rules-compress-residents-r2`。开工快照 `research/prompts/rules-compress-residents-r2-snapshot/`。正文 `research/prompts/_rules-compress-residents-r2-body.md`。前一轮判决 `research/prompts/rules-compress-residents-r1-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-residents-r2-opus-output.md`，辩方 `research/prompts/rules-compress-residents-r2-sonnet-output.md`。
- 主 agent 直接跑 `python3 scripts/agents/cite-check.py <报告>`（两轮二十份逐份跑）：没有对不上的引文；列成「没判」的是裸行号与认不出路径的短写，腿报告里各自写了手核。
- 被判的原件这一轮没被改过：开工时 `sha256sum -c research/prompts/rules-compress-residents-r2-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份，所以没有另起 `snapshot-tree.py`。
- 腿的说法：攻方打中 6 处、第一轮没细看的 18 份逐句对过没找到丢失；辩方判第一轮判决七条全部站得住。
- 辩方对第一轮核查方式的批评成立，这一轮照改：`cite-check.py` 主 agent 自己跑了；下面每条写明主 agent 现读了哪一处；去向表的条数由主 agent 自己数。

## 二、判决

第一轮判决各条经辩方复核站得住。第二版上「规则没丢、没有二义、修补都改对了」仍不成立，逐条：

1. 指上游的 43 处指针被换成括号写法、出了门禁的射程：两条腿都报。主 agent 现查第三版规则目录，括号写法 0 处、反引号文件名加「节」的写法 36 处。
2. 变异清单只补了装置那张表、原件的条件句丢了、同一条判据住了两处：进第三版。
3. 用户定案第 18、24 条没落到三份技能：进第三版。
4. 矛盾总表又有两行标成只改措辞而动了做法：标签更正。
5. 主 agent 第一轮判决里「不采纳 11 条」的数对不上去向表：主 agent 这一次自己数第二轮去向表，结论列写不成立的 7 行、部分的 8 行。

腿报告里其余的打中、遗留与观察逐条进了去向表 `research/prompts/rules-compress-r3-merged/dispositions-r2.md`。

## 三、改法

改法落在第三版合稿树 `research/prompts/rules-compress-r3-merged/`，相对第二版的差在同目录 `diff-from-r2.patch`。照用户定案第 23 条，治理文件的轮照代码轮收：这些修补再攻一轮（第三轮，最后一轮），没新打中就收；又打中的改了收、写进交用户表，不开第四轮。

## 交用户的

这一轮交用户、用户已定的在调度记录里编号 23 到 26。修稿一方另列了拿不准的五条（`research/prompts/rules-compress-r3-merged/revise-result.md` 第 ④ 节），其中循环逐个发的射程照第 20 条以钩子为准；其余四条留到第三轮判决一起交。

## 这一轮被判过的文件

- .claude/rules/artifact-line-pointer.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/changelog-format.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/code-discipline-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/command-safety-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/communication-discipline-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/design-doc-discipline-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/directory-layout.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/engineering-philosophy.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/evidence-discipline.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/exhaustive-interleaving.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/format-evolution.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/fs-design.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/implementation-first.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/long-run-split.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/machine-first.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/mutation-sampling.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/path-moves.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/pushback-discipline-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/service-stop-readback.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/session-wrapup.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/show-me-test-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/test-discipline.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/test-script-discipline.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/timer-only-remind.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/verify-before-claiming.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/writing-discipline-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/crash-test/commit-stages.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/crash-test/SKILL.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/decide/SKILL.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/gate/SKILL.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/gate/staged-gate.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/halt/SKILL.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/queue-health/SKILL.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/remind/SKILL.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/skills/trilingual/SKILL.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。
