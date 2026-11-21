//! 重建分配器读根环：读一个槽时把读回全零单分出来、两遍逐槽并成一份、只有一遍自证过的槽，
//! 挂载那一刻读坏的槽能住哪几个 txg，根环读一遍或两遍与读数分好的几类。

use super::*;

/// 挂载重建分配器与进程内重读时读一个根环槽读到了什么：在 [`read_root_ring_slot`] 的分法之上，把读回全零的槽单分出来
/// （回收门槛里照「自证不过」那一类钉住，一次读回全零的槽盘上可能住着一条有效根；准入抬 F 的上限同读坏的槽一样并它）。
/// 择根与算上限那一遍用 [`read_root_ring_slot`]；清扫（`instance_table_sweep`）自己逐槽解析，调读者同一个落点核
/// [`crate::root_ring::root_sits_in_the_ring_slot_the_formula_gives`]，全零与读不出同待遇（D18（块里携带什么信息） 已定项 11）。
#[allow(
    clippy::large_enum_variant,
    reason = "一次读一个槽、按值交回马上拆开，装箱只多一次分配"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootRingSlotReadingTellingNeverWrittenApart {
    SelfVerified(RootRecord),
    /// 设备报读错，或读得出、不是全零而自证不过（校验和不过、fsid 不是本池的），或自证得过而不住在它的 txg 按落点公式该住的那一槽
    /// （错位读，C743；第 0 代根照 mkfs 种根的槽另认）。
    BadAndNotAllZero,
    /// 读得出、每个字节都是 0：多半从没写过，也可能是一次读坏的有效根（回收门槛照这一种钉住）。
    AllZeroNeverWritten,
}

/// 读一个根环槽，只读一次、不重试：读不出、全零、自证过与否按 [`RootRingSlotReadingTellingNeverWrittenApart`] 分。偏移与盘的取法与
/// [`read_root_ring_slot`] 相同（区域归属表给出盘、槽距给出偏移）；要看字节是不是全零，所以不经它读（经它再读一次，两次之间的瞬时读错
/// 会让两个读数对不上）。
pub(super) fn read_root_ring_slot_telling_never_written_apart<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slot: RootRingSlot,
) -> RootRingSlotReadingTellingNeverWrittenApart {
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset =
        crate::root_ring::slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
    let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
        return RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero;
    };
    if bytes.iter().all(|byte| *byte == 0) {
        return RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten;
    }
    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
        // 记新类之前核落点公式（C743）：自证得过、却不住在它的 txg 该住的那一槽的根（错位读，第 0 代根照 mkfs 种根的槽另认）
        // 照挂载那一刻读坏记，两遍逐槽并（`better_reading_of_a_root_ring_slot_over_the_two_passes`）与「两遍里只有一遍自证过」
        // 那一类（`ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes`）都不收它。
        Some(root)
            if !root_sits_in_the_ring_slot_the_formula_gives(
                root.checkpoint_txg,
                ring_slot,
                immutable_sizes.root_ring_slots_per_region,
            ) =>
        {
            RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
        }
        Some(root) => RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root),
        None => RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero,
    }
}

/// 一个根环槽两遍读数里较好的那一份（用户 2026-10-05 定 R1「按槽并两遍」，判决 `research/prompts/m3-batch10-code-r2-main-verification.md`
/// 第六节）：任一遍自证过的根（两遍都自证过取重读那一条）＞ 任一遍读坏、不是全零 ＞ 全零。
/// 自证过的根不是读错造得出来的（整槽校验和过、fsid 是本池的），挂载期间又没有别的写者，第一遍读出的那一条重读时仍住在那一槽：
/// 只拿重读那一遍、丢掉第一遍读出的根，第一遍读得出、重读读坏的被抛弃根会掉出影子账，它独占的单元当场发得出去
/// （攻方第二次抽样 `research/prompts/m3-batch10-code-r2-opus-s2-output.md` K2-2，违反 D23（journal 的角色与格式） 已定项 14
/// 「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」）。读坏压过全零：读坏那一类回收门槛钉住、准入抬 F 的上限也并，
/// 全零那一类只在回收门槛里钉住（用户 2026-10-05 定乙），两类里读坏的约束多。
fn better_reading_of_a_root_ring_slot_over_the_two_passes(
    first_reading: RootRingSlotReadingTellingNeverWrittenApart,
    reading_of_the_reread: RootRingSlotReadingTellingNeverWrittenApart,
) -> RootRingSlotReadingTellingNeverWrittenApart {
    use RootRingSlotReadingTellingNeverWrittenApart::{
        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
    };
    match (first_reading, reading_of_the_reread) {
        (SelfVerified(_), SelfVerified(root_of_the_reread)) => SelfVerified(root_of_the_reread),
        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {
            SelfVerified(root_of_the_reread)
        }
        (SelfVerified(root_of_the_first_pass), BadAndNotAllZero | AllZeroNeverWritten) => {
            SelfVerified(root_of_the_first_pass)
        }
        (BadAndNotAllZero, BadAndNotAllZero) => BadAndNotAllZero,
        (BadAndNotAllZero, AllZeroNeverWritten) | (AllZeroNeverWritten, BadAndNotAllZero) => {
            BadAndNotAllZero
        }
        (AllZeroNeverWritten, AllZeroNeverWritten) => AllZeroNeverWritten,
    }
}

