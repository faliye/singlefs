//! 抬 F 那一串（准入与卸载共用，[`raise_the_floor_through`]）：重算影子账与回收门槛读根环、这一串空发布的 txg 与计划、
//! 回收与在分配器拷贝上的预演。

use super::*;

/// 抬 F 那一遍重算影子账与回收门槛读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：每个槽读一次；读坏的槽分三类——
/// 这个进程不知道住着根的当没有根；知道住着根的重读一次，仍坏就拒（C554 乙报告 Q6）；重建分配器两遍里只有一遍自证过的
/// （[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]，它们也在「知道住着根的槽」里，这里先按它们分）重读一次，
/// 读出来照读出来的算，仍坏不拒、拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲：有效的照它本身进回收门槛与候选集，
/// 被抛弃的照它引用的落点进影子账；有效还是被抛弃由调用方按同一张实例表判）。根环表里记成被抛弃根的槽（它们也在「知道住着根的槽」里，
/// 这里同样先按它们分，[`ring_slots_recorded_as_holding_an_abandoned_root_by`]）这一遍读坏不拒、不重读：读不出它的根记录，交回它挂载时
/// 记在根环表里的落点，由影子账照它补隔离、不进回收门槛（用户 2026-10-08 定 H1「被抛弃根的槽不拒」）。重读读得出也是同一条被抛弃根、
/// 影子账同样取表里记着的那一份（[`PlacementsOfAbandonedRoots`]），所以不重读结局相同。交回读得出（或顶上）的每一条根（重读读出来的与
/// 顶上的排在后面）与那几槽记着的落点。只读盘。
///
/// # Errors
/// 知道住着根的槽（两遍里只有一遍自证过的、记成被抛弃根的除外）重读仍坏 ⇒
/// `NewerStateStillUnreadableAfterOneReread(StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot(..))`。
fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    allocator: &PoolAllocator,
) -> Result<RootsReadForRaisingTheFloor, MountError> {
    let ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass =
        ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator);
    let ring_slots_recorded_as_holding_an_abandoned_root =
        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
    let ring_slots_whose_reread_still_bad_refuses_the_raise: BTreeSet<RootRingSlot> =
        ring_slots_known_to_hold_a_root_by(allocator)
            .into_iter()
            .filter(|ring_slot| {
                !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
                    .contains_key(ring_slot)
            })
            .filter(|ring_slot| {
                !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
            })
            .collect();
    let mut roots_with_ring_slots =
        readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
            reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            &ring_slots_whose_reread_still_bad_refuses_the_raise,
        )
        .map_err(|still_unreadable| {
            MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
        })?;
    let ring_slots_read: BTreeSet<RootRingSlot> = roots_with_ring_slots
        .iter()
        .map(|(ring_slot, _)| *ring_slot)
        .collect();
    // 重读那一读绕过这次挂载记下的那一份、再读一次盘（`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
    // 迭代上界是两遍里只有一遍自证过的槽数（至多根环的槽数）；跨轮只往交回的根里加；没有提前出口。上面那一遍读坏、当没有根的才在这里重读一次。
    for (ring_slot, root_record_self_verified_on_one_rebuild_pass) in
        &ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
    {
        if ring_slots_read.contains(ring_slot) {
            continue;
        }
        match read_root_ring_slot(
            &rereading_reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            *ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => {
                roots_with_ring_slots.push((*ring_slot, root))
            }
            RootRingSlotReading::Bad(_still_bad) => roots_with_ring_slots
                .push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),
        }
    }
    let placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: Vec<PlacementOnDevice> =
        ring_slots_recorded_as_holding_an_abandoned_root
            .into_iter()
            .filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))
            .flat_map(|(_, recorded_placements)| recorded_placements)
            .collect();
    Ok(RootsReadForRaisingTheFloor {
        roots_with_ring_slots,
        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
    })
}

/// 抬 F 那一遍读根环读出来的（[`readable_roots_for_raising_the_floor`]）。
struct RootsReadForRaisingTheFloor {
    /// 读得出、自证过的每一条根，与两遍里只有一遍自证过、这一遍重读仍坏的槽记着的那条根，各带读出它（或顶上它）的那一槽：
    /// 认盘、认「这一遍在哪几槽读出了根」都按这一槽，不按落点公式从 txg 推（第 0 代根三份 txg 都是 0，各在各的区域的槽 0；
    /// 判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 3 问，与准入算上限那一处同一个口径）。
    roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)>,
    /// 根环表里记成被抛弃根、这一遍读坏的槽各记着的落点（摊平）：影子账照它补隔离（H1）。
    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: Vec<PlacementOnDevice>,
}

/// 抬 F 那一遍的回收门槛里「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall：不按一圈窗口滤）：这一遍读出来的根里
/// 按 `table` 有效的，并上分配器根环表里挂载那一刻读坏或读回全零、这一遍也没在那一槽读出根的槽（各钉在它可能住过的最旧 txg，用户 2026-10-04 定，
/// 读回全零的 2026-10-05 定乙；
/// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一），取最小；最小的是一个读坏的槽时照 E2
/// 先重读那一槽（[`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`]，与进程内记根同一取法）。重建分配器两遍里只有一遍自证过、
/// 这一遍重读一次仍坏的槽不另钉：顶上的那条根已在 `roots_with_ring_slots_read_from_the_ring` 里（用户 2026-10-06 定 T1 甲）。
/// 「这一遍在那一槽读出了根」按读出它的那一槽认（[`ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass`]）。只读盘，不改分配器。
fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    allocator: &PoolAllocator,
    roots_with_ring_slots_read_from_the_ring: &[(RootRingSlot, RootRecord)],
    table: &InstanceTableRecords,
) -> Option<CheckpointTxg> {
    let valid_root_txgs: Vec<CheckpointTxg> = roots_with_ring_slots_read_from_the_ring
        .iter()
        .filter(|(_, root)| !abandoned_by_table(root, table))
        .map(|(_, root)| root.checkpoint_txg)
        .collect();
    let bad_at_mount = ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass(
        allocator.root_ring_occupancy().map_or_else(
            BTreeMap::new,
            RootRingOccupancy::ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held,
        ),
        roots_with_ring_slots_read_from_the_ring,
    );
    oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
        &valid_root_txgs,
        bad_at_mount,
        &mut |ring_slot| {
            reread_of_a_root_ring_slot_bad_at_mount(
                reader,
                parameters_of_the_pool,
                table,
                ring_slot,
            )
        },
    )
    .0
}

/// 根环表里挂载那一刻读坏或读回全零而钉着的槽（`pinned_since_the_mount`，各带钉值）里，抬 F 那一遍没在那一槽读出根的那几个
/// （[`oldest_valid_root_in_the_ring_for_raising_the_floor`] 照 E2 并进环里最旧有效根的就是它们）。「在那一槽读出了根」按读出它
/// （或顶上它）的那一槽认，不按落点公式从 txg 推：mkfs 在三个区域的槽 0 各种一份第 0 代根，txg 都是 0，公式把三份都推到区域 0 的槽 0——
/// 区域 1、2 的槽 0 挂载时读坏、这一遍读出了第 0 代根，按公式认会当没读出、照旧钉着；区域 0 的槽 0 挂载时读坏、这一遍又读坏，
/// 按公式认会因为别的区域读出第 0 代根而从钉里拿掉（判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 3 问，
/// 与准入算上限那一处同一个口径）。别的根都按公式写进槽，两种认法只在第 0 代根上不同。
fn ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass(
    pinned_since_the_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
    roots_with_ring_slots_read_on_this_pass: &[(RootRingSlot, RootRecord)],
) -> BTreeMap<RootRingSlot, CheckpointTxg> {
    let ring_slots_read_from_the_ring: BTreeSet<RootRingSlot> =
        roots_with_ring_slots_read_on_this_pass
            .iter()
            .map(|(ring_slot_the_root_was_read_from, _)| *ring_slot_the_root_was_read_from)
            .collect();
    pinned_since_the_mount
        .into_iter()
        .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))
        .collect()
}

