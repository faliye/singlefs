use super::*;

/// I-3.1 钉带出来的那一截（C773 合成形态，`05-空间记账.md` I-3.1 那一行「钉落盘之后的并法」）：释放代落在 (t, 不带钉现算的门槛] 的已释放记录罩住的槽并进来；
/// 释放代落在 t 及以下（写者已回收）或门槛之上（F 之下的那一份管）、仍分配的不并；被抛弃根引用着的不并（C684 保持「不并」）；已经数过的不重复。
#[test]
fn the_band_the_pin_kept_takes_released_slots_above_the_root_threshold_up_to_the_threshold_without_the_pin_except_abandoned_references(
) {
    let records = [
        allocation_record_on_device_zero(50_200, 2, 7, true),
        allocation_record_on_device_zero(50_210, 1, 6, true),
        allocation_record_on_device_zero(50_220, 1, 12, true),
        allocation_record_on_device_zero(50_230, 1, 9, false),
        allocation_record_on_device_zero(50_240, 2, 11, true),
        allocation_record_on_device_zero(50_250, 1, 8, true),
    ];
    let referenced_by_an_abandoned_root = BTreeSet::from([(0, 50_241)]);
    let already_counted = BTreeSet::from([(0, 50_250)]);
    assert_eq!(
            slots_the_pin_kept_in_defer_among(
                &records,
                6,
                11,
                &referenced_by_an_abandoned_root,
                &already_counted
            ),
            BTreeSet::from([(0, 50_200), (0, 50_201), (0, 50_240)]),
            "t = 6、不带钉的门槛 11：释放代 7 的两槽、11 的那一条去掉被抛弃根引用的 50241；释放代 6、12 的与仍分配的不并，已数过的 50250 不重复"
        );
}

/// 钉槽 p 的读法（C773）：255 是没有钉着；小于环长 3S 的还原成 (区域 p mod 3, 槽 p div 3)，钉值是那一槽可能住过的最旧 txg
/// （槽 0 是第 0 代根的 0，别的槽是 p 本身 采乙）；
/// 不是 255 又不小于 3S 的还原不成一个槽。
#[test]
fn the_pinned_ring_slot_byte_reads_as_no_pin_a_ring_slot_or_outside_the_ring() {
    let geometry_of_a_ring_of_twelve = geometry_of_a_ring_of_twelve();
    assert_eq!(
        pinned_ring_slot_of_the_root(255, &geometry_of_a_ring_of_twelve),
        PinnedRingSlotOfTheRoot::NoSlotPinned
    );
    assert_eq!(
        pinned_ring_slot_of_the_root(8, &geometry_of_a_ring_of_twelve),
        PinnedRingSlotOfTheRoot::Pinned {
            pinned_txg: 8,
            region: 2,
            slot: 2
        },
        "p 8 = 3 × 槽 2 + 区域 2，钉值就是 8"
    );
    for (pinned_ring_slot, region) in [(0u8, 0u64), (1, 1), (2, 2)] {
        assert_eq!(
            pinned_ring_slot_of_the_root(pinned_ring_slot, &geometry_of_a_ring_of_twelve),
            PinnedRingSlotOfTheRoot::Pinned {
                pinned_txg: 0,
                region,
                slot: 0
            },
            "p {pinned_ring_slot} 是区域 {region} 的槽 0：那一槽可能还住着第 0 代根，钉值 0"
        );
    }
    assert_eq!(
        pinned_ring_slot_of_the_root(12, &geometry_of_a_ring_of_twelve),
        PinnedRingSlotOfTheRoot::OutsideTheRing(12),
        "环长 12：p 12 不在环里"
    );
}

/// C773 第 0 代根那一形（`.claude/kb/checks-owed.md` C773 登记行写者一侧那处疑点
/// 第二节 R1 采乙）：环长 12，第 0 代根留在区域 1 槽 0、那一槽在写行那次挂载时读坏，写行那次（实例 3、txg 14，落区域 2 槽 0）照条款钉
/// 区域 1 槽 0、钉值取第 0 代根的 0，写下 t = max(F 0, 0) = 0、p = 3 × 0 + 1 = 1。镜像上那一槽仍读不出（不在根环里），
/// 环里别的根是区域 0 槽 0 的 txg 12 与槽 1–3 的 txg 3–11（实例 2）。
/// 池级 checker 照钉值 0 现算 max(0, min(3, 0)) = 0 = t，I-7.18 成立；p 指的区域 1 槽 0 没有本实例的根，I-7.19 成立。
/// 把 p 当钉值的读法现算 max(0, min(3, 1)) = 1，I-7.18 误红。
#[test]
fn row_root_pinning_the_unreadable_genesis_slot_of_region_one_at_txg_zero_holds_the_reclaim_threshold_and_the_pinned_slot_invariants(
) {
    let geometry = geometry_of_a_ring_of_twelve();
    let mut roots: Vec<(u64, u64, crate::RootView)> = (3..=12u64)
        .map(|checkpoint_txg| {
            (
                checkpoint_txg % geometry.regions,
                (checkpoint_txg / geometry.regions) % geometry.slots_per_region,
                ring_root_of(2, checkpoint_txg, 0, 0, ROOT_RECORD_NO_RING_SLOT_PINNED),
            )
        })
        .collect();
    roots.push((2, 0, ring_root_of(3, 14, 0, 0, 1)));
    assert!(
        !roots
            .iter()
            .any(|(region, slot, _)| (*region, *slot) == (1, 0)),
        "区域 1 槽 0 读不出，不在根环里"
    );
    let newest_index = roots.len() - 1;
    let mut judgements = Judgements::default();
    judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(
        &roots,
        newest_index,
        &geometry,
        &|_| RootStandingInTheNewestRootsTable::Valid,
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,
        AccountOfTheNewestRoot::BuiltByThisPublish,
        &mut judgements,
    );
    let report: BTreeMap<&'static str, InvariantVerdict> =
        judgements.into_report().into_iter().collect();
    assert_eq!(
        report[ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD],
        InvariantVerdict::Holds,
        "p 1 是区域 1 槽 0、钉值 0：现算 max(0, min(3, 0)) = 0，等于写下的 t 0"
    );
    assert_eq!(
        report[PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE],
        InvariantVerdict::Holds,
        "区域 1 槽 0 没有实例 3 的根"
    );
}

/// I-3.21 前置那两形用例里最新根的树表住的单元区槽号。
const SLOT_OF_THE_TREE_TABLE_OF_THE_NEWEST_ROOT_OF_THE_WALKED_START_CASES: u64 = 2;

/// I-3.21 前置那两形用例共用：最新根（实例 1、txg 3）的根记录只带树表指针 `tree_table_pointer`（别的指针全零），
/// 遍历走到一个引用起点（盘 0、槽 7），checker 现算的回收门槛 0；在 `reader` 上判这一条，交回它的结论。
fn walked_reference_start_verdict_against_the_newest_root_carrying_the_tree_table_pointer(
    reader: &dyn ImageReader,
    tree_table_pointer: &[u8],
) -> InvariantVerdict {
    let mut newest_root = ring_root_of(1, 3, 0, 0, ROOT_RECORD_NO_RING_SLOT_PINNED);
    newest_root.record_bytes = vec![0u8; 466];
    newest_root.record_bytes[36..122].copy_from_slice(tree_table_pointer);
    let mut judgements = Judgements::default();
    judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root(
        reader,
        &newest_root,
        &BTreeSet::from([(0u32, 7u64)]),
        0,
        &mut IndexNodeCache::new(),
        &mut judgements,
    );
    verdict_of(
        judgements,
        WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
    )
}