/// 重建分配器读根环的两遍（[`every_root_ring_slot`] 的次序，每槽各读一次）逐槽并成一份
/// （[`better_reading_of_a_root_ring_slot_over_the_two_passes`]）。两遍按同一个次序读同一组槽，槽对不上是这里的 bug、断言。
fn merged_readings_of_the_two_root_ring_passes(
    first_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
    readings_of_the_reread: Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)>,
) -> Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> {
    assert_eq!(
        first_readings.len(),
        readings_of_the_reread.len(),
        "两遍都按 every_root_ring_slot 读整个环，槽数相同"
    );
    first_readings
        .iter()
        .zip(readings_of_the_reread)
        .map(
            |((ring_slot_of_the_first_pass, first_reading), (ring_slot, reading_of_the_reread))| {
                assert_eq!(
                    *ring_slot_of_the_first_pass, ring_slot,
                    "两遍都按 every_root_ring_slot 的次序读，第 n 个读数是同一个槽"
                );
                (
                    ring_slot,
                    better_reading_of_a_root_ring_slot_over_the_two_passes(
                        *first_reading,
                        reading_of_the_reread,
                    ),
                )
            },
        )
        .collect()
}

/// 重建分配器读根环的两遍里只有一遍自证过的槽（另一遍读不出、读得出而自证不过或读回全零）：并出来是那一遍读出的根
/// （[`better_reading_of_a_root_ring_slot_over_the_two_passes`]），根环表把它另记一类（用户 2026-10-06 定 S1，
/// `allocator::RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）。两遍按同一个次序读同一组槽，槽对不上是这里的 bug、断言。
fn ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
    first_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
    readings_of_the_reread: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
) -> BTreeSet<RootRingSlot> {
    use RootRingSlotReadingTellingNeverWrittenApart::{
        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
    };
    assert_eq!(
        first_readings.len(),
        readings_of_the_reread.len(),
        "两遍都按 every_root_ring_slot 读整个环，槽数相同"
    );
    first_readings
        .iter()
        .zip(readings_of_the_reread)
        .filter_map(
            |((ring_slot_of_the_first_pass, first_reading), (ring_slot, reading_of_the_reread))| {
                assert_eq!(
                    ring_slot_of_the_first_pass, ring_slot,
                    "两遍都按 every_root_ring_slot 的次序读，第 n 个读数是同一个槽"
                );
                match (first_reading, reading_of_the_reread) {
                    (SelfVerified(_), BadAndNotAllZero | AllZeroNeverWritten)
                    | (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => Some(*ring_slot),
                    (SelfVerified(_), SelfVerified(_))
                    | (
                        BadAndNotAllZero | AllZeroNeverWritten,
                        BadAndNotAllZero | AllZeroNeverWritten,
                    ) => None,
                }
            },
        )
        .collect()
}

/// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的根环槽各能住哪几个 txg（区域内槽位只由 txg 定，
/// D22（单元原子性怎么合成） 已定项 2）：
/// - 可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg；mkfs 种第 0 代根的那几个槽取第 0 代根的 txg
///   （[`oldest_txg_a_ring_slot_bad_at_mount_can_have_held`]）。回收门槛钉在这里（用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里
///   钉在它可能住过的最旧 txg，不再只在一圈窗口里取——窗口之外、崩在记录之后根槽之前留在洞里的旧根也可能就住在这一槽里）。
/// - 能住的最大 txg：(N − 3S, first_txg − 1] 里落到这个槽的最大 txg（D16（发布语义） 已定项 1「根环表记最大的」，
///   `research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行）：准入抬 F 的上限照 Q3 按它并。上沿取新实例第一次发布之前的那个 txg：
///   读坏的槽里可能住着比环里读得出的最新根还新的根（被藏住的最新根、由记录施加出来的那一版那次发布的根）。
///
/// 一个槽在 [0, first_txg) 里一个 txg 都住不了（年轻的环里还没轮到的槽）就不交回。挂载时读回全零的槽也经它算（`rebuilt_allocator`），
/// 两个数都用：回收门槛照「自证不过」那一类钉在可能住过的最旧 txg（用户 2026-10-05 定乙），准入抬 F 的上限按能住的最大 txg 并
/// （用户 2026-11-02 定「并」）。
///
/// 循环：最大 txg 只走区间尾 min(3S, 区间长) 个 txg（与 first_txg 多大无关：外来记录的 txg 可以贴着 `u64::MAX`），最旧 txg 只走
/// [0, min(3S, first_txg)) ——`target_for_publish` 只看 txg mod 3S，一个槽最小的 txg 落在第一个周期里、区间里最大的 txg 落在区间尾一个周期里；
/// 轮数至多 6S，跨轮只往按槽的表里写；没有提前出口。
fn txgs_ring_slots_bad_at_mount_can_hold(
    bad_ring_slots: &BTreeSet<RootRingSlot>,
    window_anchor: CheckpointTxg,
    first_txg: CheckpointTxg,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
) -> Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> {
    let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
    let lowest_txg_inside_the_window = window_anchor
        .0
        .checked_sub(ring_length_in_txgs)
        .map_or(0, |below_the_window| below_the_window + 1);
    let interval_length_in_txgs = first_txg.0.saturating_sub(lowest_txg_inside_the_window);
    let txgs_walked_at_the_tail_of_the_interval = interval_length_in_txgs.min(ring_length_in_txgs);
    let mut highest_txg_of_each_bad_slot: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
    // 从小到大走，最后一次碰到的是最大值。
    for txg in first_txg.0 - txgs_walked_at_the_tail_of_the_interval..first_txg.0 {
        let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
        if bad_ring_slots.contains(&ring_slot) {
            highest_txg_of_each_bad_slot.insert(ring_slot, CheckpointTxg(txg));
        }
    }
    let mut oldest_txg_of_each_bad_slot: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
    // 从小到大走，第一次碰到的是最小值。
    for txg in 0..first_txg.0.min(ring_length_in_txgs) {
        let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
        if bad_ring_slots.contains(&ring_slot) {
            oldest_txg_of_each_bad_slot
                .entry(ring_slot)
                .or_insert(CheckpointTxg(txg));
        }
    }
    oldest_txg_of_each_bad_slot
        .into_iter()
        .map(|(ring_slot, oldest_txg_by_the_formula)| {
            let highest_txg_it_can_hold = *highest_txg_of_each_bad_slot.get(&ring_slot).expect(
                "这个槽在 [0, first_txg) 里住得下一个 txg，区间 (N − 3S, first_txg − 1] 要么从 0 起、要么长过一圈，它在区间里也住得下一个",
            );
            (
                ring_slot,
                TxgsARingSlotBadAtMountCanHold::new(
                    oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
                        ring_slot,
                        oldest_txg_by_the_formula,
                    ),
                    highest_txg_it_can_hold,
                ),
            )
        })
        .collect()
}

