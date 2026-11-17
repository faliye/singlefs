# 判决：规则压缩复核（验证、实现流程与三方）第一轮

日期：2026-11-16。轮名 `rules-compress-verification-r1`。开工快照 `research/prompts/rules-compress-verification-r1-snapshot/`（被判的原件 `judged-sha256.txt`，合稿树第一版 `merged-tree-sha256.txt`）。正文 `research/prompts/_rules-compress-verification-r1-body.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-verification-r1-opus-output.md`，正推 `research/prompts/rules-compress-verification-r1-sonnet-output.md`。两条腿交回时各自跑过 `scripts/agents/cite-check.py`，末行都是「对上」、没有对不上的引文；主 agent 这一轮没有另跑 `snapshot-tree.py`（任务服务排着 91 条、小任务起不来），改成对腿点名的行逐处用 grep 与 sed 现读，现读的结果记在 `records/2026-11-16-三方豁免取消与存量补轮.md`「主 agent 现查坐实的」一段。
- 腿的说法：攻方四格有打中（丢 5、二义 5、常驻不够 3、改做法没登 4），正推 173 条落点里 172 条完整、四格不成立。
- 被判的原件在腿跑着的时候没有被这一轮改过；腿中途收到的一条消息只是合稿树里两处路径改名，内容没变（两条腿都重核了 sha256）。
- 两条腿都没有写模型，没有要主 agent 复跑的数。

## 二、判决

「压缩之后规则一条没丢、没有二义、没有悄悄改做法」在第一版合稿上不成立。逐条：

1. 丢了「旧库由主 agent 删掉」：两条腿都报，主 agent 现查合稿两份 0 命中，坐实，放回。
2. 参考件两个标题同名「门禁管哪一半」：两条腿都报，主 agent 现查第 140、213 行坐实，第二个改名。
3. 原「门禁管哪一半」表里只在表里写过的四条判据随表挪走：放回常驻的验证规则。
4. 三方开轮四句的先后：用户定案第 2、17 条，代码轮与设计推论轮各写各的。
5. 「缩过域的那一趟不判」写宽：主 agent 对 scripts/gate/admission.py 文件头第 164 行现查坐实，照文件头改。
6. 照门禁文件头改了做法而没登进矛盾总表的四处：补登。
7. 「测试线程」入口只写给用 GPU 的活：常驻补一句。

其余打中的与腿的观察共同进了修稿的逐条去向表 `research/prompts/rules-compress-r2-merged/dispositions.md`（五题合计 129 条：成立 110、不成立 11、部分成立或只涉及脚本一侧 8）；主 agent 读了其中不采纳的 14 行，理由是那句话在脚本文件头而技能写了以文件头为准、或是不带判据的说明句，照收。

## 三、改法

改法都落在第二版合稿树 `research/prompts/rules-compress-r2-merged/`，相对第一版的差在同目录 `diff-from-r1.patch`。腿提的改法只当线索，丢的句子照原件原话放回。这些修补被攻过零轮，第二轮只攻它们。

## 交用户的

这一题里交用户、用户已定的条目在调度记录里编号 17 到 22；没有留着没定的。

## 这一轮被判过的文件

- .claude/rules/verification.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/implementation-workflow.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化
- .claude/rules/three-way-inference.md：对着合稿树判了它压缩前后的规则落点、二义与做法变化

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。
