use super::*;

#[test]
fn segment_two_sensitivity_pairs_four_absorb_and_borrow_rows_for_the_borrow_amount_and_six_rows_for_the_check_order() {
    let sequence = registered_sequences().into_iter().find(|sequence| sequence.name == "wa_keep1of2_a1000").expect("登记里有这一点");
    let result = run_sequence(sequence, &SEGMENT_TWO_ARMS, &SEGMENT_TWO_WINDOWS, 200_000);
    let lines = segment_two_sensitivity_lines(std::slice::from_ref(&result));
    let knob_and_row = |line: &String| -> (String, String) {
        let field = |key: &str| line.split(' ').find_map(|part| part.strip_prefix(key)).expect("每一行都有这个字段").to_string();
        (field("knob="), field("row="))
    };
    let observed: Vec<(String, String)> = lines.iter().map(knob_and_row).collect();
    let expected: Vec<(String, String)> = [
        ("borrow_amount_just_enough", "Q2p_delta3"),
        ("borrow_amount_just_enough", "Q2c_delta3"),
        ("borrow_amount_just_enough", "Q3p_delta32"),
        ("borrow_amount_just_enough", "Q3c_delta32"),
        ("check_order_left_first", "Q2p_delta2"),
        ("check_order_left_first", "Q2p_delta3"),
        ("check_order_left_first", "Q2c_delta2"),
        ("check_order_left_first", "Q2c_delta3"),
        ("check_order_left_first", "Q3p_delta32"),
        ("check_order_left_first", "Q3c_delta32"),
    ]
    .map(|(knob, row)| (knob.to_string(), row.to_string()))
    .to_vec();
    assert_eq!(observed, expected);
    let left_first_delta2 = point_value(&result, JUDGED_ROWS[1], 2);
    assert!(lines[4].contains(&format!(" opposite_value={} ", value_text(left_first_delta2))), "先查左边那一点取的是 left_first 窗口：{}", lines[4]);
    assert!(lines[4].contains(" opposite=wa_keep1of2_a1000@left_first "), "{}", lines[4]);
}
