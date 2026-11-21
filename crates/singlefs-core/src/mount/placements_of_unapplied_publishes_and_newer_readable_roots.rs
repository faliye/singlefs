//! 影子账同一次隔离、豁免同一个集合的那几份落点：水位之上没施加的 journal 记录点名的、没施加的发布的账里的、
//! 较新可读根的账里的，并上中间的发布与回退复活的落点；一次发布的记录点名的打包单元读成什么、照它认回退。

use super::*;

/// 水位之上、没被施加的 journal 记录点名的落点（C694 的 C，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条，选法 `all`：
/// 不只认带末条标志的记录），按那条记录的 txg 在根环里对应的槽（[`target_for_publish`]）归组。水位是所选那一版 `selected_version`
/// （施加之后的根）的 (实例代号, checkpoint_txg)，按实例代号为主比，与恢复判施加哪一段同一个序（D23（journal 的角色与格式） 已定项 14 注 2）；
/// 水位之下的记录要么施加过、要么比所选那一版旧，不在其内。一条记录交出的落点：它点名的每个单元的两条位置条目（跨度按单元种类），
/// 与它新根段里四条指针（树表、中央映射树根、实例表、分配记录树根；空指针不算）的位置条目——一次发布的每条记录都带整次发布的新根段
/// （已定项 15），末条读不出时前几条的新根段照样说得出那次发布的树表与映射根在哪。落点不在那块盘的单元区里的（坏盘上的记录）不交。
/// 只读内存里的记录、不读盘；读盘只问盘的字节数（落点在不在单元区）。那几次发布的账里的落点另由
/// [`placements_in_the_accounts_of_unapplied_publishes`] 读盘交出，调用方（`rebuilt_allocator`）并进同一份。
pub(super) fn placements_named_by_unapplied_journal_records<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
    let mut named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> = BTreeMap::new();
    for record in records.values().filter(|record| {
        (record.instance, record.checkpoint_txg)
            > (selected_version.instance, selected_version.checkpoint_txg)
    }) {
        let new_root_segment_pointers = [
            (&record.new_tree_table, TransactionUnit::TreeTable),
            (&record.new_mapping_root, TransactionUnit::MappingTree),
            (&record.new_instance_table, TransactionUnit::InstanceTable),
            (
                &record.new_allocation_record_tree_root,
                TransactionUnit::AllocationTree,
            ),
        ]
        .into_iter()
        .filter(|(pointer, _)| **pointer != crate::pointer::NodePointer::empty_root())
        .flat_map(|(pointer, unit)| {
            pointer
                .locations
                .iter()
                .map(move |location| (*location, unit.span_slots()))
        });
        let named_units = record.named.iter().flat_map(|named| {
            let span_slots = crate::recovery::span_in_slots_of_a_named_unit(named);
            named
                .locations
                .iter()
                .map(move |location| (*location, span_slots))
        });
        let placements: Vec<PlacementOnDevice> = named_units
            .chain(new_root_segment_pointers)
            .filter(|(location, span_slots)| {
                placement_lies_in_the_unit_area_of_its_device(
                    devices,
                    unit_area_start,
                    location.device,
                    location.slot,
                    *span_slots,
                )
                .is_ok()
            })
            .map(|(location, span_slots)| PlacementOnDevice {
                device: location.device,
                slot: location.slot,
                span: span_slots,
            })
            .collect();
        named_by_ring_slot
            .entry(target_for_publish(record.checkpoint_txg, slots_per_region))
            .or_default()
            .extend(placements);
    }
    named_by_ring_slot
}

/// C 的账那一份（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K2 T1 C 的盲区」那一行与第三节第 1 条，攻方 `C_ACCOUNT`）：
/// 水位之上每一次没施加的发布（同一 (实例代号, checkpoint_txg) 的记录），用它读得出的记录里计数器最小的那一条的新根段拼一条根
/// （每条记录都带整次发布的新根段，D23（journal 的角色与格式） 已定项 15；[`crate::recovery::root_assembled_from_the_new_root_segment_of_the_record`]），
/// 按影子账同一份读法（[`placements_referenced_by_root`]）读它的账，账里的落点按那次发布的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，
/// 调用方并进 C 的集合、与记录点名的那一份同一个时机解开。账读不出、解不开的那次发布：读得出的记录齐（序号 1..N 连着、带末条）就不交——
/// 那时它的单元多半也验不过，交 F1 乙；不齐就照被抛弃根那一份从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，
/// `C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H3 那一行与第三节第 1 条）。
/// 它补的是 C 只看读得出的记录点名了什么那一处盲区：那次发布的根槽与它的某一条记录（末条、中间一条或第一条）读回全零、单元读得出时，
/// F1 乙不拒，缺的那条记录单独点名的单元又不在别的记录里；账里是那次发布引用的全部落点。水位与 C 同一个（[`placements_named_by_unapplied_journal_records`]）。
///
/// # Errors
/// 那一支现算也不成、重读一次仍不成、现行账里又没有 C694 的 B 那道凭据 ⇒
/// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`
/// （点名那次发布的 (实例代号, checkpoint_txg)），在任何写之前。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：读者、单元区起点、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
)]
fn placements_in_the_accounts_of_unapplied_publishes<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>, MountError> {
    // 记录表按 (实例代号, 计数器) 升序：同一次发布里先遇到的就是计数器最小的那一条。
    let mut readable_records_of_each_publish: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<&crate::journal::JournalRecord>,
    > = BTreeMap::new();
    for record in records.values().filter(|record| {
        (record.instance, record.checkpoint_txg)
            > (selected_version.instance, selected_version.checkpoint_txg)
    }) {
        readable_records_of_each_publish
            .entry((record.instance, record.checkpoint_txg))
            .or_default()
            .push(record);
    }
    let mut in_the_accounts_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> =
        BTreeMap::new();
    for ((_instance, checkpoint_txg), records_of_the_publish) in readable_records_of_each_publish {
        let first_readable_record = records_of_the_publish
            .first()
            .expect("按发布归组时每一组至少放进了一条记录");
        let root_of_the_publish =
            crate::recovery::root_assembled_from_the_new_root_segment_of_the_record(
                first_readable_record,
                selected_version.filesystem_identifier,
            );
        let placements =
            match placements_referenced_by_root(devices, unit_area_start, &root_of_the_publish) {
                Some(placements) => placements,
                None if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
                    &records_of_the_publish,
                ) =>
                {
                    placements_of_an_unapplied_publish_whose_account_is_unreadable(
                        devices,
                        unit_area_start,
                        &root_of_the_publish,
                        current_records,
                        before_the_one_reread,
                    )?
                }
                None => continue,
            };
        in_the_accounts_by_ring_slot
            .entry(target_for_publish(checkpoint_txg, slots_per_region))
            .or_default()
            .extend(
                placements
                    .into_iter()
                    .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
            );
    }
    Ok(in_the_accounts_by_ring_slot)
}

