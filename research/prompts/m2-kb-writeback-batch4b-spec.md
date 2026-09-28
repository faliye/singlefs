# kb 写回规格（第四批乙：里程碑二收尾）

每条：文件、旧串、新串、依据。同一文件各条旧串互不相交，按次序施加。

## 条 1

文件：`.claude/kb/decisions/23-journal的角色与格式.md`

旧串：
```
- 用户定案 2026-09-25：点名单元两份都验过才施加、认下丢一整块盘时丢掉刚确认的那一版（C519（丢一整块盘时恢复丢掉刚确认的那一版） 实一复现：`crates/singlefs-harness/tests/second_transaction_supplement_two_c519_whole_device_loss_after_warm_up.rs`），原话在变更史；无实验：照今天的实现写成条款，另一读法「任一份验过」会不会放过 E77（发布的持久顺序） 那种嫁接没量。
```

新串：
```
- 用户定案 2026-09-25：点名单元两份都验过才施加、认下丢一整块盘时丢掉刚确认的那一版（C519（丢一整块盘时恢复丢掉刚确认的那一版） 实一复现：`crates/singlefs-harness/tests/second_transaction_supplement_two_c519_whole_device_loss_after_warm_up.rs`），原话在变更史；无实验：照今天的实现写成条款，另一读法「任一份验过」会不会放过 E77（发布的持久顺序） 那种嫁接没量。
- 用户定案 2026-09-27（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，先选乙、再补选「配置续（推荐）」）：可写挂载读到系统配置见证过的更新状态读不出，重读一次仍读不出就拒可写；取号那一写不再写 tail = 0，改写取号那一刻按同一取法读到的见证值，实现在做（实现员 2026-09-27 派出）；判据 N-配置续 定义在 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行，原话在变更史。E158（择根与修复四岔路） 实验页还没写这一段，依据先指第 4 次跑第二段（`research/prompts/e158-r4-seg2-runner-report.md` 第四、五节）与第三段 H1g（`research/prompts/e158-r4-seg3-runner-report.md` 第 9 行）两份报告，门禁 75 号那一对等实验页写好再补，这一格记欠。
```

依据：主 agent 消息（用户 2026-09-27 弹窗）；research/prompts/e158-r3-prereg.md 第 348 行与 research/prompts/e158-r4-prereg.md 第 413 行；e158-r4-seg2/seg3-runner-report.md

## 条 2

文件：`.claude/kb/decisions/23-journal的角色与格式.md`

旧串：
```
- **语义**：这个数的含义是**一次恢复重放的前缀最多 N 条**。读到一条校验和不过的记录时，恢复**断链即止**——停在它之前那一条，不追问它是「撕裂」还是「损坏」，也不因此拒绝挂载。不是 XFS 那种「最后 N 条内的坏校验和算撕裂、之外算损坏拒绝挂载」。
```

新串：
```
- **语义**：这个数的含义是**一次恢复重放的前缀最多 N 条**。读到一条校验和不过的记录时，恢复**断链即止**——停在它之前那一条，不追问它是「撕裂」还是「损坏」，也不因此拒绝只读挂载；可写挂载另按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续判（已定项 14）：系统配置见证过比断链处那一版新的发布、重读一次仍判真，拒可写。不是 XFS 那种「最后 N 条内的坏校验和算撕裂、之外算损坏拒绝挂载」。
```

依据：research/prompts/m2-impl-c554-yi-implementer-report.md 第 98–104 行

## 条 3

文件：`.claude/kb/decisions/16-发布语义.md`

旧串：
```
| 根槽这一次读坏 | 算准入抬 F 的上限时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根；挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F、报它自己的错误成员，不按有根或没根猜（用户 2026-09-26 定，推的，被攻过零轮）。读根环的别的调用方（择根、回退候选集、「环里最旧有效根」）怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判） |
```

新串：
```
| 根槽这一次读坏 | 算准入抬 F 的上限时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根；挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F、报它自己的错误成员，不按有根或没根猜（用户 2026-09-26 定，推的，被攻过零轮）。可写挂载的择根按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、回退候选集、「环里最旧有效根」怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判） |
```

依据：research/prompts/m2-impl-c554-yi-implementer-report.md 第 108–114 行

## 条 4

文件：`.claude/kb/checks-owed.md`

