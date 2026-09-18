# m2-step45-code-r3：云端正推腿报告（Sonnet）

分到的格：X6 代码与条款（T1 / T2 / T3）、方向（F_生效 回落三个改法各修哪一格）、X7 写回。
不判 X1、X2、X3、X4、X5、X8（分给别的腿），不替攻方找新反例。

## 各格一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1（G5 影子账） | **一致** | G5 与用户窄读法措辞的差别在概念上只有「txg ≥ F」一条；D23 主句在固定脚本每一步（回退、四次覆盖写、抬 F、E）都成立，副本上逐步核对过 |
| T2（读不出账的被抛弃根） | **一致** | D23 主句只讲"能读到账"的情形，对读不出的情形一个字都没说；代码的处置（跳过 + 计数 + 不拒挂）是在这个空白上补的预想，与代码、kb 描述都对得上 |
| T3（抬 F 扣住到生效） | **一致，但发现一处结构性缺口** | 扣住到"带新 F 的根落满每块盘"确实是条款说的"生效"；记账时点与 I-3.1 并集口径在崩在两条根之间的镜像上副本实测吻合（0 违例）。但 `release_reclaim_holds()` 是循环退出后无条件调用，不检查是不是真的覆盖了每块盘——今天的格式常量（ROOT_RING_REGIONS=3，区域按 txg 取模循环）让这条路径不可达，是潜在缺口不是今天的活漏洞，与 X8 的攻击面有交叉 |
| 方向（F_生效 回落三个改法） | **冲突**：r2 云端攻方腿关于改法 (a) 的判定与副本实测不符 | (a)「新根写 max(...)」按最直白的写法（只改 `establish_instance` 那一行）**不能**让 X3-A 的 I-3.1 转绿——副本实测仍红；把 `rebuilt_allocator` 内部的 `effective_floor` 也一起取 max 之后差距缩小但仍未清零。r2 报告写"X3-A 不中"（隐含三条不变量都清），按副本实测不成立 |
| X7 写回 | **一处遗漏，其余一致** | `second_transaction_step_four_rollback.rs` 第 2 行模块级文档仍写"jsn 接在 A 那条记录之后 = 4"（P1 措辞），与 r2 决策第五节 3"步 4 用例文件模块级文档 jsn 9"的说法不符；`invariants.md:120`、`checks-owed.md:311`、`02-second-txn.md:160/189`、`mount_rollback` 函数文档（702-705 行）逐一核对，与代码一致 |

## T1：G5 影子账与 D23 主句

### T1a：G5 与用户窄读法措辞的差别是不是只有「txg ≥ F」一条

`crates/singlefs-core/src/mount.rs:201-208`（`isolate_slots_referenced_only_by_abandoned_roots` 的文档注释）整段：

> 影子账（D28（挂载期承诺量） 已定项 1 第九项）：只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里
> 还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F。用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，
> 豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严（alloc-basis 那一轮的 G5 臂）：按原措辞，抬 F 到 11 之后
> A 仍豁免 mkfs 实例表那 2 槽，它们被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句。

注释字面列了两样：「多了 txg ≥ F」「抬 F 之后按新 F 重算」。核下来这两样不是并列的两条独立差别，是因果关系：

- 用户窄读法措辞（附录 `.claude/kb/decisions/23-journal的角色与格式.md:1206-1244` 段）只说「有效只按实例表判」，完全没提 F，也没提"什么时候重算"。
- 「每次挂载都要重算」这件事，**在第一轮就已经因为另一条独立的问题被修过**（`mount.rs:262` 注释「影子账只住内存，所以每次挂载都要重算，不只回退那一次（步 4 / 步 5 代码三方第一轮云端攻方腿打中：回退之后普通重开一次隔离就归零）」）——这是"普通重开要不要重算"的问题，与 F 完全无关，G5 之前就已经这样做了。
- G5 真正新增的、`rebuilt_allocator` 之外的调用点，是 `raise_rollback_floor` 里 `mount.rs:447-464` 那一段——在回收之前，**用新 F** 再调一次 `isolate_slots_referenced_only_by_abandoned_roots`。这个新增调用点之所以存在，**只是因为候选集第一次把 F 纳入了判据**：候选集缩小（F 从 0 抬到 11 之后，A 不再是候选）会让此前被 A 豁免的槽变成"只被被抛弃根引用"，如果不加这一条 F 的门槛，抬 F 这件事根本不会改变任何一条根的候选身份，也就不需要在抬 F 时重算。

