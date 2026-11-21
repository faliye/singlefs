//! 抬 F 的上限：读根环算上限、准入那一格算两遍判卡没卡在挂载时读坏的槽上，非空有效根与不同状态的去重，
//! 根环此刻的回退状态，连同这个进程分配器上那张根环表里各类槽的读数。

use super::*;

/// 一条有效根的用户可见状态有没有变（D16（发布语义） 已定项 1「非空」从盘上怎么认，2026-09-17 用户定案）：它树表里 inode 树与 extent 树的
/// 根指针、加上只读快照条目（码 14 / 15）的根指针按盘上次序排成的那一串（用户 2026-10-14 定「建快照那次发布算非空」），与它前一条有效根
/// 树表里的逐字节比；三样任一样不同就算非空。最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比。
/// 只比根指针，不比 flags：删快照的意图那次（只写「销毁中」的记号、传前驱）与只放 deadlist 条目的批不算非空，删掉那一对的最后一批算。
/// 整理搬了用户可见树的根单元的那一批：树表照映射位置重写那一条的根指针（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3），
/// 字节变了，算非空（用户 2026-10-13 定「照字节比，搬根算非空」；判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J6 核过）。
#[must_use]
pub fn user_visible_trees_changed(
    root_pointers: &UserVisibleTreeRootPointers,
    previous_valid_root_pointers: &UserVisibleTreeRootPointers,
) -> bool {
    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree
        || root_pointers.snapshot_tree_roots != previous_valid_root_pointers.snapshot_tree_roots
}

/// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定）：每个槽读一次；读坏的槽按这个进程知不知道
/// 那里住着一条根分两类——`ring_slots_known_to_hold_a_root` 里没有的（挂载那一刻就读不出或自证不过、之后也没写过）当没有根，
/// 有的（挂载那一刻读得出、或这个进程写过且 FUA 返回过）重读一次，读出来就用重读的那一条，仍坏就拒，不按有根或没根猜。
/// `root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass` 里的槽（重建分配器两遍里只有一遍自证过的，各带重建那一遍读出的整条根记录）
/// 读坏也重读一次、读出来就用重读的那一条，仍坏不拒、拿记着的那条根顶上（用户 2026-10-06 定 T1 甲：有效的照它本身进上限，被抛弃的由调用方按实例表滤掉）。
/// 交回读得出（或顶上）的每一条根连同它所在的那一槽（按根环槽的次序），与这一遍读坏、当没有根的那几个槽。
///
/// # Errors
/// 知道住着根的槽（不在 `root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass` 里的）重读仍坏 ⇒
/// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
    root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass: &BTreeMap<
        RootRingSlot,
        RootRecord,
    >,
) -> Result<RootsOfTheRingForTheCeiling, MountError> {
    let read_once = |ring_slot: RootRingSlot| {
        read_root_ring_slot(
            reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            ring_slot,
        )
    };
    // 读坏之后当场重读那一读绕过这次挂载记下的那一份、再读一次盘，读到自证过的换掉记下的那一份
    // （`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
    let reread_once_after_a_bad_read = |ring_slot: RootRingSlot| {
        read_root_ring_slot(
            &rereading_reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            ring_slot,
        )
    };
    let mut roots_with_ring_slots = Vec::new();
    let mut ring_slots_bad_on_this_read_taken_as_holding_no_root = BTreeSet::new();
    // 迭代上界是根环的槽数（3 × S）；跨轮携带的只有交回的根与当没有根的槽；提前出口只有「知道住着根的槽重读仍坏」那一条。
    for ring_slot in every_root_ring_slot(&system_configuration.immutable.sizes) {
        let first_reading = match read_once(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => {
                roots_with_ring_slots.push((ring_slot, root));
                continue;
            }
            RootRingSlotReading::Bad(first_reading) => first_reading,
        };
        if let Some(root_record_self_verified_on_one_rebuild_pass) =
            root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.get(&ring_slot)
        {
            match reread_once_after_a_bad_read(ring_slot) {
                RootRingSlotReading::SelfVerified(root) => {
                    roots_with_ring_slots.push((ring_slot, root));
                }
                RootRingSlotReading::Bad(_still_bad) => {
                    roots_with_ring_slots
                        .push((ring_slot, *root_record_self_verified_on_one_rebuild_pass));
                }
            }
            continue;
        }
        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
            ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);
            continue;
        }
        match reread_once_after_a_bad_read(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => {
                roots_with_ring_slots.push((ring_slot, root))
            }
            RootRingSlotReading::Bad(reread) => {
                return Err(
                    MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
                        ring_slot,
                        first_reading,
                        reread,
                    },
                );
            }
        }
    }
    Ok(RootsOfTheRingForTheCeiling {
        roots_with_ring_slots,
        ring_slots_bad_on_this_read_taken_as_holding_no_root,
    })
}

/// 算上限那一遍读根环读出来的（[`roots_of_the_ring_for_the_ceiling`]）。
struct RootsOfTheRingForTheCeiling {
    /// 读得出、自证过的每一条根，与重建分配器两遍里只有一遍自证过、这一遍重读一次仍坏的槽记着的那条根，各带它所在的那一槽，按根环槽的次序。
    /// 算「每块盘上最新的有效根」时按这一槽的区域认盘（mkfs 在三个区域的槽 0 各种一份的第 0 代根各归各的盘）。
    roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)>,
    /// 这一遍读不出或自证不过、又不是「这个进程知道住着根的槽」的槽：当没有根。
    ring_slots_bad_on_this_read_taken_as_holding_no_root: BTreeSet<RootRingSlot>,
}

/// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
/// 比它小的有效根里最大的那条）的不同（`user_visible_trees_changed`，2026-09-17 用户定案）。
/// 上限与「非空」在环里全部有效根上取：原来同取一圈窗口的六处之一（B2，用户 2026-10-03 定），用户 2026-10-04 定 RGall 一起退回整个环
/// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。回收门槛同时退回整个环，窗口外的有效根引用的单元不再被回收，
/// 读它的树表读得到它那一版（窗口许可复用之后读窗口外旧根的树表读到别的内容那一形随之没有了）。
/// 根环怎么读见 [`roots_of_the_ring_for_the_ceiling`]：`ring_slots_whose_reread_still_bad_refuses_the_ceiling` 是这一遍读坏、重读一次仍坏就拒的槽，
/// 取这个进程分配器上那张根环表里知道住着根、又没记成被抛弃根的槽（[`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`]，
/// 与抬 F 那一串同一个口径：被抛弃根的槽读坏不拒、当没有根、不进上限，D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1）；
/// 分配器上没装根环表的（只有用例自己拼的分配器，可写挂载与 `make_filesystem::allocator_after_make_filesystem` 都装）给空集，读坏的槽一律当没有根。
/// 这一份不知道哪几个槽是挂载那一刻读坏的（给手里没有这个进程的根环表的调用方：实验装置、用例）；这个进程的准入抬 F 走
/// [`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]。
///
/// # Errors
/// 一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`；要比的一条有效根的树表读不出或解不开 ⇒
/// `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（读不出要走修复，不按空或非空猜）；
/// `ring_slots_whose_reread_still_bad_refuses_the_ceiling` 里的根环槽重读仍坏 ⇒ `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
#[allow(
    clippy::ptr_arg,
    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub fn rollback_floor_ceiling<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
    ring_slots_whose_reread_still_bad_refuses_the_ceiling: &BTreeSet<RootRingSlot>,
) -> Result<CheckpointTxg, MountError> {
    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
        devices,
        system_configuration,
        current_floor,
        table,
        ring_slots_whose_reread_still_bad_refuses_the_ceiling,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}

/// 同 [`rollback_floor_ceiling`]，再把挂载那一刻读不出、读得出而自证不过或读回全零、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
/// （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行；RS3 用户 2026-10-04 定只并读不出的，
/// `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节，同日 Q3 重定成两类都并，
/// `research/prompts/m3-root-ring-code-r3-main-verification.md` 第六节；读回全零的那一类用户 2026-11-02 定「并」，原话在决策变更史 D16（发布语义） 节）：
/// `ring_slots_bad_since_the_mount` 是这个进程根环表里还记着的那几个槽（[`ring_slots_bad_or_reading_back_all_zero_at_mount_by`]）。
/// 三类都只认挂载那一刻在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住一个 txg 的槽
/// （N = 挂载时环里读得出的最新有效根，`rebuilt_allocator`），各按它在那里能住的最大 txg 并：读不出的那一类与回收门槛记的同一个数，
/// 自证不过与读回全零的那两类同一个算法（区域内槽位只由 txg 定）。这一遍读那一槽又读坏（读不出、自证不过或读回全零）、那个 txg 不低于今天的 F，
/// 上限就取不高于它。
/// 这一遍读得出的照读出来的根算，不另封顶。判「非空」、找「前一条」照旧只在读得出的根上取（读不出的那一条判不了空不空，F 不越过它就罩住了它是第 4 新的那一种）。
/// 「每块盘上最新的有效根」按每一条根所在那一槽的区域认盘（D16（发布语义） 已定项 1 字面「每块盘上最新的持久有效根」；判决
/// `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节「Q3 外带」：mkfs 在三个区域的槽 0 各种了一份第 0 代根，txg 都是 0，
/// 按落点公式认盘会把三份都算在区域 0 的盘上，区域 1 那块盘上只剩那一份时上限就越过了它）。
/// `ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes` 是这个进程根环表里重建分配器两遍里只有一遍自证过的槽，
/// 各带重建那一遍读出的整条根记录（[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]）：这一遍读坏就重读一次，
/// 读出来照读出来的算，仍坏不拒、拿记着的那条根顶上，与读出来的根一样按 `table` 判有效（用户 2026-10-06 定 T1 甲：有效的照它本身进上限，
/// 被抛弃的不进；它们也在 `ring_slots_known_to_hold_a_root` 里，这里先按它们分）。
///
/// # Errors
/// 同 [`rollback_floor_ceiling`]。
#[allow(
    clippy::ptr_arg,
    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub fn rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
    ring_slots_bad_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes: &BTreeMap<
        RootRingSlot,
        RootRecord,
    >,
) -> Result<CheckpointTxg, MountError> {
    admission_ceilings_not_passing_ring_slots_bad_since_the_mount(
        devices,
        devices,
        system_configuration,
        current_floor,
        table,
        ring_slots_known_to_hold_a_root,
        ring_slots_bad_since_the_mount,
        ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
    )
    .map(|ceilings| ceilings.not_passing_the_ring_slots_bad_since_the_mount)
}

impl AdmissionCeilings {
    /// 上限卡在挂载时读坏或读回全零的槽上：并进它们之后不高于 `floor`、不并它们时高于 `floor`（「抬 F 的上限」那一行那一括注，
    /// E173（上限卡在读坏槽的三种处置） 登记「推空B」那一臂：并进那几槽的上限不高于 F 而不并时的上限高于 F）。
    #[must_use]
    pub(super) fn are_held_by_ring_slots_bad_since_the_mount(self, floor: CheckpointTxg) -> bool {
        self.not_passing_the_ring_slots_bad_since_the_mount <= floor
            && self.from_the_roots_read > floor
    }
}

