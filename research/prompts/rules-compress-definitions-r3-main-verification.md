# 判决：规则压缩复核第三轮（definitions）

日期：2026-11-17。轮名 `rules-compress-definitions-r3`，最后一轮。开工快照 `research/prompts/rules-compress-definitions-r3-snapshot/`。正文 `research/prompts/_rules-compress-definitions-r3-body.md`。前两轮判决 `research/prompts/rules-compress-definitions-r1-main-verification.md`、`research/prompts/rules-compress-definitions-r2-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-definitions-r3-opus-output.md`，正推 `research/prompts/rules-compress-definitions-r3-sonnet-output.md`。
- 被判的原件这一轮没被改过：写判决前 `sha256sum -c research/prompts/rules-compress-definitions-r3-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份。
- 腿报告的引文：两条腿交回前各自跑过 `python3 scripts/agents/cite-check.py`，末行都没有对不上的；列成「没判」的裸行号由腿用 grep 手核。
- 腿的说法：两条腿都判改三处后可以换上。
- 下面每条写明主 agent 现读了第四版的哪一处。

## 二、判决

第三版上还有要改的，改在第四版 `research/prompts/rules-compress-r4-merged/`。逐条：

1. 共用约束新加的批次代号例子裸写在「」里、写闸与检出钩子会拒：正推腿写报告时被检出钩子当场拒过一次。主 agent 现读第四版共用约束，「」内裸写的批次代号 0 处。
2. 「简单任务不派发，自己直接改」没照用户定案第 25 条原话留：三条腿都报。主 agent 现读第四版第 11 行，与原件第 8 行逐字相同。
3. 调查员「查到就修」那一支新变异行的验证段占位符填不出：第四版照正推腿的命令落点表写进调查员定义。
4. 实现员建副本没带证红脚本与门禁脚本、几道闸的转述与钩子文件头有出入：第四版对着文件头改。

腿报告里其余的打中与观察逐条进了去向表 `research/prompts/rules-compress-r4-merged/dispositions-r3.md`（修稿一方数的：97 条并成 59 行，成立 44、不成立 12、部分 3）。

## 三、改法

改法落在第四版合稿树，相对第三版的差在同目录 `diff-from-r3.patch`。这是第三轮，这些修补没有再被攻过：照用户定案第 32 条，它们是规则、定义与技能的文字，不进产品代码，不做 A/B。第四版换进仓里，安装表在同目录 `install-map.tsv`。

## 交用户的

用户已定的在调度记录 `records/2026-11-17-三方豁免取消与存量补轮.md` 里编号 27 到 33。还留着的三件，换上不等它们：证红命令模板在副本里起 `prove-red.sh` 会不会被它当主工作区拒（要探针）；门禁 `crash-point-replay.sh` 文件头格名表与 feature-gated-tests 那一段写法不一（交管那道门禁的会话）；用户定案第 9 条 RefFS 依赖格收严的欠账号还没登。

## 这一轮被判过的文件

- .claude/agent-common.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/clerk.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/experiment-designer.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/experiment-runner.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/implementation-writer.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/investigator.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/kb-scribe.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/mutation-triage.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/prior-art.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/sweep.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/three-way-attack.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/three-way-defense.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/three-way-forward.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/three-way-materials.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/agents/tooling-writer.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。

## 判决之后

换进仓里的是第四版，另加判决之后的几处改动，都是换上那一步门禁点出来或用户补定的：用户定案第 31 到 33 条（删主 agent 自己加的三句、第三轮的修补进产品代码先做 A/B、一题只属一类）；门禁 doc-registries 的 governance-refs 格点出 kb 逐字引用的五句规则被改了几个字，恢复成原件原话（`.claude/rules/evidence-discipline.md` 两句、`.claude/rules/test-discipline.md` 三句）；`.claude/rules/verification-reference.md` 一处测试文件路径补成全路径；用户定案第 34 条给三份定义与共用约束补了规则点名。被并掉的八份规则已删。

`research/prompts/rules-compress-definitions-r3-snapshot/installed-sha256.txt` 是换上并做完这些改动之后补记的快照（事后补的，不是派腿之前记的）：记的是这一组此刻在仓里的每一份的 sha256。这几处判决之后的改动没有再被攻过。
