//! 影子账：只被被抛弃根引用的槽逐槽隔离（[`isolate_slots_referenced_only_by_abandoned_roots`]），豁免与判重按槽收；
//! 被抛弃根的账读不出时它引用的落点从哪来（从映射现算），一条根引用着的落点。

use super::*;

/// 影子账里一条被抛弃根引用的落点从哪来（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句「抬 F 那一串（准入与卸载）里的被抛弃根」、
/// D23（journal 的角色与格式） 已定项 14 回退段「抬 F 那一串（准入与卸载）不走这一道」，用户 2026-10-07 定）。封闭集合，`match` 不写通配臂。
pub(super) enum PlacementsOfAbandonedRoots<'recorded> {
    /// 可写挂载重建分配器那一遍：从盘上读它的账，读不出照 C393 从映射现算、重读一次、再看 C694 的 B 凭据，仍不成就拒
    /// （「崩了再可写挂载」那一道照 B 拒，不变）。
    ReadFromTheirAccounts,
    /// 抬 F 那一串（准入与卸载）：按 (实例代号, txg) 认、根环表里记着同一条被抛弃根的（`allocator::RootRingOccupancy::placements_recorded_for_the_abandoned_root`），
    /// 照挂载时算出、记着的那一串落点算，不再从盘上读它的账，读不出也不拒。挂载之后才按现行那一版的实例表判成被抛弃的根（挂载那一刻它按最新那条根的表
    /// 判有效、记成有效根）借 C 那一圈在那一槽记着的落点（那一槽记的是有效根且 txg 相同，
    /// `allocator::RootRingOccupancy::placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot`，用户 2026-10-08 定 H2），
    /// 同样不读账、读不出不拒。两样都没记着的（C 在那一槽也没记的；挂载那一刻读坏、读回全零的槽这一遍读出的被抛弃根；
    /// 进程内重读读出被抛弃根、已从表里拿掉的槽）照可写挂载那一遍读盘（「C 也没记着的照今天读盘」）。
    RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
        &'recorded BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
    ),
}

impl PlacementsOfAbandonedRoots<'_> {
    /// 根环表里记着的这条被抛弃根的落点；读盘那一臂与表里没记着它时 `None`。只读内存。
    fn recorded_in_the_root_ring_table(
        &self,
        root: &RootRecord,
    ) -> Option<Vec<(DeviceIdentity, SlotNumber, u64)>> {
        match self {
            Self::ReadFromTheirAccounts => None,
            Self::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(recorded) => recorded
                .get(&(root.instance, root.checkpoint_txg))
                .map(|placements| {
                    placements
                        .iter()
                        .map(|placement| (placement.device, placement.slot, placement.span))
                        .collect()
                }),
        }
    }
}