/// I-3.21 前置不满足那一形（「归属」：前置是最新根的树表
/// 带种类 3 记录树且读全，不满足报这一条判不了）：最新根的树表读得出、有两条（inode 树、extent 树），一条种类 3（分配记录树）都没有
/// （I-9.16 ④ 判那块畸形）。最新根的记录无从比，走到的引用起点报部分没判、说明点名树表有几条而没有种类 3；不报成立、不报红，
/// 也不报树表 0 条那一形的不适用。
#[test]
fn walked_reference_starts_are_partially_judged_when_the_tree_table_of_the_newest_root_has_entries_but_no_allocation_record_tree(
) {
    let tree_table = tree_table_node_with_both_user_visible_tree_roots_marked(b'A');
    let image = ImageHoldingUnitsAtSlots::of(&[(
        SLOT_OF_THE_TREE_TABLE_OF_THE_NEWEST_ROOT_OF_THE_WALKED_START_CASES,
        tree_table.as_slice(),
    )]);
    let tree_table_pointer = image.node_pointer_to_slot(
        SLOT_OF_THE_TREE_TABLE_OF_THE_NEWEST_ROOT_OF_THE_WALKED_START_CASES,
        node_bytes(),
        0,
    );
    let verdict =
        walked_reference_start_verdict_against_the_newest_root_carrying_the_tree_table_pointer(
            &image,
            &tree_table_pointer,
        );
    assert!(
            matches!(&verdict, InvariantVerdict::PartiallyJudged(unjudged) if unjudged.contains("的树表有 2 条、一条种类 3（分配记录树）都没有")),
            "树表非空而一条种类 3 都没有：I-3.21 的前置不满足，应当报部分没判、点名树表 2 条而没有种类 3，实际 {verdict:?}"
        );
}

/// I-3.21 前置不满足的另一形（同一行没读全）：最新根的树表指针指着的那一槽在镜像上读不出。最新根的记录读不全，
/// 走到的引用起点报部分没判、说明点名树表读不出；不报成立、不报红。
#[test]
fn walked_reference_starts_are_partially_judged_when_the_tree_table_of_the_newest_root_does_not_read_back(
) {
    let tree_table = tree_table_node_with_both_user_visible_tree_roots_marked(b'A');
    let tree_table_pointer = ImageHoldingUnitsAtSlots::of(&[(
        SLOT_OF_THE_TREE_TABLE_OF_THE_NEWEST_ROOT_OF_THE_WALKED_START_CASES,
        tree_table.as_slice(),
    )])
    .node_pointer_to_slot(
        SLOT_OF_THE_TREE_TABLE_OF_THE_NEWEST_ROOT_OF_THE_WALKED_START_CASES,
        node_bytes(),
        0,
    );
    let image_without_the_tree_table = ImageHoldingUnitsAtSlots::of(&[]);
    let verdict =
        walked_reference_start_verdict_against_the_newest_root_carrying_the_tree_table_pointer(
            &image_without_the_tree_table,
            &tree_table_pointer,
        );
    assert!(
        matches!(&verdict, InvariantVerdict::PartiallyJudged(unjudged) if unjudged.contains("的树表读不出")),
        "最新根的树表读不出：I-3.21 的前置不满足，应当报部分没判、点名树表读不出，实际 {verdict:?}"
    );
}

// ---- I-3.1（已分配统计对得上） RCW 的小世界（C697、C685；盘面的取法沿用 LR5 那一份小世界） ----
//
// 取值域与上界（穷举的范围就是这几条）：
// - 一块盘（盘 0），单元槽 0..SMALL_WORLD_UNIT_SLOT_COUNT（2 个），每个单元跨 1 槽；一版引用的单元是这 2 槽的任一子集。
// - 根环 SMALL_WORLD_ROOT_RING_SLOT_COUNT（4）个位置，txg t 的根落在位置 t mod 4（与 `root_ring::target_for_publish` 一样按 txg 取槽），
//   位置上一条根被同余的新 txg 盖掉就出了环（回收门槛与候选集取环里全部根，RGall；小世界里发布不跳号，环里的根本来都在一圈之内）。
// - 从 mkfs（实例 1、txg 1、引用槽 0、F = 0）起，最多施加 SMALL_WORLD_OPERATION_LIMIT（5）步，每步三种之一：
//   发布（同一实例 txg + 1，引用换成「现行引用 ∪ 可分配的槽」的任一子集：换下的记成已释放、释放代 = 新 txg，新引用的记成已分配）；
//   抬 F（F 抬到 (F, 现行 txg] 里任一值：先写系统配置，再推一次带新 F 的空发布）；
//   崩溃恢复落到环里一条根 L（按现行那一版的表有效、txg ≥ F，含现行那一版自己）：环里 txg 高于 L 的根
//   所属的实例各记一行 (实例, L 的 txg)、并进 L 的表；新实例代号 = 用过的最大 + 1，写行 txg = 环里最大 txg + 1，写行那次发布从 L 起，
//   引用照「发布」那样任选。
//   可分配的槽 = 现行没引用、账里没有记录或已释放且释放代 ≤ 回收门槛（max(F, 环里最旧有效根)）、
//   环里按现行那一版的表被抛弃的根都不引用（影子账隔离）的槽。
//   每条根带写者自己的回收门槛 t：记这条根那一刻环里（这条根盖掉的那一槽不算、算上这条根自己）按它的表有效的最旧根与 F 取大
//   （`crates/singlefs-core/src/allocator.rs` 记根的函数「这条根要盖掉的那一槽不算」）；写者的已分配计数 = 它的账里 |E(t)|
//   （记账行按回收之后的数写，`crates/singlefs-core/src/transaction.rs`「记账行按回收之后的数写」）。
// - 判的那一刻，环里有根的位置任取一个子集读不出（含现行那一版自己的根槽：那时最新根换成读得出的最大那条，账、表与 t 都取它的）。
// - 不在小世界里的：由记录施加出来、根槽从没落盘的那一版（候选集另补的那一种），读得出而走读读到一半断掉的根，
//   系统配置读不出（F 生效值只取根上的）、隔离、挂载时的钉。
//
// 判法：每个（盘面、读不出的位置子集）上照 checker 的取法算候选集遍历、θ_high（`deferred_slots_reclaim_threshold`），
// 把最新根的账当分配记录交给 `allocated_statistic_bounds_of_a_device`、`allocated_statistic_lies_within_its_bounds`：
// 合法写者（计数 = |E(t)|）一个都不许红；另算改成 RCW 之前的等式（遍历 + 丁-defer 原式并进的槽 == 计数），它在合法盘面上红的数不为 0
// （判别力：旧等式在合法世界误红、RCW 0 误红）。三种坏写者各判一遍：计数少记一槽；凭空多一条未释放记录、计数跟着加（泄漏）；
// 丢掉遍历走到的一槽的记录、计数跟着减。后两种 RCW 必须全红（分别只有上沿里 |Ω ∪ U| 那一半、逐槽那一判抓得到）。

