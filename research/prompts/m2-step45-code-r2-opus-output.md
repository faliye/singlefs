# m2-step45-code-r2 云端攻方腿（Opus）报告：五处改法本身的代价

攻击面 X1、X2、X3（X4、X5、X6 归本地攻方，没碰）。原仓只读，所有实验在副本
`/tmp/claude-1000/m2-step45-code-r2-opus/copy/`（`rsync -a --exclude target --exclude .git`，基准提交 5f9e449 之上的工作区）上跑；
**副本上量出的数照实报，不进 kb**。

## 复跑命令

```
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ <副本>/
cp research/prompts/m2-step45-code-r2-opus-model/opus_attack_m2_step45_r2.rs <副本>/crates/singlefs-harness/tests/opus_r2_attack.rs
cd <副本> && cargo test -p singlefs-harness --test opus_r2_attack -- --nocapture --test-threads 1
```

X1 那一格的 P1 对照要先按 `research/prompts/m2-step45-code-r2-opus-model/p1-contrast.patch.txt` 改
`crates/singlefs-core/src/mount.rs` 的 `mount_rollback`，跑完改回去。

| 文件 | sha256 |
|---|---|
| `research/prompts/m2-step45-code-r2-opus-model/opus_attack_m2_step45_r2.rs`（509 行） | `7a30c64686bffa4b250617759a3dbad0869a105713c1b8b7ab4a30a53d635d22` |
| `research/prompts/m2-step45-code-r2-opus-model/p1-contrast.patch.txt`（9 行） | `34d9aaacb1612dc380ef6ba1540ebed651d1ee84c8032750a063d82ef2d6c5ae` |

副本上 8 条用例全过（`test result: ok. 8 passed; 0 failed`），并且副本里既有的
`second_transaction_step_four_rollback`（5 条）与 `second_transaction_step_five_reuse`（5 条）也全过——
证明 P1 对照那次改动已经改回去了，副本与原仓同一份行为。

## 各格判定一览

| 格 | 判定 | 一句话 | 代价（副本实测） |
|---|---|---|---|
| **X1 P2 的代价** | **打中 2 条** | ① P2 把被抛弃的记录留在环里，一次落回 R_old 的恢复会把**整段**被抛弃的发布重放回来，同一段历史上 P1 一条都不放；② P2 与 D23 已定项 14 第 3 条字面「前缀末 + 1」不符，也与 C340 第三列写死的会红检查（P1 为准、P2 必红）反向 | 5 个故障：P2 重放 3 次已被回退抛弃的发布、读回被抛弃的第四版；P1 在同一格施加 0 条、读回 A |
| **X2 影子账每次挂载** | **打中 2 条** | ① 被抛弃根（谁都不再需要的旧时间线）的树表单元两份都坏 ⇒ **每一次可写挂载都开不了**，而只读恢复照常读得回文件——这条路是 S2「每次挂载都算」引进来的；② 保守读法隔离的槽被释放、回收之后离开「已分配」进「空闲」，却永远发不出去，而 D28 已定项 1 第九项按定义（两条根的已分配之差）扣不到它们 | ① 2 个字节 ⇒ 池不可写；② 逐盘 36 个槽记账算空闲、位图上发不出，其中 2 个（mkfs 实例表 50176–50177）在第九项的口径之外 |
| **X3 F_生效 做候选集** | **打中 2 条** | ① 抬 F 生效、回收、E 合法复用之后，**一个字节**翻掉盘 1 上唯一带 F 的根 ⇒ F_生效 回落到 0 ⇒ 新实例的根把 F **写回 0**（`mount.rs:460`）⇒ checker 的候选集重新含进第 0 代根与 A，而它们引用的单元已被合法盖掉 ⇒ I-2.1 / I-3.1 / I-5.1 三条一起红；② 同一格里故障越多反而越安全：盘 1 上 6 条根全坏时 F_生效 仍是 11、数据侧两条判绿 | ① 1 个字节 ⇒ 三条不变量红；② 1 个故障比 6 个故障更坏 |

三格都给了可达的历史与逐行的代码落点；每一条的四句自查写在各自小节的末尾。

## X1 P2 的代价

被判的是 S1：`crates/singlefs-core/src/mount.rs:661-666`

