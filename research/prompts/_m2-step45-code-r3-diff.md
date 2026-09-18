# Diff：m2-step45-code-r3（2026-09-17）

第二轮判决（`research/prompts/m2-step45-code-r2-main-verification.md`）与 alloc-basis 第二轮云端攻方腿（`research/prompts/alloc-basis-r2-opus-output.md` 2.1 节、第七节第 1 条）打中之后改的代码，整段抄自工作区（未提交，与第二轮之间没有提交点，所以不是 `git diff`）。每段前写文件名与行区间，用 `awk 'NR==A,NR==B'` 现取，行号现查（2026-09-17）。

### `crates/singlefs-core/src/mount.rs` 第 29-52 行——MountError 枚举

```rust
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面还没有发布过文件版本（树表为空）：第一版的可写挂载只接在有文件的池后面，刚 mkfs 的池走第一次可写挂载那条路。
    NoPublishedVersion,
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
}
```

### `crates/singlefs-core/src/mount.rs` 第 81-102 行——MountOutput 结构体

```rust
/// 一次可写挂载写出的东西。
#[derive(Debug)]
pub struct MountOutput {
    pub instance: InstanceGeneration,
    /// 恢复择到的根（施加前缀之前）。
    pub chosen_root: RootRecord,
    /// 施加前缀之后的根：写行与照抄都以它为准。
    pub effective_root: RootRecord,
    pub journal: JournalScanReport,
    /// 这次挂载写进实例表的行（上一个实例那一行，中间实例各一行）。
    pub rows_written: Vec<InstanceRow>,
    /// 写行那次发布（本实例的第一次发布）。
    pub row_publish: TransactionOutput,
    /// 之后的暖机空发布，直到本实例的根覆盖每块盘。
    pub warm_up_publishes: Vec<TransactionOutput>,
    /// 影子账隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」，只住内存）：只被被抛弃根引用的槽，
    /// 仍被候选集里的根引用的不在其内（对用户 2026-09-16 定的窄读法措辞的收严，见 `isolate_slots_referenced_only_by_abandoned_roots`；预想）。
    pub isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
    /// 被抛弃根里树表或分配记录树读不出、解不开的条数：这样的根影子账罩不到，只计数、不拒绝挂载
    /// （步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
    pub abandoned_roots_unreadable: u64,
}
```

### `crates/singlefs-core/src/mount.rs` 第 195-199 行——RebuiltAllocator 结构体

```rust
struct RebuiltAllocator {
    allocator: PoolAllocator,
    effective_floor: CheckpointTxg,
    abandoned_roots_unreadable: u64,
}
```

### `crates/singlefs-core/src/mount.rs` 第 201-258 行——isolate_slots_referenced_only_by_abandoned_roots 整个函数（含文档注释与 allow）

```rust
/// 影子账（D28（挂载期承诺量） 已定项 1 第九项）：只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里
/// 还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F。用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，
/// 豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严（alloc-basis 那一轮的 G5 臂）：按原措辞，抬 F 到 11 之后
/// A 仍豁免 mkfs 实例表那 2 槽，它们被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句。
/// 预想、偏离用户定案的措辞，交用户。
/// 一条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
/// 树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载；候选根读不出就当它什么都不豁免（隔离只会多不会少）。
/// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    floor: CheckpointTxg,
    current_records: &[AllocationRecord],
) -> u64 {
    let mut referenced_by_candidates: BTreeSet<(u32, u64)> = current_records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    for root in roots
        .iter()
        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
    {
        if let Ok(records) = allocation_records_under_root(devices, root) {
            referenced_by_candidates.extend(
                records
                    .iter()
                    .filter(|record| !record.is_released)
                    .map(|record| (record.device.0, record.slot.0)),
            );
        }
    }
    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut unreadable = 0;
    for root in roots.iter().filter(|root| is_abandoned(root)) {
        let Ok(records) = allocation_records_under_root(devices, root) else {
            unreadable += 1;
            continue;
        };
        for record in records {
            let key = (record.device.0, record.slot.0);
            if record.is_released
                || referenced_by_candidates.contains(&key)
                || !isolated.insert(key)
            {
                continue;
            }
            allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));
        }
    }
    unreadable
}
```

### `crates/singlefs-core/src/mount.rs` 第 260-325 行——rebuilt_allocator 整个函数（含文档注释与 allow）

```rust
/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收，再把只被被抛弃根引用的槽隔离（影子账；
/// `extra_abandoned` 是这次挂载新抛弃的根——回退时是 (txg, 实例) 大于 R_old 的那些，普通挂载没有）。
/// 影子账只住内存，所以每次挂载都要重算，不只回退那一次（步 4 / 步 5 代码三方第一轮云端攻方腿打中：回退之后普通重开一次隔离就归零）。
#[allow(
    clippy::ptr_arg,
    reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn rebuilt_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    superblock: &crate::superblock::Superblock,
    previous: &TransactionOutput,
    extra_abandoned: &dyn Fn(&RootRecord) -> bool,
    shadow_ledger: ShadowLedger,
) -> RebuiltAllocator {
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
        .collect();
    let mut allocator =
        PoolAllocator::rebuild_from_records(device_maps, previous.allocation_records.clone());
    let effective_floor = effective_rollback_floor(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let roots = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let newest_table = choose_root(devices, superblock)
        .and_then(|newest| instance_table_of_root(devices, &newest));
    let is_abandoned = |root: &RootRecord| {
        extra_abandoned(root)
            || newest_table
                .as_ref()
                .is_some_and(|table| abandoned_by_table(root, table))
    };
    let oldest_valid_root = roots
        .iter()
        .filter(|root| !is_abandoned(root))
        .map(|root| root.checkpoint_txg)
        .min();
    allocator.reclaim_released_up_to(
        reclaim_floor(effective_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    let abandoned_roots_unreadable = match shadow_ledger {
        ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            &mut allocator,
            &roots,
            &is_abandoned,
            effective_floor,
            &previous.allocation_records,
        ),
        ShadowLedger::Off => 0,
    };
    RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
    }
}
```

