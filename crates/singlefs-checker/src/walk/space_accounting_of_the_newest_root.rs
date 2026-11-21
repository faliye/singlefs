//! 最新根的空间记账：延迟槽、回收门槛与钉住的环槽、I-3.1（已分配统计对得上） 的上下界、走读引用起点与 I-3.17（隔离记号与引用相符）。

use super::*;

/// 一棵分配记录树的全部记录上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
pub(super) fn judge_allocation_record_ranges(
    records: &[AllocationRecordView],
    version_name: &str,
    judgements: &mut Judgements,
) {
    let mut ranges_per_device: BTreeMap<u32, Vec<(u64, u64)>> = BTreeMap::new();
    for record in records {
        ranges_per_device
            .entry(record.device)
            .or_default()
            .push((record.slot, record.span_slots));
    }
    for (device, mut ranges) in ranges_per_device {
        ranges.sort_unstable();
        match ranges
            .windows(2)
            .find(|pair| pair[0].0 + pair[0].1 > pair[1].0)
        {
            Some(pair) => {
                let ((first_slot, first_span), (second_slot, second_span)) = (pair[0], pair[1]);
                judgements.judge("I-5.4", false, || {
                    format!(
                        "{version_name}那棵分配记录树、盘 {device}：槽 {first_slot} 跨 {first_span} 的记录与槽 {second_slot} 跨 {second_span} 的记录罩住同一个槽"
                    )
                });
            }
            None => judgements.judge("I-5.4", true, String::new),
        }
    }
}

/// 收口表第 43 行取丁-defer（被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M1）：
/// I-3.16（碎片度与全空段统计对得上） 占着的槽并上这样的槽（I-3.1（已分配统计对得上） 改成 RCW 之前它也并进 I-3.1 的「实际遍历」）——F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的分配记录树里
/// 罩住它的记录是已释放、释放代高于回收门槛 `reclaim_threshold` = max(F_生效, 环里最旧有效根) 的（还在 defer 里，分配器还不许回收，
/// 记账照算已分配；门槛取整个环、不取一圈窗口 RGall，D16（发布语义） 已定项 1）。逐盘交回槽数。
/// 最新根的树表或分配记录树读不出时一槽都不并（照旧判）。
pub(super) fn deferred_slots_referenced_only_below_the_floor_per_device(
    reader: &dyn ImageReader,
    filesystem_identifier_low: u64,
    newest_root: &crate::RootView,
    roots_below_the_floor: &[&crate::RootView],
    walked_slots: &BTreeSet<(u32, u64)>,
    reclaim_threshold: u64,
    cache: &mut IndexNodeCache,
) -> BTreeSet<(u32, u64)> {
    let tree_table_pointer = parse_node_pointer(&newest_root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return BTreeSet::new();
    }
    let Some(tree_table) = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        cache,
    ) else {
        return BTreeSet::new();
    };
    let central_mapping =
        central_mapping_of_a_root_record_without_judging(reader, &newest_root.record_bytes, cache);
    let mut released_above_the_floor: BTreeSet<(u32, u64)> = BTreeSet::new();
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) = allocation_record_tree_without_judging(
            reader,
            &allocation_root,
            &central_mapping,
            cache,
        ) else {
            continue;
        };
        for record in records {
            if allocation_record_is_released_above_the_reclaim_threshold(&record, reclaim_threshold)
            {
                released_above_the_floor.extend(
                    (record.slot..record.slot + record.span_slots)
                        .map(|slot| (record.device, slot)),
                );
            }
        }
    }
    let mut slots_referenced_by_the_roots_below_the_floor: Vec<(u32, u64)> = Vec::new();
    for root in roots_below_the_floor {
        let mut walk_of_the_root_below_the_floor = Walk::starting_with(
            reader,
            Judgements::default(),
            filesystem_identifier_low,
            newest_root.instance,
            newest_root.checkpoint_txg,
        );
        walk_of_the_root_below_the_floor.walk_root(&root.record_bytes, false);
        for (device, start_slot, span_slots) in walk_of_the_root_below_the_floor.references.keys() {
            slots_referenced_by_the_roots_below_the_floor
                .extend((*start_slot..*start_slot + *span_slots).map(|slot| (*device, slot)));
        }
    }
    deferred_slots_among_the_references_of_the_selected_roots(
        slots_referenced_by_the_roots_below_the_floor,
        walked_slots,
        &released_above_the_floor,
    )
}

/// 隔离豁免的单元（[`quarantined_slots_exempted_per_device`] 交回的）：罩住的每一个 (盘, 槽)，与逐盘豁免的槽数（按单元的跨度加，
/// 与改出 `slots` 之前同一个数法）；连同已分配、没有根引用而没被豁免的那几个单元每块盘上那条记录的 (盘, 起点槽)
/// （I-3.1、I-3.11 判红时点名的槽）。
pub(super) struct UnitsExemptedAsQuarantined {
    pub(super) slots: BTreeSet<(u32, u64)>,
    pub(super) slot_count_per_device: BTreeMap<u32, u64>,
    start_slots_not_exempted: BTreeSet<(u32, u64)>,
}

impl UnitsExemptedAsQuarantined {
    /// 已分配、没有根引用而没被豁免的单元在 `device` 上那几条记录的 (盘, 起点槽)。
    pub(super) fn start_slots_not_exempted_on(&self, device: u32) -> Vec<(u32, u64)> {
        self.start_slots_not_exempted
            .iter()
            .filter(|(on_device, _)| *on_device == device)
            .copied()
            .collect()
    }
}

/// 一条根按最新根指着的实例表是有效的还是被抛弃的（D23（journal 的角色与格式） 已定项 14 候选集：(i, T) 有效 ⟺ 无 i 的行，
/// 或有行 (i, Ti, Wi) 且 T ≤ Ti）。丁-defer 选根与它的回收门槛都按它分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootStandingInTheNewestRootsTable {
    Valid,
    Abandoned,
}

/// 丁-defer 的选根式（收口表第 43 行）：引用要并进 I-3.16（碎片度与全空段统计对得上） 占着的槽的根，
/// 是按最新根的实例表有效、txg 在 F_生效 之下的那几条。被抛弃的根不并：它们独占的槽由影子账隔离、在现行账里不是已释放。
/// I-3.1（已分配统计对得上） 改成 RCW 之后不再用它（C697）；「被抛弃的根也并」那一式的小世界等价测试随 RCW 退役（C685）。
pub(super) fn root_references_are_merged_into_the_deferred_slots_below_the_floor(
    standing: RootStandingInTheNewestRootsTable,
    checkpoint_txg: u64,
    effective_rollback_floor: u64,
) -> bool {
    match standing {
        RootStandingInTheNewestRootsTable::Valid => checkpoint_txg < effective_rollback_floor,
        RootStandingInTheNewestRootsTable::Abandoned => false,
    }
}

/// 丁-defer 的门槛与分配器的回收门槛同一个：max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1；`research/prompts/m3-ring-r3-main-verification.md`
/// 第二节第 5 行把它与回收门槛同取一圈窗口，`research/prompts/m3-reclaim-window-r1-main-verification.md`
/// 第六节 Q1）——释放代落在 (F_生效, 环里最旧有效根] 的槽分配器已经回收、不在「已分配」里，再按 F_生效 并进来就多算。
/// 环里一条有效根都没有时取 F_生效。
pub(super) fn deferred_slots_reclaim_threshold(
    standing_and_txg_of_each_readable_root: impl IntoIterator<
        Item = (RootStandingInTheNewestRootsTable, u64),
    >,
    effective_rollback_floor: u64,
) -> u64 {
    standing_and_txg_of_each_readable_root
        .into_iter()
        .filter(|(standing, _)| match standing {
            RootStandingInTheNewestRootsTable::Valid => true,
            RootStandingInTheNewestRootsTable::Abandoned => false,
        })
        .map(|(_, checkpoint_txg)| checkpoint_txg)
        .min()
        .map_or(effective_rollback_floor, |oldest_valid_root_in_the_ring| {
            oldest_valid_root_in_the_ring.max(effective_rollback_floor)
        })
}

/// 最新根那一版的账是不是这条根自己那次发布建的（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）
/// 只判建账的发布）：看树表里记账树那一条的根指针出生 txg——带单元的发布每次都重写记账树（记账行的代段就是那一版的 txg，
/// I-3.13），零单元发布与照抄上一版账的根照抄那一条指针。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AccountOfTheNewestRoot {
    /// 树表读不出、条目窄过字段表，或里面没有记账树那一条（树表 0 条的一版）：最新根下面没有记账树。
    NoAccountingTree,
    /// 记账树根指针的出生 txg 不是这条根的 txg：账（连同 t、p）照抄更早的一次发布。
    CarriedFromAnEarlierPublish,
    /// 记账树根指针的出生 txg 等于这条根的 txg：这次发布建了账，t 与 p 是它记根那一刻采的。
    BuiltByThisPublish,
}

