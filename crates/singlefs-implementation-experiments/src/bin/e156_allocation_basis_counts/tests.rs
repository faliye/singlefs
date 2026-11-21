//! K1（新池新建文件之后的记账三项）、G27（`referenced_slots`）、R8（U7）、R3 的判别力（U8）、R-3 闭式（U10、U12）的最小单测，
//! 与第 4 次重跑登记第九节的 U11、U13–U18、第 5 次重跑登记第九节的 U19。

/// E24（第 7 次重跑登记 5.4）：同槽叠洞的新入口在 S = 4、m = 2 上造出的洞，前位置 [5, 17]、后位置 [16, 28]；
/// 登记第七节 A21 的例子 S = 8、m = 3，前位置 [5, 29, 53]、后位置 [28, 52, 76]。M51（第二个洞起每个洞多隔一个 txg）在这里红。
#[test]
fn holes_on_the_same_ring_slot_are_spaced_exactly_one_ring_apart() {
    let anchor_of = |slots_per_region: u64, front: bool, hole_count: u64| {
        super::anchor_model::run_the_anchor_schedule(
            slots_per_region,
            super::anchor_model::HoleSchedule::HolesOnTheSameRingSlot { front, hole_count },
            None,
        )
    };
    let front_two = anchor_of(4, true, 2);
    assert_eq!(front_two.holes, vec![5, 17], "S = 4、m = 2、前位置");
    assert_eq!(front_two.recovery_ends, vec![7, 19]);
    assert!(!front_two.fits, "两个洞落在同一个环槽，排不下");
    let back_two = anchor_of(4, false, 2);
    assert_eq!(back_two.holes, vec![16, 28], "S = 4、m = 2、后位置");
    assert_eq!(back_two.recovery_ends, vec![19, 31]);
    let front_one = anchor_of(4, true, 1);
    assert_eq!(front_one.holes, vec![5], "m = 1 与 Hh(1, 前) 同一个洞");
    assert!(front_one.fits);
    let eight_front_three = anchor_of(8, true, 3);
    assert_eq!(
        eight_front_three.holes,
        vec![5, 29, 53],
        "A21：S = 8、m = 3、前位置"
    );
    assert_eq!(eight_front_three.recovery_ends, vec![7, 31, 55]);
    assert_eq!(eight_front_three.end, 79);
    let eight_back_three = anchor_of(8, false, 3);
    assert_eq!(
        eight_back_three.holes,
        vec![28, 52, 76],
        "A21：S = 8、m = 3、后位置"
    );
    assert_eq!(eight_back_three.end, 102);
}

/// A20（第 7 次重跑登记第七节）：锚点模型算的 k_满、前位置排得下的最大洞数、后位置排得下的最大洞数，三个 S 上与登记的 2 / 4 / 8、3 / 5 / 9、2 / 4 / 8 相同。
/// M52（k_满 的循环多算一档）在这里红。
#[test]
fn ring_full_and_fitting_hole_counts_match_the_registered_table() {
    for (slots_per_region, inside_the_first_ring, fitting_front, fitting_back) in
        [(4u64, 2u64, 3u64, 2u64), (8, 4, 5, 4), (16, 8, 9, 8)]
    {
        assert_eq!(
            super::anchor_model::largest_front_hole_count_inside_the_first_ring(slots_per_region),
            inside_the_first_ring,
            "k_满，S = {slots_per_region}"
        );
        assert_eq!(
            super::anchor_model::largest_hole_count_that_fits(slots_per_region, true),
            fitting_front,
            "前位置排得下的最大洞数，S = {slots_per_region}"
        );
        assert_eq!(
            super::anchor_model::largest_hole_count_that_fits(slots_per_region, false),
            fitting_back,
            "后位置排得下的最大洞数，S = {slots_per_region}"
        );
    }
}