### `crates/singlefs-core/src/mount.rs` 第 392-507 行——raise_rollback_floor 整个函数（含文档注释）

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
    let records = scan_journal(&*devices, &superblock);
    let instance_table_unit = current
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::InstanceTable)
        .ok_or(MountError::RaiseNeedsWritableMountInThisProcess)?;
    let table = InstanceTableRecords::parse(&instance_table_unit.bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    let ceiling = rollback_floor_ceiling(
        devices,
        &superblock,
        &records,
        current.root.rollback_floor,
        &table,
    )
    .ok_or(RecoveryFailure::NoValidRoot)?;
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
    if shadow_ledger == ShadowLedger::On {
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
        );
    }
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
    })
}
```

### `crates/singlefs-core/src/mount.rs` 第 790-811 行——mount_rollback 里从「影子账：」注释到 establish_instance( 调用之前

```rust
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
    );
    let previous_row = PreviousInstanceRow {
        instance: target.instance,
        selected_root_txg: target.checkpoint_txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    };
```

### `crates/singlefs-core/src/allocator.rs` 第 100-127 行——DeviceFreeMap 结构体定义

```rust
pub struct DeviceFreeMap {
    pub device: DeviceIdentity,
    unit_area_slots: u64,
    /// 每个 16 KiB 槽一位：true = 已分配。
    allocated: Vec<bool>,
    /// 每个 64 槽聚簇段里已分配的槽数（增量维护全空段数用）。
    used_per_segment: Vec<u64>,
    /// 空闲槽的连续段数（增量维护）。
    free_runs: u64,
    /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
    /// （I-3.1（已分配统计对得上） 的读法 2026-09-14 用户定甲：按根环里全部有效根的引用取并集）。
    allocated_slots: u64,
    /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
    /// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。
    free_slots: u64,
    /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段；D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」）：
    /// 只被被抛弃根引用的槽，当前账里既不是已分配也不是 defer，分配器却不许发出去，直到被抛弃的根离开根环。只住内存、不进记账行。
    isolated: Vec<bool>,
    isolated_per_segment: Vec<u64>,
    isolated_slots: u64,
    /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
    deferred_slots: u64,
    /// 抬 F 回收、但 F 还没在每块盘上生效的槽：记账已经算它空闲，分配器却不许发出去，直到带新 F 的根落满每块盘
    /// （D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」；alloc-basis 第二轮云端攻方腿打中：抬 F 自己的空发布开新段时
    /// 正好开到刚回收空的那一段，崩在两条带新 F 的根之间时 F 之下的根仍是候选、它们的单元已被盖）。只住内存。
    held_until_floor_takes_effect: Vec<bool>,
    held_per_segment: Vec<u64>,
}
```

### `crates/singlefs-core/src/allocator.rs` 第 130-154 行——DeviceFreeMap::new

```rust
    /// 单元区从 50176 起到设备末尾；mkfs 占的槽由调用方标上。
    #[must_use]
    pub fn new(device: DeviceIdentity, device_bytes: u64) -> Self {
        let unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT;
        let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
        Self {
            device,
            unit_area_slots,
            allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            used_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            free_runs: u64::from(unit_area_slots > 0),
            allocated_slots: 0,
            free_slots: unit_area_slots,
            deferred_slots: 0,
            isolated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            isolated_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            isolated_slots: 0,
            held_until_floor_takes_effect: vec![
                false;
                usize::try_from(unit_area_slots)
                    .expect("单元区槽数")
            ],
            held_per_segment: vec![0; usize::try_from(segments).expect("段数")],
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 160-167 行——DeviceFreeMap::is_free

```rust
    #[must_use]
    pub fn is_free(&self, slot: SlotNumber) -> bool {
        slot.0 >= UNIT_AREA_START_SLOT
            && slot.0 < UNIT_AREA_START_SLOT + self.unit_area_slots
            && !self.allocated[Self::index(slot)]
            && !self.isolated[Self::index(slot)]
            && !self.held_until_floor_takes_effect[Self::index(slot)]
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 197-212 行——DeviceFreeMap::isolate

```rust
    /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。
    /// 隔离位独立于分配位：一个槽在当前账里已分配、后来被释放、再被回收，隔离位照样让它发不出去——抬 F 之后候选集缩小，
    /// 一个此前靠候选根豁免的槽会在回收之前被补隔离（`mount::raise_rollback_floor`）。同一个槽隔离两次不重复计数。
    pub fn isolate(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        for index in start..end {
            if !self.isolated[index] {
                self.isolated[index] = true;
                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
                self.isolated_per_segment[segment] += 1;
                self.isolated_slots += 1;
            }
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 220-232 行——DeviceFreeMap::hold_until_floor_takes_effect

```rust
    /// 抬 F 回收的落点先扣住：空闲计数照加，位图照清，但 `is_free` 与开段都绕开它，直到 `release_holds`。
    pub fn hold_until_floor_takes_effect(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        for index in start..end {
            if !self.held_until_floor_takes_effect[index] {
                self.held_until_floor_takes_effect[index] = true;
                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
                self.held_per_segment[segment] += 1;
            }
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 234-238 行——DeviceFreeMap::release_holds

```rust
    /// F 在每块盘上生效之后把扣住的槽放开。
    pub fn release_holds(&mut self) {
        self.held_until_floor_takes_effect.fill(false);
        self.held_per_segment.fill(0);
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 240-261 行——DeviceFreeMap::mark_reclaimed

```rust
    /// 回收：一个已释放、释放代 ≤ max(F_生效, 环里最旧有效根) 的落点回到空闲（D16（发布语义） 已定项 1 的可再分配谓词）：
    /// 位图清掉、占着的槽数与 defer 队列各减、空闲加——记账的空闲字节到这一刻才动（里程碑「第二个事务」步 5）。
    pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "回收的跨度里有没分配的槽"
        );
        let left_free = start > 0 && !self.allocated[start - 1];
        let right_free = end < self.allocated.len() && !self.allocated[end];
        // runs 的增量：两边都空是把两段并成一段（−1），两边都占是新开一段（+1），一边空是接上去（不变）。
        self.free_runs = self.free_runs + 1 - u64::from(left_free) - u64::from(right_free);
        for index in start..end {
            self.allocated[index] = false;
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.used_per_segment[segment] -= 1;
        }
        self.allocated_slots -= span;
        self.deferred_slots -= span;
        self.free_slots += span;
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 327-344 行——DeviceFreeMap::lowest_empty_segment

```rust
    /// 单元区内最低的 64 槽对齐全空段的起点。
    #[must_use]
    pub fn lowest_empty_segment(&self) -> Option<SlotNumber> {
        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let full_segments = self.allocated.len() / per_segment;
        (0..full_segments)
            .find(|segment| {
                self.used_per_segment[*segment] == 0
                    && self.isolated_per_segment[*segment] == 0
                    && self.held_per_segment[*segment] == 0
            })
            .map(|segment| {
                SlotNumber(
                    UNIT_AREA_START_SLOT
                        + u64::try_from(segment).expect("段号") * CLUSTER_SEGMENT_SLOTS,
                )
            })
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 347-352 行——ReclaimedReuse 枚举

```rust
/// 回收的槽什么时候能再发出去：重建分配器时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReclaimedReuse {
    Immediately,
    HeldUntilFloorTakesEffect,
}
```

### `crates/singlefs-core/src/allocator.rs` 第 533-561 行——PoolAllocator::allocate_commit_generated

```rust
    /// 提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。开放段满了就开下一个全空段。
    pub fn allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Option<Placement> {
        if self.open_segment.is_none() {
            let start = self.devices[0].lowest_empty_segment()?;
            self.open_segment = Some(start);
            self.bump_cursor = start.0;
        }
        let open = self.open_segment.expect("上面刚开");
        let segment_end = open.0 + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {
            start += 1;
        }
        if start + footprint.slots() > segment_end {
            self.open_segment = None;
            return self.allocate_commit_generated(footprint, generation);
        }
        let placement = Placement {
            slot: SlotNumber(start),
            span: footprint.slots(),
        };
        self.bump_cursor = start + footprint.slots();
        self.record(placement, generation);
        Some(placement)
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 563-604 行——PoolAllocator::reclaim_released_up_to

```rust
    /// 回收：释放代 ≤ `floor` 的已释放落点回到空闲，等着被再分配（D16（发布语义） 已定项 1：可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；
    /// 第一版根环 24 槽、种子 txg 0，环里最旧有效根恒 0，`floor` 就是 F_生效）。回收过的不重复回收；返回这次回收的落点（两盘同槽，按盘 0 报）。
    /// `reuse` 说回收的槽什么时候能发：重建时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘（`release_reclaim_holds`）。
    pub fn reclaim_released_up_to(
        &mut self,
        floor: CheckpointTxg,
        reuse: ReclaimedReuse,
    ) -> Vec<Placement> {
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
            let device_map = self
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("分配记录的盘在池里");
            device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));
            match reuse {
                ReclaimedReuse::Immediately => {}
                ReclaimedReuse::HeldUntilFloorTakesEffect => {
                    device_map
                        .hold_until_floor_takes_effect(record.slot, u64::from(record.span_slots));
                }
            }
            if record.device == self.devices[0].device {
                reclaimed_now.push(Placement {
                    slot: record.slot,
                    span: u64::from(record.span_slots),
                });
            }
        }
        reclaimed_now
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 606-611 行——PoolAllocator::release_reclaim_holds

```rust
    /// 抬 F 生效（带新 F 的根落满每块盘）之后，把扣住的回收槽放开。
    pub fn release_reclaim_holds(&mut self) {
        for device_map in &mut self.devices {
            device_map.release_holds();
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 613-621 行——PoolAllocator::isolate_abandoned

```rust
    /// 回退的影子账：把一个只被被抛弃根引用的落点在它那块盘上隔离。
    pub fn isolate_abandoned(&mut self, device: DeviceIdentity, slot: SlotNumber, span: u64) {
        let device_map = self
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("被抛弃根的分配记录的盘在池里：走读逐盘核过");
        device_map.isolate(slot, span);
    }
```

### `crates/singlefs-core/src/recovery.rs` 第 377-417 行——allocation_records_under_root

```rust
pub fn allocation_records_under_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let mut allocation_pointer = None;
    for bytes in &tree_table.entries {
        let entry = TreeTableEntry::parse(bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表条目",
        })?;
        if entry.kind == TREE_KIND_ALLOCATION {
            allocation_pointer = Some(entry.root);
        }
    }
    let Some(allocation_pointer) = allocation_pointer else {
        return Ok(Vec::new());
    };
    let allocation_bytes =
        read_unit_via_locations(reader, &allocation_pointer.locations, node_bytes)?;
    let allocation_node =
        parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "分配记录树根",
        })?;
    // 第一版的分配记录树只有一个节点（层 0），多层的树这条路还不会走（里程碑「第二个事务」步 6 的欠账）；
    // 读到层 > 0 的根就报格式错，不把内部节点的指针当分配记录解。
    if allocation_node.level != 0 {
        return Err(RecoveryFailure::UnitMalformed {
            what: "分配记录树根不止一层",
        });
    }
    Ok(allocation_node
        .entries
        .iter()
        .map(|bytes| AllocationRecord::parse(bytes))
        .collect())
}
```

### `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 第 1-476 行——整个文件

