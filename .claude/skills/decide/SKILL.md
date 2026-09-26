---
name: decide
description: 记录或变更 singlefs 的设计决策。定下一条决策、推翻一条旧决策、或发现某个选择会连锁影响其他决策时用它——包含决策记录的格式、状态机、以及和不变量清单/checker 的同步要求。
---

正文在共享层，读它：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`

**不要把正文抄到这里。** 抄了别的项目看不到，下次又会被抄一遍。

## 在本项目里

共享正文的「写进 `kb/decisions.md` 的格式」一节在本项目不适用：本项目一条决策一个文件（`.claude/kb/decisions/NN-简称.md`），索引在 `.claude/kb/decisions.md`，变更史按月写进 `.claude/kb/decisions-history/<年-月>.md` 再跑 49 号 `--write`。格式与写法以 `.claude/rules/format-evolution.md`「硬约束」「决策正文只写现状，依据写成指针；决策与实验双向登记」为准；定案之后的写回由 `kb-scribe` 做（`.claude/main-agent.md`「定案之后写回 kb」那一行）。
