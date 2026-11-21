use super::*;

/// 前台用户改动只覆盖写活数据文件的最后一个单元（按偏移写重写到文件末尾，R4：取别的单元一次要重写好几个），文件按种子在 P ÷ 16 个里均匀取。
/// 头几次的单元下标钉死（另用同一个 splitmix64 式子手算）；256 次在小池上落到 100 个以上的文件。
#[test]
fn the_user_change_overwrites_the_last_unit_of_a_live_data_file_drawn_by_the_seed() {
    let drawn = |live_data_units: u64| -> Vec<u64> {
        (0..6)
            .map(|change_number| unit_index_of_the_user_change(change_number, live_data_units))
            .collect()
    };
    assert_eq!(
        drawn(SMALL_POOL_LIVE_DATA_UNITS),
        [831_u64, 1743, 511, 1343, 1967, 1535]
    );
    assert_eq!(
        drawn(UnitAreaTier::EightGibibytes.live_data_units()),
        [52799_u64, 110959, 31823, 85487, 125055, 97759]
    );
    for live_data_units in [
        SMALL_POOL_LIVE_DATA_UNITS,
        UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes.live_data_units(),
    ] {
        let mut files = BTreeSet::new();
        for change_number in 0..256 {
            let unit_index = unit_index_of_the_user_change(change_number, live_data_units);
            let (file, index_in_file) = file_and_index_of_a_live_data_unit(unit_index);
            assert!(unit_index < live_data_units);
            assert_eq!(index_in_file + 1, LIVE_DATA_UNITS_PER_FILE);
            files.insert(file);
        }
        assert!(files.len() > 100, "{live_data_units}：{}", files.len());
    }
}
