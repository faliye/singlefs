//! 读根环与择根：每一槽各读一次、读坏的分法、择根并留下全部自证过的根，可读的根与重读一次的几种读法，
//! 根环里最高的 txg、树 ID 水位与实例代号，一版的根记录字节。

use super::*;

/// 根环三个区域全部槽里自证过的根，逐个交给 `visit`（择根与取号共用这一段遍历）。
fn visit_valid_roots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    mut visit: impl FnMut(RootRecord),
) {
    visit_valid_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |_ring_slot, root| visit(root),
    );
}

/// 同 `visit_valid_roots`，连它读出来的那个根环槽一起交出去：分配器要知道每条根住哪个槽，
/// 才认得出之后哪一次发布盖掉了它（`allocator::RootRingOccupancy`）。读不出、自证不过的槽跳过。
fn visit_valid_roots_with_ring_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    mut visit: impl FnMut(RootRingSlot, RootRecord),
) {
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        match read_root_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => visit(ring_slot, root),
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => continue,
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}
        }
    }
}

/// 根环三个区域的全部槽，区域号升序、区域内槽号升序（读根环的每一处都按这个次序走）。
#[must_use]
pub fn every_root_ring_slot(immutable_sizes: &SystemImmutableSizes) -> Vec<RootRingSlot> {
    (0..ROOT_RING_REGIONS)
        .flat_map(|region| {
            (0..immutable_sizes.root_ring_slots_per_region.count())
                .map(move |slot| RootRingSlot { region, slot })
        })
        .collect()
}

/// 读一个根环槽读坏了是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadRootRingSlotReading {
    /// 读不出：设备报错、越界、那块盘不在读者里（`PoolReader::read` 交回空）。
    Unreadable,
    /// 读得出、自证不过：校验和不过、fsid 不是本池的、从没写过的全 0 槽（`RootRecord::parse_slot` 交回空）；
    /// 也收自证得过、却不住在它的 txg 按落点公式该住的那一槽的根（错位读，C743；
    /// `root_ring::root_sits_in_the_ring_slot_the_formula_gives`，第 0 代根照 mkfs 种根的槽另认）。
    NotSelfVerified,
}

/// 读一个根环槽的结局。
#[allow(
    clippy::large_enum_variant,
    reason = "一次读一个槽、按值交回马上拆开，装箱只多一次分配"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootRingSlotReading {
    SelfVerified(RootRecord),
    Bad(BadRootRingSlotReading),
}

/// 读一个根环槽（区域归属表给出它在哪块盘、槽距给出偏移），验自证校验和与 fsid，再核落点公式（C743）：自证得过的根不住在
/// 它的 txg 该住的那一槽（第 0 代根照 mkfs 种根的槽另认），照读得出、自证不过记（错位读当这一槽这一次读坏）。只读一次，不重试。
/// 自证过、落点也对的字节报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]：当场重读那一遍的读者拿它换掉这次挂载记下的那一份）。
#[must_use]
pub fn read_root_ring_slot<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slot: RootRingSlot,
) -> RootRingSlotReading {
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
    let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
        return RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable);
    };
    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
        Some(root)
            if !root_sits_in_the_ring_slot_the_formula_gives(
                root.checkpoint_txg,
                ring_slot,
                immutable_sizes.root_ring_slots_per_region,
            ) =>
        {
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)
        }
        Some(root) => {
            reader.note_that_the_bytes_read_self_verify(device, offset, &bytes);
            RootRingSlotReading::SelfVerified(root)
        }
        None => RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
    }
}

/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7 择新；
/// 已定项 24 先逐个验证全部候选、再在有效者中择新）；最大的不止一个槽时取读根环次序里最前的那个。
/// 读法与择法同 [`choose_root_keeping_every_self_verified_root_read`]，只交回择到的那条根。
#[must_use]
pub fn choose_root(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> Option<RootRecord> {
    choose_root_keeping_every_self_verified_root_read(reader, system_configuration).chosen_root
}

/// 择根那一遍读出来的：择到的根（一条自证过的都没有时 `None`），与这一遍每个槽读出、自证过的根（根环次序，同一条根住两槽就出现两次）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootChosenWithEverySelfVerifiedRootRead {
    pub chosen_root: Option<RootRecord>,
    pub self_verified_roots_in_ring_order: Vec<RootRecord>,
}