/// C 并上的较新可读根的账（`C_NEWER_ROOTS`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1 那一行与第三节第 1 条，
/// 用户 2026-10-08 定「四形都修」）：根环里比所选那一版 `selected_version` 新（择根的序：checkpoint_txg 为主、实例代号破平局）、按最新那条根的实例表
/// 有效（`is_abandoned` 判否）、读得出的根——F2 退回之后被退掉的那条根就是这样：它的根槽读得出、它自己的实例表里它有效，影子账只管被抛弃的根、
/// 不管它；它的记录又全读不出时 C 那两份（记录点名的、记录拼根读的账）都是空的。照影子账同一份读法（[`placements_referenced_by_root`]）读它的账，
/// 账里的落点按它的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，调用方并进 C 的集合、同一个时机解开。账读不出时照没施加的发布那一份办：
/// 那次发布读得出的记录不齐就从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，仍不成拒），记录齐就不交账（交 F1 乙）、
/// 改交挂载那一刻现行账（`current_records`）里仍分配着的落点（丙，用户 2026-10-08 定，函数体里那一支的注释）；
/// 一条记录都读不出的，映射现算得出就交；现算不成时，那条根的树表与中央映射树根两个指针里有一个两份都已被本池更新实例的单元盖掉
/// （[`one_pointer_has_both_copies_overwritten_by_a_later_instance`]，单元头里的 fsid 与 `filesystem_identifier_in_unit_headers` 相同才算）
/// 就不交也不拒，否则照记录不齐那一支办（仍不成拒；函数体里那一支的注释）。
/// `roots` 是重建分配器读根环那一遍读出的根并上读阶段读到的全部根（[`roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage`]）；`filesystem_identifier_in_unit_headers` 是择到的那份系统配置里
/// fsid 的低 8 字节（单元头里记的那一截，[`crate::unit::unit_filesystem_identifier`]），不另读盘。
///
/// # Errors
/// 同 [`placements_in_the_accounts_of_unapplied_publishes`]（点名那条根的 (实例代号, checkpoint_txg)）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：读者、单元区起点、本池单元头里的 fsid、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
)]
fn placements_in_the_accounts_of_newer_readable_roots<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    filesystem_identifier_in_unit_headers: u64,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<PlacementsInTheAccountsOfNewerReadableRoots, MountError> {
    let mut in_the_accounts_of_newer_roots_by_ring_slot: BTreeMap<
        RootRingSlot,
        Vec<PlacementOnDevice>,
    > = BTreeMap::new();
    let mut txgs_of_the_roots_taking_the_current_account = Vec::new();
    for root in roots.iter().filter(|root| {
        !is_abandoned(root)
            && (root.checkpoint_txg, root.instance)
                > (selected_version.checkpoint_txg, selected_version.instance)
    }) {
        let placements_in_the_newer_roots_account =
            match placements_referenced_by_root(devices, unit_area_start, root) {
                Some(placements) => placements,
                None => {
                    let readable_records_of_its_publish: Vec<&crate::journal::JournalRecord> =
                        records
                            .values()
                            .filter(|record| {
                                (record.instance, record.checkpoint_txg)
                                    == (root.instance, root.checkpoint_txg)
                            })
                            .collect();
                    if readable_records_of_its_publish.is_empty() {
                        // 那次发布一条记录都读不出：映射现算得出就并进来。现算不成时只看那条根自己的
                        // 树表与中央映射树根两个指针（C834，用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`
                        // 「四、用户定案」，被攻过零轮；替换用户 2026-10-08 定的 Y2 收严那一判，不再读系统配置里取过号的实例代号——那个号随盘序变）：
                        // 有一个指针两份都已被本池更新实例的单元盖掉（单元头里的 fsid 与本池的相同才算，用户定「比 fsid」，零轮：
                        // 别的池的单元头不说明本池有过更新的实例），那条根那个单元两份都没了、本来就走不完，映射也就现算不成，拒了就再也做不成
                        // 可写挂载（入库用例 `urJ` 那一形），不拒、不重读；否则照「记录不齐」那一支办（重读一次、看 C694 的 B 凭据，仍不成拒）——
                        // 它的单元可能还在（只是这一次读不出，或只被盖掉一份），不拒就会在写行那次发布里写掉它们。
                        match placements_referenced_by_root_computed_from_the_mapping(
                            devices,
                            unit_area_start,
                            root,
                        ) {
                            Some(computed) => computed,
                            None
                                if !one_pointer_has_both_copies_overwritten_by_a_later_instance(
                                    devices,
                                    root,
                                    filesystem_identifier_in_unit_headers,
                                ) =>
                            {
                                placements_of_an_unapplied_publish_whose_account_is_unreadable(
                                    devices,
                                    unit_area_start,
                                    root,
                                    current_records,
                                    before_the_one_reread,
                                )?
                            }
                            None => continue,
                        }
                    } else if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
                        &readable_records_of_its_publish,
                    ) {
                        placements_of_an_unapplied_publish_whose_account_is_unreadable(
                            devices,
                            unit_area_start,
                            root,
                            current_records,
                            before_the_one_reread,
                        )?
                    } else {
                        // 记录齐：账不交（交 F1 乙），C 那一槽另并上挂载那一刻现行账里仍分配着的落点（丙，D16（发布语义） 已定项 1
                        // 「根槽这一次读坏」那一行末句，用户 2026-10-08 定「改成丙」，判决
                        // `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第三节；攻方改法丙只在攻方模型上量过、被攻过零轮）。
                        // 这条根建在所选那一版上：它引用的单元要么是它自己写的（记录点名了，C 那一份里有），要么在所选那一版的账里，
                        // 也就是现行账里这一截。它们这时分配着、影子账照豁免（第九项不数两遍）；这次挂载里释放之后，抬 F 借 C 那一槽
                        // （写行之后这条根被抛弃，H2）与进程内回收（`allocator::PoolAllocator::record_root_written_by_this_process`）照这一份补隔离。
                        // 不并它时，抬 F 只借到记录点名的那几个，它从所选那一版继承、被会话释放的单元回收出去（I-7.4 红）。
                        // 这一句的前提有两处不成立（判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 两行，
                        // 用户 2026-10-08 定「丙继续补洞」；两处补法的形是第 10z 批第十块实现员定的，被攻过零轮）：
                        // 一、这条根与所选那一版之间还有别的发布（攻方一形）：它从那几次继承的单元只记在 C 里那几次的槽上，
                        //    调用方照交回的 txg 把那几槽并进这条根那一槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）；
                        // 二、这条根是一次管理员回退（正推一形）：它复活的单元在现行账里是已释放，这里另并上
                        //    [`placements_a_rollback_publish_revived_released_in_the_current_account`]；认不出是不是回退、
                        //    或认出回退而复活集算不出时，那里退到现行账里全部已释放的落点（tri，用户 2026-10-08 定）。
                        txgs_of_the_roots_taking_the_current_account.push(root.checkpoint_txg);
                        let mut placements =
                            placements_still_allocated_in_the_account(current_records);
                        placements.extend(
                            placements_a_rollback_publish_revived_released_in_the_current_account(
                                devices,
                                unit_area_start,
                                root,
                                &readable_records_of_its_publish,
                                roots,
                                is_abandoned,
                                current_records,
                            ),
                        );
                        placements
                    }
                }
            };
        in_the_accounts_of_newer_roots_by_ring_slot
            .entry(target_for_publish(root.checkpoint_txg, slots_per_region))
            .or_default()
            .extend(
                placements_in_the_newer_roots_account
                    .into_iter()
                    .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
            );
    }
    Ok(PlacementsInTheAccountsOfNewerReadableRoots {
        by_ring_slot: in_the_accounts_of_newer_roots_by_ring_slot,
        txgs_of_the_roots_taking_the_current_account,
    })
}

