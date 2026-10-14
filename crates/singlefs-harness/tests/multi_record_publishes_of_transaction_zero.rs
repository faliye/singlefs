//! 事务号 0 的发布切成多条记录（D23（journal 的角色与格式） 已定项 17「末条再跨记录」、已定项 7 提交标记、已定项 4 读者规则）。
//!
//! - 树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）点名项多于一条记录装得下时，接到带文件那一路同一个切法
//!   （`roles_named_by_each_record_of_the_publish`）：装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志与提交标记，
//!   本次发布内序号依次 1..N（实十六接续报告 `research/prompts/m2-writepath-implementer-report.md` 第六节 Q7：这一路原先只写一条；
//!   点名项是实例表各片加分配记录树这次重写的节点，一条记录装 64 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`），多于 64 项时
//!   往 4096 字节的记录里写第 65 项越界 panic）。产品路径上要片数加重写的节点数多于 64 才走得到（分配记录树只重写一个节点时
//!   是实例表长到 64 片），这里用只供测试的开关
//!   （`PoolWriter::set_journal_record_named_entry_capacity`）把一条记录装的项数压到 1，两个单元就造得出跨记录的写行。
//! - 事务号 0 的发布跨多条记录时，前几条的提交标记写 0；读者读到提交标记 2..=255 当那条记录损坏、断链即止（
//!   与已定项 4 读者规则同一处置），池级 checker 的 I-8.8 判「提交标记字节」违例——两边说的是同一件事。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/multi_record_publishes_of_transaction_zero.rs"]
mod scenarios_of_multi_record_publishes_of_transaction_zero;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_multi_record_publishes_of_transaction_zero::row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it(
) {
    scenarios_of_multi_record_publishes_of_transaction_zero::row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_multi_record_publishes_of_transaction_zero::commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens(
) {
    scenarios_of_multi_record_publishes_of_transaction_zero::commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens(&mut PoolCheckerImages::left_to_the_checker_tier());
}
