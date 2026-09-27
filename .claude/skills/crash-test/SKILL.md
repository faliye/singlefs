---
name: crash-test
description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设备、崩溃点重放、模型对拍。判断写路径对不对、或要给并发改动补验证时用它。
---

# 验证套件

规则在 `.claude/rules/verification.md`（两档怎么分、什么时候跑）、共享的 `.claude/singlefs-ai-sop/rules/test-discipline.md` 与 `show-me-test.md`。

## 两档

| 档 | 住哪 | 什么时候跑 | 怎么跑 |
|---|---|---|---|
| harness | `crates/singlefs-harness`：单元与集成测试 | 改了代码随时跑 | `cargo test -p singlefs-harness`（实现员经 `research/scripts/run-with-memory-cap.sh` 跑自己动到的 `--test <目标>`） |
| checker 档 | `crates/singlefs-checker-tier`（崩溃枚举引擎、注入战役、装置二进制与它们的用例）与 55、57、59、87 号 | 默认只在提交时跑快档；全量由用户要求或夜间 | 命令带 `SINGLEFS_HEAVY_TESTS=commit`（提交时）或 `=user-request`（用户要求）；没带，钩子 `.claude/hooks/heavy-test-guard.sh` 拒 |

## 四样各落一道门禁

| 手段 | 验什么 | 阶段 | 覆盖声明的键（照 `.claude/gate-not-implemented.tsv` 抄） |
|---|---|---|---|
| 崩溃点重放 | 每个崩溃点截断、恢复、跑池级 checker 与记录核对器 | `.claude/gate.d/54-layer0-replay.sh`：快档 `cargo test --release -p singlefs-checker-tier --lib --tests` 加逐条核崩溃枚举用例的全绿标记（不作数的报「本次未跑」，不判红）；全量 `--full` 在 HEAD + 暂存区的 worktree 里逐条按输入复用 | `崩溃点重放` |
| 模型对拍 | 随机历史每一步与只住内存的理想模型比 | `.claude/gate.d/74-model-differential.sh`（轻阶段，整轮门禁里跑） | `模型对拍` |
| QEMU 真设备 | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 | `.claude/gate.d/55-qemu-device-streams.sh`（重型） | 不声明覆盖 |
| LKMM 内存序 | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符 | `.claude/gate.d/57-lkmm.sh`，逻辑在 `.claude/scripts/lkmm.sh`（重型；缺 herd7 直接红） | 不声明覆盖 |

阶段头部写 `# gate-covers: <键>`，键只能是 `.claude/gate-not-implemented.tsv` 里登记的（本项目登记了 `崩溃点重放`、`模型对拍`）与共享 `gate.sh` 自带的 `最终判据`、`命名纪律（shell）`，写错一个字判红。
这个阶段这一轮跑了且通过，`gate.sh` 末尾的未实现清单才把那一项换到「由项目本地阶段覆盖」下面；报了「本次未跑」的这一轮不算覆盖。
覆盖只说明「有一个阶段在做这件事，这次过了」；覆盖到多大范围，看那个阶段自己的说明与登记表第三列的提醒句（54 号：只说到它枚举过的那些写路径为止）。
这一轮无对象可判的阶段退出码写 77：记「本次未跑」，不记通过，也不算覆盖。

## 新写一条崩溃枚举用例

写在 `crates/singlefs-checker-tier/tests/<流的名字>.rs`，全量那条标 `#[ignore]`，同文件的小流快档不标；共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 指回 harness；全量那条登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`（写法见 `research/scripts/admission.py` 文件头）。`research/scripts/crash-case-check.py` 判：写在别的包里、或标了 ignore 没登记，判红。

## 判读纪律

- **「没复现问题」不等于「没问题」。** 想说崩溃一致性成立，得先说清这一轮枚举了多少个崩溃点，是不是全部；快档只跑小流，全量那几条有没有作数的全绿标记看 54 号的输出。
- **checker 不报错，也可能是那条检查压根没实现。** 先看 `.claude/kb/invariants.md` 的状态列。
- **判定结果读不到，这一轮就作废**，绝不能当成通过。装置读不到结束标记也一样。
- 谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（54、55、57、59 号）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份。
