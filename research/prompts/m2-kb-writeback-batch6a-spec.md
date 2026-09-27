文件：.claude/kb/decisions/16-发布语义.md

旧串：
```
**定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽；fsync 等根槽持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**系统配置槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一。**抬 F 那一串（准入与卸载）在它的第一次发布之前多一步**：先把新 F 写进每块盘的系统配置、过一道屏障（已定项 1「抬 F 那一串」）；这一次系统配置写不属于任何一次发布，每次发布内部的顺序照旧。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回（做法与次数在已定项 8）；管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），它的根持久之后才向管理员确认，与 fsync 同一个返回条件。
```

新串：
```
**定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回（用户 2026-09-27 JST 15:1x 定「发布返回前加屏障」，序点从三个变四个）。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**系统配置槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一。**抬 F 那一串（准入与卸载）在它的第一次发布之前多一步**：先把新 F 写进每块盘的系统配置、过一道屏障（已定项 1「抬 F 那一串」）；这一次系统配置写不属于任何一次发布，每次发布内部的顺序照旧。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回（做法与次数在已定项 8）；管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），它的根持久之后才向管理员确认，与 fsync 同一个返回条件；实现待派，排在实审 A3c 之后（同在 `transaction.rs`），代价另登记一个小实验量。
```

依据：用户 2026-09-27 JST 15:1x 定案「发布返回前加屏障」，原话在 records/2026-09-24-里程碑二收尾调度.md「三份定义的来历；模型那一件交回并打上；C577 用户定加屏障」那一行

文件：.claude/kb/decisions/16-发布语义.md

旧串：
```
- **屏障口径**：两道 FLUSH + 根槽 FUA = 每次发布三个序点，比 D25（目标负载优先级） 推导的两个多一个——那是第二道屏障的价钱，知情接受。
```

新串：
```
- **屏障口径**：两道 FLUSH + 根槽 FUA + 系统配置轮换之后一道屏障 = 每次发布四个序点，比 D25（目标负载优先级） 推导的两个多两个——多的两个是第二道屏障与「发布返回前加屏障」的价钱，知情接受（后者用户 2026-09-27 JST 15:1x 定，起因是 C577（系统配置没见证到的最新根，乙罩不到） 那条用例在抬 F 之前更早红：发布路径根槽 FUA 之后轮换系统配置就返回，没有屏障）。
```

依据：同上；用户 2026-09-27 JST 15:1x 定案

文件：.claude/kb/decisions/23-journal的角色与格式.md

旧串：
```
**可写挂载读到的更新状态读不出（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，用户 2026-09-27 定；判据 N-配置续 照 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行）**：读阶段（择根、扫 journal、重放）读完，取池里每块盘两槽里全部自证过的系统配置槽 journal tail 的最大值 c_见证；0 ⇒ 不判；所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；两条都读不出按真。为真就在这次挂载内有效的读缓存（只收读成且不是全零的落点）上把读阶段重做一遍（R = 1，取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等），重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常。重建分配器判抛弃用的最新那条根的实例表读不出，同样重读一次、仍读不出拒可写（代码审阅第 22 条）。**「续」**：取号那一写（`crates/singlefs-core/src/transaction.rs` 的 `write_acquired_instance`）不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证（用户 2026-09-27 JST 12:08 定「乙-配置续（推荐）」；实现在做，实现员 2026-09-27 JST 12:1x 派出）。系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到；这一形记欠见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 Q1。
```

新串：
```
**可写挂载读到的更新状态读不出（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，用户 2026-09-27 定；判据 N-配置续 照 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行）**：读阶段（择根、扫 journal、重放）读完，取池里每块盘两槽里全部自证过的系统配置槽 journal tail 的最大值 c_见证；0 ⇒ 不判；所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；两条都读不出按真。为真就在这次挂载内有效的读缓存（只收读成且不是全零的落点）上把读阶段重做一遍（R = 1，取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等），重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常。重建分配器判抛弃用的最新那条根的实例表读不出，同样重读一次、仍读不出拒可写（代码审阅第 22 条）。**「续」**：取号那一写（`crates/singlefs-core/src/transaction.rs` 的 `write_acquired_instance`）不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证（用户 2026-09-27 JST 12:08 定「乙-配置续（推荐）」；实现员 2026-09-27 JST 12:4x 交回并打上，落点是 `write_acquired_instance` 里的 `highest_system_configuration_journal_tail`：每块盘两槽里全部自证过、fsid 与本池相同的系统配置槽 journal_tail 取最大，一份都没有时 0，在第一道屏障之后、第一个取号写之前读一次）。取号失败的回卷写（D18（块里携带什么信息） 已定项 11）带同一个见证值，不退回 0。mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：第一个事务的字节不变。取号那一刻某块盘一份自证过的系统配置槽都没有（见证值那一遍读错，逐盘核那一遍读得出）：拒（主 agent 2026-09-27 定），实现在做（实审 A3c）。系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到；这一形记欠见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 Q1。
```