/// [`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`] 的本体，两个上限都交回（[`AdmissionCeilings`]）：读一遍根环、
/// 照读得出的根算一个，再并上挂载时读坏或读回全零的槽算另一个。
///
/// # Errors
/// 同 [`rollback_floor_ceiling`]。
#[allow(
    clippy::too_many_arguments,
    reason = "读盘的读者、配置与三份记账都在调用方手里，这个函数只算上限，包一层结构没有别的用处"
)]
#[allow(
    clippy::ptr_arg,
    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn admission_ceilings_not_passing_ring_slots_bad_since_the_mount<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    reader_of_the_tree_tables: &dyn PoolReader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
    ring_slots_bad_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes: &BTreeMap<
        RootRingSlot,
        RootRecord,
    >,
) -> Result<AdmissionCeilings, MountError> {
    let RootsOfTheRingForTheCeiling {
        roots_with_ring_slots: readable,
        ring_slots_bad_on_this_read_taken_as_holding_no_root,
    } = roots_of_the_ring_for_the_ceiling(
        devices,
        system_configuration,
        ring_slots_known_to_hold_a_root,
        ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
    )?;
    let valid_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = readable
        .iter()
        .copied()
        .filter(|(_, root)| root.checkpoint_txg >= current_floor)
        .filter(|(_, root)| !abandoned_by_table(root, table))
        .collect();
    let valid: Vec<RootRecord> = valid_with_ring_slots
        .iter()
        .map(|(_, root)| *root)
        .collect();
    let mut newest_per_device: std::collections::BTreeMap<DeviceIdentity, CheckpointTxg> =
        std::collections::BTreeMap::new();
    for (ring_slot, root) in &valid_with_ring_slots {
        // 按根所在那一槽的区域认盘，不按落点公式从 txg 推：第 0 代根三份 txg 都是 0，各在各的区域里。
        let device = system_configuration.immutable.region_devices
            [usize::try_from(ring_slot.region).expect("区域号")];
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
    let pointers_of = |root: &RootRecord| match user_visible_tree_root_pointers(
        reader_of_the_tree_tables,
        root,
    ) {
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
    // 「前一条」只在有效根里找：被抛弃时间线的根与 F 之下的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认；RGall 之后窗口外的有效根照算）。
    let previous_candidates: &[RootRecord] = &valid;
    let mut non_empty: Vec<NonEmptyValidRoot> = Vec::new();
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
            non_empty.push(NonEmptyValidRoot {
                checkpoint_txg: root.checkpoint_txg,
                instance: root.instance,
                user_visible_tree_root_pointers: root_pointers,
            });
        }
    }
    // C724（准入抬 F 上限里同名拒因不在单子上）：交 `None` 只有「第 4 新的不同状态没有、最旧有效根也没有」一条路，而最旧有效根取自 `valid`；
    // `valid` 为空时 `newest_per_device` 也为空，上面取每块盘最新那一判已经交回 `NoValidRoot`。这里与盘面、故障都无关，走不到。
    let ceiling_from_the_roots_read = ceiling_from_newest_and_non_empty_roots(
        newest_on_every_device,
        newest_first_distinct_states(non_empty),
        valid.iter().map(|root| root.checkpoint_txg).min(),
    )
    .expect(
        "valid 不空（每块盘最新的有效根取得出最小值，空的 valid 在那一判已交回 NoValidRoot），最旧有效根就有：上限一定算得出",
    );
    let lowest_txg_a_ring_slot_bad_since_the_mount_can_hold =
        ring_slots_bad_on_this_read_taken_as_holding_no_root
            .iter()
            .filter_map(|ring_slot| ring_slots_bad_since_the_mount.get(ring_slot))
            .copied()
            .filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)
            .min();
    Ok(AdmissionCeilings {
        from_the_roots_read: ceiling_from_the_roots_read,
        not_passing_the_ring_slots_bad_since_the_mount:
            match lowest_txg_a_ring_slot_bad_since_the_mount_can_hold {
                Some(highest_txg_it_can_hold) => {
                    ceiling_from_the_roots_read.min(highest_txg_it_can_hold)
                }
                None => ceiling_from_the_roots_read,
            },
    })
}

/// 一条非空的有效根：它的 (txg, 实例代号) 与它树表里用户可见的两棵树的根指针（去重按后者比）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonEmptyValidRoot {
    pub checkpoint_txg: CheckpointTxg,
    pub instance: InstanceGeneration,
    pub user_visible_tree_root_pointers: UserVisibleTreeRootPointers,
}

/// 数「第 N 新的非空有效根」与「4 个不同状态」时去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：从新到旧数，
/// 一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入——管理员回退发出的新根
/// 与它前一条有效根不同、算非空，而它的状态与更早的某一条相同，不去重就把同一个状态数两次。交回计入的那几条的 txg，从新到旧。
///
/// 根环容量是这一数的边界（同一段定案首段）：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，这里只数得出两条，
/// 上限随之取最旧的有效根（[`ceiling_from_newest_and_non_empty_roots`]）。
///
/// 循环：外层按 (txg, 实例代号) 从新到旧走一遍非空根（轮数 = 非空根的条数），跨轮携带已计入的状态；没有提前出口。
#[must_use]
pub fn newest_first_distinct_states(mut non_empty: Vec<NonEmptyValidRoot>) -> Vec<CheckpointTxg> {
    non_empty.sort_unstable_by(|left, right| {
        (right.checkpoint_txg, right.instance).cmp(&(left.checkpoint_txg, left.instance))
    });
    let mut counted_states: Vec<&UserVisibleTreeRootPointers> = Vec::new();
    let mut distinct = Vec::new();
    for root in &non_empty {
        if counted_states.contains(&&root.user_visible_tree_root_pointers) {
            continue;
        }
        counted_states.push(&root.user_visible_tree_root_pointers);
        distinct.push(root.checkpoint_txg);
    }
    distinct
}

