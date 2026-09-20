# m2-supp3-item2-code-r1 正推腿（Sonnet）报告：M1、M6

时刻 UTC（JST = UTC+9）。

## 停机：开工快照第二次核对不上

按协调消息「开工快照再 `sha256sum -c` 一次，对不上就停下报告」，复核结果：

```
$ sha256sum -c research/prompts/m2-supp3-item2-code-r1-start-snapshot.sha256
crates/singlefs-harness/src/model.rs: OK
crates/singlefs-harness/src/model_comparison.rs: OK
crates/singlefs-harness/src/history.rs: OK
crates/singlefs-harness/src/lib.rs: OK
crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs: OK
crates/mutations.tsv: OK
.claude/gate.d/74-model-differential.sh: OK
.claude/kb/milestone/02-second-txn.md: OK
.claude/kb/decisions/13-验证路线.md: OK
.claude/kb/decisions/16-发布语义.md: OK
.claude/kb/decisions/23-journal的角色与格式.md: OK
.claude/kb/decisions/04-校验和位置.md: OK
.claude/kb/decisions/18-块里携带什么信息.md: OK
.claude/kb/decisions/28-挂载期承诺量.md: FAILED
.claude/kb/decisions/03-空间分配.md: OK
.claude/kb/decisions/05-快照-空间记账机制.md: OK
.claude/kb/decisions/22-单元原子性怎么合成.md: OK
.claude/kb/decisions/08-核心索引结构.md: OK
.claude/kb/invariants.md: OK
sha256sum: WARNING: 1 computed checksum did NOT match
```

只有 `.claude/kb/decisions/28-挂载期承诺量.md` 一份对不上；同一份文件在本轮开工时第一次核对（会话最早一步）是 `OK`（19 个文件当时全部 `OK`，记录见本报告末尾「开工时的第一次核对」）。`git status --short` 现查：

```
$ git status --short .claude/kb/decisions/28-挂载期承诺量.md
 M ".claude/kb/decisions/28-\346\214\202\350\275\275\346\234\237\346\211\277\350\257\272\351\207\217.md"
```

`git diff` 显示这是一次改写体例、不改判据本身的编辑（去掉「四项全定」这类历史腔、把定案摘成短句、表格与「依据」小节重排），像是另一个并发会话在做 kb 腐化回扫（`.claude/singlefs-ai-sop/rules/writing-discipline.md`），不是本轮任务范围内的改动。哈希：现在 `5d9f9283…`，快照要的是 `14b347b8…`。

**按定义「开工先读」与协调消息的指令，本轮到此为止，不再继续判 M1 / M6 的新内容。** 下面「已做的核实」一节是停机之前已经完成、且不依赖 `28-挂载期承诺量.md` 现在这份内容的部分（详见每条后面标注的依据来源），留给主 agent 参考；它们不构成对整份分工的完整交付，因为 M1 分派的表格与 M6 的样本覆盖度检查还有未核完的部分（见「没做什么」）。

## M1：模型答案与条款一致性（停机前已核完的部分）

依据：实现员报告 `research/prompts/m2-supp3-item2-implementer-report.md` 第三、六、七节；条款原文取背景材料 `research/prompts/_m2-supp3-item2-code-r1-background.md` 里对应的「出处」代码块（这些是冻结引文，不受 D28 现在改动的影响，因为它们是当时抄录的快照文本，不是现查活文件）；代码取 `crates/singlefs-harness/src/model.rs` 现状（其哈希与快照一致，`OK`）。

### 「预想」标记：数目与位置

```
$ grep -n '预想' crates/singlefs-harness/src/model.rs
12://! 照代码今天的读法写，每一处标「预想，跟收口表第 X 行」。
633:    /// 预想，跟收口表第 ② 行（……）。
662:    /// 一块盘上没有有效根时不算它（……）。预想，跟收口表第 ② 行
1022:        // 回退的 jsn 接在环里最大的 jsn 之后（C340 取 P2）：预想，跟收口表第 ① 行。
1025:        // 新实例的根带的 F = 恢复后的 F_生效……：预想，跟收口表第 ② 行。
1224:    /// - 分配记录树（预想，跟收口表第 39 行）：……
1230:    ///   保留池 10 + 7 c_max……）在 64 倍里）。预想：D28 已定项 1 的准入式子
```

一致：**5 处**（第 12 行是解释「预想」这个记法本身的模块级注释，不是第 6 处答案；第 1224 与 1230 行是同一处 `capacity_wall_is_permitted` 的两段注释，算一处）。与报告第三节所列的 5 处逐条对上（`effective_rollback_floor`、`rollback_floor_ceiling`、jsn 接续、新实例 F、分配记录树墙）。⚠️ 第 1230 行这一句直接点名 D28（挂载期承诺量） 已定项 1；D28 那份文件在本轮里途被改写（见上），这句引用是否仍成立要在文件稳定之后重核，本轮未核。