```rust
    let highest_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0);
    let next_counter = highest_counter + 1;
```

### X1-A（打中）：一次落回 R_old 的恢复把整段被抛弃的发布重放回来，P1 在同一格一条都不放

**历史**（副本用例 `x1_a_after_p2_a_recovery_that_falls_back_to_r_old_replays_the_whole_abandoned_run`，零策略违规）：

1. mkfs → 取号 1 → 暖机 txg 1、2 → A（txg 3、jsn 3）。
2. 同一个实例连着覆盖写三次：txg 4、5、6，jsn 4、5、6（实测环里的记录
   `[(1,1,1),(1,2,2),(1,3,3),(1,4,4),(1,5,5),(1,6,6)]`，第一项是实例、第二项 jsn、第三项 txg）。
3. 进程退出，管理员回退到 A `(1, 3)`：取号 2，D 落 **txg 7、jsn 7**（P2 = 环里最大 8… 这里最大是 6，所以 7），暖机 txg 8、jsn 8。
   jsn 4、5、6 三条被抛弃的记录**原样留在环里**。
4. 翻掉 txg 4、5、6、7、8 五个根槽各一个字节（根槽不镜像，`一次 txg 只写一个槽`，所以是 5 个故障，不是 10 个）。
5. 冷启动恢复。

**实测**：`X1-A 五个故障之后：所选根 Some((1, 6))、施加 3 条、读回 根 (InstanceGeneration(1), CheckpointTxg(3))、2600 字节、= 第四版 true`。

择根落到 A `(1, 3)`，`replay_journal`（`crates/singlefs-core/src/recovery.rs:726-731`）按同实例筛候选：

```rust
    let mut above: Vec<&JournalRecord> = records
        .values()
        .filter(|record| {
            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water
        })
        .collect();
```

jsn 4、5、6 都是实例 1、txg 都大于 3、jsn 从 A 自己那条（3）连号接上 ⇒ **三条全施加**，
管理员的回退被静默撤销，读回的是被抛弃的第四版。前缀第五条救不了：`rollback_high_water_of_root`
（`recovery.rs:426-432`）要「所选根的实例在**它自己指着的**实例表里有回退行」，
回退行写在实例 2 的表里、所选根是实例 1 的旧根 ⇒ 恒 `None`（第一轮 X4 判的那条死条款，C124 还欠着）。

**P1 对照**（同一段历史，副本上把 `next_counter` 换成 `own_record.counter + 1`）：
`X1-A 回退：新实例 InstanceGeneration(2)、D 的 (txg, jsn) = (7, 4)、暖机 [(8, 5)]`，
环里的记录变成 `[(1,1,1),(1,2,2),(1,3,3),(1,6,6),(2,4,7),(2,5,8)]`——D 与暖机把 jsn 4、5 盖掉了；
同样五个故障之后 `所选根 Some((1, 3))、施加 0 条、读回 …3000 字节、= 第四版 false`，读回的是 A 的内容，**回退保住了**。

⇒ **P2 比 P1 多丢的，在这段历史上是「整段被抛弃的发布被重放回来」**：被重放的条数 =
被抛弃时间线里与 R_old 同实例、jsn 从 R_old 那条连号接上的那一段的长度，没有上界（这一格量到 3）。
P1 在同一格恒为 0，因为新实例的第一条记录正好盖掉那一段的头一条、链当场断在实例边界上。

**这一格与 C332（`.claude/kb/checks-owed.md:303`）的关系**：C332 逐字写着
「或落回 R_old 并施加 R_old 之后被抛弃的记录，回退被静默撤销、之后已确认的写丢掉」——**现象同族**。
新的是两样：① C332 的条件写的是「回退实例的根在两块盘上都读不出（2 个故障）」，
而这一格要的是「(txg, 实例) 比 R_old 大的**每一条**可读根都读不出」，这个脚本上是 5 个；
② **P1 / P2 在这一格上判得不一样**，而 C332 行里没有这一维——它 2026-09-14 立的时候 C340 还没有。
第一轮判决第六节把 C340 记成决策点、写「代价数没有」；这一格就是那个数。

### X1-B（打中）：P2 与 D23 已定项 14 第 3 条字面不符，也与 C340 第三列写死的检查反向

`.claude/kb/decisions/23-journal的角色与格式.md:1238` 逐字：