/// 抬 F 那一遍读出的每一条根与它归的那块盘：按读出它（或顶上它）的那一槽所在区域的 `region_devices` 认盘，不按落点公式从 txg 推
/// （第 0 代根三份各归各的区域的盘；与重建分配器那一格的 [`roots_on_the_devices_they_were_read_from`]、准入算上限那一处同一个口径，
/// 判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 3 问）。卸载那一格当场放开到哪儿
/// （[`newest_valid_root_txg_on_every_device`]）按它取每块盘上最新的有效根。
fn roots_on_the_devices_of_their_ring_slots(
    roots_with_ring_slots: &[(RootRingSlot, RootRecord)],
    region_devices: &[DeviceIdentity; 3],
) -> Vec<(DeviceIdentity, RootRecord)> {
    roots_with_ring_slots
        .iter()
        .map(|(ring_slot_the_root_was_read_from, root)| {
            (
                region_devices[usize::try_from(ring_slot_the_root_was_read_from.region)
                    .expect("区域号小于根环区域数 3")],
                *root,
            )
        })
        .collect()
}

/// 抬 F 那一串（准入与卸载共用，D16（发布语义） 已定项 1「抬 F 那一串」）。两个入口只差 [`RaiseFloorEntry`] 那三样：
/// 判不判上限、根带不带卸载记号、回收出来的槽什么时候能发出去（[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）。
pub(super) fn raise_the_floor_through<Device: BlockDevice>(
    entry: RaiseFloorEntry,
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<FloorRaisedThroughTheSequence, MountError> {
    // 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：这一串的第一次空发布本来就会被拒，
    // 而下面的影子账与回收在发布之前就动分配器——重发成功时分配器整个换成那次发布之后的一份，这些改动会被一起丢掉。第一道就拒，一样都不动。
    if let Some(frozen) = allocator.frozen_publish() {
        return Err(MountError::RaiseFloorSequencePublishFailed(Box::new(
            PublishSequenceFailed {
                cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
                writes_of_persisted_publishes: Vec::new(),
                writes_of_failed_publishes: Vec::new(),
            },
        )));
    }
    // 下面的影子账重算与回收在第一次空发布之前就动分配器：第一次空发布在任何写之前被拒（落点、准入、释放判定），这一串一次都没落盘、
    // F 没有一条根带出去，分配器整个换回这一份——扣住的槽放开、回收的回到 defer、补的隔离撤掉（C546（抬 F 被拒时扣住的槽不退回））。
    let allocator_before_the_raise = allocator.clone();
    // 调用方交进来的参数与盘表照可写挂载同一套核，写入口只按盘上那一份建：对不上在任何写之前拒，分配器还没动。
    let CallerInputsAgreeingWithTheDisk {
        system_configuration,
        parameters_of_the_pool,
    } = caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current)?;
    // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
    // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
    // 第一个文件版本那一版起就不带实例表单元，mkfs 同一个进程里后面发多少次都一样，而表就在根指针后面、读得出也解得开。
    // 经数只坏一份的那一层读（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：下一次用户改动发成之后另发的那一批换下它。
    // 下面拒了分配器换回去，账不退（读是真发到盘上的，账各份拷贝共用）。
    let devices_counting_unit_reads_with_one_copy_failing =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
    let instance_table_chain_read = instance_table_chain_of_root(
        &devices_counting_unit_reads_with_one_copy_failing,
        &current.root,
    );
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        devices_counting_unit_reads_with_one_copy_failing
            .locations_of_the_units_read_with_one_copy_failing(),
    );
    let table = instance_table_chain_read
        .map_err(|failure| instance_table_refusal(&failure))?
        .records;
    // F 的生效值只增不减（「抬 F 那一串」那一行）：新 F 低于盘上现算的生效值，往下「抬」条款没写，在任何写之前拒。
    // 先写系统配置那一步写之前按盘上现状再算一次、取两者的大者（`transaction::RollbackFloorOfASystemConfigurationWrite::RaisedFloor`）。
    // 判「有效根」的那张表（根环里最新那条根的实例表）读不出就重读一次，仍读不出就拒这一次抬，不按「不按表滤」算（C554 乙报告 Q6）：
    // 在动分配器与任何写之前返回，分配器还没动。
    let effective_floor = allocator
        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&**devices, |reader| {
            effective_rollback_floor_rereading_the_newest_instance_table_once(
                reader,
                &system_configuration.immutable.region_devices,
                &system_configuration.immutable.sizes,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .map_err(|still_unreadable| {
            MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
        })?;
    if new_floor < effective_floor {
        return Err(
            MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                requested: new_floor,
                effective: effective_floor,
            },
        );
    }
    let ceiling_judged_by_the_admission = match entry {
        RaiseFloorEntry::Admission => {
            let ceiling = rollback_floor_ceilings_of_this_process(
                devices,
                &system_configuration,
                &current.root,
                &table,
                allocator,
            )?
            .not_passing_the_ring_slots_bad_since_the_mount;
            if new_floor > ceiling {
                return Err(MountError::RollbackFloorAboveCeiling {
                    requested: new_floor,
                    ceiling,
                });
            }
            Some(ceiling)
        }
        // 卸载抬到现行那一版的 txg，不另判上限（「抬 F 的上限」那一行：等价于上限取最新的持久有效根、不按盘取小）。
        RaiseFloorEntry::Unmount => None,
    };
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(
            txg,
            parameters_of_the_pool.geometry.root_ring_slots_per_region,
        );
        parameters_of_the_pool.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    // 这一串的 txg 在动分配器（影子账重算、回收）之前算：越过 u64::MAX 就拒（代码审阅第 36 条），分配器与盘都不动。
    let planned_txgs = txgs_of_the_publishes_carrying_the_floor_to_every_device(
        current.root.checkpoint_txg,
        &all_devices,
        &device_of_txg,
    )?;
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
    // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
    // 落满每块盘之后再放开——这是准入那一格；卸载那一格只扣释放代高于每块盘上最新有效根的那一截（用户 2026-10-03 定甲 GA，
    // `RaiseFloorEntry::reuse_of_the_reclaimed_slots`）。
    // 记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
    // 回收门槛与影子账读的这一遍根环，照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（与算上限那一遍同一个分法）：
    // 这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按「没有根」往下走，那一槽里的被抛弃根引用的槽
    // 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根；重建分配器两遍里只有一遍自证过的槽读坏也重读一次，仍坏不拒、
    // 拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲，[`readable_roots_for_raising_the_floor`]）：被抛弃的照它引用的落点
    // 进下面的影子账重算（不顶上时它引用、挂载那一刻被候选根豁免的槽按新 F 回收之后发得出去，判决第二节 K1，I-7.4 红），
    // 有效的照它本身进回收门槛与候选集。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
    // 根环表里记成被抛弃根的槽这一遍读坏不拒（用户 2026-10-08 定 H1）：交回它们记着的落点，下面的影子账照它补隔离。
    let RootsReadForRaisingTheFloor {
        roots_with_ring_slots,
        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
    } = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;
    let roots: Vec<RootRecord> = roots_with_ring_slots
        .iter()
        .map(|(_, root)| *root)
        .collect();
    // 根环表里挂载那一刻读坏、这一遍也没读出根的槽，钉在它可能住过的最旧 txg 并进环里最旧有效根（RR4；用户 2026-10-04 定钉在最旧 txg；与进程内记根同一取法，
    // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_ring_for_raising_the_floor`）。
    let oldest_valid_root_in_the_ring = oldest_valid_root_in_the_ring_for_raising_the_floor(
        &**devices,
        &parameters_of_the_pool,
        allocator,
        &roots_with_ring_slots,
        &table,
    );
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
    // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
    // 水位之上没施加的记录点名、还没解开的落点（C694 的 C）按新候选集同一次重算：此前靠一条低于新 F 的根豁免的那几个，回收之前补上隔离。
    let placements_named_by_unapplied_journal_records: Vec<PlacementOnDevice> = allocator
        .root_ring_occupancy()
        .map(RootRingOccupancy::placements_named_by_unapplied_journal_records)
        .unwrap_or_default();
    // 抬 F 那一串里的被抛弃根（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定，判决
    // `research/prompts/m3-batch10y-code-r1-main-verification.md` 第三节）：这一遍读出的、按现行那一版的实例表被抛弃的根，根环表里那一槽记着的
    // 是同一条（实例代号与 txg 都对上）就照挂载时算出、记着的落点进影子账，不再从盘上读它的账、读不出也不拒（不走 C694 的 B）。
    // 改之前照盘上重读，挂上之后它的账读坏就照 B 拒，正常卸载与会话写都被拒（攻方 `research/prompts/m3-batch10y-code-r1-opus-output.md` 「K4-2」）。
    // 表里没记着这条被抛弃根、挂载那一刻那一槽记成同一条有效根（写行之后才被抛弃）的，借 C 那一圈在那一槽记着的落点
    // （用户 2026-10-08 定 H2「用 C 那一圈记着的落点」，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节）：
    // 同样不读账、读不出不拒。两样都没记着的照旧读盘（[`PlacementsOfAbandonedRoots`] 那一臂的注释）。那条根挂载那一刻账读不出而记录齐时，
    // C 那一槽另带着挂载那一刻现行账里仍分配着的落点（丙，用户 2026-10-08 定，[`placements_in_the_accounts_of_newer_readable_roots`]）。
    // C 那一槽有条目而落点是空的（凭据放行那一形），照空串算、不读账（「照 C 那一圈记着的落点算」，与根环表里记着空串同一处置）。
    let placements_recorded_for_abandoned_roots: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<PlacementOnDevice>,
    > = allocator
        .root_ring_occupancy()
        .map(|occupancy| {
            roots
                .iter()
                .filter(|root| abandoned_by_table(root, &table))
                .filter_map(|root| {
                    occupancy
                        .placements_recorded_for_the_abandoned_root(
                            root.instance,
                            root.checkpoint_txg,
                        )
                        .or_else(|| {
                            occupancy
                                .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                                    root.instance,
                                    root.checkpoint_txg,
                                )
                        })
                        .map(|recorded| ((root.instance, root.checkpoint_txg), recorded))
                })
                .collect()
        })
        .unwrap_or_default();
    // 重算影子账读每条候选根与表里没记着的被抛弃根的账，现行那一版也在候选里（读的是它的分配记录树、树表）：读到只坏一份的进别的读那一份账
    // （D4（校验和位置） 已定项 4「可写挂载期间别的读」）。下面拒了分配器换回去，账不退（账各份拷贝共用）。
    let devices_counting_the_reads_of_the_accounts =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
        match isolate_slots_referenced_only_by_abandoned_roots(
            &devices_counting_the_reads_of_the_accounts,
            unit_area_start_of_the_chosen_system_configuration(&system_configuration),
            allocator,
            &roots,
            &|root| abandoned_by_table(root, &table),
            new_floor,
            &current.allocation_records,
            &placements_named_by_unapplied_journal_records,
            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                &placements_recorded_for_abandoned_roots,
            ),
            &mut BeforeTheOneReread::RereadImmediately,
            &placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
        ) {
            Ok(computed) => computed.abandoned_roots_unreadable,
            Err(error) => {
                *allocator = allocator_before_the_raise;
                allocator.count_the_other_unit_reads_with_one_copy_failing(
                    devices_counting_the_reads_of_the_accounts
                        .locations_of_the_units_read_with_one_copy_failing(),
                );
                return Err(error);
            }
        }
    } else {
        0
    };
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        devices_counting_the_reads_of_the_accounts
            .locations_of_the_units_read_with_one_copy_failing(),
    );
    // 卸载那一格当场放开的那一截（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`，GA）：下面的整串预演就在分配器的拷贝上算上它们；
    // 其余那一截与准入那一格一样扣到生效。分配器记着回收过的槽不重复回收，第二次只回收第一次没回收的那几个。
    let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);
    let reclaimed = match entry.reuse_of_the_reclaimed_slots() {
        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => {
            reclaim_released_in_the_sequence_raising_the_floor_up_to(
                allocator,
                shadow_ledger,
                floor_of_the_reclaim,
                ReclaimedReuse::HeldUntilFloorTakesEffect,
            )
        }
        ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice => {
            let roots_on_their_devices = roots_on_the_devices_of_their_ring_slots(
                &roots_with_ring_slots,
                &parameters_of_the_pool.region_devices,
            );
            let mut reclaimed = match newest_valid_root_txg_on_every_device(
                &roots_on_their_devices,
                &table,
                &all_devices,
            ) {
                Some(newest_valid_root_on_every_device) => {
                    reclaim_released_in_the_sequence_raising_the_floor_up_to(
                        allocator,
                        shadow_ledger,
                        floor_of_the_reclaim.min(newest_valid_root_on_every_device),
                        ReclaimedReuse::Immediately,
                    )
                }
                None => Vec::new(),
            };
            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(
                allocator,
                shadow_ledger,
                floor_of_the_reclaim,
                ReclaimedReuse::HeldUntilFloorTakesEffect,
            ));
            reclaimed
        }
    };
    let publishes_in_the_sequence = planned_txgs.len();
    let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
    // 这一串在任何写之前在分配器的一份拷贝上整串预演（与可写挂载取号之前那一串同一个做法，`dry_run_of_the_publishes_after_acquisition`）：
    // 第 n 次在拷贝上报错（取不到落点、别的落盘之前的错），这一串一次都不发——F 不带出去，分配器整个换回抬 F 之前那一份
    // （扣住的槽放开、回收的回到 defer、补的隔离撤掉，C546（抬 F 被拒时扣住的槽不退回））。不预演时前 n − 1 次已经落盘：
    // 盘上带着新 F 而 F 没在每块盘上生效，扣住的槽留在这个进程里、计数上是空闲而发不出去（实二五报告第六节 Q3），
    // 而回退留下空档时那条落了盘的新 F 让 checker 在合法状态上判 I-3.1 红（增补 2 收口表第 43 行那一形，
    // 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」的两个种子）。
    let placements_rehearsed = match rehearse_the_publishes_raising_the_floor(
        &pool,
        allocator,
        current,
        new_floor,
        &planned_txgs,
        entry.unmount_marker(),
    ) {
        Ok(placements_rehearsed) => placements_rehearsed,
        Err(refusal) => {
            *allocator = allocator_before_the_raise;
            return Err(
                MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                    refused_publish_in_the_sequence: refusal.refused_publish_in_the_sequence,
                    publishes_in_the_sequence,
                    cause: refusal.cause,
                },
            );
        }
    };
    // 预演过了、第一次发布之前：先把新 F 写进每块盘的系统配置、过一道屏障，之后才发第一条带新 F 的根（SysPre，D16（发布语义） 已定项 1
    // 「抬 F 那一串」、已定项 7）。于是任何一条带新 F 的根落盘时，它那块盘的系统配置里已经有新 F（I-7.12（系统配置 F 不低于同盘根上的 F））；
    // 带新 F 的根全坏了，生效值照样读得出新 F。这一步报错：这一串的根一条都不发，已写进的新 F 留着、不回卷，分配器换回抬 F 之前那一份。
    let system_configuration_writes_before_the_first_publish =
        match write_the_raised_floor_into_every_system_configuration(
            &mut pool,
            new_floor,
            current.record.counter,
            current.record.instance,
        ) {
            // 写成了：这一读留在写入口上，这一串第一次空发布落盘时一并转交。
            Ok(writes) => writes,
            Err(failed) => {
                *allocator = allocator_before_the_raise;
                pool.count_the_reads_before_the_system_configuration_writes(allocator);
                return Err(
                    MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(Box::new(
                        failed,
                    )),
                );
            }
        };
    let mut publishes = Vec::new();
    // 迭代上界是预演过的次数（至多根环区域数）；跨轮携带的是现行那一版（下一次的计划接着它）。
    for planned_txg in &planned_txgs {
        let published = publish_version_with_unmount_marker(
            &mut pool,
            allocator,
            empty_publish_plan_raising_the_floor(current, new_floor),
            Some(&*current),
            entry.unmount_marker(),
        );
        let next = match published {
            Ok(next) => next,
            Err(cause) => {
                // 预演过了还在这里报错的只剩两种：落盘途中失败（那一次冻结在分配器上等原样重发，它的字节按回收之后的账装的，不换），
                // 与真发时读盘核出对不上、那一份留在已分配，而那一槽在拷贝上被这一串自己回收又发了出去（两边分叉，
                // 同 `establish_instance` 那一格）。第一次就在任何写之前被拒的，这一串什么都没落盘，分配器换回抬 F 之前的那一份；
                // 第二次起被拒的，前面已落盘的根带着新 F 与回收之后的账，扣住位照旧留在这个进程里，下一次抬 F 做成才放开
                // （C516（抬 F 那一串发布被拒时前面几次已落盘）；这一格扣住位怎么放，条款没写）。
                if publishes.is_empty() && allocator.frozen_publish().is_none() {
                    *allocator = allocator_before_the_raise.clone();
                }
                // 抬 F 的写入口是这里开的、随错一起丢掉：已经落盘的那几次各自的写与失败那一次已记的写都随错交出（增补 2 收口表第 58 行）。
                return Err(MountError::RaiseFloorSequencePublishFailed(
                    publish_sequence_failed(
                        &pool,
                        system_configuration_writes_before_the_first_publish.clone(),
                        publishes
                            .iter()
                            .map(|persisted: &TransactionOutput| persisted.writes.clone())
                            .collect(),
                        cause,
                    ),
                ));
            }
        };
        assert_eq!(
            next.root.checkpoint_txg, *planned_txg,
            "真发的 txg 与这一串计划里的那一个相同：计划从现行那一版的 txg 逐个加一，每一次都接着现行那一版加一"
        );
        publishes.push(next.clone());
        *current = next;
    }
    // 预演取的落点就是真发取的：同一个分配器、同一串分配动作。有一份换下的单元读盘核对不上时不比（那一份真发时留在已分配，
    // 拷贝上照常释放，那一槽若在这一串里被回收两边可以不同，同 `establish_instance` 那一格）。
    let some_copy_was_quarantined = publishes.iter().any(|version: &TransactionOutput| {
        !version
            .quarantined_after_release_checksum_mismatch
            .is_empty()
    });
    if !some_copy_was_quarantined {
        let placements_taken: Vec<PlacementsTakenByOnePublish> = publishes
            .iter()
            .map(placements_taken_by_a_file_version)
            .collect();
        assert_eq!(
            placements_taken, placements_rehearsed,
            "抬 F 之前在分配器的拷贝上预演取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
             不同说明两处的计划、次序或记根分叉了，预演判的不是真发的那一串"
        );
    }
    allocator.release_reclaim_holds();
    Ok(FloorRaisedThroughTheSequence {
        ceiling_judged_by_the_admission,
        system_configuration_writes_before_the_first_publish,
        publishes,
        reclaimed,
        abandoned_roots_unreadable,
    })
}

