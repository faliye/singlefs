//! 整理的一次发布：整理眼里的那一版（中央映射、钉住的槽）、发一步整理、整理发布写了多少字节、被空间拒的判定、回锁那一刻的回退门。

use super::*;

/// 整理读一版时取映射条目的那一步：整理选段与搬单元读版本（`version_as_the_compaction_reads_it`）经它取，别处不调。
/// 交回每个进映射单元按 key 在这一版的映射树里查到的映射条目，按 `mapped_units` 的次序（查不到、条目窄于字段表的不收），
/// 后面接这一版留存的映射条目，按留存的次序。
/// 公开给测试：七·6 的用例拿它逐项比逐 key 查（`central_mapping_locations_in_the_version`）的参照，判整理读漏、读错；
/// 它做的查找记在 [`LookupSite::CompactionReadsTheCentralMapping`] 那一处（`transaction::lookup_work`），用例读那一处的
/// 解叶次数与建索引次数判退回逐 key 查、每把 key 现建读者、每片叶重解（七·6 代码轮第二轮判决第三节）。
#[must_use]
pub fn central_mapping_as_the_compaction_reads_it(
    version: &TransactionOutput,
) -> Vec<CentralMappingEntry> {
    let _inside_the_compaction_read =
        lookup_work::entering(LookupSite::CompactionReadsTheCentralMapping);
    // 每把 key 查一次，叶按片解一次（七·6）：答案与逐 key 查 `central_mapping_locations_in_the_version` 逐个相同。
    let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    let mut mapping_entries: Vec<CentralMappingEntry> = version
        .mapped_units
        .iter()
        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {
            MappingLookup::Found(locations) => Some(CentralMappingEntry {
                key: key.clone(),
                locations,
            }),
            MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
        })
        .collect();
    mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(
        |(key, locations)| CentralMappingEntry {
            key: key.clone(),
            locations: *locations,
        },
    ));
    mapping_entries
}

pub(super) fn version_as_the_compaction_reads_it(
    version: &TransactionOutput,
) -> VersionAsTheCompactionReadsIt {
    let mapping_entries = central_mapping_as_the_compaction_reads_it(version);
    let pinned_slots = pinned_slots_of_the_version(version);
    let mut slots_holding_user_data = BTreeSet::new();
    for entry in &mapping_entries {
        if entry.key.first() == Some(&UNIT_CLASS_DATA) {
            let slot = entry.locations[0].slot;
            slots_holding_user_data.insert(slot);
            slots_holding_user_data.insert(SlotNumber(slot.0 + 1));
        }
    }
    VersionAsTheCompactionReadsIt {
        mapping_entries,
        pinned_slots,
        slots_holding_user_data,
    }
}

/// 一版里钉住的槽：豁免映射的三类（映射树各节点、树表、实例表链——根记录直接指着它们）罩住的每一个槽。只读这一版的单元表，不读映射：
/// 取输入时（会话锁里）算 R 要读哪几个单元用它，锁外建整份读法（[`version_as_the_compaction_reads_it`]）也用它。
pub(super) fn pinned_slots_of_the_version(version: &TransactionOutput) -> BTreeSet<SlotNumber> {
    let mut pinned_slots = BTreeSet::new();
    for unit in &version.units {
        let is_exempt_from_the_mapping = match unit.identity {
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => true,
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => false,
        };
        if is_exempt_from_the_mapping {
            for slot in unit.slot.0..unit.slot.0 + unit.identity.span_slots() {
                pinned_slots.insert(SlotNumber(slot));
            }
        }
    }
    pinned_slots
}

/// 在一个新开的写入口上发整理的一次发布（`transaction::publish_compaction_step`），接在 `previous` 后面；删意图那一支照记录核。
pub(super) fn publish_the_compaction_step_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
        parameters,
        devices,
        allocator,
        previous,
        planned,
        intent_step,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        instance,
    )
}

/// 同 [`publish_the_compaction_step_once`]，删意图那一支的核照 `region_check_of_the_intent_deletion` 判
/// （[`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、上一版、计划、对意图做什么、删意图那一支的核怎么判、实例代号"
)]
pub(super) fn publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    let switches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    publish_compaction_step_with_the_region_check_of_the_intent_deletion(
        &mut writer,
        allocator,
        previous,
        planned,
        intent_step,
        region_check_of_the_intent_deletion,
        instance,
    )
}

/// 整理的一次发布写了多少单元字节（写带宽上限数的就是它，D26（后台整理与放置回收） 已定项 13 第 6 条）：这次重写的角色加搬到目的地的单元，
/// 按槽数乘 16384、每块盘各一份（第一版每个单元落池里每一块盘）；journal 记录与根槽那几次写不计。搬走的单元的跨度取这一版里它目的落点那条
/// 分配记录的（目的地那条记录就是这次发布记下的，找不到不计）。
pub(super) fn bytes_written_by_a_compaction_publish(
    version: &TransactionOutput,
    devices: usize,
) -> u64 {
    let slots_of_the_rewritten_roles: u64 =
        version.rewritten.iter().map(|role| role.span_slots()).sum();
    let slots_of_the_relocated_units: u64 = version
        .units_relocated_by_this_publish
        .iter()
        .filter_map(|relocated| {
            let destination = relocated.destination[0];
            version
                .allocation_records
                .iter()
                .find(|record| {
                    record.device == destination.device && record.slot == destination.slot
                })
                .map(|record| u64::from(record.span_slots))
        })
        .sum();
    (slots_of_the_rewritten_roles + slots_of_the_relocated_units)
        .saturating_mul(SLOT_BYTES)
        .saturating_mul(u64::try_from(devices).expect("盘数装得进 u64"))
}

