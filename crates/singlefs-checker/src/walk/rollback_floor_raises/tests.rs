use super::*;

/// 那一对历史在环长 12（R = 3、S = 4、区域落盘 (0, 1, 0)）上留下的根环，两边逐条相同：txg 1（实例 1、状态 A，txg 13 的根槽没落，
/// 洞里留着它）、txg 14（实例 2 第一条根、状态 E）、txg 15–24（实例 2、F 5、状态 E）；抬 F 的是 txg 15，F 从 0 抬到 5。
/// 交回 (txg, 实例, F, 状态标记)。
fn root_ring_left_by_the_pair_of_histories() -> Vec<(u64, u32, u64, u8)> {
    let mut roots = vec![(1u64, 1u32, 0u64, b'A'), (14, 2, 0, b'E')];
    roots.extend((15..=24u64).map(|checkpoint_txg| (checkpoint_txg, 2u32, 5u64, b'E')));
    roots
}

/// 那一对历史只差 txg 9、10、11 三次发布空不空：合法写者那边依次是 B、C、D（txg 12 是 E），抬 F 那一刻整环的上限是 9、抬到 5 合法；
/// 坏写者那边三次都是空发布（状态仍是 A），那一刻的上限是 1、抬到 5 越界。txg 2–8 都是空发布（状态 A）。交回 txg 3–12 的状态标记
/// （抬 F 之后被盖掉、要按 journal 认的那几版）。
fn journal_states_of_the_legal_history() -> Vec<(u64, u8)> {
    let mut states: Vec<(u64, u8)> = (3..=8u64).map(|txg| (txg, b'A')).collect();
    states.extend([(9, b'B'), (10, b'C'), (11, b'D'), (12, b'E')]);
    states
}

fn journal_states_of_the_bad_writer_history() -> Vec<(u64, u8)> {
    let mut states: Vec<(u64, u8)> = (3..=11u64).map(|txg| (txg, b'A')).collect();
    states.push((12, b'E'));
    states
}

/// 按镜像根环（[`root_ring_left_by_the_pair_of_histories`]）从抬 F 那条根（txg 15）看过去的每一槽。
fn ring_slots_of_the_pair_seen_from_the_raise(
    geometry: &PoolGeometry,
) -> BTreeMap<(u64, u64), RingSlotOfTheImageSeenFromTheRaise> {
    root_ring_left_by_the_pair_of_histories()
        .into_iter()
        .map(|(checkpoint_txg, instance, _, marker)| {
            let (region, slot) = root_ring_slot_the_txg_names(checkpoint_txg, geometry);
            let seen = if checkpoint_txg < 15 {
                RingSlotOfTheImageSeenFromTheRaise::HoldsARootOlderThanTheRaise(
                    RootBeforeTheRaise {
                        device: geometry.region_devices[usize::try_from(region).expect("区域号")],
                        checkpoint_txg,
                        instance,
                        user_visible_tree_root_pointers: Some(user_visible_state_marked(marker)),
                    },
                )
            } else {
                RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise
            };
            ((region, slot), seen)
        })
        .collect()
}

fn versions_of_instance_one_named_by_the_journal(
    states: &[(u64, u8)],
) -> BTreeMap<u64, Vec<VersionNamedByTheJournal>> {
    states
        .iter()
        .map(|(checkpoint_txg, marker)| {
            (
                *checkpoint_txg,
                vec![VersionNamedByTheJournal {
                    instance: 1,
                    user_visible_tree_root_pointers: Some(user_visible_state_marked(*marker)),
                }],
            )
        })
        .collect()
}

/// 抬 F 那条根（实例 2）的实例表：实例 1 的行 (1, 13, 0)——txg 13 的记录落了、根槽没落，恢复把它施加出来、新实例写行。
fn instance_table_rows_of_the_raise_in_the_pair() -> Vec<InstanceTableRow> {
    vec![InstanceTableRow {
        instance: 1,
        published_checkpoint_txg: 13,
        applied_transaction_high_water_mark: 0,
    }]
}

fn rollback_floor_of_the_pair_against_the_reconstructed_ring(
    journal_states: &[(u64, u8)],
    raised_floor: u64,
) -> RollbackFloorAgainstTheRingAtTheRaise {
    let geometry = geometry_of_a_ring_of_twelve();
    rollback_floor_against_the_ring_reconstructed_at_the_raise(
        &geometry,
        &ring_slots_of_the_pair_seen_from_the_raise(&geometry),
        &versions_of_instance_one_named_by_the_journal(journal_states),
        (15, 2),
        &instance_table_rows_of_the_raise_in_the_pair(),
        0,
        raised_floor,
    )
}

/// C717 合法那一边：按 journal 重建抬 F 那一刻的环（txg 1、3–12、14），非空有效根 1、9、10、11、12，第 4 新的是 9，盘 1 上最新的是 10，
/// 上限 9，F 5 成立。
#[test]
fn the_legal_history_of_the_pair_holds_against_the_ring_reconstructed_from_the_journal() {
    assert_eq!(
        rollback_floor_of_the_pair_against_the_reconstructed_ring(
            &journal_states_of_the_legal_history(),
            5
        ),
        RollbackFloorAgainstTheRingAtTheRaise::Holds,
        "合法写者：抬 F 那一刻的上限 9，F 5 不越过它"
    );
}