/// 抬 F 那一串空发布的 txg：从现行那一版的下一个 txg 起逐次加一，直到每块盘上都落过一条（那一次的根落在哪块盘由根环落点公式定，
/// `device_of_txg`），至多根环区域数那么多次（D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」）。预演与真发都按这一串推。
///
/// # Errors
/// 下一个 txg 越过 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条；调用方在动分配器与任何写之前算它）。
pub(super) fn txgs_of_the_publishes_carrying_the_floor_to_every_device(
    current_txg: CheckpointTxg,
    all_devices: &[DeviceIdentity],
    device_of_txg: &dyn Fn(CheckpointTxg) -> DeviceIdentity,
) -> Result<Vec<CheckpointTxg>, MountError> {
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publishes: Vec<CheckpointTxg> = Vec::new();
    // 迭代上界是根环区域数；跨轮携带的是已经落到了哪几块盘与上一次的 txg。
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let txg = checkpoint_txg_after_the_version(
            publishes.last().map_or(current_txg, |previous| *previous),
        )?;
        let device = device_of_txg(txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(txg);
    }
    Ok(publishes)
}

/// 计划里的下一次空发布接在现行那一版后面：它的 txg 加一。计划那一串在任何写之前按同一个加一算过
/// （`warm_up_publish_txgs`、`txgs_of_the_publishes_carrying_the_floor_to_every_device`，越过 `u64::MAX` 就在那里拒）。
///
/// # Panics
/// 现行那一版的 txg 已是 `u64::MAX`：那几个计划函数先拒了，走到这里说明调用方没按计划推。
pub(super) fn checkpoint_txg_of_the_planned_publish_after(current: CheckpointTxg) -> CheckpointTxg {
    CheckpointTxg(current.0.checked_add(1).expect(
        "计划那一串在任何写之前按同一个加一算过、没越过 u64::MAX（warm_up_publish_txgs / txgs_of_the_publishes_carrying_the_floor_to_every_device）",
    ))
}