⇒ 判定：**一致**。「多了 txg ≥ F」是唯一的定义性差别；「抬 F 之后按新 F 重算」是这条差别的必然推论，不是第二条独立选择。**什么现象会推翻它**：如果能找到一处「不涉及 F 的候选身份变化」却仍然需要在抬 F 时补一次隔离的场景（即重算的触发条件与 F 无关），那就说明重算是独立于 F 的第二条差别，判定要改成"至少两条"。

### T1b：D23 主句在 G5 下逐格成立吗（固定脚本每一步）

D23 主句（附录）逐字：「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」。逐步核：

1. **回退那一次**（`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 里
   `rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content`）：
   隔离 34 槽（B 10 + 写行 6 + 暖机 4+4 + C 10），mkfs 实例表 2 槽因 A 仍是候选（F=0）而不隔离——但 A 自己的账（`rebuild_version(A)` 重建出的 `previous`）里这 2 槽本来就标记为已分配未释放，`PoolAllocator::rebuild_from_records`（`crates/singlefs-core/src/allocator.rs:397-430`）据此把它们的 `allocated[i]` 置 true，分配器天然发不出去，不需要靠隔离位再挡一次。这一步测试断言 0 违例，副本上重跑该用例通过（见「变异复跑」一节，baseline 全绿）。
2. **四次覆盖写**（回退之后，同一进程内，`four_overwrites_after_the_rollback`）：不触发任何重建或抬 F，隔离位是纯 in-memory 状态、只增不减（`allocator.rs:200-212` 的 `isolate` 函数：`if !self.isolated[index] { ... }`，没有清除路径），34 个隔离槽原样保留。
3. **抬 F**（`raise_rollback_floor`，`mount.rs:447-464`）：F 从 0 抬到 11 之后 A 掉出候选集，`mount.rs:456-464` 在回收（`reclaim_released_up_to`）之前用新 F 重新调用隔离函数，把 mkfs 实例表 2 槽（此时已经在 `current.allocation_records` 里被 D 的写行发布释放、进入 defer 队列）新增隔离；`step_five_reuse.rs` 的
   `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 断言 E 拿到 50178 而不是 mkfs 实例表所在的 50176——即 D23 主句在这一步仍然成立（0 违例）。
4. **E**：同一测试的 checker 断言全绿（含 I-3.1、I-2.1、I-5.2、I-7.2）。

⇒ 判定：**一致**，四步都有对应的用例把 0 违例钉住，副本上重跑（见后）逐个复现。**什么现象会推翻它**：若能构造一段历史让 34/36 之外还有第三类槽既不在候选集里也不在隔离集里，且被后续发布覆盖，就会打穿这一格——但这属于攻方腿（X1/X2）的攻击面，我没有去找这样的历史。
## T2：D23 主句对「读不出账的被抛弃根」说了什么、没说什么

D23 主句与做法半句（附录）逐字：「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中只被被抛弃根引用的槽」。

这句话的施动对象是"环里每一个**可读**根"——它预设了账是读得出来的，对"账读不出"的情形一个字都没提，既没说"读不出就拒绝挂载"，也没说"读不出就跳过"。这是一处真空白，不是我读漏了：`.claude/kb/checks-owed.md:311` 的 C340 是"回退之后记录链从哪条之后接没有定义"，管的是另一件事（jsn 怎么接），没有覆盖"账读不出"这一格；全文搜 D23 已定项 14 整段（`.claude/kb/decisions/23-journal的角色与格式.md:1206-1244`）不出现"读不出""解不开""撕裂"这几个词。

代码的处置（`mount.rs:213-258` `isolate_slots_referenced_only_by_abandoned_roots`）：