```rust
//! 里程碑「第二个事务」步 4 的验收：发布 C 之后进程退出、重开走管理员回退到 A 的根 (1, 3)——不施加 A 之后的任何记录、
//! 取实例代号 3、在 A 那一版实例表上写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、发布 D（txg 9，jsn 接在 A 那条记录之后 = 4）、
//! 暖机一次（txg 10 落盘 1）——冷启动择实例 3 的根读回第一次的内容；只被被抛弃根引用的槽由影子账隔离；池级 checker 全绿。

mod common;

use common::{build_pool, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_root, choose_superblock, recover, replay_journal, scan_journal, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    pool
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 进程退出、重开走回退到 A 的根；回来的可写态装回 pool。
fn rollback_to_first_root(pool: &mut BuiltPool, shadow_ledger: ShadowLedger) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let rolled_back =
        mount_rollback(&parameters(), &mut devices, first_root(), shadow_ledger).expect("回退");
    pool.devices = Some(devices);
    pool.allocator = rolled_back.allocator.clone();
    pool.output = rolled_back.current.clone();
    rolled_back
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 一版账里这块盘上占着的每个槽（记录按跨度展开）。
fn slots_of(output: &TransactionOutput, device: DeviceIdentity) -> BTreeSet<u64> {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == device)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 验收第一条：回退行 (1, 3, 0)、中间实例行 (2, 0, 0)；D 的根 (3, 9)、jsn 9（接在环里最大的 jsn 8 之后，C340 取 P2）、事务号 0、反向链 0，
/// 重写实例表 + 四个固定点单元；暖机一次落到另一块盘；一条记录都不施加；冷启动读回第一次的内容；被抛弃根独占的槽逐盘 34 个、
/// D 与暖机一个都不落在上面；checker 全绿（I-3.1 的并集按实例表把被抛弃的根排除，I-3.8 看见回退行）。
#[test]
fn rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content(
) {
    let mut pool = build_through_third_publish("step-four-rollback");
    let third = pool.output.clone();
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    let output = &rolled_back.output;
    assert_eq!(output.instance, InstanceGeneration(3));
    assert_eq!(
        output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(3),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "回退行与中间实例行"
    );
    assert_eq!(output.journal.prefix_applied, 0, "A 之后的记录一条都不施加");
    assert_eq!(
        (
            output.effective_root.instance,
            output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3))
    );
    let rollback_publish = &output.row_publish;
    assert_eq!(
        (
            rollback_publish.root.instance,
            rollback_publish.root.checkpoint_txg,
            rollback_publish.record.counter,
            rollback_publish.record.transaction,
            rollback_publish.record.back_chain
        ),
        (InstanceGeneration(3), CheckpointTxg(9), 9, 0, 0),
        "D：txg = max(根环 8, 记录 8) + 1；jsn 接在环里最大的 8 之后（C340 取 P2，被抛弃的记录一条不盖）；本实例第一条反向链 0"
    );
    assert_eq!(
        rollback_publish.rewritten,
        vec![
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
    );
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "D 落盘 0、txg 10 落盘 1，一次就够"
    );
    let warm_up = &output.warm_up_publishes[0];
    assert_eq!(warm_up.root.checkpoint_txg, CheckpointTxg(10));
    assert_ne!(region_device(9), region_device(10));
    assert_eq!(warm_up.record.counter, 10);

    // 影子账：被抛弃的根 B、(2, 5)、(2, 6)、(2, 7)、C 引用而 A 不引用的槽——B 10、写行 6、暖机 4 + 4、C 10 = 34 个槽，逐盘。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let abandoned: BTreeSet<u64> = slots_of(&third, device)
            .difference(&slots_of(rollback_publish, device))
            .copied()
            .collect();
        assert_eq!(abandoned.len(), 34, "盘 {device:?} 上只被被抛弃根引用的槽");
        // 影子账按窄读法只隔离这 34 个：mkfs 实例表那 2 个槽 A（候选）与 B 都引用，不在其内。
        assert!(
            output.isolated_slots_per_device.contains(&(device, 34)),
            "隔离的槽数 = 只被被抛弃根引用的 34 个 {:?}",
            output.isolated_slots_per_device
        );
        for publish in std::iter::once(rollback_publish).chain(output.warm_up_publishes.iter()) {
            for placement in publish.placements() {
                for slot in placement.slot.0..placement.slot.0 + placement.span {
                    assert!(
                        !abandoned.contains(&slot),
                        "txg {} 的落点 {slot} 落在被抛弃根引用的槽上",
                        publish.root.checkpoint_txg.0
                    );
                }
            }
        }
    }

    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(10)),
            content: file_content()
        },
        "{:?}",
        report.journal
    );
    assert_eq!(
        report.journal.valid_records, 10,
        "jsn 1–8 原样在（P2 一条不盖）、9 是 D、10 是暖机"
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "(3, 10) 之后 jsn 11 一条都没有"
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在回退之后的镜像上要成立"
        );
    }
    for must_hold in ["I-3.1", "I-3.8", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 回退候选集（D23 已定项 14）：实例表里有行 (i, Ti, Wi) 的实例，只有 T ≤ Ti 的根可选——B 的根 (1, 4) 与 C 的根 (2, 8) 都是被抛弃时间线的；
/// 根环里没有的 (1, 42) 另报。
#[test]
fn rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused() {
    let mut pool = build_through_third_publish("step-four-refused");
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    for (target, expected) in [
        (
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
        ),
        (
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(8),
            },
            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
        ),
    ] {
        let mut devices = pool.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        match refused {
            Err(MountError::RollbackTargetNotACandidate {
                target: reported,
                reason,
            }) => assert_eq!((reported, reason), (target, expected)),
            other => panic!(
                "{target:?} 该被拒：{:?}",
                other.map(|mounted| mounted.output.instance)
            ),
        }
    }
    let mut devices = pool.reopen_recorded();
    let missing = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(42),
    };
    let refused = mount_rollback(&parameters(), &mut devices, missing, ShadowLedger::On);
    pool.devices = Some(devices);
    assert!(
        matches!(refused, Err(MountError::RollbackTargetNotInRing(reported)) if reported == missing),
        "根环里没有 (1, 42)"
    );
}

/// C314（回退可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入：关掉影子账，回退之后再发两版文件，
/// 数据单元落回 B 与 C 的数据槽（50182、50184）；把实例 3 的四个根槽都改坏，恢复挂上 C 的根 (2, 8)，它的数据单元已被盖掉，读不出第三次的内容。
/// 影子账开着：两版数据落 50186、50188，同样改坏四个根槽之后恢复挂上 C 的根、第三次的内容原样读回。
#[test]
fn without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit(
) {
    for (shadow_ledger, expected_slots, expect_third_content_readable) in [
        (ShadowLedger::Off, [50182, 50184], false),
        (ShadowLedger::On, [50186, 50188], true),
    ] {
        let mut pool = build_through_third_publish("step-four-shadow");
        let rolled_back = rollback_to_first_root(&mut pool, shadow_ledger);
        let expected_isolated = if shadow_ledger == ShadowLedger::On {
            34
        } else {
            0
        };
        assert!(
            rolled_back
                .output
                .isolated_slots_per_device
                .iter()
                .all(|(_, isolated)| *isolated == expected_isolated),
            "{shadow_ledger:?}：{:?}",
            rolled_back.output.isolated_slots_per_device
        );
        let fourth = overwrite_in_process(&mut pool, &content_of(3000, 5), InstanceGeneration(3));
        let fifth = overwrite_in_process(&mut pool, &content_of(3100, 9), InstanceGeneration(3));
        assert_eq!(
            [
                fourth.data_pointer.locations[0].slot.0,
                fifth.data_pointer.locations[0].slot.0
            ],
            expected_slots,
            "{shadow_ledger:?} 下回退之后两版的数据落点"
        );
        let mut image = pool.memory_pool();
        for txg in [9u64, 10, 11, 12] {
            let target = target_for_publish(CheckpointTxg(txg));
            image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
        }
        let report = recover(&image, JournalPolicy::Consult);
        if expect_third_content_readable {
            assert_eq!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账开着：C 引用的单元一个没被盖，回到 C 读第三次的内容"
            );
        } else {
            assert_ne!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账关着：C 的数据单元已被第五版盖掉，第三次的内容读不回来"
            );
        }
    }
}

/// 前缀第五条（D23 已定项 14）：所选根的实例有回退行时，该实例的记录只施加到回退行的 W 为止——直接喂 `replay_journal`：
/// 到 B 为止的镜像上选 A 的根 (1, 3)，不带回退行施加 B 那条（事务号 2）；W = 0 一条都不施加；W = 2 施加到 B；W = 1 停在 B 之前。
#[test]
fn the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water() {
    let mut pool = build_pool("step-four-cap");
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let newest = choose_root(&image, &superblock).expect("B 的根");
    assert_eq!(newest.checkpoint_txg, CheckpointTxg(4));
    let records = scan_journal(&image, &superblock);
    let roots = singlefs_core::recovery::readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let first = roots
        .iter()
        .find(|root| root.checkpoint_txg == CheckpointTxg(3))
        .copied()
        .expect("A 的根在环里");
    for (high_water, expected_applied, expected_txg) in [
        (None, 1, 4u64),
        (Some(0), 0, 3),
        (Some(1), 0, 3),
        (Some(2), 1, 4),
    ] {
        let (report, effective) = replay_journal(
            &image,
            &first,
            superblock.geometry.journal_ring_bytes,
            &records,
            true,
            high_water,
        );
        assert_eq!(
            (report.prefix_applied, effective.checkpoint_txg.0),
            (expected_applied, expected_txg),
            "回退行 W = {high_water:?}"
        );
    }
}

/// C340 取 P2：回退之后新实例的第一条 jsn 接在环里最大的 jsn 之后，被抛弃发布的记录一条不盖——B 的记录已提交、根还没落盘时发起回退，
/// 崩在回退生效之前那次恢复才仍「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中 P1 盖掉 B 那条）。
/// 影子账只住内存，所以每次挂载都重算：回退之后普通重开一次，被抛弃根引用的槽照样隔离（同一轮攻方腿打中重开后隔离归零）。
#[test]
fn rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation() {
    let mut pool = build_through_third_publish("step-four-p2-remount");
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    assert_eq!(
        rolled_back.output.row_publish.record.counter, 9,
        "D 的 jsn 接在 C 的 8 之后"
    );
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let records = scan_journal(&image, &superblock);
    for (instance, counter) in [(1u32, 4u64), (2, 5), (2, 8), (3, 9), (3, 10)] {
        assert!(
            records.contains_key(&(InstanceGeneration(instance), counter)),
            "记录 ({instance}, jsn {counter}) 该原样在环里：{:?}",
            records.keys().collect::<Vec<_>>()
        );
    }
    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("回退之后普通重开");
    pool.devices = Some(devices);
    pool.allocator = remounted.allocator.clone();
    pool.output = remounted.current.clone();
    assert_eq!(remounted.output.instance, InstanceGeneration(4));
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)],
        "按 D 那一版实例表判被抛弃的根（B、实例 2 的四条）引用的槽，普通重开照样隔离"
    );
    assert_eq!(remounted.output.abandoned_roots_unreadable, 0);
    // 被抛弃根引用的槽：B 的数据 50182–50183、C 的数据 50184–50185 与它们的节点（隔离），以及 mkfs 实例表 50176–50177
    // （A 也引用、不隔离，但 D 释放它之后还在 defer 里）——重开后的发布一个都不该落上去。
    let abandoned: BTreeSet<u64> = (50176..50178).chain(50182..50186).collect();
    let next = overwrite_in_process(&mut pool, &content_of(2100, 41), InstanceGeneration(4));
    for placement in next.placements() {
        for slot in placement.slot.0..placement.slot.0 + placement.span {
            assert!(
                !abandoned.contains(&slot),
                "重开后的发布落到了被抛弃根引用的槽 {slot}"
            );
        }
    }
}

/// 被抛弃根 C 的树表单元在两块盘上都改坏：影子账罩不到 C（读不出就没法知道它引用谁），挂载照样成功、只计数一条读不出的被抛弃根，
/// 别的被抛弃根引用的槽照旧隔离（步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
#[test]
fn torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount() {
    let mut pool = build_through_third_publish("step-four-torn-abandoned");
    let third = pool.output.clone();
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    let mut devices = pool.reopen_recorded();
    for location in &third.root.tree_table.locations {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("C 的树表所在的盘");
        let offset = location.slot.to_device_offset();
        let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
        recorded.read_at(offset, &mut bytes).expect("读 C 的树表");
        bytes[200] ^= 0xff;
        recorded
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏 C 的树表");
    }
    let remounted =
        mount_writable(&parameters(), &mut devices).expect("被抛弃根的树表撕裂不拒绝挂载");
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "C 读不出");
    // C 独占的槽（它的数据 2 + 它的节点）罩不到；B、写行与暖机那些照旧隔离。
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 24), (DeviceIdentity(1), 24)],
        "少了只被 C 引用的 10 个槽"
    );
}
```