/// 根环此刻的回退状态（后台整理的回退门与挂载那一份预算读它；D16（发布语义） 已定项 1「4 个不同状态」，数法同 [`newest_first_distinct_states`]，
/// 整理搬了用户可见树的根单元的那一批照字节比算非空）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RollbackStatesOfTheRing {
    /// 根环里此刻不同用户状态的个数。
    pub(crate) states_now: u64,
    /// 下一次发布写的那一槽与它之后余量（[`root_ring_slots_left_by_a_compaction_publish`]）那几槽里的根都走了之后还剩几个（走的那一条是某个状态在环里最后一条时才少一个）。
    pub(crate) states_after_the_next_publish_and_the_margin: u64,
    /// 比最新那个用户状态的根新的有效根条数（环里一个状态都没有时是有效根的条数）：挂载那一份预算当它是已经过去的发布数。
    pub(crate) publishes_after_the_newest_state: u64,
}

/// 一串有效根（不分先后）从新到旧的不同状态：按 (txg, 实例代号) 逐条跟前一条比认非空（同 `admission_ceilings_not_passing_ring_slots_bad_since_the_mount`
/// 那一判，最旧的一条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比），再照 [`newest_first_distinct_states`] 去重。纯函数，读根与读树表在调用方。
fn distinct_states_of_the_valid_roots<'root>(
    valid_roots: impl Iterator<Item = &'root (RootRecord, UserVisibleTreeRootPointers)>,
) -> Vec<CheckpointTxg> {
    let mut oldest_first: Vec<&(RootRecord, UserVisibleTreeRootPointers)> = valid_roots.collect();
    oldest_first.sort_by_key(|(root, _)| (root.checkpoint_txg, root.instance));
    let absent = UserVisibleTreeRootPointers::ABSENT;
    let mut previous_pointers = &absent;
    let mut non_empty: Vec<NonEmptyValidRoot> = Vec::new();
    for (root, pointers) in oldest_first {
        if user_visible_trees_changed(pointers, previous_pointers) {
            non_empty.push(NonEmptyValidRoot {
                checkpoint_txg: root.checkpoint_txg,
                instance: root.instance,
                user_visible_tree_root_pointers: pointers.clone(),
            });
        }
        previous_pointers = pointers;
    }
    newest_first_distinct_states(non_empty)
}

/// 读根环，交回 [`RollbackStatesOfTheRing`]。有效根同算抬 F 的上限：不低于现行那一版的 F、不被现行那一版的实例表抛弃。读不出的槽当没有根
/// （少数一个状态：回退门宁可不发）。读的是整个根环与每条有效根的树表，代价随根环槽数（3S），不随盘容量。
///
/// # Errors
/// 现行那一版的实例表读不通，或一条有效根的树表读不通。
pub(crate) fn rollback_states_of_the_ring(
    reader: &dyn PoolReader,
    parameters: &MakeFilesystemParameters,
    current_root: &RootRecord,
) -> Result<RollbackStatesOfTheRing, RecoveryFailure> {
    let table = instance_table_chain_of_root(reader, current_root)?.records;
    let ring_slots_of_the_next_publish_and_the_margin: Vec<RootRingSlot> = (1..=1
        + root_ring_slots_left_by_a_compaction_publish(
            parameters.geometry.root_ring_slots_per_region,
        ))
        .map(|publishes_ahead| {
            target_for_publish(
                CheckpointTxg(current_root.checkpoint_txg.0 + publishes_ahead),
                parameters.geometry.root_ring_slots_per_region,
            )
        })
        .collect();
    let mut valid_roots: Vec<(RootRecord, UserVisibleTreeRootPointers)> = Vec::new();
    let mut valid_roots_past_the_next_publish_and_the_margin: Vec<(
        RootRecord,
        UserVisibleTreeRootPointers,
    )> = Vec::new();
    for ring_slot in every_root_ring_slot(&parameters.geometry) {
        let reading = read_root_ring_slot(
            reader,
            &parameters.region_devices,
            &parameters.geometry,
            &parameters.filesystem_identifier,
            ring_slot,
        );
        let RootRingSlotReading::SelfVerified(root) = reading else {
            continue;
        };
        if root.checkpoint_txg < current_root.rollback_floor || abandoned_by_table(&root, &table) {
            continue;
        }
        let pointers = user_visible_tree_root_pointers(reader, &root)?;
        if !ring_slots_of_the_next_publish_and_the_margin.contains(&ring_slot) {
            valid_roots_past_the_next_publish_and_the_margin.push((root, pointers.clone()));
        }
        valid_roots.push((root, pointers));
    }
    let states_now = distinct_states_of_the_valid_roots(valid_roots.iter());
    let states_after_the_next_publish_and_the_margin =
        distinct_states_of_the_valid_roots(valid_roots_past_the_next_publish_and_the_margin.iter());
    let newest_state = states_now.first().copied();
    let publishes_after_the_newest_state = valid_roots
        .iter()
        .filter(|(root, _)| newest_state.is_none_or(|newest| root.checkpoint_txg > newest))
        .count();
    Ok(RollbackStatesOfTheRing {
        states_now: u64::try_from(states_now.len()).expect("状态数不超过根环槽数"),
        states_after_the_next_publish_and_the_margin: u64::try_from(
            states_after_the_next_publish_and_the_margin.len(),
        )
        .expect("状态数不超过根环槽数"),
        publishes_after_the_newest_state: u64::try_from(publishes_after_the_newest_state)
            .expect("根数不超过根环槽数"),
    })
}

