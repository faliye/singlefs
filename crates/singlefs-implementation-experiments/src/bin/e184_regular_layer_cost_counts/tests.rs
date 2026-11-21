use super::*;

#[test]
fn the_local_constants_agree_with_the_crates_and_the_stop_checks_of_the_constants_pass() {
    let readings = local_constants_checks();
    assert_eq!(readings.len(), 3);
    assert!(
        readings.iter().all(|reading| reading.passed),
        "{readings:?}"
    );
}

#[test]
fn the_own_source_file_name_points_at_this_very_source_in_the_source_directory_of_the_binaries() {
    let path = std::path::Path::new(OWN_SOURCE_PATH);
    assert!(
        path.ends_with(OWN_SOURCE_FILE_NAME),
        "交给 preflight 的文件名 {OWN_SOURCE_FILE_NAME} 不是 src/bin 下的这一份 {}",
        path.display()
    );
    let text = std::fs::read_to_string(path).expect("preflight 要判的源文件读得到");
    assert_eq!(
        text,
        OWN_SOURCE_TEXT,
        "{} 就是编进来的这一份",
        path.display()
    );
    assert!(
        text.contains("// admission:") && text.contains("// run-condition:"),
        "文件头要有准入与运行条件"
    );
}

#[test]
fn the_selfcheck_passes_every_stop_check_and_every_anchor_on_the_current_crates() {
    assert_eq!(
        run_selfcheck_command(&["unexpected".to_string()]),
        EXIT_CODE_USAGE
    );
    assert_eq!(
        run_selfcheck_command(&[]),
        EXIT_CODE_OK,
        "停机 T1、T3、T4 与锚点 A1–A6 全过（B1 对不上是登记预计的，不算在内）"
    );
}