/// 挂载那一刻读坏或读回全零、公式在 [0, first_txg) 里落得到的一个槽可能住过的最旧 txg（回收门槛钉在这里）：mkfs 种第 0 代根的那几个槽
/// （三个区域的槽 0，[`crate::make_filesystem::genesis_root_ring_slots`]）取第 0 代根的 txg——区域 1、2 的槽 0 照公式要等 txg 1、2 才被盖掉，
/// 那两次的根槽写崩在记录之后、根槽之前时第 0 代根就留在那里（C773（读坏与全零根槽的回收门槛钉只在内存里），判决
/// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R1 采乙「第 0 代根所在槽的钉值取 0」）；别的槽取公式落到它的最小 txg
/// `oldest_txg_by_the_formula`。公式在 [0, first_txg) 里一个 txg 都落不到的槽（年轻的环里还没轮到的）照旧不交回、不钉：
/// 那时第 0 代根还在区域 0 槽 0，那一槽读得出门槛就是 0、读坏就钉在 0。
fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
    ring_slot: RootRingSlot,
    oldest_txg_by_the_formula: CheckpointTxg,
) -> CheckpointTxg {
    if crate::make_filesystem::genesis_root_ring_slots()
        .any(|genesis_slot| genesis_slot == ring_slot)
    {
        crate::make_filesystem::GENESIS_CHECKPOINT_TXG
    } else {
        oldest_txg_by_the_formula
    }
}

/// 根环每一槽各读一次（影子账、回收门槛与分配器那张根环表读的是同一遍：两遍之间的瞬时读错会让几边看见的根不一样）。
#[allow(
    clippy::ptr_arg,
    reason = "根环读走 &Vec 那一份读者：Vec 实现了 PoolReader、切片不能转成 dyn"
)]
fn every_root_ring_slot_read_once<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> {
    every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .map(|ring_slot| {
            (
                ring_slot,
                read_root_ring_slot_telling_never_written_apart(
                    devices,
                    &system_configuration.immutable.region_devices,
                    &system_configuration.immutable.sizes,
                    &system_configuration.immutable.filesystem_identifier,
                    ring_slot,
                ),
            )
        })
        .collect()
}

/// 窗口锚：一遍读数里不被抛弃的自证有效根的最大 txg（算挂载时读坏、读回全零的槽能住哪几个 txg 用它）。
fn newest_valid_root_txg_of_the_readings(
    readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
    newest_table: &InstanceTableRecords,
) -> Option<CheckpointTxg> {
    readings
        .iter()
        .filter_map(|(_, reading)| match reading {
            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
                (!abandoned_by_table(root, newest_table)).then_some(root.checkpoint_txg)
            }
            RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
            | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
        })
        .max()
}