```rust
for root in roots.iter().filter(|root| is_abandoned(root)) {
    let Ok(records) = allocation_records_under_root(devices, root) else {
        unreadable += 1;
        continue;
    };
    ...
}
```

跳过、计数（`unreadable`，经 `rebuilt_allocator` 传出为 `MountOutput::abandoned_roots_unreadable`，`mount.rs:99-101`），不拒绝挂载。用例
`torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount`（改坏 C 的树表两份）验收：挂载成功、计 1、隔离从 34 降到 24（少了只被 C 引用的 10 槽）。

⇒ 判定：**一致**。代码注释（`mount.rs:207`「树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载」）与我读到的 D23 原文空白完全对得上——代码没有"违反"或"曲解"D23 的字面，只是在 D23 没说的地方做了一个明确标注为"预想"的选择（`mount.rs:205`「预想、偏离用户定案的措辞，交用户」这句其实是紧跟在 T1 那句上，但 T2 的读不出处置也在同一注释块里、同样只是预想，`02-second-txn.md:160` 与 r2 决策第六节表格第 4 行都把它列为决策点，未定案）。**什么现象会推翻它**：如果 D23 别的地方（比如某条尚未读到的分项）其实已经规定了"读不出时拒绝挂载"，这个判定就要改成"冲突"——我在附录给出的整段 D23 已定项 14 与已定项 3、7 相关引用里没找到这样的句子。

**候选根读不出的对称性**：`mount.rs:230` 用 `if let Ok(records) = ... { ... }`（没有 `unreadable` 计数）——候选根读不出时该根"什么都不豁免"，隔离只会多不会少，这与 T2（被抛弃根读不出则"什么都不隔离"，隔离只会少不会多）方向相反，但两者都在代码注释里显式写明（`mount.rs:207-208`），且都不是 D23 明文规定的，是同一处空白的两半——一半宁可多隔离，一半宁可不隔离，代码对这个不对称性没有回避，如实写在注释与 `02-second-txn.md:160` 里。
## T3：D16 已定项 1 生效行、可再分配行与 I-3.1 并集口径

### T3a：扣住到「带新 F 的根落满每块盘」是不是条款说的「生效」

D16 已定项 1（附录 `.claude/kb/decisions/16-发布语义.md:358-416`）表格「生效」行逐字：「每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值」。

代码（`mount.rs:397-507` `raise_rollback_floor`）：先回收（`mount.rs:465-468`，`ReclaimedReuse::HeldUntilFloorTakesEffect`）、扣住位单独维护（`allocator.rs:122-126` 字段定义、`allocator.rs:220-232` `hold_until_floor_takes_effect`），然后推空发布直到 `covered` 覆盖 `all_devices`（`mount.rs:472-500`），循环结束后 `mount.rs:501` `allocator.release_reclaim_holds()` 放开。`covered` 的判定标准是"这次推空发布真落到了哪块盘"（`device_of_txg`），与 D16 的"生效"定义（每块幸存盘都有带新 F 的持久根）字面对得上——**只要循环是因为 `covered` 覆盖了 `all_devices` 而正常退出**，两者是同一件事。

⇒ 就"扣住到生效才放开"这个概念本身：**一致**。

**但我核出一处循环退出条件与释放时机脱钩的缺口**：`mount.rs:472-476` 的 while 条件是

```rust
while all_devices.iter().any(|identity| !covered.contains(identity))
    && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
```

`mount.rs:501` 的 `release_reclaim_holds()` 在循环**之后无条件调用**，不检查 `covered` 是不是真的等于 `all_devices`。也就是说：如果循环是因为触到 `ROOT_RING_REGIONS`（`singlefs_format::ROOT_RING_REGIONS = 3`，`crates/singlefs-format/src/lib.rs:185`）上限而不是因为覆盖完成退出的，扣住的槽照样会被放开——这与"扣住到生效"的字面不符，是代码与条款的一处不对齐。

