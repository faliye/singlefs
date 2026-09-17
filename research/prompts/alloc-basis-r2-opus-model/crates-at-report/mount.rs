//! 可写挂载（里程碑「第二个事务」步 3）：进程重开镜像之后先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根、
//! 上一版全部角色的单元与指针、分配器、下一条 jsn——再取实例代号、给上一个实例写行（D18（块里携带什么信息） 已定项 11）、
//! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 甲′），之后本实例的发布才接在后面。
//! 第一版没有干净关闭标记，重开一律走恢复。

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use crate::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use crate::block_device::BlockDevice;
use crate::journal::back_chain_of;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::recovery::{
    allocation_records_under_root, choose_root, choose_superblock, effective_rollback_floor,
    highest_root_txg, instance_table_of_root, readable_roots, rebuild_version, replay_journal,
    rollback_high_water_of_root, scan_journal, JournalScanReport, RecoveryFailure,
};
use crate::root_record::RootRecord;
use crate::root_ring::target_for_publish;
use crate::transaction::{
    acquire_instance, publish_version, AcquisitionFailed, InstanceTablePlan, PoolWriter,
    PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::ROOT_RING_REGIONS;
use std::collections::BTreeSet;

pub use crate::instance_table::{InstanceRow, InstanceTableRecords};

/// 可写挂载没做成。
#[derive(Debug)]
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面还没有发布过文件版本（树表为空）：第一版的可写挂载只接在有文件的池后面，刚 mkfs 的池走第一次可写挂载那条路。
    NoPublishedVersion,
    /// 所选根指着的实例表单元解不出行与链指针。
    InstanceTableMalformed,
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

/// 回退的目标：管理员带外从回退候选集里选的那条根（D23（journal 的角色与格式） 已定项 14 的显式例外）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackTarget {
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
}

/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：关掉影子账，回退之后只被被抛弃根引用的槽照常可分配——
/// C314（回退可以复用被抛弃的根引用的单元） 那两格必红靠它强制进入；产品路径恒 `On`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowLedger {
    On,
    Off,
}

impl From<RecoveryFailure> for MountError {
    fn from(failure: RecoveryFailure) -> Self {
        MountError::Recovery(failure)
    }
}

impl From<PublishError> for MountError {
    fn from(error: PublishError) -> Self {
        MountError::Publish(error)
    }
}

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
    /// 影子账隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」，只住内存）；可写挂载恒 0。
    pub isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
}

/// 可写挂载的结果：写出的东西、重建并推进过的分配器、接下来的发布要接在后面的那一版。
#[derive(Debug)]
pub struct Mounted {
    pub output: MountOutput,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
}

fn map_rebuild_failure(failure: RecoveryFailure) -> MountError {
    match failure {
        RecoveryFailure::InvariantViolated {
            invariant: "挂载",
        ..
        } => MountError::NoPublishedVersion,
        other @ (RecoveryFailure::NoValidSuperblock { .. }
        | RecoveryFailure::SuperblocksDisagree
        | RecoveryFailure::NoValidRoot
        | RecoveryFailure::UnitUnreadable { .. }
        | RecoveryFailure::UnitMalformed { .. }
        | RecoveryFailure::InvariantViolated { .. }
        | RecoveryFailure::MappingMiss { .. }
        | RecoveryFailure::MappingStillUnreadable { .. }) => MountError::Recovery(other),
    }
}

/// 新实例写行那一行的来历：上一个实例（普通挂载）或被退回的实例（回退）那一行；中间实例的行由 `establish_instance` 补 (i, 0, 0)。
struct PreviousInstanceRow {
    instance: InstanceGeneration,
    selected_root_txg: CheckpointTxg,
    applied_transaction_high_water: u64,
    is_rollback: bool,
}

/// 恢复（或回退）之后建立新实例要带的东西。
struct InstanceStart {
    chosen_root: RootRecord,
    effective_root: RootRecord,
    journal: JournalScanReport,
    previous: TransactionOutput,
    table: InstanceTableRecords,
    previous_row: PreviousInstanceRow,
    first_txg: CheckpointTxg,
    next_counter: u64,
    /// 新实例的根带的回退下界 = 恢复后生效的 F（各幸存盘所带 F 最大值的最小值）：与重建分配器时回收用的同一个值，
    /// 一条根带的 F 与它的记账行才说同一件事。
    effective_floor: CheckpointTxg,
}

/// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。
fn first_txg_of_new_instance<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    superblock: &crate::superblock::Superblock,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
) -> CheckpointTxg {
    let highest_record_txg = records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(CheckpointTxg(0));
    let highest_ring_txg = highest_root_txg(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .unwrap_or(CheckpointTxg(0));
    CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)
}