旧串：
```
| C554 | 崩溃恢复抛弃的根暂时读不出时影子账算不到 | 被抛弃实例的根在重开时全部暂时读不出 ⇒ 影子账读不到它们的账、隔离 0；新实例复用了它们的槽；撤故障后再把新实例的根全改坏，恢复落在被抛弃实例的根上读不出。影子账开 / 关两臂逐字相同；与管理员回退无关（管理员回退不抛弃根），是多重故障下的一格（D23（journal 的角色与格式） 已定项 14 射程「挂着时回退的已知边角」②）。前半段今天的行为有用例钉着：崩溃恢复造出的被抛弃根 C 的根槽每次读都失败时，它独占的 14 个槽全掉出隔离集，`abandoned_roots_unreadable` 仍是 0（`crates/singlefs-harness/tests/second_transaction_supplement_two_unreadable_abandoned_root_slot.rs` 的 `an_abandoned_root_whose_own_root_slot_is_unreadable_is_neither_isolated_nor_counted`）；那条用例钉的是今天的样子，不是该有的样子。E158（择根与修复四岔路） 第 3 次跑第一段查到这一形丢写的机理：撤故障之后新实例写行那次发布取了与被藏根相同的 txg，根槽位置只由 txg 定，写行那次发布把被藏根的根槽盖掉了（阳性对照 PC-N 在今天那一臂上 `hidden_in_ring_after=false`、被抛弃集合为空）。代码审阅第 22 条并进这一条：重建影子账时第二次读实例表失败就无声关掉隔离——`crates/singlefs-core/src/mount.rs` 里先 `choose_root(devices, system_configuration)`、再对选中的根 `instance_table_of_root(devices, &newest)` 读实例表，读不出（交 None）时每条根都判成没被抛弃、一个槽不隔离，`abandoned_roots_unreadable` 仍是 0，而前面 `rebuild_previous_version` 已经读出过这张表。checker 那一侧：I-7.4（近 K 代块未被复用） 被抛弃根那一半 2026-09-27 起照实判（用户定，代码审阅第 7 条），这一形在池级 checker 上照实红，等这一条修好转绿 | 多次挂载的崩溃点重放里造这段历史（被抛弃实例的根重开时全部暂时读不出 → 新实例复用 → 撤故障 → 新实例的根全坏），断言恢复落到的根引用的单元没被复用，或挂载报错；判别力自证：今天的代码必须红。**验收**（乙改完之后要绿的三条）：调查员最短复现（`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED=7463871032432355115` 跑 `shrink_one_failing_seed_from_the_environment`，3 步，`research/prompts/m2-investigate-gate74-reds-report.md`）不再判红；`second_transaction_step_three_formatted_pool.rs` 里那条 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 用例改判「只红 I-7.4（近 K 代块未被复用）」；`second_transaction_supplement_two_unreadable_abandoned_root_slot.rs` 的 `an_abandoned_root_whose_own_root_slot_is_unreadable_is_neither_isolated_nor_counted` 改名、按新行为钉（暂时读错、重读成功时择对根、不红；R 轮之后仍读不出则拒可写、只读照常） | 用户已定（2026-09-27 弹窗答复）：修法选乙 重读后再判，重读次数 R = 1（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行最后一格：取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」；判「有一条更新的根读不出」用系统配置记的位置）；同一判据与重读要罩住代码审阅第 22 条那一格（重建影子账时第二次读实例表失败）。频率：门禁 74 号随机历史快档 96 段里 28 段是这一形，最短复现 3 步（`research/prompts/m2-investigate-gate74-reds-report.md`）。按 (区域, 槽) 让根槽读返回失败的开关已有（`singlefs_harness::fault_injection` 的 `RootRingSlotTarget` 与 `FaultSchedule::every_read_of_named_root_ring_slots_fails`，钉着今天行为的那条用例就用它）。实现在 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙手上。 | 2026-09-25 回退改形态第一轮三方攻方（`research/prompts/m2-rollback-forward-r1-main-verification.md`「越格：崩溃恢复抛弃的根暂时读不出时，影子账算不到它们」一节）；实八 2026-09-27 在崩溃注入种子基 + 2 的诊断里另见一形：(1, 4) 那次发布的根还在根环里、被抛弃时读不出、影子账看不到它、它的单元已被复用（`research/prompts/m2-impl8-implementer-report.md`「交你定的」第 3 条），实现员推与这一条同根源、没核，改法要连这一格一起看；E158（择根与修复四岔路） 第 3 次跑第一段执行员报告 `research/prompts/e158-r3-runner-seg1-report.md`「一、结论」两处阳性对照那一条第 1 点；代码审阅第 22 条（`research/prompts/m2-code-review-6c/00-handover-message.md` 第 47 行） |
```