/// C717 坏写者那一边：根环与合法那一边逐条相同，journal 里 txg 9–11 是空发布：重建的环里非空有效根只有 1 与 12，不足 4 个，
/// 上限落到洞里留着的最旧有效根 X = txg 1，F 5 越过它，判红（锚在镜像最新根上的窗口只看得见 txg 14，判成立）。
#[test]
fn the_bad_writer_history_of_the_pair_violates_against_the_ring_reconstructed_from_the_journal() {
    let verdict = rollback_floor_of_the_pair_against_the_reconstructed_ring(
        &journal_states_of_the_bad_writer_history(),
        5,
    );
    let RollbackFloorAgainstTheRingAtTheRaise::Violated(detail) = verdict else {
        panic!("坏写者：重建的环上限是洞里的 txg 1，F 5 越过它要判红，实际 {verdict:?}");
    };
    assert!(
        detail.contains("上限 1") && detail.contains("最旧有效根 txg 1"),
        "说明写出上限落在洞里的 txg 1：{detail}"
    );
}

/// 判别力：同一张坏写者镜像把 F 放回 X（txg 1）以下（抬到 1），由红转绿。
#[test]
fn the_bad_writer_history_of_the_pair_holds_once_the_floor_goes_back_to_the_root_left_in_the_hole()
{
    assert_eq!(
        rollback_floor_of_the_pair_against_the_reconstructed_ring(
            &journal_states_of_the_bad_writer_history(),
            1
        ),
        RollbackFloorAgainstTheRingAtTheRaise::Holds,
        "F 不越过洞里的 txg 1"
    );
}

/// journal 罩不到：抬 F 那一刻住在区域 0 槽 1 的 txg 3 那一版已转出 journal 环，那一槽认不出，环重建不出，不报红。
#[test]
fn the_pair_whose_journal_no_longer_holds_an_overwritten_version_leaves_the_ring_not_reconstructed()
{
    let journal_without_txg_three: Vec<(u64, u8)> = journal_states_of_the_bad_writer_history()
        .into_iter()
        .filter(|(txg, _)| *txg != 3)
        .collect();
    let verdict =
        rollback_floor_of_the_pair_against_the_reconstructed_ring(&journal_without_txg_three, 5);
    let RollbackFloorAgainstTheRingAtTheRaise::RingNotReconstructed(stuck_at) = verdict else {
        panic!("journal 里没有 txg 3：重建不出，实际 {verdict:?}");
    };
    assert!(stuck_at.contains("txg 3"), "说明点名卡在 txg 3：{stuck_at}");
}

/// 那一对历史的环、抬 F 那条根（实例 2、txg 15）的实例表里实例 1 那一行的 T 改成 12（txg 12 是实例 1 的最后一版，u = T）：
/// 抬 F 那一刻区域 0 槽 0 住的可能是 txg 12 那一版（它的根槽可以没落）、也可能是落点式的上一圈——那一槽比 12 小的正 txg 没有，
/// 只剩 mkfs 种的第 0 代根（区域 0 的槽 0，不另算空着；抬之前的 F 是 0，第 0 代根算有效根，树表是 mkfs 种的 0 条那一版、两棵用户可见树都没有）。
/// 两种可能的环各判一次。
fn rollback_floor_of_the_pair_against_the_ring_whose_instance_one_ends_at_txg_twelve(
    journal_states: &[(u64, u8)],
    raised_floor: u64,
) -> RollbackFloorAgainstTheRingAtTheRaise {
    let geometry = geometry_of_a_ring_of_twelve();
    let rows_ending_instance_one_at_txg_twelve = vec![InstanceTableRow {
        instance: 1,
        published_checkpoint_txg: 12,
        applied_transaction_high_water_mark: 0,
    }];
    rollback_floor_against_the_ring_reconstructed_at_the_raise(
        &geometry,
        &ring_slots_of_the_pair_seen_from_the_raise(&geometry),
        &versions_of_instance_one_named_by_the_journal(journal_states),
        (15, 2),
        &rows_ending_instance_one_at_txg_twelve,
        0,
        raised_floor,
    )
}

/// u = T 那一槽两种住户都算、结论一致时照判：
/// 合法写者的历史上两种可能的环——那一槽住 txg 12（状态 E）、住第 0 代根——上限都是 9（txg 12 那一种非空有效根 1、9、10、11、12，
/// 第 0 代根那一种 1、9、10、11、14，第 4 新的都是 9；盘 1 上最新的是 10），F 5 成立。
#[test]
fn an_overwritten_version_at_the_row_of_its_instance_holds_when_every_possible_ring_bounds_the_floor(
) {
    assert_eq!(
        rollback_floor_of_the_pair_against_the_ring_whose_instance_one_ends_at_txg_twelve(
            &journal_states_of_the_legal_history(),
            5
        ),
        RollbackFloorAgainstTheRingAtTheRaise::Holds,
        "两种可能的环上限都是 9，F 5 成立"
    );
}

