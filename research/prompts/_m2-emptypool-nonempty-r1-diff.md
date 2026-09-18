# 附录二：空池可写挂载与「非空」按树表认，代码改动（工作区当前内容整段抄，不是 git diff；2026-09-17）

基准：HEAD `5f9e449`（`里程碑二步 1 / 步 2「发布 B」落地并走完代码三方两轮；crates 变异表与门禁 59 号`）。
以下每段都是任务给定的文件与函数 / 类型名，在**当前工作区**（相对上面这条 HEAD 尚未提交的改动）里的整段原文，
用 `awk 'NR==A,NR==B'` 对工作区文件现取，行号是当前工作区的行号；不是 `git diff`（这批改动与同一时段别的会话改的
`crates/` 文件混在同一个未提交工作区里，一次 `git diff HEAD -- crates/` 会把别处的改动一起带进来）。
生成时刻：2026-09-17 19:10 JST（UTC 10:10）。

它不并进背景材料，各腿按正文里写的路径去读工作区（`crates/singlefs-core/src/mount.rs` 等）；这里只是把
主 agent 点名的那几个函数 / 类型 / 用例的原文集中摘出来，方便核对行号与原文一致。

## 一、`crates/singlefs-core/src/mount.rs`

### `MountError`（30-79）

```rust
/// 可写挂载没做成。
#[derive(Debug)]
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面有文件版本，而环里一条自证过的记录都没有：从盘上重建上一版要一条记录顶着，拿不出来
    /// （树表 0 条的根用不到记录，刚 mkfs 的池不走这里）。
    FileVersionWithoutAnyJournalRecord,
    /// 所选根指着的实例表单元解不出行与链指针。
    InstanceTableMalformed,
    /// 抬 F 要读当前版本的实例表判候选集，而这个进程还没做过可写挂载（实例表单元没重写过）：只 mkfs 加第一个事务的进程不能抬 F
    /// （alloc-basis 第二轮云端攻方腿第七节第 1 条：`TransactionOutput::unit` 在那里会 panic）。
    RaiseNeedsWritableMountInThisProcess,
    Acquisition(AcquisitionFailed),
    Publish(PublishError),
    /// 回退的目标根不在根环里（没有那个 (实例, txg) 的可读根槽）。
    RollbackTargetNotInRing(RollbackTarget),
    /// 回退的目标根在根环里，却不在回退候选集里：被抛弃时间线的根，或 txg 低于回退下界 F。
    RollbackTargetNotACandidate {
        target: RollbackTarget,
        reason: &'static str,
    },
    /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
    RollbackFloorAboveCeiling {
        requested: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
    /// 所选根的树表 0 条（还没发布过文件版本），而这次挂载要给 [max(所选根的实例, 1), 新实例) 写实例表行：写行要重写实例表，
    /// 没有文件版本的一版上它的落点记在哪没有条款，第一版不支持。在取号之前拒绝，盘上一个字节都不动。
    InstanceRowsOnVersionWithoutFileUnsupported {
        chosen_root: RollbackTarget,
        first_row_instance: InstanceGeneration,
        instance_to_acquire: InstanceGeneration,
    },
    /// 回退的目标根的树表 0 条（还没发布过文件版本）：回退行要重写实例表，没有文件版本的一版上它的落点记在哪没有条款，
    /// 第一版不支持。在任何写之前拒绝。
    RollbackToVersionWithoutFileUnsupported(RollbackTarget),
    /// 树表 0 条、而根记录指着的实例表或树表不是 mkfs 写的那一版（指针的诞生 txg 不是 0）：这样一版的分配记录在哪没有条款，
    /// 这个实现自己写不出这样的根（坏盘或别的写者才有），拒绝挂载。
    VersionWithoutFileNotWrittenByMakeFilesystem {
        root: RollbackTarget,
        instance_table_birth_txg: CheckpointTxg,
        tree_table_birth_txg: CheckpointTxg,
    },
    /// 算抬 F 的上限时一条有效根（不是被抛弃的、txg ≥ F）的树表读不出或解不开：它算不算非空判不了，读不出要走修复、不跳过也不猜，
    /// 这次抬 F 拒绝（不回收、不发布）。
    RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
        root: RollbackTarget,
        failure: RecoveryFailure,
    },
}
```

### `PreviousVersion`（148-160）

```rust
/// 恢复（或回退）之后从盘上重建出来的上一版，带文件的连同写行要接在后面的那一版实例表。
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有一个，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
enum PreviousVersion {
    /// 树表 0 条：这一版只有根自己指着的实例表与树表两个单元。
    WithoutFile(RootRecord),
    WithFile {
        output: TransactionOutput,
        table: InstanceTableRecords,
    },
}
```

### `format_time_allocator`（385-425）

```rust
/// 树表 0 条的一版的账：盘上没有分配记录树，这一版的全部落点就是根记录直接指着的实例表与树表两个单元——mkfs 写在单元区里的那两个
/// （字节表五里它们两盘各一条、分配代 0）。与 mkfs 同一个进程里第一个事务用的分配器同一个起点（`PoolAllocator::mark_format_time_units`），
/// 第一个文件版本换下 mkfs 那片树表时照样把它释放。
///
/// # Errors
/// 根记录指着的实例表或树表不是 mkfs 写的那一版（诞生 txg 不是 0）⇒ `VersionWithoutFileNotWrittenByMakeFilesystem`：这一版的落点记在哪没有条款
/// （D16（发布语义） 已定项 9 只定了树表 0 条时空发布写零个单元）。这个实现写出的树表 0 条的根都指着 mkfs 那两个单元——重写实例表或树表的
/// 只有 `publish_version`，而它写出的树表恒有 7 条；零单元发布照抄上一版的指针；树表 0 条时要写行或回退都在任何写之前拒绝。
fn format_time_allocator(
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    if root.instance_table.head.birth_txg != CheckpointTxg(0)
        || root.tree_table.head.birth_txg != CheckpointTxg(0)
    {
        return Err(MountError::VersionWithoutFileNotWrittenByMakeFilesystem {
            root: RollbackTarget {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
            },
            instance_table_birth_txg: root.instance_table.head.birth_txg,
            tree_table_birth_txg: root.tree_table.head.birth_txg,
        });
    }
    let placement_of = |pointer: &crate::pointer::NodePointer, identity: TransactionUnit| {
        assert_eq!(
            pointer.locations[0].slot, pointer.locations[1].slot,
            "两盘同槽（D2（RAID 条带策略） 已定项 10）"
        );
        Placement {
            slot: pointer.locations[0].slot,
            span: identity.span_slots(),
        }
    };
    let mut allocator = PoolAllocator::new(device_maps);
    allocator.mark_format_time_units(
        placement_of(&root.instance_table, TransactionUnit::InstanceTable),
        placement_of(&root.tree_table, TransactionUnit::TreeTable),
    );
    Ok(allocator)
}
```

### `rollback_floor_ceiling`（438-519）