/// C834 那一判的凭据（用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`「四、用户定案」，被攻过零轮）：
/// `root` 的根记录里树表与中央映射树根两个指针中有一个，它两条位置条目指着的两个槽都读出自证过的本池单元头（头里的 fsid 与
/// `filesystem_identifier_in_unit_headers` 相同）、写序里的实例代号比 `root` 的实例代号大（[`instance_in_the_self_verified_unit_header_at`]）。
/// 全零的空指针没有槽可读，不算证据。只读盘、不写盘。
/// 只要一份不够（攻方 `research/prompts/c834-r3-opus-output.md` 第二节 T2：盖掉一份、另一份这一次读回全零时那条根还走得完）。
/// 别的池的单元头不算（用户定「比 fsid」，零轮）：它的实例代号是那个池的号，不说明本池有过比 `root` 更新的实例写过这两个槽。
fn one_pointer_has_both_copies_overwritten_by_a_later_instance<Reader: PoolReader>(
    devices: &Reader,
    root: &RootRecord,
    filesystem_identifier_in_unit_headers: u64,
) -> bool {
    [&root.tree_table, &root.mapping_root]
        .into_iter()
        .filter(|pointer| **pointer != crate::pointer::NodePointer::empty_root())
        .any(|pointer| {
            pointer.locations.iter().all(|location| {
                instance_in_the_self_verified_unit_header_at(
                    devices,
                    location,
                    filesystem_identifier_in_unit_headers,
                )
                .is_some_and(|instance| instance > root.instance)
            })
        })
}

/// `location` 那一槽起读出的自证过的本池单元头（头校验和与载荷校验和都过，头里的 fsid 与 `filesystem_identifier_in_unit_headers` 相同）
/// 里写序的实例代号：先按索引节点读 16 KiB，解不开再读 32 KiB、依次按数据单元与打包记录单元解；读不出（越过盘尾、没有那块盘）、
/// 三类都解不开或解得开而 fsid 不是本池的交 `None`。只读盘、不写盘。
fn instance_in_the_self_verified_unit_header_at<Reader: PoolReader>(
    devices: &Reader,
    location: &LocationEntry,
    filesystem_identifier_in_unit_headers: u64,
) -> Option<InstanceGeneration> {
    let offset = location.slot.to_device_offset();
    let index_node_header = PoolReader::read(
        devices,
        location.device,
        offset,
        usize::try_from(NODE_BYTES).expect("16384"),
    )
    .and_then(|bytes| crate::unit::parse_index_node(&bytes).ok());
    let (filesystem_identifier_in_the_header, instance) = match index_node_header {
        Some(header) => (header.filesystem_identifier, header.instance),
        None => {
            let unit_bytes = PoolReader::read(
                devices,
                location.device,
                offset,
                usize::try_from(DATA_UNIT_BYTES).expect("32768"),
            )?;
            crate::unit::parse_data_unit(&unit_bytes)
                .map(|header| (header.filesystem_identifier, header.write_order.instance))
                .or_else(|_not_a_data_unit| {
                    crate::unit::parse_packed_unit(&unit_bytes)
                        .map(|header| (header.filesystem_identifier, header.write_order.instance))
                })
                .ok()?
        }
    };
    (filesystem_identifier_in_the_header == filesystem_identifier_in_unit_headers)
        .then_some(instance)
}

/// [`placements_in_the_accounts_of_newer_readable_roots`] 交回的两样。
struct PlacementsInTheAccountsOfNewerReadableRoots {
    /// 较新可读根的账里的落点，按那条根的 txg 在根环里对应的槽（[`target_for_publish`]）归组。
    by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
    /// 账读不出而记录齐、那一槽改交现行账里仍分配着的落点（丙）的那几条根的 txg，按根环里读出的次序：调用方把所选那一版与它之间
    /// 那几次发布在 C 里记着的落点并进它那一槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）。
    txgs_of_the_roots_taking_the_current_account: Vec<CheckpointTxg>,
}

/// 丙并中间（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节改法表第一行 `chain`，判决
/// `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 攻方一形；第 10z 批第十块实现员定形，被攻过零轮）：
/// 走了丙那一支的根 R（`txgs_of_the_roots_taking_the_current_account`）与所选那一版之间还有别的发布 Q 时，R 从 Q 继承的单元既不是 R 写的、
/// 也不在所选那一版的账里，只记在 C 里 Q 那一槽；这个进程写掉 Q 那一槽时那一份解开，R 那一槽要晚几个 txg 才写掉，其间它们发得出去。
/// 这里把 (所选那一版, R) 之间每个 txg 在根环里对应的槽在 C 里记着的落点（`named_by_ring_slot`，C 的三份并好之后那一份）并进 R 那一槽，
/// 已在 R 那一槽里的（盘、起点槽、跨度都同）不再并：它们就随 R 那一槽一起解开。中间的 txg 至多取一整圈（环长 3S 个），
/// 两条根相差超过一圈时环里每一槽都已取到。只算内存，不读盘。
fn with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
    named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
    selected_version_txg: CheckpointTxg,
    txgs_of_the_roots_taking_the_current_account: &[CheckpointTxg],
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
    let ring_length_in_txgs =
        usize::try_from(ROOT_RING_REGIONS * slots_per_region.count()).expect("环长 3S ≤ 48");
    let mut chained = named_by_ring_slot.clone();
    for newer_root_txg in txgs_of_the_roots_taking_the_current_account {
        let ring_slot_of_the_newer_root = target_for_publish(*newer_root_txg, slots_per_region);
        let ring_slots_in_between: BTreeSet<RootRingSlot> = (selected_version_txg.0 + 1
            ..newer_root_txg.0)
            .rev()
            .take(ring_length_in_txgs)
            .map(|txg_in_between| {
                target_for_publish(CheckpointTxg(txg_in_between), slots_per_region)
            })
            .filter(|ring_slot| *ring_slot != ring_slot_of_the_newer_root)
            .collect();
        let on_the_ring_slot_of_the_newer_root =
            chained.entry(ring_slot_of_the_newer_root).or_default();
        let mut already_there: BTreeSet<(DeviceIdentity, SlotNumber, u64)> =
            on_the_ring_slot_of_the_newer_root
                .iter()
                .map(|placement| (placement.device, placement.slot, placement.span))
                .collect();
        for placement in ring_slots_in_between
            .iter()
            .filter_map(|ring_slot| named_by_ring_slot.get(ring_slot))
            .flatten()
        {
            if already_there.insert((placement.device, placement.slot, placement.span)) {
                on_the_ring_slot_of_the_newer_root.push(*placement);
            }
        }
    }
    chained
}