/// 一遍读数里每条自证有效根与它归的那块盘。一条根归哪块盘按读出它的那个槽所在区域的 `region_devices` 定（判「每块盘都读出了有效根」与
/// 当场放开到哪儿取 m 都用它，同一处定义）。不按 txg 的落点公式（`target_for_publish`）：mkfs 在三个区域的槽 0 各写一份第 0 代根
/// （`make_filesystem::genesis_root_ring_slots`），公式把三份都归到盘 0，环里只剩第 0 代根时盘 1 实际读出了有效根、却算成一条都没有，
/// rr2 照样整环重读（判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第二节 K2 第一、二、三格）。别的根都按公式写进槽，
/// 两种归法只在第 0 代根上不同。
pub(super) fn roots_on_the_devices_they_were_read_from(
    readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Vec<(DeviceIdentity, RootRecord)> {
    readings
        .iter()
        .filter_map(|(ring_slot, reading)| match reading {
            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => Some((
                system_configuration.immutable.region_devices
                    [usize::try_from(ring_slot.region).expect("区域号")],
                *root,
            )),
            RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
            | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
        })
        .collect()
}

/// 根环整环读一遍；这一遍有一块盘一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就整环再读一遍（用户 2026-10-05 定 Q4 rr2，
/// 岔路单 `research/prompts/m3-batch10-code-r1-forks.md` Q4 那一行：改之前只在整个环一条都没读出时重读，只坏这一遍、只有一块盘没读出的那一形
/// m 取不到、当场只放到环里最旧有效根，窄盘上取不到落点被拒；重读之前调 `before_the_one_reread`，与另几处重读一样），
/// 根环表、回收门槛与影子账都用两遍逐槽并起来的那一份（用户 2026-10-05 定 R1「按槽并两遍」，[`merged_readings_of_the_two_root_ring_passes`]：
/// 只拿重读那一遍时，第一遍读出、重读读坏的被抛弃根掉出影子账；并起来那一份仍有一块盘没读出的，照 m 取不到那一支往下走）。
/// 两遍里只有一遍自证过的槽另记一类（用户 2026-10-06 定 S1，`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）：
/// 并出来照读出来的根算，抬 F 那一串读它仍坏时拿读出来的那条根顶上、不拒（用户 2026-10-06 定 T1 甲）。
///
/// # Errors
/// 两遍并起来整个环还是一条都没有 ⇒ `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，在任何写之前（用户 2026-10-05 定「重读」，
/// 岔路单 `research/prompts/m3-batch17-code-r1-forks.md` 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行
/// 与第四节；拒因另起成员，Q3；R1 之后第一遍读出过一条有效根的不拒）。不重读、给空表往下走时，写行那次记根算回收门槛只看得到它自己，
/// 释放代高于 F_生效 的一截当场放开、被暖机写掉，盘上环里更旧的有效根还引用着（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）。
#[allow(
    clippy::ptr_arg,
    reason = "根环读走 &Vec 那一份读者：Vec 实现了 PoolReader、切片不能转成 dyn"
)]
pub(super) fn root_ring_read_once_or_twice_for_the_rebuild<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    newest_table: &InstanceTableRecords,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<RootRingReadForTheRebuild, MountError> {
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let first_readings = every_root_ring_slot_read_once(devices, system_configuration);
    // 这一遍每块盘都读出了一条不被抛弃的自证有效根（与当场放开到哪儿那一步取 m 同一处定义 [`newest_valid_root_txg_on_every_device`]）。
    let some_valid_root_was_read_on_every_device = newest_valid_root_txg_on_every_device(
        &roots_on_the_devices_they_were_read_from(&first_readings, system_configuration),
        newest_table,
        &all_devices,
    )
    .is_some();
    if let Some(window_anchor) =
        newest_valid_root_txg_of_the_readings(&first_readings, newest_table)
            .filter(|_| some_valid_root_was_read_on_every_device)
    {
        return Ok(RootRingReadForTheRebuild {
            ring_slot_readings: first_readings,
            window_anchor,
            read: RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
            ring_slots_self_verified_on_only_one_of_the_two_passes: BTreeSet::new(),
        });
    }
    before_the_one_reread.before_rereading();
    let readings_of_the_reread = every_root_ring_slot_read_once(devices, system_configuration);
    let ring_slots_self_verified_on_only_one_of_the_two_passes =
        ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
            &first_readings,
            &readings_of_the_reread,
        );
    let readings_of_the_two_passes_merged =
        merged_readings_of_the_two_root_ring_passes(&first_readings, readings_of_the_reread);
    let Some(window_anchor) =
        newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged, newest_table)
    else {
        return Err(MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread);
    };
    Ok(RootRingReadForTheRebuild {
        ring_slot_readings: readings_of_the_two_passes_merged,
        window_anchor,
        read: RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
        ring_slots_self_verified_on_only_one_of_the_two_passes,
    })
}

impl RootRingReadingsSortedForTheRebuild {
    /// 挂载时读坏的槽能住哪几个 txg（[`txgs_ring_slots_bad_at_mount_can_hold`]）：N = 环里读得出的最新有效根的 txg（D16（发布语义） 已定项 1
    /// 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1）。读回全零的槽按同一个算法另算一份、两个数都留着：
    /// 回收门槛里照「自证不过」那一类钉在可能住过的最旧 txg（用户 2026-10-05 定乙），准入抬 F 的上限同读坏的槽一样按能住的最大 txg 并
    /// （D16（发布语义） 已定项 1「抬 F 的上限」那一行，用户 2026-11-02 定「并」）。
    /// 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。窗口锚由读根环那一步取到（有一块盘没读出就重读、两遍并起来整个环仍取不到就拒），
    /// 两类不会因为它是空表。
    pub(super) fn of(
        ring_slot_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
        window_anchor: CheckpointTxg,
        first_txg: CheckpointTxg,
        slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
    ) -> Self {
        let roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = ring_slot_readings
            .iter()
            .filter_map(|(ring_slot, reading)| match reading {
                RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
                    Some((*ring_slot, *root))
                }
                RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
                | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
            })
            .collect();
        let bad_ring_slots_not_all_zero: BTreeSet<RootRingSlot> = ring_slot_readings
            .iter()
            .filter_map(|(ring_slot, reading)| match reading {
                RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => Some(*ring_slot),
                RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_)
                | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
            })
            .collect();
        let ring_slots_reading_back_all_zero: BTreeSet<RootRingSlot> = ring_slot_readings
            .iter()
            .filter_map(|(ring_slot, reading)| match reading {
                RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {
                    Some(*ring_slot)
                }
                RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_)
                | RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => None,
            })
            .collect();
        let roots: Vec<RootRecord> = roots_with_ring_slots
            .iter()
            .map(|(_, root)| *root)
            .collect();
        let ring_slots_bad_at_mount = txgs_ring_slots_bad_at_mount_can_hold(
            &bad_ring_slots_not_all_zero,
            window_anchor,
            first_txg,
            slots_per_region,
        );
        let ring_slots_reading_back_all_zero_at_mount = txgs_ring_slots_bad_at_mount_can_hold(
            &ring_slots_reading_back_all_zero,
            window_anchor,
            first_txg,
            slots_per_region,
        );
        Self {
            roots_with_ring_slots,
            roots,
            ring_slots_bad_at_mount,
            ring_slots_reading_back_all_zero_at_mount,
        }
    }
}