```rust
/// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
/// 比它小的有效根里最大的那条）的不同（`user_visible_trees_changed`，2026-09-17 用户定案）。
///
/// # Errors
/// 一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`；要比的一条有效根的树表读不出或解不开 ⇒
/// `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（读不出要走修复，不按空或非空猜）。
#[allow(
    clippy::ptr_arg,
    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub fn rollback_floor_ceiling<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    superblock: &crate::superblock::Superblock,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
) -> Result<CheckpointTxg, MountError> {
    let readable = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let valid: Vec<RootRecord> = readable
        .iter()
        .copied()
        .filter(|root| root.checkpoint_txg >= current_floor)
        .filter(|root| !abandoned_by_table(root, table))
        .collect();
    let mut newest_per_device: std::collections::BTreeMap<DeviceIdentity, CheckpointTxg> =
        std::collections::BTreeMap::new();
    for root in &valid {
        let device = superblock.region_devices
            [usize::try_from(target_for_publish(root.checkpoint_txg).region).expect("区域号")];
        let newest = newest_per_device
            .entry(device)
            .or_insert(root.checkpoint_txg);
        *newest = (*newest).max(root.checkpoint_txg);
    }
    let newest_on_every_device = newest_per_device
        .values()
        .copied()
        .min()
        .ok_or(RecoveryFailure::NoValidRoot)?;
    // 读不出或解不开就拒绝：它算不算非空、它后面那条跟谁比都判不了，读不出要走修复（D16（发布语义） 已定项 1「非空」从盘上怎么认只说了比树表里的根指针）。
    let pointers_of = |root: &RootRecord| match user_visible_tree_root_pointers(devices, root) {
        Ok(pointers) => Ok(pointers),
        Err(failure) => Err(
            MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
                root: RollbackTarget {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
                failure,
            },
        ),
    };
    // 「前一条」只在有效根里找：被抛弃时间线的根与 F 之下的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
    let previous_candidates: &[RootRecord] = &valid;
    let mut non_empty: Vec<CheckpointTxg> = Vec::new();
    for root in &valid {
        let previous_valid_root = previous_candidates
            .iter()
            .filter(|candidate| {
                (candidate.checkpoint_txg, candidate.instance)
                    < (root.checkpoint_txg, root.instance)
            })
            .max_by_key(|candidate| (candidate.checkpoint_txg, candidate.instance));
        let previous_pointers = match previous_valid_root {
            Some(previous) => pointers_of(previous)?,
            None => UserVisibleTreeRootPointers::ABSENT,
        };
        let root_pointers = pointers_of(root)?;
        if user_visible_trees_changed(&root_pointers, &previous_pointers) {
            non_empty.push(root.checkpoint_txg);
        }
    }
    Ok(
        ceiling_from_newest_and_non_empty_roots(newest_on_every_device, non_empty, &valid)
            .ok_or(RecoveryFailure::NoValidRoot)?,
    )
}
```

### `ceiling_from_newest_and_non_empty_roots`（521-533）

```rust
/// 上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根)；非空有效根不足 4 个时取最旧的有效根；一条有效根都没有时 `None`。
fn ceiling_from_newest_and_non_empty_roots(
    newest_on_every_device: CheckpointTxg,
    mut non_empty: Vec<CheckpointTxg>,
    valid: &[RootRecord],
) -> Option<CheckpointTxg> {
    non_empty.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest = non_empty
        .get(3)
        .copied()
        .or_else(|| valid.iter().map(|root| root.checkpoint_txg).min())?;
    Some(newest_on_every_device.min(fourth_newest))
}
```

### `raise_rollback_floor`（547-658）

```rust
/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）：正常的触发是准入不够，这里是只供测试的强制入口（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
/// 推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收。
///
/// # Errors
/// 这个进程没做过可写挂载；`new_floor` 超过上限；发布失败。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<RaisedFloor, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let instance_table_unit = current
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::InstanceTable)
        .ok_or(MountError::RaiseNeedsWritableMountInThisProcess)?;
    let table = InstanceTableRecords::parse(&instance_table_unit.bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    let ceiling =
        rollback_floor_ceiling(devices, &superblock, current.root.rollback_floor, &table)?;
    if new_floor > ceiling {
        return Err(MountError::RollbackFloorAboveCeiling {
            requested: new_floor,
            ceiling,
        });
    }
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
    // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
    // 落满每块盘之后再放开。记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
    let oldest_valid_root = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| !abandoned_by_table(root, &table))
    .map(|root| root.checkpoint_txg)
    .min();
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
    // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
        let roots = readable_roots(
            devices,
            &superblock.region_devices,
            &superblock.geometry,
            &superblock.filesystem_identifier,
        );
        isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            allocator,
            &roots,
            &|root| abandoned_by_table(root, &table),
            new_floor,
            &current.allocation_records,
        )
    } else {
        0
    };
    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = publish_version(
            &mut pool,
            allocator,
            PublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                transaction: 0,
                instance: current.root.instance,
                back_chain: back_chain_of(&current.record_bytes),
                file: None,
                instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                tree_birth_txg: current.tree_birth_txg(),
                tree_identifier_watermark: current.root.tree_identifier_watermark,
                rollback_floor: new_floor,
            },
            Some(&*current),
        )?;
        let device = device_of_txg(next.root.checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(next.clone());
        *current = next;
    }
    allocator.release_reclaim_holds();
    Ok(RaisedFloor {
        ceiling,
        publishes,
        reclaimed,
        abandoned_roots_unreadable,
    })
}
```

### `refuse_instance_rows_on_version_without_file`（742-762）

```rust
/// 所选根的树表 0 条而这次要写的行区间 [max(所选根的实例, 1), 要取的号) 不为空：在取号之前拒绝（写行要重写实例表，没有文件版本的一版上
/// 它的落点记在哪没有条款）。不靠坏盘就走得到：第一个事务暖机之后、第一个文件版本之前崩溃，重开时所选根 (1, 2)、要写 (1, 2, 0)。
fn refuse_instance_rows_on_version_without_file(
    start: &InstanceStart,
    instance_to_acquire: InstanceGeneration,
) -> Result<(), MountError> {
    let first_row_instance = start.previous_row.instance.0.max(1);
    match &start.previous {
        PreviousVersion::WithoutFile(chosen_root) if first_row_instance < instance_to_acquire.0 => {
            Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported {
                chosen_root: RollbackTarget {
                    instance: chosen_root.instance,
                    checkpoint_txg: chosen_root.checkpoint_txg,
                },
                first_row_instance: InstanceGeneration(first_row_instance),
                instance_to_acquire,
            })
        }
        PreviousVersion::WithoutFile(_) | PreviousVersion::WithFile { .. } => Ok(()),
    }
}
```

### `establish_instance`（764-882）

```rust
/// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘：可写挂载与回退共用的后半段。
fn establish_instance<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mut allocator: PoolAllocator,
    start: InstanceStart,
) -> Result<Mounted, MountError> {
    let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let instance_to_acquire = instance_generation_to_acquire(&pool);
    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;
    let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;

    // 写行：给 [max(上一个实例, 1), 新实例) 里每个实例各一行——上一个实例 (i, T, W)（回退时是回退行，flags bit0 = 1），
    // 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。
    let mut rows_written = Vec::new();
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
    }
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, table } => {
            PoolVersion::WithFile(publish_rows_on_file_version(
                &mut pool,
                &mut allocator,
                output,
                table,
                &rows_written,
                RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    rollback_floor: start.effective_floor,
                },
            )?)
        }
        PreviousVersion::WithoutFile(previous_root) => {
            assert!(
                rows_written.is_empty(),
                "取号之前 refuse_instance_rows_on_version_without_file 按同一个号核过：树表 0 条的一版上要写的行为空"
            );
            // 上一个实例是 0、要写的行为空：第一次可写挂载的形态，零单元（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；
            // 第一次可写挂载要写的区间是空的，第一个事务的字节不变，layout/01-first-txn.md 八「第一次之后的可写挂载（写行）」那一行 2026-09-14 的改名注）。
            PoolVersion::WithoutFile(
                publish_without_units(
                    &mut pool,
                    previous_root,
                    ZeroUnitPublishPlan {
                        txg: start.first_txg,
                        counter: start.next_counter,
                        instance,
                        back_chain: 0,
                        rollback_floor: start.effective_floor,
                    },
                )
                .map_err(PublishError::from)?,
            )
        }
    };

    // 暖机（D16（发布语义） 已定项 8 甲′）：本实例的根覆盖两块盘之前连推空发布，次数按落点公式现算，至多根环的区域数那么多次。
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish.root().checkpoint_txg)];
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = publish_empty_after(&mut pool, &mut allocator, &current, instance)?;
        let device = device_of_txg(next.root().checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        warm_up_publishes.push(next.clone());
        current = next;
    }

    Ok(Mounted {
        output: MountOutput {
            instance,
            chosen_root: start.chosen_root,
            effective_root: start.effective_root,
            journal: start.journal,
            rows_written,
            row_publish,
            warm_up_publishes,
            isolated_slots_per_device,
            abandoned_roots_unreadable: start.abandoned_roots_unreadable,
        },
        allocator,
        current,
    })
}
```

### `mount_writable`（884-957）

```rust
/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。只做过 mkfs 的池（所选根的树表 0 条、
/// 环里没有记录）同样走这条路：取号 1、不写行、零单元的发布推到本实例的根覆盖每块盘，第一个文件版本接在 `current` 后面。
///
/// # Errors
/// 恢复失败、所选根下有文件而环里一条记录都没有、实例表解不开、取号失败、发布失败。
pub fn mount_writable<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let chosen_root = choose_root(&*devices, &superblock).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(&*devices, &superblock);
    let (journal, effective_root) = replay_journal(
        &*devices,
        &chosen_root,
        superblock.geometry.journal_ring_bytes,
        &records,
        true,
        rollback_high_water_of_root(&*devices, &chosen_root),
    );
    // 所选根自己那条记录读得出就拿它当上一版的记录；读不出（两份都撕了）就拿最大 jsn 那条顶着——本实例的第一条反向链恒 0、
    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录（只做过 mkfs 的池环里一条都没有）。
    let own_record = records
        .values()
        .find(|record| {
            record.instance == effective_root.instance
                && record.checkpoint_txg == effective_root.checkpoint_txg
        })
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned();
    let previous = rebuild_previous_version(devices, &effective_root, own_record)?;
    // 环里没有记录时从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条：计数器全池接着走）。
    let next_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0)
        + 1;
    let first_txg = first_txg_of_new_instance(devices, &superblock, &records);
    let rebuilt = rebuilt_allocator(
        devices,
        &superblock,
        &previous,
        &|_| false,
        ShadowLedger::On,
    );
    let RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
    } = rebuilt?;
    let previous_row = PreviousInstanceRow {
        instance: effective_root.instance,
        selected_root_txg: effective_root.checkpoint_txg,
        applied_transaction_high_water: journal.maximum_applied_transaction,
        is_rollback: false,
    };
    establish_instance(
        parameters,
        devices,
        allocator,
        InstanceStart {
            chosen_root,
            effective_root,
            journal,
            previous,
            previous_row,
            first_txg,
            next_counter,
            effective_floor,
            abandoned_roots_unreadable,
        },
    )
}
```

### `mount_rollback`（959-1086）

```rust
/// 管理员回退（D23（journal 的角色与格式） 已定项 14 的显式例外）：带外选一条回退候选集里的旧根 R_old，不施加它之后的任何记录，
/// 取新实例代号，在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)，回退行、这次发布的单元与
/// 第一个新根同一次发布；只被被抛弃根引用的槽由影子账隔离（D28（挂载期承诺量） 已定项 1 第九项）；之后暖机同可写挂载。
/// 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 取 P2，预想、等用户定）。
///
/// # Errors
/// 目标根不在根环、不在候选集、它自己那条记录读不出、取号或发布失败。
pub fn mount_rollback<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let newest_root = choose_root(&*devices, &superblock).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(&*devices, &superblock);
    let roots = readable_roots(
        &*devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let target_root = roots
        .iter()
        .find(|root| {
            root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg
        })
        .copied()
        .ok_or(MountError::RollbackTargetNotInRing(target))?;
    // 候选集：按最新根指着的实例表判仍然有效——(i, T) 可选 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti；且 txg ≥ F_生效。
    let newest_table = instance_table_of_root(&*devices, &newest_root)
        .ok_or(MountError::InstanceTableMalformed)?;
    // txg ≥ F_生效（各幸存盘所带 F 最大值的最小值），不是最新根自己带的 F（步 4 / 步 5 代码三方第一轮正推腿判「窄化」）。
    let effective_floor = effective_rollback_floor(
        &*devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    if target.checkpoint_txg < effective_floor {
        return Err(MountError::RollbackTargetNotACandidate {
            target,
            reason: "txg 低于生效的回退下界 F",
        });
    }
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
    // 回退到树表 0 条的根（例如第一个事务里 txg 1、2 的暖机根）：回退行要重写实例表，没有文件版本的一版上它的落点记在哪、换下的 mkfs 实例表
    // 与候选根引用的单元谁护着都没有条款（D16（发布语义） 已定项 9 只定了树表 0 条时空发布写零个单元）——在任何写之前拒绝。
    if tree_table_has_no_entries(&*devices, &target_root)? {
        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));
    }
    // R_old 自己那条记录读得出就拿它当上一版的记录（只有 jsn 会被用到），读不出就拿最大 jsn 那条顶着，与可写挂载同一条路。
    let own_record = records
        .values()
        .find(|record| {
            record.instance == target.instance && record.checkpoint_txg == target.checkpoint_txg
        })
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
        .ok_or(MountError::FileVersionWithoutAnyJournalRecord)?;
    // 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 的 P2；预想、等用户定）：
    // 接 R_old 那条之后（P1）会把「B 的记录已提交、根还没落盘」那个窗口里 B 的记录盖掉，崩在回退生效之前那次恢复就不再
    // 「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中）。被抛弃的记录原样留在盘上，靠候选集与实例表挡。
    let highest_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0);
    let next_counter = highest_counter + 1;
    let previous = rebuild_previous_version(devices, &target_root, Some(own_record))?;
    // 不施加 R_old 之后的任何记录：扫描报告只记环里有多少条自证过的。
    let journal = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    let first_txg = first_txg_of_new_instance(devices, &superblock, &records);
    // 影子账：这次回退新抛弃的根 = 根环里 (txg, 实例) 大于 R_old 的每一条可读根；只被它们引用的槽隔离，
    // 连同按实例表早已被抛弃的根一起在重建里算。
    let newly_abandoned = |root: &RootRecord| {
        (root.checkpoint_txg, root.instance) > (target.checkpoint_txg, target.instance)
    };
    let RebuiltAllocator {
        allocator,
        effective_floor: _,
        abandoned_roots_unreadable,
    } = rebuilt_allocator(
        devices,
        &superblock,
        &previous,
        &newly_abandoned,
        shadow_ledger,
    )?;
    let previous_row = PreviousInstanceRow {
        instance: target.instance,
        selected_root_txg: target.checkpoint_txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    };
    establish_instance(
        parameters,
        devices,
        allocator,
        InstanceStart {
            chosen_root: newest_root,
            effective_root: target_root,
            journal,
            previous,
            previous_row,
            first_txg,
            next_counter,
            effective_floor,
            abandoned_roots_unreadable,
        },
    )
}
```

## 二、`crates/singlefs-core/src/recovery.rs`

### `RebuiltVersion`（458-468）

```rust
/// 从盘上按所选根重建出来的上一版。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只重建一个，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
pub enum RebuiltVersion {
    /// 树表 0 条：所选根下面还没有发布过文件版本（mkfs 的第 0 代根、第一次可写挂载的暖机根），这一版只有根自己指着的两个单元。
    WithoutFile,
    WithFile(TransactionOutput),
}
```

### `rebuild_version`（485-679）

```rust
/// 从盘上按所选根重建「上一版」：全部角色的单元字节、指针、树表、分配记录、记账行与 inode 记录，交给发布路径当上一版
/// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record_standing_for_root` 是所选根自己那条记录
/// （读不出时由调用方顶一条；树表 0 条时用不到）。
///
/// # Errors
/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；单元读不到或解不开 ⇒ 走读同款的错。
pub fn rebuild_version(
    reader: &dyn PoolReader,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersion, RebuildVersionFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let instance_table_bytes =
        read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    if tree_table.entries.is_empty() {
        return Ok(RebuiltVersion::WithoutFile);
    }
    let record =
        record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
    let record_bytes = record.to_bytes();
    let mut tree_table_entries = Vec::with_capacity(tree_table.entries.len());
    for bytes in &tree_table.entries {
        tree_table_entries.push(TreeTableEntry::parse(bytes).ok_or(
            RecoveryFailure::UnitMalformed {
                what: "树表条目"
            },
        )?);
    }
    let pointer_of = |kind: u16| {
        tree_table_entries
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.root)
            .ok_or(RecoveryFailure::UnitMalformed {
                what: "树表里没有那棵树",
            })
    };
    let extent_pointer = pointer_of(TREE_KIND_EXTENT)?;
    let inode_root_pointer = pointer_of(TREE_KIND_INODE)?;
    let allocation_pointer = pointer_of(TREE_KIND_ALLOCATION)?;
    let accounting_pointer = pointer_of(TREE_KIND_ACCOUNTING)?;
    let read_node = |pointer: &NodePointer, what: &'static str| {
        let bytes = read_unit_via_locations(reader, &pointer.locations, node_bytes)?;
        let node =
            parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
        Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
    };
    let (extent_bytes, extent_node) = read_node(&extent_pointer, "extent 树根")?;
    let (_, data_pointer) = parse_extent_record(extent_node.entries.first().ok_or(
        RecoveryFailure::UnitMalformed {
            what: "extent 树根没有记录",
        },
    )?);
    let data_unit_bytes = read_unit_via_locations(reader, &data_pointer.locations, data_bytes)?;
    let (inode_root_bytes, inode_root_node) = read_node(&inode_root_pointer, "inode 树根")?;
    let (_, _, inode_leaf_pointer) =
        parse_inode_internal_entry(inode_root_node.entries.first().ok_or(
            RecoveryFailure::UnitMalformed {
                what: "inode 树根没有条目",
            },
        )?);
    let inode_leaf_bytes =
        read_unit_via_locations(reader, &inode_leaf_pointer.locations, data_bytes)?;
    let inode_leaf =
        parse_packed_unit(&inode_leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "inode 叶容器",
        })?;
    let inode_record = InodeRecord::parse(inode_leaf.records.first().ok_or(
        RecoveryFailure::UnitMalformed {
            what: "inode 叶容器没有记录",
        },
    )?)
    .ok_or(RecoveryFailure::UnitMalformed {
        what: "inode 记录"
    })?;
    let (allocation_bytes, allocation_node) = read_node(&allocation_pointer, "分配记录树根")?;
    let allocation_records: Vec<AllocationRecord> = allocation_node
        .entries
        .iter()
        .map(|bytes| AllocationRecord::parse(bytes))
        .collect();
    let (accounting_bytes, accounting_node) = read_node(&accounting_pointer, "记账树根")?;
    let accounting_entries: Vec<AccountingEntry> = accounting_node
        .entries
        .iter()
        .map(|bytes| AccountingEntry::parse(bytes))
        .collect();
    let (mapping_bytes, mapping_node) = read_node(&root.mapping_root, "映射树根")?;
    let mapping_keys: Vec<Vec<u8>> = mapping_node
        .entries
        .iter()
        .map(|entry| parse_mapping_entry(entry).0)
        .collect();
    let mapped_units = vec![
        (
            TransactionUnit::Data,
            mapping_key_for_data(data_pointer.head, data_pointer.write_order),
        ),
        (
            TransactionUnit::ExtentRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
        ),
        (
            TransactionUnit::InodeLeaf,
            mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer),
        ),
        (
            TransactionUnit::InodeRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
        ),
        (
            TransactionUnit::AllocationTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
        ),
        (
            TransactionUnit::AccountingTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
        ),
    ];
    let unit =
        |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
            slot: locations[0].slot,
            identity,
            bytes,
        };
    let units = vec![
        unit(
            TransactionUnit::Data,
            &data_pointer.locations,
            data_unit_bytes,
        ),
        unit(
            TransactionUnit::ExtentRoot,
            &extent_pointer.locations,
            extent_bytes,
        ),
        unit(
            TransactionUnit::InodeLeaf,
            &inode_leaf_pointer.locations,
            inode_leaf_bytes,
        ),
        unit(
            TransactionUnit::InodeRoot,
            &inode_root_pointer.locations,
            inode_root_bytes,
        ),
        unit(
            TransactionUnit::AllocationTree,
            &allocation_pointer.locations,
            allocation_bytes,
        ),
        unit(
            TransactionUnit::AccountingTree,
            &accounting_pointer.locations,
            accounting_bytes,
        ),
        unit(
            TransactionUnit::MappingTree,
            &root.mapping_root.locations,
            mapping_bytes,
        ),
        unit(
            TransactionUnit::TreeTable,
            &root.tree_table.locations,
            tree_table_bytes,
        ),
        unit(
            TransactionUnit::InstanceTable,
            &root.instance_table.locations,
            instance_table_bytes,
        ),
    ];
    Ok(RebuiltVersion::WithFile(TransactionOutput {
        root: *root,
        record,
        record_bytes,
        units,
        rewritten: Vec::new(),
        data_pointer,
        mapping_keys,
        allocation_records,
        accounting_entries,
        tree_table_entries,
        inode_record,
        mapped_units,
        released: Vec::new(),
        key_order_mismatches: 0,
    }))
}
```

### `user_visible_tree_root_pointers`（698-741）

```rust
/// 读一条根的树表，取 inode 树与 extent 树的根指针盘上字节。
///
/// # Errors
/// 树表单元读不到、解不开；树表条目解不开、种类没登记，或同一种树出现两条。
pub fn user_visible_tree_root_pointers(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<UserVisibleTreeRootPointers, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
    for entry_bytes in &tree_table.entries {
        let entry = TreeTableEntry::parse(entry_bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表条目",
        })?;
        let slot_for_this_tree = match entry.kind {
            TREE_KIND_INODE => &mut pointers.inode_tree,
            TREE_KIND_EXTENT => &mut pointers.extent_tree,
            TREE_KIND_ALLOCATION
            | TREE_KIND_ACCOUNTING
            | TREE_KIND_LIVELIST
            | TREE_KIND_SPARSE_SIDE_TABLE
            | TREE_KIND_DEADLIST => continue,
            _ => {
                return Err(RecoveryFailure::UnitMalformed {
                    what: "树的种类没登记",
                })
            }
        };
        if slot_for_this_tree
            .replace(TreeTableEntry::root_pointer_bytes(entry_bytes).to_vec())
            .is_some()
        {
            return Err(RecoveryFailure::UnitMalformed {
                what: "树表里同一种树有两条",
            });
        }
    }
    Ok(pointers)
}
```

## 三、`crates/singlefs-core/src/transaction.rs`

### `instance_generation_to_acquire`（284-298）

```rust
/// 取号会取到的号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的实例代号) + 1（只读盘、不写）：可写挂载要在取号之前
/// 按它判这次要写的行区间（没有文件版本的一版上要写行就在任何写之前拒绝），`acquire_instance` 取的也是它。
#[must_use]
pub fn instance_generation_to_acquire<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
) -> InstanceGeneration {
    let highest_root = highest_root_instance(
        &*pool.devices,
        &pool.parameters.region_devices,
        &pool.parameters.geometry,
        &pool.parameters.filesystem_identifier,
    )
    .unwrap_or(MKFS_INSTANCE_GENERATION);
    InstanceGeneration(highest_superblock_instance(pool).max(highest_root).0 + 1)
}
```

### `acquire_instance`（300-323）

```rust
/// 取号（D23（journal 的角色与格式） 已定项 16、D18（块里携带什么信息） 已定项 11；2026-09-14 用户定案，
/// C322（取号那一步的屏障怎么放没有条款） 三轮三方）：新号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的
/// 实例代号) + 1；逐盘写一次超级块槽（世代号逐盘 +1）；两写之后一道屏障，屏障完成才把新号交出去，于是本实例的第一个
/// 非超级块写一定排在它之后。首次挂载路径上它就是暖机开场那道：暖机那道屏障前面没有写，写入口不再发第二次，录制流与设备
/// 收到的 FLUSH 数都与只发一道时相同。任一块盘的取号写或那道屏障报错 ⇒ 已写出的那几份回卷成旧代号（取号之前全部自证过的
/// 槽中最大的实例代号），报失败。⚠️ D18（块里携带什么信息） 已定项 11 的「重试到 T_retry 用尽才算失败」这里没有做：
/// 块设备报的第一个错就判失败（写路径上的重试全仓都还没有）。
pub fn acquire_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
) -> Result<InstanceGeneration, AcquisitionFailed> {
    let previous_instance = highest_superblock_instance(pool);
    let instance = instance_generation_to_acquire(pool);
    let mut written = Vec::new();
    for index in 0..pool.devices.len() {
        if let Err(cause) = pool.write_superblock_slot(index, 0, instance) {
            return Err(pool.roll_back_acquisition(&written, previous_instance, cause));
        }
        written.push(index);
    }
    if let Err(cause) = pool.perform(CommitStep::Barrier) {
        return Err(pool.roll_back_acquisition(&written, previous_instance, cause));
    }
    Ok(instance)
}
```

### `publish_without_units`（386-442）

```rust
/// 零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）：屏障 → 空记录 → 屏障 → 根槽 FUA → 超级块槽轮换；
/// 空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号与回退下界。
/// 第一次可写挂载的暖机与「只做过 mkfs 的池」上的可写挂载都走它（第一个事务的字节不变）。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn publish_without_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> Result<ZeroUnitPublishOutput, BlockDeviceError> {
    pool.perform(CommitStep::Barrier)?;
    let record = JournalRecord {
        instance: plan.instance,
        counter: plan.counter,
        checkpoint_txg: plan.txg,
        transaction: 0,
        is_commit: true,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
        new_tree_table: previous_root.tree_table,
        new_mapping_root: previous_root.mapping_root,
        new_tree_identifier_watermark: previous_root.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        named: Vec::new(),
    };
    let record_bytes = record.to_bytes();
    pool.perform(CommitStep::WriteJournalRecordToEveryDevice {
        counter: plan.counter,
        record: &record_bytes,
    })?;
    pool.perform(CommitStep::Barrier)?;
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        instance: plan.instance,
        checkpoint_txg: plan.txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: previous_root.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: previous_root.instance_table,
        mapping_root: previous_root.mapping_root,
    };
    let root_slot = root.to_slot(pool.root_slot_bytes());
    pool.perform(CommitStep::WriteRootRecordForceUnitAccess {
        checkpoint_txg: plan.txg,
        root_slot: &root_slot,
    })?;
    pool.perform(CommitStep::RotateSuperblockSlots {
        journal_tail: plan.counter,
        journal_instance: plan.instance,
    })?;
    Ok(ZeroUnitPublishOutput {
        root,
        record,
        record_bytes,
    })
}
```

### `PoolVersion`（含 impl，444-492）

```rust
/// 池里现行的那一版：还没发布过文件版本（树表 0 条，零单元发布写出的根），或带文件的一版。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有几个这样的值、不进集合，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
pub enum PoolVersion {
    WithoutFile(ZeroUnitPublishOutput),
    WithFile(TransactionOutput),
}