const SMALL_WORLD_UNIT_SLOT_COUNT: u64 = 2;
const SMALL_WORLD_ROOT_RING_SLOT_COUNT: u64 = 4;
const SMALL_WORLD_OPERATION_LIMIT: usize = 5;
const SMALL_WORLD_DEVICE: u32 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SmallWorldVersion {
    instance: u32,
    checkpoint_txg: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SmallWorldRecord {
    Allocated,
    Released {
        release_generation: u64,
        /// 换下这一槽的那次发布之前的现行那一版（引用着这一槽）：只给判例外那一形用，盘上没有这个字段。
        version_released_from: SmallWorldVersion,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SmallWorldRoot {
    version: SmallWorldVersion,
    rollback_floor: u64,
    referenced_unit_slots: BTreeSet<u64>,
    account: BTreeMap<u64, SmallWorldRecord>,
    /// 实例表：实例 → 它最后一版的 txg Ti；(i, T) 被抛弃 ⟺ 有 i 的行且 T > Ti。
    instance_table_rows: BTreeMap<u32, u64>,
    /// 写者记这条根时的回收门槛 t（根记录偏移 428）。
    reclaim_threshold_on_the_root: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SmallWorldPool {
    /// 根环位置 → 住在那里的根。
    ring: BTreeMap<u64, SmallWorldRoot>,
    current: SmallWorldRoot,
    system_configuration_floor: u64,
    highest_instance_used: u32,
}

fn small_world_standing(
    instance_table_rows: &BTreeMap<u32, u64>,
    version: SmallWorldVersion,
) -> RootStandingInTheNewestRootsTable {
    match instance_table_rows.get(&version.instance) {
        Some(last_txg_of_the_instance) if version.checkpoint_txg > *last_txg_of_the_instance => {
            RootStandingInTheNewestRootsTable::Abandoned
        }
        Some(_) | None => RootStandingInTheNewestRootsTable::Valid,
    }
}

/// 写进一条新根之前那一刻可分配的槽：门槛按 `instance_table_rows` 判有效、取环里全部有效根。
fn small_world_allocatable_unit_slots(
    pool: &SmallWorldPool,
    base: &SmallWorldRoot,
    instance_table_rows: &BTreeMap<u32, u64>,
) -> BTreeSet<u64> {
    let reclaim_threshold = deferred_slots_reclaim_threshold(
        pool.ring.values().map(|root| {
            (
                small_world_standing(instance_table_rows, root.version),
                root.version.checkpoint_txg,
            )
        }),
        pool.system_configuration_floor,
    );
    let isolated_by_the_shadow_ledger: BTreeSet<u64> = pool
        .ring
        .values()
        .filter(|root| {
            small_world_standing(instance_table_rows, root.version)
                == RootStandingInTheNewestRootsTable::Abandoned
        })
        .flat_map(|root| root.referenced_unit_slots.iter().copied())
        .collect();
    (0..SMALL_WORLD_UNIT_SLOT_COUNT)
        .filter(|unit_slot| {
            !base.referenced_unit_slots.contains(unit_slot)
                && !isolated_by_the_shadow_ledger.contains(unit_slot)
                && match base.account.get(unit_slot) {
                    None => true,
                    Some(SmallWorldRecord::Allocated) => false,
                    Some(SmallWorldRecord::Released {
                        release_generation, ..
                    }) => *release_generation <= reclaim_threshold,
                }
        })
        .collect()
}

/// `candidates` 的全部子集。
fn small_world_subsets(candidates: &BTreeSet<u64>) -> Vec<BTreeSet<u64>> {
    let members: Vec<u64> = candidates.iter().copied().collect();
    (0..1u32 << members.len())
        .map(|membership_mask| {
            members
                .iter()
                .enumerate()
                .filter(|(member_index, _)| membership_mask & (1 << member_index) != 0)
                .map(|(_, unit_slot)| *unit_slot)
                .collect()
        })
        .collect()
}

/// 从 `base` 那一版发一版新的、写进根环、成为现行那一版。
fn small_world_publish(
    pool: &SmallWorldPool,
    base: &SmallWorldRoot,
    version: SmallWorldVersion,
    referenced_unit_slots: BTreeSet<u64>,
    instance_table_rows: BTreeMap<u32, u64>,
) -> SmallWorldPool {
    let mut account = base.account.clone();
    for released in base
        .referenced_unit_slots
        .difference(&referenced_unit_slots)
    {
        account.insert(
            *released,
            SmallWorldRecord::Released {
                release_generation: version.checkpoint_txg,
                version_released_from: base.version,
            },
        );
    }
    for allocated in referenced_unit_slots.difference(&base.referenced_unit_slots) {
        account.insert(*allocated, SmallWorldRecord::Allocated);
    }
    let position = version.checkpoint_txg % SMALL_WORLD_ROOT_RING_SLOT_COUNT;
    // t：这条根盖掉的那一槽不算，算上这条根自己（它的 txg 最大，只在环里只剩它时才定下门槛）。
    let reclaim_threshold_on_the_root = deferred_slots_reclaim_threshold(
        pool.ring
            .iter()
            .filter(|(ring_position, _)| **ring_position != position)
            .map(|(_, root)| root.version)
            .chain(std::iter::once(version))
            .map(|ring_version| {
                (
                    small_world_standing(&instance_table_rows, ring_version),
                    ring_version.checkpoint_txg,
                )
            }),
        pool.system_configuration_floor,
    );
    let root = SmallWorldRoot {
        version,
        rollback_floor: pool.system_configuration_floor,
        referenced_unit_slots,
        account,
        instance_table_rows,
        reclaim_threshold_on_the_root,
    };
    let mut next = pool.clone();
    next.ring.insert(position, root.clone());
    next.current = root;
    next
}

fn small_world_successors(pool: &SmallWorldPool) -> Vec<SmallWorldPool> {
    let mut successors = Vec::new();
    let current = &pool.current;
    let next_version_of_this_instance = SmallWorldVersion {
        instance: current.version.instance,
        checkpoint_txg: current.version.checkpoint_txg + 1,
    };
    // 发布。
    let allocatable =
        small_world_allocatable_unit_slots(pool, current, &current.instance_table_rows);
    let reachable: BTreeSet<u64> = current
        .referenced_unit_slots
        .union(&allocatable)
        .copied()
        .collect();
    for referenced_unit_slots in small_world_subsets(&reachable) {
        successors.push(small_world_publish(
            pool,
            current,
            next_version_of_this_instance,
            referenced_unit_slots,
            current.instance_table_rows.clone(),
        ));
    }
    // 抬 F：先写系统配置，再推一次带新 F 的空发布。
    for new_floor in pool.system_configuration_floor + 1..=current.version.checkpoint_txg {
        let mut raised = pool.clone();
        raised.system_configuration_floor = new_floor;
        successors.push(small_world_publish(
            &raised,
            current,
            next_version_of_this_instance,
            current.referenced_unit_slots.clone(),
            current.instance_table_rows.clone(),
        ));
    }
    // 崩溃恢复落到环里一条根 L。
    let highest_txg_in_the_ring = pool
        .ring
        .values()
        .map(|root| root.version.checkpoint_txg)
        .max()
        .expect("环里至少有现行那一版");
    for landed in pool.ring.values() {
        let landed_txg = landed.version.checkpoint_txg;
        let can_land = small_world_standing(&current.instance_table_rows, landed.version)
            == RootStandingInTheNewestRootsTable::Valid
            && landed_txg >= pool.system_configuration_floor;
        if !can_land {
            continue;
        }
        let mut instance_table_rows = landed.instance_table_rows.clone();
        for newer in pool
            .ring
            .values()
            .filter(|root| root.version.checkpoint_txg > landed_txg)
        {
            let row = instance_table_rows
                .entry(newer.version.instance)
                .or_insert(landed_txg);
            *row = (*row).min(landed_txg);
        }
        let mut recovered = pool.clone();
        recovered.highest_instance_used += 1;
        recovered.current = landed.clone();
        let row_publish_version = SmallWorldVersion {
            instance: recovered.highest_instance_used,
            checkpoint_txg: highest_txg_in_the_ring + 1,
        };
        let allocatable_after_the_recovery =
            small_world_allocatable_unit_slots(&recovered, landed, &instance_table_rows);
        let reachable_after_the_recovery: BTreeSet<u64> = landed
            .referenced_unit_slots
            .union(&allocatable_after_the_recovery)
            .copied()
            .collect();
        for referenced_unit_slots in small_world_subsets(&reachable_after_the_recovery) {
            successors.push(small_world_publish(
                &recovered,
                landed,
                row_publish_version,
                referenced_unit_slots,
                instance_table_rows.clone(),
            ));
        }
    }
    successors
}

fn small_world_slot_keys(unit_slots: &BTreeSet<u64>) -> impl Iterator<Item = (u32, u64)> + '_ {
    unit_slots
        .iter()
        .map(|unit_slot| (SMALL_WORLD_DEVICE, *unit_slot))
}

/// 最新根的账当成分配记录（每个单元跨 1 槽）交给 RCW 的判法。
fn small_world_records_of_the_account(
    account: &BTreeMap<u64, SmallWorldRecord>,
) -> Vec<AllocationRecordView> {
    account
        .iter()
        .map(|(unit_slot, record)| {
            let (generation, is_released) = match record {
                SmallWorldRecord::Allocated => (0, false),
                SmallWorldRecord::Released {
                    release_generation, ..
                } => (*release_generation, true),
            };
            AllocationRecordView {
                device: SMALL_WORLD_DEVICE,
                slot: *unit_slot,
                span_slots: 1,
                generation,
                is_released,
                quarantine_mark: QuarantineMarkInTheSpanField::Absent,
            }
        })
        .collect()
}

/// 写者在一份账上按门槛 t 记的已分配槽数：未释放的加释放代高于 t 的（记账行按回收之后的数写）。
fn small_world_allocated_slots_the_writer_counts(
    account: &BTreeMap<u64, SmallWorldRecord>,
    reclaim_threshold_on_the_root: u64,
) -> u64 {
    u64::try_from(
        account
            .values()
            .filter(|record| match record {
                SmallWorldRecord::Allocated => true,
                SmallWorldRecord::Released {
                    release_generation, ..
                } => *release_generation > reclaim_threshold_on_the_root,
            })
            .count(),
    )
    .expect("小世界的槽数")
}

/// checker 在一个（盘面、读不出的位置）上看到的：最新根、候选集遍历走到的槽、现算的门槛 θ_high，
/// 与改成 RCW 之前那条等式右边的「遍历 + 丁-defer 原式并进的槽」。
struct SmallWorldSeenByTheChecker<'pool> {
    newest: &'pool SmallWorldRoot,
    walked_slots: BTreeSet<(u32, u64)>,
    reclaim_threshold_computed_by_the_checker: u64,
    walked_and_deferred_slots_of_the_former_equation: u64,
}

/// 读不出之后环里一条根都不剩时交 `None`。
fn small_world_seen_by_the_checker<'pool>(
    pool: &'pool SmallWorldPool,
    unreadable_positions: &BTreeSet<u64>,
) -> Option<SmallWorldSeenByTheChecker<'pool>> {
    let readable: Vec<&SmallWorldRoot> = pool
        .ring
        .iter()
        .filter(|(position, _)| !unreadable_positions.contains(position))
        .map(|(_, root)| root)
        .collect();
    let newest = *readable
        .iter()
        .max_by_key(|root| (root.version.checkpoint_txg, root.version.instance))?;
    let standing =
        |version: SmallWorldVersion| small_world_standing(&newest.instance_table_rows, version);
    let effective_rollback_floor = readable
        .iter()
        .filter(|root| {
            root.version == newest.version
                || standing(root.version) == RootStandingInTheNewestRootsTable::Abandoned
        })
        .map(|root| root.rollback_floor)
        .fold(pool.system_configuration_floor, u64::max);
    let walked_slots: BTreeSet<(u32, u64)> = readable
        .iter()
        .filter(|root| {
            root.version == newest.version
                || (standing(root.version) == RootStandingInTheNewestRootsTable::Valid
                    && root.version.checkpoint_txg >= effective_rollback_floor)
        })
        .flat_map(|root| small_world_slot_keys(&root.referenced_unit_slots))
        .collect();
    let reclaim_threshold_computed_by_the_checker = deferred_slots_reclaim_threshold(
        readable
            .iter()
            .map(|root| (standing(root.version), root.version.checkpoint_txg)),
        effective_rollback_floor,
    );
    let released_above_the_reclaim_threshold: BTreeSet<(u32, u64)> =
        small_world_records_of_the_account(&newest.account)
            .iter()
            .filter(|record| {
                allocation_record_is_released_above_the_reclaim_threshold(
                    record,
                    reclaim_threshold_computed_by_the_checker,
                )
            })
            .map(|record| (record.device, record.slot))
            .collect();
    let deferred_by_the_former_selection =
        deferred_slots_among_the_references_of_the_selected_roots(
            readable
                .iter()
                .filter(|root| {
                    root_references_are_merged_into_the_deferred_slots_below_the_floor(
                        standing(root.version),
                        root.version.checkpoint_txg,
                        effective_rollback_floor,
                    )
                })
                .flat_map(|root| small_world_slot_keys(&root.referenced_unit_slots)),
            &walked_slots,
            &released_above_the_reclaim_threshold,
        );
    let walked_and_deferred_slots_of_the_former_equation =
        u64::try_from(walked_slots.len() + deferred_by_the_former_selection.len())
            .expect("小世界的槽数");
    Some(SmallWorldSeenByTheChecker {
        newest,
        walked_slots,
        reclaim_threshold_computed_by_the_checker,
        walked_and_deferred_slots_of_the_former_equation,
    })
}