/// 回退复活（判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 正推一形，用户 2026-10-08 定「丙继续补洞」；
/// 第 10z 批第十块实现员定形，被攻过零轮）：走了丙那一支的根 `newer_root` 是一次管理员回退（向前发布）时，它从回退目标 R_old 取用户可见树，
/// R_old 引用、现行账记成已释放的单元被它复活；回退发布不写它们，记录不点名，现行账里又是已释放，丙那一截滤掉了它们。
/// 认回退照那次发布读得出的记录点名的打包单元（类标签 3）分三种结局（[`rollback_recognized_among_the_packed_units_named_by_a_publish`]）：
/// - 是回退：解开的谱系片（打包记录类型 5）里有一条 rollback_txg 等于这条根 txg 的谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条：
///   回退那次发布追加一条 (T_rb, T_old)），T_old 就是回退目标的 txg。根环里读出、按最新那条根的实例表有效、txg 等于 T_old 的根就是 R_old，
///   读它的账（影子账同一份读法 [`placements_referenced_by_root`]）。交回 R_old 账里仍分配着、现行账里记成已释放的落点（按盘与起点槽对上）：
///   只护复活的那一截，不并现行账里别的已释放落点（那是攻方乙那一类，第四块判决第二节 K2-2 改法行：384 槽盘上隔离约 370 个槽、600 段里 588 段被拒）。
/// - 肯定不是回退：点名的打包单元全读得出、解得开，里面没有这条谱系记录：交空，一槽不多。
/// - 认不出时宽护（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；
///   判法照正推 tri 臂 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`，细节第 10z 批第十二块实现员定，被攻过零轮）：
///   没认出回退而点名的打包单元有读不出、解不开的（不知道是不是回退；删除发布的墓碑片读不出也走到这里），或认出回退而根环里没读出 R_old
///   （它的根槽读不出，或它已不在环里）、R_old 的账读不出（是回退、复活集算不出），都交回现行账里全部已释放的落点（正推的对照臂那一截）。
///   宽护那一截与别的 C 落点同一个时机解开（这个进程写掉这条根那一槽时）。
///
/// 只读盘，不改分配器。
fn placements_a_rollback_publish_revived_released_in_the_current_account<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    newer_root: &RootRecord,
    readable_records_of_its_publish: &[&crate::journal::JournalRecord],
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    current_records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let all_released_in_the_current_account = || -> Vec<(DeviceIdentity, SlotNumber, u64)> {
        current_records
            .iter()
            .filter(|record| record.is_released)
            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
            .collect()
    };
    let rollback_target_txg = match rollback_recognized_among_the_packed_units_named_by_a_publish(
        newer_root.checkpoint_txg,
        &packed_units_named_by_the_records_of_a_publish_as_read(
            devices,
            readable_records_of_its_publish,
        ),
    ) {
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(rollback_target_txg) => {
            rollback_target_txg
        }
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback => return Vec::new(),
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback => {
            return all_released_in_the_current_account();
        }
    };
    let accounts_of_the_rollback_target: Option<Vec<Vec<(DeviceIdentity, SlotNumber, u64)>>> =
        roots
            .iter()
            .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)
            .map(|rollback_target| {
                placements_referenced_by_root(devices, unit_area_start, rollback_target)
            })
            .collect();
    match accounts_of_the_rollback_target {
        Some(accounts) if !accounts.is_empty() => {
            let released_in_the_current_account: BTreeSet<(DeviceIdentity, SlotNumber)> =
                current_records
                    .iter()
                    .filter(|record| record.is_released)
                    .map(|record| (record.device, record.slot))
                    .collect();
            accounts
                .into_iter()
                .flatten()
                .filter(|(device, slot, _)| {
                    released_in_the_current_account.contains(&(*device, *slot))
                })
                .collect()
        }
        // 根环里没读出 R_old（它的根槽读不出，或它已不在环里），或 R_old 的账读不出：是回退、复活集算不出（tri 的第二种退法）。
        Some(_) | None => all_released_in_the_current_account(),
    }
}

/// 一次发布读得出的记录点名的一个打包单元（类标签 3）读成了什么。
enum PackedUnitNamedByAPublishAsRead {
    /// 两条位置条目都读不出，或读出来整单元校验和对不上条目里的。
    UnreadableOrChecksumMismatch,
    /// 读出来校验和对得上，`parse_packed_unit` 解不开。
    ReadButUnparsable,
    /// 解开了：打包记录类型与逐条记录的字节。
    Parsed {
        record_type: u16,
        records: Vec<Vec<u8>>,
    },
}

/// 一次发布读得出的记录点名的每个打包单元（类标签 3），按记录与点名的次序，经位置条目里的校验和读、解。只读盘。
fn packed_units_named_by_the_records_of_a_publish_as_read<Reader: PoolReader>(
    devices: &Reader,
    readable_records_of_the_publish: &[&crate::journal::JournalRecord],
) -> Vec<PackedUnitNamedByAPublishAsRead> {
    let packed_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    readable_records_of_the_publish
        .iter()
        .flat_map(|record| record.named.iter())
        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)
        .map(
            |named| match read_unit_via_locations(devices, &named.locations, packed_unit_bytes) {
                Err(_unreadable) => PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
                Ok(bytes) => match crate::unit::parse_packed_unit(&bytes) {
                    Err(_unparsable) => PackedUnitNamedByAPublishAsRead::ReadButUnparsable,
                    Ok(packed) => PackedUnitNamedByAPublishAsRead::Parsed {
                        record_type: packed.identity.record_type,
                        records: packed.records,
                    },
                },
            },
        )
        .collect()
}