### `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 第 1-442 行——整个文件

```rust
//! 里程碑「第二个事务」步 5 的验收：回退之后再覆盖写四次（txg 11–14；第一次把 A 的八个单元释放、释放代 11），抬回退下界 F 到 11
//! （上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根) = min(13, 11)；两次空发布 txg 15、16 让两块盘各有一条带 F = 11 的根），
//! 释放代 ≤ 11 的落点回收、之后的仍在 defer 队列里；发布 E（txg 17）把数据单元落回 50178（mkfs 树表那 1 槽回收了、50179 从没分配过；mkfs 实例表那片 50176 也回收了但 B 的根还引用它、影子账隔离着；A 的数据单元 50180 排在后面）；冷启动读回 E；checker 全绿。
//! 必红：不抬 F 就回收（复用窗口置 0），第 0 代根还在候选集里（F = 0）、它们引用的 mkfs 树表单元被 E 盖掉，checker 在它们上判 I-2.1 红。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{Placement, ReclaimedReuse};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RaisedFloor, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::recovery::{
    choose_superblock, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 D：A、B、重开取号 2、写行、暖机两次、C、重开回退到 (1, 3)、D、暖机一次。
fn build_through_rollback(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator;
    pool.output = rolled_back.current;
    pool
}

/// 回退之后再覆盖写四次（txg 11–14）：第一次释放 A 的八个单元（释放代 11）。
fn four_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [17usize, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(3))
        })
        .collect()
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    pool.output = current;
    raised
}

fn newest_root_floor(pool: &BuiltPool) -> CheckpointTxg {
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|root| (root.checkpoint_txg, root.instance))
    .expect("根")
    .rollback_floor
}

/// 验收第一、二条：上限 11；两次空发布带 F = 11 落到两块盘；A 的八个落点（10 槽）回收、defer 队列从 40 槽减到 30；E 的数据单元落 50180、
/// 它的分配记录改写成代 17、未释放；后释放的（代 12–14）仍占着；冷启动读回 E；根记录 F = 11；checker 全绿（A 的根在 F 之下、不在候选集）。
#[test]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    let mut pool = build_through_rollback("step-five-reuse");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    assert_eq!(overwrites[0].root.checkpoint_txg, CheckpointTxg(11));
    assert_eq!(overwrites[3].root.checkpoint_txg, CheckpointTxg(14));
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            51,
            "A 的账里 mkfs 树表 1 槽已释放；D 释放 A 的四个固定点单元与 mkfs 实例表（6 槽）、暖机释放 D 的四个（4 槽）、四次覆盖写各释放上一版的 10 个槽"
        );
    }
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    assert_eq!(
        raised.ceiling,
        CheckpointTxg(11),
        "min(每块盘最新的有效根 14 / 13, 第 4 新的非空 11)"
    );
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| (
                publish.root.checkpoint_txg.0,
                publish.root.rollback_floor.0,
                publish.record.transaction
            ))
            .collect::<Vec<_>>(),
        vec![(15, 11, 0), (16, 11, 0)],
        "两次带新 F 的空发布"
    );
    assert!(raised.reclaimed.contains(&Placement {
        slot: SlotNumber(50180),
        span: 2
    }));
    assert_eq!(
        raised.reclaimed.len(),
        18,
        "释放代 ≤ 11 的落点：A 放掉的 mkfs 树表（代 3）、D 放掉的 5 个（代 9）、暖机放掉的 4 个（代 10）、第一次覆盖写放掉 A 的 8 个（代 11）"
    );
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            38,
            "回收了 1 + 6 + 4 + 10 = 21 个槽，抬 F 的两次空发布又各放掉上一版的 4 个"
        );
        assert!(device.is_free(SlotNumber(50180)) && device.is_free(SlotNumber(50181)));
        for later in &overwrites[..3] {
            let slot = later.data_pointer.locations[0].slot;
            assert!(
                !device.is_free(slot),
                "释放代 > F 的数据单元 {slot:?} 仍占着"
            );
        }
    }
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(reuse.root.checkpoint_txg, CheckpointTxg(17));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50178),
        "E 的数据单元落回最低的可再分配偶数槽对 50178–50179：mkfs 树表那 1 槽（A 换下、释放代 3）回收了、50179 从没分配过；mkfs 实例表那片 50176 虽被 D 放掉、也回收了，但 B 的根还引用它、被影子账隔离；A 的数据单元 50180 排在后面"
    );
    let reused_record = pool
        .allocator
        .record_for(DeviceIdentity(0), SlotNumber(50178))
        .expect("50178 的记录");
    assert_eq!(
        (
            reused_record.generation,
            reused_record.is_released,
            reused_record.span_slots
        ),
        (CheckpointTxg(17), false, 2),
        "复用时那条记录改写"
    );
    assert_eq!(
        pool.allocator
            .records()
            .iter()
            .filter(|record| record.device == DeviceIdentity(0) && record.slot == SlotNumber(50178))
            .count(),
        1,
        "同盘同槽只有一条记录"
    );
    for device in &pool.allocator.devices {
        assert_eq!(device.deferred_slots(), 48, "E 又释放了第四版的 10 个槽");
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(17)),
            content: content_of(2000, 31)
        },
        "{:?}",
        report.journal
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在 E 之后的镜像上要成立"
        );
    }
    for must_hold in ["I-2.1", "I-3.1", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 生效（D16 已定项 1）：每块幸存盘上都有带新 F 的持久根才生效，恢复后生效值 = 各盘所带 F 最大值的最小值——把 txg 16 的根槽（盘 1 上唯一带 F = 11 的根）改坏，
/// 重开之后 F_生效 回到 0：所选根是 txg 15、链上 txg 16 的记录照样施加（同实例），盘上写着已释放的 59 个槽一个都不回收（新实例写行再放 6、暖机两次各放 4 ⇒ 73）；根槽都好时回收 21 个槽（59 − 21 + 14 = 52）。
#[test]
fn one_device_carrying_the_floor_alone_does_not_take_effect_on_remount() {
    for (damage_second_carrier, expected_deferred, expected_chosen_txg) in
        [(false, 52, 16), (true, 73, 15)]
    {
        let mut pool = build_through_rollback("step-five-effective");
        four_overwrites_after_the_rollback(&mut pool);
        let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
        let second_carrier = raised.publishes[1].root.checkpoint_txg;
        assert_eq!(second_carrier, CheckpointTxg(16));
        let mut devices = pool.reopen_recorded();
        if damage_second_carrier {
            let target = target_for_publish(second_carrier);
            let device =
                parameters().region_devices[usize::try_from(target.region).expect("区域号")];
            let offset = slot_offset(target, 4096);
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("那块盘");
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读根槽");
            bytes[100] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏根槽");
        }
        let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
        pool.devices = Some(devices);
        assert_eq!(
            mounted.output.chosen_root.checkpoint_txg,
            CheckpointTxg(expected_chosen_txg)
        );
        for device in &mounted.allocator.devices {
            assert_eq!(
                device.deferred_slots(),
                expected_deferred,
                "改坏第二块盘的载体 = {damage_second_carrier}：F_生效 = 各盘 F 最大值的最小值；数里含写行放掉的 6 与暖机两次放掉的 8"
            );
        }
    }
}

/// 上限：第 4 新的非空持久有效根是 11（非空的有 14、13、12、11；D 与暖机是空发布不算；A 的根 3 是第 5 新）⇒ 抬到 12 被拒；
/// 抬到 11 之后上限仍是 11，再抬 12 仍被拒。
#[test]
fn raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused() {
    let mut pool = build_through_rollback("step-five-ceiling");
    four_overwrites_after_the_rollback(&mut pool);
    for attempt in [0, 1] {
        let refused = raise_floor(&mut pool, CheckpointTxg(12));
        assert!(
            matches!(
                refused,
                Err(MountError::RollbackFloorAboveCeiling {
                    requested: CheckpointTxg(12),
                    ceiling: CheckpointTxg(11)
                })
            ),
            "第 {attempt} 次：上限 11"
        );
        if attempt == 0 {
            raise_floor(&mut pool, CheckpointTxg(11)).expect("抬到上限");
        }
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
}

/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 A 的数据落点 50176——
/// F = 0 时 A（txg 3）还是候选，影子账按窄读法豁免它引用的槽、没隔离 50176；checker 走 A 时那片数据的校验和对不上 ⇒ I-2.1 红。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    let mut pool = build_through_rollback("step-five-window-zero");
    four_overwrites_after_the_rollback(&mut pool);
    let reclaimed = pool
        .allocator
        .reclaim_released_up_to(CheckpointTxg(11), ReclaimedReuse::Immediately);
    assert_eq!(reclaimed.len(), 18);
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50176),
        "A 的数据落点被拿走：A（F = 0 时仍是候选）还引用它，窄读法没隔离它"
    );
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(0), "F 没抬");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        violated.contains(&"I-2.1"),
        "A 的根还是候选，它的数据单元被盖了：{verdicts:?}"
    );
}

/// 回退候选集的 F 用 F_生效（各幸存盘所带 F 最大值的最小值），不是最新根自己带的 F：抬到 11 之后把盘 1 的载体（txg 16）改坏，
/// F_生效 回到 0，txg 9 的根 D 仍是候选、退得到；按最新根（txg 15，F = 11）自己的 F 判会把它拒掉。
#[test]
fn roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates() {
    let mut pool = build_through_rollback("step-five-candidate-floor");
    four_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    let mut devices = pool.reopen_recorded();
    let target = target_for_publish(second_carrier);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = slot_offset(target, 4096);
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("那块盘");
    let mut bytes = vec![0u8; 4096];
    recorded.read_at(offset, &mut bytes).expect("读根槽");
    bytes[100] ^= 0xff;
    recorded
        .write_at(offset, &bytes, WriteDurability::Plain)
        .expect("改坏根槽");
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(3),
            checkpoint_txg: CheckpointTxg(9),
        },
        ShadowLedger::On,
    )
    .expect("F_生效 是 0，txg 9 的根仍在候选集里");
    pool.devices = Some(devices);
    assert_eq!(rolled_back.output.instance, InstanceGeneration(4));
    assert_eq!(
        rolled_back.output.row_publish.root.rollback_floor,
        CheckpointTxg(0),
        "新实例的根写 F_生效"
    );
}

/// 抬 F 生效之前回收的槽不许发出去（D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」；alloc-basis 第二轮云端攻方腿打中：
/// 抬 F 自己的空发布在开放段满了之后按最低全空段开新段，刚回收空的那一段正好中选，崩在两条带新 F 的根之间时 F 之下的根仍是候选、
/// 它们的单元已被盖）。历史照那条腿的：A、B、重开取号 2、六次覆盖写（txg 8–13，开放段用满）、抬 F 到 8——两次空发布的落点一个都不在
/// 这次回收的槽上。
#[test]
fn slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect() {
    let mut pool = build_pool("step-five-hold-until-effective");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    for seed in [31usize, 37, 41, 43, 47, 53] {
        overwrite_in_process(
            &mut pool,
            &content_of(2000 + seed, seed),
            InstanceGeneration(2),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(13));
    let raised = raise_floor(&mut pool, CheckpointTxg(8)).expect("抬到 8");
    assert!(
        raised.publishes.len() >= 2,
        "至少两次空发布才能让两块盘各有一条带新 F 的根：{}",
        raised.publishes.len()
    );
    assert!(!raised.reclaimed.is_empty(), "回收了 A 与前几版释放的落点");
    let reclaimed_slots: std::collections::BTreeSet<u64> = raised
        .reclaimed
        .iter()
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect();
    for publish in &raised.publishes {
        for placement in publish.placements() {
            for slot in placement.slot.0..placement.slot.0 + placement.span {
                assert!(
                    !reclaimed_slots.contains(&slot),
                    "txg {} 的固定点落在这次回收的槽 {slot} 上：F 还没在两块盘上生效",
                    publish.root.checkpoint_txg.0
                );
            }
        }
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(8));
}

/// 只 mkfs 加第一个事务、没做过可写挂载的进程里抬 F：当前版本没有重写过的实例表单元，报错而不是 panic
/// （alloc-basis 第二轮云端攻方腿第七节第 1 条）。
#[test]
fn raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking() {
    let mut pool = build_pool("step-five-raise-without-mount");
    let refused = raise_floor(&mut pool, CheckpointTxg(1));
    assert!(
        matches!(
            refused,
            Err(MountError::RaiseNeedsWritableMountInThisProcess)
        ),
        "第一个事务之后直接抬 F 该报没做过可写挂载：{:?}",
        refused.as_ref().err()
    );
}
```

