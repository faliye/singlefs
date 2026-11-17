# 判决：规则压缩复核第二轮（contradictions）

日期：2026-11-16。轮名 `rules-compress-contradictions-r2`。开工快照 `research/prompts/rules-compress-contradictions-r2-snapshot/`。正文 `research/prompts/_rules-compress-contradictions-r2-body.md`。前一轮判决 `research/prompts/rules-compress-contradictions-r1-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-contradictions-r2-opus-output.md`，辩方 `research/prompts/rules-compress-contradictions-r2-sonnet-output.md`。
- 主 agent 直接跑 `python3 scripts/agents/cite-check.py <报告>`（两轮二十份逐份跑）：没有对不上的引文；列成「没判」的是裸行号与认不出路径的短写，腿报告里各自写了手核。
- 被判的原件这一轮没被改过：开工时 `sha256sum -c research/prompts/rules-compress-contradictions-r2-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份，所以没有另起 `snapshot-tree.py`。
- 腿的说法：攻方判点名的 26 条里改得对 22、改错 1、改了判据 2；辩方判第一轮判决六条都站得住，从两条腿都判改得对的 81 条里抽 24 条重判、全部改得对。
- 辩方对第一轮核查方式的批评成立，这一轮照改：`cite-check.py` 主 agent 自己跑了；下面每条写明主 agent 现读了哪一处；去向表的条数由主 agent 自己数。

## 二、判决

第一轮判决各条经辩方复核站得住。第二版上「规则没丢、没有二义、修补都改对了」仍不成立，逐条：

1. 用户定-19 落得不全（调查员验证命令）：同定义那一题，进第三版。
2. 用户定-20 与常驻-10 的标签：更正为改了判据、照脚本改；常驻-10 多要求的 --wait 改回以钩子为准。
3. 总表第二版 281 处引文与行号里 31 处与树不符：第三版整张重出，表头写回查命令与结果 rows=118 users=26 refs_ok=455 problems=0（修稿一方的数，第三轮核）。
4. 用户定案第 21 到 26 条没进总表：第三版收到第 26 条。
5. 攻方交的两条核不动：弹窗次数由用户定案第 24 条定；循环逐个发的射程照第 20 条以钩子为准。

腿报告里其余的打中、遗留与观察逐条进了去向表 `research/prompts/rules-compress-r3-merged/dispositions-r2.md`。

## 三、改法

改法落在第三版合稿树 `research/prompts/rules-compress-r3-merged/`，相对第二版的差在同目录 `diff-from-r2.patch`。照用户定案第 23 条，治理文件的轮照代码轮收：这些修补再攻一轮（第三轮，最后一轮），没新打中就收；又打中的改了收、写进交用户表，不开第四轮。

## 交用户的

这一轮交用户、用户已定的在调度记录里编号 23 到 26。修稿一方另列了拿不准的五条（`research/prompts/rules-compress-r3-merged/revise-result.md` 第 ④ 节），其中循环逐个发的射程照第 20 条以钩子为准；其余四条留到第三轮判决一起交。

## 这一轮被判过的文件

- CLAUDE.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/main-agent.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agent-common.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
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
- .claude/rules/implementation-workflow.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
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
- .claude/rules/three-way-inference.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/timer-only-remind.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/verification.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/verify-before-claiming.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/writing-discipline-by-review.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/clerk.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/experiment-designer.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/experiment-runner.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/implementation-writer.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/investigator.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/kb-scribe.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/mutation-triage.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/prior-art.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/sweep.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/three-way-attack.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/three-way-defense.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/three-way-forward.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/three-way-materials.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/agents/tooling-writer.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
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