/// 同一个 u = T 那一形、坏写者的历史（txg 9–11 是空发布）：txg 12 那一种上限 1（非空有效根只有 1 与 12），
/// 第 0 代根那一种上限 0（最旧有效根 txg 0），F 5 在两种上都越过上限，判红。
#[test]
fn an_overwritten_version_at_the_row_of_its_instance_violates_when_every_possible_ring_falls_below_the_floor(
) {
    let verdict = rollback_floor_of_the_pair_against_the_ring_whose_instance_one_ends_at_txg_twelve(
        &journal_states_of_the_bad_writer_history(),
        5,
    );
    let RollbackFloorAgainstTheRingAtTheRaise::Violated(detail) = verdict else {
        panic!("两种可能的环 F 5 都越过上限：判红，实际 {verdict:?}");
    };
    assert!(
        detail.contains("可能的环 2 种都判红"),
        "说明写出两种可能的环都判红：{detail}"
    );
}

/// 同一个 u = T 那一形、坏写者的历史、F 抬到 1：txg 12 那一种上限 1、F 1 成立，第 0 代根那一种上限 0、F 1 越过它，
/// 结论不一致，报可能的环判得不一致，不报成立也不报红。
#[test]
fn an_overwritten_version_at_the_row_of_its_instance_is_left_unjudged_when_the_possible_rings_disagree(
) {
    let verdict = rollback_floor_of_the_pair_against_the_ring_whose_instance_one_ends_at_txg_twelve(
        &journal_states_of_the_bad_writer_history(),
        1,
    );
    let RollbackFloorAgainstTheRingAtTheRaise::PossibleRingsDisagree(possible_rings) = verdict
    else {
        panic!("两种可能的环一种成立、一种红：判得不一致，实际 {verdict:?}");
    };
    assert!(
        possible_rings.contains("可能的环 2 种各判一次：成立 1、F 高于上限 1"),
        "说明写出几种、各判成什么：{possible_rings}"
    );
}

/// 第 0 代根那一形：环长 12，txg 1–11（实例 1，状态 1–6 是 A，7 是 B，8 是 C，9 是 D，10、11 是 E）各住各的槽，抬 F 的是 txg 12
/// （实例 1、抬之前的 F 0），落在区域 0 槽 0、盖掉了那一槽。那一槽比 12 小的正 txg 没有：抬 F 那一刻它住的是 mkfs 种的第 0 代根
/// （mkfs 在每个区域的槽 0 都种了它，不另算空着；树表是 mkfs 种的 0 条那一版，两棵用户可见树都没有，判决
/// `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q3 第 3 问「定」）。非空有效根 1、7、8、9、10，第 4 新的是 7；
/// 盘 0 上最新的 11、盘 1 上最新的 10，上限 7，F 7 成立。
#[test]
fn raise_over_the_slot_of_the_genesis_root_holds_when_the_genesis_root_as_the_only_root_of_the_slot_leaves_the_ceiling_at_the_fourth_newest_non_empty_root(
) {
    let geometry = geometry_of_a_ring_of_twelve();
    let state_of = |checkpoint_txg: u64| match checkpoint_txg {
        7 => b'B',
        8 => b'C',
        9 => b'D',
        10 | 11 => b'E',
        _ => b'A',
    };
    let mut ring_slots: BTreeMap<(u64, u64), RingSlotOfTheImageSeenFromTheRaise> = (1..=11u64)
        .map(|checkpoint_txg| {
            let (region, slot) = root_ring_slot_the_txg_names(checkpoint_txg, &geometry);
            (
                (region, slot),
                RingSlotOfTheImageSeenFromTheRaise::HoldsARootOlderThanTheRaise(
                    RootBeforeTheRaise {
                        device: geometry.region_devices[usize::try_from(region).expect("区域号")],
                        checkpoint_txg,
                        instance: 1,
                        user_visible_tree_root_pointers: Some(user_visible_state_marked(state_of(
                            checkpoint_txg,
                        ))),
                    },
                ),
            )
        })
        .collect();
    assert_eq!(
        ring_slots.insert(
            (0, 0),
            RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise
        ),
        None,
        "txg 1–11 一条都不住区域 0 槽 0"
    );
    assert_eq!(
        rollback_floor_against_the_ring_reconstructed_at_the_raise(
            &geometry,
            &ring_slots,
            &BTreeMap::new(),
            (12, 1),
            &[],
            0,
            7,
        ),
        RollbackFloorAgainstTheRingAtTheRaise::Holds,
        "第 0 代根那一种上限 7，F 7 成立"
    );
}