依据：实现员报告 research/prompts/m2-impl-c554-yi-carry-implementer-report.md 第五节 1/2/3/4 条；用户 2026-09-27 JST 12:08 定「乙-配置续」、主 agent 定 Q1 拒绝条款

文件：.claude/kb/decisions/18-块里携带什么信息.md

旧串：
```
  - **取号之前逐盘核带不带所选那一版**（主 agent 2026-09-25 定，被攻过零轮；实现 `crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`，实二七）：可写挂载核施加前缀之后那一版；交进来的盘有一块不带就在取号之前、任何写之前拒可写（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`），只读挂载不做这道核、逐条试位置条目读所选那一版。「不带」两种：两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不「可见」；世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同。放行两种：只落后、单元都在（根落盘之后、轮换之前崩；取号失败回卷写回 tail 0），这是正常崩溃状态；系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）。挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前同样核「可见」那一支（不核落后那一支，规格表第 1 行只要这一支），一块盘一份本池 fsid 的自证系统配置槽都没有就拒；会话「每次发布」按「每次用户改动」读（一次调用只核一次，在循环之前），循环里为空间不够推的每一串抬 F 自己再核一次、紧挨着重发之前，所以实际发生的每个写之前都核过——这是推的，没有用例在循环中途把盘换掉。已实现（`research/prompts/m2-impl-z3a-yi-implementer-report.md`），拒时报的成员：正常卸载、抬 F 报 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`disagreeing_device_table` 带 `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`）；管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`；会话报 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`。
```

新串：
```
  - **取号之前逐盘核带不带所选那一版**（主 agent 2026-09-25 定，被攻过零轮；实现 `crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`，实二七）：可写挂载核施加前缀之后那一版；交进来的盘有一块不带就在取号之前、任何写之前拒可写（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`），只读挂载不做这道核、逐条试位置条目读所选那一版。「不带」两种：两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不「可见」；世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同。放行两种：只落后、单元都在（根落盘之后、轮换之前崩；取号失败回卷写回取号那一刻的见证值（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续）），这是正常崩溃状态；系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）。挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前同样核「可见」那一支（不核落后那一支，规格表第 1 行只要这一支），一块盘一份本池 fsid 的自证系统配置槽都没有就拒；会话「每次发布」按「每次用户改动」读（一次调用只核一次，在循环之前），循环里为空间不够推的每一串抬 F 自己再核一次、紧挨着重发之前，所以实际发生的每个写之前都核过——这是推的，没有用例在循环中途把盘换掉。已实现（`research/prompts/m2-impl-z3a-yi-implementer-report.md`），拒时报的成员：正常卸载、抬 F 报 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`disagreeing_device_table` 带 `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`）；管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`；会话报 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`。
```

依据：实现员报告 research/prompts/m2-impl-c554-yi-carry-implementer-report.md 第五节「清单外……」一条

文件：.claude/kb/checks-owed.md

旧串：
```
| C577 | 系统配置没见证到的最新根，乙罩不到 | 判据只认系统配置槽里的 tail；发布的系统配置轮换是普通写，返回之前没有尾随屏障（代码审阅第 19 条），「根已 FUA、调用方拿到了返回、轮换还在缓存里」崩掉之后见证就没了，重开时那条根暂时读不出，乙照旧把它当成被抛弃、把它的单元再发出去（池级 checker 的 I-7.4（近 K 代块未被复用） 照实红）；钉这一格的用例：`a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread`、formatted_pool 的 `crash_recovery_abandoning_the_unwitnessed_row_publish_…`、`raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43` | 已定续，实现在做：取号那一写（`transaction.rs` 的 `write_acquired_instance`）不再写 tail = 0，改写取号那一刻按乙-配置同一取法读到的见证值（用户 2026-09-27 JST 12:08 定「乙-配置续（推荐）」，岔路单 `research/prompts/c554-fix-forks.md` 第 2 行）；实现落地后按「要拦什么」一栏点名的三条用例断言不再判 I-7.4（近 K 代块未被复用） 红；判别力自证：把「续」这一步去掉（回到写 0），检查必须变 | 实现员 2026-09-27 JST 12:1x 派出（只动 `transaction.rs` 的 `write_acquired_instance`）；依据先指 E158（择根与修复四岔路） 第 4 次跑第二段与第三段 H1g 两份报告，E158（择根与修复四岔路） 实验页还没写这一段，门禁 75 号那一对等实验页补上再登记，这一格记欠 | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q1）；用户定续见 `records/2026-09-24-里程碑二收尾调度.md`「用户定乙-配置续」那一行 |
```