### `crates/mutations.tsv` 以「步 4」「步 5」开头的行（第 31-43、45-56 行，跳过第 44 行——那行属于步 3）

```tsv
# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。
步 4：回退行不带回退位	crates/singlefs-core/src/mount.rs	        applied_transaction_high_water: 0,\n        is_rollback: true,	        applied_transaction_high_water: 0,\n        is_rollback: false,	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：只写被退回的实例那一行、不写中间实例行	crates/singlefs-core/src/mount.rs	    for row_instance in first_row_instance..instance.0 {	    for row_instance in first_row_instance..first_row_instance + 1 {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1，盖掉被抛弃发布的记录槽）	crates/singlefs-core/src/mount.rs	    let next_counter = highest_counter + 1;	    let next_counter = own_record.counter + 1;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));	            let _ = (record.device, record.slot, record.span_slots);	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !below_floor {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 5：回收不看释放代（复用窗口置 0）	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released)	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
步 5：抬 F 的上限不看第 4 新的非空根	crates/singlefs-core/src/mount.rs	    Some(newest_on_every_device.min(fourth_newest))	    Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_above	raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused
步 5：抬 F 的空发布只推一次（F 只落在一块盘上）	crates/singlefs-core/src/mount.rs	        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            if self.reclaimed.remove(&key) {	            if self.reclaimed.remove(&key) && false {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !abandoned {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：F_生效 取各盘 F 最大值的最大值而不是最小值	crates/singlefs-core/src/recovery.rs	    highest_per_device\n        .values()\n        .copied()\n        .min()\n        .unwrap_or(CheckpointTxg(0))	    highest_per_device\n        .values()\n        .copied()\n        .max()\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-harness --test second_transaction_step_five_reuse -- one_device_carrying_the_floor_alone	one_device_carrying_the_floor_alone_does_not_take_effect_on_remount
步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）	crates/singlefs-core/src/mount.rs	        &|_| false,\n        ShadowLedger::On,\n    );	        &|_| false,\n        ShadowLedger::Off,\n    );	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	crates/singlefs-core/src/mount.rs	    if target.checkpoint_txg < effective_floor {	    if target.checkpoint_txg < newest_root.rollback_floor {	-p singlefs-harness --test second_transaction_step_five_reuse -- roots_below_a_floor_carried	roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates
步 5：回收门槛不看环里最旧有效根	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |_oldest| effective_floor)	-p singlefs-core reclaim_floor_takes	reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root
步 4：影子账把被抛弃根账里已释放的落点也隔离	crates/singlefs-core/src/mount.rs	            if record.is_released\n                || referenced_by_candidates.contains(&key)\n                || !isolated.insert(key)	            if referenced_by_candidates.contains(&key) || !isolated.insert(key)	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：重建分配器时不认第 0 版树表单元（另一个进程里的第一个文件版本漏释放它）	crates/singlefs-core/src/allocator.rs	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released\n                    && false	-p singlefs-core rebuild_from_records_remembers	rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released
步 4：影子账不豁免候选根引用的槽（保守读法）	crates/singlefs-core/src/mount.rs	            if record.is_released\n                || referenced_by_candidates.contains(&key)\n                || !isolated.insert(key)	            if record.is_released || !isolated.insert(key)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：被抛弃根的树表读不出时不计数	crates/singlefs-core/src/mount.rs	        let Ok(records) = allocation_records_under_root(devices, root) else {\n            unreadable += 1;\n            continue;\n        };	        let Ok(records) = allocation_records_under_root(devices, root) else {\n            continue;\n        };	-p singlefs-harness --test second_transaction_step_four_rollback -- torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
步 5：抬 F 之后不按新候选集重算影子账	crates/singlefs-core/src/mount.rs	    if shadow_ledger == ShadowLedger::On {\n        let roots = readable_roots(	    if shadow_ledger == ShadowLedger::Off {\n        let roots = readable_roots(	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：影子账的豁免不看候选根的 txg 是否 ≥ F	crates/singlefs-core/src/mount.rs	        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)	        .filter(|root| !is_abandoned(root))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	        ReclaimedReuse::HeldUntilFloorTakesEffect,	        ReclaimedReuse::Immediately,	-p singlefs-harness --test second_transaction_step_five_reuse -- slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
步 5：抬 F 生效之后不放开扣住的槽	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();\n    Ok(RaisedFloor {	    Ok(RaisedFloor {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错	crates/singlefs-core/src/mount.rs	        .ok_or(MountError::RaiseNeedsWritableMountInThisProcess)?;	        .expect("实例表单元");	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_in_a_process_that_never_mounted	raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking
```