impl PoolVersion {
    #[must_use]
    pub fn root(&self) -> &RootRecord {
        match self {
            PoolVersion::WithoutFile(version) => &version.root,
            PoolVersion::WithFile(version) => &version.root,
        }
    }
    #[must_use]
    pub fn record(&self) -> &JournalRecord {
        match self {
            PoolVersion::WithoutFile(version) => &version.record,
            PoolVersion::WithFile(version) => &version.record,
        }
    }
    #[must_use]
    pub fn record_bytes(&self) -> &[u8] {
        match self {
            PoolVersion::WithoutFile(version) => &version.record_bytes,
            PoolVersion::WithFile(version) => &version.record_bytes,
        }
    }
    /// 带文件的那一版；还没发布过文件版本时 `None`。
    #[must_use]
    pub fn file_version(&self) -> Option<&TransactionOutput> {
        match self {
            PoolVersion::WithoutFile(_) => None,
            PoolVersion::WithFile(version) => Some(version),
        }
    }
    #[must_use]
    pub fn into_file_version(self) -> Option<TransactionOutput> {
        match self {
            PoolVersion::WithoutFile(_) => None,
            PoolVersion::WithFile(version) => Some(version),
        }
    }
}
```

### `publish_first_file`（947-979）

```rust
/// 第一个事务（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
pub fn publish_first_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    genesis: &RootRecord,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: FIRST_TRANSACTION_TXG,
            transaction: FIRST_TRANSACTION_NUMBER,
            instance,
            back_chain: back_chain_of(previous_record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: txg,
                change_count: 1,
            }),
            instance_table: InstanceTablePlan::Carry(genesis.instance_table),
            tree_birth_txg: txg,
            tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
            rollback_floor: CheckpointTxg(0),
        },
        None,
    )
}
```

## 四、新测试文件全文

### `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`（1-284，全文）

```rust
//! 里程碑「第二个事务」步 3 的另一格（2026-09-17 用户定：只做过 mkfs 的池允许可写挂载）：mkfs 之后进程退出、重开两个镜像走可写挂载——
//! 恢复择到 mkfs 的第 0 代根、环里一条记录都没有；取号 1；上一个实例是 0，不写行（D18（块里携带什么信息） 已定项 11）；
//! 树表 0 条 ⇒ 写行那次发布与暖机都写零个单元（D16（发布语义） 已定项 9），txg 1 落盘 1、txg 2 落盘 0；之后在同一个进程里发布第一个文件版本（txg 3）。
//! mkfs 之后的整条录制流与 mkfs 同一个进程里跑第一个事务逐字节相同（第一次可写挂载要写的区间是空的，第一个事务的字节不变）；
//! 冷启动读回那个文件；每一步的镜像池级 checker 一条违例都没有（没有文件的两步里没有记账树与 inode 树的 8 条报不适用）。