/// 抬 F 那一串的一次空发布的计划：接在现行那一版之后，带新 F。预演与真发读同一张计划（`raise_rollback_floor`）。
fn empty_publish_plan_raising_the_floor<'plan>(
    current: &TransactionOutput,
    new_floor: CheckpointTxg,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: checkpoint_txg_of_the_planned_publish_after(current.root.checkpoint_txg),
        counter: current.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current
            .highest_transaction_number_in_this_instance,
        instance: current.root.instance,
        back_chain: back_chain_of(&current.record_bytes),
        file: None,
        // 抬 F 的这几次空发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current.root.instance_table),
        tree_birth_txg: current.tree_birth_txg(),
        // 一个树 ID 都不发：现行那一版的水位就是根环里的 max（本会话每次发布只照抄或推高它）。
        tree_identifier_watermark: current.root.tree_identifier_watermark,
        rollback_floor: new_floor,
    }
}

/// 抬 F 那一串的回收（[`raise_rollback_floor`]）：影子账开着时回收出来的槽里隔离还护着的当场补隔离
/// （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）——C 那一圈的落点在抬 F 重算影子账那一刻
/// 分配位置着的不隔离（D28（挂载期承诺量） 已定项 1 第九项，[`isolate_slots_referenced_only_by_abandoned_roots`] 那一圈的注释），
/// 不在这里补，它们回收之后就发得出去。影子账关着（只供测试的对照，什么都不护）时只回收。
fn reclaim_released_in_the_sequence_raising_the_floor_up_to(
    allocator: &mut PoolAllocator,
    shadow_ledger: ShadowLedger,
    floor: CheckpointTxg,
    reuse: ReclaimedReuse,
) -> Vec<Placement> {
    match shadow_ledger {
        ShadowLedger::On => allocator
            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),
        ShadowLedger::Off => allocator.reclaim_released_up_to(floor, reuse),
    }
}