新串：
```
| C554 | 崩溃恢复抛弃的根暂时读不出时影子账算不到 | 被抛弃实例的根在重开时全部暂时读不出 ⇒ 影子账读不到它们的账、隔离 0；新实例复用了它们的槽；撤故障后再把新实例的根全改坏，恢复落在被抛弃实例的根上读不出。影子账开 / 关两臂逐字相同；与管理员回退无关（管理员回退不抛弃根），是多重故障下的一格（D23（journal 的角色与格式） 已定项 14 射程「挂着时回退的已知边角」②）。前半段今天的行为有用例钉着：崩溃恢复造出的被抛弃根 C 的根槽每次读都失败时，它独占的 14 个槽全掉出隔离集，`abandoned_roots_unreadable` 仍是 0（`crates/singlefs-harness/tests/second_transaction_supplement_two_unreadable_abandoned_root_slot.rs` 的 `an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable`）；那条用例钉的是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续实现之后的样子：系统配置没见证到的最新根被抛弃时仍不隔离、不计数，见证过的那一形已被挡住（记欠见 Q1，已定续、实现在做）。E158（择根与修复四岔路） 第 3 次跑第一段查到这一形丢写的机理：撤故障之后新实例写行那次发布取了与被藏根相同的 txg，根槽位置只由 txg 定，写行那次发布把被藏根的根槽盖掉了（阳性对照 PC-N 在今天那一臂上 `hidden_in_ring_after=false`、被抛弃集合为空）。代码审阅第 22 条并进这一条：重建影子账时第二次读实例表失败就无声关掉隔离——`crates/singlefs-core/src/mount.rs` 里先 `choose_root(devices, system_configuration)`、再对选中的根 `instance_table_of_root(devices, &newest)` 读实例表，读不出（交 None）时每条根都判成没被抛弃、一个槽不隔离，`abandoned_roots_unreadable` 仍是 0，而前面 `rebuild_previous_version` 已经读出过这张表。checker 那一侧：I-7.4（近 K 代块未被复用） 被抛弃根那一半 2026-09-27 起照实判（用户定，代码审阅第 7 条），这一形在池级 checker 上照实红，等这一条修好转绿 | 多次挂载的崩溃点重放里造这段历史（被抛弃实例的根重开时全部暂时读不出 → 新实例复用 → 撤故障 → 新实例的根全坏），断言恢复落到的根引用的单元没被复用，或挂载报错；判别力自证：今天的代码必须红。**验收**（乙改完之后要绿的三条）：调查员最短复现（`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED=7463871032432355115` 跑 `shrink_one_failing_seed_from_the_environment`，3 步，`research/prompts/m2-investigate-gate74-reds-report.md`）不再判红；`second_transaction_step_three_formatted_pool.rs` 里那条 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 用例改判「只红 I-7.4（近 K 代块未被复用）」；`second_transaction_supplement_two_unreadable_abandoned_root_slot.rs` 的 `an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable`（乙-配置续实现时已改名、按新行为钉：暂时读错、重读成功时择对根、不红；R 轮之后仍读不出则拒可写、只读照常） | 用户已定（2026-09-27 弹窗答复）：修法选乙 重读后再判，重读次数 R = 1（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行最后一格：取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」；判「有一条更新的根读不出」用系统配置记的位置）；同一判据与重读要罩住代码审阅第 22 条那一格（重建影子账时第二次读实例表失败）。频率：门禁 74 号随机历史快档 96 段里 28 段是这一形，最短复现 3 步（`research/prompts/m2-investigate-gate74-reds-report.md`）。按 (区域, 槽) 让根槽读返回失败的开关已有（`singlefs_harness::fault_injection` 的 `RootRingSlotTarget` 与 `FaultSchedule::every_read_of_named_root_ring_slots_fails`，钉着今天行为的那条用例就用它）。乙-配置续已实现挂载读阶段那一半（`crates/singlefs-core/src/mount.rs` 重读一次仍读不出拒可写，`research/prompts/m2-impl-c554-yi-implementer-report.md`）；代码审阅第 22 条那一格已罩住；取号那一写改写见证值那一半待派（等 A3a 交回再派，记欠见 Q1）；模型跟上乙在做，记欠见 Q2。 | 2026-09-25 回退改形态第一轮三方攻方（`research/prompts/m2-rollback-forward-r1-main-verification.md`「越格：崩溃恢复抛弃的根暂时读不出时，影子账算不到它们」一节）；实八 2026-09-27 在崩溃注入种子基 + 2 的诊断里另见一形：(1, 4) 那次发布的根还在根环里、被抛弃时读不出、影子账看不到它、它的单元已被复用（`research/prompts/m2-impl8-implementer-report.md`「交你定的」第 3 条），实现员推与这一条同根源、没核，改法要连这一格一起看；E158（择根与修复四岔路） 第 3 次跑第一段执行员报告 `research/prompts/e158-r3-runner-seg1-report.md`「一、结论」两处阳性对照那一条第 1 点；代码审阅第 22 条（`research/prompts/m2-code-review-6c/00-handover-message.md` 第 47 行） |
```

