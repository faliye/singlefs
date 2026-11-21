use super::*;

/// 合成世界被判那一版的 checkpoint txg：两个快照之后又发布过几次。
const VERSION_TXG_OF_THE_SYNTHETIC_WORLD: u64 = 9;

/// 一把码 1 的中央映射 key：出生树 65536、出生 txg `birth_txg`、实例 1、事务号 `transaction`（birth 相同的几个单元靠它分开）。
fn data_unit_mapping_key(birth_txg: u64, transaction: u64) -> CentralMappingKey {
    let mut key = vec![0u8; mapping_key_bytes()];
    key[MAPPING_KEY_UNIT_CLASS_OFFSET] = MappingKeyUnitClass::DataUnit.tag();
    key[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TXG_OFFSET]
        .copy_from_slice(&65536u64.to_le_bytes());
    key[MAPPING_KEY_BIRTH_TXG_OFFSET..MAPPING_KEY_INSTANCE_OFFSET]
        .copy_from_slice(&birth_txg.to_le_bytes());
    key[MAPPING_KEY_INSTANCE_OFFSET..MAPPING_KEY_TAIL_OFFSET].copy_from_slice(&1u32.to_le_bytes());
    key[MAPPING_KEY_TAIL_OFFSET..].copy_from_slice(&transaction.to_le_bytes()[..6]);
    CentralMappingKey(key)
}

fn deadlist_entry_of_the_head(epoch: u64, unit: &CentralMappingKey) -> DeadlistEntryOfTheScan {
    DeadlistEntryOfTheScan {
        head: HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
        epoch,
        unit: unit.clone(),
    }
}

/// 合成世界里的四个单元。
struct UnitsOfTheSyntheticWorld {
    /// birth 3，活头、S1、S2 都引用。
    shared_by_the_head_and_both_snapshots: CentralMappingKey,
    /// birth 3，S1、S2 引用，S2 之后被杀（纪元 6）。
    killed_after_the_second_snapshot: CentralMappingKey,
    /// birth 5，只有 S2 引用，S2 之后被杀（纪元 6）。
    born_between_the_snapshots: CentralMappingKey,
    /// birth 2，谁都不引用。
    referenced_by_nobody: CentralMappingKey,
}

fn units_of_the_synthetic_world() -> UnitsOfTheSyntheticWorld {
    UnitsOfTheSyntheticWorld {
        shared_by_the_head_and_both_snapshots: data_unit_mapping_key(3, 1),
        killed_after_the_second_snapshot: data_unit_mapping_key(3, 2),
        born_between_the_snapshots: data_unit_mapping_key(5, 3),
        referenced_by_nobody: data_unit_mapping_key(2, 4),
    }
}

/// 两个活快照 S1（S.txg 4、前驱 0）、S2（S.txg 6、前驱 4），活头前驱 6；S2 之后被杀的两个单元各一条、纪元 6（进活头那一份 [6, ∞)）。合法。
fn world_with_two_live_snapshots(
    units: &UnitsOfTheSyntheticWorld,
) -> DeadlistAndSnapshotReachOfAVersion {
    DeadlistAndSnapshotReachOfAVersion {
        head_inode_tree: HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
        head_previous_snapshot_txg: SECOND_SNAPSHOT_TXG,
        units_of_the_head: BTreeSet::from([units.shared_by_the_head_and_both_snapshots.clone()]),
        live_snapshots: vec![
            LiveSnapshotReach {
                snapshot_inode_root_tree: FIRST_SNAPSHOT_TREE,
                snapshot_txg: FIRST_SNAPSHOT_TXG,
                previous_snapshot_txg: 0,
                units: BTreeSet::from([
                    units.shared_by_the_head_and_both_snapshots.clone(),
                    units.killed_after_the_second_snapshot.clone(),
                ]),
            },
            LiveSnapshotReach {
                snapshot_inode_root_tree: SECOND_SNAPSHOT_TREE,
                snapshot_txg: SECOND_SNAPSHOT_TXG,
                previous_snapshot_txg: FIRST_SNAPSHOT_TXG,
                units: BTreeSet::from([
                    units.shared_by_the_head_and_both_snapshots.clone(),
                    units.killed_after_the_second_snapshot.clone(),
                    units.born_between_the_snapshots.clone(),
                ]),
            },
        ],
        snapshots_being_destroyed: Vec::new(),
        deadlist_entries: vec![
            deadlist_entry_of_the_head(
                SECOND_SNAPSHOT_TXG,
                &units.killed_after_the_second_snapshot,
            ),
            deadlist_entry_of_the_head(SECOND_SNAPSHOT_TXG, &units.born_between_the_snapshots),
        ],
        checkpoint_txg_of_the_version: VERSION_TXG_OF_THE_SYNTHETIC_WORLD,
        keys_of_the_central_mapping: CentralMappingKeysOfAVersion::ReadInFull(BTreeSet::from([
            units.shared_by_the_head_and_both_snapshots.clone(),
            units.killed_after_the_second_snapshot.clone(),
            units.born_between_the_snapshots.clone(),
            units.referenced_by_nobody.clone(),
        ])),
    }
}