/// 择根（[`choose_root`] 那一份择法），连同这一遍读出、自证过的全部根一起交回：每个槽只读一次，择到的根与交回的全部根出自同一遍读数。
/// 可写挂载的读阶段拿它择根，读到的全部根留给读阶段之后的几遍（`mount` 的 `rebuilt_allocator` 里 C 并的较新可读根），之后那几遍读坏的根不因此漏掉。
/// 取最大那一截在 `singlefs_replay_prefix::choose_root_slot` 里（Verus 验的是那一份）；这里只读槽、验自证校验和与 fsid，
/// 把每个槽摊成核的入参（自证过的给择新键，读不出、自证不过的给 None），再照核交回的下标取回那条根。
#[must_use]
pub fn choose_root_keeping_every_self_verified_root_read(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> RootChosenWithEverySelfVerifiedRootRead {
    let immutable = &system_configuration.immutable;
    let self_verified_roots_in_ring_order: Vec<Option<RootRecord>> =
        every_root_ring_slot(&immutable.sizes)
            .into_iter()
            .map(|ring_slot| {
                match read_root_ring_slot(
                    reader,
                    &immutable.region_devices,
                    &immutable.sizes,
                    &immutable.filesystem_identifier,
                    ring_slot,
                ) {
                    RootRingSlotReading::SelfVerified(root) => Some(root),
                    RootRingSlotReading::Bad(
                        BadRootRingSlotReading::Unreadable
                        | BadRootRingSlotReading::NotSelfVerified,
                    ) => None,
                }
            })
            .collect();
    let choice_keys_in_ring_order: Vec<Option<RootChoiceKey>> = self_verified_roots_in_ring_order
        .iter()
        .map(|self_verified_root| {
            self_verified_root.map(|root| RootChoiceKey {
                checkpoint_txg: PrefixKernelCheckpointTxg(root.checkpoint_txg.0),
                instance: PrefixKernelInstanceGeneration(root.instance.0),
            })
        })
        .collect();
    let chosen_root = choose_root_slot(&choice_keys_in_ring_order).map(|chosen_slot| {
        self_verified_roots_in_ring_order[chosen_slot]
            .expect("择根核只交回自证过的槽（它的 ensures：所选槽在入参里是 Some），这里是 None 说明核或摊入参错了")
    });
    RootChosenWithEverySelfVerifiedRootRead {
        chosen_root,
        self_verified_roots_in_ring_order: self_verified_roots_in_ring_order
            .into_iter()
            .flatten()
            .collect(),
    }
}

/// 根环全部自证过的根里最大的 checkpoint_txg：新实例的第一次发布取 max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）；一条都没有时 None。
#[must_use]
pub fn highest_root_txg<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Option<CheckpointTxg> {
    let mut highest: Option<CheckpointTxg> = None;
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| {
            highest = Some(highest.map_or(root.checkpoint_txg, |current| {
                current.max(root.checkpoint_txg)
            }));
        },
    );
    highest
}

/// 本实例第一次发布要带的树 ID 水位：根环全部自证过的根的「树 ID 水位」取 max（D8（核心索引结构） 已定项 8 ②
/// 「根环里全部根记录该字段的 max」——被抛弃时间线上的根也算，它们仍承载那一代发过的号），
/// 并上 `records`（环里全部自证通过的 journal 记录）新根段里带的水位（D23（journal 的角色与格式） 已定项 15）。
///
/// 记录那一半是根读不出时的最保守读法：根槽撕了、被盖了或那块盘掉了，而它那次发布的记录还在，就按记录带的水位算
/// （C342（树 ID 水位在根读不出时退回去重发） 的载体；条款没写读不出的根怎么算，报告里交主 agent）。
/// 根与它那条记录都读不出的那一格仍然盖不住。与新实例第一次发布的 txg 取 max(环里的根, 环里的记录) 同一个形态
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。一条根、一条记录都没有时 None。
#[must_use]
pub fn highest_tree_identifier_watermark_in_the_ring<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> Option<u64> {
    let mut highest: Option<u64> = records
        .values()
        .map(|record| record.new_tree_identifier_watermark)
        .max();
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| {
            highest = Some(highest.map_or(root.tree_identifier_watermark, |current| {
                current.max(root.tree_identifier_watermark)
            }));
        },
    );
    highest
}

