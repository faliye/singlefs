//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 62 行（C533（记录新根段缺实例表与分配记录树两个根指针））：复现，只坐实、不修。
//!
//! 历史：只做过 mkfs 的池 → 可写挂载（取号 1、不写行、零单元发布 txg 1、暖机 txg 2）→ 进程退出 → 再可写挂载：取号 2，
//! 写行那次发布在树表 0 条的一版上另写实例表与一棵分配记录树（D16（发布语义） 已定项 9；按位置寻址，D8（核心索引结构） 已定项 14：
//! 4 GiB 两块盘上五个节点），txg 3、jsn 3——崩在这次发布的记录落盘之后、根槽落盘之前 → 重开。
//! 下文说的「那两个单元」指实例表与分配记录树（后者是五个节点），一共六个单元。
//!
//! 这条用例钉住今天的样子，逐条对着收口表第 62 行要问的几样：
//! - 那两个单元落在哪：写行那次发布取的两个落点，崩溃镜像上两块盘都有、校验和对得上；记录 jsn 3 两份也在。
//! - 恢复重建的根拿不拿得到它们：拿不到，而且那条记录**根本不施加**——它是实例 2 的第一条，所选根是实例 1 的 (1, 2)，
//!   前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1），水位之上一条都没有。重建出来的根就是 (1, 2) 本身：
//!   实例表指针是 mkfs 那一片、分配记录树根指针全零。写行那次发布恒是新实例的第一次发布，所以「施加一条写行记录」今天走不到。
//! - checker 怎么算它们：遍历从根走，谁也不引用这两个单元，一条违例都没有；把它们改坏，判决除 I-8.11（点名项与被指单元头相同）
//!   之外逐条不变（走读不读它们；I-8.11 按记录 jsn 3 的点名项读它们，改坏之后校验和对不上、没有可比的，从成立转成不适用）。
//! - 分配器怎么算它们：重开之后的可写挂载从 mkfs 那一版的账重建，这两个落点不在任何一条分配记录里；记录 jsn 3 在水位之上、
//!   没被施加，C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条，删除线第 33 批 T）把它点名的六个落点隔离，
//!   这次挂载写行那次发布取的是别的槽、一个都不碰它们（改 C 之前取的正是这六个槽、把它们盖掉）。之后 checker 仍一条违例都没有。
//!
//! 条款怎么定（新根段加这两样 / 这种发布整次不施加 / 别的）交用户；这里只记今天的结局。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/row_publish_record_without_its_root.rs"]
mod scenarios_of_row_publish_record_without_its_root;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_row_publish_record_without_its_root::c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_row_publish_record_without_its_root::c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount,
    );
}