这一点恰好是这一轮判据表里 X8（T3 扣住，归 Opus 攻方）列出的问句之一（背景材料判据表 X8 行：「`release_reclaim_holds` 放开的时机是循环之后，循环因为 `ROOT_RING_REGIONS` 上限退出而没落满每块盘时也放开」）。我没有去构造让它真的打中的历史（那是攻方腿的事），但我核了一下**今天的格式常量下这条路径是不是可达**：`target_for_publish`（`crates/singlefs-core/src/root_ring.rs:33-38`）的区域号是 `checkpoint_txg % ROOT_RING_REGIONS`，周期恰好等于 3；`region_devices` 是一个固定的 3 元数组，把 3 个区域映射到实际存在的盘（当前是 2 块）。**任意连续 3 个 txg 的区域号，按模 3 的性质，是 {0,1,2} 的一个排列**——也就是说，无论从哪个 txg 起步，连续推 `ROOT_RING_REGIONS`（3）次空发布，一定会把 3 个区域都推过一遍，因而覆盖 `region_devices` 里出现过的每一块盘。这意味着"循环因为 `ROOT_RING_REGIONS` 上限退出而没有覆盖完"这件事，在**当前 3 区域 / 循环上限也是 3**这个数值巧合下，是**不可达**的——上限恰好等于覆盖所需的最坏步数上界。

⇒ 判定：**一致（就今天能走到的历史而言），但代码结构本身不是靠"检查覆盖完成"来保证这一点，而是靠一个数值巧合**（ROOT_RING_REGIONS 恰好等于区域循环长度）。**什么现象会推翻它**：如果以后盘数变多、或者 `region_devices` 的某一块盘只出现在很靠后的区域号里、或者 `ROOT_RING_REGIONS` 与实际盘数的关系变化，这条无条件释放就会真的在没覆盖完时触发——这正是 X8 该去找的格；我在这里只核实了"今天数值上够不着"，不构成"以后也够不着"的证据。
### T3b：记账时点与 I-3.1 并集口径在崩在两条根之间的镜像上对不对得上（副本上崩了跑 checker）

`mount.rs:433-436` 注释：回收（记账变动）在写第一条带新 F 的根**之前**动；`walk.rs:786-799` 的 I-3.1 并集口径读的是**最新根自己带的 F**（`newest_rollback_floor`，从 `record_bytes[130..138]` 取）。这两句分别来自不同的判据（前者是"哪一刻改记账"，后者是"checker 用哪个 F 收候选集"），我在副本上直接构造了"崩在两条带新 F 的根之间"这个镜像并跑 checker，而不是靠推理。

**做法**：`build_through_rollback` → 四次覆盖写（11–14）→ `raise_floor(11)`（两次空发布 txg15/盘0、txg16/盘1）→ 翻掉 txg16 根槽一个字节（模拟"第二条带新 F 的根没有持久"）→ **不重新挂载**，直接对这个镜像跑 `check_pool_image`。

副本探针（`/tmp/claude-1000/m2-step45-code-r3-sonnet/copy`，新增测试
`probe_r3_t3b_checker_on_the_crashed_between_two_floor_roots_image_without_remounting`，只在副本，不进正式验收）原样输出：

```
R3-T3B-VERDICTS [("I-1.1", Holds), ("I-1.3", Holds), ("I-1.4", Holds), ("I-1.6", Holds), ("I-1.7", Holds), ("I-2.1", Holds), ("I-2.3", Holds), ("I-2.4", Holds), ("I-2.5", Holds), ("I-3.1", Holds), ("I-3.8", Holds), ("I-5.1", Holds), ("I-5.2", Holds), ("I-7.1", Holds), ("I-7.2", Holds), ("I-7.6", Holds), ("I-7.7", Holds), ("I-7.8", Holds), ("I-9.1", Holds), ("I-9.2", Holds), ("I-9.4", Holds), ("I-9.7", Holds), ("I-9.10", Holds), ("I-9.13", Holds)]
R3-T3B-VIOLATED []
test probe_r3_t3b_checker_on_the_crashed_between_two_floor_roots_image_without_remounting ... ok
```

