# 增补 3 第 2 件代码三方第二轮：Sonnet 正推腿（M3、M2「树表 0 条」格）

分到的格：M3（改法与报告对得上）；M2 里「树表 0 条」那一格。不判 M1、M4；不替攻方造历史。

被判对象：`research/prompts/m2-supp3-item2-fix-implementer-report.md`（实现员报告）与工作区里 `crates/` 的实际改动（`git status`/`git diff --stat -- crates` 现查，见下）。第一轮判决 `research/prompts/m2-supp3-item2-code-r1-main-verification.md` 第三节第 1、2、4 条。全部命令原样跑于 2026-09-19（东京时间 JST），工作区当前状态。

## 一、改法是不是第一轮判决第三节第 1、2、4 条说的

r1 判决第三节（`research/prompts/m2-supp3-item2-code-r1-main-verification.md:32-37`）：

> 1. **M2** ⇒ 收窄分配记录墙的区间：墙拒时从镜像上现数这次发布之后的真条数（用 `crates/singlefs-checker` 的解析，不用分配器的状态），真条数 ≤ 812 而实现拒了，算对不上；上界那一端不动。补会红的用例（W1 形态）与变异。
> 2. **M3** ⇒ 给 `RollbackTargetNotACandidate` 加一个判别字段（低于 F_生效 / 被回退行判成被抛弃 / 不在环里 / 树表 0 条），胶水按字段映射，不按文字映射；`NoSpaceFor` 按拒绝原因拆开（容量不够 / 各盘落点不一致 / 小盘写满，与收口表第 20 行 C368 仍欠的那一半一起），胶水只把容量不够那一类映射成单元区墙。补会红的用例（R1 形态、非容量拒绝）与变异。这是 `crates/singlefs-core` 的错误类型改动，调用方的 `match` 跟着改。
> 4. **M1、M4、M5** 没打中，不改。M4 那条设计事实（模型不记落点，落点归 checker）写进 `model.rs` 的模块文档，交实现员一起写。

逐条核代码：

**第 1 条（分配记录墙收窄）**：`crates/singlefs-checker/src/walk.rs` 新增 `allocation_record_count_under_root`（`git diff` 显示 +38 行），`crates/singlefs-harness/src/history.rs` 用它数「模型点名的那一版」的真条数（不读分配器状态），`crates/singlefs-harness/src/model.rs` 的 `capacity_wall_is_permitted` 要求「上界超 且 真条数超」两头都过、真条数取不到就不放行——与判决「上界那一端不动」「真条数 ≤ 812 而实现拒了算对不上」一致。做的是 `crates/singlefs-checker` 的解析（走 `RootView` 的树表 / 分配记录树叶节点），不是分配器状态，符合判决点名的路径。**判：一致**。

**第 2 条（判别字段 + NoSpaceFor 拆分）**：见下方 `crates/singlefs-core/src/mount.rs:122-133` 原文抄录，`RollbackCandidateExclusion` 四个成员 `NotInRing` / `BelowEffectiveFloor` / `OnAbandonedTimeline` / `TargetVersionWithoutFileUnsupported`，与判决列的四条（不在环里 / 低于 F_生效 / 被抛弃 / 树表 0 条）一一对应。`transaction.rs` 的 `PublishError::PlacementRefused { unit, refusal: PlacementRefusal }`（`transaction.rs:930`，`grep -n "PlacementRefused {" crates/singlefs-core/src/transaction.rs` 命中该行）带着分配器的四种原因（现查 `allocator.rs` 定义 `PlacementRefusal` 四个成员：`NoFreeSlotOnAnyDevice`、`SomeDevicesFullDeviceSetSelectionUndefined`、`UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported`、`CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined`）。判决写的是三类（容量不够 / 各盘落点不一致 / 小盘写满），实现给了四个成员——`SomeDevicesFullDeviceSetSelectionUndefined` 与 `UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported` 都算「各盘落点不一致」一类下的两种具体形状（小盘写满时设备集合怎么选未定 vs 落点不同）。胶水 `model_comparison.rs:170-179` 只把 `NoFreeSlotOnAnyDevice` 映射成单元区墙，另外三个都映射成 `Unexplained`（对不上），与判决「胶水只把容量不够那一类映射成单元区墙」一致。**判：一致**（成员数比判决字面的三类多一个是对同一类的进一步细分，不是新增了一类判决没说的拒绝原因；实现员报告第六节第 2 条自己也交代了这一点，见下方「要主 agent 定的事」核实）。