/// 影子账（D28（挂载期承诺量） 已定项 1 第九项）：只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里
/// 还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F。用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，
/// 豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严（alloc-basis 那一轮的 G5 臂）：按原措辞，抬 F 到 11 之后
/// A 仍豁免 mkfs 实例表那 2 槽，它们被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句。
/// 预想、偏离用户定案的措辞，交用户。
/// 一条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
/// 被抛弃根的树表或分配记录树读不出、解不开时不跳过：从中央映射现算它引用的落点照样隔离，计进 `abandoned_roots_unreadable`
/// （C393 取 (b)-从映射现算）；映射也现算不成就重读一次（重读之前调钩子），仍不成时看现行账（`current_records`）：现行账里那条根的树表槽
/// （与中央映射树根槽，映射根是空指针时只看树表槽）每块盘上都有一条分配代大于那条根的 txg、罩住它的记录，就是那几槽已被那条根之后的合法写占了，
/// 那条根引用的单元已经没有要护的，只计数、不隔离、不拒（C694 的 B，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条：
/// 凭据只读内存里的现行账，不读那一槽）；没有这样的记录（那几槽只是这一次读不出）就在任何写之前拒：挂载在取号之前拒可写、抬 F 拒这一次抬
/// （抬 F 那一串只在根环表里没记着这条根时才读盘、才走到这一拒）
/// （[`StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`]）。候选根读不出就当它什么都不豁免（隔离只会多不会少）。
/// 同一次也隔离水位之上没施加的 journal 记录点名的落点、那几次发布的账里的落点与比所选那一版新的可读根的账里的落点
/// （`placements_named_by_unapplied_journal_records`，C694 的 C 连同 `C_ACCOUNT`、`C_NEWER_ROOTS`、`C_MAPPING`）：
/// 只豁免现行账里仍分配着的槽，不豁免候选集里的根引用的槽（根环线 K4-1，函数体里那一圈的注释）；落点里分配位置着的槽
/// （已释放、还在 defer 窗口里的也算）不隔离、回收时补（同一圈的注释），D28（挂载期承诺量） 已定项 1 第九项
/// 按分配器实际隔离的槽数取，两部分都在里面。
/// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
/// 被抛弃根引用的只在它离开根环的那一次发布里清，水位之上的记录点名的只在这个进程写掉那条记录的 txg 对应的根环槽那一次发布里清
/// （`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点与分配器根环表记着的那一份）。
/// 被抛弃根引用的落点从哪来由 `placements_of_abandoned_roots` 定（[`PlacementsOfAbandonedRoots`]）：可写挂载那一遍读盘，
/// 抬 F 那一串取根环表里记着的那一份，表里没记着这条根的才读盘（上面那一串读不出的处置只走在这一支上）。
/// `placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad` 是抬 F 那一遍根槽读坏、没读出来的被抛弃根挂载时记在根环表里的落点
/// （D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1「被抛弃根的槽不拒」，[`readable_roots_for_raising_the_floor`]）：
/// 与读出来的被抛弃根同一种豁免（候选集里的根引用的、当前账里还分配着的不隔离），不计进 `abandoned_roots_unreadable`（它们的账一个字都没读）；
/// 可写挂载那一遍给空。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：读者、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、被抛弃根的落点从哪来、重读前的快照、根槽这一遍读坏的被抛弃根记着的落点"
)]
pub(super) fn isolate_slots_referenced_only_by_abandoned_roots<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    allocator: &mut PoolAllocator,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    floor: CheckpointTxg,
    current_records: &[AllocationRecord],
    placements_named_by_unapplied_journal_records: &[PlacementOnDevice],
    placements_of_abandoned_roots: PlacementsOfAbandonedRoots<'_>,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[PlacementOnDevice],
) -> Result<ShadowLedgerComputed, MountError> {
    // 豁免与判重一律按槽：每份落点罩着的每一槽各进集合（D23（journal 的角色与格式） 已定项 14「只隔离其中只被被抛弃根引用的槽」，
    // 判决 `research/prompts/m3-c554-c572-r1-main-verification.md` 第二节 B3 那一行）。按起点槽认时，同一起点、跨度不同的两份落点
    // 窄的那份把宽的整份豁免或判重掉，宽的那份多出来的槽没人罩（攻方 `research/prompts/m3-c554-c572-r1-opus-output.md`「B3」一节的三种形）。
    let referenced_by_the_current_account: BTreeSet<(u32, u64)> = current_records
        .iter()
        .filter(|record| !record.is_released)
        .flat_map(|record| {
            slots_covered_by_a_placement(record.device, record.slot, u64::from(record.span_slots))
        })
        .collect();
    let mut referenced_by_candidates: BTreeSet<(u32, u64)> =
        referenced_by_the_current_account.clone();
    // 候选 = 按实例表有效 ∧ txg ≥ F（D16（发布语义） 已定项 1：影子账的候选过滤是原来同取一圈窗口的六处之一，用户 2026-10-04 定 RGall
    // 一起退回整个环：窗口外的有效根照样是候选，它引用的槽豁免）。
    for root in roots
        .iter()
        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
    {
        if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {
            referenced_by_candidates.extend(placements.iter().flat_map(
                |(device, slot, span_slots)| {
                    slots_covered_by_a_placement(*device, *slot, *span_slots)
                },
            ));
        }
    }
    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut unreadable = 0;
    let mut placements_referenced_by_abandoned_root = BTreeMap::new();
    for root in roots.iter().filter(|root| is_abandoned(root)) {
        // C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮；
        // `research/prompts/unreadable-at-mount-r2-main-verification.md`「L3　C393」一节）：账（树表或分配记录树）读不出时不跳过，
        // 从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报（显式报告）；
        // 现算也不成（映射树读不出、解不开）才重读一次（重读之前调钩子），账与现算都仍不成就在取号之前拒可写、只读照常。
        // 抬 F 那一串先取根环表里记着的那一份（`or_else` 是惰性的：取到就不读盘，读不出的处置也走不到）。
        let recorded_in_the_root_ring_table =
            placements_of_abandoned_roots.recorded_in_the_root_ring_table(root);
        // 回收时补不补得上隔离（第 10z 批第十四块实现员报告第七节第 2 条，派发第十六块第 2 条）：根环表里记着这条根的落点（表里的被抛弃根、
        // 借 C 那一圈的），或可写挂载那一遍（这一遍读出的落点随后整串记进表里，`rebuilt_allocator` 建根环表那一步），回收出来的槽照表补隔离
        // （`allocator::RootRingOccupancy::placements_the_isolation_still_protects`），分配位置着的槽就不在这里隔离、不与「已分配」双扣；
        // 抬 F 那一串读盘得来、表里没记着的，回收时没有一份补得上，照旧整段隔离。
        let reclaim_isolates_what_this_root_references = match (
            &placements_of_abandoned_roots,
            &recorded_in_the_root_ring_table,
        ) {
            (PlacementsOfAbandonedRoots::ReadFromTheirAccounts, _)
            | (
                PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                    _,
                ),
                Some(_),
            ) => true,
            (
                PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                    _,
                ),
                None,
            ) => false,
        };
        let placements = match recorded_in_the_root_ring_table
            .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))
        {
            Some(placements) => placements,
            None => {
                unreadable += 1;
                placements_of_an_abandoned_root_whose_account_is_unreadable(
                    devices,
                    unit_area_start,
                    root,
                    current_records,
                    before_the_one_reread,
                )?
            }
        };
        for (device, slot, span_slots) in placements.iter().copied() {
            for slot_to_isolate in slots_of_a_placement_to_isolate_now(
                device,
                slot,
                span_slots,
                &referenced_by_candidates,
                &mut isolated,
            ) {
                if reclaim_isolates_what_this_root_references {
                    allocator
                        .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
                            device,
                            slot_to_isolate,
                            SPAN_OF_ONE_SLOT,
                        );
                } else {
                    allocator.isolate_abandoned(device, slot_to_isolate, SPAN_OF_ONE_SLOT);
                }
            }
        }
        placements_referenced_by_abandoned_root.insert(
            (root.instance, root.checkpoint_txg),
            placements
                .into_iter()
                .map(|(device, slot, span)| PlacementOnDevice { device, slot, span })
                .collect(),
        );
    }
    // 根槽这一遍读坏的被抛弃根（H1）：读不出它的根记录，照它挂载时记着的落点补隔离，豁免与读出来的被抛弃根同一个集合。
    // 落点取自根环表，回收时照表补隔离：分配位置着的槽不在这里隔离（同上一圈）。
    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {
        for slot_to_isolate in slots_of_a_placement_to_isolate_now(
            placement.device,
            placement.slot,
            placement.span,
            &referenced_by_candidates,
            &mut isolated,
        ) {
            allocator
                .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
                    placement.device,
                    slot_to_isolate,
                    SPAN_OF_ONE_SLOT,
                );
        }
    }
    // C 那一份不豁免候选根引用的槽（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K1 / K2 T1 C 的候选豁免」那一行，
    // 根环线 K4-1）：水位之上没施加的那次发布的根读得出、账读得出时它自己就是候选根（按实例表有效、txg ≥ F），豁免它引用的槽就是不护它的单元——
    // 它比所选那一版新，它新分配的单元不在现行账里、回收门槛也罩不到，挂载之后当空闲槽发出去。只豁免现行账里仍分配着的槽：它们这时发不出去，
    // 这次挂载里被释放、回收出来的时候，分配器按还没解开的这一份补隔离（`PoolAllocator::record_root_written_by_this_process`）；
    // 不豁免它们，第九项与「已分配」就把同一个槽数两遍。
    // 已释放、还在 defer 窗口里的槽同样在「已分配」里（分配位置着，`DeviceFreeMap::allocated_slots` 的读法甲），D28（挂载期承诺量） 已定项 1
    // 第九项「不在这里再算一次」：落点里分配位置着的槽一律不隔离（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md`
    // 第二节 K1-3 H2：宽护并进来的已释放落点被第九项与「已分配」各扣一次，1536 槽盘挂载那一刻隔离 2242 槽、写全被拒），它们回收的那一刻
    // 分配器照还没解开的这一份补隔离：挂载之内记根时（`PoolAllocator::record_root_written_by_this_process`），抬 F 那一串回收时
    // （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`，[`raise_rollback_floor`]）。
    for placement in placements_named_by_unapplied_journal_records {
        for slot_to_isolate in slots_of_a_placement_to_isolate_now(
            placement.device,
            placement.slot,
            placement.span,
            &referenced_by_the_current_account,
            &mut isolated,
        ) {
            allocator
                .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
                    placement.device,
                    slot_to_isolate,
                    SPAN_OF_ONE_SLOT,
                );
        }
    }
    Ok(ShadowLedgerComputed {
        abandoned_roots_unreadable: unreadable,
        placements_referenced_by_abandoned_root,
    })
}