0 违例。机理：这个崩溃点上，最新自证根仍是 txg15（txg16 已损坏、`choose_root` 择不到它），txg15 自己的 F=11、自己的记账树（在写 txg15 之前已经完成回收）与自己作为"最新根"时 checker 用来收候选集的 F（同样是 11）是**同一条根的同一份数据**——两者天然对齐，不需要额外机制保证。

⇒ 判定：**一致**。**什么现象会推翻它**：如果崩溃点落在"记账已经反映新 F 的效果、但最新自证根却是一个带着旧 F 的根"这种组合（比如记账树和根记录不是原子一起写的），才会看到两者脱节——但当前提交路径下记账树与根记录同一次发布原子落盘，我没有找到能拆开它们的历史（也不在我分到的格里去找）。这一格容易与"方向"里 X3-A 的"F 回落"混淆：X3-A 问的是**合法复用之后、再损坏根槽导致 F 倒退**（跨越多次挂载，见下一节），T3b 问的是**单次抬 F 过程内部、两条带新 F 的根之间**——两者是不同的崩溃点，T3b 这里没有复现出 X3-A 的问题。
## 方向：F_生效 回落那一格的三个改法各修哪一格

背景材料（`_m2-step45-code-r3-background.md` 第 37 行）与 r2 云端攻方腿报告（`m2-step45-code-r2-opus-output.md` 第 305 行）给出的表：

> (a) 新实例的根写 max(上一条根的 F, F_生效)——X3-A / X3-B 不中，X3-C / X3-D 仍中；
> (b) 回退候选集也用 max(…)——X3-C / X3-D 不中，但第一轮正推腿判的那条「窄化」回来；
> (c) 只改 checker（候选集用 F 的历史最大值）——X3-A 的 I-2.1 / I-5.1 不中，I-3.1 仍中，X3-C / X3-D 全中。

r2 报告自己在第七节写明「我提的改法都被攻过零轮，而且没在副本上量过……全是按代码推的，一条都没实现、没跑」。我在副本上把 (a) 实现了两版，对着 X3-A 的原始历史（回退 → 四次覆盖写 → 抬 F 到 11 → E 合法复用 50178 → 翻掉盘 1 唯一带 F=11 的根 → 重开可写挂载）逐格实测。

**候选 (a)，naive 版**：只改 `establish_instance` 里写新根 F 字段那一行（`mount.rs:562` 附近，`rollback_floor: start.effective_floor` → `start.effective_floor.max(previous.root.rollback_floor)`）。

副本原样输出（`probe_r3_direction_x3a_baseline_today`，先跑未打补丁的基线）：

```
R3-X3A-NEW-ROOT-F 0
R3-X3A-VIOLATED ["I-2.1", "I-3.1", "I-5.1"]
```

（基线复现了 r2 报告的原始数字，一致。）打上 naive 版补丁之后重跑：

```
R3-X3A-NEW-ROOT-F 11
R3-X3A-VIOLATED ["I-3.1"]
```

新根的 F 确实不再回落（11，而不是 0），I-2.1 与 I-5.1 确实转绿，**但 I-3.1 仍然红**：

```
R3-X3A-FULL row_publish.F=11 warm_up_len=1 warm_up_F=[11]
R3-X3A-FULL I-3.1 Violated("盘 0：记账的已分配 Some(1474560)，遍历全部有效根得到 1081344")
```

这与 r2 报告写的「(a)……X3-A 不中」（隐含三条不变量都清）**不符**：naive 版只清了两条，第三条（I-3.1）依旧红，而且违例数字与"没打补丁"时完全不同（说明不是同一个违例延续，是补丁改变了记账路径之后新产生的另一种不一致）。

**候选 (a)，深化版**：naive 版只改了"写进新根的 F 字段"，`rebuilt_allocator` 内部真正用来算回收门槛与影子账门槛的 `effective_floor`（`mount.rs:280-284` 现算值）没有跟着变。我进一步把 `rebuilt_allocator` 里 `effective_floor` 也取 `max(现算值, previous.root.rollback_floor)`（`mount.rs:280-285` 附近，`previous` 是重建时喂进来的上一版）。重跑：