/// 同一个世界里 S2 写了销毁意图（意图与最后一批之间）：S2 带销毁中记号、不再是活快照，活头的前驱已传成 S2 的前驱 4；
/// birth 5 的那个单元只有 S2 引用、纪元 6 落在 [S2.txg, ∞) 里、birth 5 > P = 4——在待放集里，还没被放掉。合法。
fn world_between_the_destroy_intent_of_the_second_snapshot_and_its_last_batch(
    units: &UnitsOfTheSyntheticWorld,
) -> DeadlistAndSnapshotReachOfAVersion {
    let mut world = world_with_two_live_snapshots(units);
    world.live_snapshots.truncate(1);
    world.snapshots_being_destroyed = vec![SnapshotBeingDestroyedOfTheScan {
        snapshot_inode_root_tree: SECOND_SNAPSHOT_TREE,
        snapshot_txg: SECOND_SNAPSHOT_TXG,
        previous_snapshot_txg: FIRST_SNAPSHOT_TXG,
    }];
    world.head_previous_snapshot_txg = FIRST_SNAPSHOT_TXG;
    world
}

fn verdict_of_the_deadlist_judge(
    facts: &DeadlistAndSnapshotReachOfAVersion,
    invariant: &'static str,
) -> InvariantVerdict {
    let mut judgements = Judgements::default();
    judge_the_deadlist_against_the_head_and_the_live_snapshots(
        "合成的那一版",
        facts,
        &mut judgements,
    );
    judgements
        .into_report()
        .into_iter()
        .find(|(candidate, _)| *candidate == invariant)
        .expect("清单里有这一条")
        .1
}

fn assert_violated_mentioning(verdict: &InvariantVerdict, fragment: &str, what: &str) {
    match verdict {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(fragment),
            "{what}：违例说明里要有「{fragment}」，实际 {detail}"
        ),
        InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_) => {
            panic!("{what}：要判红，实际 {verdict:?}")
        }
    }
}

/// 两个活快照的合法 deadlist：I-3.5（③ ④ 与逆向）与 I-3.6 都判到且成立。
#[test]
fn legal_deadlist_under_two_live_snapshots_holds_the_reference_intervals_and_the_tight_bound() {
    let units = units_of_the_synthetic_world();
    let world = world_with_two_live_snapshots(&units);
    assert_eq!(
        verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "③ ④ 与逆向都成立"
    );
    assert_eq!(
        verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
        ),
        InvariantVerdict::Holds,
        "两条的 birth（3、5）都不大于活头那一侧的前驱 6"
    );
}

/// 必红世界（I-3.5 ④）：把一个活头还引用的块放进 deadlist。
#[test]
fn deadlist_entry_pinning_a_unit_the_head_still_references_violates_the_reference_intervals() {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    world.deadlist_entries.push(deadlist_entry_of_the_head(
        SECOND_SNAPSHOT_TXG,
        &units.shared_by_the_head_and_both_snapshots,
    ));
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        "活头还引用着",
        "活头引用的块进了 deadlist",
    );
}

/// 必红世界（I-3.5 逆向）：deadlist 里一条钉着谁都不引用、也不在待放集里的单元（不加逆向时这一类三条全绿）。
#[test]
fn deadlist_entry_pinning_a_unit_no_live_snapshot_references_violates_the_reverse_direction() {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    world.deadlist_entries.push(deadlist_entry_of_the_head(
        SECOND_SNAPSHOT_TXG,
        &units.referenced_by_nobody,
    ));
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        "逆向",
        "没有活快照钉着的单元进了 deadlist",
    );
}

