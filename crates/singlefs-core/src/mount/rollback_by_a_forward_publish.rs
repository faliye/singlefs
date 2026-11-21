//! 管理员回退（[`roll_back_by_a_forward_publish`]）：候选集、复活与逐份验、回退那次发布的水位与它接在后面的那一版，
//! 连同回退交回的错与结果。

use super::*;

/// 管理员回退没做成（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格与候选集）。成员按管理员要做的决定分：
/// 换一条目标（不在候选集里）、先修盘或放弃这次回退（两版的账读不出、单元读不出、账对不上）、先原样重发冻结着的那次发布、
/// 回退那次发布本身没发成。除了 `Publish` 落盘途中失败那一格，都在任何写之前返回，盘上逐字节不变、分配器与调用方的现行版本不动。
#[derive(Debug)]
pub enum RollbackError {
    /// 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：重发成功才建下一次发布）：
    /// 先原样重发它（`transaction::resend_the_frozen_publish`）。在任何读写之前拒绝。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 系统配置择不出来（与挂载那一步同一个错）。
    Recovery(RecoveryFailure),
    /// 调用方交进来的参数或盘表不对（盘表里有身份交了不止一次；参数、盘数或某块盘的本盘设备号与盘上择到的系统配置不一致），
    /// 照可写挂载同一套核（[`caller_inputs_agreeing_with_the_disk`]）：回退那次发布的写入口按盘上那份参数建、
    /// 系统配置轮换按盘表逐项写，拿错的参数或盘表回退会改写盘上不可变段、让多交的那块盘收到本池的写
    /// （D22（单元原子性怎么合成） 已定项 26 第一档）。有盘不「可见」、或落后于现行那一版又缺它的单元或它的根（D18（块里携带什么信息） 已定项 11
    /// 「挂着之后收盘表的每个入口……核三样」）也报在这里。在任何写之前（重复身份在任何读之前）拒，盘上逐字节不变。
    CallerInputsDisagreeWithTheDisk(CallerInputsDisagreeingWithTheDisk),
    /// 现行那一版的 checkpoint_txg 已是 `u64::MAX`，回退那次发布的 txg（它加一）装不下（代码审阅第 36 条）：那个 txg 是盘上读来的，
    /// 坏镜像、外来镜像才走得到。在任何写之前、动分配器之前拒。
    NextCheckpointTxgPastTheTopOfItsRange { current: CheckpointTxg },
    /// 现行那一版的根指着的实例表沿链读不出或解不开：候选集「按现行那一版的实例表判仍然有效」判不了。
    CurrentInstanceTableMalformed,
    /// 目标不在回退候选集里，`exclusion` 说是哪一条。
    TargetNotACandidate {
        target: RollbackTarget,
        exclusion: RollbackCandidateExclusion,
    },
    /// 回退目标那一版从盘上重建不出来或自相矛盾：它的树表、inode 树、extent 树、分配记录树（「R_old 那一版的账有一个节点读不出」）、
    /// 记账树、中央映射树有一个节点读不出或解不开，它引用的用户可见单元在它自己的映射里查不到、映射条目的位置项不指池里两块不同的盘，
    /// 或它自己的账里没有它引用的单元的已分配记录。`failure` 说是哪一样。
    TargetVersionUnreadable {
        target: RollbackTarget,
        failure: RecoveryFailure,
    },
    /// 现行那一版的账（它的根指着的分配记录树）从盘上读，有一个节点读不出或解不开（「cur 那一版的账有一个节点读不出」）。
    CurrentAccountUnreadable { failure: RecoveryFailure },
    /// 回退目标那一版的十二棵树的号与现行那一版的不同：新根要取目标的 inode 树与 extent 树、别的树按现行那一版重算，两边号不同时
    /// 新根的树表怎么排条款没写（D23（journal 的角色与格式） 已定项 14 只写了取哪几棵树的根指针），第一版不支持。
    /// 合法历史里走不到：按现行那一版的实例表判仍然有效的根都在同一条时间线上，那条线上的第一个文件版本只发一次号。
    TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
        target: RollbackTarget,
    },
    /// 现行那一版或回退目标那一版的树表里有只读快照条目（码 14 / 15）：第一版有快照就拒回退（用户 2026-10-13 定「第一版有快照就拒」，
    /// 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S8 与第七节岔路 3）。`version` 说是哪一版先被认出带着快照
    /// （现行那一版在任何读之前判，目标那一版在重建出来之后判），`snapshots` 是那一版码 14 的条数。在任何写之前拒，盘上逐字节不变。
    SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
        version: VersionOfTheRollback,
        snapshots: usize,
    },
    /// 回退目标引用的一个用户可见单元，在现行那一版的账里这块盘上没有它的记录（没有起点是这个槽、跨度相同的那一条）。
    UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 回退目标引用的一个用户可见单元，在现行那一版的账里这块盘上仍分配、而分配代与回退目标那一版账里的不同：那一槽被复用过。
    UserVisibleUnitStillAllocatedUnderAnotherGeneration {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
        current_account_generation: CheckpointTxg,
        target_account_generation: CheckpointTxg,
    },
    /// 复活集里一个单元（现行账记成已释放、这一次要改回已分配）在这块盘上的那一份读不出或整单元 CRC-32C 与映射条目里的对不上。
    ResurrectedUnitCopyUnreadableOrMismatched {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 回退目标引用的一个隔离单元（现行账里每块盘上都仍分配、分配代相同，现行那一版却不引用它：释放之前读盘核出过一份对不上、
    /// 两份都留在已分配，D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）在这块盘上的那一份读不出或整单元 CRC-32C 与映射条目里的对不上
    /// （D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格，用户 2026-10-12 定 C572「隔离单元也进逐盘验」）。
    IsolatedUnitCopyUnreadableOrMismatched {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 释放现行那一版不再被引用的用户可见单元时，释放判定路径报错（映射里查不到、条目切不动、两条位置条目不同槽、这块盘的记录
    /// 不在册 / 已释放 / 跨度不对、位置项指池外的盘或两条指同一块盘）：`cause` 原样是发布路径的那一种。
    CurrentVersionUnitNotReleasable { cause: PublishError },
    /// 判候选集时读根环或算 F_生效 重读一次仍读坏（[`rollback_candidate`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行的读法）：
    /// 只有 [`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`] 与
    /// [`StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor`] 两个成员走得到。目标在不在候选集里判不了，
    /// 不按有根或没根猜。在任何写之前、动分配器之前拒，盘上逐字节不变。
    CandidateJudgementStillUnreadableAfterOneReread(Box<StillUnreadableAfterOneReread>),
    /// 回退那次发布没发成，连同这次回退开的写入口交得出的写账（[`PublishSequenceFailed`]：之前没有别的写、已落盘的发布一次都没有）。
    /// 落盘之前被拒（`cause` 是落点、释放判定这一类）：分配器换回回退之前那一份，盘上逐字节不变。落盘途中失败（`cause` 是块设备错）：
    /// 这次发布冻结在分配器上等原样重发（`transaction::resend_the_frozen_publish`），重发成功就是回退做成。
    Publish(Box<PublishSequenceFailed>),
}

/// 回退牵涉的两版里的哪一版（[`RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionOfTheRollback {
    Current,
    Target,
}

/// 一版树表里码 14（只读快照的 inode 根）的条数。
fn snapshots_in_the_tree_table(version: &TransactionOutput) -> usize {
    version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .count()
}

/// 复活的一份：哪个单元、哪块盘、落点、写回的分配代（回退目标那一版账里的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResurrectedCopy {
    pub unit: TransactionUnit,
    pub device: DeviceIdentity,
    pub placement: Placement,
    pub allocation_generation: CheckpointTxg,
}

