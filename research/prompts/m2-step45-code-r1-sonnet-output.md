# m2-step45-code-r1：云端正推腿报告

立场：核「代码做的是不是条款说的」，逐格给代码路径 + 附录原文 + 判定。不判攻方腿的格（X1–X9）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| S1 | **不一致（窄化，见细节）** | 候选集的三重判据（环内、实例表有效、F 之下）代码全做了，但 F 那一半代码用「最新根自己的 F 字段」，条款字面要的是 D16 已定项 1 定义的 `F_生效`（各幸存盘所带 F 最大值的最小值），两者只在 F 已「生效」（全盘落齐）时重合 |
| S2 | 一致 | `next_counter = own_record.counter + 1`（P1）与记录核对器的 `record_lost` 逐句对上 C340 取 P1 与 D23 已定项 14 第 3 条 |
| S3 | 一致 | 回退行 `is_rollback: true, applied_transaction_high_water: 0`；中间实例行 `(i, 0, 0)`，与同一次发布一起写 |
| S4 | 一致 | 窄读法（只隔离 R_old 不引用而更新根引用的槽）、只住内存、只让 `is_free` / `lowest_empty_segment` 绕开，逐句对上 |
| S5 | **一致，附一句未覆盖的字面** | `high_water == 0` 分支精确覆盖「W=0 一条不施加」；「事务号 0 的记录也不施加」那半句在 W>0 时代码没有独立判断，现状历史下不可观测出差异（见细节） |
| S6 | 一致（代码是条款的第一版特例化，kb 已言明理由） | `reclaim_released_up_to(floor)` 直接比 `floor`，不算 `max(F_生效, 环里最旧有效根)`；kb 现状段自陈「第一版环里最旧有效根恒 0，floor 就是 F_生效」 |
| S7 | 一致（条款本身标「预想」，不算不一致） | ceiling = min(每盘最新有效根, 第4新非空)，非空按 `record.transaction != 0`；kb 明确标注这是决策点，非已定条款 |
| S8 | 一致 | 先回收再推空发布、写进新实例根的 `effective_floor`；「F 写低」是 kb 自己标的决策点，不算代码违反条款 |
| S9 | 一致 | I-3.1 并集与 I-2.1 判定的执行路径都只覆盖 `walk_root` 走到的候选集（`abandoned`/`below_floor` 两个过滤器） |
| S10 | 一致，且与 E142 第十次跑产物、单测三方吻合 | `format_time_tree_table_to_release` 与产物 `defer_queue_per_device=16384`、`mkfs_generation_records=2`、单测 `generation==CheckpointTxg(3)` 计 18 逐字对上 |
| S11 | 一致 | 测试文件里的 `versions` 表逐条构造与 kb 现状描述（回退后 (3,9)/(3,10) 读第一次内容、抬 F 两条根读第四版、E 读自己）字面相同 |
| S12 | 一致 | `chain_start_txg_without_anchor = root.checkpoint_txg + 1`，仅在 `root_own_record_counter` 为 `None` 时启用 |
| X10 | 见细节：milestone 与 layout/01-first-txn 现状段与代码对得上；invariants.md 的 I-3.1/I-2.1 读法与 walk.rs 逐字对上 | — |

## 各格判定的分工与限度（先声明）

这条腿只做「代码 vs 附录原文」的逐句核对，不构造历史、不判攻方腿的格（X1–X9）。判定里出现的"决策点"标注全部来自 kb 自己（`.claude/kb/milestone/02-second-txn.md` 步 4 / 步 5 的"决策点"清单），不是这条腿新发现的空白。

## S1：回退候选集

**代码怎么做**（`crates/singlefs-core/src/mount.rs`，`mount_rollback`）：

- 目标根不在环里：`mount.rs:537-543`
  ```rust
  let target_root = roots
      .iter()
      .find(|root| {
          root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg
      })
      .copied()
      .ok_or(MountError::RollbackTargetNotInRing(target))?;
  ```
- F 之下的根：`mount.rs:547-552`
  ```rust
  if target.checkpoint_txg < newest_root.rollback_floor {
      return Err(MountError::RollbackTargetNotACandidate {
          target,
          reason: "txg 低于回退下界 F",
      });
  }
  ```
- 按最新根实例表判有效：`mount.rs:545-546`、`553-562`
  ```rust
  let newest_table = instance_table_of_root(&*devices, &newest_root)
      .ok_or(MountError::InstanceTableMalformed)?;
  ...
  if newest_table
      .rows
      .iter()
      .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)
  {
      return Err(MountError::RollbackTargetNotACandidate {
          target,
          reason: "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
      });
  }
  ```
