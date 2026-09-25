# 里程碑二收尾：代码轮第二轮判决（2026-09-25）

<!-- doc-lint:not-numbers Z7 Z8 Z9 Z10 Z11 Z12 -->

## 一、这一轮的材料与证据

- **正文**：`research/prompts/_m2-final-code-r2-body.md`。
- **材料**：
  - 背景 `_m2-final-code-r2-background.md`；
  - 清单 `_m2-final-code-r2-checklist.md`；
  - 附录 `_m2-final-code-r2-appendix.md`；
  - diff `_m2-final-code-r2-diff.md`。
- **开工快照**：`research/prompts/m2-final-code-r2-snapshot/`，内有代码、定义、kb 三份 sha256。
  - 核查员交回时三份重新 `sha256sum -c` 全绿，所以腿引的行都落在快照上。
  - 主工作区的 kb 在腿跑着时被书记八、书记九改过；腿只读快照，不受影响。
- **腿与核对表**：
  - 云端攻方 `m2-final-code-r2-opus-output.md`，模型在 `m2-final-code-r2-opus-model/`；
  - 云端正推 `m2-final-code-r2-sonnet-output.md`；
  - 本地攻方 `m2-final-code-r2-local-attack-output-s1.md`、`-s2.md`；
  - 核查员 `m2-final-code-r2-verifier-output.md`：核 95 处，✓ 87、✗ 4、分不清 4。四个 ✗ 都是位置写错（「末两行」、两处定义行号、一句 grep 的命中数），核查员逐条确认不改实质；分不清的 4 处是主工作区里改动中的 hook 与门禁，不在快照里。

## 二、按路径点名

**这一轮判的 `crates/*/src/*.rs`**（第一轮冻结树到这一轮冻结树的 diff）：

- checker：
  - `crates/singlefs-checker/src/image.rs`
  - `crates/singlefs-checker/src/lib.rs`
  - `crates/singlefs-checker/src/walk.rs`
- core：
  - `crates/singlefs-core/src/allocator.rs`
  - `crates/singlefs-core/src/code_two_tree.rs`
  - `crates/singlefs-core/src/journal.rs`
  - `crates/singlefs-core/src/lib.rs`
  - `crates/singlefs-core/src/make_filesystem.rs`
  - `crates/singlefs-core/src/mounted_read.rs`
  - `crates/singlefs-core/src/mount.rs`
  - `crates/singlefs-core/src/recovery.rs`
  - `crates/singlefs-core/src/rollback_witness.rs`
  - `crates/singlefs-core/src/system_configuration.rs`
  - `crates/singlefs-core/src/transaction.rs`
  - `crates/singlefs-core/src/write_accounting.rs`
- format：
  - `crates/singlefs-format/src/lib.rs`
- harness：
  - `crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs`
  - `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`
  - `crates/singlefs-harness/src/history.rs`
  - `crates/singlefs-harness/src/model_comparison.rs`
  - `crates/singlefs-harness/src/model.rs`
  - `crates/singlefs-harness/src/on_device_modes.rs`
- **不判**：`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`。它是 E158 的实验装置，由 E158 页的变异表与单测判，这一轮只点名。

**这一轮判的定义**：
- `.claude/agent-common.md`
- `.claude/agents/crash-verifier.md`
- `.claude/agents/experiment-runner.md`
- `.claude/agents/gate-triage.md`
- `.claude/agents/implementation-writer.md`
- `.claude/agents/mutation-triage.md`
- `.claude/main-agent.md`
- `.claude/rules/implementation-workflow.md`

冻结之后又改过的几处，改法在第三节 Z11；这几处还要进第三轮点名。

## 三、逐格判