/// 上限 = min(每块盘上最新的有效根, 第 4 新的不同状态的非空有效根)；去重之后不足 4 个时取最旧的有效根；一条有效根都没有时 `None`。
/// `distinct_states_newest_first` 是 [`newest_first_distinct_states`] 交回的那一串，`oldest_valid_root` 是有效根里最小的 txg。
fn ceiling_from_newest_and_non_empty_roots(
    newest_on_every_device: CheckpointTxg,
    distinct_states_newest_first: Vec<CheckpointTxg>,
    oldest_valid_root: Option<CheckpointTxg>,
) -> Option<CheckpointTxg> {
    let fourth_newest = distinct_states_newest_first
        .get(3)
        .copied()
        .or(oldest_valid_root)?;
    Some(newest_on_every_device.min(fourth_newest))
}

/// 这个进程分配器上那张根环表里挂载那一刻读坏（读不出，或读得出、不是全零而自证不过）或读回全零、之后没写过也没重读出有效根或被抛弃的根的槽，
/// 各按它能住的最大 txg（`allocator::RootRingOccupancy::ring_slots_bad_or_reading_back_all_zero_at_mount`：Q3 读坏的两类都并；读回全零的
/// 用户 2026-11-02 定「并」，之前 2026-10-04 定只在回收门槛里钉着）；没装根环表的分配器给空表。准入抬 F 的上限按它封顶
/// （[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]）。
#[must_use]
pub fn ring_slots_bad_or_reading_back_all_zero_at_mount_by(
    allocator: &PoolAllocator,
) -> BTreeMap<RootRingSlot, CheckpointTxg> {
    allocator
        .root_ring_occupancy()
        .map_or_else(
            BTreeMap::new,
            RootRingOccupancy::ring_slots_bad_or_reading_back_all_zero_at_mount,
        )
        .into_iter()
        .map(|(ring_slot, txgs_it_can_hold)| {
            (ring_slot, txgs_it_can_hold.highest_txg_it_can_hold())
        })
        .collect()
}

/// 这个进程分配器上那张根环表里知道住着一条根的槽（`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；
/// 没装根环表的分配器（只有用例自己拼的）给空集。挂着时管理员回退判候选读根环按它分两类；算抬 F 的上限除去记成被抛弃根的槽
/// （[`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`]）。
#[must_use]
pub fn ring_slots_known_to_hold_a_root_by(allocator: &PoolAllocator) -> BTreeSet<RootRingSlot> {
    allocator.root_ring_occupancy().map_or_else(
        BTreeSet::new,
        RootRingOccupancy::ring_slots_known_to_hold_a_root,
    )
}

/// 这个进程分配器上那张根环表里重建分配器两遍里只有一遍自证过、之后没写过的槽，各带重建那一遍读出的整条根记录
/// （`allocator::RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes`）；
/// 没装根环表的分配器给空表。抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）读它们读坏、重读一次仍坏时拿记着的那条根顶上、
/// 不拒（用户 2026-10-06 定 T1 甲，[`raise_rollback_floor`] 一族）。
#[must_use]
pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
    allocator: &PoolAllocator,
) -> BTreeMap<RootRingSlot, RootRecord> {
    allocator.root_ring_occupancy().map_or_else(
        BTreeMap::new,
        RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
    )
}

/// 这个进程分配器上那张根环表里记成被抛弃根的槽与那条根挂载时记着的落点
/// （`allocator::RootRingOccupancy::ring_slots_recorded_as_holding_an_abandoned_root`）；没装根环表的分配器给空表。
/// 抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）读它们读坏、重读仍坏不拒：上限本来不看被抛弃根，影子账照这里记着的落点算
/// （用户 2026-10-08 定 H1「被抛弃根的槽不拒」）。
#[must_use]
pub fn ring_slots_recorded_as_holding_an_abandoned_root_by(
    allocator: &PoolAllocator,
) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
    allocator.root_ring_occupancy().map_or_else(
        BTreeMap::new,
        RootRingOccupancy::ring_slots_recorded_as_holding_an_abandoned_root,
    )
}