/// 根环里全部自证过的根，每个可读根槽一条（回退候选集与影子账从这里取）。
#[must_use]
pub fn readable_roots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Vec<RootRecord> {
    let mut roots = Vec::new();
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| roots.push(root),
    );
    roots
}

/// 读根环，读坏的槽按这个进程知不知道那里住着一条根分两类（D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句，用户 2026-09-26 定；
/// 与 `mount::rollback_floor_ceiling` 读根环同一个分法）：每个槽读一次；读坏（读不出 [`BadRootRingSlotReading::Unreadable`]：设备报错、越界；
/// 或自证不过 [`BadRootRingSlotReading::NotSelfVerified`]）的槽不在 `ring_slots_known_to_hold_a_root` 里（挂载那一刻就读不出或自证不过、
/// 这个进程之后也没写过）的当没有根、不重读；在里面的（挂载那一刻读得出、或这个进程写过且 FUA 返回过，
/// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）立即重读一次（C554 乙的形态，R = 1），重读自证得过就照它算，
/// 仍坏（读不出或自证不过）就报错，不按有根或没根猜。交回读得出的每一条根连同它的槽，按根环槽的次序。
/// 挂着时抬 F 重算影子账与回收门槛、管理员回退判候选与算 F 生效值用它（C554 乙报告 Q6：`readable_roots` 把读不出的槽一律当没有根，
/// 那一槽里的被抛弃根引用的槽不隔离、也不计进读不出账的被抛弃根）。
///
/// # Errors
/// 知道住着根的槽重读仍坏 ⇒ [`RootRingSlotKnownToHoldARootStillBadAfterOneReread`]（按 [`every_root_ring_slot`] 的次序第一个）。
/// 只读盘。
pub fn readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
) -> Result<Vec<(RootRingSlot, RootRecord)>, RootRingSlotKnownToHoldARootStillBadAfterOneReread> {
    let read_the_slot = |ring_slot| {
        read_root_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    // 重读那一读绕过这次挂载记下的那一份、再读一次盘（[`ReadPassOverThePool::RereadAfterABadRead`]）。
    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
    let reread_the_slot = |ring_slot| {
        read_root_ring_slot(
            &rereading_reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    let mut roots = Vec::new();
    // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根。提前出口只有「知道住着根的槽重读仍坏」一个。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        let first_reading = match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => {
                roots.push((ring_slot, root));
                continue;
            }
            RootRingSlotReading::Bad(first_reading) => first_reading,
        };
        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
            continue;
        }
        match reread_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(reread) => {
                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
                    ring_slot,
                    first_reading,
                    reread,
                });
            }
        }
    }
    Ok(roots)
}

/// 同 [`readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once`]，只交回根（不带槽）。
///
/// # Errors
/// 同那一份。
pub fn readable_roots_rereading_ring_slots_known_to_hold_a_root_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
) -> Result<Vec<RootRecord>, RootRingSlotKnownToHoldARootStillBadAfterOneReread> {
    readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        ring_slots_known_to_hold_a_root,
    )
    .map(|roots_with_ring_slots| {
        roots_with_ring_slots
            .into_iter()
            .map(|(_, root)| root)
            .collect()
    })
}

/// 同 [`readable_roots_with_ring_slots`]，而一个根槽读不出（[`BadRootRingSlotReading::Unreadable`]：设备报错、越界）时立即重读那一槽一次，
/// 重读读得出就照它算，仍读不出当没有根；读得出、自证不过的槽不重读（从没写过的全 0 槽都是这一类）。给手里没有这个进程根环表、
/// 分不出哪几个槽住着根的读者用（`transaction::PoolWriter` 写系统配置之前算 F 生效值，[`effective_rollback_floor_rereading_unreadable_reads_once`]）。
#[must_use]
pub fn readable_roots_with_ring_slots_rereading_unreadable_slots_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Vec<(RootRingSlot, RootRecord)> {
    let read_the_slot = |ring_slot| {
        read_root_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    let mut roots = Vec::new();
    // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根；没有提前出口。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        let reading = match read_the_slot(ring_slot) {
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
                read_the_slot(ring_slot)
            }
            first_reading @ (RootRingSlotReading::SelfVerified(_)
            | RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)) => first_reading,
        };
        match reading {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(
                BadRootRingSlotReading::NotSelfVerified | BadRootRingSlotReading::Unreadable,
            ) => {}
        }
    }
    roots
}

