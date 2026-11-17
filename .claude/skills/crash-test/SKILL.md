---
name: crash-test
description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设备、崩溃点重放、模型对拍、Verus 形式化验证。判断写路径对不对、或要给并发改动补验证时用它。
---

# 验证套件

规则在 `.claude/rules/verification.md`（验证包 `crates/singlefs-verification` 装什么、什么时候跑）、`.claude/rules/test-discipline.md` 与 `.claude/singlefs-ai-sop/rules/show-me-test.md`。

## 验证包

验证包与门禁 qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation、research-build-and-replay 的复跑那一格没有分档：点名测试函数与项的，改了代码随时跑，实现员把自己动到的测试函数写成 `cargo test -p <包> --test <目标> -- <测试全名…> --exact`（只点 `--test <目标>` 的提交时被服务拒）、门禁与入口写成带 `--item` 的那一条，写进报告、主 agent 经 `task-server/task submit` 交；不指定范围的，主 agent 只在提交时跑，提交之外要跑照 `.claude/rules/verification.md`「一个门禁：两种跑法」「提交之外」那一条确认一次。

## 五样各落一道门禁

| 手段 | 验什么 | 阶段 | 覆盖声明的键（照 `.claude/gate-not-implemented.tsv` 抄） |
|---|---|---|---|
| 崩溃点重放 | 每个崩溃点截断、恢复、跑池级 checker 与记录核对器 | `.claude/gate.d/crash-point-replay.sh`，各格判什么在 `.claude/rules/verification-reference.md`「门禁管哪一半」 | `崩溃点重放` |
| 模型对拍 | 随机历史每一步与只住内存的理想模型比 | `.claude/gate.d/model-differential-and-scenarios.sh`（整轮门禁里跑） | `模型对拍` |
| QEMU 真设备 | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 | `.claude/gate.d/qemu-device-streams.sh`（重型） | 不声明覆盖 |
| LKMM 内存序 | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符 | `.claude/gate.d/lkmm.sh`，逻辑在 `.claude/scripts/lkmm.sh`（重型；缺 herd7 直接红） | 不声明覆盖 |
| Verus 形式化验证 | 恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过（版本钉住、证明、空真、必红世界），E165 的红例钉进钉表逐行重放 | `.claude/gate.d/formal-verification.sh`，判法在 `scripts/gate/verus-proof-check.py` 与 `scripts/experiment-apparatus/e165-pin-red-examples.py`（重型） | 不声明覆盖 |

- 阶段头部写 `# gate-covers: <键>`，键只能是 `.claude/gate-not-implemented.tsv` 里登记的（本项目登记了 `崩溃点重放`、`模型对拍`）与共享 `gate.sh` 自带的 `最终判据`、`命名纪律（shell）`，写错一个字判红。
- 覆盖了什么、报了「本次未跑」与退 77 怎么记，照 `.claude/singlefs-ai-sop/rules/show-me-test.md`「最终判据由项目定」「门禁不许假装通过」。覆盖只说明「有一个阶段在做这件事，这次过了」；覆盖到多大范围，看那个阶段自己的说明与登记表第三列的提醒句（门禁 crash-point-replay：只说到它枚举过的那些写路径为止）。

## 新写一条崩溃枚举用例

写在 `crates/singlefs-verification/tests/<流的名字>.rs`，不标 `#[ignore]`；共用的搭建模块用同一个 `tests/` 下的 `common` 一类模块（`mod common;`）；登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`（写法见 `scripts/gate/admission.py` 文件头），再让崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 的一项在第二列点名它。`scripts/crash/crash-case-check.py` 判：写在别的包里、标了 `#[ignore]` 或没登记，判红。

## 判读纪律

- 「没复现问题」「checker 不报错」照 `.claude/rules/verification.md`「文件系统特有的反推缺口」：说崩溃一致性成立之前，先说清这一轮枚举了多少个崩溃点、是不是全部；登记的崩溃枚举用例的崩溃状态块罩住没有，看门禁 crash-point-replay 的输出；checker 不报错，先看 `crates/singlefs-checker/invariants/` 的状态列。
- 判定结果读不到、装置读不到结束标记，这一轮作废，不当成通过（`.claude/rules/test-discipline.md`「读不到 ≠ 读到 0」）。
- 谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「不指定范围的只在提交时跑，指定范围的随时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 不往统一任务服务交任务，带 `--item` 的四条入口与别的跑项目代码的命令都写进报告「要主 agent 跑的命令」一节交主 agent（`.claude/agent-common.md`「不做」一节「子 agent 不交任务」「门禁只有一个，跑法两种」两条）；主 agent 照 `.claude/skills/crash-test/commit-stages.md`（六道重阶段）与 `.claude/skills/gate/staged-gate.md`（整轮门禁 `scripts/gate/gate-staged.sh`）跑。