/// 认回退的三种结局（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节）。
#[derive(Debug, PartialEq, Eq)]
enum RollbackRecognizedAmongThePackedUnitsNamedByAPublish {
    /// 解开的谱系片（打包记录类型 5）里有一条 rollback_txg 等于这次发布 txg 的谱系记录：这次发布是一次管理员回退，回到 T_old 那一版。
    RollbackTo(CheckpointTxg),
    /// 点名的打包单元全读得出、解得开，谱系片里的记录也全解得开，没有那一条：肯定不是回退。
    NotARollback,
    /// 没认出回退，而点名的打包单元有读不出、校验和对不上、解不开的，或谱系片里有一条记录解不开：不知道是不是回退。
    UnknownWhetherARollback,
}

/// 照一次发布的记录点名的打包单元读成的样子认回退（D18（块里携带什么信息） 已定项 24 第 5 条：回退那次发布追加一条 (T_rb, T_old)，
/// 装在它写的谱系片里、记录点名）。只算内存。读出来的里面认出了就是回退，不管别的几片读不读得出；没认出时，有一片读不出、解不开就是不知道。
fn rollback_recognized_among_the_packed_units_named_by_a_publish(
    checkpoint_txg_of_the_publish: CheckpointTxg,
    packed_units_as_read: &[PackedUnitNamedByAPublishAsRead],
) -> RollbackRecognizedAmongThePackedUnitsNamedByAPublish {
    let mut some_named_packed_unit_unknown = false;
    for packed_unit in packed_units_as_read {
        match packed_unit {
            PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch
            | PackedUnitNamedByAPublishAsRead::ReadButUnparsable => {
                some_named_packed_unit_unknown = true;
            }
            PackedUnitNamedByAPublishAsRead::Parsed {
                record_type,
                records,
            } => {
                if *record_type != crate::unit::PACKED_TYPE_LINEAGE {
                    continue;
                }
                for record_bytes in records {
                    match LineageRecord::parse(record_bytes) {
                        Some(lineage) if lineage.rollback_txg == checkpoint_txg_of_the_publish => {
                            return RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(
                                lineage.rollback_target_txg,
                            );
                        }
                        Some(_lineage_of_another_publish) => {}
                        None => some_named_packed_unit_unknown = true,
                    }
                }
            }
        }
    }
    if some_named_packed_unit_unknown {
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback
    } else {
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback
    }
}

/// 一份账里仍分配着的每个落点（盘、起点槽、跨度），按记录的次序。只读内存。
fn placements_still_allocated_in_the_account(
    records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
        .collect()
}

/// 水位之上一次没施加的发布（或比所选那一版新的可读根）的账读不出、那次发布读得出的记录又不齐时，它引用的落点照被抛弃根那一份办
/// （`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条）：
/// 从中央映射现算、重读一次、看 C694 的 B 那道凭据（[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
/// 与被抛弃根那一份共用）。
/// 记录齐的那几次发布不走这里（调用方判）：那时账读不出交 F1 乙，照被抛弃根那一份办会在单元全读不出、映射也现算不成的正常盘面上误拒。
///
/// # Errors
/// 映射现算不成、重读一次仍不成、凭据也没有 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`，
/// `unapplied_publish` 点名 `root_of_the_publish` 的 (实例代号, checkpoint_txg)；与 C393 那一拒分开报（用户 2026-10-08 定「乙：core 另起错误成员」），
/// 在任何写之前。
fn placements_of_an_unapplied_publish_whose_account_is_unreadable<Reader: PoolReader>(
    devices: &Reader,
    unit_area_start: UnitAreaStart,
    root_of_the_publish: &RootRecord,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
    placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread(
        devices,
        unit_area_start,
        root_of_the_publish,
        current_records,
        before_the_one_reread,
    )
    .ok_or_else(|| {
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
            StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                unapplied_publish: RollbackTarget {
                    instance: root_of_the_publish.instance,
                    checkpoint_txg: root_of_the_publish.checkpoint_txg,
                },
            },
        ))
    })
}

/// C 并的较新可读根（`C_NEWER_ROOTS`，[`placements_in_the_accounts_of_newer_readable_roots`]）从哪几条根里挑：重建分配器读根环那一遍读出的根
/// `roots_of_the_rebuild_pass`，并上读阶段择根那一遍读出、自证过的全部根 `roots_read_by_the_read_stage`（读阶段择到的那条根在内，它是判退之前的所选根），
/// 已有同一 (实例代号, checkpoint_txg) 的就不再并（这一遍里有的，与读阶段里先并进来的）。
/// 判退那一形里读阶段择到的根 R 读得出、走读失败、退到 P：R 比 P 新、按实例表有效，在 I-7.4 的回退候选集里；R 与 P 之间读阶段读得出的根 Q 同样在里面
/// （判退那两问读 Q 的根槽读坏时退过了它）。这一遍读 R 或 Q 的根槽读坏（一次瞬时读坏就够）时这一遍的读数里没有它，不并它 C 就看不见它，
/// 它的记录又读不出时它的单元一个都不护，写行那次发布照样写掉。并进来的是读阶段读到、自证过的那一份根记录，
/// 它的账照 C 那一份读法读：读不出、记录不齐、现算不成、重读一次仍不成又没有凭据就拒（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]），
/// 与这一遍读得出它时同一个结局。比所选那一版旧（没判退时读阶段读到的根都不新于所选根）或按实例表被抛弃的，由那个函数自己的筛滤掉。只拷内存，不读盘。
/// 影子账、根环表与回收门槛不用这一份：它们说的是这一遍根环槽里读到了什么。
pub(super) fn roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage(
    roots_of_the_rebuild_pass: &[RootRecord],
    roots_read_by_the_read_stage: &[RootRecord],
) -> Vec<RootRecord> {
    let mut roots = roots_of_the_rebuild_pass.to_vec();
    for root_read_by_the_read_stage in roots_read_by_the_read_stage {
        let key_of_the_root_read_by_the_read_stage = (
            root_read_by_the_read_stage.instance,
            root_read_by_the_read_stage.checkpoint_txg,
        );
        if !roots.iter().any(|root| {
            (root.instance, root.checkpoint_txg) == key_of_the_root_read_by_the_read_stage
        }) {
            roots.push(*root_read_by_the_read_stage);
        }
    }
    roots
}