- 自己那条记录读不出：`mount.rs:563-569`
  ```rust
  let own_record = records
      .values()
      .find(|record| {
          record.instance == target.instance && record.checkpoint_txg == target.checkpoint_txg
      })
      .cloned()
      .ok_or(MountError::RollbackRecordUnreadable(target))?;
  ```

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1206`，附录一整段）：

> 候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）

`.claude/kb/decisions/16-发布语义.md:361`（D16 已定项 1 表格「回退候选集」行）：

> 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条）

`F_生效` 的定义在同一份文件的「生效」行（`.claude/kb/decisions/16-发布语义.md:361`）：

> 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值

**判定：不一致（窄化）**。

- 「按实例表判仍然有效」「(i,T) 可选 ⟺ 无 i 的行或 T ≤ Ti」两句：代码逐字对上（`mount.rs:553-562`）。
- 「txg ≥ F_生效」这半句：条款把 `F_生效` 定义成一个跨设备的量（各幸存盘所带 F 最大值的最小值），有专门的函数 `recovery::effective_rollback_floor`（`recovery.rs:351-371`）实现这个公式。而 `mount_rollback` 在候选集判定这一步（`mount.rs:547`）用的是 `newest_root.rollback_floor`——**最新那一个根自己携带的 F 字段**，不是调用 `effective_rollback_floor` 算出来的那个跨设备量。这两个量只在「F 已经生效」（每块盘上最新根都带同一个新 F）时才必然相等；F 正在被抬、还没推满全部区域时，`newest_root.rollback_floor` 与 `effective_rollback_floor()` 可能不同（前者是单根字段，后者取的是"各幸存盘所带 F 最大值"的最小值）。

**什么现象会推翻这条判定**：把 `mount_rollback` 里的 F 检查换成显式调用 `effective_rollback_floor(devices, ...)` 之后重新核，若两处的值在所有可达状态下逐位相同（即找不出任何一段历史让 `newest_root.rollback_floor != effective_rollback_floor()`），说明这条"不一致"只是写法上的差异、语义等价，判定应改为"一致（等价简化）"。反之若能构造出一段历史（例如抬 F 只推进了一部分区域后崩溃、下一次挂载在推满之前就发起回退）让两个量不同且导致候选集判定结果不同，就坐实了这条不一致是真实缺口，不只是写法问题——这正是 X6（上限与生效）分到的攻击面，这条腿不做历史构造。


## S2：P1 与记录槽复用

**代码怎么做**：

- 新实例第一条 jsn 取 R_old 自己那条记录 + 1：`mount.rs:570`
  ```rust
  let next_counter = own_record.counter + 1;
  ```
- 记录核对器把「合法覆盖」与「记录流有洞」分开：`crates/singlefs-harness/src/crash.rs:481-492`
  ```rust
  let record_lost = |index: usize| {
      let write = &image.writes[index];
      !in_place(index)
          && !image.writes[index + 1..]
              .iter()
              .enumerate()
              .any(|(later_offset, later)| {
                  later.device == write.device
                      && later.offset == write.offset
                      && in_place(index + 1 + later_offset)
              })
  };
  ```
  注释（`crash.rs:479-480`）：「一份记录『丢了』= 它不在盘上，而且没有更晚的写落在同一个位置上；回退之后新实例从 R_old 那条记录之后接着写 jsn（C340 取 P1），被抛弃发布的记录槽被后来的记录合法覆盖，不是记录流有洞。」

**条款原文**：

`.claude/kb/checks-owed.md:311`（C340 整行，附录一）：

> C340 | 回退之后记录链从哪条之后接没有定义 | D23（journal 的角色与格式） 已定项 14 回退段（23:1206）只写「不施加 R_old 之后的任何记录」，切换段（23:1233）写「链从所选根覆盖的最后一条记录之后接」；回退之后新实例的第一条记录接在哪（R_old 覆盖的最后一条 + 1，还是可读链末尾 + 1）要从切换那一段借「所选根」推出来，回退自己那一段没有一句 | 回退之后第一条记录的 jsn = R_old 覆盖的最后一条 + 1（P1）；判别力自证：按可读链末尾 + 1（P2）实现必须红 | D23（journal 的角色与格式） 已定项 14 回退段补一句 | C143（inode 号水位在回退后会退回去重发） 第二轮攻方腿（H10）、第三轮辩方腿复核

`.claude/kb/decisions/23-journal的角色与格式.md:1206`（已定项 14 跨实例边界那一段第 3 条，附录一）：

> 计数器全池接着走：新实例从前缀末 + 1 接着写、不归零。

**判定：一致**。`mount.rs:570` 取的正是 R_old 自己那条记录（`own_record`，由 `mount.rs:563-569` 精确按 `(target.instance, target.checkpoint_txg)` 查到）的 `counter + 1`，逐字等于 C340 的「P1：R_old 覆盖的最后一条 + 1」——R_old 自己那条记录就是它覆盖的最后一条（回退不施加它之后任何记录）。C340 的判别力自证要求「按 P2（可读链末尾 + 1）实现必须红」：`crash.rs` 的 `record_lost` 把「被后来的记录合法覆盖」排除在「记录流有洞」之外，这正是 P1 才需要的核对逻辑——若代码真按 P2 接（取可读链末尾），R_old 之后各被抛弃发布的记录槽不会被新实例覆盖，`record_lost` 那条排除分支就不会被真正触发。这条腿没有跑变异表验证"改成 P2 之后 record_lost 会不会误判"，这一步留给攻方腿或 crash-verifier。

**什么现象会推翻这条判定**：读到 `mount_rollback` 里 `next_counter` 不是从 `own_record.counter` 算出，或 `record_lost` 的排除条件与 `own_record` 覆盖范围不一致（比如去掉 `later.offset == write.offset` 那个条件、变成任何晚发生的写都算覆盖），就会推翻"一致"。


## S3：回退行与中间实例行同一次发布

**代码怎么做**：

- `mount.rs:613-618`（`mount_rollback` 构造 `previous_row`）：
  ```rust
  let previous_row = PreviousInstanceRow {
      instance: target.instance,
      selected_root_txg: target.checkpoint_txg,
      applied_transaction_high_water: 0,
      is_rollback: true,
  };
  ```
- `mount.rs:354-373`（`establish_instance` 写行，回退行与中间实例行同一循环、同一次 `publish_version`）：
  ```rust
  let first_row_instance = start.previous_row.instance.0.max(1);
  for row_instance in first_row_instance..instance.0 {
      let row = if row_instance == start.previous_row.instance.0 {
          InstanceRow {
              instance: InstanceGeneration(row_instance),
              selected_root_txg: start.previous_row.selected_root_txg,
              applied_transaction_high_water: start.previous_row.applied_transaction_high_water,
              is_rollback: start.previous_row.is_rollback,
          }
      } else {
          InstanceRow {
              instance: InstanceGeneration(row_instance),
              selected_root_txg: CheckpointTxg(0),
              applied_transaction_high_water: 0,
              is_rollback: false,
          }
      };
      rows_written.push(row);
      table_after.rows.push(row);
  }
  let row_publish = publish_version(... InstanceTablePlan::Rewrite(table_after.to_records()) ...)?;
  ```
  两行都在进入 `publish_version` 之前推进 `table_after.rows`，随同一次 `row_publish` 一起写出。

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1206`，附录一）：

> 在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布……）；……回退与它的第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效，崩了就重做

`.claude/kb/decisions/18-块里携带什么信息.md:778`（已定项 11 实例表 kind 0 行字段表，附录一）：

> `kind` = 0 行记录：`kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1（bit0 = 回退行，其余位恒 0、非 0 拒收）| 预留 66`

