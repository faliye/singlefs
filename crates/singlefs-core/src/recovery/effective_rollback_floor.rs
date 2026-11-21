//! 生效的回退下界 F：按实例表判根与发布被不被抛弃，按最新的根那张表算 F，读不出时重读一次的几种算法。

use super::*;

/// 一条根按一张实例表判是不是被抛弃的：表里有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti（D23（journal 的角色与格式） 已定项 14
/// 候选集那一格「(i, T) 可选 ⟺ 表里无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti」反过来）。回退候选集、影子账、抬 F 的上限与
/// F 生效值里「有效根」那一半都按它判，一处定义（判法在 [`publish_is_abandoned_by_the_instance_table`]）。
#[must_use]
pub fn root_is_abandoned_by_the_instance_table(
    root: &RootRecord,
    table: &InstanceTableRecords,
) -> bool {
    publish_is_abandoned_by_the_instance_table(root.instance, root.checkpoint_txg, table)
}

/// 一次发布 (实例代号 `instance`, `checkpoint_txg`) 按一张实例表判是不是在被抛弃的时间线上：表里有 `instance` 的行 (i, Ti, Wi) 且
/// `checkpoint_txg` > Ti。[`root_is_abandoned_by_the_instance_table`] 拿根记录的这两样判；F2 判「退过去会不会跳过一次根读不出的发布」时
/// 拿那次发布带末条标志的记录的这两样判（那次发布的根读不出，没有根记录可拿）。
#[must_use]
pub fn publish_is_abandoned_by_the_instance_table(
    instance: InstanceGeneration,
    checkpoint_txg: CheckpointTxg,
    table: &InstanceTableRecords,
) -> bool {
    table
        .rows
        .iter()
        .any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)
}