### 抽查的答案-条款对照（逐条给命令与命中行号）

**冷启动择根序（D22 已定项 7）**：`model.rs` 第 42–45 行 `ModelRootKey` 注释「择根按 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7）：字段次序就是比较次序」，字段声明 `checkpoint_txg` 在前、`instance` 在后（derive 的 `Ord` 按字段声明序比较），与 `.claude/kb/decisions/22-单元原子性怎么合成.md:488`「实例代号……择新在 checkpoint_txg 平局时按它高者赢」逐字一致（`grep -n '择新在 checkpoint_txg 平局时按它高者赢' .claude/kb/decisions/22-单元原子性怎么合成.md` 命中第 488 行）。一致。

**回退候选判定 / 被抛弃判据（D23 已定项 14）**：`model.rs` 第 651–657 行 `is_abandoned_by`：`table.iter().any(|row| row.instance == root.key.instance && root.key.checkpoint_txg > row.selected_root_txg)`，即「有该实例的行 (i, Ti, Wi) 且 txg > Ti」才算被抛弃。`.claude/kb/decisions/23-journal的角色与格式.md:1209`（`grep -n '候选集 = 根环里按实例表判仍然有效' .claude/kb/decisions/23-journal的角色与格式.md` 命中第 1209 行）原文：「(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti」——取反即「有行且 T > Ti 才不可选（被抛弃）」，与代码逐字对应。一致。`answer_mount_rollback`（第 949–991 行）与 `rollback_floor_ceiling`（第 665–701 行）都用 `self.newest_root().instance_table_rows` 作为判被抛弃的那张表；条款原文没有显式说「用哪一版实例表」，但按同一份 kb 的语义，实例表是随每次发布 COW 演进的单一现行结构（不是每条根各自一份互相独立的表），"最新根指着的表"就是唯一存在的「现行」版本，模型对每条根都用它判，读法与「按实例表判仍然有效」这句的唯一自洽读法一致——不是另一种未标注的解释分支，判：**一致**。

**内容装不装得下（D4 已定项 5；D18 已定项 16）**：`model.rs` 第 212 行起 `data_unit_payload_capacity_in_bytes`（`grep -n 'pub fn data_unit_payload_capacity_in_bytes' crates/singlefs-harness/src/model.rs` 命中第 212 行）算出 `32768 − 105 − 29 = 32634`。`.claude/kb/decisions/04-校验和位置.md:290`附近「已定项 5：32 KiB 含头……单元恒 32768 字节」（grep 命中：`已定项 5（2026-09-01，用户定案）：32 KiB 含头` 在该文件出现，行号见背景材料出处标注 `285-306`）定住单元总长；`.claude/kb/decisions/18-块里携带什么信息.md` 已定项 18 的字段表给码 1「明文头 105……含预留位 134」，已定项 16 定「29 字节预留位……算进头」，两者相加 134，`32768 − 134 = 32634`。与报告第三节该行一致。

**每条写出的根：分配代比法（D3 已定项 3/7；D16 已定项 9）**：`model.rs` 第 1559–1591 行 `judge_allocation_generations`：按 `role`（角色）取 `expected.role_written_at.get(role)`（模型自己记录「这个角色是哪次发布写的」，不读实现的输出），比对 `records.len() == 设备数`、`devices_seen == devices_expected`、`!is_released`、`record.generation == *written_at` 四条。`role_written_at` 由 `next_root`（第 725–767 行）按 `kind.rewritten_roles()`（D16 已定项 9：哪些角色每次发布重写）维护，与实现的 `TransactionOutput::units`（观测值，来自胶水）完全分离——判据用的是模型自己的历史记账，不是从实现读回再拿去比对实现自己，满足 V4「模型与实现不共用代码」。N2 变异（`generation == *written_at` 改 `>=`）确实命中这一行（`crates/mutations.tsv` 第 154 行），日志 `research/prompts/m2-supp3-item2-implementer/N2-model-only.log` 显示改坏之后判红。一致。

### 核心/checker 未改、模型模块只用 std/singlefs_format

```
$ git diff --stat HEAD -- crates/singlefs-core crates/singlefs-checker
（无输出，零差异）
$ git status --short crates/
 M crates/mutations.tsv
 M crates/singlefs-harness/src/history.rs
 M crates/singlefs-harness/src/lib.rs
 M crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
?? crates/singlefs-harness/src/model.rs
?? crates/singlefs-harness/src/model_comparison.rs
```

与报告第二节、第一节「`crates/singlefs-core` 与 `crates/singlefs-checker` 一行没改」一致。

```
$ grep -n '^use' crates/singlefs-harness/src/model.rs
14:use std::collections::{BTreeMap, BTreeSet};
15:use std::rc::Rc;
17:use singlefs_format::{
```

`model_comparison.rs` 里 `model_comparison::tests::the_model_module_uses_only_the_standard_library_and_the_format_constants`（第 263–285 行）逐行扫 `model.rs` 源码文本，断言其中没有 `singlefs_core` / `singlefs_checker` 字样、且每个 `use` 行只能是 `use std::` / `use singlefs_format::` / `use super::*;`。变异表第 155 行（在 `use std::rc::Rc;` 后插一行 `use singlefs_core as _;`）命中这条断言，日志 `research/prompts/m2-supp3-item2-implementer/M6-import-core.lib.log` 判红。一致，且这条约束有会红的检查撑着，不是只写在报告里的自称。

## M6：门禁 74 号在什么输入上判绿却没对拍；报告的数与产物、代码对不对得上

### 74 号自身的逻辑边界（读代码得出，非猜测）

`.claude/gate.d/74-model-differential.sh` 对三段（快档、复用取样点、回退取样点）各自只查两件事：①报告里有没有出现「模型对拍 N 步」那一行；②N 是不是 > 0。它**不**在这个阶段里重新核对「该拒而拒 / 区间里拒 / 该成而成」三个子计数加起来是否等于 N，也不检查是否出现过 `ModelDisagreement`——脚本自己的注释写明这一点（「这个阶段只判『跑了、判过、没报对不上』」），把「对不上就会被抓到」这件事的判别力全部交给 `cargo test` 本身的失败退出码（`judge_by_model` 一旦对不上，执行器把它计成新发现，测试里的 `assert_eq!(…, 新发现 0)` 之类断言就会红，`cargo test` 非零退出，74 号在 `!cargo test …` 分支直接判红）。这是自洽的设计，不是遗漏——真正验证「模型答错会不会被抓到」的判别力，按脚本自己的注释和实现员报告，落在 `crates/mutations.tsv` 第 146–155 行（门禁 59 号）。我把这 10 行在副本上的判红结果核对了一遍（见下）。

### 亲手核对：74 号在两份样本上的实测输出与 `expect` 逐字比对

```
$ bash .claude/gate.d/74-model-differential.sh .claude/gate.d/fixtures/74-model-differential.sh/green; echo exit=$?
  ✓ 模型对拍三段都判过、实现与模型没有对不上的（查了 3 段）：
      随机历史快档：模型对拍 1912 步：……
      随机历史：偏向抬 F 之后复用的取样点：模型对拍 1183 步：……
      随机历史：偏向抬 F 之后回退的取样点：模型对拍 913 步：……
exit=0

$ bash .claude/gate.d/74-model-differential.sh .claude/gate.d/fixtures/74-model-differential.sh/red; echo exit=$?
  ✗ 测试跑过了，这几段却没有「模型对拍 N 步」那一行：「随机历史快档」
     → 怎么办：……
exit=1
```

与两份 `expect`（`exit=0`/`want=模型对拍三段都判过`/`want=模型对拍 1912 步`/`want=查了 3 段`；`exit=1`/`want=这几段却没有「模型对拍 N 步」那一行：「随机历史快档」`）逐字对上。这一对样本正确挂在通用自检阶段 `.claude/gate.d/89-stage-selftest.sh` 上（它按 `.claude/gate.d/fixtures/<脚本文件名>/{red,green}/expect` 的约定扫描全部 `.claude/gate.d/*.sh`，`74-model-differential.sh` 的目录名与脚本文件名一致），本轮没有跑 89 号全量（会牵动全部门禁阶段的样本，超出这次 M6 的核对范围，且在停机之前来不及做完），只手工单跑了 74 号本身，效果与 89 号会做的核对相同。

### 找到的样本覆盖缺口（M6 打中的一格）

74 号脚本有两条独立的失败分支：`missing`（三段里某段完全没打出「模型对拍 N 步」那一行）与 `zero`（那一行出现了，但 N = 0）。样本目录只有 `green` 与 `red` 两份：

```
$ ls .claude/gate.d/fixtures/74-model-differential.sh/
green
red
$ grep -n '模型对拍 [0-9]* 步' .claude/gate.d/fixtures/74-model-differential.sh/red/model-differential-cargo-output.log
47:  模型对拍 913 步：……
113:  模型对拍 1183 步：……
```

`red` 样本里三段中只有「随机历史快档」那一段整行被删掉（对应 `missing` 分支），另外两段仍然是正常的非零步数——**没有任何样本走过 `zero` 分支**（脚本里 `if [[ -z "$steps" || "$steps" == 0 ]]` 那一支，及其对应的失败信息「这几段模型一步都没判」）。也就是说，如果以后有人在 `zero` 分支的字符串匹配、变量置空判断（`[[ -z "$steps" ]]`）或 `sed` 抽取表达式上引入一个只在「行存在但步数是 0」这种输入下才触发的 bug，`.claude/gate.d/89-stage-selftest.sh` 现有的这两份样本**看不出来**：它们能证明「整行缺失」会被抓到，证明不了「行在但步数是 0」会被抓到。

**推翻条件**：如果 `.claude/gate.d/fixtures/74-model-differential.sh/` 之后补了第三份样本（或在 `red` 里把某一段改成「模型对拍 0 步：……」而不是整行删掉）并且 89 号选中了它、判红，这条就不成立了。反过来，如果把脚本里 `(( ${#zero[@]} > 0 ))` 那一段代码删掉或改错，现有两份样本会全绿（`missing` 分支还能触发但 `zero` 分支的代码路径完全没有测试盯着）——这是一条会红的具体断言：改坏 `zero` 分支判断、复跑 89 号，这条经过应当红而不会红。此断言本轮没有实测（会改脚本，超出「读、跑现成样本」的范围，留给主 agent 或下一轮判是否要补）。

### 报告的数与 `research/prompts/m2-supp3-item2-implementer/` 下产物逐条核对

报告第十节写「跑出来的东西都在 `/tmp/claude-1000/m2-supp3-item2-implementer/`……没进 `research/results/`」；但派发给我的分工表明写「实现员报告里的数与产物 `research/prompts/m2-supp3-item2-implementer/` 下的日志」——两者不矛盾：`research/prompts/m2-supp3-item2-implementer/` 这个目录现查确实存在且有完整日志（`ls` 命中 apply-mutation.py、`B1.log`…`N2.log`、`mutants-summary.txt`、`check.log` 等全部报告里点名的文件），推测是主 agent 在拿到报告之后把 `/tmp` 里的产物搬了进来；这是状态变化，不是报告说谎，仅记录在此供核对。

逐条核对（命令与命中原样）：

**B1**：`research/prompts/m2-supp3-item2-implementer/B1.log` 第 216–217 行：
```
新发现 ModelDisagreement { aspect: "模型说该成、实现拒了" }：第一个种子 1（……），在 Operation(16)（Some(PublishOverwrite)）之后
  模型对不上（模型说该成、实现拒了）：模型答 PublishOverwrite 该成（写出 [(21, 4)]；……）；实现 拒了：PublishError::ContentExceedsDataUnit
```
与报告表格「种子 1，第 16 步（覆盖写）……模型答 PublishOverwrite 该成（写出 [(21, 4)]……实现答拒了：PublishError::ContentExceedsDataUnit」逐字一致；三档新发现段数（22 / 36 / 47）分别对上该 log 里三段各自的「新发现」计数（22、36、47）。

**B2**：`B2.log` 第 217 行「种子 6……MountRollback 该拒：["回退到树表 0 条的根（第一版不支持）"]；实现 拒了：MountError::RollbackTargetNotACandidate（txg 低于生效的回退下界 F）」与第 219 行「Operation(10)（Some(CloseAndMountRollback)）」——与报告「种子 6，第 10 步（回退）」逐字一致；「语义那一格」引的「种子 7、36 的第 16 步」在 `B2.log` 第 8–9 行区块内以「第一个种子 7（同签名的种子 [7, 36]），在 Operation(16)」出现，答案「MountRollback 该成（写出 [(19, 4), (20, 4)]……」逐字一致；三档新发现（48 / 2 / 27）与该 log 三个区块的「新发现」字段（48、2、27）一致。

**mutations.tsv 第 146–155 行**：抽查三条锚点原文在 `crates/singlefs-core` 里各恰好命中一次：
```
$ grep -n 'if file.content.len() > data_unit_capacity {' crates/singlefs-core/src/transaction.rs
1248:        if file.content.len() > data_unit_capacity {
$ grep -n 'if target.checkpoint_txg < effective_floor {' crates/singlefs-core/src/mount.rs
1203:    if target.checkpoint_txg < effective_floor {
$ grep -n 'Some(newest_on_every_device.min(fourth_newest))' crates/singlefs-core/src/mount.rs
575:    Some(newest_on_every_device.min(fourth_newest))
```
与 `gate59-new-rows.log` 末尾「crates 变异表复跑：10 条变异各自红在点名的测试上（原文都恰好命中一次）」一致。

**gate 33 / 53 / check.sh**：`gate33-final.log`、`gate53-final.log`、`check.log` 的末行与报告第八节引的原样逐字相同（分别核对过）；`check.log` 第 492 行是「── 随机历史快档 ──」、第 526 行是「模型对拍 1912 步：……」，与报告「快档那一段在里面第 492 行起，模型那一行第 526 行」一致。

**history.rs 行号**：`awk 'NR==593'` 命中 `fn judge_by_model`，`awk 'NR==1081'` 命中 `fn only_allocated_statistic_above_walked`，均与报告及背景材料所引行号一致。`crates/singlefs-core/src/mount.rs` 第 40–43 行确为 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 的文档注释，与报告第六节第 1 条所引行号一致。

**结论**：抽查到的每一处数字、行号、原样输出都与产物、代码逐字对上，没有找到报告与实测不符的地方；这一部分核对**在 D28 快照对不上之前已经完成**，不依赖那份文件，判定维持有效。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| M1：冷启动择根序（D22 已定项 7） | 一致 | 字段序即比较序，与「txg 平局按实例代号高者赢」逐字对应 |
| M1：回退候选 / 被抛弃判据（D23 已定项 14） | 一致 | `is_abandoned_by` 与「有行且 T > Ti」条款取反逐字对应 |
| M1：哪一张实例表参与判定 | 一致（非条款字面，但唯一自洽读法） | 条款未点名版本，但实例表是单一演进结构，「最新根指着的」是唯一读法，不是另立分支 |
| M1：内容装不装得下（D4 已定项 5、D18 已定项 16） | 一致 | 32768−105−29=32634 可从已定项 18 码 1 字段表与已定项 16 的 29 字节预留位直接推出 |
| M1：分配代比法（D3 已定项 3/7、D16 已定项 9） | 一致，且与实现解耦 | 模型用自己维护的 `role_written_at`，不读实现的输出再拿去比对实现自己 |
| M1：「预想」标记数目 | 一致 | 代码里恰好 5 处（第 12 行是解释记法本身，不计入） |
| M1：D28 已定项 1 那处引用（`capacity_wall_is_permitted`） | 未核（停机） | 该文件在本轮里途改写，本轮没有对照改写后的正文重核这处引用是否仍成立 |
| M1：核心 / checker 未改，模型模块只用 std/format | 一致 | `git diff --stat` 零差异；`use` 只有 3 行，且有会红的断言（变异表第 155 行）守着 |
| M6：74 号的判别力射程 | 一致，设计自洽 | 74 号只判「跑了、N>0」，判别力交给测试自身断言 + 门禁 59 号的变异复跑，脚本注释自述了这个分工 |
| M6：74 号在两份样本上的实测 | 一致 | green/red 的实测输出与 `expect` 逐字对上；正确挂在 89 号自检 |
| M6：样本覆盖缺口（`zero` 分支） | **冲突（找到的缺口）** | 只有 `missing` 分支有样本，`zero` 分支（步数为 0）没有任何样本覆盖，89 号现有样本抓不到那条代码路径上的 bug |
| M6：报告数字与产物、代码逐条核对（B1、B2、mutations.tsv、gate33/53/check.log、行号） | 一致 | 抽查到的每一处都逐字对上，没有发现不符 |

## 没做什么

- **因开工快照第二次核对不上而停机**：M1 表格里还有「回退到这条根该不该被拒」的完整四条判据逐条核、「抬 F 该不该被拒」除已核的上限公式与「允许拒」之外的其余分支、「挂载写的行与取到的号」这一行、以及 D28（挂载期承诺量） 已定项 1/3/4 在报告第七节「容量墙」的另一处引用（单元区那一面墙），都还没有核完；这些没有核就停在这里，不代表它们有问题，也不代表它们没问题。
- 没有跑 `.claude/gate.d/89-stage-selftest.sh` 全量（会牵动全部门禁阶段的样本，超出这次单点核对 74 号的范围）；只手工单跑了 74 号本身对两份样本的输出，效果等价。
- 没有编译 Rust、没有跑 `check.sh`、没有起任何后台任务（按共用约束「不做」一节与「不编译 Rust」）；所有代码层的核对都是读源码 + 现成日志文件的比对，没有重新生成任何产物。
- 没有验证「D28 已定项 1 那处引用是否仍成立」——这份文件本轮里途被改写，稳定之后需要重核 `model.rs` 第 1230 行那句引用与改写后的 D28 正文是否还对得上。
- 不判 M2、M3、M4、M5（归 Opus 攻方腿）；不判本地攻方的算术格；不替攻方找变异（按定义「写范围」与「没做什么」）。
- 没有对 74 号是否存在别的、我没找到的判绿-却-没对拍的输入形态做穷举——只核了脚本读代码得到的两条已知分支（`missing`/`zero`）与实测跑过的两份样本；不排除还有别的输入（例如 `cargo test` 本身跑通但某一段的报告块因为多线程被截断/交错导致 `awk` 边界解析错位）没有被样本覆盖到，本轮没有编译运行去做交错压力测试（受「不编译 Rust」约束），只在多份现成日志（check.log、B1.log、B2.log 等约 15 份采样）里观察到三段的输出顺序与边界一直干净、没有交错的迹象，作为间接证据，不是穷举证明。

## 附：开工时的第一次核对（会话最早一步，供对照）

```
crates/singlefs-harness/src/model.rs: OK
crates/singlefs-harness/src/model_comparison.rs: OK
crates/singlefs-harness/src/history.rs: OK
crates/singlefs-harness/src/lib.rs: OK
crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs: OK
crates/mutations.tsv: OK
.claude/gate.d/74-model-differential.sh: OK
.claude/kb/milestone/02-second-txn.md: OK
.claude/kb/decisions/13-验证路线.md: OK
.claude/kb/decisions/16-发布语义.md: OK
.claude/kb/decisions/23-journal的角色与格式.md: OK
.claude/kb/decisions/04-校验和位置.md: OK
.claude/kb/decisions/18-块里携带什么信息.md: OK
.claude/kb/decisions/28-挂载期承诺量.md: OK
.claude/kb/decisions/03-空间分配.md: OK
.claude/kb/decisions/05-快照-空间记账机制.md: OK
.claude/kb/decisions/22-单元原子性怎么合成.md: OK
.claude/kb/decisions/08-核心索引结构.md: OK
.claude/kb/invariants.md: OK
```

全部 19 个文件当时都是 `OK`；`.claude/kb/decisions/28-挂载期承诺量.md` 是在本轮会话进行中途被改动的（另一个并发会话，`git diff` 显示是体例回扫、非判据改动），本轮工作本身没有写过这个文件。

## 续（收到协调消息后继续）：D28 用快照副本核，M1 剩余行

协调确认：`.claude/kb/decisions/28-挂载期承诺量.md` 的改动是另一个会话做的正文瘦身，不算这一轮的事；快照时刻的原样已由主 agent 取出到 `research/prompts/m2-supp3-item2-code-r1-snapshot-copies/.claude/kb/decisions/28-挂载期承诺量.md`。核对：

```
$ sha256sum "research/prompts/m2-supp3-item2-code-r1-snapshot-copies/.claude/kb/decisions/28-挂载期承诺量.md"
14b347b8880a34ecab8eddf13a39cc3d6178c5e79863eb5a005848063a7c6c44  research/prompts/m2-supp3-item2-code-r1-snapshot-copies/.claude/kb/decisions/28-挂载期承诺量.md
$ grep '28-挂载期承诺量' research/prompts/m2-supp3-item2-code-r1-start-snapshot.sha256
14b347b8880a34ecab8eddf13a39cc3d6178c5e79863eb5a005848063a7c6c44  .claude/kb/decisions/28-挂载期承诺量.md
```

哈希逐字相同，确认这份副本就是快照时刻的原样。下面涉及 D28 的引用一律按这份副本核，行号是副本自己的行号，注明「快照副本」；别的 18 个文件照旧对主树核（未变，见上文）。

### 回退到这条根该不该被拒：四条判据逐条对条款

`model.rs` 第 949–991 行 `answer_mount_rollback` 按序判四条，判据名就是 `ModelRefusalReason` 的四个成员，其文档注释（第 217 行起的 `enum ModelRefusalReason`，逐条现取行号）直接给出条款出处：

| 判据 | 代码行 | 文档注释（原文） | 条款核对 |
|---|---|---|---|
| 不在环里 | 第 962–966 行，枚举成员注释第 235 行 | 「回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）」 | `.claude/kb/decisions/23-journal的角色与格式.md:1209`「候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效……的根」——候选集本就限定在根环内，一致 |
| txg < F_生效 | 第 968–970 行，枚举成员注释第 237 行 | 「回退目标的 txg 低于 F_生效（D16（发布语义） 已定项 1「回退候选集」）」 | `.claude/kb/decisions/16-发布语义.md:375`「回退候选集 \| 按实例表判仍然有效 ∧ txg ≥ F_生效」，取反即 txg < F_生效 时拒，一致 |
| 被最新根的实例表判成被抛弃 | 第 971–973 行，枚举成员注释第 239 行 | 「回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）」 | 与前文已核的 `is_abandoned_by` 同一条判据，逐字对应，一致 |
| 树表 0 条（第一版不支持） | 第 974–976 行，枚举成员注释第 241 行 | 「回退到树表 0 条的根：回退行要重写实例表，落点记在哪没有条款，第一版不支持（照代码今天的读法）」 | 条款确未写这一格（我在 D23/D16/D22 相关正文里没有找到「回退到没有文件的根」的显式处置句），报告第六节第 2 条也把它列为「条款没写、照代码今天的读法判必须拒」的三个「第一版不支持」之一，口径一致 |

四条与报告第三节「回退到这条根该不该被拒」一行「不在环里、txg < F_生效、被最新根的实例表判成被抛弃、树表 0 条（第一版不支持）四条逐条判」逐字对应。**判定：一致。**


### 抬 F 该不该被拒：其余分支

已在停机前核过上限公式本身（`rollback_floor_ceiling`）与「允许拒」的 `RaiseWithFormatTimeInstanceTableUnsupported` 一格（报告第六节第 1 条）。这次补核剩下两条必须拒的判据：

- **`FloorAboveCeiling`**（`model.rs` 第 1140–1143 行，枚举成员注释第 243 行「要抬的 F 超过上限（D16（发布语义） 已定项 1「抬 F 的上限」）」）：`.claude/kb/decisions/16-发布语义.md:376`「抬 F 的上限 \| min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根」——与 `rollback_floor_ceiling` 函数体（第 665–701 行，已在停机前核过算法本身）算出的 `ceiling` 一起用，超过就拒，逐字对应。**一致。**
- **`current.file.is_none()` 时的 `SessionState` 分支**（`model.rs` 第 1120–1126 行）：这不是一个 `ModelRefusalReason`，是 `ModelDisagreement`（模型与实现的判定基础对不上，不是「条款要求拒」），报告第三节「抬 F 该不该被拒」一行没有把它算进「四选一」的答案空间——这是执行前提检查（执行器只在带文件时调抬 F），不是条款判据，判：不适用「条款一致性」这一格，无冲突。
- **`new_floor < current.rollback_floor` 时的 `NotModeled` 分支**（第 1127–1136 行）：报告第六节第 4 条「往下抬 F……模型不答」明确承认这一格模型不建模、只报「条款没写、模型不答」，与代码逐字对应，且报告自己说明了「走不到」的理由（生成器只取现行 F 及以上）。一致，且是显式承认的空白，不是漏标的分支。

**判定：一致。**

### 挂载写的行与取到的号（D18 已定项 11；D23 已定项 16）

- **取号**：`model.rs` 第 1001 行 `let instance = ModelInstanceGeneration(self.highest_acquired_instance.0 + 1);`，与第 47 行模块级常量注释「mkfs 的实例代号与第 0 代根的 txg（D23（journal 的角色与格式） 已定项 16：mkfs 写 0，第一次取号取 1）」互相印证。`.claude/kb/decisions/23-journal的角色与格式.md:695`（已在停机前读过，属于冻结附录引文，不受 D28 改动影响）：「mkfs 写 0……第一次可写挂载……取 max(超级块, 根环) + 1 = 1」——与代码「最高已取实例代号 + 1」逐字对应（`highest_acquired_instance` 就是 max(超级块, 根环) 在模型内的等价维护量）。**一致。**
- **写行**：`model.rs` 第 1002–1020 行，逐实例填 `[max(上一个实例, 1), 新实例)` 区间的行，上一个实例（或被退回的实例）写 `previous_row`，中间实例写 `(i, 0, 0)`；注释引「D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14」。D23 已定项 14（`.claude/kb/decisions/23-journal的角色与格式.md:1209`，已在停机前核过）的回退例外句「在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)」——「中间实例 (i, 0, 0)」逐字对应；D18 已定项 11 管的是实例表的行格式与「从 1 起、0 无效」（第 47 行注释与之呼应）。**一致。**