/// 影子账一次隔离一槽时交给分配器的跨度（[`slots_of_a_placement_to_isolate_now`] 按槽交回要隔离的槽）。
const SPAN_OF_ONE_SLOT: u64 = 1;

/// 一份落点罩着的每一槽，按 (设备号, 槽号)，槽号升序：影子账的豁免集合与判重集合都按它收（[`isolate_slots_referenced_only_by_abandoned_roots`]）。
///
/// # Panics
/// 末槽装不进 u64：进影子账的落点都判过落在那块盘的单元区里（盘上读来的经 `recovery::placement_lies_in_the_unit_area_of_its_device`，
/// 现行账里的记录经 `recovery::allocation_records_fit_the_pool_geometry`），走不到。
fn slots_covered_by_a_placement(
    device: DeviceIdentity,
    slot: SlotNumber,
    span_slots: u64,
) -> impl Iterator<Item = (u32, u64)> {
    let end_slot = slot.0.checked_add(span_slots).expect(
        "落点末槽装得进 u64：进影子账的落点都判过落在那块盘的单元区里（recovery::placement_lies_in_the_unit_area_of_its_device、allocation_records_fit_the_pool_geometry）",
    );
    (slot.0..end_slot).map(move |covered_slot| (device.0, covered_slot))
}

/// 一份要护的落点（被抛弃根引用的，或水位之上没施加的记录点名的）里这一次要隔离的槽：它罩着的每一槽里，不在 `exempt` 里、
/// 也还没进 `isolated` 的那几槽，槽号升序；交回的槽同时记进 `isolated`，`exempt` 里的不记（与改按槽之前的判重同一个次序：先看豁免，再判重）。
/// 只动两个集合，不碰分配器。
fn slots_of_a_placement_to_isolate_now(
    device: DeviceIdentity,
    slot: SlotNumber,
    span_slots: u64,
    exempt: &BTreeSet<(u32, u64)>,
    isolated: &mut BTreeSet<(u32, u64)>,
) -> Vec<SlotNumber> {
    slots_covered_by_a_placement(device, slot, span_slots)
        .filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))
        .map(|(_, covered_slot)| SlotNumber(covered_slot))
        .collect()
}

/// 一条被抛弃根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来（C393 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮；
/// C694 的 B，用户 2026-10-07 定）：[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
/// 那一份交 `None` 就拒。
///
/// # Errors
/// 账与映射重读一次仍都不成、现行账里又没有那道凭据 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`（在任何写之前）。
fn placements_of_an_abandoned_root_whose_account_is_unreadable<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
    placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread(
        devices,
        unit_area_start,
        root,
        current_records,
        before_the_one_reread,
    )
    .ok_or_else(|| {
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
                abandoned_root: RollbackTarget {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
            },
        ))
    })
}

/// 一条根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来，被抛弃根（[`placements_of_an_abandoned_root_whose_account_is_unreadable`]）
/// 与可写挂载的 C 读一次没施加的发布（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）共用这一份、交 `None` 时各报各的拒因：
/// 从中央映射现算；现算也不成就重读一次（重读之前调钩子），账或现算有一样成就用它；仍都不成时，现行账里有那条根之后的记录罩住它的树表槽
/// （与映射根槽）就当它什么都不引用（[`the_current_account_records_later_units_over_the_account_slots_of`]），没有就交 `None`。
pub(super) fn placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread<
    Reader: PoolReader,
>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Option<Vec<(DeviceIdentity, SlotNumber, u64)>> {
    if let Some(computed) =
        placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
    {
        return Some(computed);
    }
    before_the_one_reread.before_rereading();
    // 重读那一遍绕过这次挂载记下的那一份、再读一次盘（`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(devices);
    if let Some(placements) =
        placements_referenced_by_root(&rereading_reader, unit_area_start, root).or_else(|| {
            placements_referenced_by_root_computed_from_the_mapping(
                &rereading_reader,
                unit_area_start,
                root,
            )
        })
    {
        return Some(placements);
    }
    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {
        return Some(Vec::new());
    }
    None
}

