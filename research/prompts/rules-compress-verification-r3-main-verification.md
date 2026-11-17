# 判决：规则压缩复核第三轮（verification）

日期：2026-11-17。轮名 `rules-compress-verification-r3`，最后一轮。开工快照 `research/prompts/rules-compress-verification-r3-snapshot/`。正文 `research/prompts/_rules-compress-verification-r3-body.md`。前两轮判决 `research/prompts/rules-compress-verification-r1-main-verification.md`、`research/prompts/rules-compress-verification-r2-main-verification.md`。

## 一、腿与核查

- 攻方 `research/prompts/rules-compress-verification-r3-opus-output.md`，正推 `research/prompts/rules-compress-verification-r3-sonnet-output.md`。
- 被判的原件这一轮没被改过：写判决前 `sha256sum -c research/prompts/rules-compress-verification-r3-snapshot/judged-sha256.txt` 全部对上，仓里现读与倒推出的树是同一份。
- 腿报告的引文：两条腿交回前各自跑过 `python3 scripts/agents/cite-check.py`，末行都没有对不上的；列成「没判」的裸行号由腿用 grep 手核。
- 腿的说法：两条腿都判换上之前必须改 2 到 3 处，都在「代码轮与治理文件轮怎么收」那几句；正推把补丁落在这一组的 31 处改动逐处核过，29 处成立。
- 下面每条写明主 agent 现读了第四版的哪一处。

## 二、判决

第三版上还有要改的，改在第四版 `research/prompts/rules-compress-r4-merged/`。逐条：

1. 多出来的「那一轮只攻上一轮的修补」：用户定案第 27 条。主 agent 现读第四版全树 0 命中，实现流程规则第 31 行写的是修补加只抽查过的地方。
2. 「在第二轮派辩方腿」的依据记在了用户定案名下而原话没有：用户定案第 28 条补定，依据栏改记。
3. 「停」的判据住三处而写法不同：用户定案第 26、29 条。主 agent 现读第四版，判据只在三方规则「一条腿只抽一次样不算一次观测」第一条写全，另两处指过去。
4. 「现查推翻的不改、不为它开轮」「第三轮又打中的改了就收、不开第四轮」两句是主 agent 自己加的：用户定案第 31 条定删；第三轮的修补标零轮、要写进产品代码的先做 A/B 留着（用户定案第 32 条）。
5. 「一题只属一类、拆成两题」是攻方腿推的：用户定案第 33 条定留。

腿报告里其余的打中与观察逐条进了去向表 `research/prompts/rules-compress-r4-merged/dispositions-r3.md`（修稿一方数的：97 条并成 59 行，成立 44、不成立 12、部分 3）。

## 三、改法

改法落在第四版合稿树，相对第三版的差在同目录 `diff-from-r3.patch`。这是第三轮，这些修补没有再被攻过：照用户定案第 32 条，它们是规则、定义与技能的文字，不进产品代码，不做 A/B。第四版换进仓里，安装表在同目录 `install-map.tsv`。

## 交用户的

用户已定的在调度记录 `records/2026-11-17-三方豁免取消与存量补轮.md` 里编号 27 到 33。还留着的三件，换上不等它们：证红命令模板在副本里起 `prove-red.sh` 会不会被它当主工作区拒（要探针）；门禁 `crash-point-replay.sh` 文件头格名表与 feature-gated-tests 那一段写法不一（交管那道门禁的会话）；用户定案第 9 条 RefFS 依赖格收严的欠账号还没登。

## 这一轮被判过的文件

- .claude/rules/verification.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/implementation-workflow.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/three-way-inference.md：对着第三版与第四版合稿树判了第二轮修补的对错、修补引出的新问题与能不能换上
- .claude/rules/verification-reference.md：这一轮判了它的内容（压缩时新建，细则从这一组的原件挪来），换上的是第四版
- .claude/skills/three-way/SKILL.md：这一轮判了它的内容（压缩时新建，细则从这一组的原件挪来），换上的是第四版

## 回看决策

不涉及决策：这一轮判的是规则、定义与技能的文字，`.claude/kb/decisions/` 下没有哪一条的定案或依据因此变动。

## 判决之后

换进仓里的是第四版，另加判决之后的几处改动，都是换上那一步门禁点出来或用户补定的：用户定案第 31 到 33 条（删主 agent 自己加的三句、第三轮的修补进产品代码先做 A/B、一题只属一类）；门禁 doc-registries 的 governance-refs 格点出 kb 逐字引用的五句规则被改了几个字，恢复成原件原话（`.claude/rules/evidence-discipline.md` 两句、`.claude/rules/test-discipline.md` 三句）；`.claude/rules/verification-reference.md` 一处测试文件路径补成全路径；用户定案第 34 条给三份定义与共用约束补了规则点名。被并掉的八份规则已删。

`research/prompts/rules-compress-verification-r3-snapshot/installed-sha256.txt` 是换上并做完这些改动之后补记的快照（事后补的，不是派腿之前记的）：记的是这一组此刻在仓里的每一份的 sha256。这几处判决之后的改动没有再被攻过。