/// 管理员回退做成之后交回的东西。回退那次发布之后的现行那一版已经换进调用方的 `current`（它的 `released` 是那次发布换下的固定点单元）。
#[derive(Debug)]
pub struct RolledBack {
    pub target: RollbackTarget,
    /// 释放集：现行那一版引用、新根不引用的用户可见单元，按现行那一版的角色次序（两盘同槽，释放代 = 新根的 txg）。
    pub released_user_visible_units: Vec<(TransactionUnit, Placement)>,
    /// 释放集里释放之前读盘核出对不上（读不出也算）、每块盘上那一份的记录留在已分配的那几份（D19（块指针的结构与宽度预算） 已定项 5
    /// 硬规则 1，与发布路径同一段读法）。
    pub quarantined_user_visible_copies: Vec<CopyQuarantinedAfterReleaseChecksumMismatch>,
    /// 复活集，逐盘。
    pub resurrected_user_visible_copies: Vec<ResurrectedCopy>,
}

/// 一个角色是不是用户可见单元（D23（journal 的角色与格式） 已定项 14 的记号：数据单元与 extent 树、inode 树的单元）。
/// 新根的这几样取回退目标那一版的，别的（固定点单元与实例表）按回退那次发布现算或照抄现行那一版的。
#[must_use]
pub const fn is_a_user_visible_unit(identity: TransactionUnit) -> bool {
    match identity {
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot => true,
        TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
        | TransactionUnit::DeadlistTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
        // `deleted_inodes` 根不在 D23 那个记号里（新根照样取 R_old 的，见 [`is_a_unit_taken_from_the_rollback_target`]）；
        // 墓碑树、谱系树、logged_ops 与退役树随现行那一版（退役树：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）。
        TransactionUnit::DeletedInodesRoot
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
    }
}

/// 回退那次发布的新根取 R_old 那一版的单元：用户可见单元（[`is_a_user_visible_unit`]）与 `deleted_inodes` 根（D23（journal 的角色与格式）
/// 已定项 14：新根的 inode 树、extent 树与 `deleted_inodes` 的根指针取 R_old 的；墓碑树、谱系树、logged_ops 随现行那一版）。
/// 复活、释放、逐盘验与映射 key 都按这一组走：`deleted_inodes` 根指针取 R_old 的而它的单元不复活，新根就指着一个记成已释放的槽。
/// 回退到带 nlink = 0 记录的一版，队列跟着回来，由下一次挂载续做（或会话的下一次用户改动）走空（D8（核心索引结构） 已定项 16 第 6 条）。
#[must_use]
pub const fn is_a_unit_taken_from_the_rollback_target(identity: TransactionUnit) -> bool {
    match identity {
        TransactionUnit::DeletedInodesRoot => true,
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
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::LoggedOperationsRoot
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_) => is_a_user_visible_unit(identity),
    }
}

/// 一版里每个用户可见单元与它在这一版中央映射里的那两条位置项（映射是落点的权威，D19（块指针的结构与宽度预算） 已定项 5 第 1 条），
/// 按这一版 `mapped_units` 的次序。
///
/// # Errors
/// 查不到 ⇒ `ReleaseNotInMapping`；映射条目窄于字段表 ⇒ `MappingEntryNarrowerThanItsFieldTable`（与释放判定路径同两种）。
fn user_visible_units_through_the_mapping(
    version: &TransactionOutput,
) -> Result<Vec<(TransactionUnit, [LocationEntry; 2])>, PublishError> {
    version
        .mapped_units
        .iter()
        .filter(|(identity, _)| is_a_unit_taken_from_the_rollback_target(*identity))
        .map(
            |(identity, key)| match central_mapping_locations_in_the_version(version, key) {
                MappingLookup::Found(locations) => Ok((*identity, locations)),
                MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
                    Err(PublishError::ReleaseNotInMapping { unit: *identity })
                }
                MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
                    Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
                        unit: *identity,
                        entry_bytes,
                        field_table_bytes: usize::try_from(singlefs_format::MAPPING_ENTRY_BYTES)
                            .expect("55"),
                    })
                }
            },
        )
        .collect()
}

/// 回退候选集那一判（D23（journal 的角色与格式） 已定项 14「候选集」）：根环里有它 ∧ txg ≥ F_生效 ∧ 按现行那一版的实例表判仍然有效
/// ∧ 带文件（D16（发布语义） 已定项 1「回退候选集」；原来另有「txg 落在一圈窗口里」一格，用户 2026-10-04 定 RGall 退回整个环）。
/// 按 [`RollbackCandidateExclusion`] 成员的次序判。交回根环里读出来的那条根。
/// 「根环里有它」与 F_生效 两处读根环都照抬 F 同一个读法（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：
/// `ring_slots_known_to_hold_a_root`（这个进程分配器上那张根环表，[`ring_slots_known_to_hold_a_root_by`]）里没有的槽读坏当没有根；
/// 有的重读一次，仍坏就拒——不按没有根往下走（那样目标可以被判成不在环里，F_生效 可以算低、低于真 F 的目标被当成候选）。
/// F_生效 那一半最新那条根的实例表读不出同样重读一次，仍读不出就拒。
///
/// # Errors
/// `TargetNotACandidate`；目标那一版的树表读不出 ⇒ `TargetVersionUnreadable`；知道住着根的槽或最新那条根的实例表重读仍读坏 ⇒
/// `CandidateJudgementStillUnreadableAfterOneReread`（在任何写之前、动分配器之前）。
fn rollback_candidate<Reader: PoolReader>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_table: &InstanceTableRecords,
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
    target: RollbackTarget,
) -> Result<RootRecord, RollbackError> {
    let not_a_candidate = |exclusion| RollbackError::TargetNotACandidate { target, exclusion };
    let still_unreadable = |still_unreadable: StillUnreadableAfterOneReread| {
        RollbackError::CandidateJudgementStillUnreadableAfterOneReread(Box::new(still_unreadable))
    };
    let target_root = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        ring_slots_known_to_hold_a_root,
    )
    .map_err(|reading| still_unreadable(reading.into()))?
    .into_iter()
    .find(|root| root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg)
    .ok_or_else(|| not_a_candidate(RollbackCandidateExclusion::NotInRing))?;
    let effective_floor =
        effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once(
            reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            ring_slots_known_to_hold_a_root,
        )
        .map_err(|reading| still_unreadable(reading.into()))?;
    if target.checkpoint_txg < effective_floor {
        return Err(not_a_candidate(
            RollbackCandidateExclusion::BelowEffectiveFloor,
        ));
    }
    if abandoned_by_table(&target_root, current_table) {
        return Err(not_a_candidate(
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ));
    }
    match tree_table_has_no_entries(reader, &target_root) {
        Ok(true) => Err(not_a_candidate(
            RollbackCandidateExclusion::VersionWithoutFile,
        )),
        Ok(false) => Ok(target_root),
        Err(failure) => Err(RollbackError::TargetVersionUnreadable { target, failure }),
    }
}