> 3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。

回退那次的「前缀」是空的（`mount_rollback` 里 `prefix_applied: 0`，`mount.rs:673-680`），
所以「前缀末」只能指 R_old 自己覆盖的最后一条记录 = P1。今天的代码取环里最大 jsn + 1 = P2。

`.claude/kb/checks-owed.md:311`（C340）第三列（会红的检查）逐字：

> 回退之后第一条记录的 jsn = R_old 覆盖的最后一条 + 1（P1）；判别力自证：按可读链末尾 + 1（P2）实现必须红

⇒ **C340 那一行今天写的是「P1 为准、P2 必红」，而代码取 P2、变异表钉的是「接 R_old 那条之后（P1）」必红**——
两边正好反过来。C340 的第三列在第一轮改法落地时没跟着改，检索到它的人会照那一行去建检查。

⚠️ 同一条口径还有第二处：`mount_writable`（`mount.rs:556-561`）也用「环里最大 jsn + 1」，
普通挂载的前缀停在半路（点名单元验证失败、提交标记不全）时，它同样不是「前缀末 + 1」。
这一处第一轮没判过，也不在 C340 的题面里（C340 只说回退）。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | **分辨**。X1-A 在 P2 上重放 3 条、在 P1 上 0 条，同一段历史、同一组故障、只换 `next_counter` 一行 |
| 被判的系统当时看不看得到判别它的东西 | **看得到**。盘上有实例表（回退行在实例 2 的那一版里）、有环里全部记录；今天看不到只是因为前缀第五条只读所选根**自己**指着的那一版表（`recovery.rs:426-432`）。这是条款的读法问题，不是信息不在盘上 |
| 满足的是判据字面的哪一个分句 | X1 判据「触发的观测」第一个分句：**一段历史让被抛弃的记录被施加**（X1-A）。X1-B 落在判据问句里「jsn 全池最大 + 1 与 D23 第 3 条『从前缀末 + 1 接着写』字面对不对」那一问 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 反向接受条款给的是「改代码 + 补一条会红的用例与一条变异行 + 再攻一轮」。**改回 P1 在 X1-A 这一格不中，但会让第一轮 X2 那一格重新中**（P1 盖掉已提交的 B 那条记录）——两条路互斥，这一格不是「改代码」能两边都修的，要用户在 C340 上定，或者给回退行另一处落点（第一轮第六节已列的那条）。**「补一条会红的用例」这一半在今天的代码上做得到**：X1-A 那条历史就是 |

## X2 影子账每次挂载

被判的是 S2：`crates/singlefs-core/src/mount.rs:235-250`

```rust
    if shadow_ledger == ShadowLedger::On {
        let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
        for root in roots.iter().filter(|root| is_abandoned(root)) {
            // 那条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
            for record in allocation_records_under_root(devices, root)? {
```

### X2-A（打中）：被抛弃根的树表坏掉 ⇒ 每一次可写挂载都开不了，而只读恢复照常

**历史**（副本用例 `x2_a_corrupting_an_abandoned_roots_tree_table_makes_every_writable_mount_fail`）：
固定脚本到 D（A、B、重开取号 2、写行、暖机两次、C、回退到 (1, 3)、D、暖机）之后，
把**被抛弃根 C `(2, 8)` 的树表单元**两份各翻一个字节（实测落点 `[(0, 50326), (1, 50326)]`，2 个故障）。

**实测**：

```
X2-A 可写挂载失败：Recovery(UnitUnreadable { slot: SlotNumber(50326) })
X2-A 只读恢复："根 (InstanceGeneration(3), CheckpointTxg(10))、内容 3000 字节"
```

当前时间线（A 与回退之后的实例 3）一个字节都没坏，只读恢复择到 `(3, 10)`、文件原样读回；
**可写挂载却整个开不了**，而且不是一次性的——影子账在**每次**挂载的重建里算（S2 的改法本身），
被抛弃的根要到离开根环（第一版 24 槽）才不再被查 ⇒ 在那之前每一次可写挂载都撞同一堵墙。

那个 `?` 在 `mount.rs:239`；错误经 `rebuilt_allocator` → `mount_writable`（`mount.rs:563-569`）与
`mount_rollback`（`mount.rs:687-693`）原样冒出去，两条路一起关掉。