/// 必红世界（I-3.5 ③）：只有活快照引用的单元，deadlist 里一条钉它的都没有（被直接释放了）。
#[test]
fn unit_only_a_live_snapshot_references_without_a_deadlist_entry_violates_the_reference_intervals()
{
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    world
        .deadlist_entries
        .retain(|entry| entry.unit != units.born_between_the_snapshots);
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        "（③）",
        "只有 S2 引用的单元不在 deadlist 里",
    );
}

/// 必红世界（I-3.5 ③ 归属）：只有 S2 引用的单元挂在纪元 4（S2 自己那一份 [4, 6)），而它该在 S2 之后那一侧 [6, ∞)。
#[test]
fn unit_only_a_live_snapshot_references_pinned_under_an_epoch_of_another_side_violates_the_reference_intervals(
) {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    for entry in &mut world.deadlist_entries {
        if entry.unit == units.born_between_the_snapshots {
            entry.epoch = FIRST_SNAPSHOT_TXG;
        }
    }
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        "（③）",
        "只有 S2 引用的单元纪元挂错一侧",
    );
}

/// 必红世界（I-3.6）：把该直接释放的块放进 deadlist——birth 7 晚于活头那一侧的前驱 6（最后一个快照之后才生的块，被杀时该直接释放）。
#[test]
fn deadlist_entry_born_after_the_previous_snapshot_of_its_side_violates_the_tight_bound() {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    let born_after_the_previous_snapshot = data_unit_mapping_key(SECOND_SNAPSHOT_TXG + 1, 5);
    world.deadlist_entries.push(deadlist_entry_of_the_head(
        SECOND_SNAPSHOT_TXG,
        &born_after_the_previous_snapshot,
    ));
    // 它的映射条目还在（只看紧界那一句，不连带 C300 那一格）。
    let CentralMappingKeysOfAVersion::ReadInFull(keys) = &mut world.keys_of_the_central_mapping
    else {
        panic!("合成世界的映射读全了");
    };
    keys.insert(born_after_the_previous_snapshot);
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
        ),
        "该直接释放的块进了 deadlist",
        "birth 7 的块进了活头那一份",
    );
}

/// 写销毁意图与最后一批之间的合法态（base-indep 深度 6 有 8023 个合法态 I-3.6 红，全在这一段）：待放条目 birth 5 > 活头新前驱 4，
/// I-3.6 豁免它、I-3.5 逆向认它在待放集里，两条都成立。去掉豁免（`crates/mutations.tsv` 里这一条的那一行）这里红。
#[test]
fn pending_entries_between_the_destroy_intent_and_the_last_batch_hold_the_tight_bound_and_the_reverse_direction(
) {
    let units = units_of_the_synthetic_world();
    let world = world_between_the_destroy_intent_of_the_second_snapshot_and_its_last_batch(&units);
    assert_eq!(
        verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
        ),
        InvariantVerdict::Holds,
        "待放条目豁免于紧界"
    );
    assert_eq!(
        verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "待放条目在待放集里，逆向成立；S1 引用的那一条归活头那一侧 [4, ∞)，③ 成立"
    );
}

/// 必红世界（I-3.6 待放集的上界）：意图与最后一批之间，
/// 待放的那一条纪元抬到被判这一版的 txg：下一侧是活头时上界取这一版的 txg，它不再算待放，birth 5 大于活头那一侧的前驱 4。上界取 ∞ 时它照算待放、
/// 两条都成立（空真那一形）。
#[test]
fn pending_entry_whose_epoch_is_raised_to_the_txg_of_the_version_violates_the_tight_bound() {
    let units = units_of_the_synthetic_world();
    let mut world =
        world_between_the_destroy_intent_of_the_second_snapshot_and_its_last_batch(&units);
    for entry in &mut world.deadlist_entries {
        if entry.unit == units.born_between_the_snapshots {
            entry.epoch = VERSION_TXG_OF_THE_SYNTHETIC_WORLD;
        }
    }
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
        ),
        "也不在未完成销毁意图的待放集里",
        "纪元抬到这一版 txg 的条目不算待放",
    );
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        "逆向",
        "纪元抬到这一版 txg 的条目没有活快照钉着、也不在待放集里",
    );
}