/// 走最新根那一遍记下的记账树根指针出生 txg（[`Walk::accounting_tree_root_birth_txg`]）读成这一版的账是谁建的：
/// 没走到记账树那一条（树表读不出、条目窄过字段表、树表 0 条）是没有记账树。
pub(super) fn account_of_the_newest_root(
    accounting_tree_root_birth_txg: Option<u64>,
    newest_txg: u64,
) -> AccountOfTheNewestRoot {
    match accounting_tree_root_birth_txg {
        None => AccountOfTheNewestRoot::NoAccountingTree,
        Some(birth_txg) if birth_txg == newest_txg => AccountOfTheNewestRoot::BuiltByThisPublish,
        Some(_) => AccountOfTheNewestRoot::CarriedFromAnEarlierPublish,
    }
}

/// 钉槽 p 那 1 字节读成什么（C773（读坏与全零根槽的回收门槛钉只在内存里）：p 存槽的编码 `3·槽 + 区域`，没有钉着写 255）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PinnedRingSlotOfTheRoot {
    NoSlotPinned,
    /// p 小于环长 3S，还原成 (区域 p mod 3, 槽 p div 3)；钉值是那一槽可能住过的最旧 txg（[`pinned_txg_of_the_ring_slot`]）。
    Pinned {
        pinned_txg: u64,
        region: u64,
        slot: u64,
    },
    /// 不是 255，也不小于环长 3S：还原不成一个槽。
    OutsideTheRing(u8),
}

/// mkfs 把第 0 代根种在每个区域的这一槽（D22（单元原子性怎么合成） 已定项 8；实现一侧是 `crates/singlefs-core/src/make_filesystem.rs` 的
/// `genesis_root_ring_slots`，池级 checker 不引实现，这里另记一份）。
pub(super) const RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION: u64 = 0;

/// 钉着的槽的钉值：那一槽可能住过的最旧 txg（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，判决
/// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R1 采乙）。三个区域的槽 0 都可能还住着 mkfs 种下的第 0 代根，钉值是
/// 第 0 代根的 txg 0；别的槽照落点公式 (txg mod R, txg div R)（D22（单元原子性怎么合成） 已定项 16）是 `3·槽 + 区域`，就是 p 本身。
fn pinned_txg_of_the_ring_slot(pinned_ring_slot: u8, slot: u64) -> u64 {
    if slot == RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION {
        GENESIS_CHECKPOINT_TXG
    } else {
        u64::from(pinned_ring_slot)
    }
}

fn pinned_ring_slot_of_the_root(
    pinned_ring_slot: u8,
    geometry: &PoolGeometry,
) -> PinnedRingSlotOfTheRoot {
    if pinned_ring_slot == ROOT_RECORD_NO_RING_SLOT_PINNED {
        return PinnedRingSlotOfTheRoot::NoSlotPinned;
    }
    let encoded_ring_slot = u64::from(pinned_ring_slot);
    let ring_length_in_txgs = geometry.regions * geometry.slots_per_region;
    if encoded_ring_slot >= ring_length_in_txgs {
        return PinnedRingSlotOfTheRoot::OutsideTheRing(pinned_ring_slot);
    }
    // p = 3·槽 + 区域（R = 3）：0 到 3S 之内一个 p 恰一个槽。
    let slot = encoded_ring_slot / geometry.regions;
    PinnedRingSlotOfTheRoot::Pinned {
        pinned_txg: pinned_txg_of_the_ring_slot(pinned_ring_slot, slot),
        region: encoded_ring_slot % geometry.regions,
        slot,
    }
}

/// I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根）：只判最新根，只判建账的发布
/// （D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」）。
///
/// I-7.18 现算 `t* = max(这条根自己带的 F, min(环里按最新根的实例表仍有效的根的最小 txg, p 的钉值))`，与写者
/// `t = max(F, 分配器记这条根时的回收门槛)` 是同一个式子在镜像上的那一份（[`deferred_slots_reclaim_threshold`]：F 换成根自己带的、
/// 钉值当一条有效根并进去）。最新根的实例表链没走到末片时被抛弃的根滤不全，环里最旧有效根算不准：两条都报不适用。
pub(super) fn judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(
    roots: &[(u64, u64, crate::RootView)],
    newest_index: usize,
    geometry: &PoolGeometry,
    standing_in_the_newest_roots_table: &dyn Fn(
        &crate::RootView,
    ) -> RootStandingInTheNewestRootsTable,
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    account: AccountOfTheNewestRoot,
    judgements: &mut Judgements,
) {
    let newest_root = &roots[newest_index].2;
    match account {
        AccountOfTheNewestRoot::NoAccountingTree => {
            for invariant in [
                ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
                PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
            ] {
                judgements.not_applicable(invariant, "最新根下面没有记账树（树表 0 条的一版）");
            }
            return;
        }
        AccountOfTheNewestRoot::CarriedFromAnEarlierPublish => {
            for invariant in [
                ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
                PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
            ] {
                judgements.not_applicable(
                    invariant,
                    "最新根没有建账（记账树根指针的出生 txg 早于这条根）：t、p 照抄上一版，环转过之后求等会误红，不判",
                );
            }
            return;
        }
        AccountOfTheNewestRoot::BuiltByThisPublish => {}
    }
    match abandoned_roots_filtering {
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {}
        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
            for invariant in [
                ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
                PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
            ] {
                judgements.not_applicable(
                    invariant,
                    "最新根的实例表链没走到末片：被崩溃恢复抛弃的根滤不全，环里最旧有效根与钉槽里那条根是不是本实例的都算不准",
                );
            }
            return;
        }
    }
    let pinned = pinned_ring_slot_of_the_root(newest_root.pinned_ring_slot, geometry);
    let pinned_txg_counted_as_a_valid_root = match pinned {
        PinnedRingSlotOfTheRoot::Pinned { pinned_txg, .. } => Some(pinned_txg),
        PinnedRingSlotOfTheRoot::NoSlotPinned | PinnedRingSlotOfTheRoot::OutsideTheRing(_) => None,
    };
    let recomputed_threshold = deferred_slots_reclaim_threshold(
        roots
            .iter()
            .map(|(_, _, root)| {
                (
                    standing_in_the_newest_roots_table(root),
                    root.checkpoint_txg,
                )
            })
            .chain(
                pinned_txg_counted_as_a_valid_root
                    .map(|pinned_txg| (RootStandingInTheNewestRootsTable::Valid, pinned_txg)),
            ),
        newest_root.rollback_floor,
    );
    let (newest_instance, newest_txg) = (newest_root.instance, newest_root.checkpoint_txg);
    judgements.judge(
        ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
        newest_root.reclaim_threshold == recomputed_threshold,
        || {
            format!(
                "最新根（实例 {newest_instance}、txg {newest_txg}）带的回收门槛 t {}，按这条根自己带的 F {}、环里按它的实例表仍有效的根与钉槽 p {} 现算是 {recomputed_threshold}",
                newest_root.reclaim_threshold, newest_root.rollback_floor, newest_root.pinned_ring_slot
            )
        },
    );
    match pinned {
        PinnedRingSlotOfTheRoot::NoSlotPinned => {
            judgements.judge(
                PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
                true,
                String::new,
            );
        }
        PinnedRingSlotOfTheRoot::OutsideTheRing(byte) => {
            judgements.judge(PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE, false, || {
                format!(
                    "最新根（实例 {newest_instance}、txg {newest_txg}）的钉槽 p 是 {byte}：不是 255，也不小于环长 {}，还原不成一个槽",
                    geometry.regions * geometry.slots_per_region
                )
            });
        }
        PinnedRingSlotOfTheRoot::Pinned {
            pinned_txg,
            region,
            slot,
        } => {
            let root_of_this_instance_in_the_pinned_slot =
                roots.iter().find(|(root_region, root_slot, root)| {
                    (*root_region, *root_slot) == (region, slot) && root.instance == newest_instance
                });
            judgements.judge(
                PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
                root_of_this_instance_in_the_pinned_slot.is_none(),
                || {
                    let txg_in_the_pinned_slot = root_of_this_instance_in_the_pinned_slot
                        .map_or(0, |(_, _, root)| root.checkpoint_txg);
                    format!(
                        "最新根（实例 {newest_instance}、txg {newest_txg}）的钉槽 p {}（钉值 {pinned_txg}）指着区域 {region} 槽 {slot}，那一槽住着同一实例写的 txg {txg_in_the_pinned_slot} 的根：写掉那一槽时钉该解开",
                        newest_root.pinned_ring_slot
                    )
                },
            );
        }
    }
}