⚠️ **这条路是 S2 引进来的**：第一轮之前影子账只在 `mount_rollback` 里算，坏的是被抛弃根的账时只挡住回退，
普通可写挂载不受影响。射程从「发起回退那一次」扩到了「此后每一次挂载」。

⚠️ 同一段代码还有一处没断言的前提：`allocation_records_under_root`（`crates/singlefs-core/src/recovery.rs:399-409`）
只读分配记录树**根节点**的条目、直接 `AllocationRecord::parse`，**不看 `IndexNodeHeader.level`**。
checker 那边是看的（`crates/singlefs-checker/src/walk.rs:298`，`TREE_KIND_EXTENT | TREE_KIND_ALLOCATION | TREE_KIND_ACCOUNTING if node.level > 0`）。
第一版的分配记录树只有一个节点（20 条记录）所以走不到；树长到两层之后，内部节点的条目会被当成 20 字节的分配记录解出来，
`isolate_abandoned`（`mount.rs:243`）拿着乱槽号去 `DeviceFreeMap::isolate`，撞
`assert!(end <= self.allocated.len(), "跨度越过单元区末尾")`（`crates/singlefs-core/src/allocator.rs:193`）或者隔离错的槽。
这个单节点假设 `rebuild_version`（`recovery.rs:481-486`）本来就有，不是 S2 新加的；S2 把它的射程从「当前那一版」扩到「环里每一条被抛弃的根」。

### X2-B（打中）：保守读法隔离的槽被回收之后，记账算它空闲，而第九项按定义扣不到它

`mount.rs:240` 跳过 `is_released`，其余逐槽 `isolate_abandoned`；`DeviceFreeMap::isolate`
（`allocator.rs:190-202`）只动隔离位，**不动 `free_slots`**；`is_free`（`allocator.rs:150-155`）
`&& !self.isolated[Self::index(slot)]`。两边不同步的那一格是「隔离的槽后来被回收」。

**历史**（副本用例 `x2_b_isolated_slots_that_r_old_also_referenced_are_counted_as_free_after_reclaim`）：
固定脚本到 D → 四次覆盖写（txg 11–14）→ 抬 F 到 11 → E（txg 17）。实测逐盘：

```
X2-B 盘 DeviceIdentity(0)：记账空闲 211908、位图上真能发的 211872、隔离 36、差 36
X2-B 盘 DeviceIdentity(1)：记账空闲 211908、位图上真能发的 211872、隔离 36、差 36
```

「记账空闲」是 `DeviceFreeMap::free_slots()`（`df` 的输入），「位图上真能发的」是把单元区逐槽问 `is_free` 数出来的。
**逐盘差 36 槽 = 576 KiB**，两盘合计 1152 KiB。

其中 34 槽是被抛弃根独占的，D28 已定项 1 第九项「被抛弃根独占量」按定义扣得到；
**剩下 2 槽（mkfs 实例表 50176–50177）扣不到**——`.claude/kb/decisions/28-挂载期承诺量.md:30` 逐字：

> 上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量，两个数各从那条根可达的记账树根读、不遍历树、不加盘上字段；回退选中 R_old 那一刻算出

50176–50177 在回退那一刻**两边都有**（A 与 B 都引用 mkfs 实例表，步 4 用例的注逐字「隔离的槽数（独占 34 + 两边都引用的 mkfs 实例表 2）」），
在那个差里相消 ⇒ 第九项恒不含它们。它们后来被 D 放掉（释放代 9）、抬 F 到 11 时被
`mark_reclaimed`（`allocator.rs:212-231`：`allocated_slots -= span; free_slots += span`）回收，
离开「已分配」进「空闲」，**却仍被隔离、永远发不出去**。
⇒ 准入不等式八项一项都扣不到这 2 槽/盘，**可用被高估正好这么多**，而这正是 C318（`.claude/kb/checks-owed.md:296`）要拦的那类高估。