/// 回退目标引用的用户可见单元，逐个逐盘对现行那一版的账（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」前两样与「复活」）：
/// 每块盘上起点是它的槽、跨度相同的那条记录——没有就拒；仍分配而分配代与回退目标那一版账里的不同就拒；已释放的进复活集
/// （分配代写回目标那一版账里的）；仍分配、分配代相同的不动。交回复活集、与复活集里有份的单元（逐盘验要读它们每一份），
/// 与隔离单元：每块盘上都仍分配、分配代相同，而现行那一版不引用它（它的位置项的槽里没有一个是现行那一版的单元的槽，
/// `slots_of_the_units_of_the_current_version`）——现行账里仍分配而现行那一版不引用，只有释放之前读盘核出一份对不上、两份都留在已分配
/// 这一种来路（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，`PoolAllocator::release_leaving_the_record_allocated_on`）。
/// 隔离单元同样逐盘验（D23 已定项 14「在任何写之前拒」那一格「R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些……同样逐盘验」，
/// 用户 2026-10-12 定 C572「隔离单元也进逐盘验」）；现行那一版引用着的才是新旧两版共用的单元，分配器不会发出去，不验。
/// 仍分配、同分配代而带着隔离记号的那几份另交回清记号（D19（块指针的结构与宽度预算） 已定项 5「回退复活那一次，同一次发布清掉两块盘上的 0x4000」）。
///
/// 循环：外层按目标的用户可见单元（轮数 = 单元数），内层按它的两条位置项；跨轮只往三张清单里追加；提前出口都是拒。
///
/// # Errors
/// `UserVisibleUnitWithoutItsRecordInTheCurrentAccount`、`UserVisibleUnitStillAllocatedUnderAnotherGeneration`；
/// 目标那一版自己的账里没有它引用的单元的已分配记录、映射条目的位置项不指池里两块不同的盘 ⇒ `TargetVersionUnreadable`。
fn resurrection_against_the_current_account(
    target: RollbackTarget,
    target_version: &TransactionOutput,
    target_units: &[(TransactionUnit, [LocationEntry; 2])],
    current_account: &PoolAllocator,
    slots_of_the_units_of_the_current_version: &BTreeSet<SlotNumber>,
) -> Result<Resurrection, RollbackError> {
    let pool_devices: Vec<DeviceIdentity> = current_account
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    let malformed_target = |what: &'static str| RollbackError::TargetVersionUnreadable {
        target,
        failure: RecoveryFailure::UnitMalformed { what },
    };
    let mut resurrected = Vec::new();
    let mut units_to_check_on_every_device = Vec::new();
    let mut isolated_units_to_check_on_every_device = Vec::new();
    let mut quarantine_marks_to_clear = Vec::new();
    for (unit, locations) in target_units {
        refuse_locations_that_do_not_name_two_pool_devices(*unit, locations, &pool_devices)
            .map_err(|_refusal| {
                malformed_target("回退目标那一版的映射条目，位置项不指池里两块不同的盘")
            })?;
        let mut is_resurrected_on_some_device = false;
        for location in locations {
            let target_generation = target_version
                .allocation_records
                .iter()
                .find(|record| {
                    record.device == location.device
                        && record.slot == location.slot
                        && !record.is_released
                        && u64::from(record.span_slots) == unit.span_slots()
                })
                .map(|record| record.generation)
                .ok_or_else(|| {
                    malformed_target("回退目标那一版自己的账里没有它引用的用户可见单元的已分配记录")
                })?;
            let current_record = current_account
                .record_for(location.device, location.slot)
                .filter(|record| u64::from(record.span_slots) == unit.span_slots());
            match current_record {
                None => {
                    return Err(
                        RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
                            unit: *unit,
                            device: location.device,
                            slot: location.slot,
                        },
                    );
                }
                Some(record) if record.is_released => {
                    is_resurrected_on_some_device = true;
                    resurrected.push(ResurrectedCopy {
                        unit: *unit,
                        device: location.device,
                        placement: Placement {
                            slot: location.slot,
                            span: unit.span_slots(),
                        },
                        allocation_generation: target_generation,
                    });
                }
                Some(record) if record.generation != target_generation => {
                    return Err(
                        RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration {
                            unit: *unit,
                            device: location.device,
                            slot: location.slot,
                            current_account_generation: record.generation,
                            target_account_generation: target_generation,
                        },
                    );
                }
                // 仍分配、同分配代而带着隔离记号：回退目标引用它，回退那次发布清掉这块盘上的记号（D19（块指针的结构与宽度预算） 已定项 5
                // 「回退复活那一次，同一次发布清掉两块盘上的 0x4000」）。
                Some(record)
                    if record.quarantine_mark
                        == crate::allocator::AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck =>
                {
                    quarantine_marks_to_clear.push((
                        location.device,
                        Placement {
                            slot: location.slot,
                            span: unit.span_slots(),
                        },
                    ));
                }
                Some(_still_allocated_under_the_same_generation_without_the_quarantine_mark) => {}
            }
        }
        let is_referenced_by_the_current_version = locations
            .iter()
            .any(|location| slots_of_the_units_of_the_current_version.contains(&location.slot));
        if is_resurrected_on_some_device {
            units_to_check_on_every_device.push((*unit, *locations));
        } else if !is_referenced_by_the_current_version {
            isolated_units_to_check_on_every_device.push((*unit, *locations));
        }
    }
    Ok(Resurrection {
        resurrected,
        units_to_check_on_every_device,
        isolated_units_to_check_on_every_device,
        quarantine_marks_to_clear,
    })
}

/// 复活那一判交回的：复活集（逐盘），复活集里有份的单元（逐盘验要读它们每一份），与回退目标引用的隔离单元（同样逐盘验），
/// 与回退目标引用的单元里现行账带着隔离记号的那几份（逐盘：哪块盘、哪个落点；回退那次发布清掉）。
struct Resurrection {
    resurrected: Vec<ResurrectedCopy>,
    units_to_check_on_every_device: Vec<(TransactionUnit, [LocationEntry; 2])>,
    isolated_units_to_check_on_every_device: Vec<(TransactionUnit, [LocationEntry; 2])>,
    quarantine_marks_to_clear: Vec<(DeviceIdentity, Placement)>,
}

/// 回退目标引用的隔离单元在每块盘上的那一份都要读得出、整单元 CRC-32C 与映射条目里的对得上（D23（journal 的角色与格式） 已定项 14
/// 「在任何写之前拒」那一格，用户 2026-10-12 定 C572「隔离单元也进逐盘验」）：隔离就是释放之前读盘核出过一份对不上，回退之后新根又引用它，
/// 不验就把读不出的那一份当成两份都在发出去。读法与复活集那一判相同（[`first_copy_that_does_not_read_back`]），只是拒因不同。
/// 只读一次，不重读：读错只让这次回退被拒，不留下任何状态。
///
/// # Errors
/// `IsolatedUnitCopyUnreadableOrMismatched`，按单元次序、位置项次序报第一份。
fn every_copy_of_the_isolated_units_the_target_references_reads_back<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    units: &[(TransactionUnit, [LocationEntry; 2])],
) -> Result<(), RollbackError> {
    match first_copy_that_does_not_read_back(reader, units) {
        None => Ok(()),
        Some((unit, location)) => Err(RollbackError::IsolatedUnitCopyUnreadableOrMismatched {
            unit,
            device: location.device,
            slot: location.slot,
        }),
    }
}

/// 复活集里每个单元在每块盘上的那一份都要读得出、整单元 CRC-32C 与映射条目里的对得上（D23（journal 的角色与格式） 已定项 14
/// 「在任何写之前拒」第三样，主 agent 2026-09-26 定）：它们的槽在现行那一版的时间线上释放过，可能被一次没发布成的写盖掉一块盘上那一份。
/// 只读一次，不重读：读错只让这次回退被拒，不留下任何状态。
///
/// # Errors
/// `ResurrectedUnitCopyUnreadableOrMismatched`，按单元次序、位置项次序报第一份。
fn every_copy_of_the_resurrected_units_reads_back<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    units: &[(TransactionUnit, [LocationEntry; 2])],
) -> Result<(), RollbackError> {
    match first_copy_that_does_not_read_back(reader, units) {
        None => Ok(()),
        Some((unit, location)) => Err(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {
            unit,
            device: location.device,
            slot: location.slot,
        }),
    }
}

/// `units` 里按单元次序、位置项次序第一份读不出、或整单元 CRC-32C 与位置项里的校验和对不上的那一份；都对得上交 `None`。
/// 每份只读一次。
fn first_copy_that_does_not_read_back<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    units: &[(TransactionUnit, [LocationEntry; 2])],
) -> Option<(TransactionUnit, LocationEntry)> {
    units.iter().find_map(|(unit, locations)| {
        let unit_bytes = usize::try_from(unit.span_slots() * SLOT_BYTES).expect("一个单元两槽以内");
        locations
            .iter()
            .find(|location| {
                !reader
                    .read(
                        location.device,
                        location.slot.to_device_offset(),
                        unit_bytes,
                    )
                    .is_some_and(|copy| crc32_castagnoli(&copy) == location.unit_checksum)
            })
            .map(|location| (*unit, *location))
    })
}