/// 整理的一次发布被拒是不是空间不够的那几种（[`refusal_is_short_of_space`]）：是就暂停、不推抬 F（判决 G8 g1）。
pub(super) fn compaction_publish_refused_short_of_space(refused: &RelocationBatchRefused) -> bool {
    match refused {
        RelocationBatchRefused::Publish(cause) => refusal_is_short_of_space(cause),
        RelocationBatchRefused::PlannedUnitNotInTheVersion { .. }
        | RelocationBatchRefused::PinnedUnitInThePlan { .. }
        | RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition { .. }
        | RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(_)
        | RelocationBatchRefused::UnitBytesNotAvailable { .. }
        | RelocationBatchRefused::CompactionIntentAlreadyLive { .. }
        | RelocationBatchRefused::NoLiveCompactionIntent
        | RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
            ..
        }
        | RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. } => {
            false
        }
    }
}

/// 整理的一批（推一次发布，后台任务种类 [`BackgroundTask::CompactionRelocation`]），三段首尾相接、中间不放锁：会话锁里取输入
/// （[`take_the_input_of_one_compaction_batch`]）、算（[`compute_the_compaction_batch`]，要读盘上一段时当场读）、回锁里发布
/// （[`judge_the_computed_compaction_batch_at_the_relock`] 与 [`publish_the_computed_compaction_batch_on_the_current_version`]）。
/// 放锁的三段入口（[`MountedSessionWithPendingChanges::take_the_input_of_a_background_compaction_batch`]）走同几段，只是中间放锁。
/// 取输入时读一次这次挂载的后台计算设置（D26（后台整理与放置回收） 已定项 13 第 1 条：挂着请求的切换在这里换上，从这一批起生效；一趟里每一批各读一次），
/// 按它定这一批走哪条路，一批之内不再读（`.claude/rules/fs-design.md` 五条硬要求第 5 条，`background_compute::BackgroundComputeOfThisMount::end_the_batch`
/// 断言没换）。选 `gpu` 时这一批的甲、乙、丙三段经设置带着的那一份 GPU 实现算，三段各自确认、不确定、交 CPU 判的条目数记进这一批的记账；
/// 发布那一半照旧在 CPU 上，照发布路径逐条核计划与读盘核（第 8 条）。读完设置之后、开算之前照 `in_the_middle_of_each_batch` 做（只供测试的钩子）。
/// 这一批算着出了错（第 4 条）：这一批放弃、一个单元都不搬，不换回 CPU；整理这个后台任务停下，现行那一版里有活意图就单发一次只删意图的发布丢掉它，
/// 停在 [`CompactionRunStop::BatchAbandonedAfterABackgroundComputeError`]。整理停着而这一批开头占用不超过补跑阈值时不开这一批，有活意图就同样删，
/// 停在 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、这一趟的账、这次挂载的后台计算状态、一批读完设置之后做什么"
)]
pub(super) fn advance_compaction_by_one_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    run: &CompactionRunUnderWay,
    background_compute: &mut BackgroundComputeOfThisMount,
    in_the_middle_of_each_batch: &mut InTheMiddleOfEachBackgroundBatch<'_>,
) -> CompactionPublishAdvanced {
    let input = match take_the_input_of_one_compaction_batch(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        run,
        background_compute,
        in_the_middle_of_each_batch,
    ) {
        CompactionBatchTaken::AdvancedInTheLock(advanced) => return *advanced,
        CompactionBatchTaken::Input(input) => input,
    };
    let computed = {
        let devices_to_read: &[(DeviceIdentity, Device)] = &*devices;
        compute_the_compaction_batch(*input, &mut |reads: UnitReadsOfARegionScan| {
            reads.read_with(|device, slot, unit_bytes| {
                devices_to_read.read(device, slot.to_device_offset(), unit_bytes)
            })
        })
    };
    match judge_the_computed_compaction_batch_at_the_relock(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        computed,
        background_compute,
        rollback_gate_at_the_relock_of(run),
    ) {
        CompactionBatchAtTheRelock::AdvancedInTheLock(advanced) => *advanced,
        CompactionBatchAtTheRelock::ToPublishOnTheCurrentVersion(to_publish) => {
            publish_the_computed_compaction_batch_on_the_current_version(
                parameters,
                devices,
                allocator,
                compaction,
                current,
                instance,
                *to_publish,
                background_compute,
                rollback_gate_at_the_relock_of(run),
            )
        }
    }
}

/// 回锁那一次发布判不判回退门：这一趟判回退门、这一批又不是破例时判（取输入之前那一判同一个条件），只在现行那一版变了时再判。
pub(super) fn rollback_gate_at_the_relock_of(
    run: &CompactionRunUnderWay,
) -> RollbackGateAtTheRelock {
    if run.is_the_rollback_gate_judged && !run.is_the_batch_the_exception {
        RollbackGateAtTheRelock::JudgedWhenTheVersionMoved
    } else {
        RollbackGateAtTheRelock::NotJudged
    }
}

/// 回锁那一次发布判不判回退门（[`rollback_gate_at_the_relock_of`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RollbackGateAtTheRelock {
    /// 现行那一版与票里那一版不同时，在现行那一版上照取输入之前那一判再判一次（D26（后台整理与放置回收） 已定项 13 第 5 条）。
    JudgedWhenTheVersionMoved,
    /// 显式入口、可写挂载续做、开关不照水位判、破例的那一批：取输入之前就不判，回锁也不判。
    NotJudged,
}