这个数不是常数 2：它等于「R_old 与被抛弃根都引用、且在被抛弃的根离开根环之前被释放并回收」的槽数，
随 R_old 那棵树被覆盖写换掉多少而长。保守读法（S2 取的）才有这一项；
用户 2026-09-16 定的窄读法（`.claude/kb/decisions/23-journal的角色与格式.md:1209` 逐字
「查账的集合 2026-09-16 用户定案取窄读法：按主语「被抛弃时间线的根」读，**仍被有效根引用的槽不在其内**」）
按定义这一项恒 0，因为它压根不隔离两边都引用的槽。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | **分辨**。X2-A 只在「影子账在每次挂载的重建里算」这条臂上出现（第一轮之前只在回退那一次算，普通挂载不中）；X2-B 只在保守读法上出现（窄读法不隔离 R_old 也引用的槽，差恒 0） |
| 被判的系统当时看不看得到判别它的东西 | **看得到**。X2-A：坏掉的那个单元只被被抛弃的根引用，最新根指着的实例表就说得出哪些根被抛弃了，跳过读不出账的那条根需要的信息全在盘上。X2-B：隔离位与空闲计数在同一个 `DeviceFreeMap` 里，差是现成的（`isolated_slots()` 已经报出来了，`mount.rs:414-418`） |
| 满足的是判据字面的哪一个分句 | X2 判据「触发的观测」三个分句：X2-A 中第三句 **一个合法镜像挂载开不了**；X2-B 中第二句 **一个谁都不引用的槽被永远隔离**（50176–50177 在当前账里已经被释放并回收，只有被抛弃的 B 还引用着） |
| 跑前条款给的每个改法在打中的那几格上还中不中 | X2-A：把 `allocation_records_under_root` 的 `Err` 改成「跳过那条被抛弃的根」在这一格上**不中**（挂载开得了），代价是那条根引用的槽不再被隔离——要连着补一条「读不出账的被抛弃根怎么办」的条款，今天没有。X2-B：**改成用户定的窄读法在这一格上不中**（差恒 0）；而窄读法会让第一轮 X3 打中的那格（被抛弃根与 R_old 共享的槽被复用）重新暴露——两个改法各修一格，都不同时修另一格，这一格要用户在「影子账的读法」那个决策点上定 |

## X3 F_生效 做候选集

被判的是 S3。压着的条款 `.claude/kb/decisions/16-发布语义.md:375-376` 逐字：

> | 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |
> | 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |

代码：`crates/singlefs-core/src/recovery.rs:351-371`（`effective_rollback_floor`）、
`mount.rs:627-638`（候选集的下界）、`mount.rs:460`（`rollback_floor: start.effective_floor`，新实例的根写 F_生效）、
`crates/singlefs-checker/src/walk.rs:786-799`（checker 的候选集按**最新根自己带的** F 收）。

### X3-A（打中）：一个字节让 F 回落到 0，三条不变量一起红

**历史**（副本用例 `x3_losing_the_only_floor_carrier_after_a_legal_reuse_writes_the_floor_back_to_zero_and_the_checker_goes_red`，零策略违规）：

1. 固定脚本到 D → 四次覆盖写（txg 11–14）→ 抬 F 到 11（空发布 txg 15 落盘 0、txg 16 落盘 1，生效）。
2. E（txg 17）**合法**复用 50178（mkfs 树表那一槽，释放代 3 ≤ F = 11）。此刻 checker 全绿（实测 `违例 []`）。
3. 翻掉 **txg 16 的根槽一个字节**——盘 1 上唯一带 F = 11 的根（盘 1 的根是 txg 1、4、7、10、13、16，只有 16 带 11）。1 个故障。
4. 重开可写挂载。

**实测**：

```
X3-A 一个字节之后：新实例的根写 F = 0；违例 ["I-2.1", "I-3.1", "I-5.1"]
  I-2.1 Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")
  I-3.1 Violated("盘 0：记账的已分配 Some(1474560)，遍历全部有效根得到 1425408")
  I-5.1 Violated("盘 0：树表单元（槽 50178 跨 1）与 数据单元（槽 50178）重叠")
```

**机理**：盘 1 少了那条根之后 `effective_rollback_floor` 的 min 里盘 1 那一项掉回 0 ⇒ F_生效 = 0；
`establish_instance` 把新实例的根的 `rollback_floor` 写成 F_生效（`mount.rs:460`）⇒ **最新根带的 F 从 11 变成 0**；
checker 的候选集按最新根自己的 F 收（`walk.rs:786-795`）⇒ 第 0 代根与 A 重新进候选集，
而它们引用的 mkfs 树表单元（50178）已经被 E 合法盖掉 ⇒ 三条一起红。