/// 影子账同一次隔离、豁免同一个集合的那几份落点，按根环槽归组（C694 的 C）：水位之上没施加的记录点名的落点
/// （[`placements_named_by_unapplied_journal_records`]），并上那几次没施加的发布的账里的落点（C 的账那一份，
/// [`placements_in_the_accounts_of_unapplied_publishes`]；账读不出、读得出的记录不齐时从中央映射现算，仍不成就拒），
/// 再并上环里比所选那一版新、按实例表有效、读得出的根自己的账（`C_NEWER_ROOTS`，[`placements_in_the_accounts_of_newer_readable_roots`]：
/// F2 退回之后被退掉的那条根，影子账不管它，它的记录又读不出时前两份都是空的），解开时机相同；三份都并好之后，走了丙那一支的根那一槽
/// 再并上它与所选那一版之间那几槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）。
///
/// # Errors
/// 那两份账读不出、从中央映射也现算不成、重读一次仍不成（同那两个函数）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：读者、系统配置、单元区起点、读阶段扫出的记录、所选那一版、读出来的根、判被抛弃、现行账、重读前的钩子"
)]
pub(super) fn placements_of_unapplied_publishes_and_newer_readable_roots<Reader: PoolReader>(
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    unit_area_start: UnitAreaStart,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>, MountError> {
    let slots_per_region = system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
    let mut named_by_ring_slot = placements_named_by_unapplied_journal_records(
        devices,
        unit_area_start,
        records,
        selected_version,
        slots_per_region,
    );
    for (ring_slot, placements_in_the_account) in placements_in_the_accounts_of_unapplied_publishes(
        devices,
        unit_area_start,
        records,
        selected_version,
        slots_per_region,
        current_records,
        before_the_one_reread,
    )? {
        named_by_ring_slot
            .entry(ring_slot)
            .or_default()
            .extend(placements_in_the_account);
    }
    let PlacementsInTheAccountsOfNewerReadableRoots {
        by_ring_slot: in_the_accounts_of_newer_roots_by_ring_slot,
        txgs_of_the_roots_taking_the_current_account,
    } = placements_in_the_accounts_of_newer_readable_roots(
        devices,
        unit_area_start,
        crate::unit::unit_filesystem_identifier(
            &system_configuration.immutable.filesystem_identifier,
        ),
        roots,
        is_abandoned,
        records,
        selected_version,
        slots_per_region,
        current_records,
        before_the_one_reread,
    )?;
    for (ring_slot, placements_in_the_newer_roots_account) in
        in_the_accounts_of_newer_roots_by_ring_slot
    {
        named_by_ring_slot
            .entry(ring_slot)
            .or_default()
            .extend(placements_in_the_newer_roots_account);
    }
    Ok(
        with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
            named_by_ring_slot,
            selected_version.checkpoint_txg,
            &txgs_of_the_roots_taking_the_current_account,
            slots_per_region,
        ),
    )
}

#[cfg(test)]
mod placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests {
    use super::with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots;
    use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
    use crate::allocator::PlacementOnDevice;
    use crate::root_ring::{target_for_publish, RootRingSlotsPerRegion};
    use std::collections::BTreeMap;

    fn placement(slot: u64) -> PlacementOnDevice {
        PlacementOnDevice {
            device: DeviceIdentity(0),
            slot: SlotNumber(slot),
            span: 2,
        }
    }

    /// 所选那一版 txg 6，走了丙那一支的根 R txg 9，其间 Q（txg 7）、txg 8 两槽在 C 里各记着落点，R 之后的 txg 10 那一槽也记着：
    /// R 那一槽并上 txg 7、8 两槽记着的、R 那一槽里还没有的落点（R 那一槽已有的不再并一次），txg 10 那一槽的不并；别的槽原样。
    #[test]
    fn the_ring_slot_of_a_root_taking_the_current_account_takes_what_the_publishes_in_between_named(
    ) {
        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
        let named_by_ring_slot = BTreeMap::from([
            (ring_slot_of(7), vec![placement(100), placement(102)]),
            (ring_slot_of(8), vec![placement(104), placement(200)]),
            (ring_slot_of(9), vec![placement(200)]),
            (ring_slot_of(10), vec![placement(300)]),
        ]);
        let chained = with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
            named_by_ring_slot.clone(),
            CheckpointTxg(6),
            &[CheckpointTxg(9)],
            slots_per_region,
        );
        assert_eq!(
            chained[&ring_slot_of(9)],
            vec![
                placement(200),
                placement(100),
                placement(102),
                placement(104)
            ],
            "R 那一槽：原有的在前，再按槽的次序并上 txg 7、8 两槽的，已有的 200 不再并"
        );
        for txg in [7, 8, 10] {
            assert_eq!(
                chained[&ring_slot_of(txg)],
                named_by_ring_slot[&ring_slot_of(txg)],
                "txg {txg} 那一槽原样（中间那几槽照旧在自己的 txg 上解开）"
            );
        }
        assert_eq!(chained.len(), named_by_ring_slot.len(), "不新开别的槽");
    }

    /// 走了丙那一支的根紧挨着所选那一版（其间没有别的发布）：什么都不并，原样交回。
    #[test]
    fn root_taking_the_current_account_right_after_the_selected_version_takes_nothing_more() {
        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
        let named_by_ring_slot = BTreeMap::from([
            (ring_slot_of(6), vec![placement(100)]),
            (ring_slot_of(7), vec![placement(200)]),
        ]);
        assert_eq!(
            with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
                named_by_ring_slot.clone(),
                CheckpointTxg(6),
                &[CheckpointTxg(7)],
                slots_per_region,
            ),
            named_by_ring_slot,
            "(6, 7) 之间没有 txg：原样"
        );
    }

    /// 两条走了丙那一支的根各并各自与所选那一版之间那几槽（正推 K5 的 E7：只并最后一条照绿）。交进来的 txg 按根环里读出的次序，
    /// 那不是 txg 的次序：S = 8 时 txg 24 在区域 0 槽 0、txg 23 在区域 2 槽 7，较新的 24 排在前面。所选那一版 txg 21，其间 Q（txg 22）。
    /// 24 那一槽并 22、23 两槽的，23 那一槽并 22 那一槽的；只并其中一条时另一条那一槽缺它的中间那几槽
    /// （只并 23 时 24 那一槽缺 23 写的 200：23 那一槽先于 24 那一槽写掉，其间 24 还在环里、引用着 200）。
    #[test]
    fn every_root_taking_the_current_account_takes_what_the_publishes_between_it_and_the_selected_version_named(
    ) {
        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
        assert!(
            ring_slot_of(24) < ring_slot_of(23),
            "根环里读出的次序是区域、槽：txg 24 排在 txg 23 前面（这条用例要的那一形）"
        );
        let named_by_ring_slot = BTreeMap::from([
            (ring_slot_of(22), vec![placement(100)]),
            (ring_slot_of(23), vec![placement(200)]),
            (ring_slot_of(24), vec![placement(300)]),
        ]);
        let chained = with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
            named_by_ring_slot.clone(),
            CheckpointTxg(21),
            &[CheckpointTxg(24), CheckpointTxg(23)],
            slots_per_region,
        );
        assert_eq!(
            chained[&ring_slot_of(24)],
            vec![placement(300), placement(100), placement(200)],
            "txg 24 那一槽：原有的在前，再并上 22、23 两槽的"
        );
        assert_eq!(
            chained[&ring_slot_of(23)],
            vec![placement(200), placement(100)],
            "txg 23 那一槽：原有的在前，再并上 22 那一槽的"
        );
        assert_eq!(
            chained[&ring_slot_of(22)],
            named_by_ring_slot[&ring_slot_of(22)],
            "txg 22 那一槽原样"
        );
    }
}