/// 抬 F 那一串在预演里第几次（从 1 数）报了什么错。
struct RaiseFloorRehearsalRefusal {
    refused_publish_in_the_sequence: usize,
    cause: PublishError,
}

/// 抬 F 那一串空发布（txg 依次是 `planned_txgs`）在分配器的一份拷贝上逐次预演：只动拷贝、不碰盘，走的就是发布路径落盘之前那一段
/// （`transaction::prepare_the_version_publish`，真发的 `publish_version_with_unmount_marker` 走同一段、给同一个 `unmount_marker`），
/// 换下的每一份不读盘核（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）、照常释放。交回每一次取到的落点。
///
/// # Errors
/// 第几次（从 1 数）的准备报了什么错（[`RaiseFloorRehearsalRefusal`]）。
fn rehearse_the_publishes_raising_the_floor<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    new_floor: CheckpointTxg,
    planned_txgs: &[CheckpointTxg],
    unmount_marker: UnmountMarker,
) -> Result<Vec<PlacementsTakenByOnePublish>, RaiseFloorRehearsalRefusal> {
    let mut copy = allocator.clone();
    let mut rehearsed_current = current.clone();
    let mut taken_by_every_publish = Vec::with_capacity(planned_txgs.len());
    // 迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）。
    for (publish_index, planned_txg) in planned_txgs.iter().enumerate() {
        let (_, rehearsed) = prepare_the_version_publish(
            pool,
            &mut copy,
            &empty_publish_plan_raising_the_floor(&rehearsed_current, new_floor),
            Some(&rehearsed_current),
            rehearsed_current.tree_identifiers,
            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
            unmount_marker,
            &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
        )
        .map_err(|cause| RaiseFloorRehearsalRefusal {
            refused_publish_in_the_sequence: publish_index + 1,
            cause,
        })?;
        assert_eq!(
            rehearsed.root.checkpoint_txg, *planned_txg,
            "预演的 txg 与这一串计划里的那一个相同：计划从现行那一版的 txg 逐个加一，预演每一次都接着预演出来的现行那一版加一"
        );
        taken_by_every_publish.push(placements_taken_by_a_file_version(&rehearsed));
        rehearsed_current = rehearsed;
    }
    Ok(taken_by_every_publish)
}

/// 抬 F 那一遍按读出根的那一槽认盘、认槽（判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 3 问）：
/// mkfs 在三个区域的槽 0 各种一份第 0 代根，txg 都是 0；按落点公式从 txg 推，三份都落到区域 0 的槽 0、区域 0 的盘上。
/// 区域归属取第一版几何的 0 / 1 / 0（D2（RAID 条带策略） 已定项 7）。
#[cfg(test)]
mod ring_slots_of_the_roots_read_for_raising_the_floor_tests {
    use std::collections::BTreeMap;

    use super::{
        newest_valid_root_txg_on_every_device,
        ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass,
        roots_on_the_devices_of_their_ring_slots,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
    use crate::instance_table::InstanceTableRecords;
    use crate::make_filesystem::GENESIS_CHECKPOINT_TXG;
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};
    use crate::root_ring::RootRingSlot;

    /// 第一版几何的区域归属：区域 0、2 在盘 0，区域 1 在盘 1。
    const REGION_DEVICES_OF_THE_FIRST_GEOMETRY: [DeviceIdentity; 3] =
        [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];