/// 环长 12 上 txg 1–11（实例 1，状态由 `state_of` 给）各住各的槽、抬 F 的是 txg 12（实例 1、抬之前的 F 0、落在区域 0 槽 0 盖掉了那一槽）时，
/// 镜像上从抬 F 那条根看过去的每一槽（区域 0 槽 0 是被盖掉的那一槽）。
/// 下面两张盘面都在这一份环上：只差 txg 1–11 的状态与 F 抬到几。
fn ring_of_twelve_raised_over_the_slot_of_the_genesis_root_at_txg_twelve(
    state_of: impl Fn(u64) -> u8,
) -> BTreeMap<(u64, u64), RingSlotOfTheImageSeenFromTheRaise> {
    let geometry = geometry_of_a_ring_of_twelve();
    let mut ring_slots: BTreeMap<(u64, u64), RingSlotOfTheImageSeenFromTheRaise> = (1..=11u64)
        .map(|checkpoint_txg| {
            let (region, slot) = root_ring_slot_the_txg_names(checkpoint_txg, &geometry);
            (
                (region, slot),
                RingSlotOfTheImageSeenFromTheRaise::HoldsARootOlderThanTheRaise(
                    RootBeforeTheRaise {
                        device: geometry.region_devices[usize::try_from(region).expect("区域号")],
                        checkpoint_txg,
                        instance: 1,
                        user_visible_tree_root_pointers: if state_of(checkpoint_txg) == 0 {
                            Some(UserVisibleTreeRootPointers::ABSENT)
                        } else {
                            Some(user_visible_state_marked(state_of(checkpoint_txg)))
                        },
                    },
                ),
            )
        })
        .collect();
    assert_eq!(
        ring_slots.insert(
            (0, 0),
            RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise
        ),
        None,
        "txg 1–11 一条都不住区域 0 槽 0"
    );
    ring_slots
}

/// 按 [`ring_of_twelve_raised_over_the_slot_of_the_genesis_root_at_txg_twelve`] 那一份环判 F 抬到 `raised_floor`（journal 里一版都不用认：
/// 被盖掉的只有区域 0 槽 0，那一槽比 12 小的正 txg 没有）。
fn rollback_floor_raised_over_the_slot_of_the_genesis_root_at_txg_twelve(
    state_of: impl Fn(u64) -> u8,
    raised_floor: u64,
) -> RollbackFloorAgainstTheRingAtTheRaise {
    rollback_floor_against_the_ring_reconstructed_at_the_raise(
        &geometry_of_a_ring_of_twelve(),
        &ring_of_twelve_raised_over_the_slot_of_the_genesis_root_at_txg_twelve(state_of),
        &BTreeMap::new(),
        (12, 1),
        &[],
        0,
        raised_floor,
    )
}

/// 被盖掉的槽 0、抬之前的 F 是 0：那一槽只交回第 0 代根一种，树表按 mkfs 种的 0 条那一版记成两棵用户可见树都没有；
/// 不是槽 0 的被盖掉的槽认完了交回空着一种；抬之前的 F 高于 0 时槽 0 也交回空着一种（第 0 代根不是有效根）。
#[test]
fn an_overwritten_slot_zero_recognized_down_to_the_end_holds_only_the_genesis_root_with_both_user_visible_trees_absent(
) {
    let geometry = geometry_of_a_ring_of_twelve();
    let recognized = |(region, slot): (u64, u64), floor_before_the_raise: u64| {
        possible_roots_in_the_ring_slot_at_the_raise(
            &geometry,
            (region, slot),
            &RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise,
            &BTreeMap::new(),
            (2, 1),
            &[],
            floor_before_the_raise,
        )
    };
    assert_eq!(
        recognized((0, 0), 0),
        Ok(vec![Some(RootBeforeTheRaise {
            device: 0,
            checkpoint_txg: 0,
            instance: 0,
            user_visible_tree_root_pointers: Some(UserVisibleTreeRootPointers::ABSENT),
        })]),
        "区域 0 槽 0 比 2 小的正 txg 没有：只有第 0 代根一种，不另算空着"
    );
    assert_eq!(
        recognized((2, 1), 0),
        Ok(vec![None]),
        "区域 2 槽 1 比 2 小的正 txg 没有，不是槽 0：空着"
    );
    assert_eq!(
        recognized((2, 0), 1),
        Ok(vec![None]),
        "抬之前的 F 1：区域 2 槽 0 的第 0 代根 txg 0 低于它，对上限与空着一样"
    );
}

/// 坏写者那张：txg 1、2 是暖机（两棵用户可见树都没有），
/// txg 3–11 是同一个带文件的状态 A。抬 F 那一刻区域 0 槽 0 住着第 0 代根：有效根 0–11，非空的只有 3，不足 4 个，上限取最旧有效根 txg 0，
/// F 抬到 1 越过它，判红。
#[test]
fn floor_raised_over_the_slot_of_the_genesis_root_past_the_oldest_valid_root_reddens_the_rollback_floor_invariant(
) {
    let verdict = rollback_floor_raised_over_the_slot_of_the_genesis_root_at_txg_twelve(
        |checkpoint_txg| if checkpoint_txg <= 2 { 0 } else { b'A' },
        1,
    );
    let RollbackFloorAgainstTheRingAtTheRaise::Violated(detail) = verdict else {
        panic!("第 0 代根那一种上限 0，F 1 越过它：判红，实际 {verdict:?}");
    };
    assert!(
        detail.contains("上限 0") && detail.contains("最旧有效根 txg 0"),
        "说明写出上限落在第 0 代根的 txg 0：{detail}"
    );
}

