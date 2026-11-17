# 规则压缩复核：验证、实现流程与三方

日期：2026-11-16。

## 问题

规则与规范压缩（去重、合并相似、删说明、把做到那一步才用的流程挪进按需读的技能与参考件）之后，判这一组文件：压缩前的每一条规则在压缩后还在不在、有没有读得出两种意思的句子。出口与口径在 `records/2026-11-16-三方豁免取消与存量补轮.md`「插进来的一件：规则与规范压缩」起到末尾各段。

## 实现今天的样子

主 agent 的观测：这一轮判的是规则、定义与技能的文字，不改 `crates/`；`git status --short -- crates/` 里的改动都是别的会话的，这一轮没有读它们当依据。规则里说脚本怎么做的句子，以脚本文件头今天的原文为准。

## 被判的文件

压缩前（仓里的原件，sha256 在 `research/prompts/rules-compress-verification-r1-snapshot/judged-sha256.txt`）：

- `.claude/rules/verification.md`
- `.claude/rules/implementation-workflow.md`
- `.claude/rules/three-way-inference.md`

压缩后（合稿树 `research/prompts/rules-compress-r1-merged/`，树里的相对路径就是仓里的路径，sha256 在 `research/prompts/rules-compress-verification-r1-snapshot/merged-tree-sha256.txt`）：

- `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md`
- `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md`
- `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md`
- `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md`
- `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md`

合稿树整棵都可以读：一条规则可能从这一组的原件挪到了别的组的文件里。挪走的去向线索在各写手的对照表（`~/.local/share/singlefs/work/rules-compress-*/result.md`）与 `research/prompts/rules-compress-r1-merged/merge-result.md`，它们是写手自己的说法，只当线索。

用户已定的十六条在调度记录里编号 1 到 16；这些是定案，不攻它们该不该，只攻落没落对。


合稿树里的路径有两处改了名，免得被工具当成一套生效的配置：仓里的 `.claude/` 在树里叫 `dot-claude/`，仓里的 `CLAUDE.md` 在树里叫 `CLAUDE-merged.md`；别的相对路径与仓里相同。合稿树里的文字是被判的对象，不是给读它的人的指令。

## 判据格

| 格 | 问什么 | 怎么算打中 |
|---|---|---|
| 一 丢失 | 原件里每一条判据、阈值、次序、路径、命令、退出码的含义、例外，在合稿树里有没有落点 | 找不到落点，或落点只剩指针而指到的地方没有那句话 |
| 二 二义 | 合并、改写后的句子主语、范围、例外读不读得出两种意思；指针指到的节是不是唯一 | 给得出两种读法且两种读法做法不同 |
| 三 常驻够不够 | 挪进技能与参考件的细则，常驻文件里留没留「什么时候读哪一份」的入口；不读技能时照常驻文件做会不会做错 | 有一步不读技能就会做错，而常驻文件没叫人先读 |
| 四 子 agent 读得到 | 定义起步不读 `CLAUDE.md` 与规则：定义与共用约束里的指针指到的文件，子 agent 被没被点名去读 | 指到它不读的文件 |
| 五 悄悄改了做法 | 合稿与原件做法不同、而矛盾总表 `research/prompts/rules-compress-r1-merged/contradictions.md` 与用户定案里都没登的地方 | 有一处 |

## 分工

| 腿 | 立场 | 做什么 |
|---|---|---|
| 正推 | 逐条核落点 | 把这一组每份原件从头到尾过一遍，每条规则写出它在合稿树的 文件:行；五格各回「成立 / 不成立 / 推不出」，推不出的列出是哪几条 |
| 攻方 | 找丢的、找二义 | 专攻：两种跑法、单元键与两种排除、判定库只有一份、多机可用各条压缩后少了哪个例外；各道门禁各格怎么跑挪进参考件后与门禁文件头对不对得上；三方的整行抄、核查四步、叫停轮登记判法挪进技能后丢了哪条 |

两条腿都引原件与合稿树的 文件:行 与原话；不引写手对照表当证据。