mod common;

use common::{
    build_pool, disk_snapshot, file_content, format_pool, parameters, FormattedPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::Placement;
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::{mount_writable, MountError, RollbackTarget};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
};

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 最新的根下面还没有记账树与 inode 树时（mkfs 之后、零单元发布之后）checker 报不适用的那 8 条；其余 18 条要真被评估过且成立。
const NOT_APPLICABLE_WITHOUT_FILE: [&str; 8] = [
    "I-3.1", "I-5.2", "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10", "I-9.13",
];

/// 这一步的镜像上 checker 的 26 条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
fn assert_checker_verdicts(pool: &FormattedPool, step: &str, not_applicable: &[&str]) {
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        verdicts.len(),
        singlefs_checker::image::IMPLEMENTED_INVARIANTS.len()
    );
    for (invariant, verdict) in &verdicts {
        if not_applicable.contains(invariant) {
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "{step}：{invariant} 该报不适用：{verdict:?}"
            );
        } else {
            assert_eq!(
                *verdict,
                InvariantVerdict::Holds,
                "{step}：{invariant} 要真被评估过且成立"
            );
        }
    }
}

/// 没有文件版本的一版上要写实例表行，第一版不支持（设计没定），在取号之前拒绝：第一个事务取号、暖机两次之后、第一个文件版本之前崩溃
/// （同一个进程里做到暖机就停、镜像关掉），重开可写挂载——所选根 (1, 2)、树表 0 条，要取的号 2，要写的行区间 [1, 2) 不为空 ⇒
/// 返回 `InstanceRowsOnVersionWithoutFileUnsupported`；两盘超级块四个槽逐字节不变（实例代号仍是 1）、根环没有新根、录制流一步都没多。
#[test]
fn writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance(
) {
    let mut formatted = format_pool("step-three-formatted-crash-after-warm-up");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        warm_up(&mut writer, &formatted.genesis.root, instance).expect("暖机");
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        before
            .readable_roots
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .max(),
        Some((InstanceGeneration(1), CheckpointTxg(2))),
        "崩溃点：暖机两次的根都在盘上"
    );
    let mut devices = formatted.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(devices);
    assert!(
        matches!(
            refused,
            Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported {
                chosen_root: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(2)
                },
                first_row_instance: InstanceGeneration(1),
                instance_to_acquire: InstanceGeneration(2),
            })
        ),
        "树表 0 条的一版上要写 (1, 2, 0)：{:?}",
        refused.as_ref().err()
    );
    let after = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        after.superblock_slots, before.superblock_slots,
        "两盘超级块槽逐字节不变：没有取号"
    );
    assert_eq!(after.readable_roots, before.readable_roots, "根环没有新根");
    assert_eq!(
        after.recorded_operations, before.recorded_operations,
        "一个写、一道屏障都没发"
    );
}

/// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
/// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
/// mkfs 之后的录制流与第一个事务那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有——mkfs 之后与挂载之后
/// 18 条成立、没有记账树与 inode 树的 8 条报不适用，第一个文件版本之后 26 条全成立。
#[test]
fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
) {
    let mut formatted = format_pool("step-three-formatted-pool");
    assert_checker_verdicts(&formatted, "mkfs 之后", &NOT_APPLICABLE_WITHOUT_FILE);

    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    formatted.devices = Some(devices);
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(1),
        "取号 = max(超级块 0, 根环 0) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(0), CheckpointTxg(0)),
        "所选根是 mkfs 的第 0 代根"
    );
    assert_eq!(output.effective_root, output.chosen_root);
    assert_eq!(output.journal.valid_records, 0, "环里一条记录都没有");
    assert!(output.rows_written.is_empty(), "上一个实例是 0：不写行");
    let PoolVersion::WithoutFile(row) = &output.row_publish else {
        panic!("树表 0 条：写行那次发布写零个单元")
    };
    assert_eq!(
        (
            row.root.instance,
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.transaction,
            row.record.back_chain,
            row.record.named.len()
        ),
        (InstanceGeneration(1), CheckpointTxg(1), 1, 0, 0, 0),
        "txg = max(根环 0, 记录无) + 1；jsn 从 1 起；事务号 0；本实例第一条反向链 0；不点名"
    );
    assert_eq!(
        (
            row.root.tree_table,
            row.root.instance_table,
            row.root.mapping_root
        ),
        (
            formatted.genesis.root.tree_table,
            formatted.genesis.root.instance_table,
            formatted.genesis.root.mapping_root
        ),
        "根记录照 mkfs 的根：树表、实例表、映射树根都没换"
    );
    assert_eq!(region_device(1), DeviceIdentity(1), "txg 1 落盘 1");
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "txg 2 落盘 0，一次就覆盖两块盘"
    );
    let PoolVersion::WithoutFile(warm_up) = &output.warm_up_publishes[0] else {
        panic!("树表 0 条：暖机写零个单元")
    };
    assert_eq!(
        (
            warm_up.root.checkpoint_txg,
            warm_up.record.counter,
            warm_up.record.transaction,
            warm_up.record.back_chain
        ),
        (CheckpointTxg(2), 2, 0, back_chain_of(&row.record_bytes))
    );
    assert_eq!(region_device(2), DeviceIdentity(0), "txg 2 落盘 0");
    assert_eq!(
        mounted.current, output.warm_up_publishes[0],
        "接下来的发布接在 txg 2 后面"
    );
    assert_eq!(
        output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "没有被抛弃的根"
    );
    for device_map in &mounted.allocator.devices {
        assert_eq!(
            (device_map.allocated_slots(), device_map.deferred_slots()),
            (3, 0),
            "只有 mkfs 的实例表 2 槽与树表 1 槽"
        );
    }
    assert_checker_verdicts(&formatted, "可写挂载之后", &NOT_APPLICABLE_WITHOUT_FILE);
    assert_eq!(
        recover(&formatted.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2))
        }
    );

    // 第一个文件版本：接在 txg 2 那条零单元发布后面。
    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (
            first.root.instance,
            first.root.checkpoint_txg,
            first.record.counter,
            first.record.transaction,
            first.record.back_chain
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(3),
            3,
            1,
            back_chain_of(current.record_bytes())
        )
    );
    assert_eq!(
        first.data_pointer.locations[0].slot,
        SlotNumber(50180),
        "mkfs 的两个单元占着 50176–50178"
    );
    assert_eq!(
        first.released,
        vec![Placement {
            slot: SlotNumber(50178),
            span: 1
        }],
        "换下 mkfs 那片第 0 版树表"
    );

    let reference = build_pool("step-three-formatted-pool-reference");
    assert_eq!(
        formatted.retained_operations()[formatted.mkfs_operation_count..],
        reference.retained_operations()[reference.mkfs_operation_count..],
        "mkfs 之后的录制流与 mkfs 同一个进程里跑第一个事务逐字节相同"
    );
    assert_eq!(first, reference.output, "写出的那一版也相同");

    assert_checker_verdicts(&formatted, "第一个文件版本之后", &[]);
    let reopened = formatted.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content
        }
    );
    assert_eq!(report.journal.valid_records, 3);
}
```

### `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs`（1-211，全文）

```rust
//! 只做过 mkfs 的池上的可写挂载（里程碑「第二个事务」步 3 的另一格，2026-09-17 用户定）在层 0 的证据：mkfs → 进程退出、重开走可写挂载
//! （取号 1、零单元的写行发布 txg 1 与暖机 txg 2）→ 同一个进程里发布第一个文件版本（txg 3），把 mkfs 之后的整条录制流按 D13（验证路线） 已定项 4
//! 枚举全部崩溃状态，每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器。段序列与 mkfs 同一个进程里跑第一个事务相同。
//! 平时 `cargo test` 不展开 18 写的那一段；全量那条标 ignored，在 release 下跑（打印 `LAYER0F` 行）。