/// 合法写者那张：txg 1–5 是状态 A、6 是 B、7 是 C、
/// 8–11 是 D。抬 F 那一刻区域 0 槽 0 住着第 0 代根（两棵用户可见树都没有）：非空有效根 1、6、7、8，第 4 新的是 1；盘 1 上最新的 10，
/// 上限 1，F 抬到 1 成立。
#[test]
fn floor_raised_over_the_slot_of_the_genesis_root_to_the_fourth_newest_non_empty_root_holds_the_rollback_floor_invariant(
) {
    assert_eq!(
        rollback_floor_raised_over_the_slot_of_the_genesis_root_at_txg_twelve(
            |checkpoint_txg| match checkpoint_txg {
                6 => b'B',
                7 => b'C',
                8..=11 => b'D',
                _ => b'A',
            },
            1,
        ),
        RollbackFloorAgainstTheRingAtTheRaise::Holds,
        "第 0 代根那一种非空有效根 1、6、7、8，上限 1，F 1 成立"
    );
}

/// 往下逐层认到认出为止：环长 12，抬 F 的是实例 4 的 txg 30，区域 2 槽 0 在抬 F 之后被盖。落点式住进那一槽、比 30 小的是 26、14、2：
/// 26 是实例 3 的 T、14 是实例 2 的 T（根槽都可以没落），2 是实例 1 中间那一版（2 < 实例 1 那一行的 T 5，根槽落了盘），认到 2 为止。
#[test]
fn an_overwritten_slot_is_recognized_layer_by_layer_down_to_the_version_whose_root_landed() {
    let geometry = geometry_of_a_ring_of_twelve();
    let version_of = |instance: u32, marker: u8| {
        vec![VersionNamedByTheJournal {
            instance,
            user_visible_tree_root_pointers: Some(user_visible_state_marked(marker)),
        }]
    };
    let versions_named_by_the_journal = BTreeMap::from([
        (2, version_of(1, b'A')),
        (14, version_of(2, b'B')),
        (26, version_of(3, b'C')),
    ]);
    let rows = [(1u32, 5u64), (2, 14), (3, 26)].map(|(instance, published)| InstanceTableRow {
        instance,
        published_checkpoint_txg: published,
        applied_transaction_high_water_mark: 0,
    });
    let possible_root = |checkpoint_txg: u64, instance: u32, marker: u8| {
        Some(RootBeforeTheRaise {
            device: 0,
            checkpoint_txg,
            instance,
            user_visible_tree_root_pointers: Some(user_visible_state_marked(marker)),
        })
    };
    assert_eq!(
        possible_roots_in_the_ring_slot_at_the_raise(
            &geometry,
            (2, 0),
            &RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise,
            &versions_named_by_the_journal,
            (30, 4),
            &rows,
            0,
        ),
        Ok(vec![
            possible_root(26, 3, b'C'),
            possible_root(14, 2, b'B'),
            possible_root(2, 1, b'A'),
        ]),
        "26、14 两层都可能没落，认到根槽落了盘的 2 为止"
    );
}

/// 往下认到窗口外为止：同一槽、实例 3 那一行的 T 是 26（u = T），抬之前的 F 是 20。上一圈 14 低于抬之前的 F：住它或更早的哪一条都不是有效根，
/// 记一种「空着或无效」、不再往下认（journal 里没有 14 那一版也不卡）。
#[test]
fn an_overwritten_slot_stops_recognizing_once_the_layer_lies_below_the_floor_before_the_raise() {
    let geometry = geometry_of_a_ring_of_twelve();
    let versions_named_by_the_journal = BTreeMap::from([(
        26,
        vec![VersionNamedByTheJournal {
            instance: 3,
            user_visible_tree_root_pointers: Some(user_visible_state_marked(b'C')),
        }],
    )]);
    let rows = [InstanceTableRow {
        instance: 3,
        published_checkpoint_txg: 26,
        applied_transaction_high_water_mark: 0,
    }];
    assert_eq!(
        possible_roots_in_the_ring_slot_at_the_raise(
            &geometry,
            (2, 0),
            &RingSlotOfTheImageSeenFromTheRaise::OverwrittenSinceTheRaise,
            &versions_named_by_the_journal,
            (30, 4),
            &rows,
            20,
        ),
        Ok(vec![
            Some(RootBeforeTheRaise {
                device: 0,
                checkpoint_txg: 26,
                instance: 3,
                user_visible_tree_root_pointers: Some(user_visible_state_marked(b'C')),
            }),
            None,
        ]),
        "26 可能没落；14 低于抬之前的 F 20，那一层与再往下都不是有效根"
    );
}

/// 窗口里的有效根（盘、txg、状态标记），按 txg 升序；实例都是 1。
fn window_ceiling_of_the_roots(roots: &[(u32, u64, u8)]) -> RollbackFloorCeilingBeforeTheRaise {
    let valid_roots_oldest_first: Vec<RootBeforeTheRaise> = roots
        .iter()
        .map(|(device, checkpoint_txg, marker)| RootBeforeTheRaise {
            device: *device,
            checkpoint_txg: *checkpoint_txg,
            instance: 1,
            user_visible_tree_root_pointers: Some(user_visible_state_marked(*marker)),
        })
        .collect();
    rollback_floor_ceiling_of_the_valid_roots(&valid_roots_oldest_first).expect("至少一条有效根")
}