/// 合法态：先删掉的快照留下纪元 6 的条目（birth 3 不大于它的前驱 4，那一次没放），之后删 S1（lo = 4、P = 0），
/// 意图与最后一批之间这一条纪元 6 大于 lo、小于这一版的 txg、birth 3 > 0，在待放集里。上界收成 lo + 1 的写法这里红。
#[test]
fn pending_entry_whose_epoch_is_above_the_lower_bound_of_the_intent_holds_the_tight_bound_and_the_reverse_direction(
) {
    let units = units_of_the_synthetic_world();
    let world = DeadlistAndSnapshotReachOfAVersion {
        head_inode_tree: HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
        head_previous_snapshot_txg: 0,
        units_of_the_head: BTreeSet::from([units.shared_by_the_head_and_both_snapshots.clone()]),
        live_snapshots: Vec::new(),
        snapshots_being_destroyed: vec![SnapshotBeingDestroyedOfTheScan {
            snapshot_inode_root_tree: FIRST_SNAPSHOT_TREE,
            snapshot_txg: FIRST_SNAPSHOT_TXG,
            previous_snapshot_txg: 0,
        }],
        deadlist_entries: vec![deadlist_entry_of_the_head(
            SECOND_SNAPSHOT_TXG,
            &units.killed_after_the_second_snapshot,
        )],
        checkpoint_txg_of_the_version: VERSION_TXG_OF_THE_SYNTHETIC_WORLD,
        keys_of_the_central_mapping: CentralMappingKeysOfAVersion::ReadInFull(BTreeSet::from([
            units.shared_by_the_head_and_both_snapshots.clone(),
            units.killed_after_the_second_snapshot.clone(),
        ])),
    };
    assert_eq!(
        verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
        ),
        InvariantVerdict::Holds,
        "纪元 6 > lo 4 的待放条目豁免于紧界"
    );
    assert_eq!(
        verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "纪元 6 > lo 4 的待放条目在待放集里，逆向成立"
    );
}

/// 必红世界（I-3.6 每个纪元恰好落进一份区间，C145 第一条）：S1 的前驱改成 2，前驱链照样首尾相接（S2 的前驱 4 = S1.txg、活头的前驱 6 = S2.txg），
/// 一条纪元 1 的条目比最早那一侧的前驱小，落不进任何一侧。
#[test]
fn deadlist_entry_whose_epoch_falls_below_every_side_of_a_contiguous_chain_violates_the_tight_bound(
) {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    world.live_snapshots[0].previous_snapshot_txg = 2;
    for entry in &mut world.deadlist_entries {
        if entry.unit == units.killed_after_the_second_snapshot {
            entry.epoch = 1;
        }
    }
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
        ),
        "落不进任何一侧",
        "纪元 1 小于最早那一侧的前驱 2",
    );
}

/// 前驱链不首尾相接（活头的前驱 7 ≠ S2.txg 6）时落不进恰好一侧的条目报部分没判（归 I-9.25 判红；C701），不判红、不报不适用。
#[test]
fn deadlist_entry_outside_every_side_of_a_broken_chain_is_left_unjudged_by_the_tight_bound() {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    world.head_previous_snapshot_txg = SECOND_SNAPSHOT_TXG + 1;
    assert_eq!(
        verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
        ),
        InvariantVerdict::PartiallyJudged(format!(
            "合成的那一版：{DEADLIST_EPOCHS_NOT_SPLIT_BY_THE_PREVIOUS_SNAPSHOT_CHAIN}"
        )),
        "纪元 6 落在 S2 那一份 [4, 6) 与活头那一份 [7, ∞) 之间，链断了：射程里判不了，报部分没判"
    );
}

/// 必红世界（I-3.6 deadlist key 在同一版中央映射里查得到，C300 的 checker 一半）：只有 S2 引用的那个单元的映射条目从这一版的映射里没了。
#[test]
fn deadlist_entry_whose_mapping_key_is_missing_from_the_central_mapping_violates_the_tight_bound() {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    let CentralMappingKeysOfAVersion::ReadInFull(keys) = &mut world.keys_of_the_central_mapping
    else {
        panic!("合成世界的映射读全了");
    };
    assert!(
        keys.remove(&units.born_between_the_snapshots),
        "映射里本来有它"
    );
    assert_violated_mentioning(
        &verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
        ),
        "中央映射里查不到",
        "deadlist 钉着的单元映射条目没了",
    );
}