新串：
```
| C577 | 系统配置没见证到的最新根，乙罩不到 | 判据只认系统配置槽里的 tail；发布的系统配置轮换是普通写，返回之前没有尾随屏障（代码审阅第 19 条），「根已 FUA、调用方拿到了返回、轮换还在缓存里」崩掉之后见证就没了，重开时那条根暂时读不出，乙照旧把它当成被抛弃、把它的单元再发出去（池级 checker 的 I-7.4（近 K 代块未被复用） 照实红）；钉这一格的用例：`a_newest_root_the_system_configuration_never_witnessed_is_abandoned_without_a_reread`、formatted_pool 的 `crash_recovery_abandoning_the_unwitnessed_row_publish_…`、`raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43` | 已定续，实现已落地（`transaction.rs` 的 `write_acquired_instance` 改写取号那一刻按乙-配置同一取法读到的见证值，实现员 2026-09-27 JST 12:4x 交回并打上，`m2-impl-c554-yi-carry-implementer-report.md`）；按「要拦什么」一栏点名的三条用例，`raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43` 仍红得更早：抬 F 之前 checker 已报 I-7.4（近 K 代块未被复用），因为发布路径根槽 FUA 之后轮换系统配置就返回、没有屏障（`transaction.rs:1109`）；用户已定「发布返回前加屏障」（2026-09-27 JST 15:1x，D16（发布语义） 已定项 7），加屏障落地后这条用例断言不再判 I-7.4（近 K 代块未被复用） 红；判别力自证：把「续」这一步去掉（回到写 0）或把新屏障去掉，检查都必须变红 | 乙-配置续已派出并交回（实现员 2026-09-27 JST 12:1x 派出，12:4x 交回并打上，只动 `transaction.rs` 的 `write_acquired_instance`）；加屏障待主 agent 派实现员，排在实审 A3c 之后（`records/2026-09-24-里程碑二收尾调度.md`「三份定义的来历……」那一行）；依据先指 E158（择根与修复四岔路） 第 4 次跑第二段与第三段 H1g 两份报告，E158（择根与修复四岔路） 实验页还没写这一段，门禁 75 号那一对等实验页补上再登记，这一格记欠 | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q1）；用户定续见 `records/2026-09-24-里程碑二收尾调度.md`「用户定乙-配置续」那一行 |
```

依据：任务1（D16 已定项 7 加屏障，实现待派）与任务2（C554 乙-配置续，实现员 2026-09-27 JST 12:4x 交回）

文件：.claude/kb/decisions-history/2026-09.md

旧串：
```
### 2026-09-27（其十三）：D9（加密） 已定项 10：读路径判单元头、系统配置加密类型与两处指针头部，checker 那一半在做
```