/// 窗口定不定得住抬 F 那一刻的上限（不定就走丁）：两块盘（环长 12 的几何，区域落盘 (0, 1, 0)）上都有窗口里的有效根、
/// 去掉最旧那条之后非空有效根去重仍有 4 条，才定得住；少一块盘、或只剩 3 条（最旧那条跟「两棵都没有」比出的非空不算）都定不住。
#[test]
fn the_window_bounds_the_ceiling_at_the_raise_only_with_every_device_and_four_newer_non_empty_roots(
) {
    let geometry = geometry_of_a_ring_of_twelve();
    let on_both_devices_with_four_newer_states = window_ceiling_of_the_roots(&[
        (0, 10, b'A'),
        (1, 11, b'B'),
        (0, 12, b'C'),
        (1, 13, b'D'),
        (0, 14, b'E'),
    ]);
    assert!(
        window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise(
            &on_both_devices_with_four_newer_states,
            &geometry
        ),
        "两块盘都有、最旧那条之后有 B、C、D、E 四个状态：定得住"
    );
    let on_both_devices_with_three_newer_states =
        window_ceiling_of_the_roots(&[(0, 10, b'A'), (1, 11, b'B'), (0, 12, b'C'), (1, 13, b'D')]);
    assert!(
        !window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise(
            &on_both_devices_with_three_newer_states,
            &geometry
        ),
        "最旧那条之后只有 B、C、D 三个状态：最旧那条 A 的非空不作数，定不住"
    );
    let on_device_zero_only = window_ceiling_of_the_roots(&[
        (0, 10, b'A'),
        (0, 11, b'B'),
        (0, 12, b'C'),
        (0, 13, b'D'),
        (0, 14, b'E'),
    ]);
    assert!(
        !window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise(
            &on_device_zero_only,
            &geometry
        ),
        "区域 1 所在的盘 1 上窗口里一条有效根都没有：定不住"
    );
}

/// 落点式：区域 0 槽 1 住 3、15、27……，比 15 小的最大正 txg 是 3；区域 0 槽 0 住 12、24……（0 是第 0 代根），比 12 小的没有正 txg。
#[test]
fn the_largest_positive_txg_a_ring_slot_names_below_the_raise_follows_the_slot_formula() {
    let geometry = geometry_of_a_ring_of_twelve();
    assert_eq!(
        largest_positive_txg_the_ring_slot_names_below(0, 1, 15, &geometry),
        Some(3)
    );
    assert_eq!(
        largest_positive_txg_the_ring_slot_names_below(0, 0, 15, &geometry),
        Some(12)
    );
    assert_eq!(
        largest_positive_txg_the_ring_slot_names_below(0, 0, 12, &geometry),
        None,
        "区域 0 槽 0 比 12 小的只有第 0 代根"
    );
    assert_eq!(
        largest_positive_txg_the_ring_slot_names_below(2, 3, 11, &geometry),
        None,
        "区域 2 槽 3 最小的正 txg 是 11，不比 11 小"
    );
    assert_eq!(
        largest_positive_txg_the_ring_slot_names_below(2, 3, 12, &geometry),
        Some(11)
    );
}

/// C961（全零根槽没并进 checker 的抬 F 上限）的 `first_example`（
/// S = 4、hole_txg 3、raise_at 8）那一圈环：区域 0 槽 1（落点式住 3）读回全零，区域 2 槽 3（住 11）也全零（还没轮到），别的槽坐着根。
/// 抬 F 那条根 txg 9、抬之前的 F 0、抬 F 那个实例（实例 2）在环里最旧的根 txg 4：区域 0 槽 1 能住的最大 txg 3 并进上限；
/// 区域 2 槽 3 比 9 小的正 txg 没有、不定上限。抬之前的 F 已是 4 时 3 低于它（写者只并不低于今天的 F 的槽），一槽都不定上限；
/// 那个实例在环里最旧的根是 3 时 3 不比它小，同样不定。读不出、自证过的槽不在这里。
#[test]
fn ring_slot_reading_back_all_zero_holds_the_floor_raise_at_the_largest_txg_it_can_hold() {
    let geometry = geometry_of_a_ring_of_twelve();
    let readings: BTreeMap<(u64, u64), RootRingSlotReading> = (0..3u64)
        .flat_map(|region| (0..4u64).map(move |slot| (region, slot)))
        .map(|(region, slot)| {
            let reading = match (region, slot) {
                (0, 1) | (2, 3) => RootRingSlotReading::AllZero,
                (1, 1) => RootRingSlotReading::DoesNotReadBack,
                _ => RootRingSlotReading::SelfVerified {
                    checkpoint_txg: region + 3 * slot,
                },
            };
            ((region, slot), reading)
        })
        .collect();
    assert_eq!(
            rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero(
                &geometry, &readings, 9, 4, 0
            ),
            Some((3, (0, 1))),
            "区域 0 槽 1 能住的最大 txg 3（比抬 F 那个实例在环里最旧的根 4 小）定上限；读不出的区域 1 槽 1 不在这里"
        );
    assert_eq!(
        rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero(
            &geometry, &readings, 9, 4, 4
        ),
        None,
        "能住的最大 txg 3 低于抬之前的 F 4：不定上限"
    );
    assert_eq!(
        rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero(
            &geometry, &readings, 9, 3, 0
        ),
        None,
        "能住的最大 txg 3 不比抬 F 那个实例在环里最旧的根 3 小（那个进程该写过这一槽）：不定上限"
    );
}