/// 这个进程的准入抬 F 的上限，两个都交回（[`admission_ceilings_not_passing_ring_slots_bad_since_the_mount`]、[`AdmissionCeilings`]），
/// 三类槽都取这个进程分配器上那张根环表：知道住着根的槽、挂载那一刻读坏或读回全零的槽、重建分配器两遍里只有一遍自证过的槽
/// （后一类读坏重读仍坏时拿记着的那条根顶上，用户 2026-10-06 定 T1 甲）。
/// 知道住着根的槽里记成被抛弃根的那几槽不算「重读仍坏就拒」的槽（用户 2026-10-08 定 H1「被抛弃根的槽不拒」）：这一遍读坏就当没有根，
/// 上限只在有效根上取、本来不看它们；它们也不是挂载那一刻读坏或读回全零的槽，不并进上限。
///
/// 今天的 F 取现行那一版的根 `current_root` 带的 F。
///
/// 共享函数里「一条有效根都没有」那一错（`Recovery(NoValidRoot)`）在本进程准入路径上走不到（C724（准入抬 F 上限里同名拒因不在单子上），判决
/// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R2「写明走不到」）：现行那一版的根是这个进程写的，它在有效根里，
/// 要两条前提，都不是这个函数自己保证的：
/// 1. 这个进程写出的每一条现行根都经记根（`PoolAllocator::record_root_written_by_this_process`）进了根环表：它的槽是「知道住着根的槽」，
///    这一遍读坏就重读、仍坏就拒成 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，读得出就进有效根（这个实例在现行那一张实例表里没有行，
///    不会被判抛弃）；
/// 2. 根带的 F 不高于它自己的 txg（`valid` 滤的是 txg ≥ F）。
///
/// 第 2 条在这里断言。第 1 条只在分配器是这个进程挂载建的那一份时成立：公开入口（[`raise_rollback_floor_to_the_admission_ceiling`]、
/// [`raise_rollback_floor`]）收的分配器是调用方给的，不带这个进程根环表的那一种（实验装置、用例手拼的分配器）在根环全读坏时合法地走到
/// `NoValidRoot`，这里照共享函数原样交回（代码轮 `research/prompts/m3-close-b19-code-r1-main-verification.md` 第二节 T2：共享函数保留这个错，
/// 公开入口交回它、不 panic）。第 1 条破了时的断言只留在本进程准入路径那一处（[`push_one_floor_raise_within_the_admission_budget`]）。
///
/// # Errors
/// 同 [`rollback_floor_ceiling`]。
///
/// # Panics
/// `current_root` 带的 F 高于它自己的 txg。
#[allow(
    clippy::ptr_arg,
    reason = "admission_ceilings_not_passing_ring_slots_bad_since_the_mount 走 &Vec：要一个有大小的读者"
)]
pub(super) fn rollback_floor_ceilings_of_this_process<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_root: &RootRecord,
    table: &InstanceTableRecords,
    allocator: &PoolAllocator,
) -> Result<AdmissionCeilings, MountError> {
    assert!(
        current_root.rollback_floor <= current_root.checkpoint_txg,
        "现行那一版的根带的 F {:?} 高于它自己的 txg {:?}：写者给新根带的 F 取盘上已有的根与系统配置里的，新实例第一次发布的 txg 高于环里一切\
         （D23（journal 的角色与格式） 已定项 14 射程两条前提之二，C724）",
        current_root.rollback_floor,
        current_root.checkpoint_txg
    );
    allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
        admission_ceilings_not_passing_ring_slots_bad_since_the_mount(
            devices,
            reader,
            system_configuration,
            current_root.rollback_floor,
            table,
            &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
            &ring_slots_bad_or_reading_back_all_zero_at_mount_by(allocator),
            &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
                allocator,
            ),
        )
    })
}

/// 算抬 F 的上限读根环时，这一遍读坏、重读一次仍坏就拒的槽（[`rollback_floor_ceiling`] 的 `ring_slots_whose_reread_still_bad_refuses_the_ceiling`）：
/// 这个进程分配器上那张根环表里知道住着根的槽（[`ring_slots_known_to_hold_a_root_by`]），除去记成被抛弃根的那几槽
/// （[`ring_slots_recorded_as_holding_an_abandoned_root_by`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1
/// 「被抛弃根的槽不拒」：读坏就当没有根，上限只在有效根上取、本来不看它们）。这个进程的准入抬 F（[`rollback_floor_ceilings_of_this_process`]）
/// 与自算上限的调用方（实验装置、验证包的运行模式）都取这一份。没装根环表的分配器给空集。
#[must_use]
pub fn ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(
    allocator: &PoolAllocator,
) -> BTreeSet<RootRingSlot> {
    let ring_slots_recorded_as_holding_an_abandoned_root =
        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
    ring_slots_known_to_hold_a_root_by(allocator)
        .into_iter()
        .filter(|ring_slot| {
            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
        })
        .collect()
}

#[cfg(test)]
mod non_empty_root_tests {
    use super::{
        ceiling_from_newest_and_non_empty_roots, newest_first_distinct_states,
        user_visible_trees_changed, NonEmptyValidRoot,
    };
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::recovery::UserVisibleTreeRootPointers;

    fn non_empty_root(checkpoint_txg: u64, state: u8) -> NonEmptyValidRoot {
        NonEmptyValidRoot {
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            instance: InstanceGeneration(2),
            user_visible_tree_root_pointers: pointers(state, state),
        }
    }

    /// 「4 个不同状态」去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：从新到旧数，状态与更新的已计入根相同的不计入。
    /// A(3)、B(4)、C(8) 各一个状态，D(9) 是回退到 A 发出的新根、状态同 A：从新到旧计入 D、C、B，A 不计——三个不同状态，
    /// 第 4 新的没有，上限落到最旧的有效根。不去重时 A 被当成第 4 个，上限是 3。
    #[test]
    fn the_rollback_root_repeating_an_older_state_is_counted_once_and_the_ceiling_falls_to_the_oldest_valid_root(
    ) {
        let distinct = newest_first_distinct_states(vec![
            non_empty_root(3, 0xA),
            non_empty_root(4, 0xB),
            non_empty_root(8, 0xC),
            non_empty_root(9, 0xA),
        ]);
        assert_eq!(
            distinct,
            vec![CheckpointTxg(9), CheckpointTxg(8), CheckpointTxg(4)],
            "从新到旧：D（状态 A）、C、B 计入，A 与 D 同一个状态不再计"
        );
        assert_eq!(
            ceiling_from_newest_and_non_empty_roots(
                CheckpointTxg(9),
                distinct,
                Some(CheckpointTxg(1))
            ),
            Some(CheckpointTxg(1)),
            "不同状态只有 3 个：上限取最旧的有效根（根环容量边界那一格）"
        );
        let four_states = newest_first_distinct_states(vec![
            non_empty_root(3, 0xA),
            non_empty_root(4, 0xB),
            non_empty_root(8, 0xC),
            non_empty_root(9, 0xD),
        ]);
        assert_eq!(
            ceiling_from_newest_and_non_empty_roots(
                CheckpointTxg(9),
                four_states,
                Some(CheckpointTxg(1))
            ),
            Some(CheckpointTxg(3)),
            "四个不同状态：上限取第 4 新的那一条"
        );
    }