新串：
```
### 2026-09-27（其十五）：D23（journal 的角色与格式） 已定项 14、D18（块里携带什么信息） 已定项 11：C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续实现落地

> 快查·改前：D23（journal 的角色与格式） 已定项 14「续」那句写「实现在做，实现员 2026-09-27 JST 12:1x 派出」，没写回卷写与 mkfs 的字节不变，也没写取号那一刻某块盘见证槽都读不出怎么办；D18（块里携带什么信息） 已定项 11 写「取号失败回卷写回 tail 0」。
>
> 快查·改后：「续」改写实现落点（`write_acquired_instance` 的 `highest_system_configuration_journal_tail`）并交回已打上；加两句：回卷写带同一见证值不退回 0，mkfs 写 tail 0 第一个事务字节不变；再加一句：取号那一刻某块盘见证槽都读不出就拒（主 agent 定，实审 A3c 在做）。D18（块里携带什么信息） 已定项 11 改成「取号失败回卷写回取号那一刻的见证值」。

- 改前：D23（journal 的角色与格式） 已定项 14「续」那句只写「取号那一写不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证；实现在做，实现员 2026-09-27 JST 12:1x 派出」；D18（块里携带什么信息） 已定项 11 写「只落后、单元都在（根落盘之后、轮换之前崩；取号失败回卷写回 tail 0）」。
- 改后：实现员 2026-09-27 JST 12:4x 交回并打上（`transaction.rs` 的 `highest_system_configuration_journal_tail`，第一道屏障之后、第一个取号写之前读一次，逐盘取号写与回卷写都带这一次读到的值）；D23（journal 的角色与格式） 已定项 14 补三句：取号失败的回卷写带同一个见证值不退回 0，mkfs 写 tail 0、第一个事务字节不变，取号那一刻某块盘一份自证过的系统配置槽都没有就拒（主 agent 2026-09-27 定，实审 A3c 在做）；D18（块里携带什么信息） 已定项 11 那句改成「取号失败回卷写回取号那一刻的见证值」。
- 依据：实现员报告 `research/prompts/m2-impl-c554-yi-carry-implementer-report.md` 第五节 1–4 条；用户 2026-09-27 JST 12:08 定「乙-配置续（推荐）」（`records/2026-09-24-里程碑二收尾调度.md`「乙-配置续交回并打上；A3c 派出」那一行）。

### 2026-09-27（其十三）：D9（加密） 已定项 10：读路径判单元头、系统配置加密类型与两处指针头部，checker 那一半在做
```

依据：任务2：C554 乙-配置续实现落地（实现员 2026-09-27 JST 12:4x 交回）

文件：.claude/kb/decisions-history/2026-09.md

旧串：
```
### 2026-09-27（其十三）：D9（加密） 已定项 10：读路径判单元头、系统配置加密类型与两处指针头部，checker 那一半在做
```

新串：
```
### 2026-09-27（其十四）：D16（发布语义） 已定项 7：发布返回前加系统配置轮换之后的屏障，序点从三个变四个

> 快查·改前：定案写「……→ 根槽（FUA）→ 系统配置槽；fsync 等根槽持久之后才返回」，屏障口径写「两道 FLUSH + 根槽 FUA = 每次发布三个序点」。
>
> 快查·改后：定案加一道屏障——「……→ 根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回」，屏障口径改成「两道 FLUSH + 根槽 FUA + 系统配置轮换之后一道屏障 = 每次发布四个序点」；实现待派，排在实审 A3c 之后。

- 改前：D16（发布语义） 已定项 7 定案「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽；fsync 等根槽持久之后才返回」，射程「屏障口径」写「两道 FLUSH + 根槽 FUA = 每次发布三个序点，比 D25（目标负载优先级） 推导的两个多一个」。
- 改后：主 agent 在主工作区现状（乙、乙-配置续都已合入）单跑 `raising_the_floor_into_the_txg_of_the_root_abandoned_by_crash_recovery_ends_in_the_known_red_form_of_closeout_row_43`，红得更早——抬 F 之前 checker 已报 I-7.4（近 K 代块未被复用），因为发布路径根槽 FUA 之后轮换系统配置就返回、没有屏障（`transaction.rs:1109`）。定案与屏障口径改成：系统配置槽之后再加一道屏障，fsync 等系统配置轮换持久之后才返回，序点从三个变四个；实现待派，排在实审 A3c 之后（同在 `transaction.rs`），代价另登记一个小实验量。C577（系统配置没见证到的最新根，乙罩不到） 那一行状态跟着改。
- 依据：用户定案 2026-09-27 JST 15:1x「发布返回前加屏障（推荐）」，原话在 `records/2026-09-24-里程碑二收尾调度.md` 2026-09-27 15:0x–15:1x 那一段调度记录（标题里含 C577（系统配置没见证到的最新根，乙罩不到） 那一行）；岔路单 `research/prompts/c554-fix-forks.md` 第 4 行。

### 2026-09-27（其十三）：D9（加密） 已定项 10：读路径判单元头、系统配置加密类型与两处指针头部，checker 那一半在做
```

依据：任务1：D16 已定项 7 加屏障（用户 2026-09-27 JST 15:1x 定案）