mod common;

use common::{file_content, format_pool, geometry, parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions, enumerate_layer0_versions,
    writes_and_segments, Layer0Tally, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// 被判的那次根槽 FUA 写：第一个文件版本的根。
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn prepare(tag: &str) -> Prepared {
    let mut formatted = format_pool(tag);
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    let mut allocator = mounted.allocator;
    let content = file_content();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            mounted.current.record_bytes(),
        )
        .expect("第一个文件版本");
    }
    formatted.devices = Some(devices);
    let base = formatted.memory_pool_after_mkfs();
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 2],
        "取号两写 | 写行发布的记录两写 | 根 | 超级块两写 | 暖机的记录两写 | 根 | 超级块两写与第一个文件版本的十六个单元写 | 记录两写 | 根 | 超级块两写"
    );
    assert_eq!(
        writes.len(),
        33,
        "取号 2 + 两次零单元发布各 5 + 文件版本 21"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 3, "txg 1、2、3 各一条根槽写");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("三条根槽写"),
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content,
        }],
    }
}

fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据在已定的持久顺序下恒 0"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例的状态数必须是 0：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    for must_evaluate in ["I-3.1", "I-5.2", "I-5.1", "I-7.2", "I-7.7"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过"
        );
    }
}

/// 段序列：十段、闭式 262165（1 + 六个 2 写段各 3 + 三个 1 写段各 1 + 一个 18 写段 262143），与 mkfs 同一个进程里跑第一个事务那条流相同。
#[test]
fn the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence() {
    let prepared = prepare("formatted-layer0-registered");
    assert_eq!(prepared.segments.len(), 10);
    assert_eq!(closed_form_state_count(&prepared.segments), 262_165);
}

