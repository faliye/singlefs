#[test]
fn the_sample_test_that_really_exists() {
    assert_eq!(1 + 1, 2, "样本：这个测试名要被已还清那一格找到");
}

// 旧名 the_sample_test_left_only_in_a_comment 只剩在这句注释里：测试本身已经没了
#[test]
fn the_sample_test_renamed_with_suffix_v2() {
    assert_eq!(2 + 2, 4, "样本：改名加了后缀，已还清那一行引的旧名按整词找不到");
}