/// RCW 在小世界里判一份（账、计数）：计数按槽数给，交 `allocated_statistic_lies_within_its_bounds` 的结论。
fn small_world_allocated_statistic_holds(
    seen: &SmallWorldSeenByTheChecker<'_>,
    account: &BTreeMap<u64, SmallWorldRecord>,
    allocated_slots: u64,
) -> bool {
    let bounds = allocated_statistic_bounds_of_a_device(
        SMALL_WORLD_DEVICE,
        &small_world_records_of_the_account(account),
        &seen.walked_slots,
        u64::try_from(seen.walked_slots.len()).expect("小世界的槽数"),
        &BTreeSet::new(),
        0,
        seen.reclaim_threshold_computed_by_the_checker,
        seen.newest.reclaim_threshold_on_the_root,
    );
    allocated_statistic_lies_within_its_bounds(Some(allocated_slots * SLOT_BYTES), &bounds)
}

/// GPU 判器的扫描内核判丁-defer 只按 F_生效 挑已释放记录（`crates/singlefs-verification/src/crash_judge_checker_scan.wgsl` 的
/// `c_deferred_below_the_floor`），这里的门槛是 max(F_生效, 环里最旧有效根)（`deferred_slots_reclaim_threshold`）：hand_out 第二位报告
/// 「交主 agent 定的」第 1 条疑两边口径不同。丁-defer 并进的槽只来自 F_生效 之下的有效根
/// （`root_references_are_merged_into_the_deferred_slots_below_the_floor`），这样的根一有，环里最旧有效根就在 F_生效 之下，门槛就是
/// F_生效；一条都没有时并进的是空集、门槛取多少都一样。所以能并进槽的每个盘面上两边同一个门槛，造不出两边判得不同的镜像。
/// 这里在（有效 / 被抛弃，txg）的全部组合（至多三条根、txg 与 F_生效 各取 0..6、0..4）上钉住它：有一条根并得进来，门槛就是 F_生效。
/// 判别力（`crates/mutations.tsv`）：门槛取「环里最新有效根」（`min` 换成 `max`），环里另有 F_生效 之上的有效根时门槛高于 F_生效，这条红。
#[test]
fn the_deferral_threshold_is_the_effective_floor_whenever_a_root_below_the_floor_can_merge_its_slots(
) {
    let standings = [
        RootStandingInTheNewestRootsTable::Valid,
        RootStandingInTheNewestRootsTable::Abandoned,
    ];
    let choices: Vec<(RootStandingInTheNewestRootsTable, u64)> = standings
        .iter()
        .flat_map(|standing| (0..6u64).map(move |checkpoint_txg| (*standing, checkpoint_txg)))
        .collect();
    let mut rings_with_a_mergeable_root = 0u64;
    for effective_rollback_floor in 0..4u64 {
        // 至多三条根：长度 0、1、2、3 的每一串各一次（逐层从上一层长一条，不重不漏）
        let mut rings: Vec<Vec<(RootStandingInTheNewestRootsTable, u64)>> = vec![Vec::new()];
        let mut previous_layer = rings.clone();
        for _root_count in 0..3 {
            let next_layer: Vec<Vec<(RootStandingInTheNewestRootsTable, u64)>> = previous_layer
                .iter()
                .flat_map(|ring| {
                    choices.iter().map(move |choice| {
                        let mut extended = ring.clone();
                        extended.push(*choice);
                        extended
                    })
                })
                .collect();
            rings.extend(next_layer.iter().cloned());
            previous_layer = next_layer;
        }
        for ring in &rings {
            let some_root_merges_its_slots = ring.iter().any(|(standing, checkpoint_txg)| {
                root_references_are_merged_into_the_deferred_slots_below_the_floor(
                    *standing,
                    *checkpoint_txg,
                    effective_rollback_floor,
                )
            });
            if !some_root_merges_its_slots {
                continue;
            }
            rings_with_a_mergeable_root += 1;
            assert_eq!(
                    deferred_slots_reclaim_threshold(ring.iter().copied(), effective_rollback_floor),
                    effective_rollback_floor,
                    "环 {ring:?}、F_生效 {effective_rollback_floor}：有一条根并得进来，门槛就是 F_生效（GPU 扫描内核按它挑）"
                );
        }
    }
    assert!(
        rings_with_a_mergeable_root > 1000,
        "枚举到的、有根并得进来的盘面太少：{rings_with_a_mergeable_root}"
    );
}