**第 4 条（M4 设计事实写文档）**：`crates/singlefs-harness/src/model.rs` 模块文档（报告称第 14–18 行）现查：

`crates/singlefs-harness/src/model.rs` 第 14 行起（`grep -n "模型不记落点" crates/singlefs-harness/src/model.rs` 命中第 14 行）：「模型不记落点：单元落在哪个槽、分配记录写得对不对（回收门槛、释放时改没改写记录、复用时罩住的槽删没删）归池级 checker 判……分配记录的真条数模型同样不记：分配记录墙拒时，执行器按 checker 的解析从镜像上现数，交给模型判区间的下端」。与判决第三节第 4 条要求的设计事实（模型不记落点，落点归池级 checker 判）逐句对得上。**判：一致**。

## 二、报告里的数与日志、代码对不对得上

### 1. 8 条新测试

逐个 `grep -n "fn <名字>"` 现查（命令与命中行见下），全部存在且行号与报告一致：

| 测试 | 报告称行号 | 现查行号 |
|---|---|---|
| `allocation_record_wall_sampling_without_the_checker_refuses_only_above_one_node_by_the_true_count` | `tests/…random_history.rs:286` | 286（一致） |
| `allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file` | 同文件 :339 | 339（一致） |
| `an_overwrite_that_fills_the_allocation_node_to_exactly_812_records_succeeds_and_the_next_is_refused` | 同文件 :389 | 389（一致） |
| `rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned` | 同文件 :455 | 455（一致） |
| `model::tests::the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812` | `src/model.rs:1998` | 1998（一致） |
| `model::tests::mount_and_raise_count_the_wall_from_the_version_their_admission_starts_from` | `src/model.rs:2040` | 2040（一致） |
| `model_comparison::tests::only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall` | `src/model_comparison.rs:330` | 330（一致） |
| `model_comparison::tests::each_rollback_candidate_exclusion_maps_to_its_own_reason` | `src/model_comparison.rs:376` | 376（一致） |

**判：一致（8/8，行号无一处偏差）。**

### 2. 9 条变异

`wc -l crates/mutations.tsv` = 164；`sed -n '156,164p'` 命中 9 行，报告称「第 156–164 行（九行）」——164 − 156 + 1 = 9，一致。r1 判决落地前（本轮开工快照）该文件应为 155 行（r1 背景材料点名「第 146–155 行」），164 − 155 = 9，与新增行数一致。**判：一致**。

### 3. 各设置判出的段数

命令与原样输出（现查 `research/prompts/m2-supp3-item2-fix-implementer/logs/`）：

- W1、门禁新取样点（`[0,32)×150`，不跑 checker）：`grep -n "同签名的种子" .../proof-w1-1.log` → `同签名的种子 [5, 9, 10, 11, 13, 16, 27, 30, 31]`（9 个），与报告「9 段」一致，与背景材料「9 / 32 段」一致。
- W1、攻方长历史 broad（`[0,96)×200`）：`grep -n "新发现" .../long-w1-broad.log` → `新发现 12`，种子表 `[3, 13, 26, 27, 32, 33, 53, 57, 60, 65, 77, 93]`（12 个），与报告一致。
- W1、攻方长历史 reuse：`long-w1-reuse.log` → `新发现 33`，种子表数出 33 个，与报告一致。
- base（改法在，无 W1）：`long-base-broad.log` / `long-base-reuse.log` → `每个新发现的种子：[]`（均为 0），与报告一致。
- R1、回退取样点（`[0,48)×30`，跑 checker）：`grep -n "历史 48 段" .../proof-r1-1.log` → `历史 48 段：跑完 30、以已知红收尾 {0: 6}、新发现 12`，种子表 12 个，与报告「12 / 48 段」一致。
- R1、快档（`[0,96)×30`，跑 checker）：`grep -n "历史 96 段" .../proof-r1-1.log`（同一文件，快档那一节）→ `历史 96 段：跑完 43、以已知红收尾 {0: 48, 1: 1}、新发现 4`，种子表 `[8, 9, 29, 61]`（4 个），与报告「4 / 96 段」一致。

