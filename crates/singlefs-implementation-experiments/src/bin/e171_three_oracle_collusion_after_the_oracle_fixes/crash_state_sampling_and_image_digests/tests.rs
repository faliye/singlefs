//! 三批取样点、撕裂扫描的取样序号、状态掩码与崩溃镜像的摘要的单测。

use super::*;

/// 空消息的 sha256（`python3 -c "import hashlib; print(hashlib.sha256(b'').hexdigest())"` 现算）。
const EMPTY_MESSAGE_SHA256: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
/// 盘 1、扇区 2、512 个 0xAB 那一条的摘要（`struct.pack('<IQ', 1, 2) + b'\xab' * 512` 喂 hashlib 现算）。
const ONE_SECTOR_OF_0XAB_SHA256: &str =
    "c71cae1b3fc678e4cabfa21d575c728edd44a4b372b3389ce763856273abecbf";

#[test]
fn torn_landing_folds_to_one_and_a_persisted_write_to_two() {
    // 写表：0 持久、1 没持久、2 原写没持久而撕裂镜像（枚举写表下标 3）持久。
    let persisted = vec![true, false, false, true];
    let torn_image_index_by_write = BTreeMap::from([(2usize, 3usize)]);
    assert_eq!(
        landing_vector_of(&persisted, 3, &torn_image_index_by_write),
        "201",
        "撕裂那一态折回 1，不是 2"
    );
    let persisted_with_the_torn_image_absent = vec![true, false, false, false];
    assert_eq!(
        landing_vector_of(
            &persisted_with_the_torn_image_absent,
            3,
            &torn_image_index_by_write
        ),
        "200"
    );
}

#[test]
fn an_all_zero_sector_does_not_enter_the_image_digest() {
    let mut message = Vec::new();
    append_sector_to_digest_message(&mut message, 0, 7, &[0u8; 512]);
    assert!(message.is_empty(), "全零扇区不进摘要");
    assert_eq!(sha256_text(&sha256_digest(&message)), EMPTY_MESSAGE_SHA256);
    append_sector_to_digest_message(&mut message, 1, 2, &[0xAB; 512]);
    assert_eq!(message.len(), 4 + 8 + 512);
    assert_eq!(
        sha256_text(&sha256_digest(&message)),
        ONE_SECTOR_OF_0XAB_SHA256
    );
}

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

#[test]
fn merged_sector_ranges_join_overlapping_and_touching_ranges_only() {
    assert_eq!(
        merged_sector_ranges(vec![(10, 12), (0, 2), (2, 3), (11, 15), (20, 21)]),
        vec![(0, 3), (10, 15), (20, 21)]
    );
}