依据：research/prompts/m2-impl-c554-yi-implementer-report.md 第 65–68、76–119 行；主 agent 消息（乙-配置续）

## 条 5

文件：`.claude/kb/decisions-history/2026-09.md`

旧串：
```
## 历史版本

### 2026-09-27（其七）：D28（挂载期承诺量） 已定项 4：分配记录树那一项的式子补根层级读法与代价数，欠账收口
```

新串：
```
## 历史版本

### 2026-09-27（其八）：D23（journal 的角色与格式） 已定项 14、D16（发布语义） 已定项 1：C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续——系统配置见证过的更新状态读不出，重读一次仍读不出拒可写

> 快查·改前：影子账与按实例表判抛弃留着，理由是崩溃恢复；D16（发布语义） 已定项 1「根槽这一次读坏」一行没写可写挂载的择根怎么处置系统配置见证过的更新状态读不出；D23（journal 的角色与格式） 已定项 18「不因此拒绝挂载」不分只读与可写。
>
> 快查·改后：读阶段判出系统配置见证过比所选那一版新的发布读不出（N-配置续）就重读一次，仍读不出拒可写、盘上不变，只读挂载照常；取号那一写改写见证值而不是 0（实现待派）；影子账留着的理由收窄成系统配置没见证到的那一形。

- 改前：D23（journal 的角色与格式） 已定项 14 的射程只写崩溃恢复因暂时读错落到旧根会抛弃较新的根、留影子账兜底（第一轮 H6），没有判据挡住「系统配置见证过、暂时读不出」这一形；D16（发布语义） 已定项 1「根槽这一次读坏」一行只管抬 F 的上限，没写可写挂载的择根；D23（journal 的角色与格式） 已定项 18「语义」写「也不因此拒绝挂载」，不分只读与可写；D16（发布语义） 已定项 1 表里同一行没提可写挂载的择根。
- 改后：D23（journal 的角色与格式） 已定项 14 补一段判据（N-配置续）：读阶段读完取全部自证过的系统配置槽 journal tail 最大值 c_见证，判出「有更新的发布系统配置见证过而读不出」就在读缓存上重做一遍读阶段（R = 1），仍为真就在取号之前拒可写挂载，只读挂载照常；重建分配器判抛弃时同样重读一次（代码审阅第 22 条）；取号那一写（`transaction.rs` 的 `write_acquired_instance`）改写见证值 c_见证，不再写 0（实现在做，实现员 2026-09-27 派出）；已定项 14 两处已知边角与已定项 18「语义」各补一句收窄影子账与拒绝挂载的射程；D16（发布语义） 已定项 1 表里那一行补可写挂载的择根按这条判。
- 依据：实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「七、条款」一节）；判据 N-配置续 定义在 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行（E158（择根与修复四岔路） 实验页还没写这一段，依据先指第 4 次跑第二、三段两份报告，门禁 75 号那一对等实验页补上再登记）；用户定案 2026-09-27（选乙）与」），原话在 `records/2026-09-24-里程碑二收尾调度.md`「用户定乙-配置续」那一行。

### 2026-09-27（其九）：D18（块里携带什么信息） 已定项 11：逐盘「不可见」核的措辞收窄成只核「可见」那一支，Z3-A 乙已实现

> 快查·改前：挂着之后收盘表的每个入口「同样逐盘核」，读法有歧义；「实现随「Z3-A 乙」那一件（还没派，写「实现待派」）」；没写三类入口各自的拒绝成员名；没写「会话每次发布」按什么颗粒度核。
>
> 快查·改后：改成只核「可见」那一支（不核落后那一支）；已实现（`research/prompts/m2-impl-z3a-yi-implementer-report.md`），三类入口的拒绝成员名补全；「会话每次发布」按「每次用户改动」读，循环里重发之前紧挨着的是那一串抬 F 自己的核。

- 改前：已定项 11 第 314 行末句只写「挂着之后收盘表的每个入口……在任何写之前同样逐盘核……实现随「Z3-A 乙」那一件（还没派，写「实现待派」）」，「同样逐盘核」读起来可以是两支（不可见、落后）都核，没写拒绝成员名，没写「会话每次发布」核的颗粒度。
- 改后：实审 Z3-A 乙落地后按代码逐项对过，改写成：只核「可见」那一支（不核落后那一支，规格表第 1 行只要这一支）；已实现，四类入口（正常卸载、抬 F 报 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`；管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`；会话报 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`）各自的拒绝成员名补全；「会话每次发布」按「每次用户改动」读（一次调用只核一次，在循环之前），循环里为空间不够推的每一串抬 F 自己经同一道核再核一次，紧挨着重发之前——这是推的，没有用例在循环中途把盘换掉。
- 依据：实 Z3-A 乙报告（`research/prompts/m2-impl-z3a-yi-implementer-report.md`「三、条款与代码对照」一节逐项对照表）。