**判定：一致。**


### `model.rs` 引的 D28 已定项 1 / 3 / 4（按快照副本核，行号是副本自己的行号）

`model.rs` 里只有两处引 D28（`grep -n 'D28' crates/singlefs-harness/src/model.rs` 命中第 227、1230 行）：

**第 227 行**：`/// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。`——快照副本第 16–22 行（已定项 1 标题与准入不等式）：
```
可用 = Σ设备( 容量 − 已分配 − 不可回收 − defer 待释放 − 挂载期承诺量 − 被抛弃根独占量 ) − 待删占用 − 已承诺预留 − checkpoint 保留池
```
「单元区装不下」对应式子里「可用 ≤ 0」这一类情形，用的是这条准入不等式定的「可用」概念，不是另立算法。**一致。**

**第 1230 行**：`///   保留池 10 + 7 c_max 与切换预留（D16（发布语义） 已定项 1；D28（挂载期承诺量） 已定项 3）在 64 倍里）。预想：D28 已定项 1 的准入式子` ——这一句把「保留池 10+7c_max」记给 D16 已定项 1（停机前已核：`.claude/kb/decisions/16-发布语义.md:376`「保留池 = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max」，B=8），把「切换预留」记给 D28 已定项 3。快照副本第 78–94 行（已定项 3 全文）给出切换预留的权威公式：第 82 行「预留 = (N_switch + 1) × 这个最坏量」，第 84 行给出第一个事务几何下的具体数「每块盘 8 块」，第 88 行加暖机那一半后「每块盘 56 块」。`model.rs` 这一句只是点名「切换预留」这个量出自 D28 已定项 3，不复述具体数字，与副本正文的量纲（按设备算的一个块数）一致，**没有把已定项 3 的公式抄错或抄成另一个数**。一致。