/// C697、C685：RCW 在小世界的每个（盘面、读不出的位置）上对合法写者 0 误红，而改成 RCW 之前那条等式在合法盘面上红得出来；
/// 三种坏写者各判一遍（取值域、写者的 t 与计数、坏写者的写法见上面那一段注释）。计数钉住，换一个取样点或改坏判法会动。
#[test]
fn allocated_statistic_records_counted_within_the_walk_holds_on_every_small_world_where_the_former_equation_reddens_some(
) {
    let mkfs_root = SmallWorldRoot {
        version: SmallWorldVersion {
            instance: 1,
            checkpoint_txg: 1,
        },
        rollback_floor: 0,
        referenced_unit_slots: BTreeSet::from([0]),
        account: BTreeMap::from([(0, SmallWorldRecord::Allocated)]),
        instance_table_rows: BTreeMap::new(),
        reclaim_threshold_on_the_root: 1,
    };
    let mkfs_pool = SmallWorldPool {
        ring: BTreeMap::from([(1, mkfs_root.clone())]),
        current: mkfs_root,
        system_configuration_floor: 0,
        highest_instance_used: 1,
    };
    let mut seen_pools: BTreeSet<SmallWorldPool> = BTreeSet::from([mkfs_pool.clone()]);
    let mut frontier = vec![mkfs_pool];
    for _ in 0..SMALL_WORLD_OPERATION_LIMIT {
        let mut next_frontier = Vec::new();
        for pool in &frontier {
            for successor in small_world_successors(pool) {
                if seen_pools.insert(successor.clone()) {
                    next_frontier.push(successor);
                }
            }
        }
        frontier = next_frontier;
    }
    let mut judged_worlds = 0u64;
    let mut worlds_the_former_equation_reddens = 0u64;
    let mut worlds_counting_one_slot_less = 0u64;
    let mut worlds_counting_one_slot_less_reddened = 0u64;
    let mut worlds_with_a_leaked_record = 0u64;
    let mut worlds_with_a_walked_record_dropped = 0u64;
    for pool in &seen_pools {
        let occupied_positions: BTreeSet<u64> = pool.ring.keys().copied().collect();
        for unreadable_positions in small_world_subsets(&occupied_positions) {
            let Some(seen) = small_world_seen_by_the_checker(pool, &unreadable_positions) else {
                continue;
            };
            judged_worlds += 1;
            let account = &seen.newest.account;
            let allocated_slots = small_world_allocated_slots_the_writer_counts(
                account,
                seen.newest.reclaim_threshold_on_the_root,
            );
            assert!(
                    small_world_allocated_statistic_holds(&seen, account, allocated_slots),
                    "合法写者（计数 {allocated_slots} = |E(t)|）上 RCW 红了：盘面 {pool:?}，读不出的位置 {unreadable_positions:?}"
                );
            if seen.walked_and_deferred_slots_of_the_former_equation != allocated_slots {
                worlds_the_former_equation_reddens += 1;
            }
            // 坏写者一：计数少记一槽。
            if allocated_slots > 0 {
                worlds_counting_one_slot_less += 1;
                if !small_world_allocated_statistic_holds(&seen, account, allocated_slots - 1) {
                    worlds_counting_one_slot_less_reddened += 1;
                }
            }
            // 坏写者二（泄漏）：一个从没记过的槽凭空多一条未释放记录、计数跟着加。
            if let Some(never_recorded) =
                (0..SMALL_WORLD_UNIT_SLOT_COUNT).find(|unit_slot| !account.contains_key(unit_slot))
            {
                worlds_with_a_leaked_record += 1;
                let mut leaked = account.clone();
                leaked.insert(never_recorded, SmallWorldRecord::Allocated);
                assert!(
                        !small_world_allocated_statistic_holds(&seen, &leaked, allocated_slots + 1),
                        "泄漏一槽 {never_recorded}（记录与计数一起加）RCW 没红：盘面 {pool:?}，读不出的位置 {unreadable_positions:?}"
                    );
            }
            // 坏写者三：遍历走到的一槽、写者照算已分配的那条记录丢掉，计数跟着减。
            if let Some((_, dropped)) = seen.walked_slots.iter().find(|(_, unit_slot)| {
                account.get(unit_slot).is_some_and(|record| match record {
                    SmallWorldRecord::Allocated => true,
                    SmallWorldRecord::Released {
                        release_generation, ..
                    } => *release_generation > seen.newest.reclaim_threshold_on_the_root,
                })
            }) {
                worlds_with_a_walked_record_dropped += 1;
                let mut dropped_account = account.clone();
                dropped_account.remove(dropped);
                assert!(
                        !small_world_allocated_statistic_holds(
                            &seen,
                            &dropped_account,
                            allocated_slots - 1
                        ),
                        "遍历走到的槽 {dropped} 的记录丢了、计数跟着减，RCW 没红：盘面 {pool:?}，读不出的位置 {unreadable_positions:?}"
                    );
            }
        }
    }
    assert!(
        worlds_the_former_equation_reddens > 0,
        "改成 RCW 之前的等式在合法盘面上一个都没红：这份小世界分不出两种判法"
    );
    assert_eq!(
            (
                seen_pools.len(),
                judged_worlds,
                worlds_the_former_equation_reddens,
                worlds_counting_one_slot_less,
                worlds_counting_one_slot_less_reddened,
                worlds_with_a_leaked_record,
                worlds_with_a_walked_record_dropped,
            ),
            (180_794, 2_710_980, 714_144, 2_359_738, 1_669_786, 889_459, 1_999_587),
            "盘面数、判过的（盘面、读不出的位置）数、旧等式在合法盘面上红的数、计数少记一槽的数与其中 RCW 红的数、泄漏的数、丢记录的数"
        );
}

// ---- I-3.1（已分配统计对得上） 配不上的那块落在 I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数） 判红的单元里时报部分没判
// 盘 0 上：槽 10 起一个两槽单元，
// 槽 12 起一个一槽单元；回收门槛 θ_high 与 t 都取 5，未释放记录的分配代取 1。 ----

const SPAN_READING_RECLAIM_THRESHOLD: u64 = 5;

/// 起点槽 → 那个单元那一类占的槽数（单元头用得了的那几个；不在表里的头用不了）。
fn span_reading_unit_classes() -> BTreeMap<u64, u64> {
    BTreeMap::from([(10, 2), (12, 1), (30, 2)])
}

/// 盘 0 上按 `records` 与遍历走到的槽 `walked_unit_slots` 算 RCW 的界（遍历的槽数按走到的槽数算，没有隔离豁免）。
fn span_reading_bounds(
    records: &[AllocationRecordView],
    walked_unit_slots: &[u64],
) -> AllocatedStatisticBoundsOfADevice {
    span_reading_bounds_counting_the_walk_as(
        records,
        walked_unit_slots,
        u64::try_from(walked_unit_slots.len()).expect("槽数"),
    )
}