#[cfg(test)]
mod txgs_ring_slots_bad_at_mount_can_hold_tests {
    use super::txgs_ring_slots_bad_at_mount_can_hold;
    use crate::address::CheckpointTxg;
    use crate::allocator::TxgsARingSlotBadAtMountCanHold;
    use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
    use singlefs_format::ROOT_RING_REGIONS;
    use std::collections::{BTreeMap, BTreeSet};

    /// 两个槽之外环里每个槽都读坏（两个槽读出了根）。
    fn bad_ring_slots_but_two(slots_per_region: RootRingSlotsPerRegion) -> BTreeSet<RootRingSlot> {
        let ring_slots_read = [
            RootRingSlot { region: 0, slot: 0 },
            RootRingSlot { region: 1, slot: 2 },
        ];
        (0..ROOT_RING_REGIONS)
            .flat_map(|region| {
                (0..slots_per_region.count()).map(move |slot| RootRingSlot { region, slot })
            })
            .filter(|ring_slot| !ring_slots_read.contains(ring_slot))
            .collect()
    }

    /// 参照：[0, first_txg) 与区间 (N − 3S, first_txg − 1] 里的 txg 一个一个走（只给区间装得下的组用）。可能住过的最旧 txg：
    /// 槽号为 0 的槽（mkfs 在每个区域的槽 0 种一条第 0 代根，txg 0）是 0，别的槽是走到的第一个 txg（C773 判决 R1 乙；这里按槽号另写一份，
    /// 不调实现认第 0 代根槽的那一处）。
    fn walk_every_txg_below_the_first(
        bad_ring_slots: &BTreeSet<RootRingSlot>,
        window_anchor: CheckpointTxg,
        first_txg: CheckpointTxg,
        slots_per_region: RootRingSlotsPerRegion,
    ) -> Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> {
        let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
        let lowest_txg_inside_the_window = window_anchor
            .0
            .checked_sub(ring_length_in_txgs)
            .map_or(0, |below_the_window| below_the_window + 1);
        let mut oldest_of_each: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
        let mut highest_of_each: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
        for txg in 0..first_txg.0 {
            let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
            if !bad_ring_slots.contains(&ring_slot) {
                continue;
            }
            oldest_of_each
                .entry(ring_slot)
                .or_insert(if ring_slot.slot == 0 {
                    CheckpointTxg(0)
                } else {
                    CheckpointTxg(txg)
                });
            if txg >= lowest_txg_inside_the_window {
                highest_of_each.insert(ring_slot, CheckpointTxg(txg));
            }
        }
        oldest_of_each
            .into_iter()
            .map(|(ring_slot, oldest)| {
                (
                    ring_slot,
                    TxgsARingSlotBadAtMountCanHold::new(
                        oldest,
                        *highest_of_each
                            .get(&ring_slot)
                            .expect("住得下就在区间里也住得下"),
                    ),
                )
            })
            .collect()
    }

