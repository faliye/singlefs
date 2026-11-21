//! 第三段：坏状态（登记 5.5、5.8「第三段」）的常量、坏族、表的版本与行的类型的单测。

use super::*;

/// splitmix64 从状态 0 起的头两个输出（公开参考实现的已知值，草稿目录里另用 Python 按同一式子算过）。
#[test]
fn splitmix64_from_state_zero_gives_the_reference_outputs() {
    let mut generator = SplitMix64 { state: 0 };
    assert_eq!(generator.next_value(), 0xE220_A839_7B1D_CDAF);
    assert_eq!(generator.next_value(), 0x6E78_9E6A_A1B9_65F4);
    assert!(generator.below(7) < 7);
}

/// 第 1 次重跑登记第十一节 S5 的单测形态：同一个种子、同一条流（流一，驱动快），两版的表逐行相同，只差 P8 的期望判定与末尾多出的 P10；
/// P10 的底态与 P9-Mk+ 同一个、补丁与同底态族的 P8-Mk 同一个（同一个改法落在 Mk⁺ 上）。
#[test]
fn the_second_run_table_differs_from_the_first_only_in_the_previous_version_landing_expectation_and_a_final_lower_bound_row_on_the_first_run_mk_plus_base(
) {
    let first_run_seed = 0x8546_b125_8e42_7c34_u64;
    let stream = prepare_stream(StreamName::NewPoolFileCreation);
    let facts = bad_stream_facts(&stream);
    let (_, first_run_rows) = bad_rows_of_stream(
        &facts,
        &mut SplitMix64 {
            state: first_run_seed,
        },
        BadTableVersion::FirstRun,
        &mut Vec::new(),
    );
    let (_, second_run_rows) = bad_rows_of_stream(
        &facts,
        &mut SplitMix64 {
            state: first_run_seed,
        },
        BadTableVersion::SecondRun,
        &mut Vec::new(),
    );
    let patch_texts = |row: &BadRow| -> Vec<(u32, u64, Vec<u8>)> {
        row.patches
            .iter()
            .map(|patch| (patch.device.0, patch.offset, patch.bytes.clone()))
            .collect()
    };
    assert_eq!(
        second_run_rows.len(),
        first_run_rows.len() + 1,
        "第 1 次重跑的表比第一次的多恰一行（P10）"
    );
    for (first_run_row, second_run_row) in first_run_rows.iter().zip(&second_run_rows) {
        assert_eq!(first_run_row.name, second_run_row.name);
        assert_eq!(first_run_row.family, second_run_row.family);
        assert_eq!(first_run_row.base.persisted, second_run_row.base.persisted);
        assert_eq!(patch_texts(first_run_row), patch_texts(second_run_row));
        assert_eq!(
            first_run_row.expected_landing.text(),
            second_run_row.expected_landing.text()
        );
        if second_run_row.name.starts_with("P8-") {
            assert_eq!(
                first_run_row.expected_verdict, "not_in",
                "{}",
                first_run_row.name
            );
            assert_eq!(
                second_run_row.expected_verdict, "in",
                "{}",
                second_run_row.name
            );
        } else {
            assert_eq!(
                first_run_row.expected_verdict, second_run_row.expected_verdict,
                "{}",
                first_run_row.name
            );
        }
    }
    let lower_bound_row = second_run_rows.last().expect("表不空");
    assert_eq!(lower_bound_row.name, LOWER_BOUND_ROW_NAME);
    assert_eq!(lower_bound_row.family, BadFamily::AcknowledgementLowerBound);
    assert_eq!(lower_bound_row.expected_verdict, "not_in");
    let first_run_mk_plus_row = second_run_rows
        .iter()
        .find(|row| row.name == FIRST_RUN_MK_PLUS_ROW_NAME)
        .expect("流一有 Mk⁺ 底态、有 P9 行");
    assert_eq!(
        lower_bound_row.base.persisted, first_run_mk_plus_row.base.persisted,
        "P10 的底态与 P9-Mk+ 是同一个落地向量（S5b）；取了 Mk 就差一个写"
    );
    let previous_version_landing_mk_row = second_run_rows
        .iter()
        .find(|row| row.name == "P8-Mk")
        .expect("有 P8-Mk 行");
    assert_eq!(
        patch_texts(lower_bound_row),
        patch_texts(previous_version_landing_mk_row)
    );
    assert_eq!(
        lower_bound_row.expected_landing.text(),
        previous_version_landing_mk_row.expected_landing.text()
    );
    assert!(
        lower_bound_row.base.persisted != previous_version_landing_mk_row.base.persisted,
        "Mk⁺ 比 Mk 多一个持久的写"
    );
}
