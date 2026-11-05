// 样本：池级 checker 自己的单测（登记表 excluded 那一行），整份不判
#[test]
fn a_bad_image_is_refused() { assert!(!check_pool_image(&bad).is_empty()); }