这正是步 5 那条必红用例
`reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`
（C22 的必红）要造的那个状态——**只是这一次没有任何人违反政策，只有一个坏字节**。
抬 F 买到的东西被一个坏槽整个退掉了，而且退的是**已经发生过的**复用：复用那一刻它是合法的，
事后被重新判成非法。步 5 的决策点里那一句「F 只在一块盘上时新实例的根写 F_生效（把 F 写低）是不是条款的意思」
（`.claude/kb/milestone/02-second-txn.md:189`）问的就是这一步，而它没记这个后果。

`.claude/kb/invariants.md:107`（I-2.1 状态列）逐字写着这条读法的射程：
「2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判」。
F 能**往回掉**这件事不在那句话的射程里。

### X3-B（打中）：一个故障比六个故障更坏

同一个脚本，只换损坏集合（副本用例 `x3_b_damaging_only_the_carrier_is_worse_than_damaging_every_root_on_that_device`）：

```
X3-B 只坏载体（1 个故障）：新实例的根写 F = 0、违例 ["I-2.1", "I-3.1", "I-5.1"]
X3-B 盘 1 全部根都坏（6 个故障）：新实例的根写 F = 11、违例 ["I-3.1"]
```

`effective_rollback_floor`（`recovery.rs:357-370`）只把**有可读根的盘**放进 `highest_per_device`，
一条根都读不出的盘不进 min（函数头的注逐字「一块盘上一条根都没有就不算它」）。
⇒ 盘 1 六条根全坏时它整块不算「幸存盘」，F_生效 = 11，数据侧两条判绿；
只坏那一条载体时盘 1 还算幸存、贡献 0，F_生效 = 0。**故障单调性反了**。

对照用例 `x3_b_control_damaging_every_root_on_device_one_without_any_reuse`（同样 6 个故障、不做 E 那次复用）实测
`新实例的根写 F = 11、违例 [("I-3.1", Violated("盘 0：记账的已分配 Some(1048576)，遍历全部有效根得到 819200"))]`
⇒ 6 个故障那一格的 I-3.1 是**丢掉 6 条根本身**造成的（并集少了那些根的引用），与复用无关；
数据侧的 I-2.1 / I-5.1 在那一格确实是绿的。所以「1 个故障比 6 个更坏」这句成立的是 I-2.1 与 I-5.1 这两条。

### X3-C / X3-D（打中的下游）：F 回落之后退到那些根会怎样

判据问句里的「回退到第 0 代根、到 A 各会怎样」，副本上逐格量了：

```
X3-C 退到 (0, 0)：Recovery(UnitUnreadable { slot: SlotNumber(50178) })
X3-C 退到 (1, 3)：成功，新实例 InstanceGeneration(4)
X3-D E 落 50178、下一版落 50180
X3-D 翻字节之前的违例：[]
X3-D 退到 A 被拒：Recovery(UnitUnreadable { slot: SlotNumber(50180) })
```

三样：

1. **不是走读 panic，也不是读到坏单元**：位置条目带校验和，被盖的单元读不出来，`rebuild_version` 报
   `UnitUnreadable`。就正确性而言这是安全的一边。
2. **但候选集在说谎**：`mount_rollback` 先做完两道候选检查（`mount.rs:633-648`），
   报给管理员的却是一个指到具体槽号的底层错（`UnitUnreadable { slot: 50178 }`），
   不是「这条根在 F 之下 / 它的单元已经被回收复用」。管理员按候选集选的根退不过去，而错误信息说不出为什么。
3. **X3-C 第二行更糟：退到 A 成功了**。那一格 A 的数据单元 50180 还没被拿走（E 拿的是 50178），
   于是回退跑完、建起实例 4；再多复用一步（X3-D，下一版拿 50180）之后同一个回退就被拒。
   **同一条根在候选集里的可退性取决于它的单元有没有恰好被拿走**，而候选集判据里没有这一维。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | **分辨**。X3-A / X3-B 只在「候选集与新根的 F 都用 F_生效」这条臂上出现：按最新根自己的 F 走（S3 之前的写法），txg 16 坏掉之后最新根 txg 17 仍带 F = 11，候选集不会变宽、新根也不会把 F 写低。代价是第一轮正推腿判的那条「窄化」回来 |