/// 一片实例表（码 3 打包记录类型 4、片序号 0）：给的几行，末尾一条「没有下一片」的链指针记录；头校验和与载荷校验和都封好。
fn instance_table_page_holding_the_rows(rows: &[InstanceTableRow]) -> Vec<u8> {
    let record_count = rows.len() + 1;
    let mut unit = vec![0u8; data_unit_bytes()];
    unit[..4].copy_from_slice(b"SFSU");
    unit[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        .copy_from_slice(&UNIT_FORMAT_VERSION_THIS_CHECKER_READS.to_le_bytes());
    unit[6] = 3;
    unit[8..10].copy_from_slice(
        &u16::try_from(record_count * INSTANCE_TABLE_RECORD_BYTES)
            .expect("装得进一片")
            .to_le_bytes(),
    );
    unit[42] = 3;
    unit[51..53].copy_from_slice(&PACKED_TYPE_INSTANCE_TABLE.to_le_bytes());
    unit[69..71].copy_from_slice(&u16::try_from(record_count).expect("条数").to_le_bytes());
    unit[71..73].copy_from_slice(
        &u16::try_from(INSTANCE_TABLE_RECORD_BYTES)
            .expect("88")
            .to_le_bytes(),
    );
    for (index, row) in rows.iter().enumerate() {
        let start = 136 + index * INSTANCE_TABLE_RECORD_BYTES;
        unit[start] = 0;
        unit[start + 1..start + 5].copy_from_slice(&row.instance.to_le_bytes());
        unit[start + 5..start + 13].copy_from_slice(&row.published_checkpoint_txg.to_le_bytes());
        unit[start + 13..start + 21]
            .copy_from_slice(&row.applied_transaction_high_water_mark.to_le_bytes());
        unit[start + INSTANCE_TABLE_ROW_FLAGS_OFFSET] =
            INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION;
    }
    unit[136 + rows.len() * INSTANCE_TABLE_RECORD_BYTES] = 1;
    let payload_checksum = crc32_castagnoli_table(&unit[107..]);
    unit[89..93].copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum = crc32_castagnoli_table(&unit[..107]);
    unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
    unit
}

/// 单元区槽号：实例表那一片住槽 1，状态 A–E 的树表各住槽 2–6。
const SLOT_OF_THE_INSTANCE_TABLE_OF_THE_PAIR: u64 = 1;

fn slot_of_the_tree_table_of_the_state(marker: u8) -> u64 {
    2 + u64::from(marker - b'A')
}

/// 那一对历史的镜像（只放 I-7.9 读的东西）：根环 12 条根的记录字节（树表指针指着它那个状态的树表，抬 F 的 txg 15 另带实例表指针）、
/// 根环槽读数、重放读的那一份环（txg 3–12 各一条带提交标记、带末条标志的记录，新根段的树表指针指着那一版的树表），交回 I-7.9 的判定。
fn rollback_floor_verdict_of_the_image_of_the_pair(
    journal_states: &[(u64, u8)],
) -> InvariantVerdict {
    let geometry = geometry_of_a_ring_of_twelve();
    let instance_table_page =
        instance_table_page_holding_the_rows(&instance_table_rows_of_the_raise_in_the_pair());
    let tree_tables: Vec<(u64, Vec<u8>)> = (b'A'..=b'E')
        .map(|marker| {
            (
                slot_of_the_tree_table_of_the_state(marker),
                tree_table_node_with_both_user_visible_tree_roots_marked(marker),
            )
        })
        .collect();
    let mut units_by_slot: Vec<(u64, &[u8])> = vec![(
        SLOT_OF_THE_INSTANCE_TABLE_OF_THE_PAIR,
        instance_table_page.as_slice(),
    )];
    units_by_slot.extend(
        tree_tables
            .iter()
            .map(|(slot, unit)| (*slot, unit.as_slice())),
    );
    let image = ImageHoldingUnitsAtSlots::of(&units_by_slot);
    let tree_table_pointer_of_the_state = |marker: u8| {
        image.node_pointer_to_slot(slot_of_the_tree_table_of_the_state(marker), node_bytes(), 0)
    };
    let instance_table_pointer =
        image.node_pointer_to_slot(SLOT_OF_THE_INSTANCE_TABLE_OF_THE_PAIR, data_unit_bytes(), 0);
    let roots: Vec<(u64, u64, crate::RootView)> = root_ring_left_by_the_pair_of_histories()
        .into_iter()
        .map(|(checkpoint_txg, instance, rollback_floor, marker)| {
            let (region, slot) = root_ring_slot_the_txg_names(checkpoint_txg, &geometry);
            let mut root = ring_root_of(
                instance,
                checkpoint_txg,
                rollback_floor,
                0,
                ROOT_RECORD_NO_RING_SLOT_PINNED,
            );
            root.record_bytes = vec![0u8; 466];
            root.record_bytes[36..122].copy_from_slice(&tree_table_pointer_of_the_state(marker));
            root.record_bytes[170..256].copy_from_slice(&instance_table_pointer);
            (region, slot, root)
        })
        .collect();
    let root_ring_slot_readings: BTreeMap<(u64, u64), RootRingSlotReading> = roots
        .iter()
        .map(|(region, slot, root)| {
            (
                (*region, *slot),
                RootRingSlotReading::SelfVerified {
                    checkpoint_txg: root.checkpoint_txg,
                },
            )
        })
        .collect();
    let scanned_records: Vec<ScannedJournalRecord> = journal_states
        .iter()
        .map(|(checkpoint_txg, marker)| {
            let mut record = scanned_record_of(1, 0, 0);
            record.checkpoint_txg = *checkpoint_txg;
            record.new_root_segment = vec![0u8; NEW_ROOT_SEGMENT_PINNED_RING_SLOT.end];
            record.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER]
                .copy_from_slice(&tree_table_pointer_of_the_state(*marker));
            record
        })
        .collect();
    let records_the_replay_reads: BTreeMap<(u32, u64), &ScannedJournalRecord> = scanned_records
        .iter()
        .map(|record| ((record.instance, record.checkpoint_txg), record))
        .collect();
    let mut judgements = Judgements::default();
    judge_rollback_floor_raises_against_their_ceilings(
        &image,
        &geometry,
        &roots,
        OneRevolutionWindow {
            anchor_txg: 24,
            root_ring_slot_count: 12,
        },
        &root_ring_slot_readings,
        &records_the_replay_reads,
        &mut IndexNodeCache::new(),
        &mut judgements,
    );
    verdict_of(judgements, "I-7.9")
}