/// 生效的回退下界 F（D16（发布语义） 已定项 1「生效」那一行，SysPre）：F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值,
/// 系统配置里读得出的 F 的最大值)。
///
/// - 根那一半：根环里自证过的根，按根环落点公式归到它那块盘（区域归属表），每块盘取按实例表判仍然有效（[`root_is_abandoned_by_the_instance_table`]
///   判不是被抛弃的）的根里 (txg, 实例代号) 最大的那一条，取它带的 F；各盘取最大。判有效用的实例表是根环里 (txg, 实例代号) 最大那条根指着的
///   那一张（恢复择的就是它，`choose_root`）；那张表读不出、解不开时不按表滤，每块盘取读得出的根里最新的那条。
///   被抛弃时间线上的根带的 F 也算进来（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」）：根环里按那张表判被抛弃的根，取它们带的 F 的最大值，与各盘那一份取大；那张表读不出时没有「被抛弃」可判，照旧。
/// - 系统配置那一半：池里每块盘两槽里全部自证过（整槽校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）
///   的槽带的 F 的最大值。
///
/// 两处一条都读不出时 0（mkfs 写的就是 0）。
///
/// ⚠️ 读不出的根槽当没有根、那张表读不出时「不按表滤」是无声放过（C554 乙报告 Q6）：产品路径都不走这一份——
/// 可写挂载按重读过的那张表算（[`effective_rollback_floor_under_the_newest_roots_table`]），抬 F 走
/// [`effective_rollback_floor_rereading_the_newest_instance_table_once`]，管理员回退走
/// [`effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`]，写系统配置槽走
/// [`effective_rollback_floor_rereading_unreadable_reads_once`]。留给单外的实验装置与用例。
#[must_use]
pub fn effective_rollback_floor<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> CheckpointTxg {
    let roots_with_ring_slots = readable_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
    );
    let newest_roots_table = newest_root_among(&roots_with_ring_slots)
        .and_then(|newest| instance_table_chain_of_root(reader, &newest).ok())
        .map(|chain| chain.records);
    effective_rollback_floor_of_the_roots_read(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        &roots_with_ring_slots,
        newest_roots_table.as_ref(),
    )
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），判「有效根」用调用方交进来的那张表：调用方已经把根环里最新那条根的实例表
/// 读出来了（可写挂载重建分配器时读过、读不出重读过一次，`mount` 的 `instance_table_of_the_newest_root_read_at_most_twice`），
/// 不再读一遍、也就没有「读不出就不按表滤」那一支。
#[must_use]
pub fn effective_rollback_floor_under_the_newest_roots_table<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    newest_roots_table: &InstanceTableRecords,
) -> CheckpointTxg {
    let roots_with_ring_slots = readable_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
    );
    effective_rollback_floor_of_the_roots_read(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        &roots_with_ring_slots,
        Some(newest_roots_table),
    )
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），根环里最新那条根的实例表这一次读不出、解不开（或一条自证过的根都择不到）时
/// 立即重读一次根环与那张表（C554 乙的形态，R = 1；D16（发布语义） 已定项 1「重读一次」），仍是这样就报错，不按「不按表滤」往下走。
///
/// # Errors
/// 重读仍读不出 ⇒ [`InstanceTableOfTheNewestRootStillUnreadableAfterOneReread`]。只读盘、不写盘。
pub fn effective_rollback_floor_rereading_the_newest_instance_table_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Result<CheckpointTxg, InstanceTableOfTheNewestRootStillUnreadableAfterOneReread> {
    let read_the_ring = |pass| {
        Ok::<_, std::convert::Infallible>(readable_roots_with_ring_slots(
            &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
            region_devices,
            immutable_sizes,
            filesystem_identifier,
        ))
    };
    let Ok(floor) = effective_rollback_floor_rereading_the_newest_instance_table_once_over(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        read_the_ring,
    );
    match floor {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(floor) => Ok(floor),
        EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
            newest_root_on_the_reread,
            ..
        } => Err(InstanceTableOfTheNewestRootStillUnreadableAfterOneReread {
            newest_root_on_the_reread,
        }),
    }
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），管理员回退判候选用：读根环照
/// [`readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once`]（这个进程知道住着根的槽读不出就重读一次，
/// 别的读不出的槽当没有根，D16（发布语义） 已定项 1「根槽这一次读坏」那一行的分法），根环里最新那条根的实例表照
/// [`effective_rollback_floor_rereading_the_newest_instance_table_once`] 读不出就连根环一起重读一次。
/// 不按原 [`effective_rollback_floor`] 那样把读不出的槽当没有根、把读不出的表当不按表滤（那样 F 生效值可以算低，
/// 低于真 F 的目标被当成候选）。
///
/// # Errors
/// 知道住着根的槽重读仍坏 ⇒ [`EffectiveFloorReadingStillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`]；
/// 最新那条根的实例表重读仍读不出 ⇒ [`EffectiveFloorReadingStillUnreadableAfterOneReread::InstanceTableOfTheNewestRoot`]。只读盘、不写盘。
pub fn effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
) -> Result<CheckpointTxg, EffectiveFloorReadingStillUnreadableAfterOneReread> {
    let read_the_ring = |pass| {
        readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
            &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slots_known_to_hold_a_root,
        )
    };
    match effective_rollback_floor_rereading_the_newest_instance_table_once_over(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        read_the_ring,
    )
    .map_err(EffectiveFloorReadingStillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot)?
    {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(floor) => Ok(floor),
        EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
            newest_root_on_the_reread,
            ..
        } => Err(
            EffectiveFloorReadingStillUnreadableAfterOneReread::InstanceTableOfTheNewestRoot(
                InstanceTableOfTheNewestRootStillUnreadableAfterOneReread {
                    newest_root_on_the_reread,
                },
            ),
        ),
    }
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），给手里没有这个进程根环表、也没有错误可报的写者用
/// （`transaction::PoolWriter` 每次写系统配置槽之前算要写的 F）：读根环照
/// [`readable_roots_with_ring_slots_rereading_unreadable_slots_once`]（读不出的槽重读一次，仍读不出当没有根），根环里最新那条根的实例表
/// 读不出就连根环一起重读一次，仍读不出不按表滤（与原 [`effective_rollback_floor`] 同一个退路）。
/// 为什么这一处不拒：见 `transaction::PoolWriter::write_system_configuration_slot` 里那一段注释。只读盘、不写盘。
#[must_use]
pub fn effective_rollback_floor_rereading_unreadable_reads_once<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> CheckpointTxg {
    let read_the_ring = |pass| {
        Ok::<_, std::convert::Infallible>(
            readable_roots_with_ring_slots_rereading_unreadable_slots_once(
                &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
                region_devices,
                immutable_sizes,
                filesystem_identifier,
            ),
        )
    };
    let Ok(floor) = effective_rollback_floor_rereading_the_newest_instance_table_once_over(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        read_the_ring,
    );
    match floor {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(floor) => floor,
        EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
            roots_with_ring_slots_on_the_reread,
            ..
        } => effective_rollback_floor_of_the_roots_read(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            &roots_with_ring_slots_on_the_reread,
            None,
        ),
    }
}

