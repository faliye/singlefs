use super::*;

#[test]
fn root_versions_compare_the_txg_first_and_the_instance_breaks_ties() {
    let version = |checkpoint_txg: u64, instance: u32| RootVersion {
        checkpoint_txg,
        instance,
    };
    assert!(
        version(100, 1) > version(99, 9),
        "txg 为主：txg 大的版本大，不看实例代号"
    );
    assert!(
        version(100, 2) > version(100, 1),
        "txg 相等时实例代号大的版本大"
    );
    // txg 与实例都相等时两边谁也不比谁小：比较器的实例那一段写成非严格（相等也判小）时这里红，上面两条照绿。
    let one_version = version(7, 3);
    let the_same_version = version(7, 3);
    assert!(
        !(one_version < the_same_version) && !(the_same_version < one_version),
        "txg 与实例都相等：谁也不比谁小"
    );
}

#[test]
fn the_warm_up_plan_covers_both_devices_with_at_most_two_empty_publishes_after_the_row_write() {
    assert_eq!(root_choice_warm_up_plan(1), vec![1, 2]);
    assert_eq!(root_choice_warm_up_plan(2), vec![2, 3, 4]);
    assert_eq!(root_choice_warm_up_plan(3), vec![3, 4]);
    assert_eq!(root_choice_warm_up_plan(4), vec![4, 5]);
    for row_write_txg in 1..=30 {
        let plan = root_choice_warm_up_plan(row_write_txg);
        let mut covered = [false; 2];
        for checkpoint_txg in &plan {
            covered[root_choice_device_of_publish(*checkpoint_txg)] = true;
        }
        assert!(
            covered[0] && covered[1],
            "写行 txg {row_write_txg} 的计划 {plan:?} 没覆盖两块盘"
        );
        assert!(
            plan.len() <= 3,
            "写行之后至多两次空发布（几何的结果）：{plan:?}"
        );
    }
}