/// 平时跑的那一份：18 写的那一段不展开（只以整段持久进入后面的状态），其余每段任意子集。
#[test]
fn every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims(
) {
    let prepared = prepare("formatted-layer0-fast");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 10)
        .cloned()
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "展开的段按闭式数"
    );
    assert_eq!(tally.states, 22, "1 + 六个 2 写段各 3 + 三个 1 写段各 1");
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    assert!(tally.file_read_states > 0 && tally.no_file_states > 0);
    assert_checker_and_record_checker_clean(&tally);
}

/// 全量：十段、闭式 262165 个状态，每个两遍恢复 + checker；在 release 下跑，认下面打印的 `LAYER0F` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量 262165 个状态、每个两遍恢复 + checker，debug 下太慢；在 release 下跑"]
fn full_enumeration_of_the_formatted_pool_mount_stream_is_exhaustive_and_clean() {
    let prepared = prepare("formatted-layer0-full");
    let closed_form = closed_form_state_count(&prepared.segments);
    assert_eq!(closed_form, 262_165, "闭式：1 + Σ(2^|段| − 1)，十段");
    let tally = enumerate_layer0_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
    );
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    println!(
        "LAYER0F states={} closed_form={closed_form} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    assert_checker_and_record_checker_clean(&tally);
}
```

## 五、`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 新加的三条用例

### `torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling`（317-362）

```rust
/// 「非空」从盘上认（D16（发布语义） 已定项 1，2026-09-17 用户定案）：比树表里 inode 树与 extent 树的根指针，不看 journal 记录——
/// 把 txg 14 那次覆盖写的记录在两块盘上都改坏（根槽与单元不动），非空的有效根仍是 14、13、12、11，上限仍是 11，抬到 12 被拒；
/// 按「环里有它自己那条记录且事务号非 0」认的话 txg 14 成了空根，第 4 新的非空根掉到 A 的 3，上限掉到 3。
#[test]
fn torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling() {
    let mut pool = build_through_rollback("step-five-ceiling-torn-record");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    let fourth = &overwrites[3];
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(14), 14)
    );
    {
        let offset = record_offset(
            fourth.record.counter,
            parameters().geometry.journal_ring_bytes,
        );
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for (_, recorded) in devices.iter_mut() {
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读记录");
            bytes[300] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏记录");
        }
    }
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    assert!(
        !scan_journal(&image, &superblock).contains_key(&(InstanceGeneration(3), 14)),
        "txg 14 那条记录两份都读不出"
    );
    let refused = raise_floor(&mut pool, CheckpointTxg(12));
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackFloorAboveCeiling {
                requested: CheckpointTxg(12),
                ceiling: CheckpointTxg(11)
            })
        ),
        "txg 14 的记录坏了，它的树表照样与 13 的不同：上限仍是 11：{:?}",
        refused.as_ref().err()
    );
}
```

### `the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it`（364-391）

```rust
/// 「前一条」只在有效根里找：回退那次发布 D（txg 9）照抄 A 的文件，树表里两棵树的根指针与前一条有效根 A（txg 3）的相同 ⇒ 空；
/// 夹在 A 与 D 之间的被抛弃根 C（txg 8）带着第三次的内容，拿它比 D 就成了非空。回退之后覆盖写三次（txg 11、12、13）：
/// 非空的有效根是 13、12、11、3，第 4 新的是 3，每块盘上最新的有效根是 12（盘 0）与 13（盘 1），上限 min(12, 3) = 3，抬到 4 被拒。
#[test]
fn the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it(
) {
    let mut pool = build_through_rollback("step-five-ceiling-previous-valid-root");
    for seed in [17usize, 19, 23] {
        overwrite_in_process(
            &mut pool,
            &content_of(3000 + seed, seed),
            InstanceGeneration(3),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(13));
    let refused = raise_floor(&mut pool, CheckpointTxg(4));
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackFloorAboveCeiling {
                requested: CheckpointTxg(4),
                ceiling: CheckpointTxg(3)
            })
        ),
        "D 不算非空、第 4 新的非空有效根是 A：{:?}",
        refused.as_ref().err()
    );
}
```

### `raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable`（423-481）

```rust
/// 算上限时一条有效根的树表读不出：拒绝抬 F，不按空或非空猜（用户：读不出就要走修复、不能跳过）——回退之后覆盖写四次，
/// 把有效根 txg 12 的树表两盘都改坏，抬 F 到 11 ⇒ 返回 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（点名 (3, 12)）；
/// 分配器、现行那一版、录制流都不变（没回收、没发布）。按「读不出算空」猜的话 txg 12、13 都与前一条不同，上限照样是 11、抬 F 成功。
#[test]
fn raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable() {
    let mut pool = build_through_rollback("step-five-ceiling-unreadable-valid-root");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    let damaged = &overwrites[1];
    assert_eq!(damaged.root.checkpoint_txg, CheckpointTxg(12));
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for location in &damaged.root.tree_table.locations {
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == location.device)
                .expect("txg 12 的树表所在的盘");
            let offset = location.slot.to_device_offset();
            let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
            recorded
                .read_at(offset, &mut bytes)
                .expect("读 txg 12 的树表");
            bytes[200] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏 txg 12 的树表");
        }
    }
    let allocator_before = allocator_state(&pool);
    let current_before = pool.output.clone();
    let recorded_before = pool.stream.operations().len();
    let refused = raise_floor(&mut pool, CheckpointTxg(11));
    assert!(
        matches!(
            refused,
            Err(
                MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
                    root: RollbackTarget {
                        instance: InstanceGeneration(3),
                        checkpoint_txg: CheckpointTxg(12)
                    },
                    ..
                }
            )
        ),
        "有效根 txg 12 的树表读不出：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        allocator_state(&pool),
        allocator_before,
        "没有回收、没有隔离"
    );
    assert_eq!(pool.output, current_before, "现行那一版没动");
    assert_eq!(
        pool.stream.operations().len(),
        recorded_before,
        "一个写、一道屏障都没发"
    );
}
```

