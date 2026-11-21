use super::*;

#[test]
fn shard_file_counts_as_finished_only_when_its_last_line_counts_every_line() {
    // 守卫认领临时目录：用例做完、断言失败、panic 都删掉它
    let directory = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(format!("e183-shard-file-{}", std::process::id())),
    );
    std::fs::create_dir_all(&*directory).expect("临时目录");
    let path = directory.join("h5-example");
    let lines = vec!["E7RESULT name=a".to_string(), "E7RESULT name=b".to_string()];
    write_shard_file_atomically(&path, "h5-example", &lines).expect("写得成");
    let text = std::fs::read_to_string(&path).expect("读得回");
    assert_eq!(
        text.lines().last(),
        Some("E7RESULT name=shard_done shard=h5-example lines=3")
    );
    assert_eq!(
        lines_of_a_finished_shard(&text, "h5-example"),
        Some(lines.clone())
    );
    assert_eq!(
        lines_of_a_finished_shard(&text, "h5-other"),
        None,
        "片名对不上不算"
    );
    let cut: String = text
        .lines()
        .skip(1)
        .map(|line| format!("{line}\n"))
        .collect();
    assert_eq!(
        lines_of_a_finished_shard(&cut, "h5-example"),
        None,
        "行数对不上不算"
    );
    let leftovers = std::fs::read_dir(&directory).expect("列目录").count();
    assert_eq!(leftovers, 1, "临时名已改名，目录里只剩片文件");
}

#[test]
fn result_lines_parse_into_named_fields_and_the_determinism_check_skips_only_timing_fields() {
    let line = ResultLine::parse(
        "E7RESULT name=h5_mount shard=x mount=3 sweep_matches_the_threshold=true",
    )
    .expect("产物行");
    assert_eq!(line.name(), Some("h5_mount"));
    assert_eq!(line.number("mount"), Some(3));
    assert_eq!(line.truth("sweep_matches_the_threshold"), Some(true));
    assert_eq!(ResultLine::parse("name=no_prefix"), None);
    let first =
        [ResultLine::parse("E7RESULT name=a shard=one mount_nanoseconds=5 rows0=7").expect("行")];
    let same =
        [ResultLine::parse("E7RESULT name=a shard=two mount_nanoseconds=9 rows0=7").expect("行")];
    let other =
        [ResultLine::parse("E7RESULT name=a shard=two mount_nanoseconds=9 rows0=8").expect("行")];
    assert_eq!(
        first_line_differing_outside_the_timing_fields(&first, &same),
        None
    );
    assert_eq!(
        first_line_differing_outside_the_timing_fields(&first, &other),
        Some(1)
    );
    assert_eq!(
        first_line_differing_outside_the_timing_fields(&first, &[]),
        Some(1),
        "少一行算不同"
    );
}