    fn root_of(instance: u32, checkpoint_txg: CheckpointTxg) -> RootRecord {
        RootRecord {
            filesystem_identifier: [0; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg,
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    fn genesis_ring_slot_of_region(region: u64) -> RootRingSlot {
        RootRingSlot { region, slot: 0 }
    }

    /// 场景：卸载那一格这一遍读出三个区域槽 0 的第 0 代根与区域 0 槽 1 的 txg 3（实例 1）。
    /// 预期：盘 1 上读出了区域 1 槽 0 那一份第 0 代根，每块盘上最新的有效根取最小是 0（盘 0 是 3、盘 1 是 0）。
    /// 判别力：按落点公式从 txg 推盘时三份第 0 代根都归盘 0、盘 1 一条有效根都没有，交回 None（变异「抬 F 那一遍按 txg 认盘」）。
    #[test]
    fn the_genesis_root_read_from_slot_zero_of_region_one_stands_for_the_newest_valid_root_of_device_one(
    ) {
        let roots_with_ring_slots = [
            (
                genesis_ring_slot_of_region(0),
                root_of(0, GENESIS_CHECKPOINT_TXG),
            ),
            (
                genesis_ring_slot_of_region(1),
                root_of(0, GENESIS_CHECKPOINT_TXG),
            ),
            (
                genesis_ring_slot_of_region(2),
                root_of(0, GENESIS_CHECKPOINT_TXG),
            ),
            (
                RootRingSlot { region: 0, slot: 1 },
                root_of(1, CheckpointTxg(3)),
            ),
        ];
        let roots_on_their_devices = roots_on_the_devices_of_their_ring_slots(
            &roots_with_ring_slots,
            &REGION_DEVICES_OF_THE_FIRST_GEOMETRY,
        );
        assert_eq!(
            newest_valid_root_txg_on_every_device(
                &roots_on_their_devices,
                &InstanceTableRecords { rows: Vec::new() },
                &[DeviceIdentity(0), DeviceIdentity(1)],
            ),
            Some(GENESIS_CHECKPOINT_TXG),
            "区域 1 槽 0 读出的第 0 代根归盘 1：盘 0 最新 3、盘 1 最新 0，取最小 0；按 txg 推盘时盘 1 一条都没有、交回 None"
        );
    }

    /// 场景：区域 1 的槽 0 挂载那一刻读坏、钉在第 0 代根的 txg 0；抬 F 这一遍在区域 0、区域 1 的槽 0 都读出了第 0 代根。
    /// 预期：区域 1 的槽 0 这一遍读出了根，不再并进环里最旧有效根（不交回）。
    /// 判别力：按落点公式从 txg 推槽时两份第 0 代根都认成区域 0 的槽 0，区域 1 的槽 0 照旧当没读出、留在交回里；
    /// 这一遍读出根的槽一个都不从钉里滤掉时同样留在交回里
    /// （`crates/mutations.tsv`「抬 F 那一遍读出根的槽不从钉里滤掉」）。
    #[test]
    fn ring_slot_pinned_since_the_mount_where_this_pass_reads_a_genesis_root_outside_region_zero_is_no_longer_pinned(
    ) {
        let pinned_since_the_mount =
            BTreeMap::from([(genesis_ring_slot_of_region(1), GENESIS_CHECKPOINT_TXG)]);
        let roots_with_ring_slots_read_on_this_pass = [
            (
                genesis_ring_slot_of_region(0),
                root_of(0, GENESIS_CHECKPOINT_TXG),
            ),
            (
                genesis_ring_slot_of_region(1),
                root_of(0, GENESIS_CHECKPOINT_TXG),
            ),
        ];
        assert_eq!(
            ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass(
                pinned_since_the_mount,
                &roots_with_ring_slots_read_on_this_pass,
            ),
            BTreeMap::new(),
            "区域 1 的槽 0 这一遍读出了第 0 代根，不再钉着"
        );
    }

    /// 场景：区域 0 的槽 0 挂载那一刻读坏、钉在 0，抬 F 这一遍又没读出它；这一遍在区域 1、区域 2 的槽 0 读出了第 0 代根。
    /// 预期：区域 0 的槽 0 照旧钉着（交回它与钉值 0）。
    /// 判别力：按落点公式从 txg 推槽时别的区域读出的第 0 代根被认成区域 0 的槽 0，把它从钉里拿掉（交回空表）。
    /// 调用点按 txg 推槽的那一行变异（`crates/mutations.tsv`「抬 F 那一遍按 txg 认读出根的槽」）点名的是
    /// `readable_roots_for_raising_the_floor_tests` 里 `oldest_valid_root_in_the_ring_for_raising_the_floor` 的那一条。
    #[test]
    fn ring_slot_of_region_zero_pinned_since_the_mount_stays_pinned_though_this_pass_reads_genesis_roots_of_the_other_regions(
    ) {
        let pinned_since_the_mount =
            BTreeMap::from([(genesis_ring_slot_of_region(0), GENESIS_CHECKPOINT_TXG)]);
        let roots_with_ring_slots_read_on_this_pass = [
            (
                genesis_ring_slot_of_region(1),
                root_of(0, GENESIS_CHECKPOINT_TXG),
            ),
            (
                genesis_ring_slot_of_region(2),
                root_of(0, GENESIS_CHECKPOINT_TXG),
            ),
        ];
        assert_eq!(
            ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass(
                pinned_since_the_mount.clone(),
                &roots_with_ring_slots_read_on_this_pass,
            ),
            pinned_since_the_mount,
            "区域 0 的槽 0 这一遍没读出根，照旧钉在 0"
        );
    }
}

#[cfg(test)]
mod readable_roots_for_raising_the_floor_tests {
    use super::{
        oldest_valid_root_in_the_ring_for_raising_the_floor, readable_roots_for_raising_the_floor,
        RootsReadForRaisingTheFloor,
    };
    use crate::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    };
    use crate::allocator::{
        PlacementOnDevice, PoolAllocator, RootReadOnOneRebuildPass, RootRingOccupancy,
        RootRingOccupant, TxgsARingSlotBadAtMountCanHold,
    };
    use crate::instance_table::InstanceTableRecords;
    use crate::make_filesystem::{MakeFilesystemParameters, GENESIS_CHECKPOINT_TXG};
    use crate::pointer::NodePointer;
    use crate::recovery::{every_root_ring_slot, PoolReader};
    use crate::root_record::{PinnedRingSlot, RootRecord, UnmountMarker};
    use crate::root_ring::{slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
    use crate::system_configuration::{
        SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
        SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
        JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
    };
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"raise-floor-ring";
    /// 第一版几何的区域归属：区域 0、2 在盘 0，区域 1 在盘 1。
    const REGION_DEVICES: [DeviceIdentity; 3] =
        [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];
    const SLOTS_PER_REGION: RootRingSlotsPerRegion = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    /// 根槽宽（物理块 512）。
    const ROOT_SLOT_BYTES: usize = 512;
    /// 读得出、不是全零而自证不过的根槽（校验和对不上）每个字节的值。
    const BYTE_OF_A_SLOT_FAILING_SELF_VERIFICATION: u8 = 0xa5;
    /// 这个进程下一条根的 txg 减一：根环表里最新的那条根（被抛弃的 txg 9）。
    const LATEST_TXG_BEFORE_THE_NEXT_PUBLISH: CheckpointTxg = CheckpointTxg(9);

    fn sizes() -> SystemImmutableSizes {
        SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: SLOTS_PER_REGION,
        }
    }

    fn system_configuration_with_the_first_geometry() -> SystemConfiguration {
        SystemConfiguration {
            immutable: SystemImmutableConfiguration {
                filesystem_identifier: FILESYSTEM_IDENTIFIER,
                this_device: DeviceIdentity(0),
                device_count: 2,
                region_devices: REGION_DEVICES,
                sizes: sizes(),
                journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
                root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
            },
            mutable: SystemMutableConfiguration,
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation: 1,
                journal_tail: 0,
                journal_instance: InstanceGeneration(1),
                rollback_floor: CheckpointTxg(0),
            },
        }
    }

    fn root_of(instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> RootRecord {
        RootRecord {
            filesystem_identifier: FILESYSTEM_IDENTIFIER,
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance,
            checkpoint_txg,
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: CheckpointTxg(0),
            pinned_ring_slot: PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    /// mkfs 种进 `region` 那个区域槽 0 的第 0 代根（实例 0、txg 0）。
    fn genesis_ring_slot_of_region(region: u64) -> RootRingSlot {
        RootRingSlot { region, slot: 0 }
    }

    /// `checkpoint_txg` 那条根照落点公式住的槽。
    fn ring_slot_of_txg(checkpoint_txg: CheckpointTxg) -> RootRingSlot {
        target_for_publish(checkpoint_txg, SLOTS_PER_REGION)
    }

    fn slot_bytes_of(root: &RootRecord) -> Option<Vec<u8>> {
        Some(root.to_slot(ROOT_SLOT_BYTES))
    }

    fn slot_bytes_failing_self_verification() -> Option<Vec<u8>> {
        Some(vec![
            BYTE_OF_A_SLOT_FAILING_SELF_VERIFICATION;
            ROOT_SLOT_BYTES
        ])
    }

    fn slot_bytes_all_zero() -> Option<Vec<u8>> {
        Some(vec![0; ROOT_SLOT_BYTES])
    }

    /// 一个根环槽在盘上的位置（读者拿它认读的是哪一槽）。
    fn device_and_offset_of_ring_slot(
        ring_slot: RootRingSlot,
    ) -> (DeviceIdentity, DeviceOffsetInBytes) {
        (
            REGION_DEVICES[usize::try_from(ring_slot.region).expect("区域号小于根环区域数 3")],
            slot_offset(ring_slot, sizes().fixed_structure_slot_spacing),
        )
    }

    /// 读根环的读者，每一槽两种读数分开排：这一遍读（`read`）照 `readings_of_this_pass` 交，没排的槽交全 0（从没写过）；
    /// 读坏之后当场重读那一读（`reread_after_a_bad_read`）照 `readings_of_the_reread` 交，没排重读读数的槽被重读就 panic。
    /// `None` 是设备报读错。别处不读。
    struct RootRingReadingsOfThisPassAndTheReread {
        readings_of_this_pass: BTreeMap<RootRingSlot, Option<Vec<u8>>>,
        readings_of_the_reread: BTreeMap<RootRingSlot, Option<Vec<u8>>>,
    }

    impl RootRingReadingsOfThisPassAndTheReread {
        /// 读的那一段是哪一个根环槽；读的不是一个整根槽就 panic。
        fn ring_slot_read_at(
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> RootRingSlot {
            assert_eq!(length, ROOT_SLOT_BYTES, "只读整根槽");
            every_root_ring_slot(&sizes())
                .into_iter()
                .find(|ring_slot| device_and_offset_of_ring_slot(*ring_slot) == (device, offset))
                .expect("只读根环槽")
        }
    }

    impl PoolReader for RootRingReadingsOfThisPassAndTheReread {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            vec![DeviceIdentity(0), DeviceIdentity(1)]
        }
        fn device_size_in_bytes(&self, _device: DeviceIdentity) -> Option<u64> {
            unreachable!("读根环不问盘多大")
        }
        fn physical_block_size_probed_on_the_device(
            &self,
            _device: DeviceIdentity,
        ) -> Option<crate::block_device::PhysicalBlockSizeInBytes> {
            unreachable!("读根环不探测块宽")
        }
        fn read(
            &self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Option<Vec<u8>> {
            let ring_slot = Self::ring_slot_read_at(device, offset, length);
            self.readings_of_this_pass
                .get(&ring_slot)
                .cloned()
                .unwrap_or_else(slot_bytes_all_zero)
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            unreachable!("读根环不扫 journal 环")
        }
        fn reread_after_a_bad_read(
            &self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Option<Vec<u8>> {
            let ring_slot = Self::ring_slot_read_at(device, offset, length);
            self.readings_of_the_reread
                .get(&ring_slot)
                .cloned()
                .expect("只重读排了重读读数的槽：两遍里只有一遍自证过、这一遍读坏的那几槽")
        }
    }

    fn valid_root_self_verified_on_only_one_rebuild_pass(root: RootRecord) -> RootRingOccupant {
        RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
            root: RootReadOnOneRebuildPass::ValidRoot {
                checkpoint_txg: root.checkpoint_txg,
            },
            root_record_self_verified_on_one_rebuild_pass: Box::new(root),
        }
    }

    fn ring_slot_bad_at_mount_that_can_hold_only(
        checkpoint_txg: CheckpointTxg,
    ) -> RootRingOccupant {
        RootRingOccupant::BadAtMount(TxgsARingSlotBadAtMountCanHold::new(
            checkpoint_txg,
            checkpoint_txg,
        ))
    }

    /// 场景：根环（S = 8）里几种槽各一读——区域 0 槽 0 的第 0 代根、txg 3 这一遍读得出（根环表记成有效根）；
    /// 两遍里只有一遍自证过的四槽：区域 1 槽 0（第 0 代根）这一遍设备报读错、重读读出，区域 2 槽 0（第 0 代根）这一遍读错、重读读坏不是全零，
    /// txg 4 那一槽这一遍读坏不是全零、重读读回全零，txg 8 那一槽这一遍就读得出；挂载那一刻读坏的两槽：txg 5 那一槽这一遍读得出，
    /// txg 6 那一槽这一遍仍读坏；记成被抛弃根的 txg 9 那一槽这一遍读坏；别的槽全零（从没写过）。
    /// 预期：交回的每一条根带的槽是它真住的那一槽——这一遍读出的按读根环次序，重读读出与顶上的按槽次序排在后面；
    /// 区域 1、区域 2 槽 0 那两份第 0 代根各带自己的槽，不带公式从 txg 0 推出的区域 0 槽 0；挂载那一刻读坏、这一遍仍读坏的槽当没有根，
    /// 被抛弃根那一槽交回它记着的落点。
    /// 判别力：`crates/mutations.tsv`「抬 F 那一遍两遍里只有一遍自证过的槽重读读出的根按 txg 推槽」让区域 1 槽 0 那一份带成区域 0 槽 0，
    /// 「抬 F 那一遍两遍里只有一遍自证过的槽重读仍坏顶上的根按 txg 推槽」让区域 2 槽 0 那一份带成区域 0 槽 0，各红在交回的根那一条断言上。
    #[test]
    fn each_root_read_for_raising_the_floor_carries_the_ring_slot_it_lives_in_however_the_slot_reads(
    ) {
        let genesis_root = root_of(InstanceGeneration(0), GENESIS_CHECKPOINT_TXG);
        let root_of_txg_three = root_of(InstanceGeneration(1), CheckpointTxg(3));
        let root_of_txg_four = root_of(InstanceGeneration(1), CheckpointTxg(4));
        let root_of_txg_five = root_of(InstanceGeneration(1), CheckpointTxg(5));
        let root_of_txg_eight = root_of(InstanceGeneration(1), CheckpointTxg(8));
        let ring_slot_of_txg_six = ring_slot_of_txg(CheckpointTxg(6));
        let ring_slot_of_the_abandoned_root = ring_slot_of_txg(LATEST_TXG_BEFORE_THE_NEXT_PUBLISH);
        assert_eq!(
            ring_slot_of_txg(GENESIS_CHECKPOINT_TXG),
            genesis_ring_slot_of_region(0),
            "前提：落点公式把 txg 0 推到区域 0 槽 0，区域 1、区域 2 槽 0 那两份第 0 代根按公式推会带错槽"
        );
        let placement_referenced_by_the_abandoned_root = PlacementOnDevice {
            device: DeviceIdentity(1),
            slot: SlotNumber(50180),
            span: 2,
        };
        let mut allocator = PoolAllocator::new(Vec::new());
        allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
            SLOTS_PER_REGION,
            vec![
                (
                    genesis_ring_slot_of_region(0),
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: GENESIS_CHECKPOINT_TXG,
                    },
                ),
                (
                    ring_slot_of_txg(root_of_txg_three.checkpoint_txg),
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: root_of_txg_three.checkpoint_txg,
                    },
                ),
                (
                    genesis_ring_slot_of_region(1),
                    valid_root_self_verified_on_only_one_rebuild_pass(genesis_root),
                ),
                (
                    genesis_ring_slot_of_region(2),
                    valid_root_self_verified_on_only_one_rebuild_pass(genesis_root),
                ),
                (
                    ring_slot_of_txg(root_of_txg_four.checkpoint_txg),
                    valid_root_self_verified_on_only_one_rebuild_pass(root_of_txg_four),
                ),
                (
                    ring_slot_of_txg(root_of_txg_eight.checkpoint_txg),
                    valid_root_self_verified_on_only_one_rebuild_pass(root_of_txg_eight),
                ),
                (
                    ring_slot_of_txg(root_of_txg_five.checkpoint_txg),
                    ring_slot_bad_at_mount_that_can_hold_only(root_of_txg_five.checkpoint_txg),
                ),
                (
                    ring_slot_of_txg_six,
                    ring_slot_bad_at_mount_that_can_hold_only(CheckpointTxg(6)),
                ),
                (
                    ring_slot_of_the_abandoned_root,
                    RootRingOccupant::AbandonedRoot {
                        instance: InstanceGeneration(1),
                        checkpoint_txg: LATEST_TXG_BEFORE_THE_NEXT_PUBLISH,
                        referenced_placements: vec![placement_referenced_by_the_abandoned_root],
                    },
                ),
            ],
            CheckpointTxg(0),
            LATEST_TXG_BEFORE_THE_NEXT_PUBLISH,
        ));
        let reader = RootRingReadingsOfThisPassAndTheReread {
            readings_of_this_pass: BTreeMap::from([
                (genesis_ring_slot_of_region(0), slot_bytes_of(&genesis_root)),
                (
                    ring_slot_of_txg(root_of_txg_three.checkpoint_txg),
                    slot_bytes_of(&root_of_txg_three),
                ),
                (genesis_ring_slot_of_region(1), None),
                (genesis_ring_slot_of_region(2), None),
                (
                    ring_slot_of_txg(root_of_txg_four.checkpoint_txg),
                    slot_bytes_failing_self_verification(),
                ),
                (
                    ring_slot_of_txg(root_of_txg_eight.checkpoint_txg),
                    slot_bytes_of(&root_of_txg_eight),
                ),
                (
                    ring_slot_of_txg(root_of_txg_five.checkpoint_txg),
                    slot_bytes_of(&root_of_txg_five),
                ),
                (ring_slot_of_txg_six, slot_bytes_failing_self_verification()),
                (
                    ring_slot_of_the_abandoned_root,
                    slot_bytes_failing_self_verification(),
                ),
            ]),
            readings_of_the_reread: BTreeMap::from([
                (genesis_ring_slot_of_region(1), slot_bytes_of(&genesis_root)),
                (
                    genesis_ring_slot_of_region(2),
                    slot_bytes_failing_self_verification(),
                ),
                (
                    ring_slot_of_txg(root_of_txg_four.checkpoint_txg),
                    slot_bytes_all_zero(),
                ),
            ]),
        };
        let RootsReadForRaisingTheFloor {
            roots_with_ring_slots,
            placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
        } = readable_roots_for_raising_the_floor(
            &reader,
            &system_configuration_with_the_first_geometry(),
            &allocator,
        )
        .expect("知道住着根而这一遍读坏的槽都是两遍里只有一遍自证过的或记成被抛弃根的：不拒");
        assert_eq!(
            roots_with_ring_slots,
            vec![
                (genesis_ring_slot_of_region(0), genesis_root),
                (
                    ring_slot_of_txg(root_of_txg_three.checkpoint_txg),
                    root_of_txg_three
                ),
                (
                    ring_slot_of_txg(root_of_txg_five.checkpoint_txg),
                    root_of_txg_five
                ),
                (
                    ring_slot_of_txg(root_of_txg_eight.checkpoint_txg),
                    root_of_txg_eight
                ),
                (genesis_ring_slot_of_region(1), genesis_root),
                (
                    ring_slot_of_txg(root_of_txg_four.checkpoint_txg),
                    root_of_txg_four
                ),
                (genesis_ring_slot_of_region(2), genesis_root),
            ],
            "这一遍读出的四条按读根环次序带各自的槽；重读读出的（区域 1 槽 0）、重读读坏不是全零而顶上的（区域 2 槽 0）、\
             重读读回全零而顶上的（txg 4 那一槽）按槽次序排在后面，各带它真住的那一槽；txg 6 那一槽当没有根"
        );
        assert_eq!(
            placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
            vec![placement_referenced_by_the_abandoned_root],
            "记成被抛弃根的那一槽这一遍读坏：交回它记着的落点"
        );
    }

    fn parameters_of_the_first_geometry() -> MakeFilesystemParameters {
        MakeFilesystemParameters {
            filesystem_identifier: FILESYSTEM_IDENTIFIER,
            region_devices: REGION_DEVICES,
            geometry: sizes(),
        }
    }

    /// 抬 F 那一遍算环里最旧有效根时重读挂载那一刻钉着的槽所用的读者：每一槽的读数照 `readings` 交，读了哪几槽按次序记进 `ring_slots_read`。
    /// 读没排读数的槽、或走读坏之后当场重读那一读，就 panic：这一步只重读钉着的槽，各读一次，走 `read`。`None` 是设备报读错。
    struct RootRingSlotsReadOneByOne {
        readings: BTreeMap<RootRingSlot, Option<Vec<u8>>>,
        ring_slots_read: RefCell<Vec<RootRingSlot>>,
    }

    impl PoolReader for RootRingSlotsReadOneByOne {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            vec![DeviceIdentity(0), DeviceIdentity(1)]
        }
        fn device_size_in_bytes(&self, _device: DeviceIdentity) -> Option<u64> {
            unreachable!("重读根环槽不问盘多大")
        }
        fn physical_block_size_probed_on_the_device(
            &self,
            _device: DeviceIdentity,
        ) -> Option<crate::block_device::PhysicalBlockSizeInBytes> {
            unreachable!("重读根环槽不探测块宽")
        }
        fn read(
            &self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Option<Vec<u8>> {
            let ring_slot =
                RootRingReadingsOfThisPassAndTheReread::ring_slot_read_at(device, offset, length);
            self.ring_slots_read.borrow_mut().push(ring_slot);
            self.readings
                .get(&ring_slot)
                .cloned()
                .expect("只读排了读数的槽：挂载那一刻钉着的那几槽")
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            unreachable!("重读根环槽不扫 journal 环")
        }
        fn reread_after_a_bad_read(
            &self,
            _device: DeviceIdentity,
            _offset: DeviceOffsetInBytes,
            _length: usize,
        ) -> Option<Vec<u8>> {
            unreachable!("算环里最旧有效根只把钉着的槽读一次，不走读坏之后当场重读那一读")
        }
    }

    /// 场景：根环（S = 8）里区域 0 槽 0 挂载那一刻读坏，钉在第 0 代根的 txg 0；抬 F 这一遍在区域 1、区域 2 槽 0 各读出一份第 0 代根，
    /// 区域 0 槽 0 没读出根；重读区域 0 槽 0 时设备报读错。
    /// 预期：交回的环里最旧有效根是 txg 0。区域 0 槽 0 照旧钉着：钉值 0 不大于两份第 0 代根的 txg 0，照规矩先重读这一槽一次，读者只读到它。
    /// 判别力：`crates/mutations.tsv`「抬 F 那一遍按 txg 认读出根的槽」在调用点按落点公式从 txg 推槽（取池的 S），两份第 0 代根都被认成区域 0 槽 0，
    /// 区域 0 槽 0 从钉里拿掉、不重读，读到的槽那一条断言红。交回的 txg 在基线与变异下都是 0，那一条不红。
    #[test]
    fn the_oldest_valid_root_for_raising_the_floor_keeps_the_ring_slot_of_region_zero_pinned_though_genesis_roots_are_read_from_the_other_regions(
    ) {
        let genesis_root = root_of(InstanceGeneration(0), GENESIS_CHECKPOINT_TXG);
        assert_eq!(
            ring_slot_of_txg(GENESIS_CHECKPOINT_TXG),
            genesis_ring_slot_of_region(0),
            "前提：落点公式把 txg 0 推到区域 0 槽 0，区域 1、区域 2 槽 0 那两份第 0 代根按公式推会认成区域 0 槽 0"
        );
        let mut allocator = PoolAllocator::new(Vec::new());
        allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
            SLOTS_PER_REGION,
            vec![
                (
                    genesis_ring_slot_of_region(0),
                    ring_slot_bad_at_mount_that_can_hold_only(GENESIS_CHECKPOINT_TXG),
                ),
                (
                    genesis_ring_slot_of_region(1),
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: GENESIS_CHECKPOINT_TXG,
                    },
                ),
                (
                    genesis_ring_slot_of_region(2),
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: GENESIS_CHECKPOINT_TXG,
                    },
                ),
            ],
            CheckpointTxg(0),
            GENESIS_CHECKPOINT_TXG,
        ));
        let reader = RootRingSlotsReadOneByOne {
            readings: BTreeMap::from([(genesis_ring_slot_of_region(0), None)]),
            ring_slots_read: RefCell::new(Vec::new()),
        };
        let oldest_valid_root = oldest_valid_root_in_the_ring_for_raising_the_floor(
            &reader,
            &parameters_of_the_first_geometry(),
            &allocator,
            &[
                (genesis_ring_slot_of_region(1), genesis_root),
                (genesis_ring_slot_of_region(2), genesis_root),
            ],
            &InstanceTableRecords { rows: Vec::new() },
        );
        assert_eq!(
            oldest_valid_root,
            Some(GENESIS_CHECKPOINT_TXG),
            "两份第 0 代根有效，区域 0 槽 0 钉在 0：环里最旧有效根是 txg 0"
        );
        assert_eq!(
            reader.ring_slots_read.into_inner(),
            vec![genesis_ring_slot_of_region(0)],
            "区域 0 槽 0 这一遍没读出根、照旧钉着，最小的是它的钉值，先重读它一次；按 txg 推槽时它从钉里拿掉、一槽都不读"
        );
    }
}