/// 算 F 生效值时读根环、择最新那条根、读它的实例表，至多重读一次之后的结局
/// （[`effective_rollback_floor_rereading_the_newest_instance_table_once_over`]）。
enum EffectiveFloorAfterAtMostOneReread {
    /// 第一遍或重读那一遍读出了最新那条根的实例表，按它算出的 F 生效值。
    UnderTheNewestRootsTable(CheckpointTxg),
    /// 重读那一遍仍没有表（表读不出、解不开，或一条自证过的根都择不到）：交回重读那一遍读到的根环与择到的最新那条根，
    /// 调用方定拒还是不按表滤。
    NewestRootsTableStillUnreadable {
        roots_with_ring_slots_on_the_reread: Vec<(RootRingSlot, RootRecord)>,
        /// 重读那一遍择到的最新那条根（实例代号, checkpoint_txg）；一条都没择到时 `None`。
        newest_root_on_the_reread: Option<(InstanceGeneration, CheckpointTxg)>,
    },
}

/// 读一遍根环（`read_the_ring`：怎么读、读坏的槽怎么办由调用方定）、择最新那条根、沿链读它的实例表；表读不出、解不开或一条根都择不到时
/// 立即把根环与表一起重读一次（C554 乙的形态，R = 1；D16（发布语义） 已定项 1「重读一次」），读出表就按它算 F 生效值。
/// 第一次读与重读一次走同一段；重读那一遍根环与表都按 [`ReadPassOverThePool::RereadAfterABadRead`] 读（`read_the_ring` 收到哪一遍）。
///
/// # Errors
/// `read_the_ring` 的错原样交回（那一遍之后不再读）。
fn effective_rollback_floor_rereading_the_newest_instance_table_once_over<
    Reader: PoolReader + ?Sized,
    RingReadingFailure,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    read_the_ring: impl Fn(
        ReadPassOverThePool,
    ) -> Result<Vec<(RootRingSlot, RootRecord)>, RingReadingFailure>,
) -> Result<EffectiveFloorAfterAtMostOneReread, RingReadingFailure> {
    let read_the_ring_and_the_newest_roots_table = |pass: ReadPassOverThePool| {
        let roots_with_ring_slots = read_the_ring(pass)?;
        let newest = newest_root_among(&roots_with_ring_slots);
        let newest_roots_table = newest
            .and_then(|newest| {
                instance_table_chain_of_root(
                    &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
                    &newest,
                )
                .ok()
            })
            .map(|chain| chain.records);
        Ok((roots_with_ring_slots, newest, newest_roots_table))
    };
    let floor_under = |roots_with_ring_slots: &[(RootRingSlot, RootRecord)],
                       table: &InstanceTableRecords| {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(
            effective_rollback_floor_of_the_roots_read(
                reader,
                region_devices,
                immutable_sizes,
                filesystem_identifier,
                roots_with_ring_slots,
                Some(table),
            ),
        )
    };
    if let (roots_with_ring_slots, _, Some(table)) =
        read_the_ring_and_the_newest_roots_table(ReadPassOverThePool::FirstRead)?
    {
        return Ok(floor_under(&roots_with_ring_slots, &table));
    }
    Ok(
        match read_the_ring_and_the_newest_roots_table(ReadPassOverThePool::RereadAfterABadRead)? {
            (roots_with_ring_slots, _, Some(table)) => floor_under(&roots_with_ring_slots, &table),
            (roots_with_ring_slots_on_the_reread, newest_on_the_reread, None) => {
                EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
                    roots_with_ring_slots_on_the_reread,
                    newest_root_on_the_reread: newest_on_the_reread
                        .map(|root| (root.instance, root.checkpoint_txg)),
                }
            }
        },
    )
}