/// 认回退的三种结局（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」）：只算内存的那一半
/// （[`super::rollback_recognized_among_the_packed_units_named_by_a_publish`]）。读盘那一半与落到 C 里的那一截由 验证包
/// `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 里回退复活那几条用例判。
#[cfg(test)]
mod rollback_recognized_among_the_packed_units_named_by_a_publish_tests {
    use super::{
        rollback_recognized_among_the_packed_units_named_by_a_publish,
        PackedUnitNamedByAPublishAsRead, RollbackRecognizedAmongThePackedUnitsNamedByAPublish,
    };
    use crate::address::CheckpointTxg;
    use crate::records::LineageRecord;
    use crate::unit::{PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE};

    /// 这几条用例里那次发布的 txg。
    const CHECKPOINT_TXG_OF_THE_PUBLISH: CheckpointTxg = CheckpointTxg(9);

    fn lineage_record_bytes(rollback_txg: u64, rollback_target_txg: u64) -> Vec<u8> {
        LineageRecord {
            rollback_txg: CheckpointTxg(rollback_txg),
            rollback_target_txg: CheckpointTxg(rollback_target_txg),
            inode_number_watermark_of_the_version_rolled_back_from: 3,
        }
        .to_bytes()
    }

    fn parsed(record_type: u16, records: Vec<Vec<u8>>) -> PackedUnitNamedByAPublishAsRead {
        PackedUnitNamedByAPublishAsRead::Parsed {
            record_type,
            records,
        }
    }

    /// 谱系片里有 rollback_txg 等于这次发布 txg 的那一条：是回退，回到它的 T_old；同一片里别的发布的谱系记录不碍事。
    #[test]
    fn lineage_record_of_this_publish_recognizes_a_rollback_to_its_target() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
                    parsed(
                        PACKED_TYPE_LINEAGE,
                        vec![lineage_record_bytes(7, 3), lineage_record_bytes(9, 5)]
                    ),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(CheckpointTxg(5)),
        );
    }

    /// 点名的打包单元全解得开，谱系片里只有别的发布（txg 7）的谱系记录：肯定不是回退（正推 K5 的 E2：不比 rollback_txg 时认成回到 3）。
    #[test]
    fn lineage_records_of_other_publishes_only_are_not_a_rollback() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(7, 3)]),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
        );
    }

    /// 不是谱系片的打包单元里的记录不当谱系记录读：一片 inode 叶里的 24 字节记录恰好拼得出 (9, 5, 3)，也不认成回退。
    #[test]
    fn records_of_a_packed_unit_that_is_not_a_lineage_unit_are_not_read_as_lineage_records() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[parsed(PACKED_TYPE_INODE, vec![lineage_record_bytes(9, 5)])],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
        );
    }

    /// 点名的打包单元有一片读不出（或校验和对不上），读得出的那几片里没认出：不知道是不是回退。
    #[test]
    fn an_unreadable_named_packed_unit_without_a_recognized_rollback_is_unknown() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
                    PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
        );
    }

    /// 点名的打包单元有一片读得出、解不开：同样不知道。
    #[test]
    fn an_unparsable_named_packed_unit_without_a_recognized_rollback_is_unknown() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    PackedUnitNamedByAPublishAsRead::ReadButUnparsable,
                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(7, 3)]),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
        );
    }

    /// 谱系片解开了、里面有一条记录解不开（长度不是 24），别的记录都不是这次发布的：同样不知道。
    #[test]
    fn an_unparsable_lineage_record_without_a_recognized_rollback_is_unknown() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[parsed(
                    PACKED_TYPE_LINEAGE,
                    vec![lineage_record_bytes(7, 3), vec![0; 23]]
                )],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
        );
    }

    /// 读得出的那几片里认出了回退，就是回退：另一片读不出不改结局（回退那条谱系记录只这一次发布写）。
    #[test]
    fn recognized_rollback_stands_though_another_named_packed_unit_is_unreadable() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(9, 5)]),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(CheckpointTxg(5)),
        );
    }

    /// 一片打包单元都没点名（读得出的记录里没有类标签 3 的）：肯定不是回退（回退那次发布必定写谱系片）。
    #[test]
    fn no_named_packed_unit_is_not_a_rollback() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
        );
    }
}