/// 同 [`span_reading_bounds`]，遍历的槽数给定（按引用的跨度加：几条引用罩同一槽时比走到的槽数多）。
fn span_reading_bounds_counting_the_walk_as(
    records: &[AllocationRecordView],
    walked_unit_slots: &[u64],
    walked_slot_count: u64,
) -> AllocatedStatisticBoundsOfADevice {
    let walked_slots: BTreeSet<(u32, u64)> =
        walked_unit_slots.iter().map(|slot| (0, *slot)).collect();
    allocated_statistic_bounds_of_a_device(
        0,
        records,
        &walked_slots,
        walked_slot_count,
        &BTreeSet::new(),
        0,
        SPAN_READING_RECLAIM_THRESHOLD,
        SPAN_READING_RECLAIM_THRESHOLD,
    )
}

/// 按两种数法判一块盘：交回（按记录自述那一份三判成立不成立，取宽那一支——改记那一份整份三判加逐槽两判——成立不成立；
/// 没有 I-3.18 判红的记录时后者是 `None`）。
fn span_reading_verdicts(
    records: &[AllocationRecordView],
    walked_unit_slots: &[u64],
    allocated_slots: u64,
) -> (bool, Option<bool>) {
    span_reading_verdicts_counting_the_walk_as(
        records,
        walked_unit_slots,
        u64::try_from(walked_unit_slots.len()).expect("槽数"),
        allocated_slots,
    )
}

/// 同 [`span_reading_verdicts`]，遍历的槽数给定（[`span_reading_bounds_counting_the_walk_as`]）。
fn span_reading_verdicts_counting_the_walk_as(
    records: &[AllocationRecordView],
    walked_unit_slots: &[u64],
    walked_slot_count: u64,
    allocated_slots: u64,
) -> (bool, Option<bool>) {
    let allocated = Some(allocated_slots * SLOT_BYTES);
    let as_recorded =
        span_reading_bounds_counting_the_walk_as(records, walked_unit_slots, walked_slot_count);
    let classes = span_reading_unit_classes();
    let unit_class_reading =
        records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(
            records,
            |record| classes.get(&record.slot).copied(),
        )
        .map(|records_set_to_the_unit_classes| {
            allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading(
                allocated,
                &span_reading_bounds_counting_the_walk_as(
                    &records_set_to_the_unit_classes,
                    walked_unit_slots,
                    walked_slot_count,
                ),
            )
        });
    (
        allocated_statistic_lies_within_its_bounds(allocated, &as_recorded),
        unit_class_reading,
    )
}

// ---- 取宽那一支只认改记那一份读法整份过三判、另加逐槽两判
// 下面几条里 `span_reading_verdicts` 交回的第二个数就是这一判。 ----

/// 两槽单元那条记录只记一槽、记账照两槽记（I-3.18 那份坏镜像的形状）：按记录自述 (P) 漏了槽 11、上沿少一槽，三判不成立；
/// 改记成那一类的两槽之后那一份整份三判与逐槽两判都成立——配不上的全落在 I-3.18 判红的单元里，I-3.1 报部分没判。
#[test]
fn record_one_slot_short_of_its_two_slot_unit_with_the_accounting_counting_both_slots_is_left_to_the_span_invariant(
) {
    let records = [
        allocation_record_on_device_zero(10, 1, 1, false),
        allocation_record_on_device_zero(12, 1, 1, false),
    ];
    assert_eq!(
        span_reading_bounds(&records, &[10, 11, 12]).walked_slots_outside_the_records,
        vec![11],
        "按记录自述，遍历走到的槽 11 不在记录里"
    );
    assert_eq!(
        span_reading_verdicts(&records, &[10, 11, 12], 3),
        (false, Some(true)),
        "按记录自述三判不成立，改记那一份整份成立"
    );
}

/// 同一份记录、记账只记两槽（写者照记录数：槽 10 那条一槽、槽 12 那条一槽）：按记录自述只有 (P) 不成立；改记那一份 E 是 10、11、12 三槽，
/// 记账两槽少于它，(L) 不成立，判红。改成只认改记那一份之前（下沿取两份 |E(θ_high)| 的小者 2）这一形报部分没判：窄记录漏数的那一槽
/// 与记账一起少了，记账对记录照得上，只有改记那一份看得出单元真占的槽比记账多。
#[test]
fn record_one_slot_short_of_its_two_slot_unit_with_the_accounting_counting_by_the_records_reddens_the_allocated_statistic(
) {
    let records = [
        allocation_record_on_device_zero(10, 1, 1, false),
        allocation_record_on_device_zero(12, 1, 1, false),
    ];
    assert_eq!(
        span_reading_verdicts(&records, &[10, 11, 12], 2),
        (false, Some(false)),
        "改记那一份下沿 3 槽，记账 2 槽少于它"
    );
}

/// 记录比单元那一类宽（两槽单元记三槽）、记账照单元记两槽：按记录自述下沿 3 槽多于记账，三判不成立；改记成两槽之后整份成立
/// （多罩的那一槽 32 遍历没走到、改记那一份不罩它，逐槽两判都成立），照新判法不改判。
#[test]
fn record_wider_than_its_two_slot_unit_is_left_to_the_span_invariant() {
    let records = [allocation_record_on_device_zero(30, 3, 1, false)];
    assert_eq!(
        span_reading_verdicts(&records, &[30, 31], 2),
        (false, Some(true))
    );
}

/// 宽的方向打中的那一形：
/// 槽 12 一个一槽单元起头的记录跨 3（罩 12、13、14），槽 13 是一个遍历走到、没有自己记录的一槽单元，槽 14 空着；记账照两个活单元记 2 槽。
/// 按记录自述 E 三槽，(L) 不成立；改记成一槽之后 E 只剩 12，遍历走到的槽 13 不在里面、上沿 1 槽也少于记账，(P)(U) 不成立：判红。
/// 逐判混取（下沿取改记那一份、逐槽取自述那一份、上沿取两份的大者）拼得出一种两边都不认的组合，报部分没判。
#[test]
fn record_wider_than_its_one_slot_unit_over_a_walked_neighbor_without_its_own_record_reddens_the_allocated_statistic(
) {
    let records = [allocation_record_on_device_zero(12, 3, 1, false)];
    assert_eq!(
        span_reading_verdicts(&records, &[12, 13], 2),
        (false, Some(false)),
        "改记那一份逐槽的下沿在槽 13 不成立"
    );
}

/// 逐槽的上沿：两槽单元那条记录只记一槽，另有一条未释放、罩着遍历没走到的槽 40（头用不了、不改记）的记录；遍历走到 10、11，
/// 按引用的跨度加是 3 槽（槽 10 有两条引用），记账 3 槽。改记那一份 E 是 10、11、40 三槽，(L)(U) 与逐槽的下沿都成立，
/// 而槽 40 不在遍历走到的槽里、不是隔离豁免、也不在 defer 里：逐槽的上沿不成立，判红。
#[test]
fn unit_class_reading_covering_a_slot_outside_the_walk_and_the_defer_reddens_the_allocated_statistic(
) {
    let records = [
        allocation_record_on_device_zero(10, 1, 1, false),
        allocation_record_on_device_zero(40, 1, 1, false),
    ];
    assert_eq!(
        span_reading_verdicts_counting_the_walk_as(&records, &[10, 11], 3, 3),
        (false, Some(false)),
        "改记那一份 E(t) 的槽 40 不在 Ω ∪ U 里"
    );
}