**判：一致（六组数全部与日志逐字对上）。**

背景材料（正文第一节）把 W1 长历史两组数写成「12 / 33 段」——不是报告写的「12 段（broad）」与「33 段（reuse）」两个独立计数各自 /96 的意思，而是把两个不同设置各自的新发现段数并排写成一个形似分数的短语。数值本身（12、33）与日志一致，但这行短语不带「各自 /96」，单看这一行容易误读成「12 段 / 33 段总量」的一个分数；不算错，是背景材料自己的记法歧义，不算实现员报告的错，列在这里备查。

### 4. 三段计数改前改后相同

`research/prompts/m2-supp3-item2-fix-implementer/logs/gate-74.log` 原样（改法落地之后、主 agent 跑门禁 74 号）：

```
  ✓ 模型对拍三段都判过、实现与模型没有对不上的（查了 3 段）：
      随机历史快档：模型对拍 1912 步：该拒而拒 572、区间里拒 34、该成而成 1306；……
      随机历史：偏向抬 F 之后复用的取样点：模型对拍 1183 步：该拒而拒 242、区间里拒 37、该成而成 904；……
      随机历史：偏向抬 F 之后回退的取样点：模型对拍 913 步：该拒而拒 400、区间里拒 28、该成而成 485；……
```

「改之前」的独立来源是 r1 的攻方腿产物（不是实现员自己另跑的一份 before 基线）：`research/prompts/m2-supp3-item2-code-r1-opus-output.md:5`（`grep -n "快档 1912 步" research/prompts/m2-supp3-item2-code-r1-opus-output.md` 命中第 5 行）逐字「不设时基线三段与实现员报告逐数相同（快档 1912 步 572 / 34 / 1306，第 121 行取样点 1183 步，回退取样点 913 步；`test result: ok. 9 passed`）」；同文件第 39 行「基线三段报的『区间里拒 34 / 37 / 28』全是抬 F 的『允许拒』那一格」。三段的步数（1912、1183、913）与「区间里拒」（34、37、28）逐数与 `gate-74.log` 相同；「该拒而拒」「该成而成」两项 r1 攻方腿只给了快档一段的分解（572 / 34 / 1306），复用取样点、回退取样点两段没有给出对应的分解数，**这两段的『该拒而拒 / 该成而成』改前是否逐数相同，本轮没有独立来源可核**（下同见「没做什么」）。

**判：部分一致——三段的步数与「区间里拒」三项全部核对一致；复用与回退两段的「该拒而拒/该成而成」缺改前独立来源，不算否证，也不算核过。**

## 三、调用方的 `match`：`RollbackTargetNotACandidate`、`RollbackCandidateExclusion`、`PublishError::PlacementRefused`、`PlacementRefusal` 有没有 `_ =>`

`grep -rn "_ =>" crates/singlefs-core/src/mount.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocator.rs crates/singlefs-harness/src/model_comparison.rs crates/singlefs-harness/src/history.rs crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs` 全仓（这十个文件，即所有引用这两个类型的 `.rs` 文件，由先跑 `grep -rln "RollbackTargetNotACandidate\|RollbackCandidateExclusion\|PlacementRefused\|PlacementRefusal"` 现查得到的清单）命中恰好一处：

```
crates/singlefs-harness/src/history.rs:519:                _ => RollbackTargetChoice::BeyondNewestRoot {
```

读上下文（`history.rs:512-522`）：这个 `match` 匹配的是 `source.below(5)`（造历史时随机抽一个 0–4 的整数决定回退目标怎么选），不是 `RollbackCandidateExclusion` 或 `PlacementRefusal` 的判别，与本轮判决要求穷举的那两个类型无关。

逐一核 `model_comparison.rs`（`refusal_reason_of_placement_refusal`、`refusal_reason_of_rollback_candidate_exclusion`、`refusal_reason_of_mount_error`、`reported_ceiling_of_mount_error`）与 `history.rs`（`placement_refusal_member`、`mount_error_member`）里对这两个类型（以及外层 `MountError`/`PublishError`）的全部 `match`：均按具名成员逐一列出或用 `|` 合并同类分支，没有一处落到通配臂——见附录（第五节）逐段抄录。

**判：一致。全仓命中的唯一 `_ =>` 与这两个错误类型无关。**