/// `ring_roots` 里每一条带文件的根，它那一版记账里的 inode 号水位，取最大；从盘上重建那一版读不出的根不算（「环里读得出的根」）。
/// 只有管理员回退读它（D23（journal 的角色与格式） 已定项 14「水位」一格），交进来的只是按 cur 的实例表没被崩溃恢复抛弃的那几条
/// （Opt2，[`ring_roots_not_abandoned_by_crash_recovery`]）。`record_standing_for_every_root` 只是给重建顶着的一条记录
/// （重建出来的版本只拿它的 jsn 与事务号，这里都不用）。
fn highest_inode_number_watermark_in_the_ring(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    ring_roots: &[RootRecord],
    record_standing_for_every_root: &crate::journal::JournalRecord,
) -> Option<u64> {
    ring_roots
        .iter()
        .filter_map(|root| {
            match rebuild_version_in_the_unit_area_starting_at(
                reader,
                unit_area_start,
                root,
                Some(record_standing_for_every_root.clone()),
            ) {
                Ok(
                    RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version),
                ) => Some(version.inode_number_watermark()),
                Ok(RebuiltVersion::WithoutFile) | Err(_) => None,
            }
        })
        .max()
}

/// 根环里读得出的根里，按 cur 的实例表（`current_table`）没被崩溃恢复抛弃的那几条（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2：
/// 管理员回退取 inode 号水位只看它们）。被抛弃的根在它那条时间线上发过的号，崩溃恢复落到较旧的根之后本来就会重发（D8（核心索引结构） 已定项 6 射程），
/// 算进来会让回退把水位抬过这条时间线没发过的号。
fn ring_roots_not_abandoned_by_crash_recovery(
    ring_roots: &[RootRecord],
    current_table: &InstanceTableRecords,
) -> Vec<RootRecord> {
    ring_roots
        .iter()
        .filter(|root| !abandoned_by_table(root, current_table))
        .cloned()
        .collect()
}

/// 管理员回退那次发布的 inode 号水位（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2）：max(cur 那一版内存里的水位,
/// 根环里按 cur 的实例表没被崩溃恢复抛弃的那几条根各自那一版的水位)。环里那一项由 `highest_inode_number_watermark_of` 去盘上重建
/// （回退路径交 [`highest_inode_number_watermark_in_the_ring`]）；取哪几条根、怎么与 cur 那一项合在这里，构造态单测直接调它
/// （判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F3：环那一项在走得到的状态上不大于 cur，去掉它整套用例照绿）。
/// cur 那一项不能去（只取环，回退那次就重发号，C143）。
fn inode_number_watermark_of_the_rollback(
    ring_roots: &[RootRecord],
    current_table: &InstanceTableRecords,
    inode_number_watermark_of_the_current_version: u64,
    highest_inode_number_watermark_of: impl FnOnce(&[RootRecord]) -> Option<u64>,
) -> u64 {
    highest_inode_number_watermark_of(&ring_roots_not_abandoned_by_crash_recovery(
        ring_roots,
        current_table,
    ))
    .map_or(inode_number_watermark_of_the_current_version, |ring| {
        ring.max(inode_number_watermark_of_the_current_version)
    })
}

/// 管理员回退那次发布的树 ID 水位（D23（journal 的角色与格式） 已定项 14「水位」一格）：max(cur 那一版内存里的水位, 根环里读得出的**全部**根的水位)。
/// 与 inode 号水位（[`inode_number_watermark_of_the_rollback`]）不同，被崩溃恢复抛弃的根照旧算：树 ID 发过就不再发，被抛弃的时间线上发过的号
/// 照样不能再发给别的树。`_current_table_not_consulted` 是 cur 的实例表，这里不读它：收它是为了与 inode 号水位那一段同形，
/// 构造态单测交一条按它被抛弃的根，「改成只取没被抛弃的根」这一错写得出来、会红（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
/// 第三节 G5：回退用例在走得到的历史上分不出这两种取法）。
fn tree_identifier_watermark_of_the_rollback(
    ring_roots: &[RootRecord],
    _current_table_not_consulted: &InstanceTableRecords,
    tree_identifier_watermark_of_the_current_version: u64,
) -> u64 {
    ring_roots
        .iter()
        .map(|root| root.tree_identifier_watermark)
        .fold(tree_identifier_watermark_of_the_current_version, u64::max)
}

/// 回退那次发布接在后面的「上一版」：用户可见的那一半（数据单元与指针、extent 树、inode 树的叶容器与根、第一个文件那条 inode 记录）
/// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的——中央映射树只是节点取现行那一版的，
/// 用户可见单元的条目由发布路径按目标那一版的位置装、位置不同的那几把 key 所在的路重写（[`mapping_keys_whose_target_locations_differ_from_the_current_mapping`]）；
/// 根记录、末条记录、事务号取现行那一版的（这次发布接着它发，反向链、计数器、txg 都接着它）。
/// 发布路径照这一版照抄用户可见单元（inode 树根指针从树表条目取，所以树表条目里 inode 树与 extent 树那两条换成目标那一版的）、
/// 按分配器现算固定点单元、换下现行那一版被重写的固定点单元——正是「新根的用户可见树取 R_old 的，树表单元、分配记录树、中央映射树与
/// 记账统计量按这次发布照调整之后的分配器现算」（D23（journal 的角色与格式） 已定项 14）。
/// 记账行原样取 cur 的：inode 号水位的台阶行照抄 cur 的（D5（快照 / 空间记账机制） 已定项 2 射程「管理员回退那次从 cur 带、不取 R_old 的」），
/// 回退那次发布的水位（「水位」一格的 max）由发布路径从谱系那一步拿（`publish_version_appending_a_lineage_record`），不改这里的行。
fn version_to_publish_the_rollback_after(
    current: &TransactionOutput,
    target_version: &TransactionOutput,
) -> TransactionOutput {
    let user_visible_units_of_the_target = target_version
        .units
        .iter()
        .filter(|unit| is_a_unit_taken_from_the_rollback_target(unit.identity))
        .cloned();
    let other_units_of_the_current_version = current
        .units
        .iter()
        .filter(|unit| !is_a_unit_taken_from_the_rollback_target(unit.identity))
        .cloned();
    let mapped_units = target_version
        .mapped_units
        .iter()
        .filter(|(identity, _)| is_a_unit_taken_from_the_rollback_target(*identity))
        .chain(
            current
                .mapped_units
                .iter()
                .filter(|(identity, _)| !is_a_unit_taken_from_the_rollback_target(*identity)),
        )
        .cloned()
        .collect();
    let target_root_of_the_tree_of_kind = |kind: u16| {
        target_version
            .tree_table_entries
            .iter()
            .find(|target_entry| target_entry.kind == kind)
            .expect("重建带文件的一版时判过树表里 extent 树与 inode 树各有一条（`recovery::rebuild_version`）")
            .root
    };
    let tree_table_entries = current
        .tree_table_entries
        .iter()
        .map(|entry| TreeTableEntry {
            kind: entry.kind,
            tree: entry.tree,
            root: if entry.kind == TREE_KIND_EXTENT
                || entry.kind == TREE_KIND_INODE
                || entry.kind == TREE_KIND_DELETED_INODES
            {
                target_root_of_the_tree_of_kind(entry.kind)
            } else {
                entry.root
            },
            previous_snapshot_txg: entry.previous_snapshot_txg,
            birth_txg: entry.birth_txg,
            head_identifier: entry.head_identifier,
        })
        .collect();
    let units: Vec<PublishedUnit> = user_visible_units_of_the_target
        .chain(other_units_of_the_current_version)
        .collect();
    // 装好这一版的 units 之后记下两棵容器树根单元的下标（下一次准入读 H 按它直接取）。
    let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);
    TransactionOutput {
        root: current.root,
        record: current.record.clone(),
        record_bytes: current.record_bytes.clone(),
        earlier_records_of_this_publish: Vec::new(),
        units,
        rewritten: Vec::new(),
        data_pointers: target_version.data_pointers.clone(),
        // 第一个文件之外各文件的数据单元与下段同第一个文件的，取回退目标那一版的（用户可见单元）。
        files_other_than_the_first: target_version.files_other_than_the_first.clone(),
        mapping_keys: current.mapping_keys.clone(),
        allocation_records: current.allocation_records.clone(),
        allocation_record_tree: current.allocation_record_tree.clone(),
        extent_tree: target_version.extent_tree.clone(),
        accounting_entries: current.accounting_entries.clone(),
        accounting_tree: current.accounting_tree.clone(),
        // 映射树的节点是现行那一版的（这次发布照它们 COW、换下的照它们释放）；用户可见单元的条目取目标那一版的位置：发布路径按这一版的
        // 指针装条目（目标那一版重建时已按它自己的映射覆盖过提示），装着位置不同的那几把 key 的叶连同到根的路重写
        // （[`mapping_keys_whose_target_locations_differ_from_the_current_mapping`]，判决 m4-segment-code-r1 第三节 F1）。
        central_mapping_tree: current.central_mapping_tree.clone(),
        // 两版都不带快照（回退之前判过，判决 S8）：deadlist 与留存的映射条目照现行那一版（删完全部快照之后是空的）。
        deadlist_entries: current.deadlist_entries.clone(),
        deadlist_tree: current.deadlist_tree.clone(),
        mapping_entries_kept_for_the_snapshots: current
            .mapping_entries_kept_for_the_snapshots
            .clone(),
        snapshots_being_destroyed: current.snapshots_being_destroyed.clone(),
        tree_table_entries,
        tree_identifiers: current.tree_identifiers,
        inode_record: target_version.inode_record,
        // `deleted_inodes` 取 R_old 的（树表那一条上面换了），墓碑树随现行那一版（D23（journal 的角色与格式） 已定项 14）。
        deleted_inodes: target_version.deleted_inodes.clone(),
        // logged_ops 的根与条目照现行那一版（发布路径照它的根判换下）；截断条目取 R_old 的那一换由回退那次发布做
        // （`LoggedOperationsStepOfThePublish::TakeTheTruncationEntryOfTheRollbackTarget`，判决 m4-trunc-r1 K1）：未完成的整理意图跨回退留着、接着推。
        logged_operations: current.logged_operations.clone(),
        tombstone_containers: current.tombstone_containers.clone(),
        tombstone_tree_index_nodes: current.tombstone_tree_index_nodes.clone(),
        // 谱系树同样随现行那一版（D18（块里携带什么信息） 已定项 24 第 5 条「随现行那一版、只增不删」）：回退这次往它上面追加。
        lineage_containers: current.lineage_containers.clone(),
        lineage_tree_index_nodes: current.lineage_tree_index_nodes.clone(),
        // 退役树同样随现行那一版、不取 R_old（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：
        // 回退那次不写退役记录，R_old 之后身份消失、回退之后又活过来的叶容器，它的退役记录死亡 txg 不在新这一版的父链上。
        retirement_containers: current.retirement_containers.clone(),
        retirement_tree_index_nodes: current.retirement_tree_index_nodes.clone(),
        inode_leaf_containers: target_version.inode_leaf_containers.clone(),
        mapped_units,
        released: Vec::new(),
        quarantined_after_release_checksum_mismatch: Vec::new(),
        units_relocated_by_this_publish: Vec::new(),
        key_order_mismatches: 0,
        writes: WritesByStructureKind::NOTHING_WRITTEN,
        highest_transaction_number_in_this_instance: current
            .highest_transaction_number_in_this_instance,
        container_tree_root_unit_positions,
        // 回退那一版的 inode 树取 R_old 那一版的，两个数也照抄它的。
        inode_record_counts: target_version.inode_record_counts,
    }
}