/// 这一版的中央映射读不全：deadlist key 查不查得到判不了，I-3.6 报部分没判、点名映射读不全、不判红（另两样都成立；C701（别的判法静默跳过
/// 读不出的部分））。
#[test]
fn central_mapping_not_read_in_full_leaves_the_mapping_lookup_of_the_deadlist_unjudged() {
    let units = units_of_the_synthetic_world();
    let mut world = world_with_two_live_snapshots(&units);
    world.keys_of_the_central_mapping = CentralMappingKeysOfAVersion::NotReadInFull;
    assert_eq!(
        verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
        ),
        InvariantVerdict::PartiallyJudged(
            CENTRAL_MAPPING_OF_THE_DEADLIST_VERSION_NOT_READ_IN_FULL.to_string()
        ),
        "映射读不全：那一格判不了，报部分没判"
    );
}

/// 一个快照都没有、deadlist 一条都没有：I-3.6 判一次、成立（空 deadlist），I-3.5 ③ ④ 没有对象、不判。
#[test]
fn an_empty_deadlist_without_snapshots_holds_the_tight_bound_and_leaves_the_reference_intervals_unjudged(
) {
    let world = DeadlistAndSnapshotReachOfAVersion {
        head_inode_tree: HEAD_INODE_TREE_OF_THE_SYNTHETIC_WORLD,
        head_previous_snapshot_txg: 0,
        units_of_the_head: BTreeSet::new(),
        live_snapshots: Vec::new(),
        snapshots_being_destroyed: Vec::new(),
        deadlist_entries: Vec::new(),
        checkpoint_txg_of_the_version: VERSION_TXG_OF_THE_SYNTHETIC_WORLD,
        keys_of_the_central_mapping: CentralMappingKeysOfAVersion::ReadInFull(BTreeSet::new()),
    };
    assert_eq!(
        verdict_of_the_deadlist_judge(
            &world,
            DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
        ),
        InvariantVerdict::Holds,
        "空 deadlist 判一次、成立"
    );
    assert!(
        matches!(
            verdict_of_the_deadlist_judge(&world, REFERENCE_INTERVALS_ARE_EXACT),
            InvariantVerdict::NotApplicable(_)
        ),
        "③ ④ 没有对象，I-3.5 在这里不判"
    );
}

/// 一版的快照集合：(码 14 树 ID, S.txg, 样子)。
fn snapshots_of_a_synthetic_version(
    checkpoint_txg: u64,
    snapshots: &[(u64, u64, SnapshotStanding)],
) -> SnapshotsOfAVersion {
    SnapshotsOfAVersion {
        name: format!("合成的那一版（txg {checkpoint_txg}）"),
        checkpoint_txg,
        snapshots: snapshots
            .iter()
            .map(|(tree, snapshot_txg, standing)| (*tree, (*snapshot_txg, *standing)))
            .collect(),
    }
}

fn verdict_of_the_consecutive_versions_judge(versions: &[SnapshotsOfAVersion]) -> InvariantVerdict {
    let mut judgements = Judgements::default();
    judge_live_snapshots_of_consecutive_versions(versions, &mut judgements);
    judgements
        .into_report()
        .into_iter()
        .find(|(candidate, _)| *candidate == LIVE_SNAPSHOTS_ARE_ENUMERABLE)
        .expect("I-3.7 在清单里")
        .1
}

/// 建 S1、建 S2、写 S2 的销毁意图、最后一批删掉 S2：相接各版之间的变化都照条款，I-3.7 成立。
#[test]
fn snapshot_sets_changing_by_creation_destroy_intent_and_last_batch_hold_the_enumerable_invariant()
{
    let live = SnapshotStanding::Live;
    let being_destroyed = SnapshotStanding::BeingDestroyed;
    let versions = [
        snapshots_of_a_synthetic_version(3, &[]),
        snapshots_of_a_synthetic_version(4, &[(FIRST_SNAPSHOT_TREE, 4, live)]),
        snapshots_of_a_synthetic_version(
            5,
            &[
                (FIRST_SNAPSHOT_TREE, 4, live),
                (SECOND_SNAPSHOT_TREE, 5, live),
            ],
        ),
        snapshots_of_a_synthetic_version(
            6,
            &[
                (FIRST_SNAPSHOT_TREE, 4, live),
                (SECOND_SNAPSHOT_TREE, 5, being_destroyed),
            ],
        ),
        snapshots_of_a_synthetic_version(7, &[(FIRST_SNAPSHOT_TREE, 4, live)]),
    ];
    assert_eq!(
        verdict_of_the_consecutive_versions_judge(&versions),
        InvariantVerdict::Holds
    );
}