/// 两槽单元那条记录只记一槽，另有一个遍历走到、一条记录都没有的槽 20（不在任何 I-3.18 判红的单元里）；记账照三个有单元头的槽记 3 槽，
/// 改记那一份上下沿都成立，逐槽的下沿在槽 20 不成立，判红。
#[test]
fn walked_slot_outside_every_record_and_every_unit_the_span_invariant_reddens_stays_red() {
    let records = [
        allocation_record_on_device_zero(10, 1, 1, false),
        allocation_record_on_device_zero(12, 1, 1, false),
    ];
    assert_eq!(
        span_reading_verdicts(&records, &[10, 11, 12, 20], 3),
        (false, Some(false)),
        "槽 20 不在改记那一份里：判红"
    );
}

/// 跨度都对、遍历走到的槽 11 真没有记录（两槽单元的记录不见了，没有 I-3.18 判红的记录）：没有第二种数法，照判红。
#[test]
fn walked_slot_without_a_record_and_no_record_the_span_invariant_reddens_stays_red() {
    let records = [allocation_record_on_device_zero(12, 1, 1, false)];
    assert_eq!(
        span_reading_verdicts(&records, &[10, 11, 12], 3),
        (false, None)
    );
}

// ---- 取宽那一支第五判：改记之后未释放的记录罩住的槽区间不与别的记录相交
// 部分没判的说明里带的数（同一判决 Q1 第 3 问）。 ----

/// 改记那一份里（`span_reading_unit_classes` 那几个单元）相交的每一对。
fn overlapping_pairs_of_the_unit_class_reading(
    records: &[AllocationRecordView],
) -> Vec<RecordsOverlappingInTheUnitClassReading> {
    let classes = span_reading_unit_classes();
    unreleased_records_overlapping_another_record_in_the_unit_class_reading(
        records,
        &records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(
            records,
            |record| classes.get(&record.slot).copied(),
        )
        .expect("这几条用例都有 I-3.18 判红的记录"),
    )
}

/// 窄记录加尾槽上另一条记录的最小例：
/// 槽 10 起的两槽单元那条记录只记一槽，另一条已释放、释放代 4（不高于门槛 5）的记录压在它的尾槽 11 上；
/// 遍历走到 10、11，记账 2 槽。按记录自述 (P) 在槽 11 不成立；改记成两槽之后三判与逐槽两判都成立，
/// 而改记之后的 [10, 12) 与那条已释放记录在槽 11 相交：取宽那一支判红。
#[test]
fn narrowed_record_whose_tail_lies_under_a_released_record_reddens_the_unit_class_reading() {
    let narrowed = allocation_record_on_device_zero(10, 1, 1, false);
    let released_on_the_tail = allocation_record_on_device_zero(11, 1, 4, true);
    let records = [narrowed, released_on_the_tail];
    assert_eq!(
        span_reading_verdicts(&records, &[10, 11], 2),
        (false, Some(true)),
        "按记录自述三判不成立，改记那一份三判与逐槽两判都成立"
    );
    assert_eq!(
        overlapping_pairs_of_the_unit_class_reading(&records),
        vec![RecordsOverlappingInTheUnitClassReading {
            unreleased_record: AllocationRecordView {
                span_slots: 2,
                ..narrowed
            },
            unreleased_record_span: SpanInTheUnitClassReading::SetToTheUnitClass,
            other_record: released_on_the_tail,
            other_record_span: SpanInTheUnitClassReading::AsRecorded,
            first_overlapping_slot: 11,
            end_of_the_overlapping_slots: 12,
        }],
        "改记之后的 [10, 12) 与已释放的那条在槽 11 相交"
    );
}

/// 两条未释放记录各罩各的、改记之后也不相交（两槽单元那条只记一槽、一槽单元那条照记）：第五判没有相交的一对，取宽那一支照旧报部分没判。
#[test]
fn records_of_the_unit_class_reading_side_by_side_overlap_nowhere() {
    let records = [
        allocation_record_on_device_zero(10, 1, 1, false),
        allocation_record_on_device_zero(12, 1, 1, false),
    ];
    assert_eq!(
        overlapping_pairs_of_the_unit_class_reading(&records),
        Vec::new()
    );
}

/// 两条已释放的记录相交不比；一条未释放的与一条已释放的相交照比，不论谁的起点在前。
#[test]
fn records_overlapping_in_the_unit_class_reading_are_judged_only_with_an_unreleased_one() {
    let released_wide = allocation_record_on_device_zero(20, 3, 6, true);
    let released_inside = allocation_record_on_device_zero(21, 1, 7, true);
    assert_eq!(
        unreleased_records_overlapping_another_record_in_the_unit_class_reading(
            &[released_wide, released_inside],
            &[released_wide, released_inside]
        ),
        Vec::new(),
        "两条都已释放：不比"
    );
    let unreleased_inside = allocation_record_on_device_zero(21, 1, 1, false);
    assert_eq!(
        unreleased_records_overlapping_another_record_in_the_unit_class_reading(
            &[unreleased_inside, released_wide],
            &[unreleased_inside, released_wide]
        ),
        vec![RecordsOverlappingInTheUnitClassReading {
            unreleased_record: unreleased_inside,
            unreleased_record_span: SpanInTheUnitClassReading::AsRecorded,
            other_record: released_wide,
            other_record_span: SpanInTheUnitClassReading::AsRecorded,
            first_overlapping_slot: 21,
            end_of_the_overlapping_slots: 22,
        }],
        "已释放的那条起点在前、未释放的那条在它里面：照比"
    );
}

/// 第五判的违例说明按实际情形写：
/// 槽 10 起的两槽单元那条记录只记一槽（I-3.18 判红、改记成两槽），压在槽 11 那条上；另有两条在记录自述那一份里就相交
/// （槽 20 起跨 2、槽 21 起跨 1，都没被改记）。被改记的那一对写「改记之后跨 2 槽」；两条都没被改记的那一对只写跨几槽，
/// 说明里（连同开头那一句）没有「改记之后」。
#[test]
fn the_overlap_violation_does_not_say_set_to_the_unit_class_for_a_pair_neither_of_whose_records_was_set(
) {
    let classes = BTreeMap::from([(10u64, 2u64), (20, 2)]);
    let records = [
        allocation_record_on_device_zero(10, 1, 1, false),
        allocation_record_on_device_zero(11, 1, 1, false),
        allocation_record_on_device_zero(20, 2, 1, false),
        allocation_record_on_device_zero(21, 1, 1, false),
    ];
    let set_to_the_unit_classes =
        records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(
            &records,
            |record| classes.get(&record.slot).copied(),
        )
        .expect("槽 10 那条跨度段比那一类少一槽");
    let overlapping = unreleased_records_overlapping_another_record_in_the_unit_class_reading(
        &records,
        &set_to_the_unit_classes,
    );
    assert_eq!(
        overlapping
            .iter()
            .map(|pair| (pair.unreleased_record.slot, pair.other_record.slot))
            .collect::<Vec<(u64, u64)>>(),
        vec![(10, 11), (20, 21)],
        "改记之后 [10, 12) 压着槽 11；[20, 22) 与槽 21 在记录自述那一份里就相交"
    );
    let detail_of_the_pair_neither_set =
        allocated_statistic_violated_by_records_overlapping_in_the_unit_class_reading(
            0,
            &overlapping[1..],
        );
    assert!(
            !detail_of_the_pair_neither_set.contains("改记之后")
                && detail_of_the_pair_neither_set.contains(
                    "槽 20 的记录（未释放，跨 2 槽）与槽 21 的记录（未释放，跨 1 槽）在槽 [21, 22) 相交"
                ),
            "两条都没被改记：只写跨几槽、不说改记之后：{detail_of_the_pair_neither_set}"
        );
    let detail_of_both_pairs =
        allocated_statistic_violated_by_records_overlapping_in_the_unit_class_reading(
            0,
            &overlapping,
        );
    assert!(
            detail_of_both_pairs.contains(
                "槽 10 的记录（未释放，改记之后跨 2 槽）与槽 11 的记录（未释放，跨 1 槽）在槽 [11, 12) 相交"
            ),
            "被改记的那一条写改记之后跨几槽：{detail_of_both_pairs}"
        );
}

