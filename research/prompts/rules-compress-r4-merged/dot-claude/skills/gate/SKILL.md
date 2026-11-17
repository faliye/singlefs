---
name: gate
description: 跑 singlefs 的准入门禁。提交代码前、判断一个改动能不能收时用它——包含门禁各阶段的含义、怎么判读结果、哪些"失败"是环境问题而不是代码问题。
---

正文在共享层，读它：`.claude/singlefs-ai-sop/skills/gate/SKILL.md`。不把正文抄到这里。

## 在本项目里

- `gate.sh` 与 `check.sh` 是重型测试，不是快速反馈：只在提交时（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行）与用户要求时，由主 agent 照 `.claude/skills/gate/staged-gate.md` 交 `bash scripts/gate/gate-staged.sh`（它跑 `gate.sh --staged`），其余时候不跑；提交之外要跑的确认照 `.claude/rules/verification.md`「一个门禁：两种跑法」「提交之外」那一条。
- 门禁阶段（含各类 lint）不带 `--check`、`--item` 整道跑是不指定范围，同样只在提交时跑；带 `--check`、`--item` 的是指定范围，照 `.claude/rules/verification.md`「一个门禁：两种跑法」。
- 脚本与钩子的 `--selftest`、门禁样本档：改了谁，主 agent 经 `task-server/task submit` 交谁、跑到绿；子 agent 不交（`.claude/agent-common.md`「门禁」）。
- 共享正文的阶段表不全，现有阶段以 `.claude/singlefs-ai-sop/scripts/gate.sh` 里 `run_stage` 那几行与 `.claude/gate.d/` 目录为准。