**判定：一致**。回退行的 `(instance=r_old, selected_root_txg=T_old, applied_transaction_high_water=0, is_rollback=true)` 对应条款的 `(r_old, T_old, 0, 回退位)`；中间实例行 `(instance=i, selected_root_txg=0, applied_transaction_high_water=0, is_rollback=false)` 对应 `(i, 0, 0)`。两行都在 `establish_instance` 里进入同一次 `row_publish`（`mount.rs:374-390`），与条款「回退行、这次发布的单元与回退根同一次发布」逐句相符。「回退行的 W 恒 0」这句 kb 自己标成"实做时定下的（标预想、交用户）"（`.claude/kb/milestone/02-second-txn.md` 步 4 现状段），代码里 `applied_transaction_high_water: 0` 写死为常量，与该标注一致（这是"条款没说、实做定下"的情形，不是"不一致"）。

**什么现象会推翻这条判定**：读到回退行与中间实例行分两次 `publish_version` 调用写出（即中间存在一次可能崩溃的窗口），或回退行的 W 字段不是硬编码 0 而是取了某个非零值。


## S4：影子账（窄读法）

**代码怎么做**（`mount.rs:589-612`）：

```rust
if shadow_ledger == ShadowLedger::On {
    let own_slots: BTreeSet<(u32, u64)> = previous
        .allocation_records
        .iter()
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    for root in &roots {
        if (root.checkpoint_txg, root.instance) <= (target.checkpoint_txg, target.instance) {
            continue;
        }
        for record in allocation_records_under_root(&*devices, root)? {
            let key = (record.device.0, record.slot.0);
            if own_slots.contains(&key) || !isolated.insert(key) {
                continue;
            }
            allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));
        }
    }
}
```

只住内存、不进已分配 / defer / 空闲：`allocator.rs:115-119`（`DeviceFreeMap` 的 `isolated` / `isolated_per_segment` / `isolated_slots` 字段，注释「只住内存、不进记账行」）；只让用户数据落点与开放段绕开：`allocator.rs:150-155`（`is_free` 判 `!self.isolated[...]`）与 `allocator.rs:300-313`（`lowest_empty_segment` 判 `self.isolated_per_segment[*segment] == 0`）。被抛弃根独占量逐盘报数、不进记账行：`mount.rs:341-345`（`isolated_slots_per_device` 只放进 `MountOutput`，不写进 `PublishPlan` 或记账条目）。`ShadowLedger::Off` 只在测试里出现（`grep -rn "mount_rollback(" crates/ | grep -v /tests/` 零命中，见小节末尾"没做什么"）。

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1206`，附录一）：

> 分配器多查环里每一个可读根的账、只隔离其中只被被抛弃根引用的槽——查账的集合 2026-09-16 用户定案取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西

`.claude/kb/decisions/28-挂载期承诺量.md:16`（已定项 1 第九项，附录一）：

> 第九项「被抛弃根独占量」……上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量，两个数各从那条根可达的记账树根读、不遍历树、不加盘上字段；回退选中 R_old 那一刻算出、只住内存、按设备算

**判定：一致**。代码里 `own_slots`（R_old 自己的分配记录）与 `isolated`（要隔离的集合）的构造：遍历 `(txg,instance) > (target.checkpoint_txg, target.instance)` 的每个可读根（"环里每一个可读根"）、取它的分配记录（`allocation_records_under_root`）、排除 `own_slots` 里已有的（"R_old 也引用的槽不在其内"），逐句对上"窄读法"的定义。只住内存（`isolate_abandoned` 只改 `DeviceFreeMap` 的运行时字段，不写盘、不进 `PoolAllocator::record`）与"上界从两条根各自可达的记账树读、不遍历树"（`allocation_records_under_root` 只读树表条目里那一根指针再读一个节点，不遍历整棵树，`recovery.rs:377-410`）也都对上。

**什么现象会推翻这条判定**：`isolated_slots_per_device` 被写进某条记账行、或 `isolate` 被用在 `own_slots` 命中的槽上（会撞 `allocator.rs:192-195` 的断言 panic）、或 `ShadowLedger::Off` 出现在非测试调用点。


## S5：前缀第五条

**代码怎么做**：

- `recovery.rs:426-432`（`rollback_high_water_of_root`，只读所选根自己指着的表）：
  ```rust
  pub fn rollback_high_water_of_root(reader: &dyn PoolReader, root: &RootRecord) -> Option<u64> {
      instance_table_of_root(reader, root)?
          .rows
          .iter()
          .find(|row| row.instance == root.instance && row.is_rollback)
          .map(|row| row.applied_transaction_high_water)
  }
  ```
- 调用点 `mount.rs:465`：`rollback_high_water_of_root(&*devices, &chosen_root)`（`chosen_root` 就是所选根）。
- `recovery.rs:756-762`（`replay_journal` 前缀第五条判断）：
  ```rust
  if let Some(high_water) = rollback_high_water {
      if high_water == 0 || record.transaction > high_water {
          break;
      }
  }
  ```

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1206`，附录一，「前缀判定的完整口径是六条」一段）：

> 所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的 W 为止（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2，2026-09-05：否则一次落在 R_old 上的普通恢复会把被回退抛弃的那段时间线整段重放回来，管理员的回退被静默撤销）

**判定：一致，附一句未覆盖的字面**。

- "只读所选根自己指着的实例表"：`rollback_high_water_of_root` 的入参 `root` 就是 `chosen_root`，函数体只调用 `instance_table_of_root(reader, root)` 一次、不看别的根——一致。
- "只施加到 W 为止"：`record.transaction > high_water` 时 `break`——一致。
- S1 判据表里写的括注「W = 0 一条不施加、事务号 0 的记录也不施加」：`high_water == 0` 那一支精确覆盖"W=0 时一条都不施加"（第一条记录进来就无条件 `break`，不看它自己的 `record.transaction` 是多少）。但"事务号 0 的记录也不施加"若读成一条独立于 W 的规则（即：不管 W 是多少，凡是 `record.transaction == 0` 的记录都不该被这条前缀第五条施加），代码没有为它单独写判断——只有 `record.transaction > high_water` 这一个比较，`transaction == 0` 且 `high_water > 0` 时 `0 > high_water` 为假，不会被这条规则挡下。
  这句在当前实现能不能被观测出差异，取决于「一个实例自己的记录链里，会不会在某条 `transaction > 0` 的记录之后又出现一条 `transaction == 0` 的记录」——`establish_instance`（`mount.rs:335-446`）里暖机（空发布，`transaction: 0`）总是在取号之后、写行之后立刻做完，不会在真实文件事务（`transaction >= 1`）之后再插入一条空发布，所以在**这条腿看到的代码路径**里这个次序不会发生，字面上的缺口不可观测。

