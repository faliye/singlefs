use super::*;

#[test]
fn the_cell_that_cannot_be_measured_is_named_and_left_out_of_every_reading() {
    let flat = synthetic_counts_text(&synthetic_unit_area_points(4, 8));
    let full_five = format!(
        "{flat}E203RESULT name=rounds_taken point=unit_area_2_pow_24 state=same_mount taken=5 attempts=5\n"
    );
    let all_measured = report_of_counts(&full_five);
    assert!(
        all_measured.contains("name=verdict clause=void_8 cells_not_measurable_in_counts=0 cells_not_measurable_in_timing=0"),
        "{all_measured}"
    );
    assert!(
        !all_measured.contains("name=b8_input_gap"),
        "{all_measured}"
    );
    let three_of_five = format!(
        "{flat}E203RESULT name=rounds_taken point=unit_area_2_pow_24 state=same_mount taken=3 attempts=10\n"
    );
    let report = report_of_counts(&three_of_five);
    assert!(
        report.contains("name=void_8_cell build=counts point=unit_area_2_pow_24 state=same_mount kind=overwrite taken=3 attempts=10 result=not_measurable"),
        "{report}"
    );
    assert!(
        report.contains("name=verdict clause=void_8 cells_not_measurable_in_counts=1 cells_not_measurable_in_timing=0"),
        "{report}"
    );
    assert!(
        report.contains(
            "name=b8_input_gap point=unit_area_2_pow_24 state=same_mount kind=overwrite"
        ),
        "量不到的格整格拿掉，B8 的输入记读不到：{report}"
    );
    assert!(
        !report.contains("name=b8_input_gap point=unit_area_2_pow_24 state=remounted"),
        "只拿掉量不到的那一种挂载状态：{report}"
    );
    assert!(
        report.contains("name=verdict quantity=Q6 result=SomeInputUnread"),
        "有格读不到不记「不随」：{report}"
    );
}