```
R3-X3A-FULL row_publish.F=11 warm_up_len=1 warm_up_F=[11]
R3-X3A-FULL I-3.1 Violated("盘 0：记账的已分配 Some(1146880)，遍历全部有效根得到 1081344")
```

差距从 393216 字节（1474560−1081344）缩小到 65536 字节（1146880−1081344，正好 4 个槽），**但仍未清零**。我没有再往下追这 4 个槽具体是哪几个（时间不够），这超出了我分到的"核对表述准不准"这个格，留给后续。

⇒ 判定：**冲突**。r2 报告对候选 (a) 在 X3-A 这一格的判定（"不中"，即三条不变量都清）按副本实测**不成立**——至少 naive 版和"深化到 rebuilt_allocator 内部 effective_floor"这一版都留了 I-3.1 一条尾巴没清。r2 报告对候选 (c) 的判定（"I-3.1 仍中"）与我 naive 版 (a) 观测到的现象（I-3.1 仍红、I-2.1/I-5.1 转绿）在**方向上一致**——这是因为 naive 版 (a) 实际上只改了 checker 读到的"最新根的 F 字段"，对 checker 而言与候选 (c)（只改 checker 用哪个 F）的效果几乎等价，只是多了一步真的把这个值写上盘。这说明 r2 报告表格里 (a)/(c) 两行本身互相之间是自洽的（都承认 naive 改法留下 I-3.1），**问题出在 (a) 那一行的结论句把"新根 F 不回落"和"三条不变量都清"划了等号，这一步跳跃没有被验证过**。

**有没有第四个改法能把四格都修掉、不改用户定过的字面**：按我这次实测，"深化版 (a)"（新根的 F 字段与 `rebuilt_allocator` 内部的 `effective_floor` 一起取 max）已经比 r2 报告列出的三个改法都更接近"只改一个概念、不动 D16 已定项 1 的生效行/候选集行/checker 用哪个 F 那几处字面"，但它仍有 65536 字节（4 槽）的残差没有清零，**我没有查出这 4 槽是什么、也没有验证这条路径下 X3-B/C/D 四格的表现**，所以不能回答"有没有第四个改法能把四格都修掉"——只能说：至少现在报告里列出的三个改法（连同我试的深化版）都没有做到，这件事本身值得写进交用户的材料里，而不是照抄 r2 报告"三个改法各修一格"的现成结论。
## X7：写回与代码对不对得上

逐处核（引用原文整行抄，行号现查活仓）：