### 2026-09-27（其七）：D28（挂载期承诺量） 已定项 4：分配记录树那一项的式子补根层级读法与代价数，欠账收口
```

依据：research/prompts/m2-impl-c554-yi-implementer-report.md；research/prompts/m2-impl-z3a-yi-implementer-report.md

## 条 6

文件：`.claude/kb/checks-owed.md`

旧串：
```
| C576 | 三条用例被准入先拒连带红，改法待选 | `devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written`、`user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written` 等用例在准入先拒之后结局或数值可能改变；改法两条路（装只供测试的开关跳过准入、或改钉成 `SpaceAdmissionRefused`）与它们点名的第 12、76、77、78、80、145、149 行变异要不要换靶子没定 | 选定改法之后按选定的结局改钉这几条用例，变异表按新靶子核一遍；判别力自证：换回另一条改法，钉住的结局必须由绿转红 | 改法待主 agent 选 | 2026-09-27 A4d 报告（`research/prompts/m2-rev-a4d-implementer-report.md`「七、交主 agent 的设计问题」第 4 条；用例清单见报告第六节第 1、2 条） |
```

新串：
```
| C576 | 三条用例被准入先拒连带红，改法待选 | `devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written`、`user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written` 等用例在准入先拒之后结局或数值可能改变；改法两条路（装只供测试的开关跳过准入、或改钉成 `SpaceAdmissionRefused`）与它们点名的第 12、76、77、78、80、145、149 行变异要不要换靶子没定 | 选定改法之后按选定的结局改钉这几条用例，变异表按新靶子核一遍；判别力自证：换回另一条改法，钉住的结局必须由绿转红 | 改法待主 agent 选 | 2026-09-27 A4d 报告（`research/prompts/m2-rev-a4d-implementer-report.md`「七、交主 agent 的设计问题」第 4 条；用例清单见报告第六节第 1、2 条） |
| C577 | 系统配置没见证到的最新根，乙罩不到 | 判据只认系统配置槽里的 tail；发布的系统配置轮换是普通写，返回之前没有尾随屏障（代码审阅第 19 条），「根已 FUA、调用方拿到了返回、轮换还在缓存里」崩掉之后见证就没了，重开时那条根暂时读不出，乙照旧把它当成被抛弃、把它的单元再发出去（池级 checker 的 I-7.4（近 K 代块未被复用） 照实红）；钉这一格的用例：`a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread`、formatted_pool 的 `crash_recovery_abandoning_the_unwitnessed_row_publish_…`、`raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43` | 已定续，实现在做：取号那一写（`transaction.rs` 的 `write_acquired_instance`）不再写 tail = 0，改写取号那一刻按乙-配置同一取法读到的见证值（用户 2026-09-27 定「乙-配置续（推荐）」，岔路单 `research/prompts/c554-fix-forks.md` 第 2 行）；实现落地后按上面三条用例断言不再判 I-7.4（近 K 代块未被复用） 红；判别力自证：把「续」这一步去掉（回到写 0），检查必须变 | 等 A3a 交回再派实现；依据先指 E158（择根与修复四岔路） 第 4 次跑第二段与第三段 H1g 两份报告，E158（择根与修复四岔路） 实验页还没写这一段，门禁 75 号那一对等实验页补上再登记，这一格记欠 | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q1）；用户定续见 `records/2026-09-24-里程碑二收尾调度.md`「用户定乙-配置续」那一行 |
| C578 | 乙的读缓存代价没量 | 读阶段第一遍就经缓存读，扫 journal 读成、非全零的每个记录槽都进缓存；环写满一圈之后是环长 × 盘数（默认 768 MiB 环、两块盘约 1.5 GiB，推的，没量），活到读阶段判完 | 量一次挂载读阶段的峰值内存，对照有无这道缓存两组；要不要只在判出 N 为真之后才开缓存（第一遍不收，重读那一遍整遍打到盘上），或另立「乙-窄读」，交主 agent 选；判别力自证：把缓存关掉，峰值内存必须降 | 改法待选 | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q4） |
| C579 | 判据在单故障合法状态上也拒可写 | 所选那一版就是最新的、系统配置见证的就是它，而它那次发布的末条记录两份都读不出 ⇒ 判不出 ⇒ 按真 ⇒ 重读仍读不出 ⇒ 拒可写；照登记定义没改，只读挂载照常 | 造这一格镜像（末条记录两份都读不出，所选根即最新根），断言今天拒可写；判别力自证：把「两条都读不出按真」改成「按假」，检查必须变 | 条款没写要不要改这一判，交主 agent | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q5） |
| C580 | 两处读不出无声放过没动 | `recovery::effective_rollback_floor` 自己再择一次根、读实例表，读不出就「不按表滤」；挂着时抬 F 重算影子账用 `readable_roots`，根槽读不出的被抛弃根照旧不隔离、不计数（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 前半段） | 造这两格镜像，断言按定下的判法处置；判别力自证：把「不按表滤」/「不隔离」换成拒绝，今天的实现必须由绿转红 | 不在这一件的射程里，没改 | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q6） |
```

依据：research/prompts/m2-impl-c554-yi-implementer-report.md「六、停下交主 agent 的设计问题」

## 条 7

文件：`.claude/kb/decisions/18-块里携带什么信息.md`

旧串：
```
  - **取号之前逐盘核带不带所选那一版**（主 agent 2026-09-25 定，被攻过零轮；实现 `crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`，实二七）：可写挂载核施加前缀之后那一版；交进来的盘有一块不带就在取号之前、任何写之前拒可写（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`），只读挂载不做这道核、逐条试位置条目读所选那一版。「不带」两种：两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不「可见」；世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同。放行两种：只落后、单元都在（根落盘之后、轮换之前崩；取号失败回卷写回 tail 0），这是正常崩溃状态；系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）。 挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前同样逐盘核，一块盘一份本池 fsid 的自证系统配置槽都没有就拒；实现随「Z3-A 乙」那一件（还没派，写「实现待派」）。
```

