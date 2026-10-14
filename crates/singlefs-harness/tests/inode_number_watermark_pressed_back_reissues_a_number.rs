//! inode 号水位被只供测试的开关压回一个已发过的号（`.claude/kb/checks-owed.md` C479）：产品路径上号只从记账里的水位发、
//! 水位只增，「复用一个已发过的号」造不出来；开关（`PoolWriter::set_inode_number_watermark_source`）把这次发布的起号压回去，
//! 再建一个 inode，看池级 checker 的 I-9.6（水位大于两处最大号）拦不拦得住。
//! 判别力自证：同一段流把开关关掉，I-9.6 由红转绿。
//! I-9.6 拦不住的那一片（攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N7」一节逐点扫过）：压回到 p、建 n 个之后记账的水位是 p + n，
//! 只要 p + n 大于 checker 判的两处最大号——它走到的各条根里 inode 树的最大 key，与所选根墓碑树里已发布墓碑的最大对象 ID——两半都成立，
//! 被重发的那几个号（p 到 p + n − 1 里原来发过的：记录还在树里被原地换掉，或已 unlink 删完、墓碑在所选根的墓碑树里）I-9.6 拦不住。
//! 只建一个（n = 1）时这一片正是「压回的正是发过的最大号」；建几个时压回的号可以低于发过的最大号（攻方那 90 个判不出的点里 42 个是这样，
//! 都在 n ≥ 2）。这一片这里不钉，交主 agent 定要不要另立一条不变量（C479 那一行；判决 `research/prompts/m3-batch19-code-r1-main-verification.md`
//! 第二节「N7 附带」那一行交给 C479 那一问当材料）。
//! p + n 不大于墓碑最大对象号、又大于各条根里 inode 树最大 key 的那一形，I-9.6 墓碑那一半判得出，钉在
//! [`creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant`]。
//! C479 那一问用户 2026-10-05 定「写」（D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」），立 I-9.29（同号只许一个出生代，
//! 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T10 乙）：重发的号在回退候选集里还有一版留着旧对象时它红，
//! 下面三条用例都红它；I-9.6 拦不住的「压回的正是发过的最大号」那一形红它与 I-9.30，钉在
//! [`pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants`]。
//! I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 第 49 批）三条都红：压回之前那次建 inode 的发布留了一级台阶 (g, H)，
//! 重发的号小于 H、它的新对象晚于 g 出生（②）；水位压到那一级之下时 ① 也不成立。
mod common;
mod common_deletion_in_progress;
mod common_images_for_the_checker_tier;
mod common_pointer_head_corruption;
#[path = "common_images_for_the_checker_tier/inode_number_watermark_pressed_back_reissues_a_number.rs"]
mod scenarios_of_inode_number_watermark_pressed_back_reissues_a_number;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants,
    );
}