/// 回退那次发布要把中央映射里的条目换回目标那一版位置的那几把映射 key（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
/// 第三节 F1，`research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「映射是唯一权威」）：目标那一版取来的用户可见单元
/// （[`is_a_unit_taken_from_the_rollback_target`]）里，在现行那一版的中央映射里查到的不是目标那一版映射里那两条位置项的。
/// 整理在两版之间搬过的单元就是这样：key 不变、现行那一版的条目指着目的落点，而回退释放的正是目的落点、复活的是源落点。
/// 发布路径只按 key 进出定映射树重写哪些节点，key 相同的叶照抄上一版（现行那一版）的字节，条目就留在被释放的目的落点上；
/// 交出这几把 key，装着它们的叶连同到根的路这次重写（与整理的一批同一条路，`transaction::publish_version_appending_a_lineage_record`）。
/// 现行那一版查不到、条目切不动的也交出（重写一条路是合法的 COW）；目标那一版自己查不到的在 [`user_visible_units_through_the_mapping`] 那一判拒过。
/// 只读两版内存里的映射叶，不读盘。
fn mapping_keys_whose_target_locations_differ_from_the_current_mapping(
    current: &TransactionOutput,
    target_version: &TransactionOutput,
) -> Vec<Vec<u8>> {
    target_version
        .mapped_units
        .iter()
        .filter(|(identity, _)| is_a_unit_taken_from_the_rollback_target(*identity))
        .filter(|(_, key)| {
            central_mapping_locations_in_the_version(current, key)
                != central_mapping_locations_in_the_version(target_version, key)
        })
        .map(|(_, key)| key.clone())
        .collect()
}

