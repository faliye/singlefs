// 样本：include! 一份不属任何包的 .inc、include_str! 读一份不属任何包的 .txt：不论后缀都判红。
include!("../../../shared/snippet.inc");

const TABLE: &str = include_str!("../../../notes/table.txt");

#[test]
fn unpackaged_files_are_read() {
    assert_eq!(from_unpackaged_inc(), 1);
    assert!(!TABLE.is_empty());
}