「预想：D28 已定项 1 的准入式子第一版没实现，各项没有现值」——核对：快照副本已定项 1 的准入式子（第 21 行）本身是**存在**的定案（不是空白条款），但它是不等式的**权威形式**，式子里「挂载期承诺量」「被抛弃根独占量」「checkpoint 保留池」三项各自要靠已定项 3/4（切换预留公式、保留池按结构现算）与另一份决策（C318 影子账）另算，且已定项 4 正文明写（副本第 102 行）「E148 的 9 块是模型数……条款里不写常数」——也就是说，除了已定项 3 给出的「第一个事务几何」这个特例数字外，一般情形下这个准入式子**没有一个可以直接拿来当「上限现值」的常数**，与 `model.rs` 「各项没有现值」的说法一致。同时 `capacity_wall_is_permitted`（单元区那一面墙）用的是「沿来路的占槽上界 × 64」这个模型自己拍的倍数，不是把已定项 1 的准入不等式直接实现出来去算「真的可用」——这与「第一版没实现」的说法吻合：`crates/singlefs-core` 里确实没有一处按已定项 1 的九项式子完整计算准入（我在停机前确认过 `crates/singlefs-core`/`crates/singlefs-checker` 零改动，这次额外用 `grep` 复核实现里没有算出「被抛弃根独占量」这类量）：