/// 管理员回退：挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14「管理员回退是挂着时的一次向前发布」）。
/// 管理员从回退候选集里选 R_old（`target`）；文件系统在现行那一版（`current`，cur）后面发一次普通发布（D16（发布语义） 已定项 7 的
/// 持久顺序）：新根的 inode 树、extent 树取 R_old 的；树表单元、分配记录树、中央映射树与记账统计量按这次发布照调整之后的分配器现算；
/// 实例表照 cur 的；checkpoint_txg 加一；不新增实例、不取号、不写实例表行、不施加也不删除任何记录。
///
/// - 候选集：根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件（[`rollback_candidate`]）。
/// - 释放：cur 引用、新根不引用的用户可见单元（按落点比），释放代 = 新根的 txg；释放之前照发布路径读盘核，对不上的那几份留在已分配。
///   cur 被这次重写换下的固定点单元由发布路径照常释放。
/// - 复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的；cur 的账里带着隔离记号的，
///   同一次发布清掉记号（D19（块指针的结构与宽度预算） 已定项 5）。
/// - 水位：inode 号水位 = max(cur 那一版内存里的, 环里读得出、按 cur 的实例表没被崩溃恢复抛弃的根的)（Opt2）；
///   树 ID 水位 = max(cur 那一版内存里的, 环里读得出的全部根的)。台阶行照抄 cur 的；谱系记录带 H_cur = cur 的 inode 号水位（从 cur 读）。
/// - 调用方的参数与盘表：照可写挂载同一套核（[`caller_inputs_agreeing_with_the_disk`]），写入口只按盘上择到的系统配置里那份参数建。
/// - 在任何写之前拒：见 [`RollbackError`] 除 `Publish` 落盘途中失败之外的每一个成员。
///
/// 新根持久之后才交回 `Ok`（与 fsync 同一个返回条件）；那时 `current` 换成回退那次发布之后的一版。回退那次发布的事务号写 0
/// （它不写数据单元，与空发布同）。回退的调用方要带文件的现行版本：cur 树表 0 条时它的时间线上还没有带文件的版本，
/// 按实例表判仍然有效、带文件的根一条都没有（候选集空），所以这里不收 `PoolVersion`。
/// 调用方手里有会话这一侧的整理状态（挂着的会话）时走 [`roll_back_by_a_forward_publish_in_the_session`]：它另清会话上的上一条完成的意图。
///
/// # Errors
/// 见 [`RollbackError`]。
pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    target: RollbackTarget,
) -> Result<RolledBack, RollbackError> {
    if let Some(frozen) = allocator.frozen_publish() {
        return Err(
            RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                checkpoint_txg: frozen.checkpoint_txg(),
                instance: frozen.instance(),
            },
        );
    }
    // 第一版有快照就拒回退（判决 S8、岔路 3）：现行那一版在内存里，在任何读之前判；目标那一版重建出来之后判。
    let snapshots_in_the_current_version = snapshots_in_the_tree_table(current);
    if snapshots_in_the_current_version > 0 {
        return Err(
            RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
                version: VersionOfTheRollback::Current,
                snapshots: snapshots_in_the_current_version,
            },
        );
    }
    // 调用方的参数与盘表照可写挂载同一套核，写入口只按盘上那一份建。
    let CallerInputsAgreeingWithTheDisk {
        system_configuration,
        parameters_of_the_pool,
    } = caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current).map_err(
        |refused| match refused {
            CallerInputsCheckRefused::Disagreeing(disagreeing) => {
                RollbackError::CallerInputsDisagreeWithTheDisk(disagreeing)
            }
            CallerInputsCheckRefused::SystemConfigurationNotChosen(failure) => {
                RollbackError::Recovery(failure)
            }
        },
    )?;
    let rollback_txg = current
        .root
        .checkpoint_txg
        .0
        .checked_add(1)
        .map(CheckpointTxg)
        .ok_or(RollbackError::NextCheckpointTxgPastTheTopOfItsRange {
            current: current.root.checkpoint_txg,
        })?;
    // 回退读现行那一版与目标那一版按位置条目读单元读到只坏一份都进别的读那份账（D4（校验和位置） 已定项 4「可写挂载期间别的读」）：还住在回退之后那一版里的，
    // 下一次用户改动发成之后另发的那一批换下，已经不在那一版里的到那一批认不出，照丢。
    let current_table = allocator
        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
            instance_table_chain_of_root(reader, &current.root)
        })
        .map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?
        .records;
    let target_root = allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
        &*devices,
        |reader| {
            rollback_candidate(
                reader,
                &system_configuration,
                &current_table,
                &ring_slots_known_to_hold_a_root_by(allocator),
                target,
            )
        },
    )?;
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    // R_old 那一版整个从盘上重建（它的账、它的三棵用户可见树、记账树、中央映射树都读回来）：有一个节点读不出就拒。
    // 顶着的记录只给重建填字段，这里只用那一版的用户可见一半与它的账。
    let target_version = match allocator
        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
            rebuild_version_in_the_unit_area_starting_at(
                reader,
                unit_area_start,
                &target_root,
                Some(current.record.clone()),
            )
        }) {
        // 有树而文件集合为空的一版也是候选：候选那一判按树表有没有条目判（带文件读成「那一版写过文件」），回退到它就是回到删空的那一刻。
        Ok(RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version)) => {
            version
        }
        // 候选那一判刚读过它的树表、不是 0 条：两次读之间的瞬时读错，按候选集那一判同一个结论报。
        Ok(RebuiltVersion::WithoutFile) => {
            return Err(RollbackError::TargetNotACandidate {
                target,
                exclusion: RollbackCandidateExclusion::VersionWithoutFile,
            });
        }
        Err(RebuildVersionFailure::Walk(failure)) => {
            return Err(RollbackError::TargetVersionUnreadable { target, failure });
        }
        Err(RebuildVersionFailure::NoRecordStandingForFileVersion) => {
            panic!(
                "交给重建的是 Some(现行那一版的末条记录)：rebuild_version 只在拿不出记录时报这一格"
            )
        }
    };
    // cur 那一版的账有一个节点读不出就拒：从盘上沿它的根读一遍分配记录树（判定与后面的写读的都是内存里那一份账，这一读只判读不读得出）。
    if let Err(failure) = allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
        &*devices,
        |reader| {
            allocation_records_under_root_in_the_unit_area_starting_at(
                reader,
                unit_area_start,
                &current.root,
            )
        },
    ) {
        return Err(RollbackError::CurrentAccountUnreadable { failure });
    }
    let snapshots_in_the_target_version = snapshots_in_the_tree_table(&target_version);
    if snapshots_in_the_target_version > 0 {
        return Err(
            RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
                version: VersionOfTheRollback::Target,
                snapshots: snapshots_in_the_target_version,
            },
        );
    }
    if target_version.tree_identifiers != current.tree_identifiers {
        return Err(
            RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
                target,
            },
        );
    }
    let target_units = user_visible_units_through_the_mapping(&target_version).map_err(|_| {
        RollbackError::TargetVersionUnreadable {
            target,
            failure: RecoveryFailure::UnitMalformed {
                what: "回退目标那一版引用的用户可见单元在它自己的映射里查不到或条目切不动",
            },
        }
    })?;
    // 现行那一版的单元的槽（两盘同槽）：现行账里仍分配、同分配代而不在其中的，是隔离单元（[`resurrection_against_the_current_account`]）。
    let slots_of_the_units_of_the_current_version: BTreeSet<SlotNumber> =
        current.units.iter().map(|unit| unit.slot).collect();
    let Resurrection {
        resurrected,
        units_to_check_on_every_device,
        isolated_units_to_check_on_every_device,
        quarantine_marks_to_clear,
    } = resurrection_against_the_current_account(
        target,
        &target_version,
        &target_units,
        allocator,
        &slots_of_the_units_of_the_current_version,
    )?;
    every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;
    every_copy_of_the_isolated_units_the_target_references_reads_back(
        &*devices,
        &isolated_units_to_check_on_every_device,
    )?;
    // 释放集：cur 引用的用户可见单元里，落点不在 R_old 引用的那几个里的（在的那几个上面判过同槽同分配代，是两版共用的）。
    let slots_referenced_by_the_target: BTreeSet<(DeviceIdentity, SlotNumber)> = target_units
        .iter()
        .flat_map(|(_, locations)| {
            locations
                .iter()
                .map(|location| (location.device, location.slot))
        })
        .collect();
    let pool_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let not_releasable = |cause| RollbackError::CurrentVersionUnitNotReleasable { cause };
    let mut released_user_visible_units = Vec::new();
    let mut quarantined_user_visible_copies = Vec::new();
    for (unit, locations) in
        user_visible_units_through_the_mapping(current).map_err(not_releasable)?
    {
        if locations.iter().any(|location| {
            slots_referenced_by_the_target.contains(&(location.device, location.slot))
        }) {
            continue;
        }
        refuse_locations_that_do_not_name_two_pool_devices(unit, &locations, &pool_devices)
            .map_err(not_releasable)?;
        let placement =
            placement_to_release_after_checking_every_device(unit, &locations, allocator)
                .map_err(not_releasable)?;
        quarantined_user_visible_copies.extend(
            allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
                &*devices,
                |reader| {
                    copies_of_a_released_unit_failing_the_release_checksum_check(
                        unit, &locations, reader,
                    )
                },
            ),
        );
        released_user_visible_units.push((unit, placement));
    }
    let ring_roots = readable_roots(
        &*devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let tree_identifier_watermark = tree_identifier_watermark_of_the_rollback(
        &ring_roots,
        &current_table,
        current.root.tree_identifier_watermark,
    );
    // inode 号水位（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）：环里只取按 cur 的实例表没被崩溃恢复抛弃的根；
    // max 里 cur 那一版内存里的那一项不能去——只取环，回退那次就重发号（C143）。
    let inode_number_watermark = inode_number_watermark_of_the_rollback(
        &ring_roots,
        &current_table,
        current.inode_number_watermark(),
        |ring_roots_not_abandoned| {
            allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
                &*devices,
                |reader| {
                    highest_inode_number_watermark_in_the_ring(
                        reader,
                        unit_area_start,
                        ring_roots_not_abandoned,
                        &current.record,
                    )
                },
            )
        },
    );
    let previous = version_to_publish_the_rollback_after(current, &target_version);
    let mapping_keys_the_rollback_moves_back_to_the_target_locations =
        mapping_keys_whose_target_locations_differ_from_the_current_mapping(
            current,
            &target_version,
        );
    // 动分配器：先复活、再释放（释放代 = 新根的 txg），之后交给发布路径——它在这份账上照常释放 cur 被换下的固定点单元、取落点、
    // 装记账行。发布在落盘之前被拒时整个换回这一份之前的样子。
    let allocator_before_the_rollback = allocator.clone();
    for copy in &resurrected {
        allocator.resurrect_released_record(
            copy.device,
            copy.placement,
            copy.allocation_generation,
        );
    }
    for (device, placement) in &quarantine_marks_to_clear {
        allocator.clear_the_quarantine_mark_of_a_record_the_rollback_target_references(
            *device, *placement,
        );
    }
    for (_, placement) in &released_user_visible_units {
        let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> =
            quarantined_user_visible_copies
                .iter()
                .filter(|copy: &&CopyQuarantinedAfterReleaseChecksumMismatch| {
                    copy.placement == *placement
                })
                .map(|copy| copy.device)
                .collect();
        allocator.release_leaving_the_record_allocated_on(
            *placement,
            rollback_txg,
            &devices_whose_copy_failed_the_checksum,
        );
    }
    let plan = PublishPlan {
        txg: rollback_txg,
        counter: current.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current
            .highest_transaction_number_in_this_instance,
        instance: current.root.instance,
        back_chain: back_chain_of(&current.record_bytes),
        file: None,
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current.root.instance_table),
        tree_birth_txg: current.tree_birth_txg(),
        tree_identifier_watermark,
        rollback_floor: current.root.rollback_floor,
    };
    // 回退那次发布追加一条谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条）：T_rb 是这次的 txg，T_old 是 R_old 的 txg，
    // H_cur 是 cur 那一版的 inode 号水位（从 cur 读，不从交给发布路径的「上一版」现算）。
    let lineage_record = LineageRecord {
        rollback_txg,
        rollback_target_txg: target_root.checkpoint_txg,
        inode_number_watermark_of_the_version_rolled_back_from: current.inode_number_watermark(),
    };
    let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
    // 谱系容器与容器树节点的容量取分配器上装的只供测试的开关（产品路径按格式），与会话里的删除同一个做法。
    pool.set_container_tree_capacities(allocator.deletion_switches().container_tree_capacities());
    match publish_version_appending_a_lineage_record(
        &mut pool,
        allocator,
        plan,
        &previous,
        lineage_record,
        inode_number_watermark,
        &mapping_keys_the_rollback_moves_back_to_the_target_locations,
        // 截断条目取 R_old 那一版的（判决 m4-trunc-r1 K1）：截断批间回退到批间那一版，续做照它那一条截到底；
        // 现行那一版截到一半而 R_old 没有截断条目，这一条随回退一起去掉，不截 R_old 的内容。
        target_version
            .logged_operations
            .iter()
            .copied()
            .find(|entry| entry.operation.kind() == LoggedOperationKind::Truncation),
    ) {
        Ok(published) => {
            *current = published;
            Ok(RolledBack {
                target,
                released_user_visible_units,
                quarantined_user_visible_copies,
                resurrected_user_visible_copies: resurrected,
            })
        }
        Err(cause) => {
            // 落盘之前被拒：发布路径把分配器换回了复活、释放之后那一份，这里再换回回退之前的。落盘途中失败：那次发布冻结在分配器上
            // （连同它成立之后的那一份），分配器留着复活、释放之后那一份，重发成功时整个换成冻结着的那一份。
            if allocator.frozen_publish().is_none() {
                *allocator = allocator_before_the_rollback;
            }
            Err(RollbackError::Publish(publish_sequence_failed(
                &pool,
                WritesByStructureKind::NOTHING_WRITTEN,
                Vec::new(),
                cause,
            )))
        }
    }
}