/// 「钉带出来的那一截」（C773（读坏与全零根槽的回收门槛钉只在内存里） 合成形态）：最新根的账里罩住它的记录已释放、释放代落在
/// (t, 不带钉现算的门槛] 的槽——写者因为钉没回收它们，账照算已分配——不看引用并进 I-3.16（碎片度与全空段统计对得上） 占着的槽；
/// 被抛弃根引用着的不并。I-3.1（已分配统计对得上） 改成 RCW 之后不再并它：上沿里的 U 按最新根带的 t 取，这一截本来就在里面。
/// `already_counted` 是已经并进去的槽（全部走过的版本的引用、隔离豁免、F 之下仍在 defer 的），这里不重复数。
/// 最新根的树表或分配记录树读不出时一槽都不并（照旧判）。
pub(super) fn released_slots_the_pin_kept_in_defer(
    reader: &dyn ImageReader,
    newest_root: &crate::RootView,
    reclaim_threshold_without_the_pin: u64,
    slots_referenced_by_abandoned_roots: &BTreeSet<(u32, u64)>,
    already_counted: &BTreeSet<(u32, u64)>,
    cache: &mut IndexNodeCache,
) -> BTreeSet<(u32, u64)> {
    let reclaim_threshold_on_the_root = newest_root.reclaim_threshold;
    if reclaim_threshold_on_the_root >= reclaim_threshold_without_the_pin {
        return BTreeSet::new();
    }
    let tree_table_pointer =
        parse_node_pointer(&newest_root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER]);
    if tree_table_pointer.all_zero {
        return BTreeSet::new();
    }
    let Some(tree_table) = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        cache,
    ) else {
        return BTreeSet::new();
    };
    let central_mapping =
        central_mapping_of_a_root_record_without_judging(reader, &newest_root.record_bytes, cache);
    let mut kept_in_defer: BTreeSet<(u32, u64)> = BTreeSet::new();
    // 迭代上界是树表的条目数；只有分配记录树那几条（合法镜像上恰一条）往下读。
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) = allocation_record_tree_without_judging(
            reader,
            &allocation_root,
            &central_mapping,
            cache,
        ) else {
            continue;
        };
        kept_in_defer.extend(slots_the_pin_kept_in_defer_among(
            &records,
            reclaim_threshold_on_the_root,
            reclaim_threshold_without_the_pin,
            slots_referenced_by_abandoned_roots,
            already_counted,
        ));
    }
    kept_in_defer
}

/// [`released_slots_the_pin_kept_in_defer`] 的判法那一半（只看一棵分配记录树读出来的记录，不读盘）：已释放、释放代落在
/// (`reclaim_threshold_on_the_root`, `reclaim_threshold_without_the_pin`] 的记录罩住的槽，去掉被抛弃根引用着的与已经数过的。
fn slots_the_pin_kept_in_defer_among(
    records: &[AllocationRecordView],
    reclaim_threshold_on_the_root: u64,
    reclaim_threshold_without_the_pin: u64,
    slots_referenced_by_abandoned_roots: &BTreeSet<(u32, u64)>,
    already_counted: &BTreeSet<(u32, u64)>,
) -> BTreeSet<(u32, u64)> {
    records
        .iter()
        .filter(|record| {
            record.is_released
                && record.generation > reclaim_threshold_on_the_root
                && record.generation <= reclaim_threshold_without_the_pin
        })
        .flat_map(|record| {
            (record.slot..record.slot + record.span_slots).map(|slot| (record.device, slot))
        })
        .filter(|key| {
            !slots_referenced_by_abandoned_roots.contains(key) && !already_counted.contains(key)
        })
        .collect()
}

/// 环里被最新根的实例表判抛弃的根（不是最新根）引用着的槽：各走一遍、判定丢掉（与 [`judge_blocks_referenced_by_abandoned_roots`] 同一个选法），
/// 只收引用。I-3.1 那一截（[`released_slots_the_pin_kept_in_defer`]）按它排除。
pub(super) fn slots_referenced_by_abandoned_roots(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    newest_index: usize,
    standing_in_the_newest_roots_table: &dyn Fn(
        &crate::RootView,
    ) -> RootStandingInTheNewestRootsTable,
    filesystem_identifier_low: u64,
) -> BTreeSet<(u32, u64)> {
    let newest_root = &roots[newest_index].2;
    let mut referenced: BTreeSet<(u32, u64)> = BTreeSet::new();
    for (index, (_, _, root)) in roots.iter().enumerate() {
        let is_abandoned = match standing_in_the_newest_roots_table(root) {
            RootStandingInTheNewestRootsTable::Abandoned => true,
            RootStandingInTheNewestRootsTable::Valid => false,
        };
        if index == newest_index || !is_abandoned {
            continue;
        }
        let mut walk_of_the_abandoned_root = Walk::starting_with(
            reader,
            Judgements::default(),
            filesystem_identifier_low,
            newest_root.instance,
            newest_root.checkpoint_txg,
        );
        walk_of_the_abandoned_root.walk_root(&root.record_bytes, false);
        for (device, start_slot, span_slots) in walk_of_the_abandoned_root.references.keys() {
            referenced.extend((*start_slot..*start_slot + *span_slots).map(|slot| (*device, slot)));
        }
    }
    referenced
}

/// 最新根的账里这条记录还在 defer 里：已释放、释放代高于回收门槛（分配器还不许回收，记账照算已分配）。
fn allocation_record_is_released_above_the_reclaim_threshold(
    record: &AllocationRecordView,
    reclaim_threshold: u64,
) -> bool {
    record.is_released && record.generation > reclaim_threshold
}

/// 丁-defer 并进来的槽：选中的根引用着（`slots_referenced_by_the_selected_roots`，(盘, 槽)，可以重复）、候选集那一遍没走到、
/// 最新根的账里还在 defer 里的。
fn deferred_slots_among_the_references_of_the_selected_roots(
    slots_referenced_by_the_selected_roots: impl IntoIterator<Item = (u32, u64)>,
    walked_slots: &BTreeSet<(u32, u64)>,
    released_above_the_reclaim_threshold: &BTreeSet<(u32, u64)>,
) -> BTreeSet<(u32, u64)> {
    slots_referenced_by_the_selected_roots
        .into_iter()
        .filter(|key| {
            !walked_slots.contains(key) && released_above_the_reclaim_threshold.contains(key)
        })
        .collect()
}

/// 隔离的记录（D19（块指针的结构与宽度预算） 已定项 5：释放之前读盘核出对不上的那个单元，每块盘上的分配记录都留在已分配、
/// 跨度段带隔离记号，映射条目去掉、没有任何根再引用它）在 I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读） 上
/// 怎么认：**只认记号**——已分配而没有根引用的记录按单元（起点槽、跨度）
/// 归组，组里每一条（每块盘上那一条）都带记号的整组豁免，不读副本；有一条不带记号的照旧判违例，不退回按副本判。
/// 按单元而不按份判：写者隔离时两块盘的记录一起带上记号（D19 已定项 5「另一块盘上那一份对得上也一起留，各盘的账保持对称」）；
/// 只一块盘带记号的那一形由 I-3.17（隔离记号与引用相符） 的第三句判红，这里那一组不豁免。
///
/// 读的是 `root` 那棵分配记录树（树表里种类 3 那一条指着的根，按位置整棵走下去，`allocation_record_tree_without_judging`）：未释放、`referenced_start_slots`
/// 里没有哪个引用起在它那一槽的记录。逐盘交回豁免的槽数；树表或分配记录树读不出、位置对不上时那一棵一格都不豁免（照旧判）。
pub(super) fn quarantined_slots_exempted_per_device(
    reader: &dyn ImageReader,
    root: &crate::RootView,
    referenced_start_slots: &BTreeSet<(u32, u64)>,
    cache: &mut IndexNodeCache,
) -> UnitsExemptedAsQuarantined {
    let mut exempted = UnitsExemptedAsQuarantined {
        slots: BTreeSet::new(),
        slot_count_per_device: BTreeMap::new(),
        start_slots_not_exempted: BTreeSet::new(),
    };
    // 一个单元（起点槽、跨度）→ 记着它、没有根引用的那几条未释放记录。
    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<AllocationRecordView>> = BTreeMap::new();
    for record in allocation_records_of_the_newest_root_without_judging(reader, root, cache)
        .into_iter()
        .flatten()
    {
        if record.is_released || referenced_start_slots.contains(&(record.device, record.slot)) {
            continue;
        }
        unreferenced_units
            .entry((record.slot, record.span_slots))
            .or_default()
            .push(record);
    }
    for ((slot, span_slots), copies_of_the_unit) in unreferenced_units {
        if every_copy_of_the_unit_carries_the_quarantine_mark(&copies_of_the_unit) {
            for copy in copies_of_the_unit {
                *exempted
                    .slot_count_per_device
                    .entry(copy.device)
                    .or_insert(0) += span_slots;
                exempted.slots.extend(
                    (slot..slot + span_slots).map(|covered_slot| (copy.device, covered_slot)),
                );
            }
        } else {
            exempted
                .start_slots_not_exempted
                .extend(copies_of_the_unit.iter().map(|copy| (copy.device, slot)));
        }
    }
    exempted
}