| 格 | 腿说什么 | 主 agent 现查与判 |
|---|---|---|
| Z7 树分裂 | 正推：分裂点、分隔 key 维护、收缩降高、108 / 113 两个常量都兑现 D8（核心索引结构） 已定项 11、D19（块指针的结构与宽度预算） 已定项 5；checker 按父条目核的是三样，没有单独的树高检查 | **兑现了条款**。正文问句写「四样」是我数错了：树高就是根节点头的层级 + 1，没有第二个存储位置可对照。不改代码 |
| Z8 失败原样重发、读者两格、再跨记录 | 攻方：失败点 25 + 29、重发时再失败 625 + 841、每个重发前缀崩 600 + 812，`mismatches=0` | **没打中**（攻方一次；这一格是找反例，按三方规则「没打中」一次不算观测，但这一轮只派了一条云端攻方，记成「一次没打中」，不写「站得住」） |
| Z9-A 见证表写满 | 攻方：根环 24 槽全读得出，连着 24 次回退、每次崩在写行轮换之后暖机之前，第 24 次在取号之前被拒 `RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 }`；核查员用 `rerun.sh` 逐字节复现 | **打中**。D23（journal 的角色与格式） 已定项 14「表写不满」那句不成立；C547（回退见证表的删除规则与写满没有条款）立账时的前提「有槽持续读不出」也不是唯一来路。改法见第四节 |
| Z9-B 写行根落了、见证轮换没落 | 攻方：之后 m = 1..3 次可写挂载都不补这一条，较新的根读不出时落到被抛弃的根；对照一边落到 R_old。核查员复现 | **打中**（替没写的条款做了选择：条款没说之后的挂载要不要补）。m = 0 那一格是用户选 post 写序时认的窗口（`m2-witness-r1-main-verification.md`），不算新打中。改法见第四节 |
| Z9 删除规则删掉仍护着根的条目 / 表满之前先因别的原因挂不上 | 攻方：前者没打中；后者是 C544 那一格（实二一在做） | 没打中 / 已知 |
| Z10 真设备抬 F | 正推：`requested_floor=` 打印的是内存里那一版的 F，55 号只读这一行；冷重开择 (2, 11) 不看 F；checker 的 I-7.9 只在 F 真的变大时判上界 | **替没写的条款做了选择**：55 号判「F 抬到 3」没有独立读回盘上的 F。主 agent 现查 `first_transaction_on_device.rs:1153-1155` 坐实。改法：实二五让冷重开之后打印盘上读回的 `rollback_floor_on_disk=`，55 号改判那一行 |
| Z11 定义改动 | 正推：重型测试清单与 hook 一致、「交回之后不许续做」与续做闸一致；两处说反话：`agent-common.md` 写检测器拒两种、hook 拒三种；`main-agent.md` 新加一句论证 | **和条款说反话**两处，已改（06:1x JST）：`agent-common.md` 那句写全（此后检测器又加了第四种，同步写成四种）；`main-agent.md` 删掉「它们沿用派发那一刻的定义，看不到后来的改动」。另按阶段同步回扫改了 `crash-verifier.md` 第 13 行、`implementation-workflow.md` 第 1 行的旧小节名，`main-agent.md` 看门狗叫醒条件补一条。这几处冻结之后的改动进第三轮 |
| Z12 预演与真发 | 攻方：把小容量、少点名项的测试开关强接进挂载写者，19062 次挂载成功、0 次 panic；带文件、小容量那一半多数组早早撞上分配记录墙，偏薄 | **没打中**（一次）。偏薄的那一半随实二一拆掉分配记录墙之后，第三轮再攻 |
| 算术 | 本地攻方两次抽样十格全对：47、753、1234 ≤ 4096、477 / 150、294 / 143、树高 1、80 块盘、67 | 两次都没打中，记「没打中」 |

## 四、改法（主 agent 定；用户 2026-09-25 JST 01:2x 授权不明确的自行定）

| # | 改法 | 依据 | 被攻过几轮 |
|---|---|---|---|
| 1 | 回退见证删除规则加一条：条目 (N1, r, T) 被另一条 (N2, r2, T2) 罩住（N2 ≥ N1 且 (r2, T2) ≤ (r, T)，实例代号为主比）就删 | 被罩住的那条抛弃的每一条根，罩住它的那条也抛弃，删掉之后并集判法逐字不变（等式，不靠测）；攻方 A1 在它的副本上不再写满 | 零轮 |
| 2 | 「没抛弃任何可读的根或自证记录就删」**不采纳** | 暂时读不出的根或记录之后又读得出时会被放回候选，正是删除规则 ① 要防的；攻方自己标了「推的」 | — |
| 3 | 每次可写挂载在删除规则之前，按所选根实例表里的回退行，把缺的见证条目补回来 | Z9-B：所选根实例表里带着回退行，缺的那一条从盘上推得出；攻方副本上 m ≥ 1 三格修好 | 零轮 |
| 4 | 采纳 1、3 之后，A2 那段历史（每次回退到最新根、每次崩在同一窗口）仍在第 24 次写满，照旧在取号之前拒；D23（journal 的角色与格式） 已定项 14 删掉「表写不满」，C547（回退见证表的删除规则与写满没有条款） 补上「根环全读得出也写得满」 | 攻方 A2 在 1、3 下都照样写满（量过） | — |
| 5 | 真设备二进制的抬 F 模式打印盘上读回的 F，55 号改判它 | Z10 | 零轮 |

1、3、5 交实二五（接在实二一之后，规格 `/tmp/claude-1000/impl-m2-e25/spec.md`）；4 与第 1、3 条的条款文字交书记员。1、3、5 都被攻过零轮，第三轮代码三方攻它们的实现。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 推翻 | 2026-09-25 改了：「表写不满」不成立；删除规则加「被罩住的删」、每次可写挂载按实例表回退行补条目，交书记员写回 |
| D22（单元原子性怎么合成） 已定项 9 | 不影响 | 2026-09-25 不受影响：见证表落点与宽度不变 |
| D8（核心索引结构） 已定项 11 | 不影响 | 2026-09-25 不受影响：Z7 兑现条款 |
| D19（块指针的结构与宽度预算） 已定项 5 | 不影响 | 2026-09-25 不受影响：Z7 兑现条款 |