/// 会话内回滚：挂着的会话上做一次 [`roll_back_by_a_forward_publish`]，连同会话这一侧的整理状态（`compaction`）一起交进来。
/// 回退做成（新根持久、交回 `Ok`）时清掉会话上跨趟留着的上一条完成的意图（[`CompactionOfThisMount::last_completed_intent`]），
/// 下一趟整理从空起、照常签新意图：D26（后台整理与放置回收） 已定项 13 第 6 条「签下一条意图那一判跨趟留着」留到回滚为止，
/// 用户 2026-10-26 定「回滚时清掉」（`records/2026-10-19-里程碑四GPU垃圾回收开工.md`「写段冒出的新发现」那一行）。
/// 回退被拒、落盘途中失败都没做成，那一条照旧留着。盘上的未完成意图（现行那一版 logged_ops 里那条）不归这里：它随现行那一版跨回退留着、接着推。
///
/// # Errors
/// 同 [`roll_back_by_a_forward_publish`]。
pub fn roll_back_by_a_forward_publish_in_the_session<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    compaction: &mut CompactionOfThisMount,
    target: RollbackTarget,
) -> Result<RolledBack, RollbackError> {
    let rolled_back =
        roll_back_by_a_forward_publish(parameters, devices, allocator, current, target)?;
    compaction.last_completed_intent = None;
    Ok(rolled_back)
}

/// 管理员回退取 inode 号水位那一段（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）的构造态单测：环里那一项在走得到的历史上
/// 不大于 cur 内存里的，去掉它回退用例与随机历史全绿（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第二节 K1、第三节 F3），
/// 这里直接交一份环里那条根的水位高于 cur 的输入。
#[cfg(test)]
mod inode_number_watermark_of_the_rollback_tests {
    use std::collections::BTreeMap;