/// C717 在池级 checker 的判法上：那一对根环逐条相同的镜像，锚在镜像最新根上的窗口只看得见 txg 14、两张都判成立；
/// 按 journal 重建抬 F 那一刻的环之后，合法写者那张成立、坏写者那张红。
#[test]
fn the_pool_checker_reddens_the_bad_writer_image_of_the_pair_and_holds_the_legal_one() {
    assert_eq!(
        rollback_floor_verdict_of_the_image_of_the_pair(&journal_states_of_the_legal_history()),
        InvariantVerdict::Holds,
        "合法写者那张：抬 F 那一刻的上限 9，F 5 成立"
    );
    let verdict_of_the_bad_writer_image = rollback_floor_verdict_of_the_image_of_the_pair(
        &journal_states_of_the_bad_writer_history(),
    );
    assert!(
        matches!(&verdict_of_the_bad_writer_image, InvariantVerdict::Violated(detail) if detail.contains("按 journal 重建")),
        "坏写者那张：按 journal 重建的环上限 1，F 5 判红，实际 {verdict_of_the_bad_writer_image:?}"
    );
}

/// journal 罩不到（txg 3–12 一条都不在）时池级 checker 报部分没判，不报成立、不报红。
#[test]
fn the_pool_checker_leaves_the_pair_partially_judged_when_the_journal_no_longer_holds_the_overwritten_versions(
) {
    let verdict = rollback_floor_verdict_of_the_image_of_the_pair(&[]);
    assert!(
        matches!(&verdict, InvariantVerdict::PartiallyJudged(unjudged) if unjudged.contains("journal 罩不到")),
        "journal 里没有被盖掉的那几版：部分没判，实际 {verdict:?}"
    );
}

/// D16（发布语义） 已定项 1 的「非空」与去重：两棵用户树的根不变、只多出一对快照条目的那一版，
/// 与前一版是两个不同的状态，数「第 N 新的非空有效根」时各计一个；只比两棵树的读法把它们并成一个（`crates/mutations.tsv` 里这一条的那一行）。
#[test]
fn root_that_only_created_a_snapshot_counts_as_one_more_distinct_state() {
    let before = synthetic_tree_table_with_snapshots(&[]);
    let after_creating_a_snapshot =
        synthetic_tree_table_with_snapshots(&[(FIRST_SNAPSHOT_TREE, FIRST_SNAPSHOT_TXG, 0, 0)]);
    let mut after_with_the_head_entries_unchanged = before.clone();
    after_with_the_head_entries_unchanged.extend(after_creating_a_snapshot[2..].iter().cloned());
    let state_before = user_visible_tree_root_pointers_of_the_tree_table_entries(&before)
        .expect("两棵用户树各一条、条目都是字段表宽");
    let state_after = user_visible_tree_root_pointers_of_the_tree_table_entries(
        &after_with_the_head_entries_unchanged,
    )
    .expect("两棵用户树各一条、条目都是字段表宽");
    assert_eq!(
        (&state_before.inode_tree, &state_before.extent_tree),
        (&state_after.inode_tree, &state_after.extent_tree),
        "两棵用户树的根指针没动"
    );
    assert_ne!(state_before, state_after, "多了一对快照树根，算一个新状态");
    assert_eq!(
        newest_first_distinct_state_txgs(
            &[
                (FIRST_SNAPSHOT_TXG - 1, 1, state_before),
                (FIRST_SNAPSHOT_TXG, 1, state_after),
            ],
            &[],
        ),
        vec![FIRST_SNAPSHOT_TXG, FIRST_SNAPSHOT_TXG - 1],
        "建快照那一版与它之前那一版各计一个状态"
    );
}