1. **`.claude/kb/milestone/02-second-txn.md:160`（步 4 现状）**：与 `mount_rollback`（`mount.rs:709-829`）、`isolate_slots_referenced_only_by_abandoned_roots`（`mount.rs:213-258`）、`rebuilt_allocator`（`mount.rs:267-325`）逐句核对——候选集判据、影子账窄读法与做法半句、抬 F 后重算、读不出计数、取号 3、D 的 txg/jsn/事务号/反向链、暖机一次，全部对得上代码与用例断言。**一致**。
2. **`.claude/kb/milestone/02-second-txn.md:189`（步 5 现状）**：与 `raise_rollback_floor`（`mount.rs:397-507`）、`reclaim_released_up_to`（`allocator.rs:566-604`）、`hold_until_floor_takes_effect`/`release_holds`（`allocator.rs:220-238`）逐句核对——回收门槛、扣住到生效、重建分配器认第 0 版树表单元（`allocator.rs:397-430`）、没做过可写挂载抬 F 报错（`mount.rs:407-411`）全部对得上。**一致**。
3. **`.claude/kb/invariants.md:120`（I-3.1 状态列）**：与 `walk.rs:786-799` 逐句核——checker 读最新根自己那份 F（`record_bytes[130..138]`）、回退候选集用跨盘的 F_生效、两者可以不同（这正是"方向"一节 X3-A 复现的现象）。**一致**。
4. **`.claude/kb/checks-owed.md:311`（C340 第三列，"要拦什么"）**：与附录 D23 原文逐句核——"不施加 R_old 之后的任何记录"（附录已定项 14 段）、"链从所选根覆盖的最后一条记录之后接"（附录已定项 14 注 1）两句都在附录原文里逐字找到，C340 第三列对这两句的引用没有摘句、没有走样。**一致**。
5. **`mount_rollback` 函数级文档（`mount.rs:702-705`）**：「新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 取 P2，预想、等用户定）」——与代码实现（`mount.rs:769-774` `let next_counter = highest_counter + 1;`，`highest_counter` 取全环最大 counter）一致，也与 r2 决策第五节 3「`mount_rollback` 函数级文档改成 P2」的说法对得上。**一致**。
6. **`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 第 2 行（模块级文档）**：现读原样：

   ```
   //! 取实例代号 3、在 A 那一版实例表上写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、发布 D（txg 9，jsn 接在 A 那条记录之后 = 4）、
   ```

   「jsn 接在 A 那条记录之后 = 4」是 **P1** 的措辞（jsn 接在 R_old 自己那条记录之后）。而 r2 决策第五节 3 明写「步 4 用例文件模块级文档 jsn 9」——按这句话，这一行本该被改成类似"jsn 接在环里最大的 jsn 之后 = 9"。**这一行没有被改**，与 r2 决策"已改"的记录不符。作为对照，同一文件里函数级文档（`the_rollback_row_caps_the_prefix...` 上一个测试，即
   `rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content`
   紧邻的断言注释，`second_transaction_step_four_rollback.rs` 里）**已经正确写成** P2/jsn9 的措辞（「jsn 接在环里最大的 8 之后（C340 取 P2，被抛弃的记录一条不盖）」），说明这处遗漏只发生在模块级文档这一行，不是全文件都没改。

   ⇒ 判定：**不一致**。这是一处遗漏的写回，不是代码逻辑错误（代码本身按 P2 实现，测试断言也按 P2 写，只有这一句模块级说明没跟上）。**什么现象会推翻它**：如果这一行原本就是有意保留 P1 措辞来描述某种别的场景（比如描述这个文件"历史上"曾经的实现），那就不算遗漏——但 `.claude/singlefs-ai-sop/rules/design-doc-discipline.md`「正文只写当前现状」的纪律要求正文不留历史陈述，而这不是"历史版本"节，是模块开头的当前说明，所以按这条纪律看仍是遗漏。
## 变异复跑（`crates/mutations.tsv` 第 50 行之后的每条新变异）

在副本 `/tmp/claude-1000/m2-step45-code-r3-sonnet/copy` 上，用一段照 `.claude/gate.d/59-crates-mutation-replay.sh` 同样逻辑写的脚本（原文命中 1 次才改、跑点名测试、必须判红、还原），只跑第 51–56 行（第 50 行本身"步 4：影子账不豁免候选根引用的槽（保守读法）"不在"第 50 行之后"范围内，未跑；已在下方"没做什么"注明）：

```
loaded 6 rows (lines >= 51)
51 步 4：被抛弃根的树表读不出时不计数
52 步 5：抬 F 之后不按新候选集重算影子账
53 步 5：影子账的豁免不看候选根的 txg 是否 ≥ F
54 步 5：抬 F 回收的槽不扣住、生效之前就能发出去
55 步 5：抬 F 生效之后不放开扣住的槽
56 步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错

=== RESULTS ===
51	步 4：被抛弃根的树表读不出时不计数	RED (as expected)
52	步 5：抬 F 之后不按新候选集重算影子账	RED (as expected)
53	步 5：影子账的豁免不看候选根的 txg 是否 ≥ F	RED (as expected)
54	步 5：抬 F 回收的槽不扣住、生效之前就能发出去	RED (as expected)
55	步 5：抬 F 生效之后不放开扣住的槽	RED (as expected)
56	步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错	RED (as expected)