**什么现象会推翻这条判定**：读到某处在一个实例已经发出 `transaction >= 1` 的记录之后又发出一条 `transaction == 0` 的记录（比如后续里程碑给"切换"或"抬 F"加了在真实事务之后插入空发布的路径），且那条记录在有回退行、`high_water > 0` 的场景下被 `replay_journal` 施加了——那时"事务号 0 的记录也不施加"这半句才会从"字面缺口、不可观测"变成"真的不一致"。


## S6：可再分配与回收

**代码怎么做**（`allocator.rs:511-540`）：

```rust
pub fn reclaim_released_up_to(&mut self, floor: CheckpointTxg) -> Vec<Placement> {
    let mut reclaimed_now = Vec::new();
    let candidates: Vec<AllocationRecord> = self
        .records
        .iter()
        .filter(|record| record.is_released && record.generation <= floor)
        .copied()
        .collect();
    for record in candidates {
        let key = (record.device, record.slot);
        if !self.reclaimed.insert(key) {
            continue;
        }
        let device_map = self.devices.iter_mut().find(|device_map| device_map.device == record.device)
            .expect("分配记录的盘在池里");
        device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));
        ...
    }
    reclaimed_now
}
```

`mark_reclaimed`（`allocator.rs:213-232`）：位图清、`allocated_slots -= span`、`deferred_slots -= span`、`free_slots += span`，runs 增量按左右邻居更新。复用时记录改写而非追加：`record()` 函数 `allocator.rs:411-438`，命中 `reclaimed` 集合时走改写分支（`existing.span_slots = ...; existing.generation = ...; existing.is_released = false`），否则才 `push` 新记录；两条分支下槽号在每盘都不重复（改写分支要求 `find` 命中已有记录，新增分支才 `push`）。

**条款原文**（`.claude/kb/decisions/16-发布语义.md:361`，附录一）：

> 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)

`.claude/kb/decisions/03-空间分配.md:175`（已定项 7，附录一）：

> **落点释放时条目不删、不点删**……改写成「已释放 + 释放代」……条目留到该落点被重新分配时覆盖

**判定：一致（现状段已言明这是第一版特例化）**。`reclaim_released_up_to(floor)` 只比较 `record.generation <= floor`，不显式算 `max(floor, 环里最旧有效根)`。但 `.claude/kb/milestone/02-second-txn.md` 步 5 现状段自己写明：「第一版环里最旧有效根恒 0，floor 就是 F_生效」——根环 24 槽、mkfs 种子 txg 恒为 0 且第一版脚本走不到 24 次发布，所以"环里最旧有效根"这个量在这条脚本能达到的状态里恒为 0，`max(F_生效, 0) == F_生效`，代码直接传 `F_生效`（即调用方传入的 `floor` 参数，来自 `effective_rollback_floor` 或 `raise_rollback_floor` 里显式传入的 `new_floor`）在数值上与条款公式相等。这是 kb 自己交代过的简化，不算代码与条款不一致。复用时记录改写、槽号每盘唯一：单测 `releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue`（`allocator.rs:676-708`）与 `first_transaction_placements_follow_the_byte_table`（`allocator.rs:589-643`）都实测核过。

**什么现象会推翻这条判定**：环里最旧有效根在某段可达历史里非 0（比如根环真的转满一圈之后），而 `reclaim_released_up_to` 仍只比较 `F_生效`——那时代码会比条款公式更激进地回收，是真的不一致，需要传入 `max(F_生效, 环里最旧有效根)`。这条腿没有验证"环里最旧有效根"在这条脚本以外的可达性，留给攻方腿。


## S7：抬 F 的上限

**代码怎么做**（`mount.rs:194-245`，`rollback_floor_ceiling`）：

```rust
let valid: Vec<RootRecord> = readable_roots(...)
    .into_iter()
    .filter(|root| root.checkpoint_txg >= current_floor)
    .filter(|root| { !table.rows.iter().any(|row| row.instance == root.instance
        && root.checkpoint_txg > row.selected_root_txg) })
    .collect();
let newest_on_every_device = ...; // 每块盘上最新有效根，取 min
let mut non_empty: Vec<CheckpointTxg> = valid.iter()
    .filter(|root| records.values().any(|record|
        record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
        && record.transaction != 0))
    .map(|root| root.checkpoint_txg).collect();
non_empty.sort_unstable_by(|left, right| right.cmp(left));
let fourth_newest = non_empty.get(3).copied()
    .or_else(|| valid.iter().map(|root| root.checkpoint_txg).min())?;
Some(newest_on_every_device.min(fourth_newest))
```

**条款原文**（`.claude/kb/decisions/16-发布语义.md:361`，附录一）：

> 抬 F 的上限 | min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根

S7 判据表自己的括注：「非空按『环里有它自己那条记录且事务号 ≠ 0』认（预想）」。

**判定：一致**。代码里 `valid` 的过滤条件（`txg >= current_floor` 且按实例表判有效）对应"有效"；`newest_on_every_device`（按区域分设备取每盘最大再取 min）对应"每块盘上最新的持久有效根"；`non_empty` 用 `record.transaction != 0` 判"非空"、`sort` 降序后取下标 3（第 4 个）对应"第 4 新的非空持久有效根"；`non_empty.get(3)` 落空时 `.or_else` 取 `valid` 里最小 txg，对应"不足 4 个取最旧的有效根"。这条的"非空按事务号判"在 kb 里本来就标了"预想"、且在 `.claude/kb/milestone/02-second-txn.md` 步 5 的决策点清单里列成"checker 怎么认『非空持久有效根』（今天按环里记录的事务号）"——是已知未定项，不是代码违反了已定条款。

**什么现象会推翻这条判定**：读到 `rollback_floor_ceiling` 的过滤逻辑与上面贴的代码不同（比如漏了某个 `filter`），或 kb 撤回"预想"标注、把"非空"正式定义成别的量而代码没跟着改。


## S8：抬 F 的流程与生效

**代码怎么做**（`mount.rs:261-332`，`raise_rollback_floor`）：