/// 必红世界（I-3.7 少一条）：S2 在相接的后一版里没带销毁中记号就没了。
#[test]
fn live_snapshot_vanishing_between_consecutive_versions_violates_the_enumerable_invariant() {
    let live = SnapshotStanding::Live;
    let versions = [
        snapshots_of_a_synthetic_version(
            5,
            &[
                (FIRST_SNAPSHOT_TREE, 4, live),
                (SECOND_SNAPSHOT_TREE, 5, live),
            ],
        ),
        snapshots_of_a_synthetic_version(6, &[(FIRST_SNAPSHOT_TREE, 4, live)]),
    ];
    assert_violated_mentioning(
        &verdict_of_the_consecutive_versions_judge(&versions),
        "少一条",
        "S2 没经销毁意图就没了",
    );
}

/// 必红世界（I-3.7 多一条）：后一版多出一个活快照，诞生 txg 2 不是后一版的 txg 6。
#[test]
fn live_snapshot_appearing_with_another_txg_violates_the_enumerable_invariant() {
    let live = SnapshotStanding::Live;
    let versions = [
        snapshots_of_a_synthetic_version(5, &[(FIRST_SNAPSHOT_TREE, 4, live)]),
        snapshots_of_a_synthetic_version(
            6,
            &[
                (FIRST_SNAPSHOT_TREE, 4, live),
                (SECOND_SNAPSHOT_TREE, 2, live),
            ],
        ),
    ];
    assert_violated_mentioning(
        &verdict_of_the_consecutive_versions_judge(&versions),
        "多一条",
        "多出来的快照不是后一版建的",
    );
}

/// txg 不相接的两版之间（环上有洞）不比：少一条也不判红，一对都没比时这一格不判。
#[test]
fn versions_whose_txgs_are_not_consecutive_are_not_compared() {
    let live = SnapshotStanding::Live;
    let versions = [
        snapshots_of_a_synthetic_version(4, &[(FIRST_SNAPSHOT_TREE, 4, live)]),
        snapshots_of_a_synthetic_version(6, &[]),
    ];
    assert!(matches!(
        verdict_of_the_consecutive_versions_judge(&versions),
        InvariantVerdict::NotApplicable(_)
    ));
}

fn verdict_of_the_capture_judge(
    version_txg: u64,
    tree_table_entries: &[Vec<u8>],
) -> InvariantVerdict {
    let mut judgements = Judgements::default();
    judge_live_snapshots_are_captures_before_their_version(
        "合成的那一版",
        version_txg,
        &snapshot_entries_of_the_tree_table(tree_table_entries),
        &mut judgements,
    );
    judgements
        .into_report()
        .into_iter()
        .find(|(candidate, _)| *candidate == LIVE_SNAPSHOTS_ARE_ENUMERABLE)
        .expect("I-3.7 在清单里")
        .1
}

/// I-3.7 每一版那一格：S.txg 4、6 的两个活快照在 txg 6 那一版成立；同一张表在 txg 5 那一版红（S2 的诞生 txg 6 晚于这一版）。
#[test]
fn live_snapshot_whose_txg_is_after_its_version_violates_the_enumerable_invariant() {
    let entries = synthetic_tree_table_with_snapshots(&[
        (FIRST_SNAPSHOT_TREE, FIRST_SNAPSHOT_TXG, 0, 0),
        (
            SECOND_SNAPSHOT_TREE,
            SECOND_SNAPSHOT_TXG,
            FIRST_SNAPSHOT_TXG,
            0,
        ),
    ]);
    assert_eq!(
        verdict_of_the_capture_judge(SECOND_SNAPSHOT_TXG, &entries),
        InvariantVerdict::Holds
    );
    assert_violated_mentioning(
        &verdict_of_the_capture_judge(SECOND_SNAPSHOT_TXG - 1, &entries),
        "不是这一版之前的一次捕获",
        "S2 的诞生 txg 晚于这一版",
    );
}