    /// 建快照那次发布算非空（用户 2026-10-14 定，D16（发布语义） 已定项 1「「非空」从盘上怎么认」）：它与前一条有效根只差快照条目的根指针
    /// （活头两棵树照抄），照样算非空；去重的元组也带快照根，两条根只差一对快照条目时是两个不同状态。删快照的意图那次只改 flags、
    /// 快照根指针不变，不算非空。
    #[test]
    fn root_that_only_adds_a_snapshot_pair_is_non_empty_and_counts_as_a_state_of_its_own() {
        let before_the_snapshot = pointers(1, 2);
        let with_the_snapshot = UserVisibleTreeRootPointers {
            inode_tree: Some(vec![1; 86]),
            extent_tree: Some(vec![2; 86]),
            snapshot_tree_roots: vec![vec![1; 86], vec![2; 86]],
        };
        assert!(
            user_visible_trees_changed(&with_the_snapshot, &before_the_snapshot),
            "活头两棵树照抄、多了一对快照根：非空"
        );
        assert!(
            !user_visible_trees_changed(&with_the_snapshot, &with_the_snapshot.clone()),
            "快照根也照抄（删快照的意图那次、只放 deadlist 的批）：不算非空"
        );
        let distinct = newest_first_distinct_states(vec![
            NonEmptyValidRoot {
                checkpoint_txg: CheckpointTxg(5),
                instance: InstanceGeneration(2),
                user_visible_tree_root_pointers: before_the_snapshot,
            },
            NonEmptyValidRoot {
                checkpoint_txg: CheckpointTxg(6),
                instance: InstanceGeneration(2),
                user_visible_tree_root_pointers: with_the_snapshot,
            },
        ]);
        assert_eq!(
            distinct,
            vec![CheckpointTxg(6), CheckpointTxg(5)],
            "只差一对快照根的两条根是两个不同状态"
        );
    }

    /// 上限取两者里小的那一个的另一半（上面那条两次都传 9，「每块盘上最新的有效根」从没胜出）：四个不同状态、第 4 新的是 3，
    /// 而有一块盘上最新的有效根只到 2（那块盘之后的根都读不出），上限取 2。只取第 4 新的那一条时上限是 3，越过那块盘上最新的有效根。
    #[test]
    fn the_ceiling_is_the_newest_valid_root_on_every_device_when_it_is_older_than_the_fourth_newest_distinct_state(
    ) {
        let four_states = newest_first_distinct_states(vec![
            non_empty_root(3, 0xA),
            non_empty_root(4, 0xB),
            non_empty_root(8, 0xC),
            non_empty_root(9, 0xD),
        ]);
        assert_eq!(
            ceiling_from_newest_and_non_empty_roots(
                CheckpointTxg(2),
                four_states,
                Some(CheckpointTxg(1))
            ),
            Some(CheckpointTxg(2)),
            "每块盘上最新的有效根 2 低于第 4 新的 3：上限取 2"
        );
    }

    fn pointers(inode_tree: u8, extent_tree: u8) -> UserVisibleTreeRootPointers {
        UserVisibleTreeRootPointers {
            inode_tree: Some(vec![inode_tree; 86]),
            extent_tree: Some(vec![extent_tree; 86]),
            snapshot_tree_roots: Vec::new(),
        }
    }

    /// 两棵树任一棵的根指针变了就算非空：只换 extent 树（同一个 inode 的内容覆盖写、inode 记录原地不动的那种发布）与只换 inode 树都要认出来，
    /// 两棵都没变的（空发布、回退那次照抄上一版的）不算；最旧的有效根与「树表里没有这两棵树的条目」比。
    #[test]
    fn root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root(
    ) {
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &pointers(1, 3)),
            "只有 extent 树的根指针变了"
        );
        assert!(
            user_visible_trees_changed(&pointers(4, 2), &pointers(1, 2)),
            "只有 inode 树的根指针变了"
        );
        assert!(
            !user_visible_trees_changed(&pointers(1, 2), &pointers(1, 2)),
            "两棵树都照抄：空发布"
        );
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &UserVisibleTreeRootPointers::ABSENT),
            "最旧的有效根下面有文件"
        );
        assert!(
            !user_visible_trees_changed(
                &UserVisibleTreeRootPointers::ABSENT,
                &UserVisibleTreeRootPointers::ABSENT
            ),
            "mkfs 的第 0 代根与暖机根：树表里没有这两棵树"
        );
    }
}

#[cfg(test)]
mod newest_valid_root_on_every_device_agrees_between_the_ceiling_and_the_effective_floor_tests {
    use super::pool_in_memory_after_the_first_file::pool_after_the_first_file;
    use super::rollback_floor_ceiling;
    use crate::address::{CheckpointTxg, DeviceIdentity};
    use crate::recovery::{
        choose_root, choose_system_configuration, effective_rollback_floor,
        instance_table_chain_of_root, read_root_ring_slot, RootRingSlotReading,
    };
    use crate::root_ring::{slot_offset, target_for_publish};
    use std::collections::BTreeSet;

