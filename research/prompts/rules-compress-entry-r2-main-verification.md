# 判决：规则压缩复核第二轮（entry）

日期：2026-11-16。轮名 `rules-compress-entry-r2`。开工快照 `research/prompts/rules-compress-entry-r2-snapshot/`。正文 `research/prompts/_rules-compress-entry-r2-body.md`。前一轮判决 `research/prompts/rules-compress-entry-r1-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-entry-r2-opus-output.md`，辩方 `research/prompts/rules-compress-entry-r2-sonnet-output.md`。
- 主 agent 直接跑 `python3 scripts/agents/cite-check.py <报告>`（两轮二十份逐份跑）：没有对不上的引文；列成「没判」的是裸行号与认不出路径的短写，腿报告里各自写了手核。
- 被判的原件这一轮没被改过：开工时 `sha256sum -c research/prompts/rules-compress-entry-r2-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份，所以没有另起 `snapshot-tree.py`。
- 腿的说法：攻方打中 8 处；辩方判第一轮判决七条里六条站得住、「只有两类」一条部分站得住。
- 辩方对第一轮核查方式的批评成立，这一轮照改：`cite-check.py` 主 agent 自己跑了；下面每条写明主 agent 现读了哪一处；去向表的条数由主 agent 自己数。

## 二、判决

第一轮判决各条经辩方复核站得住。第二版上「规则没丢、没有二义、修补都改对了」仍不成立，逐条：

1. 「只有两类」没真去掉、修稿另加了半句新判据：两条腿都报。主 agent 现读第三版入口第 9 行，「只有两类」0 命中，原件那一句的原意在，用户定案第 25 条。
2. 技能重读只管了验证段与派发两件事：主 agent 现读第三版入口第 5 行，已补「被叫醒后处置之前、读交回之前」。
3. 「整批重交」新加的那句可读成一条指令、调查员例外没进写段第 6 条、「也不往哪一棵树里手写」宾语空着、五条判据各住两处：进第三版，去向在去向表。
4. 原件「规矩变过了，先读再跑」丢了、「本项目只接这一份」先行词不明：放回、写明。

腿报告里其余的打中、遗留与观察逐条进了去向表 `research/prompts/rules-compress-r3-merged/dispositions-r2.md`。

## 三、改法

改法落在第三版合稿树 `research/prompts/rules-compress-r3-merged/`，相对第二版的差在同目录 `diff-from-r2.patch`。照用户定案第 23 条，治理文件的轮照代码轮收：这些修补再攻一轮（第三轮，最后一轮），没新打中就收；又打中的改了收、写进交用户表，不开第四轮。

## 交用户的

这一轮交用户、用户已定的在调度记录里编号 23 到 26。修稿一方另列了拿不准的五条（`research/prompts/rules-compress-r3-merged/revise-result.md` 第 ④ 节），其中循环逐个发的射程照第 20 条以钩子为准；其余四条留到第三轮判决一起交。

## 这一轮被判过的文件

- CLAUDE.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/main-agent.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。