/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：根环 24 槽，第 0 代根被盖之前
/// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。
#[must_use]
pub fn reclaim_floor(
    effective_floor: CheckpointTxg,
    oldest_valid_root: Option<CheckpointTxg>,
) -> CheckpointTxg {
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}

/// 一条根按最新根指着的实例表判是不是被抛弃的：有它那个实例的行 (i, Ti, Wi) 且 txg > Ti（D23（journal 的角色与格式） 已定项 14
/// 回退段的候选集规则反过来）。
fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {
    table
        .rows
        .iter()
        .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
}

/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收，再把被抛弃根引用的每一个槽隔离（影子账；
/// `extra_abandoned` 是这次挂载新抛弃的根——回退时是 (txg, 实例) 大于 R_old 的那些，普通挂载没有）。
/// 影子账只住内存，所以每次挂载都要重算，不只回退那一次（步 4 / 步 5 代码三方第一轮云端攻方腿打中：回退之后普通重开一次隔离就归零）。
///
/// # Errors
/// 被抛弃根的树表或分配记录树读不到、解不开。
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
) -> Result<(PoolAllocator, CheckpointTxg), MountError> {
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
    allocator.reclaim_released_up_to(reclaim_floor(effective_floor, oldest_valid_root));
    if shadow_ledger == ShadowLedger::On {
        let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
        for root in roots.iter().filter(|root| is_abandoned(root)) {
            // 那条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
            for record in allocation_records_under_root(devices, root)? {
                if record.is_released || !isolated.insert((record.device.0, record.slot.0)) {
                    continue;
                }
                allocator.isolate_abandoned(
                    record.device,
                    record.slot,
                    u64::from(record.span_slots),
                );
            }
        }
    }
    Ok((allocator, effective_floor))
}

/// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
/// 有效 = 自证合法 ∧ 按最新根指着的实例表仍然有效 ∧ txg ≥ 今天的 F；非空 = 环里有它自己那条记录且事务号非 0
/// （预想：checker 怎么从盘上认「非空」没有条款，里程碑「第二个事务」步 5 的决策点）。
#[must_use]
pub fn rollback_floor_ceiling<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    superblock: &crate::superblock::Superblock,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
) -> Option<CheckpointTxg> {
    let valid: Vec<RootRecord> = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.checkpoint_txg >= current_floor)
    .filter(|root| {
        !table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    })
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
    let newest_on_every_device = newest_per_device.values().copied().min()?;
    let mut non_empty: Vec<CheckpointTxg> = valid
        .iter()
        .filter(|root| {
            records.values().any(|record| {
                record.instance == root.instance
                    && record.checkpoint_txg == root.checkpoint_txg
                    && record.transaction != 0
            })
        })
        .map(|root| root.checkpoint_txg)
        .collect();
    non_empty.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest = non_empty
        .get(3)
        .copied()
        .or_else(|| valid.iter().map(|root| root.checkpoint_txg).min())?;
    Some(newest_on_every_device.min(fourth_newest))
}

/// 抬 F 的空发布做完之后的东西。
pub struct RaisedFloor {
    pub ceiling: CheckpointTxg,
    /// 带新 F 的空发布，直到每块盘上都有一条带新 F 的持久根（生效条件）。
    pub publishes: Vec<TransactionOutput>,
    /// 生效之后回收的落点（两盘同槽，按盘 0 报）。
    pub reclaimed: Vec<Placement>,
}

/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）：正常的触发是准入不够，这里是只供测试的强制入口（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
/// 推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收。
///
/// # Errors
/// `new_floor` 超过上限；发布失败。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
) -> Result<RaisedFloor, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let records = scan_journal(&*devices, &superblock);
    let table = InstanceTableRecords::parse(&current.unit(TransactionUnit::InstanceTable).bytes)
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
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。生效（两块盘都有带新 F 的根）之前这个进程不会再发布，
    // 回收的槽在生效之前发不出去。
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
    let reclaimed = allocator.reclaim_released_up_to(reclaim_floor(new_floor, oldest_valid_root));
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
    Ok(RaisedFloor {
        ceiling,
        publishes,
        reclaimed,
    })
}

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
    let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;

    // 写行：给 [max(上一个实例, 1), 新实例) 里每个实例各一行——上一个实例 (i, T, W)（回退时是回退行，flags bit0 = 1），
    // 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。
    let previous = start.previous;
    let mut rows_written = Vec::new();
    let mut table_after = start.table.clone();
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
    let row_publish = publish_version(
        &mut pool,
        &mut allocator,
        PublishPlan {
            txg: start.first_txg,
            counter: start.next_counter,
            transaction: 0,
            instance,
            back_chain: 0,
            file: None,
            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: start.effective_floor,
        },
        Some(&previous),
    )?;

    // 暖机（D16（发布语义） 已定项 8 甲′）：本实例的根覆盖两块盘之前连推空发布，次数按落点公式现算，至多根环的区域数那么多次。
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish.root.checkpoint_txg)];
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = publish_version(
            &mut pool,
            &mut allocator,
            PublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                transaction: 0,
                instance,
                back_chain: back_chain_of(&current.record_bytes),
                file: None,
                instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                tree_birth_txg: current.tree_birth_txg(),
                tree_identifier_watermark: current.root.tree_identifier_watermark,
                rollback_floor: current.root.rollback_floor,
            },
            Some(&current),
        )?;
        let device = device_of_txg(next.root.checkpoint_txg);
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
        },
        allocator,
        current,
    })
}