    /// 锚 40、first_txg 41（S = 4，区间 [29, 40] 正好一圈）：区域 2 槽 3 能住的最大 txg 是区间里余 11 的 35，
    /// 可能住过的最旧 txg 是第一圈的 11——回收门槛钉在 11，不在窗口里的 35（用户 2026-10-04 定）。
    #[test]
    fn bad_slot_is_pinned_at_the_oldest_txg_it_can_have_held_below_the_window() {
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
            .expect("4 是 S 区间 [4, 16] 的下界");
        let txgs = txgs_ring_slots_bad_at_mount_can_hold(
            &BTreeSet::from([RootRingSlot { region: 2, slot: 3 }]),
            CheckpointTxg(40),
            CheckpointTxg(41),
            slots_per_region,
        );
        assert_eq!(
            txgs,
            vec![(
                RootRingSlot { region: 2, slot: 3 },
                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(11), CheckpointTxg(35))
            )]
        );
    }

    /// 第 0 代根留过的槽（C773（读坏与全零根槽的回收门槛钉只在内存里），判决 `research/prompts/m3-close-b19-core-r1-main-verification.md`
    /// 第二节 R1 乙）：first_txg 14、S = 4，区域 1 槽 0 与区域 2 槽 0 落点公式最早住 txg 1、2，mkfs 却在那里种过第 0 代根（txg 0）：
    /// 可能住过的最旧 txg 取 0；能住的最大 txg 照公式（13、2——区间 [2, 13] 里落到区域 2 槽 0 的只有 2）。区域 0 槽 1 照公式 3。
    #[test]
    fn bad_ring_slot_where_make_filesystem_planted_a_genesis_root_is_pinned_at_txg_zero() {
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
            .expect("4 是 S 区间 [4, 16] 的下界");
        assert_eq!(
            txgs_ring_slots_bad_at_mount_can_hold(
                &BTreeSet::from([
                    RootRingSlot { region: 0, slot: 1 },
                    RootRingSlot { region: 1, slot: 0 },
                    RootRingSlot { region: 2, slot: 0 },
                ]),
                CheckpointTxg(13),
                CheckpointTxg(14),
                slots_per_region,
            ),
            vec![
                (
                    RootRingSlot { region: 0, slot: 1 },
                    TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(3), CheckpointTxg(3))
                ),
                (
                    RootRingSlot { region: 1, slot: 0 },
                    TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(0), CheckpointTxg(13))
                ),
                (
                    RootRingSlot { region: 2, slot: 0 },
                    TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(0), CheckpointTxg(2))
                ),
            ]
        );
    }

    /// 年轻的环（first_txg 5，S = 4）：txg 5 起才轮到的槽一个 txg 都住不了，不交回。
    #[test]
    fn bad_slot_the_young_ring_has_not_reached_holds_no_txg() {
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
            .expect("4 是 S 区间 [4, 16] 的下界");
        let not_reached = target_for_publish(CheckpointTxg(7), slots_per_region);
        let reached = target_for_publish(CheckpointTxg(4), slots_per_region);
        assert_eq!(
            txgs_ring_slots_bad_at_mount_can_hold(
                &BTreeSet::from([not_reached, reached]),
                CheckpointTxg(4),
                CheckpointTxg(5),
                slots_per_region,
            ),
            vec![(
                reached,
                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(4), CheckpointTxg(4))
            )]
        );
    }

    /// 区间短于 3S、正好 3S、长于 3S 若干圈、年轻的环与空区间几组逐项比参照；first_txg 是 `u64::MAX` 那一组（外来记录的 txg 是
    /// `u64::MAX − 1`）参照走不完，比的是按 3S 周期手算的期望。
    #[test]
    fn walking_one_period_at_each_end_matches_walking_every_txg_below_the_first() {
        for declared_slots_per_region in [4, 5] {
            let slots_per_region =
                RootRingSlotsPerRegion::from_system_configuration_field(declared_slots_per_region)
                    .expect("4 与 5 落在 S 的区间 [4, 16] 里");
            let ring_length_in_txgs = ROOT_RING_REGIONS * declared_slots_per_region;
            let bad_ring_slots = bad_ring_slots_but_two(slots_per_region);
            let anchor_above_one_revolution = 40;
            let lowest_txg_inside_the_window =
                anchor_above_one_revolution - ring_length_in_txgs + 1;
            let anchor_and_first_txg_of_each_group = [
                (anchor_above_one_revolution, anchor_above_one_revolution + 1),
                (
                    anchor_above_one_revolution,
                    lowest_txg_inside_the_window + ring_length_in_txgs + 5,
                ),
                (
                    anchor_above_one_revolution,
                    lowest_txg_inside_the_window + 2 * ring_length_in_txgs - 1,
                ),
                (
                    anchor_above_one_revolution,
                    lowest_txg_inside_the_window + 4 * ring_length_in_txgs + 7,
                ),
                (5, 6),
                (5, 9),
                (5, 5 * ring_length_in_txgs + 3),
                (0, 0),
            ];
            for (window_anchor, first_txg) in anchor_and_first_txg_of_each_group {
                assert_eq!(
                    txgs_ring_slots_bad_at_mount_can_hold(
                        &bad_ring_slots,
                        CheckpointTxg(window_anchor),
                        CheckpointTxg(first_txg),
                        slots_per_region,
                    ),
                    walk_every_txg_below_the_first(
                        &bad_ring_slots,
                        CheckpointTxg(window_anchor),
                        CheckpointTxg(first_txg),
                        slots_per_region,
                    ),
                    "S = {declared_slots_per_region}，窗口锚 = {window_anchor}，first_txg = {first_txg}"
                );
            }
        }

        // S = 4，3S = 12：txg 模 12 的余数 r 落在区域 r mod 3 的槽 r div 3（D22 已定项 2 的槽位公式在一个周期里展开），
        // 可能住过的最旧 txg 就是 r（槽 0 那三个余数 0、1、2 是第 0 代根的 0）。2^64 模 12 是 4，所以 u64::MAX 模 12 是 3：区间尾 u64::MAX − 12 ..= u64::MAX − 1 里余数是 r 的那个
        // txg 是 u64::MAX − k，k 取 1..=12 里 (3 − k) 模 12 等于 r 的那一个。
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
            .expect("4 是 S 区间 [4, 16] 的下界");
        let ring_length_in_txgs = 12;
        assert_eq!(u64::MAX % ring_length_in_txgs, 3);
        let bad_ring_slots = bad_ring_slots_but_two(slots_per_region);
        let mut expected: Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> = (0
            ..ring_length_in_txgs)
            .map(|residue| {
                let distance_modulo_the_ring =
                    (3 + ring_length_in_txgs - residue) % ring_length_in_txgs;
                let distance_below_u64_maximum = if distance_modulo_the_ring == 0 {
                    ring_length_in_txgs
                } else {
                    distance_modulo_the_ring
                };
                (
                    RootRingSlot {
                        region: residue % 3,
                        slot: residue / 3,
                    },
                    TxgsARingSlotBadAtMountCanHold::new(
                        CheckpointTxg(if residue / 3 == 0 { 0 } else { residue }),
                        CheckpointTxg(u64::MAX - distance_below_u64_maximum),
                    ),
                )
            })
            .filter(|(ring_slot, _)| bad_ring_slots.contains(ring_slot))
            .collect();
        expected.sort_by_key(|(ring_slot, _)| *ring_slot);
        assert_eq!(
            txgs_ring_slots_bad_at_mount_can_hold(
                &bad_ring_slots,
                CheckpointTxg(40),
                CheckpointTxg(u64::MAX),
                slots_per_region,
            ),
            expected
        );
    }
}