/// C694 的 B 那道凭据（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条）：被抛弃根 `root` 的树表槽、与它的中央映射树根槽
/// （映射根是空指针时没有这一槽，只看树表槽）的每一条位置条目，在现行账 `current_records` 里那块盘上都有一条分配代大于 `root` 的 txg、
/// 罩住那一槽的记录（已释放的也算：分配代说的是那一槽在 `root` 之后被重新分配过）。只读内存里的现行账，不读那几槽的内容——读回旧内容的
/// 瞬时故障下读那一槽判「诞生身份不同」会把活着的根判成已被占（判决第二节 K3 那一行，W1 出局）。
fn the_current_account_records_later_units_over_the_account_slots_of(
    root: &RootRecord,
    current_records: &[AllocationRecord],
) -> bool {
    let mapping_root_locations: &[crate::pointer::LocationEntry] =
        if root.mapping_root == crate::pointer::NodePointer::empty_root() {
            &[]
        } else {
            &root.mapping_root.locations
        };
    root.tree_table
        .locations
        .iter()
        .chain(mapping_root_locations)
        .all(|location| {
            current_records.iter().any(|record| {
                record.device == location.device
                    && record.slot.0 <= location.slot.0
                    && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
                    && record.generation > root.checkpoint_txg
            })
        })
}

/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，
/// 不读分配记录、不读树表，按这条根现算它引用的全部落点：根记录直接指着的豁免单元（实例表链上认得出的每一片、
/// 树表、中央映射树的每个节点）加中央映射树里每一条映射条目的落点（数据单元、打包单元、四棵树的节点，含分配记录树自己的节点）。
/// 树表自己的落点取根记录里那条指针。只在内存里用，不写盘。中央映射树读不出、解不开（映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内），
/// 映射条目解不开、落点不同槽或不在单元区：交 `None`。
pub(super) fn placements_referenced_by_root_computed_from_the_mapping<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let mapping_tree = crate::code_two_tree::read_code_two_tree(
        &root.mapping_root,
        &crate::transaction::MultiLevelCodeTwoTree::CentralMapping
            .read_expectation(root.mapping_root.head.birth_tree),
        crate::code_two_tree::CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
        root,
        crate::unit::unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &crate::pointer::NodePointer| {
            read_unit_via_locations(devices, &pointer.locations, node_bytes)
        },
    )
    .ok()?;
    let mut located: Vec<([LocationEntry; 2], u64)> = Vec::new();
    for page in instance_table_page_pointers_as_far_as_readable(devices, root) {
        located.push((page.locations, TransactionUnit::InstanceTable.span_slots()));
    }
    located.push((
        root.tree_table.locations,
        TransactionUnit::TreeTable.span_slots(),
    ));
    for pointer in &mapping_tree.version.pointers {
        located.push((pointer.locations, TransactionUnit::MappingTree.span_slots()));
    }
    for entry in &mapping_tree.leaf_entries_in_key_order {
        let (key, locations) = crate::records::parse_mapping_entry(entry)?;
        let span = match key.first().copied()? {
            crate::unit::UNIT_CLASS_DATA | crate::unit::UNIT_CLASS_PACKED => 2,
            crate::unit::UNIT_CLASS_INDEX_NODE => 1,
            _ => return None,
        };
        located.push((locations, span));
    }
    let mut placements = Vec::new();
    let identities = devices.device_identities();
    for (locations, span) in located {
        let slot = slot_shared_by_both_location_entries(&locations).ok()?;
        for identity in &identities {
            placement_lies_in_the_unit_area_of_its_device(
                devices,
                unit_area_start,
                *identity,
                slot,
                span,
            )
            .ok()?;
            placements.push((*identity, slot, span));
        }
    }
    Some(placements)
}