    use super::inode_number_watermark_of_the_rollback;
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::instance_table::{InstanceRow, InstanceTableRecords};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};

    /// cur 那一版内存里的 inode 号水位。
    const INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION: u64 = 40;
    /// 环里那条根（实例 1、txg 7）那一版的 inode 号水位：高于 cur 的。
    const INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION: u64 = 90;
    /// 环里另一条根（实例 1、txg 6）那一版的水位：低于 cur 的。
    const INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION: u64 = 25;
    const INSTANCE_OF_THE_RING_ROOTS: InstanceGeneration = InstanceGeneration(1);
    const TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION: CheckpointTxg = CheckpointTxg(7);
    const TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION: CheckpointTxg = CheckpointTxg(6);

    fn ring_root(checkpoint_txg: CheckpointTxg) -> RootRecord {
        RootRecord {
            filesystem_identifier: [7; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: INSTANCE_OF_THE_RING_ROOTS,
            checkpoint_txg,
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 65_549,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: crate::address::CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    /// cur 的实例表：实例 1 那一行的所选根 txg 是 `selected_root_txg`（txg 大于它的实例 1 的根被崩溃恢复抛弃）。
    fn current_table_selecting(selected_root_txg: CheckpointTxg) -> InstanceTableRecords {
        InstanceTableRecords {
            rows: vec![InstanceRow {
                instance: INSTANCE_OF_THE_RING_ROOTS,
                selected_root_txg,
                applied_transaction_high_water: 0,
            }],
        }
    }

    /// 替盘上重建：交进来的每条根那一版的水位（按 txg 查），取最大；一条都没交进来是 `None`。
    fn highest_watermark_of_the_roots_handed_in(roots: &[RootRecord]) -> Option<u64> {
        let watermark_of_each_txg = BTreeMap::from([
            (
                TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
                INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
            ),
            (
                TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION,
                INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION,
            ),
        ]);
        roots
            .iter()
            .map(|root| {
                *watermark_of_each_txg
                    .get(&root.checkpoint_txg)
                    .expect("只交这几条构造出来的根")
            })
            .max()
    }

    /// 环里一条没被抛弃的根（实例表里实例 1 选到 txg 9，txg 7 不大于它）那一版的水位 90 高于 cur 的 40：取 90。
    #[test]
    fn ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark() {
        assert_eq!(
            inode_number_watermark_of_the_rollback(
                &[ring_root(TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION)],
                &current_table_selecting(CheckpointTxg(9)),
                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
                highest_watermark_of_the_roots_handed_in,
            ),
            INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
            "环里那条根没被抛弃，水位取它与 cur 的最大"
        );
    }

    /// 同一条根，实例表里实例 1 只选到 txg 5（txg 7 > 5，被崩溃恢复抛弃）：它那一版的水位不算，取 cur 的 40。
    #[test]
    fn ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark() {
        assert_eq!(
            inode_number_watermark_of_the_rollback(
                &[ring_root(TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION)],
                &current_table_selecting(CheckpointTxg(5)),
                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
                highest_watermark_of_the_roots_handed_in,
            ),
            INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
            "被崩溃恢复抛弃的根那一版发过的号本来就会重发，水位不跳到它"
        );
    }

    /// 环里没被抛弃的根那一版的水位 25 低于 cur 的 40：cur 那一项不能去（只取环，回退那次就重发号，C143），取 40。
    #[test]
    fn ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark() {
        assert_eq!(
            inode_number_watermark_of_the_rollback(
                &[ring_root(TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION)],
                &current_table_selecting(CheckpointTxg(9)),
                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
                highest_watermark_of_the_roots_handed_in,
            ),
            INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
            "水位取环与 cur 的最大，不只取环"
        );
    }
}

/// 管理员回退取树 ID 水位那一段（D23（journal 的角色与格式） 已定项 14「水位」一格：树 ID 水位照旧取环里读得出的全部根）的构造态单测：
/// 回退用例在走得到的历史上分不出「全部根」与「只取没被崩溃恢复抛弃的根」（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
/// 第二节 K3、第三节 G5），这里直接交一条按 cur 的实例表被抛弃、树 ID 水位高于 cur 的根。
#[cfg(test)]
mod tree_identifier_watermark_of_the_rollback_tests {
    use super::{abandoned_by_table, tree_identifier_watermark_of_the_rollback};
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::instance_table::{InstanceRow, InstanceTableRecords};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};

    /// cur 那一版内存里的树 ID 水位。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION: u64 = 65_549;
    /// 环里被抛弃的那条根（实例 1、txg 7）的树 ID 水位：高于 cur 的。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION: u64 =
        65_560;
    /// 环里一条没被抛弃的根（实例 1、txg 6）的树 ID 水位：低于 cur 的。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION: u64 =
        65_540;
    /// 环里被抛弃的另一条根（实例 1、txg 8）的树 ID 水位：低于 cur 的。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_BELOW_THE_CURRENT_VERSION: u64 =
        65_545;
    const INSTANCE_OF_THE_RING_ROOTS: InstanceGeneration = InstanceGeneration(1);
    /// cur 的实例表里实例 1 那一行的所选根 txg：txg 大于它的实例 1 的根被崩溃恢复抛弃。
    const SELECTED_ROOT_TXG_OF_THE_CURRENT_TABLE: CheckpointTxg = CheckpointTxg(6);

    fn ring_root(checkpoint_txg: CheckpointTxg, tree_identifier_watermark: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [7; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: INSTANCE_OF_THE_RING_ROOTS,
            checkpoint_txg,
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: crate::address::CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    fn current_table() -> InstanceTableRecords {
        InstanceTableRecords {
            rows: vec![InstanceRow {
                instance: INSTANCE_OF_THE_RING_ROOTS,
                selected_root_txg: SELECTED_ROOT_TXG_OF_THE_CURRENT_TABLE,
                applied_transaction_high_water: 0,
            }],
        }
    }

    /// 环里一条没被抛弃的根（txg 6，水位 65540，低于 cur）与一条按 cur 的实例表被抛弃的根（txg 7 > 6，水位 65560，高于 cur 的 65549）：
    /// 取 65560——被抛弃的那条照旧算。
    #[test]
    fn an_abandoned_ring_root_whose_tree_identifier_watermark_is_above_the_current_version_sets_the_watermark(
    ) {
        let abandoned_root = ring_root(
            CheckpointTxg(7),
            TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
        );
        let root_not_abandoned = ring_root(
            CheckpointTxg(6),
            TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION,
        );
        let current_table = current_table();
        assert!(
            abandoned_by_table(&abandoned_root, &current_table)
                && !abandoned_by_table(&root_not_abandoned, &current_table),
            "前提：txg 7 那条按 cur 的实例表被抛弃，txg 6 那条没有"
        );
        assert_eq!(
            tree_identifier_watermark_of_the_rollback(
                &[root_not_abandoned, abandoned_root],
                &current_table,
                TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
            ),
            TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
            "树 ID 水位取环里读得出的全部根：被崩溃恢复抛弃的那条照旧算"
        );
    }

    /// 环里两条根（txg 6 没被抛弃、水位 65540；txg 8 被抛弃、水位 65545）都低于 cur 的 65549：取 cur 的。
    #[test]
    fn ring_roots_all_below_the_current_version_leave_the_current_tree_identifier_watermark() {
        let current_table = current_table();
        assert_eq!(
            tree_identifier_watermark_of_the_rollback(
                &[
                    ring_root(
                        CheckpointTxg(6),
                        TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION,
                    ),
                    ring_root(
                        CheckpointTxg(8),
                        TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_BELOW_THE_CURRENT_VERSION,
                    ),
                ],
                &current_table,
                TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
            ),
            TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
            "水位取环与 cur 的最大，不只取环"
        );
    }
}

#[cfg(test)]
mod rollback_does_not_verify_units_the_current_version_shares_with_the_target_tests {
    use super::pool_in_memory_after_the_first_file::pool_after_the_first_file;
    use super::RollbackTarget;
    use crate::address::DeviceIdentity;
    use crate::transaction::QuarantinedCopyReading;
    use singlefs_format::DATA_UNIT_BYTES;

    /// 改坏一份数据单元时翻的那个字节在单元里的下标（净荷里，单元头之后）。
    const BYTE_FLIPPED_INSIDE_THE_DATA_UNIT: usize = 4096;

    /// 「cur 那一版引用的单元不进逐盘验」（判决 `research/prompts/code-backlog-b2-code-r1-main-verification.md` 第三节 A2；D23（journal 的角色与格式）
    /// 已定项 14「其余 cur 的账里仍分配的单元……不在逐盘验之内」）：第一个文件 A；盘 1 上 A 的数据单元那一份改坏，覆盖写 B 释放它时读盘核出对不上、
    /// 隔离；那一份写回原样、回退到 A（回退那次发布的新根引用这个数据单元）；再把盘 1 那一份改坏、再回退到 A。现行那一版与 A 共用这个单元，它在现行账里
    /// 分配着、分配器发不出去，不进逐盘验：回退照做。判别力：那一道去掉（`true || !is_referenced_by_the_current_version`）时它进隔离单元的逐盘验、
    /// 拒成 `IsolatedUnitCopyUnreadableOrMismatched`，这一条红。两条位置项不同槽的映射条目怎么认条款没写（K2.4），这里只取同槽。
    #[test]
    fn rolling_back_to_a_version_sharing_a_unit_with_a_bad_copy_with_the_current_version_goes_through(
    ) {
        let mut pool = pool_after_the_first_file();
        let first = pool.current.clone();
        let target = RollbackTarget {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
        };
        let data_locations = first.data_pointers[0].locations;
        assert_eq!(
            data_locations[0].slot, data_locations[1].slot,
            "前提：A 的数据单元两份同槽"
        );
        let data_slot = data_locations[0].slot;
        let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
        let intact_copy = pool.bytes_on(
            DeviceIdentity(1),
            data_slot.to_device_offset(),
            data_unit_bytes,
        );
        let mut corrupted_copy = intact_copy.clone();
        corrupted_copy[BYTE_FLIPPED_INSIDE_THE_DATA_UNIT] ^= 0xff;
        pool.write_bytes_directly_on(
            DeviceIdentity(1),
            data_slot.to_device_offset(),
            &corrupted_copy,
        );
        let second = pool.overwrite();
        assert!(
            second
                .quarantined_after_release_checksum_mismatch
                .iter()
                .any(|copy| copy.device == DeviceIdentity(1)
                    && copy.placement.slot == data_slot
                    && copy.reading == QuarantinedCopyReading::ChecksumMismatch),
            "前提：B 释放 A 的数据单元时核出盘 1 那一份对不上、隔离：{:?}",
            second.quarantined_after_release_checksum_mismatch
        );
        pool.write_bytes_directly_on(
            DeviceIdentity(1),
            data_slot.to_device_offset(),
            &intact_copy,
        );
        pool.roll_back(target)
            .expect("隔离单元两份都对得上：第一次回退照做");
        assert!(
            pool.current.data_pointers.iter().any(|pointer| pointer
                .locations
                .iter()
                .any(|location| location.slot == data_slot)),
            "前提：第一次回退之后现行那一版引用 A 的数据单元"
        );
        pool.write_bytes_directly_on(
            DeviceIdentity(1),
            data_slot.to_device_offset(),
            &corrupted_copy,
        );
        let second_rollback = pool.roll_back(target);
        assert!(
            second_rollback.is_ok(),
            "现行那一版与目标共用的单元不进逐盘验，盘 1 那一份坏着回退照做：{:?}",
            second_rollback.err()
        );
    }
}