新串：
```
  - **取号之前逐盘核带不带所选那一版**（主 agent 2026-09-25 定，被攻过零轮；实现 `crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`，实二七）：可写挂载核施加前缀之后那一版；交进来的盘有一块不带就在取号之前、任何写之前拒可写（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`），只读挂载不做这道核、逐条试位置条目读所选那一版。「不带」两种：两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不「可见」；世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同。放行两种：只落后、单元都在（根落盘之后、轮换之前崩；取号失败回卷写回 tail 0），这是正常崩溃状态；系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）。挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前同样核「可见」那一支（不核落后那一支，规格表第 1 行只要这一支），一块盘一份本池 fsid 的自证系统配置槽都没有就拒；会话「每次发布」按「每次用户改动」读（一次调用只核一次，在循环之前），循环里为空间不够推的每一串抬 F 自己再核一次、紧挨着重发之前，所以实际发生的每个写之前都核过——这是推的，没有用例在循环中途把盘换掉。已实现（`research/prompts/m2-impl-z3a-yi-implementer-report.md`），拒时报的成员：正常卸载、抬 F 报 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`disagreeing_device_table` 带 `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`）；管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`；会话报 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`。
```

依据：research/prompts/m2-impl-z3a-yi-implementer-report.md 第 44–58 行「三、条款与代码对照」；主 agent 2026-09-27 弹窗选「乙：每个入口都逐盘核，含会话发布」（records/2026-09-24-里程碑二收尾调度.md「Z1Q-B 与 Z3-A 用户定」那一行）