/// 一个单元（起点槽、跨度）的那几条未释放记录都带隔离记号（D19（块指针的结构与宽度预算） 已定项 5：记号齐的未释放记录，I-3.1、I-3.11、
/// I-3.15 豁免，不看副本）。一条都没有时不算齐。
pub(super) fn every_copy_of_the_unit_carries_the_quarantine_mark(
    copies_of_the_unit: &[AllocationRecordView],
) -> bool {
    !copies_of_the_unit.is_empty()
        && copies_of_the_unit
            .iter()
            .all(|copy| match copy.quarantine_mark {
                QuarantineMarkInTheSpanField::Present => true,
                QuarantineMarkInTheSpanField::Absent => false,
            })
}

/// `root` 那棵分配记录树里的每一条记录（树表里种类 3 那几条指着的根，按位置整棵走下去，`allocation_record_tree_without_judging`；
/// 节点按这条根自己的中央映射认位置）。树表指针全零交回空表；树表读不出交回 `None`；分配记录树读不出、位置对不上的那一棵跳过。只读不判。
fn allocation_records_of_the_newest_root_without_judging(
    reader: &dyn ImageReader,
    root: &crate::RootView,
    cache: &mut IndexNodeCache,
) -> Option<Vec<AllocationRecordView>> {
    allocation_records_of_a_root_record_without_judging(
        reader,
        root,
        AllocationRecordTreeThatDoesNotReadBack::SkipTheTree,
        cache,
    )
    .map(|account| account.records)
}

/// I-3.1（已分配统计对得上） 读最新根全部分配记录的结局（[`allocation_records_of_the_newest_root_read_in_full_without_judging`]）。
#[derive(Debug, PartialEq, Eq)]
pub(super) enum AllocationRecordsOfTheNewestRootReadInFull {
    /// 树表里种类 3 那几条指着的分配记录树都读出来了（树表指针全零时是空表）。
    Read(Vec<AllocationRecordView>),
    /// 树表非空而一条种类 3 的条目都没有：这一版的账一棵分配记录树都没有，记录口径无从比。这块畸形由 I-9.16（树表条目按树 ID 严格升序且合发号次序） ④ 判红
    /// （配不上的那块畸形别的不变量判红，那一处报部分没判）。
    TreeTableWithEntriesButNoAllocationRecordTreeEntry { tree_table_entries: usize },
    /// 树表读不出、条目窄过字段表，或有一棵分配记录树读不出。
    NotReadInFull,
}

/// I-3.1（已分配统计对得上） 的三判要最新根的全部分配记录：少一棵树，记录罩住的槽就数不全，下沿与上沿都没法比。
/// 与 [`allocation_records_of_the_newest_root_without_judging`] 同一份读法，只是树表条目窄过字段表、任何一棵分配记录树读不出时交回读不全；
/// 树表非空而没有种类 3 的条目时另交回一种（[`AllocationRecordsOfTheNewestRootReadInFull`]）。
/// GPU 判器扫描内核 `c_quarantine_exempted` 第二遍记下的 `c_newest_records_read_in_full` 与它读全那一种同一个口径。
pub(super) fn allocation_records_of_the_newest_root_read_in_full_without_judging(
    reader: &dyn ImageReader,
    root: &crate::RootView,
    cache: &mut IndexNodeCache,
) -> AllocationRecordsOfTheNewestRootReadInFull {
    match allocation_records_of_a_root_record_without_judging(
        reader,
        root,
        AllocationRecordTreeThatDoesNotReadBack::GiveUpOnTheRoot,
        cache,
    ) {
        None => AllocationRecordsOfTheNewestRootReadInFull::NotReadInFull,
        Some(AllocationRecordsOfARootRecord {
            tree_table_entries,
            allocation_record_tree_entries: 0,
            ..
        }) if tree_table_entries > 0 => {
            AllocationRecordsOfTheNewestRootReadInFull::TreeTableWithEntriesButNoAllocationRecordTreeEntry {
                tree_table_entries,
            }
        }
        Some(AllocationRecordsOfARootRecord { records, .. }) => {
            AllocationRecordsOfTheNewestRootReadInFull::Read(records)
        }
    }
}

/// 一条根的分配记录（[`allocation_records_of_a_root_record_without_judging`] 交回的），连同它的树表有几条条目、其中种类 3 的几条，
/// 与它的树表指针是不是全零。
struct AllocationRecordsOfARootRecord {
    pub(super) records: Vec<AllocationRecordView>,
    tree_table_entries: usize,
    /// 种类 3 的条目数（根指针全零的那几条也算：条目在，I-9.16 ④ 不判它缺）。
    allocation_record_tree_entries: usize,
    pub(super) tree_table_pointer: TreeTablePointerOfTheRootRecord,
}

/// 根记录里树表指针（偏移 [36, 122)）的样子。全零不是「没有树表」：core 的根记录读者照它那两条位置条目读盘 0 槽 0、读不出
/// 隔离豁免、I-3.1 与 I-3.17 那几种读法照旧按 0 条、空表读，I-3.21 把它报成部分没判
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TreeTablePointerOfTheRootRecord {
    AllZero,
    ReadFromItsLocations,
}

/// 一棵分配记录树读不出（或树表条目窄过字段表）时，读一条根全部分配记录的那一遍怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocationRecordTreeThatDoesNotReadBack {
    /// 跳过那一棵，接着读别的（隔离豁免与 I-3.17 的读法）。
    SkipTheTree,
    /// 整条根交回 `None`（I-3.1 的读法：记录要数全）。
    GiveUpOnTheRoot,
}

fn allocation_records_of_a_root_record_without_judging(
    reader: &dyn ImageReader,
    root: &crate::RootView,
    tree_that_does_not_read_back: AllocationRecordTreeThatDoesNotReadBack,
    cache: &mut IndexNodeCache,
) -> Option<AllocationRecordsOfARootRecord> {
    let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return Some(AllocationRecordsOfARootRecord {
            records: Vec::new(),
            tree_table_entries: 0,
            allocation_record_tree_entries: 0,
            tree_table_pointer: TreeTablePointerOfTheRootRecord::AllZero,
        });
    }
    let tree_table = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        cache,
    )?;
    let central_mapping =
        central_mapping_of_a_root_record_without_judging(reader, &root.record_bytes, cache);
    let mut records = Vec::new();
    let mut allocation_record_tree_entries = 0usize;
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() {
            match tree_that_does_not_read_back {
                AllocationRecordTreeThatDoesNotReadBack::SkipTheTree => continue,
                AllocationRecordTreeThatDoesNotReadBack::GiveUpOnTheRoot => return None,
            }
        }
        if read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        allocation_record_tree_entries += 1;
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records_of_the_tree)) = allocation_record_tree_without_judging(
            reader,
            &allocation_root,
            &central_mapping,
            cache,
        ) else {
            match tree_that_does_not_read_back {
                AllocationRecordTreeThatDoesNotReadBack::SkipTheTree => continue,
                AllocationRecordTreeThatDoesNotReadBack::GiveUpOnTheRoot => return None,
            }
        };
        records.extend(records_of_the_tree);
    }
    Some(AllocationRecordsOfARootRecord {
        records,
        tree_table_entries: tree_table.entries.len(),
        allocation_record_tree_entries,
        tree_table_pointer: TreeTablePointerOfTheRootRecord::ReadFromItsLocations,
    })
}