/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。
///
/// # Errors
/// 恢复失败、所选根下没有文件版本、实例表解不开、取号失败、发布失败。
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
    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。
    let own_record = records
        .values()
        .find(|record| {
            record.instance == effective_root.instance
                && record.checkpoint_txg == effective_root.checkpoint_txg
        })
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
        .ok_or(MountError::NoPublishedVersion)?;
    let own_record_bytes = own_record.to_bytes();
    let previous = rebuild_version(&*devices, &effective_root, own_record, own_record_bytes)
        .map_err(map_rebuild_failure)?;
    let table = InstanceTableRecords::parse(&previous.unit(TransactionUnit::InstanceTable).bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    let next_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0)
        + 1;
    let first_txg = first_txg_of_new_instance(devices, &superblock, &records);
    let (allocator, effective_floor) = rebuilt_allocator(
        devices,
        &superblock,
        &previous,
        &|_| false,
        ShadowLedger::On,
    )?;
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
            table,
            previous_row,
            first_txg,
            next_counter,
            effective_floor,
        },
    )
}

/// 管理员回退（D23（journal 的角色与格式） 已定项 14 的显式例外）：带外选一条回退候选集里的旧根 R_old，不施加它之后的任何记录，
/// 取新实例代号，在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)，回退行、这次发布的单元与
/// 第一个新根同一次发布；只被被抛弃根引用的槽由影子账隔离（D28（挂载期承诺量） 已定项 1 第九项）；之后暖机同可写挂载。
/// 新实例的第一条 jsn 接在 R_old 自己那条记录之后（C340（回退之后记录链从哪条之后接没有定义） 取 P1，预想、等用户定）。
///
/// # Errors
/// 目标根不在根环、不在候选集、它自己那条记录读不出、被抛弃根的账读不出、取号或发布失败。
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
    // R_old 自己那条记录读得出就拿它当上一版的记录（只有 jsn 会被用到），读不出就拿最大 jsn 那条顶着，与可写挂载同一条路。
    let own_record = records
        .values()
        .find(|record| {
            record.instance == target.instance && record.checkpoint_txg == target.checkpoint_txg
        })
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
        .ok_or(MountError::NoPublishedVersion)?;
    // 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 的 P2；预想、等用户定）：
    // 接 R_old 那条之后（P1）会把「B 的记录已提交、根还没落盘」那个窗口里 B 的记录盖掉，崩在回退生效之前那次恢复就不再
    // 「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中）。被抛弃的记录原样留在盘上，靠候选集与实例表挡。
    let highest_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0);
    let next_counter = highest_counter + 1;
    let own_record_bytes = own_record.to_bytes();
    let previous = rebuild_version(&*devices, &target_root, own_record, own_record_bytes)
        .map_err(map_rebuild_failure)?;
    let table = InstanceTableRecords::parse(&previous.unit(TransactionUnit::InstanceTable).bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
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
    // 影子账：这次回退新抛弃的根 = 根环里 (txg, 实例) 大于 R_old 的每一条可读根；它们引用的每一个槽都隔离（保守读法，见
    // `DeviceFreeMap::isolate` 的注），连同按实例表早已被抛弃的根一起在重建里算。
    let newly_abandoned = |root: &RootRecord| {
        (root.checkpoint_txg, root.instance) > (target.checkpoint_txg, target.instance)
    };
    let (allocator, _) = rebuilt_allocator(
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
            table,
            previous_row,
            first_txg,
            next_counter,
            effective_floor,
        },
    )
}

#[cfg(test)]
mod reclaim_floor_tests {
    use super::reclaim_floor;
    use crate::address::CheckpointTxg;

    /// 第 0 代根还在环里时门槛就是 F_生效；被盖之后环里最旧的有效根接管（比 F 大时取它）。
    #[test]
    fn reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root() {
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(0))),
            CheckpointTxg(11)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(0), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(reclaim_floor(CheckpointTxg(11), None), CheckpointTxg(11));
    }
}