- 先回收再推空发布：`mount.rs:294`（`allocator.reclaim_released_up_to(new_floor)`）在 `while` 循环（`mount.rs:298-326`，连推 `PublishPlan { rollback_floor: new_floor, instance_table: InstanceTablePlan::Carry(...), ... }`）之前。注释（`mount.rs:291-293`）：「回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事」。
- 推到每块盘都有一条：`mount.rs:298-301`（`while all_devices.iter().any(|identity| !covered.contains(identity)) && publishes.len() < ROOT_RING_REGIONS`）。
- 重开时 `F_生效` 既给分配器回收也写进新实例的根：`mount.rs:169-189`（`rebuilt_allocator` 里 `effective_rollback_floor(...)` 算出 `floor`、`allocator.reclaim_released_up_to(floor)`）；`mount.rs:387`（`establish_instance` 里 `rollback_floor: start.effective_floor`，`start.effective_floor` 来自同一个 `rebuilt_allocator` 的返回值，`mount.rs:490`、`586`）。

**条款原文**（`.claude/kb/decisions/16-发布语义.md:361`，附录一）：

> 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值

`.claude/kb/decisions/28-挂载期承诺量.md:16`（已定项 1，附录一，式子第五项说明）与 `.claude/kb/decisions/03-空间分配.md:175`（已定项 7）：分配器一侧"释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配"。

**判定：一致**。"先回收再推空发布""推到每块盘各有一条为止""重开时 `F_生效` 同时喂给分配器与新根"三句都在代码里找到了对应实现，且是同一个 `effective_floor` 值同时用于两处（`rebuilt_allocator` 返回的 `floor` 一路传到 `establish_instance` 的 `PublishPlan.rollback_floor`），对应条款「一条根带的 F 与它的记账行才说同一件事」。「F 可以被写低」——`.claude/kb/milestone/02-second-txn.md` 步 5 的决策点清单把"F 只在一块盘上时新实例的根写 F_生效（把 F 写低）是不是条款的意思"列成未决问题，代码的行为（`rebuilt_allocator` 直接把跨设备 min-max 算出来的 `effective_floor` 写进新根，不管这个值是不是比某块盘上旧根的 F 更小）与该决策点的描述一致，是 kb 已经标出的"条款没说、实做先这么定"，不算不一致。

**什么现象会推翻这条判定**：读到 `raise_rollback_floor` 在没有先回收的情况下就推发布，或 `establish_instance` 写进新根的 `rollback_floor` 不是 `rebuilt_allocator` 算出来的那个 `effective_floor`。


## S9：checker 的候选集

**代码怎么做**（`crates/singlefs-checker/src/walk.rs:781-799`）：

```rust
let instance_table_rows = walk.instance_table_rows.clone();
let newest_rollback_floor = u64::from_le_bytes(
    roots[newest_index].2.record_bytes[130..138].try_into().expect("8 字节"),
);
for (index, (_, _, root)) in roots.iter().enumerate() {
    let abandoned = instance_table_rows.iter().any(|(row_instance, row_txg)| {
        *row_instance == root.instance && root.checkpoint_txg > *row_txg
    });
    let below_floor = root.checkpoint_txg < newest_rollback_floor;
    if index != newest_index && !abandoned && !below_floor {
        walk.walk_root(&root.record_bytes, false);
    }
}
```

I-2.1 判定的执行点在 `crates/singlefs-checker/src/image.rs:292-317`（`read_referenced_unit`，每条位置条目都判 `"I-2.1"`），只会在 `walk_root` 遍历到的单元上被调用——而 `walk_root` 只在上面这段代码里对候选集里的根被调用（`newest_index` 那个根总是走，别的根只在 `!abandoned && !below_floor` 时走）。

**条款原文**（`.claude/kb/invariants.md:113`，I-3.1 行，附录一）：

> **2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ 最新根带的回退下界 F——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「第二个事务」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）

I-2.1 行（`.claude/kb/invariants.md:102`，附录一）：

> 2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红

**判定：一致**。`abandoned` 判断（"最新根指着的实例表有 root.instance 这一行，且 root.checkpoint_txg > 那一行的 selected_root_txg（即 Ti）"）逐句对应"有行 (i,Ti,Wi) 且 T ≤ Ti 才有效"的反命题；`below_floor`（`root.checkpoint_txg < newest_rollback_floor`）对应"txg ≥ 最新根带的回退下界 F"的反命题；两者都不满足才 `walk_root`，对应"有效根"的定义。`newest_rollback_floor` 直接从最新根的 `record_bytes[130..138]` 读——核对 `crates/singlefs-core/src/root_record.rs:12-13`（`ROOT_CHECKSUM_OFFSET = 4+16+4+4+8+86+8+8`，`rollback_floor` 字段紧接在 `tree_identifier_watermark`（偏移 122..130）之后，占偏移 130..138），字节偏移与 `RootRecord` 结构体字段定义一致；`walk.rs` 直接解字节而不复用 `RootRecord::parse_slot`，符合 D13 已定项 5"checker 与被测实现各写一份独立实现"的要求（这条腿只做了偏移核对，D13 已定项 5 全文没有列进这一轮引用清单，不展开判定）。I-2.1 只在候选集里的根上判：因为 `read_referenced_unit`（判 I-2.1）只被 `walk_root` 内部调用，而 `walk_root` 只对候选集里的根被调用，逻辑上蕴含成立。

**什么现象会推翻这条判定**：读到 `walk_root` 在 `abandoned` 或 `below_floor` 为真时仍被调用（候选集判定被绕过），或 `newest_rollback_floor` 的偏移与 `root_record.rs` 里 `rollback_floor` 字段实际写入的偏移不一致。


## S10：第一个文件版本释放 mkfs 树表单元

**代码怎么做**：

- `crates/singlefs-core/src/transaction.rs:571-592`（`format_time_tree_table_to_release`）：
  ```rust
  fn format_time_tree_table_to_release(
      allocator: &PoolAllocator,
      rewritten: &[TransactionUnit],
  ) -> Vec<Placement> {
      if !rewritten.contains(&TransactionUnit::TreeTable) {
          return Vec::new();
      }
      let Some(tree_table) = allocator.format_time_tree_table() else {
          return Vec::new();
      };
      let still_allocated = allocator.devices.first()
          .and_then(|device_map| allocator.record_for(device_map.device, tree_table.slot))
          .is_some_and(|record| !record.is_released);
      if still_allocated { vec![tree_table] } else { Vec::new() }
  }
  ```
