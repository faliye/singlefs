use super::*;

/// 登记 7.2：H-随 (a) 28 个种子、最小 7463871032432355115、最大 7463871032432355205；段数 28 + 1 + 1 = 30；H-随全 68 段。
#[test]
fn random_history_seeds_and_segment_counts_match_the_registered_values() {
    let seeds: Vec<u64> = random_red_histories()
        .iter()
        .take(RANDOM_HISTORY_RED_SEED_OFFSETS.len())
        .map(|segment| segment.history.seed.0)
        .collect();
    assert_eq!(seeds.len(), 28);
    assert_eq!(
        seeds.iter().min(),
        Some(&RANDOM_HISTORY_RED_SEED_MINIMUM_REGISTERED)
    );
    assert_eq!(
        seeds.iter().max(),
        Some(&RANDOM_HISTORY_RED_SEED_MAXIMUM_REGISTERED)
    );
    assert_eq!(
        random_red_histories().len(),
        RANDOM_HISTORY_SEGMENTS_REGISTERED
    );
    assert_eq!(random_all_histories().len() + 28, 96);
    assert!(random_all_histories()
        .iter()
        .all(|segment| segment.history.operations.len() == 30));
}

#[test]
fn seeds_listed_on_the_investigator_line_are_read_between_the_brackets() {
    let line = "新发现：第一个种子 7（同签名的种子 [7463871032432355115, 7463871032432355119]），在 Operation(4)";
    assert_eq!(
        seeds_listed_on_the_line(line),
        vec![7_463_871_032_432_355_115, 7_463_871_032_432_355_119]
    );
}
