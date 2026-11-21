use super::*;

#[test]
fn round_statistics_report_median_spread_and_stability() {
    let unstable = round_statistics(&[100.0, 110.0, 90.0, 105.0, 95.0]);
    assert_eq!(unstable.median, 100.0);
    assert_eq!(unstable.minimum, 90.0);
    assert_eq!(unstable.maximum, 110.0);
    assert_eq!(unstable.spread_percent, 20.0);
    assert!(!unstable.is_stable, "离散 20% 超过 15%");
    let boundary = round_statistics(&[100.0, 115.0, 100.0, 100.0, 100.0]);
    assert_eq!(boundary.spread_percent, 15.0);
    assert!(boundary.is_stable, "恰好 15% 算稳定");
    let even = round_statistics(&[1.0, 2.0, 3.0, 10.0]);
    assert_eq!(even.median, 2.5, "偶数个取中间两个的平均");
}

#[test]
fn summarize_takes_the_successful_attempt_and_excludes_cache_substituted_cells() {
    let product = "E152RUN_HEADER kernel=test rounds=2\n\
E152RUN configuration=ext4 round=1 attempt=1 host_load1=0.5 vm_exit=1\n\
E7RESULT name=failure configuration=ext4 round=1 stage=format detail=x\n\
E7RESULT name=done emitted=2\n\
E152RUN configuration=ext4 round=1 attempt=2 host_load1=0.5 vm_exit=0\n\
E7RESULT name=fio configuration=ext4 round=1 job=write_sequential_large write_bandwidth_kibibytes_per_second=1048576 verdict=counted\n\
E7RESULT name=fio configuration=ext4 round=1 job=read_sequential_large read_bandwidth_kibibytes_per_second=2097152 verdict=cache_substituted\n\
E7RESULT name=control configuration=ext4 round=1 second_pass_verdict=cache_substituted control_has_teeth=true\n\
E7RESULT name=done emitted=4\n\
E152RUN configuration=ext4 round=2 attempt=1 host_load1=0.7 vm_exit=0\n\
E7RESULT name=fio configuration=ext4 round=2 job=write_sequential_large write_bandwidth_kibibytes_per_second=2097152 verdict=counted\n\
E7RESULT name=control configuration=ext4 round=2 second_pass_verdict=cache_substituted control_has_teeth=true\n\
E7RESULT name=done emitted=3\n";
    let output = summarize_product(product);
    assert!(
        output.contains(&"E7RESULT name=summary configuration=ext4 metric=write_sequential_mebibytes_per_second rounds=2 median=1536.000 minimum=1024.000 maximum=2048.000 spread_percent=66.7 stability=unstable values=1:1024.000,2:2048.000".to_string()),
        "两轮都要取成功的那一次：{output:#?}"
    );
    assert!(output.contains(
        &"E7RESULT name=summary_excluded configuration=ext4 metric=read_sequential_mebibytes_per_second round=1 reason=cache_substituted".to_string()
    ));
    assert!(output.contains(&"E7RESULT name=summary_control configuration=ext4 rounds_checked=2 rounds_with_teeth=2".to_string()));
    assert_eq!(output.iter().filter(|line| line.contains("name=summary_failed_round")).count(), 0);
    assert_eq!(output.len(), 4);
    assert_eq!(output.last(), Some(&"E7RESULT name=summary_done emitted=4".to_string()));
}

#[test]
fn summarize_keeps_file_overwrite_wall_clock_only_for_rounds_whose_outer_phase_contains_inner() {
    let round_block = |round_number: u32, outer_nanoseconds: u64, inner_nanoseconds: u64, containment_field: &str| {
        format!(
            "E152RUN configuration=singlefs round={round_number} attempt=1 host_load1=2.0 vm_exit=0\n\
E7RESULT name=singlefs_timing configuration=singlefs round={round_number} file_overwrite_nanoseconds={outer_nanoseconds} file_overwrite_inner_nanoseconds={inner_nanoseconds}{containment_field}\n\
E7RESULT name=done emitted=2\n"
        )
    };
    let product = [
        round_block(1, 16_000_000, 6_000_000, " file_overwrite_outer_contains_inner=true"),
        round_block(2, 2_000_000, 6_500_000, " file_overwrite_outer_contains_inner=false"),
        round_block(3, 12_000_000, 6_200_000, " file_overwrite_outer_contains_inner=true"),
        round_block(4, 3_000_000, 6_100_000, ""),
        round_block(5, 14_000_000, 6_400_000, " file_overwrite_outer_contains_inner=true"),
    ]
    .concat();
    let output = summarize_product(&product);
    let wall_clock_lines: Vec<&str> = output
        .iter()
        .map(String::as_str)
        .filter(|line| line.contains("metric=singlefs_file_overwrite_milliseconds "))
        .collect();
    assert_eq!(
        wall_clock_lines,
        [
            "E7RESULT name=summary_excluded configuration=singlefs metric=singlefs_file_overwrite_milliseconds round=2 reason=outer_does_not_contain_inner",
            "E7RESULT name=summary_excluded configuration=singlefs metric=singlefs_file_overwrite_milliseconds round=4 reason=missing_file_overwrite_outer_contains_inner",
            "E7RESULT name=summary configuration=singlefs metric=singlefs_file_overwrite_milliseconds rounds=3 median=14.000 minimum=12.000 maximum=16.000 spread_percent=28.6 stability=unstable values=1:16.000,3:12.000,5:14.000",
        ],
        "判假与缺字段的两轮照排除的格子报，只有三轮进中位：{output:#?}"
    );
    let inner_lines: Vec<&str> = output
        .iter()
        .map(String::as_str)
        .filter(|line| line.contains("metric=singlefs_file_overwrite_inner_milliseconds "))
        .collect();
    assert_eq!(
        inner_lines,
        ["E7RESULT name=summary configuration=singlefs metric=singlefs_file_overwrite_inner_milliseconds rounds=5 median=6.200 minimum=6.000 maximum=6.500 spread_percent=8.1 stability=stable values=1:6.000,2:6.500,3:6.200,4:6.100,5:6.400"],
        "子进程自己计的数不看包含检查，五轮都进：{output:#?}"
    );
    assert_eq!(
        file_overwrite_wall_clock_if_contained(Some("NA"), Ok(2.0)),
        Err("containment_not_available".to_string()),
        "判不了的轮也不收"
    );
}