## 四、kb 与 `crates/` 注释里还写着旧成员名、要跟着改的每一处

先复核背景材料与实现员报告都点名的现查命令。**问题**：正文第一节那句行内命令写的是 `.claude/kb/**/*.md`，这个仓的默认 shell（`shopt globstar` = `off`，现查见下）下 `**` 只等价于 `*`（不递归），于是 `.claude/kb/**/*.md` 只展开成「kb 下恰好两层」的文件（`.claude/kb/<目录>/<文件>.md`），既不包含 `.claude/kb/checks-owed.md`（只一层）也不包含更深的路径。但背景材料与实现员报告都把 `checks-owed.md` 的命中列了进去——说明他们实际现查时用的是 `globstar` 打开之后的展开（`**` 递归匹配零层或多层目录），而不是字面在这个仓默认 shell 下跑出的结果。

```
$ shopt globstar
globstar       	off
```

按 `globstar` 打开之后重新展开、再 grep 一遍（覆盖到 `.claude/kb/` 全树）：

```
$ shopt -s globstar
$ grep -rn "RollbackTargetNotInRing\|RollbackToVersionWithoutFileUnsupported\|NoSpaceFor" .claude/kb/**/*.md
.claude/kb/checks-owed.md:335: ...（与背景材料、报告列的一致）
.claude/kb/milestone/02-second-txn.md:117,141,166,329,330 ...（与背景材料、报告列的一致）
.claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:8: ...（背景材料与实现员报告都没有列）
```

第三处，`.claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:8`（`grep -n "RollbackTargetNotInRing" ".claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md"` 命中第 8 行）：「……③ 回退不查目标还在不在环里（实现报 `RollbackTargetNotInRing`），H6 有 60 格受影响。」这一句是 2026-09-17 一次只读核查（E154）发现的模型 bug 记录，日期与描述都锚在那次核查（比 r1 的改动早两天），读法按 kb-discipline「带日期的现状快照也是历史」一节的判据（把旧值换成新值，这句话还是不是真话）：这里说的不是「今天 `RollbackTargetNotInRing` 这个标识符还存在」，而是「那次核查时，实现报的是这个名字」——事件句，不是现状句，换成今天的嵌套名不会让这句话更真，也不会更假，因为它陈述的是过去某次观测。**结论：这一处不必跟着改**，但背景材料与实现员报告的「kb 里还写着旧成员名的」清单本身**不完整**——两处都漏列了这一处，只是漏列的这一处经核不需要改动。

`crates/` 里另有三处历史性注释含旧名字，均已在报告或正文里被排除，现查均属实：

- `transaction.rs:928`「不是容量不够。此前这四种一律报成 `NoSpaceFor`」——「此前」明示历史，不需要改。
- `transaction.rs:1232`「落点被拒（当时叫 `NoSpaceFor`）」——「当时」明示历史，不需要改。
- `model_comparison.rs:168`、`328`「此前 `NoSpaceFor` 装着这四种」——同上。
- `second_transaction_supplement_two_commit_generated_fallback.rs:215`「补回落之前这里报 `NoSpaceFor`」——报告称「说的是补回落之前的事」，现查上下文（`crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs:214-216`）确认这条测试注释描述的是「补回落」这个此前的修复动作生效之前的旧行为，「今天抬 F 成功」在下一句里，一致。

`crates/` 里 `ModelRefusalReason::RollbackTargetNotInRing`、`ModelRefusalReason::RollbackToVersionWithoutFileUnsupported`（`model.rs`、`model_comparison.rs` 多处）不算：这是模型自己的理由枚举成员名，背景材料第一节末段已经点明「那是模型的理由、不是 core 的成员」，现查这些成员确实定义在 `model.rs` 的 `ModelRefusalReason` 里、与 `crates/singlefs-core` 的 `RollbackCandidateExclusion`/`MountError` 是两个独立的类型，一致。

**判：报告的清单本身与代码核对一致（117/141/166/329/330 五行 + checks-owed.md 一行）；但清单遗漏了 `experiments/154-…md:8` 一处含旧名字的句子——不需要改（事件句），遗漏本身仍是这份清单的一个缺口。**

## 五、附录：`_ =>` 核查涉及的全部 `match` 原文（逐段抄录，第三节引用）