/// I-3.1（已分配统计对得上） 在一块盘上比的几个数（RCW）。
/// 记 E(θ) = 最新根分配记录里罩在这块盘上、
/// 未释放的记录罩住的槽，并上已释放而释放代高于 θ 的记录罩住的槽（按槽认，几条记录罩同一槽只算一次）。
pub(super) struct AllocatedStatisticBoundsOfADevice {
    /// |E(θ_high)|：θ_high 是 checker 现算的回收门槛 max(F_生效, 环里最旧有效根)（[`deferred_slots_reclaim_threshold`]）。下沿：记账不少于它。
    pub(super) slots_counted_at_the_checker_reclaim_threshold: u64,
    /// |E(t)|：t 是最新根自己带的回收门槛（I-7.18（根记录的回收门槛等于现算的门槛））。上沿的一半。
    pub(super) slots_counted_at_the_root_reclaim_threshold: u64,
    /// |Ω ∪ U|：Ω 是遍历全部候选版本走到的槽（按引用的跨度加，与改成 RCW 之前同一个数法）加隔离豁免的槽；U 是已释放、释放代高于 t、
    /// 遍历没走到的记录罩住的槽（还在 defer 里、写者照算已分配）。上沿的另一半：记账多于它就是记着没人引用、也不在 defer 里的槽（泄漏）。
    pub(super) walked_and_still_deferred_slots: u64,
    /// 遍历走到、而不在 E(θ_high) 里的槽（逐槽那一判：引用着的槽没有记录罩、或罩它的记录早已释放）。
    pub(super) walked_slots_outside_the_records: Vec<u64>,
    /// E(t) 里不在 Ω ∪ U 里的槽（遍历没走到、不是隔离豁免、也不在 defer 里，按槽号升序）：I-3.1 取宽那一支逐槽的上沿判它
    /// （「改法」那一行，[`allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading`]）。
    pub(super) slots_counted_at_the_root_reclaim_threshold_outside_the_walk_and_the_deferred:
        Vec<u64>,
}

/// 照 RCW 的口径算一块盘上的 [`AllocatedStatisticBoundsOfADevice`]。只读传进来的记录、遍历结果与隔离豁免的槽，不读盘。
/// GPU 判器扫描内核是同一个算法，拆在记账分片那一趟（`accounting_slice`）逐项做：每条记录 `c_count_the_slots_of_a_record_into_the_bounds`、
/// 每条引用 `c_slots_of_a_walked_reference_are_recorded`，几片的计数相加、判的那几样相与，存在交接段里那一份界上。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样输入：哪块盘、最新根的记录、遍历走到的槽与它们按引用的跨度加的数、隔离豁免的槽与它们的数、两个回收门槛"
)]
pub(super) fn allocated_statistic_bounds_of_a_device(
    device: u32,
    records_of_the_newest_root: &[AllocationRecordView],
    walked_slots: &BTreeSet<(u32, u64)>,
    walked_slot_count_on_the_device: u64,
    quarantined_exempted_slots: &BTreeSet<(u32, u64)>,
    quarantined_exempted_slot_count_on_the_device: u64,
    reclaim_threshold_computed_by_the_checker: u64,
    reclaim_threshold_on_the_newest_root: u64,
) -> AllocatedStatisticBoundsOfADevice {
    let counted_at = |reclaim_threshold: u64| -> BTreeSet<u64> {
        records_of_the_newest_root
            .iter()
            .filter(|record| {
                record.device == device
                    && (!record.is_released || record.generation > reclaim_threshold)
            })
            .flat_map(|record| record.slot..record.slot.saturating_add(record.span_slots))
            .collect()
    };
    let counted_at_the_checker_reclaim_threshold =
        counted_at(reclaim_threshold_computed_by_the_checker);
    let counted_at_the_root_reclaim_threshold = counted_at(reclaim_threshold_on_the_newest_root);
    let still_deferred_outside_the_walk: BTreeSet<u64> = records_of_the_newest_root
        .iter()
        .filter(|record| {
            record.device == device
                && record.is_released
                && record.generation > reclaim_threshold_on_the_newest_root
        })
        .flat_map(|record| record.slot..record.slot.saturating_add(record.span_slots))
        .filter(|slot| !walked_slots.contains(&(device, *slot)))
        .collect();
    AllocatedStatisticBoundsOfADevice {
        slots_counted_at_the_checker_reclaim_threshold: u64::try_from(
            counted_at_the_checker_reclaim_threshold.len(),
        )
        .expect("槽数装得进 u64"),
        slots_counted_at_the_root_reclaim_threshold: u64::try_from(
            counted_at_the_root_reclaim_threshold.len(),
        )
        .expect("槽数装得进 u64"),
        walked_and_still_deferred_slots: walked_slot_count_on_the_device
            .saturating_add(quarantined_exempted_slot_count_on_the_device)
            .saturating_add(
                u64::try_from(still_deferred_outside_the_walk.len()).expect("槽数装得进 u64"),
            ),
        walked_slots_outside_the_records: walked_slots
            .iter()
            .filter(|(on_device, slot)| {
                *on_device == device && !counted_at_the_checker_reclaim_threshold.contains(slot)
            })
            .map(|(_, slot)| *slot)
            .collect(),
        slots_counted_at_the_root_reclaim_threshold_outside_the_walk_and_the_deferred:
            counted_at_the_root_reclaim_threshold
                .iter()
                .filter(|slot| {
                    !walked_slots.contains(&(device, **slot))
                        && !quarantined_exempted_slots.contains(&(device, **slot))
                        && !still_deferred_outside_the_walk.contains(slot)
                })
                .copied()
                .collect(),
    }
}

/// RCW 三判：(L) 记账不少于 |E(θ_high)|；(P) 遍历走到的每个槽都在 E(θ_high) 里；(U) 记账不多于 min(|E(t)|, |Ω ∪ U|)。
/// 记账的已分配是字节数，三个界是槽数；比的时候槽数乘槽宽放进 u128，不溢出。记账那一行读不出（`None`）判红。
pub(super) fn allocated_statistic_lies_within_its_bounds(
    allocated_bytes: Option<u64>,
    bounds: &AllocatedStatisticBoundsOfADevice,
) -> bool {
    let Some(allocated_bytes) = allocated_bytes else {
        return false;
    };
    let bytes_of = |slots: u64| u128::from(slots) * u128::from(SLOT_BYTES);
    let allocated = u128::from(allocated_bytes);
    let lower_bound_holds =
        allocated >= bytes_of(bounds.slots_counted_at_the_checker_reclaim_threshold);
    let every_walked_slot_is_recorded = bounds.walked_slots_outside_the_records.is_empty();
    let upper_bound_holds = allocated
        <= bytes_of(
            bounds
                .slots_counted_at_the_root_reclaim_threshold
                .min(bounds.walked_and_still_deferred_slots),
        );
    lower_bound_holds && every_walked_slot_is_recorded && upper_bound_holds
}

/// 最新根里跨度段与起点单元那一类的槽数不符的未释放记录——I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数） 判红的那几条，
/// 认法与它同一个（[`class_and_slots_of_the_unit_whose_usable_header_starts_at`]）——改记成那一类的槽数之后的那一份记录；
/// 一条这样的记录都没有交回 `None`。`slots_of_the_unit_starting_at` 交回记录起点槽上那个单元那一类的槽数，头用不了交回 `None`（没有对象，照记录原样）。
pub(super) fn records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(
    records: &[AllocationRecordView],
    mut slots_of_the_unit_starting_at: impl FnMut(&AllocationRecordView) -> Option<u64>,
) -> Option<Vec<AllocationRecordView>> {
    let mut some_span_was_set = false;
    let set_to_the_unit_classes = records
        .iter()
        .map(|record| {
            if record.is_released {
                return *record;
            }
            match slots_of_the_unit_starting_at(record) {
                Some(slots_of_the_class) if slots_of_the_class != record.span_slots => {
                    some_span_was_set = true;
                    AllocationRecordView {
                        span_slots: slots_of_the_class,
                        ..*record
                    }
                }
                Some(_) | None => *record,
            }
        })
        .collect();
    some_span_was_set.then_some(set_to_the_unit_classes)
}

