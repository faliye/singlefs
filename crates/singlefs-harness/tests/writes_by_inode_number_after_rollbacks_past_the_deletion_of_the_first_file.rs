//! K-U5 的 β 时序级读法只认这一版父链上的 inode 1 墓碑（D8（核心索引结构） 已定项 16 射程；代码轮判决
//! `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 1 条）：墓碑随现行那一版、管理员回退不取回退目标的，
//! 删第一个文件、回退到删之前、再删一次、再回退之后墓碑树里 inode 1 有两条，管着这一版的是父链上的那一条，不是最晚的那一条。
//! 历史取自攻方腿（`research/prompts/m3-close-w5-code-r1-opus-model/attack_histories_of_multi_file_writes_and_unlinks.rs` 的
//! `beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through` 与
//! `a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta`），这里照条款写期望。
//! 另一条钉住「inode 1 的记录还在就不判」那一句：第一个文件删除中（意图之后、队列没走空）时 β 不判，写入口落到「承诺类意图队列已走空」那道断言。
//! 都在同一个进程里直调发布入口（不经会话：会话先走空，删除中那一形走不到写入口）。

//! 本档只核实现行为；同名场景交出的镜像由 checker 档给定状态引擎两段判。

mod common;
mod common_deletion_in_progress;
mod common_images_for_the_checker_tier;
mod common_pointer_head_corruption;
#[path = "common_images_for_the_checker_tier/writes_by_inode_number_after_rollbacks_past_the_deletion_of_the_first_file.rs"]
mod scenarios;

#[test]
fn after_two_rollbacks_beta_refuses_a_write_to_an_inode_born_after_the_deletion_on_the_parent_chain(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::after_two_rollbacks_beta_refuses_a_write_to_an_inode_born_after_the_deletion_on_the_parent_chain);
}

#[test]
fn a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta);
}

#[test]
fn while_the_first_file_is_being_deleted_beta_does_not_judge_and_the_write_reaches_the_empty_queue_assertion(
) {
    scenarios::while_the_first_file_is_being_deleted_beta_does_not_judge_and_the_write_reaches_the_empty_queue_assertion();
}