/// X 是隔离单元的那一形的盘面（
/// 手造镜像 `quarantined_unit_tail_on_a_record_released_between_t_and_theta_high`）：盘 0 四槽，t = 1、θ_high = 2；
/// 槽 0 起两槽单元 Y 最新根引用、走到，记录 (0，跨 1，未释放)；槽 2 起两槽单元 X 没有根引用，记录 A = (2，跨 1，未释放，带隔离记号，隔离豁免槽 2)；
/// 槽 3 上记录 B = (3，跨 1，已释放，释放代 2，落在 (t, θ_high]、在 U 里)；记账已分配 4 槽。交回这四条记录。
fn records_of_the_world_with_a_quarantined_unit_tail_over_a_released_record(
) -> Vec<AllocationRecordView> {
    vec![
        allocation_record_on_device_zero(0, 1, 1, false),
        AllocationRecordView {
            quarantine_mark: QuarantineMarkInTheSpanField::Present,
            ..allocation_record_on_device_zero(2, 1, 1, false)
        },
        allocation_record_on_device_zero(3, 1, 2, true),
    ]
}

/// 那一份盘面上 checker 现算的回收门槛 θ_high。
const QUARANTINED_TAIL_WORLD_CHECKER_RECLAIM_THRESHOLD: u64 = 2;
/// 那一份盘面上最新根自己带的回收门槛 t。
const QUARANTINED_TAIL_WORLD_ROOT_RECLAIM_THRESHOLD: u64 = 1;

/// 那一份盘面上按 `records` 算 RCW 的界：遍历走到 0、1（2 槽），隔离豁免槽 2（1 槽）。
fn bounds_of_the_world_with_a_quarantined_unit_tail_over_a_released_record(
    records: &[AllocationRecordView],
) -> AllocatedStatisticBoundsOfADevice {
    allocated_statistic_bounds_of_a_device(
        0,
        records,
        &BTreeSet::from([(0, 0), (0, 1)]),
        2,
        &BTreeSet::from([(0, 2)]),
        1,
        QUARANTINED_TAIL_WORLD_CHECKER_RECLAIM_THRESHOLD,
        QUARANTINED_TAIL_WORLD_ROOT_RECLAIM_THRESHOLD,
    )
}

/// 那一份盘面改记之后（Y 与 A 都改成两槽）。
fn records_of_the_world_with_a_quarantined_unit_tail_set_to_the_unit_classes(
    records: &[AllocationRecordView],
) -> Vec<AllocationRecordView> {
    let classes = BTreeMap::from([(0u64, 2u64), (2, 2)]);
    records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(records, |record| {
        classes.get(&record.slot).copied()
    })
    .expect("Y 与 A 两条跨度段比那一类少一槽")
}

/// 那一份盘面上：按记录自述 (P) 在槽 1 不成立；改记之后三判与逐槽两判都成立（槽 2 隔离豁免、槽 3 在 U 里）；
/// 改记之后 A 的 [2, 4) 与 B 在槽 3 相交——X 的尾槽压在 B 那一次释放留下、还在 defer 里的槽上——取宽那一支判红。
#[test]
fn quarantined_unit_tail_over_a_record_released_between_the_root_recorded_reclaim_threshold_and_theta_high_reddens_the_unit_class_reading(
) {
    let records = records_of_the_world_with_a_quarantined_unit_tail_over_a_released_record();
    let allocated = Some(4 * SLOT_BYTES);
    assert!(
        !allocated_statistic_lies_within_its_bounds(
            allocated,
            &bounds_of_the_world_with_a_quarantined_unit_tail_over_a_released_record(&records)
        ),
        "按记录自述，遍历走到的槽 1 不在 E(θ_high) 里"
    );
    let set_to_the_unit_classes =
        records_of_the_world_with_a_quarantined_unit_tail_set_to_the_unit_classes(&records);
    assert!(
        allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading(
            allocated,
            &bounds_of_the_world_with_a_quarantined_unit_tail_over_a_released_record(
                &set_to_the_unit_classes
            )
        ),
        "改记那一份三判与逐槽两判都成立"
    );
    assert_eq!(
        unreleased_records_overlapping_another_record_in_the_unit_class_reading(
            &records,
            &set_to_the_unit_classes
        ),
        vec![RecordsOverlappingInTheUnitClassReading {
            unreleased_record: set_to_the_unit_classes[1],
            unreleased_record_span: SpanInTheUnitClassReading::SetToTheUnitClass,
            other_record: records[2],
            other_record_span: SpanInTheUnitClassReading::AsRecorded,
            first_overlapping_slot: 3,
            end_of_the_overlapping_slots: 4,
        }],
        "A 改记成两槽之后与 B 在槽 3 相交"
    );
}

/// 部分没判的说明里带的数：Ω ∪ U 的三样各自的槽数（遍历走到 2、隔离豁免 1、U 1）、被改记的两条记录、
/// 两种读法的 E(θ_high) 差的槽（改记那一份多出槽 1 与槽 3，记录自述那一份没有多出的）。盘面取上一条那一份。
#[test]
fn allocated_statistic_left_to_the_span_invariant_names_the_walked_quarantined_and_deferred_slots_the_records_set_and_the_slots_the_readings_differ_in(
) {
    let records = records_of_the_world_with_a_quarantined_unit_tail_over_a_released_record();
    let set_to_the_unit_classes =
        records_of_the_world_with_a_quarantined_unit_tail_set_to_the_unit_classes(&records);
    let detail = allocated_statistic_left_to_the_span_invariant(
        0,
        Some(4 * SLOT_BYTES),
        QUARANTINED_TAIL_WORLD_CHECKER_RECLAIM_THRESHOLD,
        QUARANTINED_TAIL_WORLD_ROOT_RECLAIM_THRESHOLD,
        2,
        1,
        (
            &bounds_of_the_world_with_a_quarantined_unit_tail_over_a_released_record(&records),
            &bounds_of_the_world_with_a_quarantined_unit_tail_over_a_released_record(
                &set_to_the_unit_classes,
            ),
        ),
        (&records, &set_to_the_unit_classes),
    );
    for field in [
        "遍历走到而不在记录里的槽 [1]",
        "（I-3.18 判它）",
        "遍历走到的 2 槽、隔离豁免 1 槽、U 1 槽",
        "被改记的记录 [\"槽 0 跨 1 → 2\", \"槽 2 跨 1 → 2\"]",
        "只在改记那一份里的 [1, 3]",
        "只在记录自述那一份里的 []",
    ] {
        assert!(detail.contains(field), "说明里有「{field}」：{detail}");
    }
}

/// 已释放的记录、起点单元头用不了的记录不改记（I-3.18 不判它们）；改了的那一条只换跨度，别的字段照原样。
#[test]
fn only_unreleased_records_with_a_usable_unit_header_are_set_to_their_unit_classes_for_the_span_invariant(
) {
    let records_not_judged_by_the_span_invariant = [
        allocation_record_on_device_zero(10, 1, 7, true),
        allocation_record_on_device_zero(40, 1, 1, false),
    ];
    let classes = span_reading_unit_classes();
    assert_eq!(
        records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(
            &records_not_judged_by_the_span_invariant,
            |record| classes.get(&record.slot).copied(),
        ),
        None,
        "已释放的那条与头用不了的那条都不改：一条都没改，交回 None"
    );
    let records_with_one_slot_short_of_its_unit = [
        allocation_record_on_device_zero(10, 1, 1, false),
        allocation_record_on_device_zero(12, 1, 1, false),
    ];
    assert_eq!(
        records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(
            &records_with_one_slot_short_of_its_unit,
            |record| classes.get(&record.slot).copied(),
        ),
        Some(vec![
            allocation_record_on_device_zero(10, 2, 1, false),
            allocation_record_on_device_zero(12, 1, 1, false),
        ])
    );
}