`crates/singlefs-harness/src/model_comparison.rs:170-179`（`refusal_reason_of_placement_refusal`）：

```rust
pub fn refusal_reason_of_placement_refusal(refusal: &PlacementRefusal) -> ObservedRefusalReason {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => explained(ModelRefusalReason::UnitAreaWall),
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. }
        | PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. }
        | PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        } => ObservedRefusalReason::Unexplained,
    }
}
```

`crates/singlefs-harness/src/model_comparison.rs:183-198`（`refusal_reason_of_rollback_candidate_exclusion`）：

```rust
pub fn refusal_reason_of_rollback_candidate_exclusion(
    exclusion: RollbackCandidateExclusion,
) -> ObservedRefusalReason {
    explained(match exclusion {
        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetNotInRing,
        RollbackCandidateExclusion::BelowEffectiveFloor => {
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor
        }
        RollbackCandidateExclusion::OnAbandonedTimeline => {
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline
        }
        RollbackCandidateExclusion::TargetVersionWithoutFileUnsupported => {
            ModelRefusalReason::RollbackToVersionWithoutFileUnsupported
        }
    })
}
```

`crates/singlefs-harness/src/history.rs:1574-1587`（`placement_refusal_member`）：

```rust
fn placement_refusal_member(refusal: &PlacementRefusal) -> &'static str {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => "NoFreeSlotOnAnyDevice",
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. } => {
            "SomeDevicesFullDeviceSetSelectionUndefined"
        }
        PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. } => {
            "UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported"
        }
        PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        } => "CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined",
    }
}
```

以上三处都对涉及的枚举逐一穷举成员（用 `|` 合并同宗多个成员算一支，不是通配臂）。`model_comparison.rs:141-164`（`refusal_reason_of_publish_error`）、`:209-243`（`refusal_reason_of_mount_error`）、`:249-266`（`reported_ceiling_of_mount_error`）与 `history.rs:1589-1615`（`publish_error_member`）、`:1647-` 起（`mount_error_member`）同样逐一穷举 `PublishError`/`MountError` 的全部成员（含 `RollbackTargetNotACandidate { .. }`、`PlacementRefused { .. }` 各一支），均无 `_ =>`，第一次跑 `grep -n "_ =>"` 时已确认整仓命中只有 `history.rs:519` 一处、与本题无关（第三节已引）。

## 六、M2「树表 0 条」那一格：按条款原文，它是候选排除，还是「在候选集里、第一版不支持」

**条款原文，逐字抄**：

- D16（发布语义） 已定项 1，`.claude/kb/decisions/16-发布语义.md:376`：「回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |」
- D23（journal 的角色与格式） 已定项 14，`.claude/kb/decisions/23-journal的角色与格式.md:1209`（该段开头）：「候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）」

两处对候选集的定义都只有两个条件：按实例表判仍然有效、且 txg ≥ F_生效。**没有第三个条件提到树表条目数或有没有发布过文件版本。** 按条款字面，一个树表 0 条（还没发布过文件版本）但满足这两个条件的根，逐字读就在候选集里。

代码自己的注释也这么写：`crates/singlefs-core/src/mount.rs:130-131`（`grep -n "按前三条它可以在候选集里" crates/singlefs-core/src/mount.rs` 命中第 130 行）：「目标那一版树表 0 条（还没发布过文件版本）：**按前三条它可以在候选集里**，但回退行要重写实例表，没有文件版本的一版上它的落点记在哪没有条款（D16（发布语义） 已定项 9 只定了树表 0 条时空发布写零个单元），第一版不支持。」——「按前三条它可以在候选集里」这半句，是实现者自己写下的、与条款原文一致的结论。

**判：按条款原文，树表 0 条的根不是候选排除，是「在候选集里、第一版不支持」**——它被拒绝的原因不是不满足候选集定义，是「回退行要重写实例表、没有文件版本的一版上落点记在哪没有条款」这个另外的、实现层面的空白（D16 已定项 9 只定了空发布写零个单元，没定回退行怎么落）。

