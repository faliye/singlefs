---
name: gate
description: 跑 singlefs 的准入门禁。提交代码前、判断一个改动能不能收时用它——包含门禁各阶段的含义、怎么判读结果、哪些"失败"是环境问题而不是代码问题。
---

正文在共享层，读它：`.claude/singlefs-ai-sop/skills/gate/SKILL.md`

**不要把正文抄到这里。** 抄了别的项目看不到，下次又会被抄一遍。

## 在本项目里

`gate.sh` 与 `check.sh` 在本项目里是重型测试，不是快速反馈：提交时由 `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`；`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行），其余时候不跑，主 agent 不带前缀跑会被 `.claude/hooks/heavy-test-guard.sh` 拒。平时要快速反馈，单跑 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。共享正文的阶段表不全，现有阶段以 `.claude/singlefs-ai-sop/scripts/gate.sh` 里 `run_stage` 那几行与 `.claude/gate.d/` 目录为准。
