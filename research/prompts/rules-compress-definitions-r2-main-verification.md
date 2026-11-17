# 判决：规则压缩复核第二轮（definitions）

日期：2026-11-16。轮名 `rules-compress-definitions-r2`。开工快照 `research/prompts/rules-compress-definitions-r2-snapshot/`。正文 `research/prompts/_rules-compress-definitions-r2-body.md`。前一轮判决 `research/prompts/rules-compress-definitions-r1-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-definitions-r2-opus-output.md`，辩方 `research/prompts/rules-compress-definitions-r2-sonnet-output.md`。
- 主 agent 直接跑 `python3 scripts/agents/cite-check.py <报告>`（两轮二十份逐份跑）：没有对不上的引文；列成「没判」的是裸行号与认不出路径的短写，腿报告里各自写了手核。
- 被判的原件这一轮没被改过：开工时 `sha256sum -c research/prompts/rules-compress-definitions-r2-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份，所以没有另起 `snapshot-tree.py`。
- 腿的说法：攻方打中 9 条；辩方判第一轮判决七条都站得住、修补都够得着。
- 辩方对第一轮核查方式的批评成立，这一轮照改：`cite-check.py` 主 agent 自己跑了；下面每条写明主 agent 现读了哪一处；去向表的条数由主 agent 自己数。

## 二、判决

第一轮判决各条经辩方复核站得住。第二版上「规则没丢、没有二义、修补都改对了」仍不成立，逐条：

1. 调查员「查到就修」的验证命令落在修之前的副本上：这一题与矛盾那一题的攻方各自独立报。主 agent 现读第三版调查员定义第 28、33 行，验证改动的命令填它改的那棵树。
2. 「规则怎么读」被写窄：两条腿都报。主 agent 的第一轮处置撤回；现读第三版共用约束第 17 行是原件原话「一律整份读，不受读入大小的限制」。
3. 进程停法在工具员第 5 步没写清、两句互相排掉：照钩子文件头写清。
4. 共用约束对弹窗闸、定时闸、项目代码闸、写闸的转述与钩子有出入的四条，「书记员」只剩一处：进第三版。
5. 修稿补丁不全（少「或卡住」一句）：那一句是主 agent 在修稿交回后补进树的；第三版的补丁由修稿一方重跑 diff 并回打核过。

腿报告里其余的打中、遗留与观察逐条进了去向表 `research/prompts/rules-compress-r3-merged/dispositions-r2.md`。

## 三、改法

改法落在第三版合稿树 `research/prompts/rules-compress-r3-merged/`，相对第二版的差在同目录 `diff-from-r2.patch`。照用户定案第 23 条，治理文件的轮照代码轮收：这些修补再攻一轮（第三轮，最后一轮），没新打中就收；又打中的改了收、写进交用户表，不开第四轮。

## 交用户的

这一轮交用户、用户已定的在调度记录里编号 23 到 26。修稿一方另列了拿不准的五条（`research/prompts/rules-compress-r3-merged/revise-result.md` 第 ④ 节），其中循环逐个发的射程照第 20 条以钩子为准；其余四条留到第三轮判决一起交。

## 这一轮被判过的文件

- .claude/agent-common.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
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

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。
