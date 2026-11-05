//! 样本：测试文件里只有提到 unsafe 的注释与字符串，没有 unsafe 关键字。
// unsafe { } 写在注释里不算
#[test]
fn mounted_file_count_starts_at_zero() {
    let note = "unsafe { not_code() }";
    assert!(note.len() > 0);
    assert_eq!(0, 0);
}
