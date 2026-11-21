use super::*;

// ───────── 第 2 次重跑（登记 `research/prompts/e185-r2-prereg.md` 第五节改动一至七、第九节 V56 起） ─────────

/// 手搭的分段输入：两个混住段（起点 0、320，各 3 个单元）、三个整段（起点 64、128、192，各按占比覆盖写），段 64 槽号最小的那个单元另由配方点名。
/// 输入次序倒过来，次序函数自己排。
fn units_to_overwrite_of_five_segments(fraction: LiveSlotFraction) -> Vec<UnitToOverwrite> {
    let mut units = Vec::new();
    for segment_start_slot in [0_u64, 320] {
        for position in 0..3_u64 {
            units.push(UnitToOverwrite {
                old_slot: segment_start_slot + 2 * position,
                inode: 1000 + segment_start_slot + position,
                segment_start_slot,
                is_in_a_segment_full_of_this_workload: false,
                is_named_by_the_recipe: false,
            });
        }
    }
    for segment_start_slot in [64_u64, 128, 192] {
        for position in 0..32_u64 {
            let is_named_by_the_recipe = segment_start_slot == 64 && position == 0;
            if fraction.is_overwritten(position) || is_named_by_the_recipe {
                units.push(UnitToOverwrite {
                    old_slot: segment_start_slot + 2 * position,
                    inode: 2000 + segment_start_slot + position,
                    segment_start_slot,
                    is_in_a_segment_full_of_this_workload: true,
                    is_named_by_the_recipe,
                });
            }
        }
    }
    units.reverse();
    units
}

/// 改法甲（占比 0.25）：混住段与配方点名的先写，整段按段起点升序，槽号最大的那一整段（起点 192）的 24 个最后写；组内按旧槽号升序。
#[test]
fn overwrite_order_at_one_quarter_writes_the_mixed_segments_first_and_the_highest_full_segment_last(
) {
    let units = units_to_overwrite_of_five_segments(LiveSlotFraction::OneQuarter);
    let order = overwrite_order_of(
        LiveSlotFraction::OneQuarter,
        OverwriteOrder::HighestFullSegmentLast,
        &units,
    );
    let expected_slots: Vec<u64> = [0_u64, 2, 4, 64, 320, 322, 324]
        .into_iter()
        .chain(
            [64_u64, 128, 192]
                .into_iter()
                .flat_map(|segment_start_slot| {
                    (0..32_u64)
                        .filter(|position| !position.is_multiple_of(4))
                        .map(move |position| segment_start_slot + 2 * position)
                }),
        )
        .collect();
    assert_eq!(
        order
            .iter()
            .map(|(old_slot, _inode)| *old_slot)
            .collect::<Vec<_>>(),
        expected_slots
    );
    assert_eq!(order.len(), 7 + 3 * 24);
    assert!(
        order[order.len() - 24..]
            .iter()
            .all(|(old_slot, _inode)| (192..256).contains(old_slot)),
        "最后 24 个全是槽号最大那一整段的"
    );
    assert_eq!(order[3], (64, 2064), "配方点名的那个单元跟着混住段先写");
}

/// 占比 0.75 用改法甲、或任何占比用 `slot-order`，都与按旧槽号升序（第 1 次重跑的 `sort_unstable`）逐项相同。
#[test]
fn overwrite_order_at_three_quarters_and_in_slot_order_equals_sorting_by_the_old_slot() {
    for (fraction, overwrite_order) in [
        (
            LiveSlotFraction::ThreeQuarters,
            OverwriteOrder::HighestFullSegmentLast,
        ),
        (LiveSlotFraction::ThreeQuarters, OverwriteOrder::SlotOrder),
        (LiveSlotFraction::OneQuarter, OverwriteOrder::SlotOrder),
    ] {
        let units = units_to_overwrite_of_five_segments(fraction);
        let mut sorted: Vec<(u64, u64)> = units
            .iter()
            .map(|unit| (unit.old_slot, unit.inode))
            .collect();
        sorted.sort_unstable();
        assert_eq!(
            overwrite_order_of(fraction, overwrite_order, &units),
            sorted,
            "{fraction:?} {overwrite_order:?}"
        );
    }
}

/// N12：前 24 份落在前沿（1000 起），后 6 份落进前面空出的旧槽；落在之后才空出的槽上的不算旧洞。
#[test]
fn copies_landing_in_holes_count_only_the_slots_opened_by_earlier_overwrites() {
    let old_slots: Vec<u64> = (0..30_u64).map(|index| 2 * index).collect();
    let new_slots: Vec<u64> = (0..24_u64)
        .map(|index| 1000 + 2 * index)
        .chain((0..6_u64).map(|index| 2 * index))
        .collect();
    assert_eq!(
        landings_of_the_copies(&old_slots, &new_slots),
        OverwriteLanding {
            copies_landing_in_holes_of_this_construction: 6,
            copies_landing_elsewhere: 24,
        }
    );
    assert_eq!(
        landings_of_the_copies(&[10, 20], &[20, 30]),
        OverwriteLanding {
            copies_landing_in_holes_of_this_construction: 0,
            copies_landing_elsewhere: 2,
        },
        "第 0 份落在第 1 次才空出的槽上，不是这之前打出的洞"
    );
    assert_eq!(predicted_copies_landing_in_holes(776), 752);
    assert_eq!(predicted_copies_landing_in_holes(6152), 6128);
    assert_eq!(predicted_copies_landing_in_holes(280), 256);
    assert_eq!(predicted_copies_landing_in_holes(2072), 2048);
}

/// 改动三的计数只读、不改池：P = 128、占比 0.75 的小池，数与不数副本落点造出的池逐块相同（第五节「怎么核」第 2 道的单测一半）。
#[test]
fn counting_the_overwrite_landings_leaves_the_pool_digests_unchanged() {
    let recipe = first_recipe_of(small_point(128));
    let counted = construct_the_pool(&recipe, LandingCount::Counted);
    let not_counted = construct_the_pool(&recipe, LandingCount::NotCounted);
    assert_eq!(
        digests_of_the_pool(&counted),
        digests_of_the_pool(&not_counted),
        "数落点不许多发一次发布、不许改池"
    );
    let landing = counted.overwrite_landing.expect("改法甲那一份数了落点");
    assert_eq!(
        landing.copies_landing_in_holes_of_this_construction + landing.copies_landing_elsewhere,
        counted.overwritten_units
    );
    assert!(not_counted.overwrite_landing.is_none());
}