```
$ grep -rn '被抛弃根独占量\|挂载期承诺量' crates/singlefs-core/src/*.rs
（无命中）
```

`crates/singlefs-core` 源码里确实没有任何地方实现「被抛弃根独占量」或「挂载期承诺量」这两个 D28 已定项 1 式子里的项，印证「第一版没实现，各项没有现值」这句话不是模型自己编的托辞，而是可以现查到的事实。**一致。**

**判定：三处 D28 引用（第 227、1230 两行,共涉及已定项 1、3）与快照副本正文逐一核对，全部一致，没有找到冲突。**（报告第三节表格与第六、七节都没有单独出现「D28 已定项 4」的直接引用——第七节表格里写的是「D16 已定项 1；D28 挂载期承诺量 已定项 3」，`model.rs` 源码里也确认只引了已定项 1 与已定项 3，没有直接引已定项 4；已定项 4 的内容——保留池按结构现算、不设上限——是已定项 1「checkpoint 保留池」那一项与已定项 3 暖机部分共用的背景，间接进了上面已核过的两处引用，不需要单独再核一条「已定项 4 的引用」。）


## 判定一览（更新版，取代前一份「没做什么」里列的未核项）

| 格 | 判定 | 一句话 |
|---|---|---|
| M1：回退候选四条判据（不在环里/txg<F_生效/被抛弃/树表 0 条） | 一致 | 四个 `ModelRefusalReason` 成员逐条对上 D23 已定项 14、D16 已定项 1、D23 已定项 14、「条款没写」三类出处 |
| M1：抬 F 的 `FloorAboveCeiling` 分支 | 一致 | 与 D16 已定项 1「抬 F 的上限」逐字对应 |
| M1：抬 F 的 `SessionState`/`NotModeled` 两个非条款分支 | 一致（不适用条款一致性，属显式承认的空白） | 前者是执行前提检查，后者报告已自陈「模型不答」 |
| M1：挂载写的行与取到的号（D18 已定项 11、D23 已定项 16） | 一致 | 取号公式与写行区间逐字对应决策原文 |
| M1：`model.rs` 第 227 行「D28 已定项 1 的准入」 | 一致 | 对应准入不等式的「可用」概念 |
| M1：`model.rs` 第 1230 行「D28 已定项 3」切换预留 | 一致 | 量纲、来源与已定项 3 公式吻合，未误引已定项 1 的数抄成已定项 3 的数 |
| M1：「预想：D28 已定项 1 的准入式子第一版没实现」 | 一致 | `crates/singlefs-core` 现查无「被抛弃根独占量」「挂载期承诺量」字样，证实确未实现 |
| M6（沿用前一份判定，未变） | 一致（结构与实测）/ 缺口（`zero` 分支无样本） | 见前一份判定一览；本轮未发现新内容需要修改这一判定 |

## 没做什么（更新）

上一份「没做什么」里因快照对不上而搁置的四项（回退四条判据、抬 F 其余分支、挂载写的行与取到的号、`model.rs` 引的 D28 已定项 1/3/4）**已在本节核完**，判定见上表，均为「一致」，不再列入未做事项。

仍未做的：

- 不判 M2、M3、M4、M5（归 Opus 攻方腿）；不判本地攻方的算术格；不替攻方找变异。
- 没有跑 `.claude/gate.d/89-stage-selftest.sh` 全量；没有编译 Rust、没有跑 `check.sh`。
- 没有对 74 号「判绿却没对拍」的输入空间做穷举，只核了脚本读代码得到的两条已知分支与实测跑过的两份样本；`zero` 分支缺样本这一格已记入判定一览，判定为找到的缺口，维持前一份的写法。
- D28 现在（改写之后）那份正文与快照版有实质差异（体例与措辞，判据本身没变，`git diff` 已在前一节贴过），如果以后有引用要对着「现在」的 D28 核，要用改写后的那份，不是这次用的快照副本；这次核的范围严格限定在「本轮判决要对齐的是快照那一刻的条款原文」。