/// 一条根引用着的落点，逐盘：带文件的一版从它那一版的分配记录里取（账里已释放的是它换下的上一版的，不算它引用）；
/// **树表 0 条的一版**引用的就是根记录直接指着的那几个单元——实例表（连同链上后面各片）、树表，写过行的一版还有它自己那片
/// 分配记录树节点（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））——两盘同槽，不读分配记录树：
/// 那一片读不出时它自己的落点照样认得出。读不出、解不开交回 `None`，怎么办归调用方：影子账读被抛弃根计一次读不出、从中央映射现算，
/// 仍不成重读一次，再不成看 B 的凭据放行或拒（[`placements_of_an_abandoned_root_whose_account_is_unreadable`]）；候选根读不出就当它什么都不豁免；
/// C 的账那一份读不出就不加（[`placements_in_the_accounts_of_unapplied_publishes`]）；
/// 解不开包括根记录里那几条指针的两条位置条目不同槽、槽号与跨度不在那块盘的单元区里（代码审阅第 32 条）。
///
/// 树表 0 条那一臂不能省：写过行的树表 0 条根指着的是一片新实例表，它既不在任何分配记录树里、也不在 mkfs 那两个单元里，
/// 少了这一臂它在影子账里就成了「账读不出」的被抛弃根，那片实例表不被隔离、也不在重建出来的账里，
/// 挂载之后当空闲槽发出去，把根环里还引用着它的那条根读坏（2026-09-23 崩溃注入打中：那时的挂载时回退里
/// I-2.1 / I-4.8 / I-7.4 三条红在「最新根引用的单元已被复用」；崩溃恢复落到树表 0 条的根时是同一格）。
pub(super) fn placements_referenced_by_root<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
    match tree_table_has_no_entries(devices, root) {
        Ok(true) => {
            let mut placements = Vec::new();
            // 实例表是一条链（D18（块里携带什么信息） 已定项 11）：认得出的每一片都是这条根引用的，只有一片时就是根记录里那一条。
            // 读不全的链照样把认得出的那几片算上——少算一片，被抛弃根的那一片就不隔离、挂载之后可能被发出去。
            let instance_table_pages =
                instance_table_page_pointers_as_far_as_readable(devices, root);
            // 写过行的一版那棵分配记录树的节点同样是这条根引用的单元（D23（journal 的角色与格式） 已定项 14：被抛弃时间线的根离开根环之前，
            // 它们引用的单元不许重新分配）；漏了它们，被抛弃根的这几片既不隔离、也不在所选那一版的账里，挂载之后是空闲槽。
            // 树可以多层（D8（核心索引结构） 已定项 14）：读得出多少认多少，读不出的节点它自己那一片照样认（父条目里有它的指针）。
            // mkfs 的第 0 代与照抄它的暖机根那一项全零，没有这棵树。
            let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
            let allocation_record_tree_nodes: Vec<crate::pointer::NodePointer> =
                if root.allocation_record_tree_root == crate::pointer::NodePointer::empty_root() {
                    Vec::new()
                } else {
                    node_pointers_as_far_as_readable(
                        &root.allocation_record_tree_root,
                        &AllocationRecordTreeGeometry::of_reader(devices),
                        &mut |pointer: &crate::pointer::NodePointer| {
                            read_unit_via_locations(devices, &pointer.locations, node_bytes).ok()
                        },
                    )
                };
            for (pointer, unit) in instance_table_pages
                .iter()
                .map(|page_pointer| (page_pointer, TransactionUnit::InstanceTable))
                .chain([(&root.tree_table, TransactionUnit::TreeTable)])
                .chain(
                    allocation_record_tree_nodes
                        .iter()
                        .map(|pointer| (pointer, TransactionUnit::AllocationTree)),
                )
            {
                let slot = slot_shared_by_both_location_entries(&pointer.locations).ok()?;
                for identity in &devices.device_identities() {
                    // 槽号是盘上读来的 6 字节：进分配器（`PoolAllocator::isolate_abandoned` 与根环表里清隔离那一步）之前判它在这块盘的单元区里，
                    // 不在就与两条位置条目不同槽同一个结局——这条根的账解不开，调用方只计数（代码审阅第 32 条；改之前是
                    // `DeviceFreeMap::index` 的 expect 与 `isolate` 的跨度断言 panic）。
                    placement_lies_in_the_unit_area_of_its_device(
                        devices,
                        unit_area_start,
                        *identity,
                        slot,
                        unit.span_slots(),
                    )
                    .ok()?;
                    placements.push((*identity, slot, unit.span_slots()));
                }
            }
            Some(placements)
        }
        Ok(false) => Some(
            allocation_records_under_root_in_the_unit_area_starting_at(
                devices,
                unit_area_start,
                root,
            )
            .ok()?
            .into_iter()
            .filter(|record| !record.is_released)
            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
            .collect(),
        ),
        Err(_failure) => None,
    }
}