/// 读出来的根里 (checkpoint_txg, 实例代号) 最大的那一条（与 [`choose_root`] 同一个择法）；一条都没有时 `None`。
/// 择新键完全相同的不止一条时取读根环次序里最前的那条，与 `singlefs_replay_prefix::choose_root_slot` 的平局取法相同
/// （只有后面那条严格更新才换），F 生效值按表判有效根用的那张表才是恢复择的那条根指着的。
/// 入参须按读根环次序排（`every_root_ring_slot`：读根环的每一处都按这个次序走）。
fn newest_root_among(roots_with_ring_slots: &[(RootRingSlot, RootRecord)]) -> Option<RootRecord> {
    roots_with_ring_slots
        .iter()
        .map(|(_, root)| *root)
        .reduce(|newest_so_far, candidate| {
            if (candidate.checkpoint_txg, candidate.instance)
                > (newest_so_far.checkpoint_txg, newest_so_far.instance)
            {
                candidate
            } else {
                newest_so_far
            }
        })
}

/// 生效的回退下界 F 的算法本身（[`effective_rollback_floor`] 的文档），根环已经读过、判「有效根」的那张表已经读过（读不出时 `None`：
/// 不按表滤，只有 [`effective_rollback_floor`] 那一份这样传）。
fn effective_rollback_floor_of_the_roots_read<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    roots_with_ring_slots: &[(RootRingSlot, RootRecord)],
    newest_roots_table: Option<&InstanceTableRecords>,
) -> CheckpointTxg {
    let mut newest_valid_root_on_each_device: BTreeMap<DeviceIdentity, RootRecord> =
        BTreeMap::new();
    for (ring_slot, root) in roots_with_ring_slots {
        if newest_roots_table
            .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        {
            continue;
        }
        let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let newest_on_this_device = newest_valid_root_on_each_device
            .entry(device)
            .or_insert(*root);
        if (root.checkpoint_txg, root.instance)
            > (
                newest_on_this_device.checkpoint_txg,
                newest_on_this_device.instance,
            )
        {
            *newest_on_this_device = *root;
        }
    }
    // 被抛弃根带的 F 也算进 F_生效（收口表第 ② 行，用户 2026-09-28 定「算进」，被攻过一轮；
    // `research/prompts/abandoned-floor-r1-main-verification.md` M2）：不算时被抛弃根带着比别处高的 F 那一形上 I-7.12 红、
    // 回退能退到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽。
    let highest_on_the_abandoned_roots = roots_with_ring_slots
        .iter()
        .filter(|(_, root)| {
            newest_roots_table
                .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        })
        .map(|(_, root)| root.rollback_floor)
        .max();
    let highest_on_the_roots = newest_valid_root_on_each_device
        .values()
        .map(|root| root.rollback_floor)
        .chain(highest_on_the_abandoned_roots)
        .max();
    let slot_spacing_in_bytes = u64::from(immutable_sizes.fixed_structure_slot_spacing);
    let highest_in_the_system_configurations = reader
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                reader,
                device,
                slot_spacing_in_bytes,
                filesystem_identifier,
            )
        })
        .map(|system_configuration| system_configuration.quantities.rollback_floor)
        .max();
    highest_on_the_roots
        .max(highest_in_the_system_configurations)
        .unwrap_or(CheckpointTxg(0))
}