/// 同 [`readable_roots`]，而一个根槽读坏（读不出或自证不过，两种都算）时立即重读那一槽一次，重读自证得过就照它算，仍坏当没有根。
/// 重读走 [`PoolReaderOfOneReadPass::rereading_after_a_bad_read`]：可写挂载里绕过这次挂载记下的那一份、再读一次盘。
/// F2 判退回的候选读根环用它（C563 第 4 处，用户 2026-10-22 定「甲：读坏的槽一律重读一次」）：那一刻手里没有这个进程的根环表、
/// 分不出哪几个槽住着根，一次瞬时读坏不该把一次做得成的退回变成拒；仍坏当没有根，交给后面越过一次根读不出的发布就不退那一关判。
/// 代价是从没写过的全 0 槽每次都多读一次（只在 F2 触发时）。
pub(super) fn readable_roots_rereading_every_bad_ring_slot_once<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Vec<RootRecord> {
    let read_the_slot = |ring_slot| {
        read_root_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
    let reread_the_slot = |ring_slot| {
        read_root_ring_slot(
            &rereading_reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    let mut roots = Vec::new();
    // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根；没有提前出口。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        let reading = match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => RootRingSlotReading::SelfVerified(root),
            RootRingSlotReading::Bad(_first_reading) => reread_the_slot(ring_slot),
        };
        match reading {
            RootRingSlotReading::SelfVerified(root) => roots.push(root),
            RootRingSlotReading::Bad(
                BadRootRingSlotReading::NotSelfVerified | BadRootRingSlotReading::Unreadable,
            ) => {}
        }
    }
    roots
}

/// 同 `readable_roots`，每条根带着它读出来的那个根环槽（挂载与抬 F 给分配器建根环那张表用，`allocator::RootRingOccupancy`）。
#[must_use]
pub fn readable_roots_with_ring_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Vec<(RootRingSlot, RootRecord)> {
    let mut roots = Vec::new();
    visit_valid_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |ring_slot, root| roots.push((ring_slot, root)),
    );
    roots
}

/// 一版的根记录 [0, `ROOT_RECORD_BYTES`) 的字节：根环里有一槽自证过、解出来与 `root` 逐字段相同的，交那一槽的原字节
/// （按根环槽的次序第一个）；没有的（由记录施加出来、根槽从没落盘的那一版）交 `root` 按字段表编出来的那几字节
/// （[`RootRecord::to_slot`]：算法类型、nonce、MAC 那 29 字节与 flags 其余位写 0）。只读。
/// 挂载透明的判官按字节区间比 R1 与 R2 的根记录用它（根记录里 `parse_slot` 不进内存的那几样——留位、flags 其余位——只有原字节里有）。
#[must_use]
pub fn root_record_bytes_of_the_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    root: &RootRecord,
) -> Vec<u8> {
    let record_bytes = usize::try_from(ROOT_RECORD_BYTES).expect("466");
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    every_root_ring_slot(immutable_sizes)
        .into_iter()
        .find_map(|ring_slot| {
            let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
            let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
            let bytes = reader.read(device, offset, root_slot_bytes)?;
            (RootRecord::parse_slot(&bytes, &root.filesystem_identifier).as_ref() == Some(root))
                .then(|| bytes[..record_bytes].to_vec())
        })
        .unwrap_or_else(|| root.to_slot(root_slot_bytes)[..record_bytes].to_vec())
}

/// 根环全部自证过的根里最大的实例代号（取号的 max 里「根环里全部根记录的实例代号」那一半）；一条都没有时 None。
#[must_use]
pub fn highest_root_instance<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Option<InstanceGeneration> {
    let mut highest: Option<InstanceGeneration> = None;
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| {
            highest = Some(highest.map_or(root.instance, |current| current.max(root.instance)));
        },
    );
    highest
}
