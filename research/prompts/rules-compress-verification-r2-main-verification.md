# 判决：规则压缩复核第二轮（verification）

日期：2026-11-16。轮名 `rules-compress-verification-r2`。开工快照 `research/prompts/rules-compress-verification-r2-snapshot/`。正文 `research/prompts/_rules-compress-verification-r2-body.md`。前一轮判决 `research/prompts/rules-compress-verification-r1-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-verification-r2-opus-output.md`，辩方 `research/prompts/rules-compress-verification-r2-sonnet-output.md`。
- 主 agent 直接跑 `python3 scripts/agents/cite-check.py <报告>`（两轮二十份逐份跑）：没有对不上的引文；列成「没判」的是裸行号与认不出路径的短写，腿报告里各自写了手核。
- 被判的原件这一轮没被改过：开工时 `sha256sum -c research/prompts/rules-compress-verification-r2-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份，所以没有另起 `snapshot-tree.py`。
- 腿的说法：攻方打中 15 处；辩方判第一轮判决七条都站得住、三处放回的话不是原件原话而与文件头一致。
- 辩方对第一轮核查方式的批评成立，这一轮照改：`cite-check.py` 主 agent 自己跑了；下面每条写明主 agent 现读了哪一处；去向表的条数由主 agent 自己数。

## 二、判决

第一轮判决各条经辩方复核站得住。第二版上「规则没丢、没有二义、修补都改对了」仍不成立，逐条：

1. 轮的分类没定义、治理文件的轮照哪种收读得出两种：用户定案第 23 条。主 agent 现读第三版实现流程规则「代码轮与治理文件轮怎么收」一节，写了定义与两种补的情形。
2. 提交之外弹几次窗：用户定案第 24 条。主 agent 现读第三版验证规则第 38 行，带标记的由钩子那一次确认、不带的主 agent 问一次。
3. 「停」与「再抽一次」的界：用户定案第 26 条。主 agent 现读第三版三方规则第 72 行。
4. 放回常驻的判器调用点那句与文件头不合、声明模块两处射程不一：照文件头统一，登进矛盾总表。
5. 参考件转述门禁文件头而不合的六处（钉全、broad-mutation 那一趟、少一格、领卡被拒半句、跑到 0 条、qemu 与 lkmm）：照各门禁文件头改；多数是原件就写错。
6. 规则引脚本的行号会漂（这些脚本此刻是暂存区版本）：主 agent 现查第三版全树，文件加行号形的脚本指针 0 处。

腿报告里其余的打中、遗留与观察逐条进了去向表 `research/prompts/rules-compress-r3-merged/dispositions-r2.md`。

## 三、改法

改法落在第三版合稿树 `research/prompts/rules-compress-r3-merged/`，相对第二版的差在同目录 `diff-from-r2.patch`。照用户定案第 23 条，治理文件的轮照代码轮收：这些修补再攻一轮（第三轮，最后一轮），没新打中就收；又打中的改了收、写进交用户表，不开第四轮。

## 交用户的

这一轮交用户、用户已定的在调度记录里编号 23 到 26。修稿一方另列了拿不准的五条（`research/prompts/rules-compress-r3-merged/revise-result.md` 第 ④ 节），其中循环逐个发的射程照第 20 条以钩子为准；其余四条留到第三轮判决一起交。

## 这一轮被判过的文件

- .claude/rules/verification.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/implementation-workflow.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分
- .claude/rules/three-way-inference.md：对着第二版合稿树判了第一轮修补的对错、修补引出的新问题与第一轮只抽查过的部分

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。
