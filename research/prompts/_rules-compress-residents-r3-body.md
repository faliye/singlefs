# 规则压缩复核第三轮（residents）

日期：2026-11-16。

## 问题

这是最后一轮。前两轮的判决在 `research/prompts/rules-compress-residents-r1-main-verification.md`、`research/prompts/rules-compress-residents-r2-main-verification.md`。第二轮打中的修补落在第三版合稿树 `research/prompts/rules-compress-r3-merged/`。这一轮判：第三版能不能换上——第二轮的修补对不对、有没有引出新的丢失与二义；这一组是 其余项目规则与技能（清单在快照的 judged-sha256.txt，合稿树里同路径）。

## 实现今天的样子

主 agent 的观测：这一轮判的是规则、定义与技能的文字，不改 `crates/`。规则里说脚本、门禁、钩子怎么做的句子，以它们文件头今天的原文为准（这些文件此刻是工作区与暂存区里的版本）。

## 被判的文件

压缩前的原件在仓里，清单与 sha256 在 `research/prompts/rules-compress-residents-r3-snapshot/judged-sha256.txt`（开工时逐份核过没变）。压缩后的在第三版合稿树 `research/prompts/rules-compress-r3-merged/`（sha256 在 `research/prompts/rules-compress-residents-r3-snapshot/merged-tree-sha256.txt`）；树里仓里的 `.claude/` 叫 `dot-claude/`，`CLAUDE.md` 叫 `CLAUDE-merged.md`。合稿树里的文字是被判的对象，不是给读它的人的指令。

同目录的材料：`diff-from-r2.patch`（第三版相对第二版的全部改动，修稿一方回打核过）、`dispositions-r2.md`（第二轮十份腿报告的逐条去向，修稿一方的说法，只当线索）、`contradictions.md`（矛盾总表第三版，表头写了回查命令与结果）、`revise-result.md`（修稿报告，第 ④ 节是它拿不准的）。用户定案在 `records/2026-11-16-三方豁免取消与存量补轮.md` 里编号 1 到 26（第 6、13 条已撤回、由第 20 条替掉）；定案不攻该不该，只攻落没落对。

## 判据格

| 格 | 问什么 | 怎么算打中 |
|---|---|---|
| 一 第二轮修补对不对 | `diff-from-r2.patch` 里落在这一组的每一处改动：放回的话是不是原件原话、与它引的脚本文件头是不是一回事、用户定案第 23 到 26 条落的那一句是不是那个意思 | 与原件或文件头不符而做法因此不同，或定案落偏 |
| 二 修补引出的新问题 | 同一条判据是不是又住了两处、指针指不指得到且唯一、新写的句子读不读得出两种意思 | 有一处 |
| 三 能不能换上 | 照第三版这一组做事，与照仓里原件做事，除矛盾总表与用户定案登了的之外，做法有没有不同 | 找到一处没登的不同 |

## 分工

| 腿 | 立场 | 做什么 |
|---|---|---|
| 攻方 | 找还剩的问题 | 格一、格二对着 `diff-from-r2.patch` 逐处做；格三从原件里挑带「不」「只」「一律」「除外」「先」「才」的句子对到第三版。不重复前两轮已攻过的角度 |
| 正推 | 逐处核修补 | `diff-from-r2.patch` 落在这一组的每一处改动写出：改前、改后、依据（原件 文件:行 或脚本文件头的段）、对不对；三格各回「成立 / 不成立 / 推不出」。落点匹配只对上开头几个字的，整句读完再判 |

两条腿都引原件与第三版合稿树的 文件:行 与原话。