/// I-3.1 取宽那一支（C937（I-3.1 照 I-3.18 改记槽数取宽的条件被攻过零轮）
/// 第二节 Q3「改法」那一行）：只认把 I-3.18 判红的那几条记录改记成单元那一类的槽数之后的那一份读法
/// （[`records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes`]），整份过三判，不跟记录自述那一份逐判混取——
/// (L) 记账不少于改记那一份的 |E(θ_high)|；(U) 记账不多于 min(改记那一份的 |E(t)|, |Ω ∪ U|)；另加逐槽两判：遍历走到的每一槽都在改记那一份
/// E(θ_high) 里（逐槽的下沿，就是改记那一份的 (P)），改记那一份 E(t) 的每一槽都在 Ω ∪ U 里（逐槽的上沿：defer 之外还罩着没人走到的槽就不成立）。
/// 成立：配不上的那块是 I-3.18 判红的那几个单元真占几槽说不准，I-3.1 报部分没判；不成立判红。记账那一行读不出（`None`）判红。
pub(super) fn allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading(
    allocated_bytes: Option<u64>,
    as_the_unit_classes: &AllocatedStatisticBoundsOfADevice,
) -> bool {
    let Some(allocated_bytes) = allocated_bytes else {
        return false;
    };
    let bytes_of = |slots: u64| u128::from(slots) * u128::from(SLOT_BYTES);
    let allocated = u128::from(allocated_bytes);
    let lower_bound_holds =
        allocated >= bytes_of(as_the_unit_classes.slots_counted_at_the_checker_reclaim_threshold);
    let upper_bound_holds = allocated
        <= bytes_of(
            as_the_unit_classes
                .slots_counted_at_the_root_reclaim_threshold
                .min(as_the_unit_classes.walked_and_still_deferred_slots),
        );
    let every_walked_slot_lies_in_the_unit_class_reading = as_the_unit_classes
        .walked_slots_outside_the_records
        .is_empty();
    let every_slot_of_the_unit_class_reading_is_walked_or_deferred = as_the_unit_classes
        .slots_counted_at_the_root_reclaim_threshold_outside_the_walk_and_the_deferred
        .is_empty();
    lower_bound_holds
        && upper_bound_holds
        && every_walked_slot_lies_in_the_unit_class_reading
        && every_slot_of_the_unit_class_reading_is_walked_or_deferred
}

/// 改记那一份里（[`records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes`]）一条未释放记录与同一块盘上另一条记录
/// 罩住的槽区间相交的那一对：相交的槽是 [`first_overlapping_slot`, `end_of_the_overlapping_slots`)。
///
/// [`first_overlapping_slot`]: RecordsOverlappingInTheUnitClassReading::first_overlapping_slot
/// [`end_of_the_overlapping_slots`]: RecordsOverlappingInTheUnitClassReading::end_of_the_overlapping_slots
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordsOverlappingInTheUnitClassReading {
    pub(super) unreleased_record: AllocationRecordView,
    unreleased_record_span: SpanInTheUnitClassReading,
    pub(super) other_record: AllocationRecordView,
    other_record_span: SpanInTheUnitClassReading,
    first_overlapping_slot: u64,
    end_of_the_overlapping_slots: u64,
}

/// 改记那一份里一条记录的跨度从哪来：照记录自述，或是 I-3.18 判红、改记成了起点单元那一类的槽数。
/// 第五判的违例说明按它写：两条都没被改记的相交不说「改记之后」
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpanInTheUnitClassReading {
    AsRecorded,
    SetToTheUnitClass,
}

/// I-3.1 取宽那一支在三判与逐槽两判之外的第五判（「改法」那一行，
/// 两腿各自提出的同一形）：改记那一份里每条未释放记录罩住的槽区间，与同一块盘上别的每一条记录（未释放或已释放）罩住的槽区间都不相交。
/// 改记把一条窄记录放宽之后，它多罩的那几槽要是压在另一条记录上——隔离单元的尾槽压在一条还在 defer 的已释放记录上、两槽单元的尾槽上另有一条记录——
/// 三判与逐槽两判按槽取并集看不见（几条记录罩同一槽只算一次），这一判看得见。两条都已释放的不比。
/// 交回相交的每一对，按（未释放那条的起点槽、另一条的起点槽）升序；两份记录只给这块盘上的、同次序（`records_as_recorded` 是记录自述那一份，
/// `records_set_to_the_unit_classes` 是改记那一份，[`records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes`]），
/// 比的是改记那一份，每条记录带上它被改记没有（[`SpanInTheUnitClassReading`]）。
/// 按起点槽升序排一次，每条往后看起点落在它区间里的那几条（起点不小于它的末槽就停）：相交的两条里起点小的那条罩着起点大的那条的起点。
///
/// # Panics
/// 两份记录条数不同（调用方要给同一块盘上同次序的两份）。
pub(super) fn unreleased_records_overlapping_another_record_in_the_unit_class_reading(
    records_as_recorded: &[AllocationRecordView],
    records_set_to_the_unit_classes: &[AllocationRecordView],
) -> Vec<RecordsOverlappingInTheUnitClassReading> {
    assert_eq!(
        records_as_recorded.len(),
        records_set_to_the_unit_classes.len(),
        "记录自述与改记两份是同一块盘上同次序的记录"
    );
    let mut ordered_by_start: Vec<(AllocationRecordView, SpanInTheUnitClassReading)> =
        records_set_to_the_unit_classes
            .iter()
            .zip(records_as_recorded)
            .map(|(set_record, record_as_recorded)| {
                let span = if set_record.span_slots == record_as_recorded.span_slots {
                    SpanInTheUnitClassReading::AsRecorded
                } else {
                    SpanInTheUnitClassReading::SetToTheUnitClass
                };
                (*set_record, span)
            })
            .collect();
    ordered_by_start.sort_unstable_by_key(|(record, _)| (record.slot, record.span_slots));
    let mut overlapping = Vec::new();
    for (position, (earlier, earlier_span)) in ordered_by_start.iter().enumerate() {
        let end_of_the_earlier = earlier.slot.saturating_add(earlier.span_slots);
        for (later, later_span) in ordered_by_start[position + 1..]
            .iter()
            .take_while(|(later, _)| later.slot < end_of_the_earlier)
        {
            let ((unreleased_record, unreleased_record_span), (other_record, other_record_span)) =
                match (earlier.is_released, later.is_released) {
                    (false, false | true) => ((*earlier, *earlier_span), (*later, *later_span)),
                    (true, false) => ((*later, *later_span), (*earlier, *earlier_span)),
                    (true, true) => continue,
                };
            overlapping.push(RecordsOverlappingInTheUnitClassReading {
                unreleased_record,
                unreleased_record_span,
                other_record,
                other_record_span,
                first_overlapping_slot: later.slot,
                end_of_the_overlapping_slots: end_of_the_earlier
                    .min(later.slot.saturating_add(later.span_slots)),
            });
        }
    }
    overlapping.sort_unstable_by_key(|pair| (pair.unreleased_record.slot, pair.other_record.slot));
    overlapping
}

/// E(θ) 在一块盘上的槽（[`AllocatedStatisticBoundsOfADevice`] 那几个界数的就是它）：未释放的记录、已释放而释放代高于 θ 的记录罩住的槽，按槽认。
/// I-3.1 部分没判的说明拿它比两种读法差在哪几槽。
fn slots_of_the_device_counted_at_the_reclaim_threshold(
    device: u32,
    records: &[AllocationRecordView],
    reclaim_threshold: u64,
) -> BTreeSet<u64> {
    records
        .iter()
        .filter(|record| {
            record.device == device
                && (!record.is_released || record.generation > reclaim_threshold)
        })
        .flat_map(|record| record.slot..record.slot.saturating_add(record.span_slots))
        .collect()
}

