use super::*;

#[test]
fn unit_writes_split_into_periods_by_the_unit_class_byte() {
    assert_eq!(period_of_unit_class(Some(1)), "data_unit");
    assert_eq!(period_of_unit_class(Some(2)), "index_node");
    assert_eq!(period_of_unit_class(Some(3)), "packed_records");
    assert_eq!(
        period_of_unit_class(Some(9)),
        "unit_write",
        "不认得的码不猜"
    );
    assert_eq!(
        period_of_unit_class(None),
        "unit_write",
        "短到没有第 6 字节"
    );
}