| 被判的系统当时看不看得到判别它的东西 | **看得到**。盘 0 上两条根带着 F = 11，`effective_rollback_floor` 自己就读到了；「F 只许涨不许落」这条要的输入（上一条根带的 F）在 `previous.root.rollback_floor` 里现成。今天是主动取 min 把它丢掉的 |
| 满足的是判据字面的哪一个分句 | X3 判据「触发的观测」逐字「一个候选根退过去之后走读失败或写出错的东西」：X3-C / X3-D 是**走读失败**那一半（`UnitUnreadable`）。X3-A / X3-B 落在判据问句的第二问「两边不一致的那段时间里 I-2.1 / I-3.1 判什么」——答案是：不一致那段时间 checker 判绿（它用 11），**下一次可写挂载把两边拉平到 0，三条一起红** |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 反向接受条款是「改代码 + 补会红的用例与变异行 + 再攻一轮」。候选的改法各自在哪一格：**(a) 新实例的根写 max(上一条根的 F, F_生效)**——X3-A / X3-B 不中（F 不回落、checker 候选集不变宽），X3-C / X3-D 仍中（回退候选集本身还是按 F_生效 收）；**(b) 回退候选集也用 max(…)**——X3-C / X3-D 不中，但第一轮正推腿判 S1 的那条「窄化」回来；**(c) 只改 checker（候选集用 F 的历史最大值）**——X3-A 的 I-2.1 / I-5.1 不中，I-3.1 仍中（并集还是按候选集算），X3-C / X3-D 全中。**三个改法没有一个把四格都修掉**，写条款时要各写明修的是哪一格 |

## 没打中的形状

| 试的形状 | 结果 | 取样范围 |
|---|---|---|
| X1「所选根是 (1, 4)（B 的根在、之后都坏）时停在哪」 | **没有新的一格**：jsn 5 是实例 2 的记录，`replay_journal` 的候选筛选（`recovery.rs:726-731`）按 `record.instance == root.instance` 先把它挡掉 ⇒ 0 条施加。但所选根 (1, 4) 自己就是被抛弃的根，读回的是 B 的内容——这就是 C332 的第一条臂，不是 P2 新添的 | 按代码推，没跑；固定脚本上要 6 个故障 |
| X1「被抛弃实例 2 的记录在什么所选根下会被接上」 | **只有所选根本身是实例 2 的根时**（同一条筛选），那已经是 C332 第一条臂。P2 没有新添一格 | 同上，按代码推 |
| X1「环绕圈够不够得着」 | **够不着，而且不是新的一格**：journal 环 768 MiB ÷ 4096 = 196608 槽，根环 24 槽——被抛弃的根 24 次发布就离开根环，而它们的记录还要在 journal 环里待 196608 个 jsn。P2 不盖任何槽 ⇒ 锚点不会错位；一条被抛弃的记录要被接上，仍然只有「它的 jsn 恰好是所选根那条的后继」这一条路，也就是 X1-A | 只做了算术；固定脚本到 txg 17，没跑到 24 次发布 |
| X2「`newest_table = None` ⇒ 按表判的被抛弃根一条都不隔离」 | **这一轮构造不出来**。`mount_rollback` 在 `mount.rs:624-625` 先 `instance_table_of_root(newest_root)`、读不出直接 `InstanceTableMalformed`（第一轮 X1② 那一格）；`mount_writable` 里最新根就是所选根，同一个实例表单元读不出时 `rebuild_version` 也读不出、整条挂载先失败。唯一的缝是「施加记录之后 effective_root 指向另一份实例表」，而 `replay_journal` 施加记录时 `rebuilt.instance_table` 原样照抄所选根的（`recovery.rs:797`）⇒ 两者恒是同一个单元 | 想了三种形状（最新根的表坏、施加记录后换表、回退目标与最新根各一份表），都被上面两条挡住；没跑用例 |
| X2「有没有一条被抛弃根引用、而每条被抛弃根账里都是已释放的槽」 | **没找到**。分配它的那条根只要还在环里且可读，它那一版账里那条记录就是未释放 ⇒ 被 `mount.rs:237-248` 扫到。漏的条件是「分配它的被抛弃根已经离开根环（24 槽）或它的根槽读不出」，而那两种情形下那条根本来就不再能被恢复选中 | 按代码推 + 固定脚本上逐槽核过（36 个隔离槽逐一对得上步 4 用例的 34 + 2）；没跑到 24 次发布那一格 |
| X3「根槽写失败落到别处或区域映射变了」 | **没打中**。`visit_valid_roots`（`recovery.rs:268-282`）逐槽读、不核「这个槽号与根自己的 txg 算出来的位置对不对」，但写者只按 `target_for_publish` 写 ⇒ 今天造不出「根落在别处」的镜像；区域映射 `region_devices` 读根与算盘用的是同一份（都来自超级块），两边不会错位 | 读了 `visit_valid_roots`、`effective_rollback_floor`、`target_for_publish` 三处；没构造 |

