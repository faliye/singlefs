//! 取样（登记 5.4）：三批取样点与撕裂扫描的单测。

use super::*;

#[test]
fn batch_points_follow_the_registered_formulas_on_a_hundred_state_range() {
    let state_range = 10u64..110;
    assert_eq!(
        first_batch_points(&state_range)
            .into_iter()
            .collect::<BTreeSet<u64>>(),
        BTreeSet::from([10, 24, 38, 52, 66, 80, 94, 109])
    );
    assert_eq!(
        second_batch_points(&state_range),
        vec![17, 31, 45, 59, 73, 87, 101]
    );
    let third = third_batch_points(&state_range);
    assert_eq!(third.len(), 24);
    assert_eq!(third[0], 10);
    assert_eq!(third[1], 14, "10 + ⌊99/23⌋");
    assert_eq!(third[23], 109);
}

#[test]
fn batch_points_on_short_ranges_collapse_onto_the_first_batch() {
    let single = 5u64..6;
    assert_eq!(
        first_batch_points(&single)
            .into_iter()
            .collect::<BTreeSet<u64>>(),
        BTreeSet::from([5])
    );
    let eight = 0u64..8;
    assert_eq!(
        first_batch_points(&eight)
            .into_iter()
            .collect::<BTreeSet<u64>>(),
        (0..8).collect::<BTreeSet<u64>>()
    );
    assert_eq!(second_batch_points(&eight), vec![0, 1, 2, 3, 4, 5, 6]);
}

#[test]
fn sampled_batches_are_disjoint_and_include_the_last_state_and_up_to_four_torn_states() {
    // 两段：[0, 8) 带撕裂写，序号为奇数的状态带 1；[8, 108) 不带。总数 109，最后那个是 108。
    let ranges = vec![0u64..8, 8..108];
    let landing = |ordinal: u64| {
        if ordinal % 2 == 1 {
            "21".to_string()
        } else {
            "20".to_string()
        }
    };
    let sampled = sample_ordinals(&ranges, 109, &[true, false], &landing);
    let [first, second, third] = &sampled.batches;
    assert!(first.contains(&108), "最后那个全持久状态进批 1");
    assert!(first.is_disjoint(second) && first.is_disjoint(third) && second.is_disjoint(third));
    assert_eq!(
        sampled.torn_scans,
        vec![TornScan {
            segment_index: 0,
            scanned: 8,
            found: 0
        }],
        "[0, 8) 的点批 1 已经全取了，扫完 8 个一个新的都没有"
    );
}

#[test]
fn torn_scan_stops_after_four_new_states() {
    let ranges: Vec<std::ops::Range<u64>> = std::iter::once(0u64..1000).collect();
    let landing = |ordinal: u64| {
        if ordinal % 3 == 2 {
            "1".to_string()
        } else {
            "0".to_string()
        }
    };
    let sampled = sample_ordinals(&ranges, 1001, &[true], &landing);
    // 批 1 的格点 0、142、285、428、570、713、856、999；带 1 的是 2、5、8、11（都不在格点上）。
    assert_eq!(sampled.torn_scans[0].found, 4);
    assert_eq!(sampled.torn_scans[0].scanned, 12);
    for ordinal in [2u64, 5, 8, 11] {
        assert!(sampled.batches[0].contains(&ordinal));
    }
}
