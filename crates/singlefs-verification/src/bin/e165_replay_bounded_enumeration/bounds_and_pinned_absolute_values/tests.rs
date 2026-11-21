use super::*;

/// 不给 `--bounds` 是不指定范围：三族的命令行判的段数都等于登记的段数（登记 5.2、5.6 两段，按登记的次序逐段判）；
/// 给了 `--bounds` 才缩到给的那一段；认不出的界拒。登记的全集罩住每一种界（加一种界不补 [`BoundsTier::REGISTERED`]，穷举那一处编不过）。
/// 判别力（`crates/mutations.tsv`）：不给 `--bounds` 只判第一段，这条红。
#[test]
fn the_command_lines_without_bounds_judge_every_registered_segment_and_bounds_narrow_to_one() {
    let arguments =
        |text: &str| -> Vec<String> { text.split_whitespace().map(str::to_string).collect() };
    for bounds in [BoundsTier::SegmentOne, BoundsTier::SegmentTwo] {
        match bounds {
            BoundsTier::SegmentOne | BoundsTier::SegmentTwo => assert!(
                BoundsTier::REGISTERED.contains(&bounds),
                "{} 在登记的全集里",
                bounds.label()
            ),
        }
    }
    let registered_segment_count = 2;
    let judged_without_bounds = [
        parse_the_command_line(&arguments("--family legal"))
            .expect("认得的参数")
            .bounds_to_judge,
        parse_the_root_choice_command_line(&arguments("--family root_choice"))
            .expect("认得的参数")
            .bounds_to_judge,
        parse_the_root_choice_faults_command_line(&arguments("--family root_choice_faults"))
            .expect("认得的参数")
            .bounds_to_judge,
    ];
    for bounds_to_judge in judged_without_bounds {
        assert_eq!(
            bounds_to_judge.len(),
            registered_segment_count,
            "不给 --bounds 判的段数等于登记的段数"
        );
        assert_eq!(
            bounds_to_judge,
            vec![BoundsTier::SegmentOne, BoundsTier::SegmentTwo],
            "按登记的次序逐段判"
        );
    }
    assert_eq!(
        parse_the_command_line(&arguments("--bounds segment2 --family legal"))
            .expect("认得的参数")
            .bounds_to_judge,
        vec![BoundsTier::SegmentTwo],
        "给了 --bounds segment2 只判那一段"
    );
    assert!(parse_the_command_line(&arguments("--bounds segment3")).is_err());
}

/// 缩过域（给了 `--max-n` 或 `--max-u`）的那一趟配置行末尾标缩过；不给一个字不加，登记界的产物逐字节不变。
#[test]
fn the_configuration_line_marks_a_run_narrowed_by_a_cap_and_adds_nothing_without_one() {
    assert_eq!(
        narrowing_field_of_the_configuration_line(None, "max_n"),
        "",
        "不给缩域参数时配置行不多一个字"
    );
    assert_eq!(
        narrowing_field_of_the_configuration_line(Some(3), "max_n"),
        " narrowed_below_the_registered_bounds=max_n",
        "给了 --max-n 的那一趟配置行写明缩过"
    );
    assert_eq!(
        narrowing_field_of_the_configuration_line(Some(1), "max_u"),
        " narrowed_below_the_registered_bounds=max_u",
        "给了 --max-u 的那一趟配置行写明缩过"
    );
}