**注**：同文件里 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable`（623 行起）名字里也含
`unreadable`，但它不是这一轮新加的三条之一（实现员报告 `research/prompts/m2-emptypool-nonempty-r1-implementer-report.md`
第 94-95 行只把 `torn_journal_record…`、`…compared_with_the_previous_valid_root…`、
`raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable` 三个记成 T1 / T2 / U3 这一轮的新用例，
第 623 行那条在 T2 行里只被当作既有上下文提到），未抄入本附录。

## 六、`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 里的用例

### `rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write`（84-110）

```rust
/// 回退到树表 0 条的根（第一个事务里 txg 2 的暖机根），第一版不支持（设计没定），在任何写之前拒绝：第一个事务之后进程退出、重开回退到 (1, 2)
/// ⇒ 返回 `RollbackToVersionWithoutFileUnsupported`；两盘超级块槽逐字节不变、根环没有新根、录制流一步都没多。
#[test]
fn rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write() {
    let mut pool = build_pool("step-four-rollback-to-warm-up-root");
    let warm_up_root = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    };
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut devices = pool.reopen_recorded();
    let refused = mount_rollback(&parameters(), &mut devices, warm_up_root, ShadowLedger::On);
    pool.devices = Some(devices);
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackToVersionWithoutFileUnsupported(target)) if target == warm_up_root
        ),
        "暖机根下面没有文件版本：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "盘上逐字节不变"
    );
}
```

## 七、`crates/singlefs-harness/tests/common/mod.rs` 的 `DiskSnapshot` 与 `disk_snapshot`（242-280）

```rust
/// 盘上可比的一份快照：两盘各两个超级块槽的原样字节、根环里全部自证过的根、录制流里已有几步。挂载或抬 F 被拒之后与拒之前逐项相等，
/// 才算「在任何写之前拒绝」（录制流不多一步 = 一个写、一道屏障都没发）。
#[derive(Debug, PartialEq, Eq)]
pub struct DiskSnapshot {
    pub superblock_slots: Vec<Vec<u8>>,
    pub readable_roots: Vec<singlefs_core::root_record::RootRecord>,
    pub recorded_operations: usize,
}

pub fn disk_snapshot(image: &MemoryPool, stream: &SharedStream) -> DiskSnapshot {
    use singlefs_core::recovery::PoolReader;
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(singlefs_format::SUPERBLOCK_SLOT_BYTES).expect("4096");
    let mut superblock_slots = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, spacing] {
            superblock_slots.push(
                PoolReader::read(
                    image,
                    device,
                    singlefs_core::address::DeviceOffsetInBytes(offset),
                    slot_bytes,
                )
                .expect("超级块槽读得到"),
            );
        }
    }
    let superblock = singlefs_core::recovery::choose_superblock(image).expect("超级块");
    DiskSnapshot {
        superblock_slots,
        readable_roots: singlefs_core::recovery::readable_roots(
            image,
            &superblock.region_devices,
            &superblock.geometry,
            &superblock.filesystem_identifier,
        ),
        recorded_operations: stream.operations().len(),
    }
}
```

## 八、`crates/mutations.tsv` 第 62–71 行

第 1 行是表头（列含义），一并附上做参照，不算在「第 62–71 行」之内：

```tsv
# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。
```

第 62-71 行（十条，六段制表符分隔：变异名 / 文件 / 原文 / 替换文 / cargo test 的参数 / 必须红的测试名）：

```tsv
步 5：「非空」退回按环里记录的事务号认（txg 14 的记录两份都坏了它就算空根，上限掉到 3）	crates/singlefs-core/src/mount.rs	        if user_visible_trees_changed(&root_pointers, &previous_pointers) {	        if scan_journal(devices, superblock).values().any(|record| {\n            record.instance == root.instance\n                && record.checkpoint_txg == root.checkpoint_txg\n                && record.transaction != 0\n        }) {	-p singlefs-harness --test second_transaction_step_five_reuse -- torn_journal_record_does_not_turn	torn_journal_record_does_not_turn_its_root_into_an_empty_root_for_the_floor_ceiling
步 5：「非空」的前一条取任意可读根（不排除被抛弃的根与 F 之下的根）	crates/singlefs-core/src/mount.rs	    let previous_candidates: &[RootRecord] = &valid;	    let previous_candidates: &[RootRecord] = &readable;	-p singlefs-harness --test second_transaction_step_five_reuse -- the_rollback_publish_is_compared	the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it
步 5：「非空」只比 inode 树的根指针、不比 extent 树的	crates/singlefs-core/src/mount.rs	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree	-p singlefs-core --lib -- root_is_non_empty_when_either	root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root
步 3：只做过 mkfs 的池重开后分配器不认 mkfs 写在单元区里的两个单元	crates/singlefs-core/src/mount.rs	    allocator.mark_format_time_units(\n        placement_of(&root.instance_table, TransactionUnit::InstanceTable),\n        placement_of(&root.tree_table, TransactionUnit::TreeTable),\n    );	    let _ = (\n        placement_of(&root.instance_table, TransactionUnit::InstanceTable),\n        placement_of(&root.tree_table, TransactionUnit::TreeTable),\n    );	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
步 3：只做过 mkfs 的池上零单元暖机的反向链写 0	crates/singlefs-core/src/mount.rs	                back_chain: back_chain_of(&current_version_without_file.record_bytes),	                back_chain: 0,	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
步 3：零单元发布在记录与根之间少一道屏障	crates/singlefs-core/src/transaction.rs	    pool.perform(CommitStep::Barrier)?;\n    let root = RootRecord {\n        filesystem_identifier: previous_root.filesystem_identifier,	    let root = RootRecord {\n        filesystem_identifier: previous_root.filesystem_identifier,	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream	the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence
步 3：只做过 mkfs 的池重开后把 mkfs 实例表按 1 槽记	crates/singlefs-core/src/mount.rs	        placement_of(&root.instance_table, TransactionUnit::InstanceTable),	        placement_of(&root.instance_table, TransactionUnit::TreeTable),	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
步 3：树表 0 条的一版上要写行时，拒绝挪到取号之后（超级块已写进新号才返回）	crates/singlefs-core/src/mount.rs	    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;\n    let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;	    let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;\n    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_between_warm_up	writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance
步 4：回退到树表 0 条的根不在写之前拒绝	crates/singlefs-core/src/mount.rs	    if tree_table_has_no_entries(&*devices, &target_root)? {	    if false && tree_table_has_no_entries(&*devices, &target_root)? {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root	rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write
步 5：算抬 F 上限时有效根的树表读不出，按「树表里没有这两棵树」猜而不是拒绝	crates/singlefs-core/src/mount.rs	        Ok(pointers) => Ok(pointers),\n        Err(failure) => Err(	        Ok(pointers) => Ok(pointers),\n        Err(_failure) if true => Ok(UserVisibleTreeRootPointers::ABSENT),\n        Err(failure) => Err(	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_is_refused_when	raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable
```
