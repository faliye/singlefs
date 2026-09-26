---
name: crash-test
description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 压测、崩溃点重放、模型对拍。判断写路径对不对、或要给并发改动补验证时用它。
---

正文在共享层，读它：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`

**不要把正文抄到这里。** 抄了别的项目看不到，下次又会被抄一遍。

## 在本项目里

共享正文不管内存序与虚机（它写明那两样由项目自己定）。本项目的四样各落在一道门禁：崩溃点重放是 `.claude/gate.d/54-layer0-replay.sh`，QEMU 真设备是 `55-qemu-first-transaction.sh`，LKMM 内存序是 `57-lkmm.sh`，模型对拍是 `74-model-differential.sh`。前三样是重型测试，谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员与门禁分诊员各跑登记给自己的那几道，别的子 agent 不跑，钩子 `.claude/hooks/heavy-test-guard.sh` 会拒。