/// 影子账按槽认豁免与判重（D23（journal 的角色与格式） 已定项 14「只隔离其中只被被抛弃根引用的槽」，判决
/// `research/prompts/m3-c554-c572-r1-main-verification.md` 第二节 B3 那一行）：攻方 `research/prompts/m3-c554-c572-r1-opus-output.md`
/// 「B3」一节点名的五处各一条会红的用例，前三种形照攻方原样（同一起点、跨度 1 与跨度 2 的两份落点），候选根、现行账两份豁免集合另各一条
/// （跨度 2 的豁免落点罩住被抛弃根落点的后一槽）。直调被测函数；两块盘照 mkfs 写成，候选根与「从盘上读账」的被抛弃根读的是 mkfs 写下的那一版，
/// 其余几条的落点都在内存里给（根环表里记着的、根槽读坏记着的、水位之上没施加的记录点名的），不读盘。
#[cfg(test)]
pub(super) mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests {
    use super::{
        isolate_slots_referenced_only_by_abandoned_roots, BeforeTheOneReread,
        PlacementsOfAbandonedRoots,
    };
    use crate::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    };
    use crate::allocator::{
        AllocationRecord, DeviceFreeMap, PlacementOnDevice, PoolAllocator, UnitAreaStart,
    };
    use crate::block_device::{
        BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
    };
    use crate::make_filesystem::{make_filesystem, MakeFilesystemParameters};
    use crate::root_record::RootRecord;
    use crate::root_ring::RootRingSlotsPerRegion;
    use crate::system_configuration::SystemImmutableSizes;
    use std::collections::BTreeMap;

    /// 每块盘的字节数：与 验证包建池同一个尺寸（4 GiB，journal 环取默认的 768 MiB），只在内存里按页存写过的字节。
    const DEVICE_BYTES: u64 = 4 << 30;
    /// 稀疏盘一页的字节数。
    const PAGE_BYTES: u64 = 4096;
    /// 现行账、被抛弃根、水位之上的记录在这几条用例里引用的起点：单元区起点之后第 10 槽，mkfs 写下的实例表（起点、两槽）与树表（起点 + 2、一槽）之外。
    const START_SLOT_AFTER_THE_UNIT_AREA_START: u64 = 10;
    /// 被抛弃根的那一版、现行账记录的分配代：这几条用例只看豁免与判重，代取多少都一样。
    const ALLOCATION_GENERATION: CheckpointTxg = CheckpointTxg(5);

    /// 只在内存里的盘：写过的页才存，没写过的读回全零；越过末尾报越界。同文件里经真发布搭池的那几组用例
    /// （[`super::pool_in_memory_after_the_first_file`]）也用它。
    pub(in crate::mount) struct SparseDeviceInMemory {
        pages: BTreeMap<u64, Vec<u8>>,
    }

    impl SparseDeviceInMemory {
        /// 一页都没写过的盘：读哪里都是全零。
        pub(in crate::mount) fn blank() -> Self {
            Self {
                pages: BTreeMap::new(),
            }
        }

        fn refuse_past_the_end(
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            if offset.0 + length > DEVICE_BYTES {
                return Err(BlockDeviceError::OutOfRange {
                    offset,
                    length,
                    device_size: DEVICE_BYTES,
                });
            }
            Ok(())
        }
    }

    impl BlockDevice for SparseDeviceInMemory {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let length = u64::try_from(buffer.len()).expect("长度装得进 u64");
            Self::refuse_past_the_end(offset, length)?;
            for (index_in_buffer, byte) in buffer.iter_mut().enumerate() {
                let address = offset.0 + u64::try_from(index_in_buffer).expect("下标装得进 u64");
                let index_in_page =
                    usize::try_from(address % PAGE_BYTES).expect("页内偏移装得进 usize");
                *byte = self
                    .pages
                    .get(&(address / PAGE_BYTES))
                    .map_or(0, |page| page[index_in_page]);
            }
            Ok(())
        }
        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let length = u64::try_from(bytes.len()).expect("长度装得进 u64");
            Self::refuse_past_the_end(offset, length)?;
            for (index_in_bytes, byte) in bytes.iter().enumerate() {
                let address = offset.0 + u64::try_from(index_in_bytes).expect("下标装得进 u64");
                let index_in_page =
                    usize::try_from(address % PAGE_BYTES).expect("页内偏移装得进 usize");
                self.pages
                    .entry(address / PAGE_BYTES)
                    .or_insert_with(|| vec![0; usize::try_from(PAGE_BYTES).expect("4096")])
                    [index_in_page] = *byte;
            }
            Ok(())
        }
        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            Self::refuse_past_the_end(offset, length)?;
            assert!(
                offset.0.is_multiple_of(PAGE_BYTES) && length.is_multiple_of(PAGE_BYTES),
                "mkfs 清零的段按 4096 对齐：系统配置槽距、根环区域与 journal 环都是 4096 的整数倍"
            );
            let first_page = offset.0 / PAGE_BYTES;
            let pages_zeroed: Vec<u64> = self
                .pages
                .range(first_page..first_page + length / PAGE_BYTES)
                .map(|(page, _)| *page)
                .collect();
            for page in pages_zeroed {
                self.pages.remove(&page);
            }
            Ok(())
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            PhysicalBlockSizeInBytes(512)
        }
        fn size_in_bytes(&self) -> u64 {
            DEVICE_BYTES
        }
    }

    /// mkfs 写成的两块盘、它的单元区起点与它写下的第 0 代根。
    struct FormattedPool {
        devices: Vec<(DeviceIdentity, SparseDeviceInMemory)>,
        unit_area_start: UnitAreaStart,
        make_filesystem_root: RootRecord,
    }

    fn formatted_pool() -> FormattedPool {
        let parameters = MakeFilesystemParameters {
            filesystem_identifier: [7; 16],
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: SystemImmutableSizes {
                physical_block_size: 512,
                minimum_input_output_bytes: 512,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
        };
        let mut devices: Vec<(DeviceIdentity, SparseDeviceInMemory)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        SparseDeviceInMemory {
                            pages: BTreeMap::new(),
                        },
                    )
                })
                .collect();
        let output =
            make_filesystem(&parameters, &mut devices).expect("4 GiB 两盘照默认几何建得成");
        FormattedPool {
            devices,
            unit_area_start: output.unit_area_start,
            make_filesystem_root: output.root,
        }
    }

    fn slot_after_the_unit_area_start(pool: &FormattedPool, slots_after: u64) -> SlotNumber {
        SlotNumber(pool.unit_area_start.slot().0 + slots_after)
    }

    fn placement_on_device_zero(slot: SlotNumber, span: u64) -> PlacementOnDevice {
        PlacementOnDevice {
            device: DeviceIdentity(0),
            slot,
            span,
        }
    }

    fn allocated_record_on_device_zero(slot: SlotNumber, span_slots: u16) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(0),
            slot,
            span_slots,
            generation: ALLOCATION_GENERATION,
            is_released: false,
            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    /// 一条被抛弃根：实例代号与 txg 各取一个 mkfs 没写过的，别的字段照 mkfs 的第 0 代根（它的落点从根环表里记着的那一份取，不读这些指针）。
    fn abandoned_root(pool: &FormattedPool, instance: u32, checkpoint_txg: u64) -> RootRecord {
        let make_filesystem_root = pool.make_filesystem_root;
        RootRecord {
            filesystem_identifier: make_filesystem_root.filesystem_identifier,
            unmount_marker: make_filesystem_root.unmount_marker,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            tree_table: make_filesystem_root.tree_table,
            tree_identifier_watermark: make_filesystem_root.tree_identifier_watermark,
            rollback_floor: make_filesystem_root.rollback_floor,
            instance_table: make_filesystem_root.instance_table,
            mapping_root: make_filesystem_root.mapping_root,
            allocation_record_tree_root: make_filesystem_root.allocation_record_tree_root,
            reclaim_threshold: crate::address::CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    /// 喂给被测函数的一次影子账：根环里的根（`abandoned_instances` 里实例代号的算被抛弃）、现行账、根环表里记着的被抛弃根的落点、
    /// 根槽这一遍读坏的被抛弃根记着的落点、水位之上没施加的记录点名的落点。
    struct ShadowLedgerInputs<'inputs> {
        roots: &'inputs [RootRecord],
        abandoned_instances: &'inputs [InstanceGeneration],
        current_records: &'inputs [AllocationRecord],
        recorded_placements_of_abandoned_roots:
            &'inputs BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad:
            &'inputs [PlacementOnDevice],
        placements_named_by_unapplied_journal_records: &'inputs [PlacementOnDevice],
    }

    /// 在一个只按现行账置好分配位的分配器上跑一次影子账（下界 F 取 0），交回跑完的分配器。
    fn allocator_after_the_shadow_ledger(
        pool: &FormattedPool,
        inputs: &ShadowLedgerInputs<'_>,
    ) -> PoolAllocator {
        let mut allocator = PoolAllocator::new(
            pool.devices
                .iter()
                .map(|(identity, _)| {
                    DeviceFreeMap::with_unit_area_start(
                        *identity,
                        DEVICE_BYTES,
                        pool.unit_area_start,
                    )
                })
                .collect(),
        );
        for record in inputs
            .current_records
            .iter()
            .filter(|record| !record.is_released)
        {
            allocator
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("用例里的记录都在盘 0 上")
                .mark_allocated(record.slot, u64::from(record.span_slots));
        }
        let is_abandoned = |root: &RootRecord| inputs.abandoned_instances.contains(&root.instance);
        isolate_slots_referenced_only_by_abandoned_roots(
            &pool.devices,
            pool.unit_area_start,
            &mut allocator,
            inputs.roots,
            &is_abandoned,
            CheckpointTxg(0),
            inputs.current_records,
            inputs.placements_named_by_unapplied_journal_records,
            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                inputs.recorded_placements_of_abandoned_roots,
            ),
            &mut BeforeTheOneReread::RereadImmediately,
            inputs.placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
        )
        .expect("mkfs 写下的那一版读得出、别的落点都在内存里：不拒");
        allocator
    }

    /// 盘 0 上 `slot` 那一槽：(隔离着, 发得出去)。
    fn isolated_and_free_on_device_zero(
        allocator: &PoolAllocator,
        slot: SlotNumber,
    ) -> (bool, bool) {
        let device_map = &allocator.devices[0];
        (device_map.is_isolated(slot), device_map.is_free(slot))
    }

    /// 攻方形一：两条被抛弃根同一起点 s，A1 引用 (s, 跨 1)、A2 引用 (s, 跨 2)。按起点槽判重时窄的先读出就把宽的整份判重掉、s+1 发得出去，
    /// 宽的先读出两槽都罩着——结局随根环读出次序变。按槽判重两种次序结局相同：两槽都隔离着。
    #[test]
    fn two_abandoned_roots_sharing_a_start_slot_isolate_both_slots_of_the_wider_placement_in_either_read_order(
    ) {
        let pool = formatted_pool();
        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
        let narrow_root = abandoned_root(&pool, 1, 8);
        let wide_root = abandoned_root(&pool, 2, 12);
        let recorded = BTreeMap::from([
            (
                (narrow_root.instance, narrow_root.checkpoint_txg),
                vec![placement_on_device_zero(start, 1)],
            ),
            (
                (wide_root.instance, wide_root.checkpoint_txg),
                vec![placement_on_device_zero(start, 2)],
            ),
        ]);
        let outcome_in_order = |roots: &[RootRecord]| {
            let allocator = allocator_after_the_shadow_ledger(
                &pool,
                &ShadowLedgerInputs {
                    roots,
                    abandoned_instances: &[narrow_root.instance, wide_root.instance],
                    current_records: &[],
                    recorded_placements_of_abandoned_roots: &recorded,
                    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                    placements_named_by_unapplied_journal_records: &[],
                },
            );
            [
                isolated_and_free_on_device_zero(&allocator, start),
                isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
            ]
        };
        let narrow_first = outcome_in_order(&[narrow_root, wide_root]);
        let wide_first = outcome_in_order(&[wide_root, narrow_root]);
        assert_eq!(
            narrow_first,
            [(true, false), (true, false)],
            "窄的先读出：s 与 s+1 都隔离着、都发不出去（按起点槽判重时 s+1 发得出去）"
        );
        assert_eq!(wide_first, narrow_first, "宽的先读出与窄的先读出结局相同");
    }

    /// 攻方形二：现行账里 (s, 跨 1) 仍分配，被抛弃根引用 (s, 跨 2)。s 被现行账引用、不隔离（分配着、发不出去）；s+1 只被被抛弃根引用，隔离。
    /// 按起点槽豁免时整份跳过，s+1 发得出去。
    #[test]
    fn current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_an_abandoned_roots_wider_placement_isolated(
    ) {
        let pool = formatted_pool();
        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
        let root = abandoned_root(&pool, 1, 8);
        let recorded = BTreeMap::from([(
            (root.instance, root.checkpoint_txg),
            vec![placement_on_device_zero(start, 2)],
        )]);
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[root],
                abandoned_instances: &[root.instance],
                current_records: &[allocated_record_on_device_zero(start, 1)],
                recorded_placements_of_abandoned_roots: &recorded,
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                placements_named_by_unapplied_journal_records: &[],
            },
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, start),
            (false, false),
            "s 被现行账引用：不隔离、分配着"
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
            (true, false),
            "s+1 只被被抛弃根引用：隔离、发不出去"
        );
    }

    /// 攻方形三：水位之上没施加的记录点名 (s, 跨 2)，现行账里 (s, 跨 1) 仍分配。C 那一圈只豁免现行账引用的槽：s 不隔离，s+1 隔离。
    #[test]
    fn current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_a_placement_named_by_unapplied_records_isolated(
    ) {
        let pool = formatted_pool();
        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[],
                abandoned_instances: &[],
                current_records: &[allocated_record_on_device_zero(start, 1)],
                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(
                    start, 2,
                )],
            },
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, start),
            (false, false),
            "s 被现行账引用：不隔离、分配着"
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
            (true, false),
            "s+1 只被水位之上没施加的记录点名：隔离、发不出去"
        );
    }

    /// 根槽这一遍读坏的被抛弃根记着 (s, 跨 2)，现行账里 (s, 跨 1) 仍分配（攻方点名的第四处，形二搬到这一圈）：s 不隔离，s+1 隔离。
    #[test]
    fn current_account_record_at_the_same_start_slot_leaves_the_second_slot_recorded_for_an_abandoned_root_whose_ring_slot_read_bad_isolated(
    ) {
        let pool = formatted_pool();
        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[],
                abandoned_instances: &[],
                current_records: &[allocated_record_on_device_zero(start, 1)],
                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[
                    placement_on_device_zero(start, 2),
                ],
                placements_named_by_unapplied_journal_records: &[],
            },
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, start),
            (false, false),
            "s 被现行账引用：不隔离、分配着"
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
            (true, false),
            "s+1 只被那条被抛弃根引用：隔离、发不出去"
        );
    }

    /// 现行账那一份豁免集合按槽收（攻方点名的第一处）：现行账里 (起点 + 1, 跨 2) 仍分配，被抛弃根是从盘上读账的 mkfs 那一版
    /// （实例表 (起点, 跨 2)、树表 (起点 + 2, 跨 1)，两块盘各一份；根环表里没记着它，整段隔离、分配位置着的槽也隔离）。
    /// 盘 0 上起点只被被抛弃根引用、隔离；起点 + 1 与起点 + 2 被现行账那条记录罩着、不隔离。只按记录的起点收时起点 + 2 不在豁免里、被隔离。
    #[test]
    fn current_account_record_exempts_every_slot_it_covers_from_the_placements_of_an_abandoned_root_read_from_its_account(
    ) {
        let pool = formatted_pool();
        let unit_area_start = pool.unit_area_start.slot();
        let root = abandoned_root(&pool, 1, 8);
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[root],
                abandoned_instances: &[root.instance],
                current_records: &[allocated_record_on_device_zero(
                    SlotNumber(unit_area_start.0 + 1),
                    2,
                )],
                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                placements_named_by_unapplied_journal_records: &[],
            },
        );
        let isolated_on_device_zero: Vec<bool> = (0..3)
            .map(|slots_after| {
                allocator.devices[0].is_isolated(SlotNumber(unit_area_start.0 + slots_after))
            })
            .collect();
        assert_eq!(
            isolated_on_device_zero,
            vec![true, false, false],
            "盘 0：起点只被被抛弃根引用、隔离；起点 + 1、起点 + 2 被现行账 (起点 + 1, 跨 2) 罩着、不隔离"
        );
    }

    /// 候选根那一份豁免集合按槽收（攻方点名的第二处）：mkfs 那一版是候选根（按实例表有效、txg 0 ≥ F = 0），它引用实例表 (起点, 跨 2)；
    /// 一条被抛弃根引用 (起点 + 1, 跨 1)。起点 + 1 被候选根罩着、不隔离；只按候选根落点的起点收时它不在豁免里、被隔离。
    #[test]
    fn candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement(
    ) {
        let pool = formatted_pool();
        let second_slot_of_the_instance_table = slot_after_the_unit_area_start(&pool, 1);
        let root = abandoned_root(&pool, 1, 8);
        let recorded = BTreeMap::from([(
            (root.instance, root.checkpoint_txg),
            vec![placement_on_device_zero(
                second_slot_of_the_instance_table,
                1,
            )],
        )]);
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[pool.make_filesystem_root, root],
                abandoned_instances: &[root.instance],
                current_records: &[],
                recorded_placements_of_abandoned_roots: &recorded,
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                placements_named_by_unapplied_journal_records: &[],
            },
        );
        assert!(
            !allocator.devices[0].is_isolated(second_slot_of_the_instance_table),
            "起点 + 1 被候选根（mkfs 那一版）的实例表 (起点, 跨 2) 罩着：不隔离"
        );
    }

    /// 先看豁免、再判重（判决 `research/prompts/code-backlog-b2-code-r1-main-verification.md` 第三节 A1）：起点 s 同时被候选根（mkfs 那一版的
    /// 实例表 (s, 跨 2)）、一条根环表里记着的被抛弃根 (s, 跨 1) 与水位之上没施加的记录 (s, 跨 1) 引用，现行账里没有它。被抛弃根那一圈豁免它
    /// （候选根引用），没施加的记录那一圈只豁免现行账，要隔离它。先判重后看豁免时，被抛弃根那一圈把 s 记进判重集合，后一圈判重跳过，s 发得出去。
    #[test]
    fn slot_an_abandoned_roots_pass_exempted_for_a_candidate_is_still_isolated_for_the_unapplied_journal_records(
    ) {
        let pool = formatted_pool();
        let start = pool.unit_area_start.slot();
        let root = abandoned_root(&pool, 1, 8);
        let recorded = BTreeMap::from([(
            (root.instance, root.checkpoint_txg),
            vec![placement_on_device_zero(start, 1)],
        )]);
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[pool.make_filesystem_root, root],
                abandoned_instances: &[root.instance],
                current_records: &[],
                recorded_placements_of_abandoned_roots: &recorded,
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(
                    start, 1,
                )],
            },
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, start),
            (true, false),
            "s 只被候选根豁免给被抛弃根那一圈，没施加的记录那一圈照隔离：隔离着、发不出去"
        );
    }

    /// 同上，豁免 s 的换成根槽这一遍读坏的被抛弃根那一圈（记着 (s, 跨 1)）。
    #[test]
    fn slot_the_pass_for_abandoned_roots_whose_ring_slot_read_bad_exempted_is_still_isolated_for_the_unapplied_journal_records(
    ) {
        let pool = formatted_pool();
        let start = pool.unit_area_start.slot();
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[pool.make_filesystem_root],
                abandoned_instances: &[],
                current_records: &[],
                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[
                    placement_on_device_zero(start, 1),
                ],
                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(
                    start, 1,
                )],
            },
        );
        assert_eq!(
            isolated_and_free_on_device_zero(&allocator, start),
            (true, false),
            "s 只被候选根豁免给根槽读坏的被抛弃根那一圈，没施加的记录那一圈照隔离：隔离着、发不出去"
        );
    }
}
