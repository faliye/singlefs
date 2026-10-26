//! 删第一个文件之外的号时，删除的各批按那个号自己还剩的单元数走（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4
//! 的取样点：正推 `research/prompts/m3-multi-file-unlink-r1-sonnet-output.md` 第 6.2 节那张表「① 计划相等」一格——对 N ≠ 1 的批计划，批大小 1、2、
//! 144、145 各一——连同攻方 U4-乙「删 N 的每一批之后第一个文件读回逐字节相同」）。
//!
//! 每条用例一个场景：第一个文件三个单元，第一个文件旁边建到 inode 150（落在 extent 上段第 1 片叶，与第一个文件不在同一片上段叶里），给 150 写若干个
//! 单元，再按一种批大小删它。比三样：每一步之后 150 还剩的单元数逐步等于纯函数 `unlink::next_unlink_step` 从 150 自己的单元数算出的那一串，
//! 那一串也等于写在用例里的数；第一个文件每一步之后读回都与删之前逐字节相同、三个单元一个不少；每一版池级 checker 一条违例都没有。
//! 批大小 1、2 用只供测试的开关压小，144、145 是产品批大小（一片下段叶的 144 个）下单元数正好一批与多一个的两形。崩溃枚举那几格（同一份崩溃镜像
//! 各自续做逐字节相同、每个写点截断重开续做到底）归 checker 档，不在这里。

//! 本档只核实现行为；同名场景交出的镜像由 checker 档给定状态引擎两段判。

mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units.rs"]
mod scenarios;

#[test]
fn inode_150_of_three_data_units_in_batches_of_one_takes_two_batches_and_a_final_batch() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::inode_150_of_three_data_units_in_batches_of_one_takes_two_batches_and_a_final_batch);
}

#[test]
fn inode_150_of_seven_data_units_in_batches_of_two_takes_three_batches_and_a_final_batch() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::inode_150_of_seven_data_units_in_batches_of_two_takes_three_batches_and_a_final_batch);
}

#[test]
fn inode_150_of_144_data_units_with_the_product_batch_goes_straight_to_the_final_batch() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::inode_150_of_144_data_units_with_the_product_batch_goes_straight_to_the_final_batch);
}

#[test]
fn inode_150_of_145_data_units_with_the_product_batch_cuts_144_and_then_releases_the_last_one() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::inode_150_of_145_data_units_with_the_product_batch_cuts_144_and_then_releases_the_last_one);
}