/// I-3.1 取宽那一支成立、一块盘报部分没判时说明里带的数：
/// 记账的已分配；Ω ∪ U 那个数连同它的三样各自的槽数（遍历走到的、隔离豁免的、U——释放代高于最新根带的门槛 t 而遍历没走到的已释放槽）；
/// 两种读法各自的下沿与上沿（字节数）；被改记的是哪几条记录（起点槽、自述的跨度 → 那一类的槽数）；两种读法的 E(θ_high) 差在哪几槽
/// （只在改记那一份里的、只在记录自述那一份里的）。`records_as_recorded` 与 `records_set_to_the_unit_classes` 是同一块盘上同次序的两份记录。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是说明里的一样数：哪块盘、记账、两个门槛、遍历与隔离豁免的槽数、两种读法的界、两种读法的记录"
)]
pub(super) fn allocated_statistic_left_to_the_span_invariant(
    device: u32,
    allocated_bytes: Option<u64>,
    reclaim_threshold_computed_by_the_checker: u64,
    reclaim_threshold_on_the_newest_root: u64,
    walked_slot_count_on_the_device: u64,
    quarantined_exempted_slot_count_on_the_device: u64,
    (as_recorded, as_the_unit_classes): (
        &AllocatedStatisticBoundsOfADevice,
        &AllocatedStatisticBoundsOfADevice,
    ),
    (records_as_recorded, records_set_to_the_unit_classes): (
        &[AllocationRecordView],
        &[AllocationRecordView],
    ),
) -> String {
    let bytes_of = |slots: u64| slots.saturating_mul(SLOT_BYTES);
    let still_deferred_outside_the_walk = as_recorded
        .walked_and_still_deferred_slots
        .saturating_sub(walked_slot_count_on_the_device)
        .saturating_sub(quarantined_exempted_slot_count_on_the_device);
    let records_set: Vec<String> = records_as_recorded
        .iter()
        .zip(records_set_to_the_unit_classes)
        .filter(|(as_recorded_record, set_record)| {
            as_recorded_record.span_slots != set_record.span_slots
        })
        .map(|(as_recorded_record, set_record)| {
            format!(
                "槽 {} 跨 {} → {}",
                as_recorded_record.slot, as_recorded_record.span_slots, set_record.span_slots
            )
        })
        .collect();
    let counted_as_recorded = slots_of_the_device_counted_at_the_reclaim_threshold(
        device,
        records_as_recorded,
        reclaim_threshold_computed_by_the_checker,
    );
    let counted_as_the_unit_classes = slots_of_the_device_counted_at_the_reclaim_threshold(
        device,
        records_set_to_the_unit_classes,
        reclaim_threshold_computed_by_the_checker,
    );
    let only_in_the_unit_class_reading: Vec<u64> = counted_as_the_unit_classes
        .difference(&counted_as_recorded)
        .copied()
        .collect();
    let only_in_the_reading_as_recorded: Vec<u64> = counted_as_recorded
        .difference(&counted_as_the_unit_classes)
        .copied()
        .collect();
    format!(
        "盘 {device}：遍历走到而不在记录里的槽 {:?}、记账对记录的上下沿配不上的那一块，全落在跨度段与起点单元那一类的槽数不符的未释放记录\
         （I-3.18 判它）罩着的单元里：那几个单元真占几槽在两种数法之间说不准，记账对记录那三判没判；\
         记账的已分配 {allocated_bytes:?}，遍历全部有效根得到 {}（遍历走到的 {walked_slot_count_on_the_device} 槽、隔离豁免 {quarantined_exempted_slot_count_on_the_device} 槽、\
         U {still_deferred_outside_the_walk} 槽：释放代高于最新根带的门槛 t {reclaim_threshold_on_the_newest_root} 而遍历没走到的已释放槽）；\
         按记录自述：下沿（checker 现算的门槛 {reclaim_threshold_computed_by_the_checker} 之上）{}、上沿（门槛 t 之上）{}；\
         按 I-3.18 判红的记录改记成单元那一类：下沿 {}、上沿（门槛 t 之上）{}，整份三判、逐槽两判与改记之后未释放记录不与别的记录相交都成立；\
         被改记的记录 {records_set:?}；两种读法在 checker 现算的门槛之上差的槽：只在改记那一份里的 {only_in_the_unit_class_reading:?}、\
         只在记录自述那一份里的 {only_in_the_reading_as_recorded:?}",
        as_recorded.walked_slots_outside_the_records,
        bytes_of(as_recorded.walked_and_still_deferred_slots),
        bytes_of(as_recorded.slots_counted_at_the_checker_reclaim_threshold),
        bytes_of(as_recorded.slots_counted_at_the_root_reclaim_threshold),
        bytes_of(as_the_unit_classes.slots_counted_at_the_checker_reclaim_threshold),
        bytes_of(as_the_unit_classes.slots_counted_at_the_root_reclaim_threshold),
    )
}

/// I-3.1 取宽那一支三判与逐槽两判都成立、而改记那一份里有未释放记录与别的记录相交时的违例说明：点名每一对的两条记录与相交的槽。
/// 每条记录按实际情形写：被改记的写「改记之后跨几槽」，没被改记的只写跨几槽——两条都没被改记的相交（记录自述那一份里就相交、I-5.4 也报它）
/// 不说「改记之后」。
pub(super) fn allocated_statistic_violated_by_records_overlapping_in_the_unit_class_reading(
    device: u32,
    overlapping: &[RecordsOverlappingInTheUnitClassReading],
) -> String {
    let pairs: Vec<String> = overlapping
        .iter()
        .map(|pair| {
            let record_described =
                |record: &AllocationRecordView, span: SpanInTheUnitClassReading| {
                    let released_or_not = if record.is_released {
                        format!("已释放、释放代 {}", record.generation)
                    } else {
                        String::from("未释放")
                    };
                    let span_described = match span {
                        SpanInTheUnitClassReading::AsRecorded => {
                            format!("跨 {} 槽", record.span_slots)
                        }
                        SpanInTheUnitClassReading::SetToTheUnitClass => {
                            format!("改记之后跨 {} 槽", record.span_slots)
                        }
                    };
                    format!(
                        "槽 {} 的记录（{released_or_not}，{span_described}）",
                        record.slot
                    )
                };
            format!(
                "{}与{}在槽 [{}, {}) 相交",
                record_described(&pair.unreleased_record, pair.unreleased_record_span),
                record_described(&pair.other_record, pair.other_record_span),
                pair.first_overlapping_slot,
                pair.end_of_the_overlapping_slots,
            )
        })
        .collect();
    format!(
        "盘 {device}：按 I-3.18 判红的记录改记成单元那一类的槽数之后三判与逐槽两判都成立，而那一份读法里有未释放的记录罩住的槽与别的记录相交\
         （判决 research/prompts/m3-close-b19-reason-r2-main-verification.md 第二节 Q1「改法」）：{}",
        pairs.join("；")
    )
}

/// I-3.21（[`WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS`]，C944（走到的单元起点没有自己的记录），
/// 「归属」那一行）：遍历全部候选版本走到的每个引用起点
/// （`start_slots_referenced_by_every_walked_version`，与 I-3.1 同一份遍历），都是最新根分配记录里一条 E(θ_high) 记录——未释放，或已释放而释放代
/// 高于 checker 现算的回收门槛 θ_high（与 I-3.1 同一个门槛）——的起点。I-3.1 的逐槽下沿 (P) 只问走到的槽有没有记录罩着，这一条另问起点对不对齐：
/// 一条宽记录恰好罩住一个走到、却没有自己记录的邻居时 (P) 成立、按槽数加总也对得上，这一条点名那个邻居的起点。
/// 比的记录（C963（树表 0 条写过行的版上写行发布漏记新单元）候选一、
/// 主 agent 定、零轮）：最新根的树表里种类 3 的那几棵；树表 0 条而根记录里分配记录树根指针（[342, 428)）非零的那一版（树表 0 条、写过行的那一版，
/// 写行那次发布的账记在根记录直接持有的那一棵里）比那一棵。
/// 没有对象、报不适用：树表 0 条而根记录那一项全零（mkfs 的第 0 代、照抄它的暖机根）。射程里判不了、报部分没判：树表指针全零（core 照它读盘 0 槽 0、
/// 读不出，同树表读不出）、树表读不出、条目窄过字段表、任何一棵分配记录树读不出（根记录持有的那一棵也算），
/// 或树表非空而一条种类 3 的条目都没有（I-9.16 ④ 判那块畸形）。
/// 射程与 I-3.1 相同、只判池里的盘：走到的起点落在池外的盘上（位置条目点名一块不在池里的盘）不判，那一份副本读不出由 I-2.1 判红
/// 每个判的起点各判一格，判红点名那个 (盘, 起点槽)。
pub(super) fn judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root(
    reader: &dyn ImageReader,
    newest_root: &crate::RootView,
    start_slots_referenced_by_every_walked_version: &BTreeSet<(u32, u64)>,
    reclaim_threshold_computed_by_the_checker: u64,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let newest_txg = newest_root.checkpoint_txg;
    match allocation_records_of_a_root_record_without_judging(
        reader,
        newest_root,
        AllocationRecordTreeThatDoesNotReadBack::GiveUpOnTheRoot,
        cache,
    ) {
        None => judgements.part_of_the_range_left_unjudged(
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
            format!(
                "最新根（txg {newest_txg}）的树表读不出、条目窄过字段表，或有一棵分配记录树读不出：最新根的记录不全，走到的引用起点没比"
            ),
        ),
        Some(AllocationRecordsOfARootRecord {
            tree_table_pointer: TreeTablePointerOfTheRootRecord::AllZero,
            ..
        }) => judgements.part_of_the_range_left_unjudged(
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
            format!(
                "最新根（txg {newest_txg}）的树表指针全零：树表读不出（core 照它读盘 0 槽 0），最新根的记录无从读起，走到的引用起点没比"
            ),
        ),
        Some(AllocationRecordsOfARootRecord {
            tree_table_entries: 0,
            ..
        }) => {
            let allocation_record_tree_held_by_the_root_record = parse_node_pointer(
                &newest_root.record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT],
            );
            if allocation_record_tree_held_by_the_root_record.all_zero {
                judgements.not_applicable(
                    WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
                    NO_ALLOCATION_RECORD_TREE_IN_THE_TREE_TABLE_OF_THE_NEWEST_ROOT,
                );
                return;
            }
            let central_mapping = central_mapping_of_a_root_record_without_judging(
                reader,
                &newest_root.record_bytes,
                cache,
            );
            match allocation_record_tree_without_judging(
                reader,
                &allocation_record_tree_held_by_the_root_record,
                &central_mapping,
                cache,
            ) {
                None => judgements.part_of_the_range_left_unjudged(
                    WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
                    format!(
                        "最新根（txg {newest_txg}）的树表 0 条、根记录直接持有的那棵分配记录树读不出：最新根的记录不全，走到的引用起点没比"
                    ),
                ),
                Some((_, records)) => judge_walked_reference_starts_against_the_counted_records(
                    reader,
                    &records,
                    start_slots_referenced_by_every_walked_version,
                    reclaim_threshold_computed_by_the_checker,
                    newest_txg,
                    judgements,
                ),
            }
        }
        Some(AllocationRecordsOfARootRecord {
            tree_table_entries,
            allocation_record_tree_entries: 0,
            ..
        }) => judgements.part_of_the_range_left_unjudged(
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
            format!(
                "最新根（txg {newest_txg}）的树表有 {tree_table_entries} 条、一条种类 3（分配记录树）都没有（I-9.16 ④ 判它）：走到的引用起点没比"
            ),
        ),
        Some(AllocationRecordsOfARootRecord { records, .. }) => {
            judge_walked_reference_starts_against_the_counted_records(
                reader,
                &records,
                start_slots_referenced_by_every_walked_version,
                reclaim_threshold_computed_by_the_checker,
                newest_txg,
                judgements,
            );
        }
    }
}