但代码把它塞进了 `RollbackCandidateExclusion`（这个类型名字面意思是「候选排除」）的第四个成员 `TargetVersionWithoutFileUnsupported`，与前三个真正的候选集排除条件（`NotInRing`、`BelowEffectiveFloor`、`OnAbandonedTimeline`）并列在同一个枚举里、由同一个 `MountError::RollbackTargetNotACandidate` 报出。这与条款原文的读法之间有一处名实不符：`RollbackCandidateExclusion` 这个类型名断言「凡在这个枚举里的都是候选集排除」，而它的第四个成员按条款原文和代码注释自己的话，**不是**候选集排除，是「在候选集里但第一版不支持」。

**这不是这一轮实现员的自选**：r1 判决第三节第 2 条原文（`research/prompts/m2-supp3-item2-code-r1-main-verification.md:35`）逐字写的就是「给 `RollbackTargetNotACandidate` 加一个判别字段（低于 F_生效 / 被回退行判成被抛弃 / 不在环里 / 树表 0 条）」——r1 判决本身已经把「树表 0 条」放进了 `RollbackTargetNotACandidate` 底下。实现员照办，并在报告第六节第 1 条把这处名实不符原样交回主 agent 定（「『树表 0 条』放进了 `RollbackTargetNotACandidate` 的字段，而它按 D16（发布语义） 已定项 1 可以在候选集里、只是第一版不支持」）。

**推翻条件**：若能在 D16 已定项 1 或 D23 已定项 14 里找到第三个显式排除条件（提到树表条目数、有没有发布过文件版本），这一格判「一致」；今天两处条款原文都只有两个条件，找不到这样的第三条件。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| M3-1 改法与判决第 1 条 | 一致 | 分配记录墙收窄按 checker 解析现数真条数，上界不动，与判决逐句对上 |
| M3-2 改法与判决第 2 条 | 一致 | 判别字段四值对应判决四条；`PlacementRefusal` 四成员是判决三类拒绝原因里「各盘落点不一致」再细分两种，不是多出一类 |
| M3-3 改法与判决第 4 条 | 一致 | `model.rs:14` 起的模块文档逐句对上判决要求写的设计事实 |
| M3-4 8 条新测试 | 一致 | 8 个函数名与行号逐一 `grep -n` 命中，与报告一致 |
| M3-5 9 条变异 | 一致 | `mutations.tsv` 156–164 共 9 行，与本轮开工前 155 行的差恰为 9 |
| M3-6 各设置判出的段数 | 一致 | W1（9/32、12/96 broad、33/96 reuse、base 两组 0）、R1（12/48、4/96）六组数逐字与日志对上 |
| M3-7 三段计数改前改后相同 | 部分一致 | 三段步数与「区间里拒」三项与 r1 攻方腿基线逐字相同；复用、回退两段「该拒而拒/该成而成」缺改前独立来源，未核 |
| M3-8 `match` 无 `_ =>` | 一致 | 全仓十个引用文件里唯一的 `_ =>`（history.rs:519）与这两个类型无关，其余全部具名穷举 |
| M3-9 kb/crates 旧成员名清单 | 部分一致 | 报告点名的 6 行准确；清单本身漏列 `experiments/154-…md:8`（事件句，不需要改，但清单不完整） |
| M2「树表 0 条」候选排除还是「候选集里、第一版不支持」 | 冲突（后者对） | D16 已定项 1、D23 已定项 14 的候选集定义只有两个条件，没有树表条目数这一条；代码注释自己也写「按前三条它可以在候选集里」，却被塞进名为「候选排除」的枚举——这处名实不符来自 r1 判决本身的措辞，实现员已原样交回主 agent |

## 八、没做什么

- 不判 M1（墙的误红漏判的算术）、M4（跳过 checker 那一段的攻击面），按分工归 Opus 攻方与本地攻方。
- 不替攻方造历史、不跑变异证明、不跑门禁 59 号整表；这些数全部转引自 `research/prompts/m2-supp3-item2-fix-implementer/logs/` 与 `research/prompts/m2-supp3-item2-code-r1-opus-output.md` 的原样内容，本轮只复核、不重跑。
- 复用取样点、回退取样点两段「该拒而拒/该成而成」改前基线没有独立来源可核（r1 攻方腿只给了快档一段的分解），列在第二节第 4 条，不算否证。
- 未对 `crates/singlefs-core/src/allocator.rs` 的 `.ok()` 丢原因改动之外的分配器内部逻辑做正确性判断（不在 M3 范围内，M1/M2 归攻方）。
- 不出判决、不采纳自己的结论，交主 agent 核实汇总。