#[cfg(test)]
mod packed_units_named_by_the_records_of_a_publish_as_read_tests {
    use super::{
        packed_units_named_by_the_records_of_a_publish_as_read, PackedUnitNamedByAPublishAsRead,
    };
    use crate::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
        TreeIdentifier,
    };
    use crate::block_device::{
        BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
    };
    use crate::checksum::crc32_castagnoli;
    use crate::journal::{
        JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnit,
    };
    use crate::pointer::{BirthSequence, LocationEntry, NodePointer};
    use crate::records::LineageRecord;
    use crate::root_record::UnmountMarker;
    use crate::unit::{
        build_packed_unit, PackedIdentity, WriteOrder, PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE,
        UNIT_CLASS_DATA, UNIT_CLASS_PACKED,
    };
    use singlefs_format::{DATA_UNIT_BYTES, LINEAGE_RECORD_BYTES, SLOT_BYTES};

    /// 这几条用例里那块盘的槽数：四个打包单元各占两槽，再留几槽。
    const SLOTS_OF_THE_DEVICE: u64 = 16;

    /// 这几条用例里点名的单元出生的那一代、那次发布的 txg。
    const CHECKPOINT_TXG_OF_THE_PUBLISH: CheckpointTxg = CheckpointTxg(9);

    /// 一块只在内存里的盘：读按偏移切字节，越过末尾报越界。
    struct DeviceOfBytesInMemory {
        bytes: Vec<u8>,
    }

    impl DeviceOfBytesInMemory {
        fn range_of(
            &self,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Result<std::ops::Range<usize>, BlockDeviceError> {
            let start = usize::try_from(offset.0).expect("用例里的偏移装得进 usize");
            if start + length > self.bytes.len() {
                return Err(BlockDeviceError::OutOfRange {
                    offset,
                    length: u64::try_from(length).expect("长度装得进 u64"),
                    device_size: self.size_in_bytes(),
                });
            }
            Ok(start..start + length)
        }
    }

    impl BlockDevice for DeviceOfBytesInMemory {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, buffer.len())?;
            buffer.copy_from_slice(&self.bytes[range]);
            Ok(())
        }
        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, bytes.len())?;
            self.bytes[range].copy_from_slice(bytes);
            Ok(())
        }
        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            let range =
                self.range_of(offset, usize::try_from(length).expect("长度装得进 usize"))?;
            self.bytes[range].fill(0);
            Ok(())
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            PhysicalBlockSizeInBytes(512)
        }
        fn size_in_bytes(&self) -> u64 {
            u64::try_from(self.bytes.len()).expect("盘宽装得进 u64")
        }
    }

    fn packed_unit_of(record_type: u16, record_width: u16, records: &[Vec<u8>]) -> Vec<u8> {
        build_packed_unit(
            PackedIdentity {
                birth_tree: TreeIdentifier(65_536),
                record_type,
                container: 0,
                container_birth: CHECKPOINT_TXG_OF_THE_PUBLISH,
            },
            record_width,
            records,
            CHECKPOINT_TXG_OF_THE_PUBLISH,
            &[1; 16],
            WriteOrder {
                instance: InstanceGeneration(1),
                transaction: 1,
            },
            BirthSequence(0),
        )
    }

    /// 点名 `slot` 上那个单元的点名项，两条位置条目都在盘 0 的这一槽、校验和是 `unit_checksum`。
    fn named_unit_at(slot: u64, unit_class: u8, unit_checksum: u32) -> NamedUnit {
        let location = LocationEntry {
            device: DeviceIdentity(0),
            slot: SlotNumber(slot),
            unit_checksum,
        };
        NamedUnit {
            locations: [location, location],
            unit_class,
            birth_tree: TreeIdentifier(65_536),
            birth_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
            key_tail: [0; 10],
        }
    }

    fn record_naming(ordinal_from_one: u32, named: Vec<NamedUnit>) -> JournalRecord {
        JournalRecord {
            instance: InstanceGeneration(1),
            counter: u64::from(ordinal_from_one),
            checkpoint_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
            transaction: u64::from(ordinal_from_one),
            is_commit: true,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal_from_one),
            place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
            back_chain: 0,
            filesystem_identifier: 0,
            new_tree_table: NodePointer::empty_root(),
            new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: 0,
            new_rollback_floor: CheckpointTxg(0),
            new_instance_table: NodePointer::empty_root(),
            new_allocation_record_tree_root: NodePointer::empty_root(),
            new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            new_reclaim_threshold: CheckpointTxg(0),
            new_pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
            named,
        }
    }

    /// 读成的样子写成一行字（用例只比这一行，被测的类型不为用例另加比较）。
    fn description_of(packed_unit: &PackedUnitNamedByAPublishAsRead) -> String {
        match packed_unit {
            PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch => {
                "读不出或校验和对不上".to_string()
            }
            PackedUnitNamedByAPublishAsRead::ReadButUnparsable => "读得出、解不开".to_string(),
            PackedUnitNamedByAPublishAsRead::Parsed {
                record_type,
                records,
            } => format!("解开了：类型 {record_type}，记录 {records:?}"),
        }
    }

    /// 读盘一层（正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 2 节 X4、X5、X18）：一次发布两条记录，第一条点名一片 inode 叶容器、
    /// 一个数据单元与一片读得出而解不开的打包单元（整单元校验和对得上条目），第二条点名一片校验和对不上条目的 inode 叶容器与一片谱系片。
    /// 交回逐条记录、逐个点名的每一片打包单元（数据单元不算），次序照记录与点名的次序，四种结局各认得出：只看第一条记录、只看第一片、
    /// 解不开的当解开了（旧写法 `.ok()` 吞掉）、不筛类标签、记录类型或记录读丢了，这一句都红。
    #[test]
    fn every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart(
    ) {
        let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
        let slots_per_unit = DATA_UNIT_BYTES / SLOT_BYTES;
        let inode_record = vec![7u8; 16];
        let inode_unit = packed_unit_of(PACKED_TYPE_INODE, 16, std::slice::from_ref(&inode_record));
        let lineage_record = LineageRecord {
            rollback_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
            rollback_target_txg: CheckpointTxg(5),
            inode_number_watermark_of_the_version_rolled_back_from: 3,
        }
        .to_bytes();
        let lineage_unit = packed_unit_of(
            PACKED_TYPE_LINEAGE,
            u16::try_from(LINEAGE_RECORD_BYTES).expect("24"),
            std::slice::from_ref(&lineage_record),
        );
        let unparsable_unit = vec![0xA5u8; unit_bytes];
        assert!(
            crate::unit::parse_packed_unit(&unparsable_unit).is_err(),
            "那一片解不开（阳性对照）"
        );
        let slot_of_the_inode_unit = 0;
        let slot_of_the_unparsable_unit = slots_per_unit;
        let slot_of_the_mismatching_unit = 2 * slots_per_unit;
        let slot_of_the_lineage_unit = 3 * slots_per_unit;
        let slot_of_the_data_unit = 4 * slots_per_unit;
        let mut device = DeviceOfBytesInMemory {
            bytes: vec![0; usize::try_from(SLOTS_OF_THE_DEVICE * SLOT_BYTES).expect("盘宽")],
        };
        for (slot, bytes) in [
            (slot_of_the_inode_unit, &inode_unit),
            (slot_of_the_unparsable_unit, &unparsable_unit),
            (slot_of_the_mismatching_unit, &inode_unit),
            (slot_of_the_lineage_unit, &lineage_unit),
        ] {
            device
                .write_at(
                    SlotNumber(slot).to_device_offset(),
                    bytes,
                    WriteDurability::Plain,
                )
                .expect("写在盘宽之内");
        }
        let devices = vec![(DeviceIdentity(0), device)];
        let first_record = record_naming(
            1,
            vec![
                named_unit_at(
                    slot_of_the_inode_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&inode_unit),
                ),
                named_unit_at(slot_of_the_data_unit, UNIT_CLASS_DATA, 0),
                named_unit_at(
                    slot_of_the_unparsable_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&unparsable_unit),
                ),
            ],
        );
        let second_record = record_naming(
            2,
            vec![
                named_unit_at(
                    slot_of_the_mismatching_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&inode_unit) ^ 1,
                ),
                named_unit_at(
                    slot_of_the_lineage_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&lineage_unit),
                ),
            ],
        );
        let descriptions: Vec<String> = packed_units_named_by_the_records_of_a_publish_as_read(
            &devices,
            &[&first_record, &second_record],
        )
        .iter()
        .map(description_of)
        .collect();
        assert_eq!(
            descriptions,
            vec![
                description_of(&PackedUnitNamedByAPublishAsRead::Parsed {
                    record_type: PACKED_TYPE_INODE,
                    records: vec![inode_record],
                }),
                description_of(&PackedUnitNamedByAPublishAsRead::ReadButUnparsable),
                description_of(&PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch),
                description_of(&PackedUnitNamedByAPublishAsRead::Parsed {
                    record_type: PACKED_TYPE_LINEAGE,
                    records: vec![lineage_record],
                }),
            ],
            "两条记录点名的四片打包单元逐个读成各自的样子、次序照记录与点名"
        );
    }
}
