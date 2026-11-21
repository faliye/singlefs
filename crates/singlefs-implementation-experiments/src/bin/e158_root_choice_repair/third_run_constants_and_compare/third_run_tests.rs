use super::*;

/// 登记 5.6 与 7.2：第 3 次跑的常量回比、偏移解码、根槽 / 环槽 / 系统配置槽数全过（V3、V4；变异 M12 要它红）。
#[test]
fn third_run_constants_and_anchors_all_pass() {
    assert!(run_third_run_constants_and_anchors("today"));
}

/// 跨臂对拍读产物行：值里不带空格的字段按 `k=v` 取得出。
#[test]
fn result_line_fields_are_split_on_spaces() {
    let fields = fields_of_a_result_line(
        "E7RESULT name=r3_h1d_cell arm=today n1=2 cut=none a_reads=12 q3c_tail=own_version:2:9",
    );
    assert_eq!(fields.get("n1").map(String::as_str), Some("2"));
    assert_eq!(
        fields.get("q3c_tail").map(String::as_str),
        Some("own_version:2:9")
    );
}
