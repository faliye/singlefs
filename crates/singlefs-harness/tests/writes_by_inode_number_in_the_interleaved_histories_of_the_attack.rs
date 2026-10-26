//! 攻方腿（m3-close-w5-code-r1）的交错历史原样钉进 harness 档（代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md`
//! 第三节第 2 条「攻方 W4 的两段历史钉进 harness 档用例」）：驱动照抄
//! `research/prompts/m3-close-w5-code-r1-opus-model/attack_histories_of_multi_file_writes_and_unlinks.rs` 的 `interleaved_history`
//! （建 inode、按号写、按号删、回退到更早的一版、可写挂载一份拷贝，抽法与次序逐字不变，同一个种子走出同一段历史），带它自己的小模型：
//! 活着的号与内容、出生代、这条时间线上 inode 1 的死亡 txg（回退时整份换成回退目标那一刻的）。每次发布之后池级 checker 全绿、每一步之后活着的号
//! 只读挂载读回与小模型相同、按号写的结局与这条时间线上的 β 相同（拒 ⇔ 第一个文件在这条时间线上删完了而目标生于那之后）。
//! 种子 200 与 1051 是攻方量到 β 对不上的两段（`out/run-seed200-trace.log`、`out/b3-5.log`）。另数这两段里「这条时间线上的 β 拒、而墓碑树里
//! inode 1 最晚那条死亡 txg 会放过」的写，至少一次：钉的历史真走到父链与最晚那条分岔的那一形。

//! 本档只核实现行为；同名场景交出的镜像由 checker 档给定状态引擎两段判。

mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/writes_by_inode_number_in_the_interleaved_histories_of_the_attack.rs"]
mod scenarios;

#[test]
#[ignore = "harness 慢档用例：攻方种子 200 那一段 40 步交错历史，每次发布与每一步都跑池级 checker、只读挂载读回，debug 下单条量到 87 秒"]
fn the_interleaved_history_of_the_attack_seed_two_hundred_agrees_with_beta_on_its_parent_chain() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::the_interleaved_history_of_the_attack_seed_two_hundred_agrees_with_beta_on_its_parent_chain);
}

#[test]
fn the_interleaved_history_of_the_attack_seed_one_thousand_and_fifty_one_agrees_with_beta_on_its_parent_chain(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::the_interleaved_history_of_the_attack_seed_one_thousand_and_fifty_one_agrees_with_beta_on_its_parent_chain);
}