FAILURES: 0
```

6 条全红，锚点各命中 1 次。另外跑了一次基线（未打任何变异）确认 `second_transaction_step_four_rollback.rs`（6 个测试）与 `second_transaction_step_five_reuse.rs`（7 个测试）在副本上全绿（原样贴在下面，只截取汇总行）：

```
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.80s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
```

## 这条腿自己的限度

1. **"方向"一节的实测只覆盖 X3-A 一格**，X3-B（1 故障比 6 故障更坏）、X3-C/X3-D（走读失败与"回退到 A 成功"两个下游现象）都没有在打了 (a) 补丁的副本上重跑，只读了 r2 报告的原文，没有验证 r2 报告对这三格的判定准不准——只对 X3-A 做了实测，X3-B/C/D 的判定沿用 r2 报告，没有独立核实。
2. **"深化版 (a)" 留下的 65536 字节（4 槽）残差，我没有查出是什么**：没有继续加日志追踪具体是哪 4 个槽在 accounting 与 walked 之间对不上，也没有测试候选 (b)、候选 (c) 是否也有类似的"改一处不够、要连带改另一处"的隐藏耦合——只测了 (a) 的两版，没有实现 (b)、(c)。
3. **T3a 的"数值巧合"论证只覆盖 ROOT_RING_REGIONS=3、2 块盘这一组格式常量**，没有扫描"3 块盘"或"ROOT_RING_REGIONS 改成别的值"这类假设情形——这原本也不在我分到的格里（那是 X8 的攻击面），我只是在核对 T3a 时顺带看到、记录下来。
4. **副本上的所有数字都不进 kb**（按 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）——包括 T3b 的 0 违例、"方向"一节 (a) naive/深化两版的违例数字，若要写进决策文档或 kb，需要主 agent 在入库装置上重做一次。
5. **没有跑层 0 崩溃点重放**（门禁 54 号全量、2104413 个状态），也没有跑门禁 59 号的完整表（56 条全量），只跑了第 51–56 行这 6 条新增的与基线两个测试文件的全部用例；命名纪律、`check.sh`、`doc-lint.sh` 都没有跑。
6. **X4、X5、X6 之外的 X1、X2、X3、X8 一个都没判**，也没有去核第一轮判决第六节里除 C340、C332 之外的决策点——按分工这些不归我。
7. **本地攻方/本地辩方/云端攻方三条腿的报告我完全没读**（禁读清单要求），"方向"一节引用的 r2 报告是**上一轮**的产物，不是这一轮别的腿的产出，允许读。
8. 没有重新核对 D28 已定项 1 第九项「被抛弃根独占量」的定义与 `isolated_slots` 是不是同一个数（这是 X4，归本地攻方）——我在读 T1 代码时顺带看到隔离机制用的是逐条枚举分配记录做集合差，而 D28 该项文字写的是"从可达的记账树根读、不遍历树"，**这两者的口径可能不同，但这是 X4 的判据，我没有去比对"每一步上是不是同一个数"，只是提醒一下这条腿在读**。

## 没做什么

- 不判 X1（T1 隔离集是否算宽/算窄的具体历史）、X2（T1 时机/崩溃点）、X3（F_生效 回落打中的原始四格，只核了改法）、X4（D28 第九项与 isolated_slots 是不是同一个数）、X5（单层守卫）、X8（T3 扣住的攻击历史）——都不归我。
- 不替攻方腿找新反例：T3a 里提到的"循环因 ROOT_RING_REGIONS 提前退出"结构性缺口，我只核实了"今天不可达"，没有去构造让它可达的历史（那需要改变盘数或格式常量，超出这一轮"只攻这三处改法"的范围，也是 X8 的活）。
- 没有实现候选 (b)（回退候选集也用 max）与候选 (c)（只改 checker）在副本上的代码，只用 (a) 的实测结果与 r2 报告的机理描述做交叉核对。
- 没有跑门禁 54 号（层 0 全量）、门禁 59 号完整表、`check.sh`、命名纪律、`doc-lint.sh`——按 `stage-owners.tsv` 这些不在我这条腿的清单上（我没有查这张表，因为这条腿的定义与派发提示都没有要求先跑门禁阶段）。
- 原仓（`/home/fy5090/code/singlefs`）没有做任何写操作，全部改动、探针测试、变异复跑都只发生在 `/tmp/claude-1000/m2-step45-code-r3-sonnet/copy` 这份副本上。
