use super::*;

/// 场景：S1 里与流无关的那几条（盘宽、物理块与 io_min、扇区宽、固定 fsid、固定写时刻、第一个文件的字节数与内容），本地常量对验证包库的值。
/// 预期：每一条都成立，至少比了一条。
/// 判别力（`crates/mutations.tsv`）：本地的盘字节数多 512，这条红。
#[test]
fn stream_independent_stop_checks_hold_against_the_verification_library() {
    let checks = stream_independent_checks();
    assert!(!checks.is_empty(), "S1 里与流无关的回比至少一条");
    let broken: Vec<&StopCheck> = checks.iter().filter(|check| !check.holds()).collect();
    assert!(broken.is_empty(), "S1 里与流无关的这几条没过：{broken:?}");
}

#[test]
fn timing_summary_takes_the_lower_middle_and_the_ninety_fifth_percentile() {
    assert_eq!(timing_summary(vec![]), (0, 0, 0, 0));
    assert_eq!(timing_summary(vec![4, 1, 3, 2]), (10, 2, 4, 4));
    let hundred: Vec<u128> = (1..=100).collect();
    assert_eq!(timing_summary(hundred), (5050, 50, 95, 100));
}

#[test]
fn the_command_line_takes_dump_an_output_directory_and_an_optional_previous_export() {
    let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
    let parsed = parse_command_line(&words("dump --output-directory a --compare-with b"))
        .expect("合法的命令行");
    assert_eq!(parsed.output_directory, PathBuf::from("a"));
    assert_eq!(parsed.compare_with, Some(PathBuf::from("b")));
    assert!(parse_command_line(&words("dump --compare-with b")).is_none());
    assert!(parse_command_line(&words("dump --output-directory a --extra")).is_none());
    assert!(parse_command_line(&words("list --output-directory a")).is_none());
}
