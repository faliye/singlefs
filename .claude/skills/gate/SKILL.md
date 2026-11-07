---
name: gate
description: 跑 singlefs 的准入门禁。提交代码前、判断一个改动能不能收时用它——包含门禁各阶段的含义、怎么判读结果、哪些"失败"是环境问题而不是代码问题。
---

正文在共享层，读它：`.claude/singlefs-ai-sop/skills/gate/SKILL.md`

**不要把正文抄到这里。**

## 在本项目里

`gate.sh` 与 `check.sh` 在本项目里是重型测试，不是快速反馈：提交时由主 agent 照 `.claude/skills/gate/staged-gate.md` 把 `bash scripts/gate/gate-staged.sh` 经 `task-server/task submit -- <命令>` 交、再用 run_in_background 起 `task-server/task wait <号>`（它跑 `gate.sh --staged`；`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行），用户要求时同样交，其余时候不跑。门禁阶段、样本自检与各类 lint 都只在提交时跑，平时不单跑；脚本与钩子的 `--selftest` 中 `.claude/hooks/selftest-registry.tsv` 登记为 light 的，改了谁就经 `task-server/task submit` 交谁、跑到绿（`.claude/agent-common.md`「门禁」一节），登记为 heavy 的与没登记的才只在提交时跑（同一个钩子在执行前拒）；`.claude/hooks/selftest-registry.tsv` 里登记为 light 的单独一个 `--selftest` 例外，谁都能跑。共享正文的阶段表不全，现有阶段以 `.claude/singlefs-ai-sop/scripts/gate.sh` 里 `run_stage` 那几行与 `.claude/gate.d/` 目录为准。