/// I-3.21 的判法本身（[`judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root`] 读好了最新根要比的那一份记录）：
/// 池里每块盘上走到的每个引用起点，在 `records` 里有一条起点在它、未释放或释放代高于 θ_high 的记录。池外的盘上的起点不判（射程同 I-3.1）。
fn judge_walked_reference_starts_against_the_counted_records(
    reader: &dyn ImageReader,
    records: &[AllocationRecordView],
    start_slots_referenced_by_every_walked_version: &BTreeSet<(u32, u64)>,
    reclaim_threshold_computed_by_the_checker: u64,
    newest_txg: u64,
    judgements: &mut Judgements,
) {
    let devices_of_the_pool: BTreeSet<u32> = reader.devices().into_iter().collect();
    let starts_of_the_counted_records: BTreeSet<(u32, u64)> = records
        .iter()
        .filter(|record| {
            !record.is_released || record.generation > reclaim_threshold_computed_by_the_checker
        })
        .map(|record| (record.device, record.slot))
        .collect();
    for (device, start_slot) in start_slots_referenced_by_every_walked_version
        .iter()
        .filter(|(device, _)| devices_of_the_pool.contains(device))
    {
        judgements.judge_naming_the_slots(
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
            starts_of_the_counted_records.contains(&(*device, *start_slot)),
            || vec![(*device, *start_slot)],
            || {
                format!(
                    "盘 {device} 槽 {start_slot}：遍历走到的引用从这一槽起，最新根（txg {newest_txg}）的分配记录里没有一条起点在这一槽、\
                     未释放或释放代高于 checker 现算的回收门槛 {reclaim_threshold_computed_by_the_checker} 的记录"
                )
            },
        );
    }
}

/// I-3.17（隔离记号与引用相符，D19（块指针的结构与宽度预算） 已定项 5「新加判据『记号与引用相符』」）：最新根的分配记录树里
/// 三句，任一句不成立判红——① 带记号的记录未释放（隔离记号 0x4000 与已释放标志 0x8000 不同时置）；② 带记号的记录罩住的槽不被最新根引用
/// （`slots_covered_by_the_newest_root`：最新根那一遍走读记下的每一个 (盘, 槽)）；③ 同一单元（起点槽、跨度）各盘那条记录的记号要么都有、要么都没有。
/// ①② 每条带记号的记录各判一格，③ 每个有记录带记号的单元判一格；最新根的树表读不出时报射程里有一部分判不了，一条带记号的记录都没有时报不适用。
pub(super) fn judge_quarantine_marks_against_the_references(
    reader: &dyn ImageReader,
    root: &crate::RootView,
    slots_covered_by_the_newest_root: &BTreeSet<(u32, u64)>,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let Some(records) = allocation_records_of_the_newest_root_without_judging(reader, root, cache)
    else {
        // 读不出的报部分没判（C701（别的判法静默跳过读不出的部分））。
        judgements.part_of_the_range_left_unjudged(
            QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
            "最新根的树表读不出：它的分配记录树走不到".to_string(),
        );
        return;
    };
    let mut records_of_each_unit: BTreeMap<(u64, u64), Vec<AllocationRecordView>> = BTreeMap::new();
    for record in &records {
        records_of_each_unit
            .entry((record.slot, record.span_slots))
            .or_default()
            .push(*record);
    }
    let mut judged = 0u64;
    for (&(slot, span_slots), records_of_the_unit) in &records_of_each_unit {
        let marked_devices: Vec<u32> = records_of_the_unit
            .iter()
            .filter(|record| record.quarantine_mark == QuarantineMarkInTheSpanField::Present)
            .map(|record| record.device)
            .collect();
        if marked_devices.is_empty() {
            continue;
        }
        for record in records_of_the_unit
            .iter()
            .filter(|record| record.quarantine_mark == QuarantineMarkInTheSpanField::Present)
        {
            judged += 1;
            judgements.judge_naming_the_slots(
                QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
                !record.is_released,
                || vec![(record.device, slot)],
                || {
                    format!(
                        "①：盘 {} 槽 {slot} 的分配记录隔离记号与已释放标志同时置（跨度段 0x4000 与 0x8000）",
                        record.device
                    )
                },
            );
            let covered_slots_referenced: Vec<u64> = (slot..slot.saturating_add(span_slots))
                .filter(|covered_slot| {
                    slots_covered_by_the_newest_root.contains(&(record.device, *covered_slot))
                })
                .collect();
            judgements.judge_naming_the_slots(
                QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
                covered_slots_referenced.is_empty(),
                || vec![(record.device, slot)],
                || {
                    format!(
                        "②：盘 {} 槽 {slot} 的分配记录带隔离记号，它罩住的槽 {covered_slots_referenced:?} 被最新根（txg {}）引用",
                        record.device, root.checkpoint_txg
                    )
                },
            );
        }
        let unmarked_devices: Vec<u32> = records_of_the_unit
            .iter()
            .filter(|record| record.quarantine_mark == QuarantineMarkInTheSpanField::Absent)
            .map(|record| record.device)
            .collect();
        judgements.judge_naming_the_slots(
            QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
            unmarked_devices.is_empty(),
            || {
                unmarked_devices
                    .iter()
                    .map(|device| (*device, slot))
                    .collect()
            },
            || {
                format!(
                    "③：起点槽 {slot}、跨 {span_slots} 槽的单元，盘 {marked_devices:?} 上的记录带隔离记号，盘 {unmarked_devices:?} 上的不带"
                )
            },
        );
    }
    if judged == 0 {
        judgements.not_applicable(
            QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
            NO_RECORD_CARRYING_THE_QUARANTINE_MARK_UNDER_THE_NEWEST_ROOT,
        );
    }
}

/// 一条根的分配记录树里有几条记录：按 checker 自己的字段表读这条根的树表，找种类 3（分配记录）的条目，数它根节点叶里的条目
/// （第一版分配记录树只有一个节点；一条记录记一个单元，D3（空间分配） 已定项 7）。只读不判——给理想模型的对拍数分配记录墙的真条数
/// （从镜像上现数，不用分配器的状态）。
/// 树表里没有分配记录树的条目、或那一条的根指针全零时是 0 条：树表 0 条的一版上没有分配记录树，mkfs 写的两个单元的记录要到
/// 第一个文件版本才落盘。树表指针全零、树表或分配记录树的节点读不出或条目比字段表窄、分配记录树有内部节点（内部条目格式没有条款，
/// 总审核 D8-D11 发现 16）时数不出，交回 None。
#[must_use]
pub fn allocation_record_count_under_root(
    reader: &dyn ImageReader,
    root: &crate::RootView,
) -> Option<usize> {
    let mut cache = IndexNodeCache::new();
    let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return None;
    }
    let tree_table = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        &mut cache,
    )?;
    let central_mapping =
        central_mapping_of_a_root_record_without_judging(reader, &root.record_bytes, &mut cache);
    let mut records = 0;
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() {
            return None;
        }
        if read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let (_, records_of_the_tree) = allocation_record_tree_without_judging(
            reader,
            &allocation_root,
            &central_mapping,
            &mut cache,
        )?;
        records += records_of_the_tree.len();
    }
    Some(records)
}

#[cfg(test)]
mod tests;