- 调用点 `transaction.rs:914-929`（`publish_version`），只在 `previous.is_none()` 时调用：
  ```rust
  let release = match previous {
      Some(previous_version) => placements_to_release_via_mapping(previous_version, allocator, &rewritten)?,
      None => format_time_tree_table_to_release(allocator, &rewritten),
  };
  ```
- 唯一以 `previous: None` 调 `publish_version` 的路径是 `publish_first_file`（`transaction.rs:838-869`）——`mount_writable` / `mount_rollback` 的 `establish_instance` 与 `raise_rollback_floor` 都传 `Some(...)`（`mount.rs:319`、`389`、`422`），`grep -n "publish_version(" crates/singlefs-core/src/*.rs` 命中 5 处，只有 `transaction.rs:847` 那一处（`publish_first_file` 内）传 `None`（`transaction.rs:867`）。

**条款原文**（`.claude/kb/decisions/03-空间分配.md:175`，附录一，已定项 7）：

> **落点释放时条目不删、不点删**……改写成「已释放 + 释放代」

`.claude/kb/milestone/02-second-txn.md:158`（步 4 现状段，附录一）虽属步 4 但字面提到"第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元释放"的位置在步 5（`.claude/kb/milestone/02-second-txn.md:187`）：

> 第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元释放（不释放它，抬 F 之后那一槽永远占着、I-3.1（已分配统计对得上） 红；第一个事务的字节表五随之改，E142（第一个事务的干跑） 第十次跑）

**产物核对**（`research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out`，本地命令 `grep -n "^E7RESULT name=accounting\|^E7RESULT name=allocation" ...` 原样输出）：

```
E7RESULT name=accounting entries=15 allocated_bytes_per_device=212992 free_bytes_per_device=3472670720 empty_cluster_segments_per_device=3310 fragmentation_runs_per_device=4 inode_watermark=2 unreclaimable_per_device=0 defer_queue_per_device=16384 pending_delete=0 committed_reservation=0 sequence_all_one=true generation=3
E7RESULT name=allocation records=20 first_slot=50176 last_slot=50248 key_bytes=10 value_bytes=10 mkfs_generation_records=2
```

`defer_queue_per_device=16384`（第 0 版树表那 1 槽 × 16384 字节）与 `mkfs_generation_records=2`（mkfs 写出的 4 条分配代 0 记录里，树表那 2 条被改写成释放代 3，剩实例表那 2 条还是代 0）——与 kb 现状段"defer_queue_per_device 0 → 16384、mkfs_generation_records 4 → 2"逐字相符。

**单测核对**（`crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:872-887, 954-960`）：

```rust
assert_eq!(records.iter().filter(|record| record.generation == CheckpointTxg(0)).count(), 2,
    "mkfs 的 m1 分配代 0；m2（第 0 版树表）被 A 换下，记录改写成已释放、释放代 3");
assert_eq!(records.iter().filter(|record| record.generation == CheckpointTxg(3)).count(), 18,
    "A 的八个落点各两盘 16 条，加 m2 那两条改写成释放代 3");
...
assert_eq!(value_of(STATISTIC_DEFER_QUEUE_BYTES), 16384,
    "defer 待释放 = mkfs 那片第 0 版树表单元（1 槽）：A 重写树表把它换下（D3 已定项 7）");
```

**判定：一致**，代码、单测、E142 第十次跑产物三方在这一格上逐字互证：`format_time_tree_table_to_release` 只在第一个文件版本（`previous.is_none()`）时触发、释放条件是"树表被重写 ∧ mkfs 树表单元还没释放过"，行为与 D3 已定项 7"改写不删除"一致，且产物与单测的具体数字（`defer_queue_per_device=16384`、`mkfs_generation_records=2`、代 0 记录数 2、代 3 记录数 18）三处互相吻合。

**什么现象会推翻这条判定**：`still_allocated` 判断被去掉（导致覆盖写第二次调用该函数时重复释放同一个落点，触发 `release()` 里"同一个落点释放了两次"的 `assert!`）、或产物 / 单测里的这几个数字与代码实际跑出来的不一致（这条腿只读了留存产物与单测源码，没有重新编译跑一遍装置，见"没做什么"）。


## S11：层 0 oracle 版本表

**代码怎么做**（`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:89-190`，`prepare` 函数里的 `versions` 构造）：

- 回退之后 (3,9)/(3,10) 读第一次内容：`second_transaction_step_zero_layer0.rs:143-149`
  ```rust
  for txg in 9..=10 {
      versions.push(PublishedVersion {
          instance: InstanceGeneration(3),
          checkpoint_txg: CheckpointTxg(txg),
          content: file_content(),   // 第一次的内容
      });
  }
  ```
- 抬 F 的两条根读第四版：`second_transaction_step_zero_layer0.rs:150-179`——`latest` 在 `for (txg, seed) in [(11,17),(12,19),(13,23),(14,29)]` 循环里最后一次赋值是第 4 次覆盖写（txg=14）的内容，随后
  ```rust
  for txg in 15..=16 {
      versions.push(PublishedVersion {
          instance: InstanceGeneration(3),
          checkpoint_txg: CheckpointTxg(txg),
          content: latest.clone(),  // 循环里最后一次（第四版）的内容
      });
  }
  ```
- E 读自己：`second_transaction_step_zero_layer0.rs:180-189`
  ```rust
  let reuse = overwrite(&mut pool, &later_content(31), InstanceGeneration(3));
  ...
  versions.push(PublishedVersion {
      instance: InstanceGeneration(3),
      checkpoint_txg: CheckpointTxg(17),
      content: later_content(31),
  });
  ```

**条款原文**（S11 判据表本身，`_m2-step45-code-r1-background.md` 第 38 行）：

> S11 | 层 0 到 E：oracle 版本表按 (txg, 实例) 找版本；回退之后 (3, 9)、(3, 10) 读第一次内容；抬 F 的两条根读第四版；E 读自己 | D13（验证路线） 已定项 4；E77（发布的持久顺序） 判据 1

D13 已定项 4（`.claude/kb/decisions/13-验证路线.md:326`，附录一）定义崩溃状态枚举的段模型，但没有单独定义"哪个 txg 该读哪版内容"这件事——那句判据来自这条测试脚本自身对"暖机 / 抬 F 空发布不改变文件内容，只有覆盖写才改变"这条语义的具体化。这条腿核对的是"代码构造的版本表是否与判据表给出的字面（(3,9)/(3,10) 读第一次、抬 F 两条根读第四版、E 读自己）相符"，不是核"这句判据本身是不是 D13 已定项 4 唯一能推出的答案"（那需要历史构造，是攻方腿的射程）。