    /// 第一个文件（txg 3）之后覆盖写几次：现行那一版落到 txg 12，根环（每区 8 槽、环长 24）还没转满。
    const OVERWRITES_AFTER_THE_FIRST_FILE: u64 = 9;
    /// 盘 1（区域 1，txg 除以 3 余 1）上读回全零的两条根：那块盘上最新的两条。
    const ROOTS_ON_DEVICE_ONE_READING_BACK_ZERO: [CheckpointTxg; 2] =
        [CheckpointTxg(7), CheckpointTxg(10)];
    /// 那两条读不出之后盘 1 上最新的有效根。
    const ROOT_LEFT_NEWEST_ON_DEVICE_ONE: CheckpointTxg = CheckpointTxg(4);

    /// 「每块盘上最新的有效根」两种算法连起来比（代码审阅 D 组第 7 条，`.claude/kb/milestone/03-third-txn.md` 五·10）：抬 F 的上限 [`rollback_floor_ceiling`] 取每块盘上
    /// 最新有效根的 txg 的最小值，生效的回退下界 [`effective_rollback_floor`] 取每块盘上最新有效根带的 F 的最大值。txg 12 的池上让盘 1
    /// 最新的两条根读回全零，盘 1 上最新的有效根退到 txg 4（低于第 4 新的不同状态 txg 8），再把 txg 4 那条根带的 F 改成 4（别的根都带 0）：
    /// 两边认的盘 1 上最新有效根是同一条时，上限与生效的 F 都是 4。只调两边的公开入口。
    #[test]
    fn the_ceiling_and_the_effective_floor_take_the_same_root_as_the_newest_valid_one_on_a_device()
    {
        let mut pool = pool_after_the_first_file();
        for _overwrite in 0..OVERWRITES_AFTER_THE_FIRST_FILE {
            pool.overwrite();
        }
        assert_eq!(
            pool.current.root.checkpoint_txg,
            CheckpointTxg(12),
            "前提：第一个文件 txg 3，之后覆盖写 9 次落到 txg 12"
        );
        let sizes = pool.parameters.geometry;
        let region_devices = pool.parameters.region_devices;
        let filesystem_identifier = pool.parameters.filesystem_identifier;
        let root_slot_bytes = usize::try_from(sizes.physical_block_size).expect("根槽宽");
        for checkpoint_txg in ROOTS_ON_DEVICE_ONE_READING_BACK_ZERO
            .into_iter()
            .chain([ROOT_LEFT_NEWEST_ON_DEVICE_ONE])
        {
            let ring_slot = target_for_publish(checkpoint_txg, sizes.root_ring_slots_per_region);
            assert_eq!(
                region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                DeviceIdentity(1),
                "前提：txg {checkpoint_txg:?} 的根落在盘 1"
            );
        }
        for checkpoint_txg in ROOTS_ON_DEVICE_ONE_READING_BACK_ZERO {
            let ring_slot = target_for_publish(checkpoint_txg, sizes.root_ring_slots_per_region);
            pool.write_bytes_directly_on(
                DeviceIdentity(1),
                slot_offset(ring_slot, sizes.fixed_structure_slot_spacing),
                &vec![0; root_slot_bytes],
            );
        }
        let ring_slot_of_the_root_left_newest = target_for_publish(
            ROOT_LEFT_NEWEST_ON_DEVICE_ONE,
            sizes.root_ring_slots_per_region,
        );
        let RootRingSlotReading::SelfVerified(mut root_left_newest) = read_root_ring_slot(
            &pool.devices,
            &region_devices,
            &sizes,
            &filesystem_identifier,
            ring_slot_of_the_root_left_newest,
        ) else {
            panic!("前提：txg 4 那条根在它的根环槽里自证过");
        };
        root_left_newest.rollback_floor = root_left_newest.checkpoint_txg;
        pool.write_bytes_directly_on(
            DeviceIdentity(1),
            slot_offset(
                ring_slot_of_the_root_left_newest,
                sizes.fixed_structure_slot_spacing,
            ),
            &root_left_newest.to_slot(root_slot_bytes),
        );
        let system_configuration =
            choose_system_configuration(&pool.devices).expect("两块盘的系统配置读得出");
        let newest_root =
            choose_root(&pool.devices, &system_configuration).expect("根环里有自证过的根");
        let newest_roots_table = instance_table_chain_of_root(&pool.devices, &newest_root)
            .expect("最新那条根的实例表读得出")
            .records;
        let ceiling = rollback_floor_ceiling(
            &pool.devices,
            &system_configuration,
            CheckpointTxg(0),
            &newest_roots_table,
            &BTreeSet::new(),
        )
        .expect("有效根都读得出树表：上限算得出");
        let effective_floor = effective_rollback_floor(
            &pool.devices,
            &region_devices,
            &sizes,
            &filesystem_identifier,
        );
        assert_eq!(
            ceiling, ROOT_LEFT_NEWEST_ON_DEVICE_ONE,
            "上限取盘 1 上最新的有效根 txg 4（盘 0 最新 txg 12，第 4 新的不同状态 txg 8）"
        );
        assert_eq!(
            effective_floor, ceiling,
            "生效的 F 取盘 1 上最新有效根带的 4：两边认的是同一条根"
        );
    }
}