## 这条腿自己的限度

1. **副本上的数不进 kb**（`.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）。
   报告里每个数都是 `/tmp/claude-1000/m2-step45-code-r2-opus/copy/` 上跑出来的。
2. **我提的改法都被攻过零轮，而且没在副本上量过**：X3 四句表里的 (a)(b)(c) 三个改法、X2-A 的
   「跳过读不出账的那条被抛弃根」、X2-B 的「回到用户定的窄读法」——全是按代码推的，一条都没实现、没跑。
3. **X1-A 用的是我自己写的脚本**（同实例连发三版、再回退），不是仓里的固定脚本。
   固定脚本上同一现象要 7 个故障（实例 3 两条 + 实例 2 四条 + B 一条），我没跑那一格，只推了。
   我这条脚本的 5 个故障也不是下界——没做最小故障集的搜索。
4. **故障注入只有「翻一个字节」**（等价于坏槽）。写失败、丢整块盘、撕裂写都没建模；
   E139 的故障模型里有这几样，我一个都没覆盖。
5. **全是进程内 + 重开的用例，不是层 0 崩溃点重放**。崩在一次发布中间的那一整类状态没碰到；
   层 0 全量（2104413 个状态）归门禁 54 号，我没跑。
6. **checker 只跑 `check_pool_image`**，没跑 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 那批；
   I-2.1 / I-3.1 / I-5.1 三条红是 checker 自己报的，我没独立算过第二遍。
7. **X3-B 那条「1 个故障比 6 个更坏」只对 I-2.1 与 I-5.1 成立**：I-3.1 在 6 个故障那一格照样红，
   对照用例证明那是丢掉 6 条根本身造成的（并集少了引用），与复用无关。这条对照我跑了，但没再往下追
   「丢掉一块盘全部根 ⇒ I-3.1 红」本身该不该，那是另一格。
8. **没判 X4、X5、X6、X7**（分工上归别的腿），也没判第一轮判决第六节里除 C340、C332 之外的决策点。

## 没做什么

- 没跑 `gate.sh`，没跑层 0 全量、QEMU（门禁 55 号）、herd7（57 号）、变异表（33 / 59 号）。
  `.claude/gate.d/stage-owners.tsv` 里**没有登记给 `three-way-attack` 的阶段**（按 `.claude/agent-common.md` 给的 awk 跑，输出为空）。
- **原仓一个已有文件都没改**，只新建了 `research/prompts/m2-step45-code-r2-opus-output.md` 与
  `research/prompts/m2-step45-code-r2-opus-model/` 下两个文件；没做任何 git 写操作。
- 没读禁读清单里的文件（这一轮别的腿的输出、`m2-step45-code-r2-sonnet.md`、`-local-*.md`）。
- 第一轮攻方腿报告 `m2-step45-code-r1-opus-output.md` 只用来避开角度，没有一格与它重复：
  它攻的是「原来的代码错在哪」（P1 盖记录、影子账只在回退里算、前缀第五条是死条款、候选集用最新根的 F），
  这一轮攻的是那四处改法**改完之后**的代价。
- 没在原仓跑过 `cargo`（`.claude/agent-common.md`「不编译 Rust，除非定义明写要做」——派发提示明写了要在副本上跑，
  所有编译都在副本里，`nice -n 19`）。开跑前 `ps` 看到两个 `second_transact` 各占 100% CPU、两个 `ray::RayWorkerP` 各 27%，
  load average 2.40；派发提示允许照常跑，全程 `nice -n 19`。