#[cfg(test)]
mod merged_readings_of_the_two_root_ring_passes_tests {
    use super::{
        better_reading_of_a_root_ring_slot_over_the_two_passes,
        merged_readings_of_the_two_root_ring_passes, RootRingSlotReadingTellingNeverWrittenApart,
    };
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};
    use crate::root_ring::RootRingSlot;
    use RootRingSlotReadingTellingNeverWrittenApart::{
        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
    };

    fn root_of(instance: u32, checkpoint_txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [0; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: crate::address::CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    /// 场景：一个根环槽两遍读数的九种组合（第一遍、重读各取自证过、读坏不是全零、全零三种），两遍都自证过时两遍读出的是两条不同的根
    /// （只为看出取的是哪一遍的，挂载里同一槽两遍读出的是同一条）。
    /// 预期：照用户 2026-10-05 定 R1 的次序——任一遍自证过的根（两遍都自证过取重读那一条）＞ 任一遍读坏不是全零 ＞ 全零。
    /// 判别力：`crates/mutations.tsv` 里「第10h批 R1 …」那几行各改一处判别，各红在点名那一组上。
    #[test]
    fn each_pair_of_readings_merges_into_the_self_verified_root_then_the_bad_reading_then_all_zero()
    {
        let root_of_the_first_pass = root_of(1, 7);
        let root_of_the_reread = root_of(1, 31);
        let pairs_and_merged = [
            (
                SelfVerified(root_of_the_first_pass),
                SelfVerified(root_of_the_reread),
                SelfVerified(root_of_the_reread),
                "两遍都自证过：取重读那一条",
            ),
            (
                BadAndNotAllZero,
                SelfVerified(root_of_the_reread),
                SelfVerified(root_of_the_reread),
                "第一遍读坏、重读自证过：取重读那一条",
            ),
            (
                AllZeroNeverWritten,
                SelfVerified(root_of_the_reread),
                SelfVerified(root_of_the_reread),
                "第一遍全零、重读自证过：取重读那一条",
            ),
            (
                SelfVerified(root_of_the_first_pass),
                BadAndNotAllZero,
                SelfVerified(root_of_the_first_pass),
                "第一遍自证过、重读读坏：取第一遍那一条",
            ),
            (
                SelfVerified(root_of_the_first_pass),
                AllZeroNeverWritten,
                SelfVerified(root_of_the_first_pass),
                "第一遍自证过、重读全零：取第一遍那一条",
            ),
            (
                BadAndNotAllZero,
                BadAndNotAllZero,
                BadAndNotAllZero,
                "两遍都读坏：读坏",
            ),
            (
                BadAndNotAllZero,
                AllZeroNeverWritten,
                BadAndNotAllZero,
                "第一遍读坏、重读全零：读坏压过全零",
            ),
            (
                AllZeroNeverWritten,
                BadAndNotAllZero,
                BadAndNotAllZero,
                "第一遍全零、重读读坏：读坏压过全零",
            ),
            (
                AllZeroNeverWritten,
                AllZeroNeverWritten,
                AllZeroNeverWritten,
                "两遍都全零：全零",
            ),
        ];
        // 迭代上界是九种组合；没有提前出口。
        for (first_reading, reading_of_the_reread, expected, case) in pairs_and_merged {
            assert_eq!(
                better_reading_of_a_root_ring_slot_over_the_two_passes(
                    first_reading,
                    reading_of_the_reread
                ),
                expected,
                "{case}"
            );
        }
    }

    /// 场景：三个槽的两遍读数整份并（第一遍：自证过、读坏、全零；重读：读坏、自证过、读坏）。
    /// 预期：逐槽并、次序与槽不变——第一个槽留第一遍的根，第二个槽取重读的根，第三个槽读坏。
    /// 判别力：并的时候槽错位、或只取其中一遍时这一项红（改回整份换成重读那一遍改的是 `rebuilt_allocator` 里的调用点，由 验证包用例罩）。
    #[test]
    fn the_two_passes_are_merged_slot_by_slot_in_ring_order() {
        let slots = [
            RootRingSlot { region: 0, slot: 0 },
            RootRingSlot { region: 1, slot: 0 },
            RootRingSlot { region: 2, slot: 0 },
        ];
        let root_of_the_first_pass = root_of(2, 8);
        let root_of_the_reread = root_of(3, 13);
        let first_readings = vec![
            (slots[0], SelfVerified(root_of_the_first_pass)),
            (slots[1], BadAndNotAllZero),
            (slots[2], AllZeroNeverWritten),
        ];
        let readings_of_the_reread = vec![
            (slots[0], BadAndNotAllZero),
            (slots[1], SelfVerified(root_of_the_reread)),
            (slots[2], BadAndNotAllZero),
        ];
        assert_eq!(
            merged_readings_of_the_two_root_ring_passes(&first_readings, readings_of_the_reread),
            vec![
                (slots[0], SelfVerified(root_of_the_first_pass)),
                (slots[1], SelfVerified(root_of_the_reread)),
                (slots[2], BadAndNotAllZero),
            ],
            "逐槽取较好的那一份，次序与槽不变"
        );
    }
}

#[cfg(test)]
mod misplaced_root_read_by_the_allocator_rebuild_tests {
    use super::{
        merged_readings_of_the_two_root_ring_passes,
        read_root_ring_slot_telling_never_written_apart,
        ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes,
        RootRingSlotReadingTellingNeverWrittenApart,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
    use crate::pointer::NodePointer;
    use crate::recovery::PoolReader;
    use crate::root_record::{RootRecord, UnmountMarker};
    use crate::root_ring::{slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
    use crate::system_configuration::SystemImmutableSizes;
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"c743-misplaced-r";
    const REGION_DEVICES: [DeviceIdentity; 3] =
        [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];
    /// 根槽宽（物理块 512）。
    const ROOT_SLOT_BYTES: usize = 512;

    fn sizes() -> SystemImmutableSizes {
        SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        }
    }

    fn root_of(checkpoint_txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: FILESYSTEM_IDENTIFIER,
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: crate::address::CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    /// 只认一个根环槽的读者：那一槽每读一次交出排好的下一份（`Some` 是读到的字节，`None` 是设备报读错），别处不读。
    struct OneRingSlotReadPassByPass {
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        readings_in_pass_order: RefCell<VecDeque<Option<Vec<u8>>>>,
    }

    impl OneRingSlotReadPassByPass {
        fn of(ring_slot: RootRingSlot, readings_in_pass_order: Vec<Option<Vec<u8>>>) -> Self {
            Self {
                device: REGION_DEVICES[usize::try_from(ring_slot.region).expect("区域号")],
                offset: slot_offset(ring_slot, sizes().fixed_structure_slot_spacing),
                readings_in_pass_order: RefCell::new(readings_in_pass_order.into()),
            }
        }
    }

    impl PoolReader for OneRingSlotReadPassByPass {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            vec![DeviceIdentity(0), DeviceIdentity(1)]
        }
        fn device_size_in_bytes(&self, _device: DeviceIdentity) -> Option<u64> {
            unreachable!("读一个根环槽不问盘多大")
        }
        fn physical_block_size_probed_on_the_device(
            &self,
            _device: DeviceIdentity,
        ) -> Option<crate::block_device::PhysicalBlockSizeInBytes> {
            unreachable!("读一个根环槽不探测块宽")
        }
        fn read(
            &self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Option<Vec<u8>> {
            assert_eq!(
                (device, offset, length),
                (self.device, self.offset, ROOT_SLOT_BYTES),
                "只读排好的那一个根环槽"
            );
            self.readings_in_pass_order
                .borrow_mut()
                .pop_front()
                .expect("每一遍读那一槽一次，排好的读数够用")
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            unreachable!("读一个根环槽不扫 journal 环")
        }
    }

    /// 重建分配器读这一槽两遍：照挂载读的那个函数各读一遍，交回两遍的读数。
    fn the_two_rebuild_passes_over(
        reader: &OneRingSlotReadPassByPass,
        ring_slot: RootRingSlot,
    ) -> (
        RootRingSlotReadingTellingNeverWrittenApart,
        RootRingSlotReadingTellingNeverWrittenApart,
    ) {
        let read_once = || {
            read_root_ring_slot_telling_never_written_apart(
                reader,
                &REGION_DEVICES,
                &sizes(),
                &FILESYSTEM_IDENTIFIER,
                ring_slot,
            )
        };
        let first_pass = read_once();
        let reread = read_once();
        (first_pass, reread)
    }

    /// 场景：重建分配器读根环的两遍，区域 1 槽 2（txg 7 的落点槽）第一遍读回别的槽里一条自证得过的根（txg 4 的，它的落点是区域 1 槽 1：错位读），
    /// 重读那一遍设备报读错（C743 那一行「怎么拦」：一遍错位读、一遍读坏）。
    /// 预期：错位读照挂载那一刻读坏记（不是全零的读坏），两遍并起来那一槽读坏；「两遍里只有一遍自证过」那一类一槽都不记、不记那条根。
    /// 阳性对照：同一槽第一遍读回住在这一槽的 txg 7 那条根、重读读坏，那一类记下这一槽（两遍并起来是那条根）——核在的时候新类照常记。
    /// 判别力：`crates/mutations.tsv`「根环一簇 R1：重建分配器读根环不核落点公式」那一行，这一条红（错位那一条当自证过，新类记下它）。
    #[test]
    fn misplaced_root_read_on_one_rebuild_pass_and_a_bad_read_on_the_other_is_not_recorded_in_the_one_pass_class(
    ) {
        let ring_slot =
            target_for_publish(CheckpointTxg(7), RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM);
        assert_eq!(
            ring_slot,
            RootRingSlot { region: 1, slot: 2 },
            "前提：S = 8 时 txg 7 落区域 1 槽 2"
        );
        let misplaced_root = root_of(4);
        assert_ne!(
            target_for_publish(
                misplaced_root.checkpoint_txg,
                RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM
            ),
            ring_slot,
            "前提：txg 4 那条根不住这一槽"
        );
        let misplaced = OneRingSlotReadPassByPass::of(
            ring_slot,
            vec![Some(misplaced_root.to_slot(ROOT_SLOT_BYTES)), None],
        );
        let (first_pass, reread) = the_two_rebuild_passes_over(&misplaced, ring_slot);
        assert_eq!(
            (first_pass, reread),
            (
                RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero,
                RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
            ),
            "错位读照读坏记（不是全零），重读读错也是读坏"
        );
        assert!(
            ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
                &[(ring_slot, first_pass)],
                &[(ring_slot, reread)]
            )
            .is_empty(),
            "新类不记错位读出来的那条根"
        );
        assert_eq!(
            merged_readings_of_the_two_root_ring_passes(
                &[(ring_slot, first_pass)],
                vec![(ring_slot, reread)]
            ),
            vec![(
                ring_slot,
                RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
            )],
            "两遍并起来那一槽读坏，不拿错位的那条根顶上"
        );

        let resident_root = root_of(7);
        let resident = OneRingSlotReadPassByPass::of(
            ring_slot,
            vec![Some(resident_root.to_slot(ROOT_SLOT_BYTES)), None],
        );
        let (first_pass_of_the_resident, reread_of_the_resident) =
            the_two_rebuild_passes_over(&resident, ring_slot);
        assert_eq!(
            first_pass_of_the_resident,
            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(resident_root),
            "阳性对照：住在这一槽的根照认"
        );
        assert_eq!(
            ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
                &[(ring_slot, first_pass_of_the_resident)],
                &[(ring_slot, reread_of_the_resident)]
            )
            .into_iter()
            .collect::<Vec<_>>(),
            vec![ring_slot],
            "阳性对照：一遍自证过、一遍读坏的那一槽照记新类"
        );
    }
}