**判定：一致**。三段代码逐字对上 S11 判据表给出的三句描述：txg 9/10 的 `content` 字段是 `file_content()`（第一次内容），txg 15/16 的 `content` 是循环变量 `latest` 在退出循环时的值（第四次覆盖写、即第四版的内容），txg 17（E）的 `content` 是它自己刚发布的 `later_content(31)`。

**什么现象会推翻这条判定**：`latest` 变量在循环外被重新赋值、或 txg 15/16 使用的不是循环最后一次迭代的值，或 txg 9/10 使用的不是 `file_content()`。

## S12：链首无锚点

**代码怎么做**（`recovery.rs:739-753`）：

```rust
let root_own_record_counter = records.values()
    .find(|record| record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg)
    .map(|record| record.counter);
let mut expected_next: Option<(InstanceGeneration, u64)> =
    root_own_record_counter.map(|counter| (root.instance, counter + 1));
let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
for record in above.into_iter().take(in_flight_limit) {
    if let Some(expected_key) = expected_next {
        if (record.instance, record.counter) != expected_key { break; }
    } else if record.checkpoint_txg != chain_start_txg_without_anchor {
        break;
    }
    ...
}
```

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1206`，附录一，跨实例边界注 1）：

> 前缀规则不跨实例边界：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（六条口径第一条原样）

S12 判据表本身：

> S12 | 上一轮改法：所选根自己那条记录读不出时链首只认 `checkpoint_txg = 根 txg + 1` 的那条（一次发布一条记录、txg 每次加一） | D23（journal 的角色与格式） 已定项 14 六条口径第一条

**判定：一致**。`root_own_record_counter` 读得出（`Some`）时，`expected_next` 按"所选根覆盖的最后一条记录之后接"（`counter + 1`）；读不出（`None`）时才落到 `chain_start_txg_without_anchor = root.checkpoint_txg + 1` 这条兜底规则——与 S12 判据表描述的"自己那条记录读不出时才用 txg+1"逐句对应，且这条兜底只在 `root_own_record_counter.is_none()` 时才会被走到（`else` 分支），不会覆盖前一分支。这条改法在 `.claude/kb/experiments/142-第一个事务的干跑.md:2`（附录一）第十次跑的历史条目里有交叉印证：「链首无锚点（所选根自己那条记录读不出）时只认 checkpoint_txg = 根 txg + 1 的那条（步 3 三方第一轮云端攻方腿打中『无条件接上会跳过撕掉的一条』）」，与代码的 `else` 分支行为一致。

**什么现象会推翻这条判定**：读到 `expected_next` 为 `None` 时代码接的不是 `chain_start_txg_without_anchor` 而是"可读链末尾 + 1"（即扫描 `records` 里 `above` 集合的最小 `(instance, counter)`），或 `root_own_record_counter` 的查找条件与"所选根自己那条记录"的定义（`instance == root.instance && checkpoint_txg == root.checkpoint_txg`）不符。


## X10：kb 写回与代码对不对得上

**核的范围**：`.claude/kb/milestone/02-second-txn.md` 步 0 / 步 4 / 步 5 的「现状」段、`.claude/kb/layout/01-first-txn.md` 八「实例切换 / 管理员回退」「抬 F 的空发布」两行与「第二条流」那句、`.claude/kb/invariants.md` I-3.1 / I-2.1 的读法。

### I-3.1 / I-2.1 的读法（invariants.md:113、102）

已在 S9 详述：`walk.rs:781-799` 的 `abandoned` / `below_floor` 双过滤逐句对上 invariants.md 里"2026-09-17 起『有效根』= 回退候选集里的根"那句定义。**一致**。

### 步 4 现状段（milestone/02-second-txn.md:158）

逐句核对（每句摘引原句 + 对应代码位置）：

- 「读根环里全部可读根（`recovery::readable_roots`）」——`mount.rs:531-536` 调用 `readable_roots(...)`。**一致**。
- 「候选集按最新根指着的实例表判……且 txg ≥ 最新根带的 F」——这句里的「最新根带的 F」与 S1 判定完全一致地对应 `mount.rs:547` 的 `newest_root.rollback_floor`。kb 这句现状描述精确地写成"最新根带的 F"而不是"F_生效"，与代码逐字一致（这句现状描述本身没有夸大到 F_生效 那个跨设备公式）——**一致**，但连带说明：S1 判定的"不一致"是相对 D16 已定项 1 与 D23 已定项 14 的条款原文而言，不是相对这句"现状"描述；这句"现状"如实反映了代码的做法。
- 「取号 3」「jsn = R_old 那条 + 1 = 4」「txg = max(根环, 记录) + 1 = 9」——被 `second_transaction_step_four_rollback.rs:153-158`（`rollback_publish.record.counter` 断言为 4，注释「jsn 接在 A 的 jsn 3 之后」）与该测试头部注释（`:2`「发布 D（txg 9，jsn 接在 A 那条记录之后 = 4）」）钉住。**一致**。
- 「暖机一次（txg 10 落盘 1）」——`second_transaction_step_four_rollback.rs:178`（`assert_eq!(warm_up.record.counter, 5)`）；这条腿没有单独核 `warm_up.root.checkpoint_txg == 10` 那个具体断言行号，但同一测试文件头部注释与该数一致，判**一致（未逐行核对 txg=10 那一句具体断言，见"没做什么"）**。
- 「被抛弃根独占的槽逐盘 34」——`second_transaction_step_four_rollback.rs:186`（`assert_eq!(abandoned.len(), 34, ...)`）与 `:188`（`output.isolated_slots_per_device.contains(&(device, 34))`）。**一致**。
- 「checker：I-3.1 的并集只取按最新根实例表仍有效的根（被抛弃时间线的根不进）」——步 4 这句没有提 F 那道过滤（F 在这条脚本走到步 4 时恒为 0），与 `walk.rs` 的双过滤器在 `below_floor` 恒假时退化成单过滤一致。**一致**。

### 步 5 现状段（milestone/02-second-txn.md:187）

- 「`PoolAllocator::reclaim_released_up_to(floor)` 把释放代 ≤ floor 的已释放落点回到空闲……第一版环里最旧有效根恒 0、floor 就是 F_生效」——与 S6 判定一致：代码只比较 `floor`，kb 自己交代了简化理由。**一致**。
- 「`rollback_floor_ceiling` = min(每块盘上最新的有效根, 第 4 新的非空有效根)（有效 = 自证 ∧ 按最新根实例表有效 ∧ txg ≥ 今天的 F；非空 = 环里有它自己那条记录且事务号非 0，预想）」——与 S7 判定一致，且这句现状自己标了"预想"。**一致**。
- 「第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元释放……第一个事务的字节表五随之改，E142 第十次跑」——与 S10 判定一致（代码 + 单测 + 产物三方互证）。**一致**。
- 「checker 的 I-3.1 并集按最新根自己的 F 收候选集（F 之下的根不走，I-2.1 也不在它们上判）」——与 S9 判定一致。**一致**。

### layout/01-first-txn.md 八「实例切换 / 管理员回退」「抬 F 的空发布」两行、「第二条流」那句（layout/01-first-txn.md:380）

- 「管理员回退那一半……[取号超级块槽 × 2 盘，世代号 4] 屏障 [实例表单元（回退行 + 中间实例行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障……」——`5 单元 × 2 盘 = 10` 与 `PublishPlan::rewritten_roles()`（`transaction.rs:812-833`）在 `file=None, InstanceTablePlan::Rewrite(_)` 时给出的角色数吻合：`InstanceTable`（因为 `Rewrite`）+ `AllocationTree` + `AccountingTree` + `MappingTree` + `TreeTable` = 5 个角色 × 2 盘 = 10。**一致**。
- 「抬 F 的空发布……[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8]」——`raise_rollback_floor` 的 `PublishPlan.instance_table` 是 `InstanceTablePlan::Carry(...)`（`mount.rs:313`），`rewritten_roles()` 在 `Carry` 时不含 `InstanceTable`，角色数是 `AllocationTree, AccountingTree, MappingTree, TreeTable` = 4 个 × 2 盘 = 8。**一致**。
- 「第二条流」那句给出的段序列与写数（`54 段、279 次写、2104413 个状态`）：这条腿**没有重新编译运行装置去逐段核对这个闭式数**，只核了它引用的机制（取号屏障、回退发布角色数、抬 F 发布角色数）都对得上代码；`279` 次写与 `2104413` 个状态的算术属于"能用命令核的事实"，但这条腿的草稿目录里没有编译环境验证权限之外的复杂度预算，**复核不了，交主 agent 或 crash-verifier 用门禁 54 号实测**。

**判定：一致（除"没做什么"里列出的两处未逐行核对 / 算术复核不了）**。


## 条款没说的取法（kb 自己已标出的决策点，随判定一并列出）

| 格 | 取法 | kb 里标记的位置 |
|---|---|---|
| S2 | 回退之后新实例第一条 jsn = R_old 那条 + 1（P1，不是可读链末尾 + 1） | `checks-owed.md` C340（回退之后记录链从哪条之后接没有定义），取 P1 |
| S3 | 回退行的 W（`applied_transaction_high_water`）恒写 0 | `milestone/02-second-txn.md` 步 4 现状段「实做时定下的（标预想、交用户）」 |
| S4 | 影子账查账集合取窄读法（按"被抛弃时间线的根"读，不查凡被环里任一可读根引用过的槽） | `decisions/23-journal的角色与格式.md:1206`「2026-09-16 用户定案取窄读法」 |
| S5 | 前缀第五条只读所选根自己指着的实例表（不读别的根的表） | 判据表 S5 本身；无独立 kb 决策点条目，是条款字面直接给的 |
| S7 | 抬 F 上限「非空」按"环里有它自己那条记录且事务号 ≠ 0"认 | `milestone/02-second-txn.md` 步 5 现状段与决策点清单，标"预想" |
| S8 | F 只在一块盘上生效时，新实例的根仍写 `F_生效`（把 F 写低）——是不是条款的意思 | `milestone/02-second-txn.md` 步 5 决策点清单，未定 |

## 这条腿自己的限度

- 只做「代码 vs 附录原文」逐句核对，不构造可达历史，不判 X1–X9（攻方腿的格）。S1、S6 里提到的"F_生效 vs 单根字段"这类潜在缺口，这条腿只报出字面差异与它在当前测试路径下是否可观测，不判定它会不会被某段真实历史触发——那是 X6 的射程。
- 没有重新编译 `crates/` 跑一遍装置：所有代码引用都是静态读源码 + 已有单测源码 + 已留存的产物文件（`research/results/e142-...out`），没有现场 `cargo test` 或跑 `mutate.sh` 复验这条腿自己给出的推断，是否与今天磁盘上编译出来的二进制行为一致，全靠背景材料里主 agent 现查的"今天的状态"一段（`_m2-step45-code-r1-background.md` 第 9 行）。
- 没有对第二条流的 `279` 次写 / `2104413` 个状态这类算术做独立复算，标记为"复核不了"，理由见 X10 末尾。
- 没有核对 `.claude/kb/milestone/02-second-txn.md` 步 4 / 步 5 现状段里每一个具体断言的行号（比如"暖机一次（txg 10 落盘 1）"的 `txg==10` 那句断言具体在文件哪一行），只核了机制层面的对应关系。

## 没做什么

- 不判攻方腿的格：X1–X9 一格没碰，禁读清单里别的腿的输出文件也没有读。
- 没有编译或运行 `crates/`、`research/e7-index-bench`，所有代码事实来自静态读源码；没有重新跑 `mutate.sh`、`replay.sh` 或门禁阶段（`.claude/gate.d/`），因为分工提示没有把跑门禁的阶段登记给这条腿。
- 第二条流的段序列闭式算术（54 段、279 次写、2104413 个状态）没有独立复算，交主 agent 或 `crash-verifier` 用门禁 54 号实测复核。
- `.claude/kb/layout/01-first-txn.md` 八表格里"第二条流"那一整句所引用的每一个具体子段（比如回退发布内部 10 个单元写的次序细节）没有逐字节比对录制流，只核了角色数量与调用路径。
- 没有核 D13 已定项 5、D9 加密相关分项等这一轮判据表没有直接引用的条款，只核了 S1–S12、X10 明确列出的条款原文。
