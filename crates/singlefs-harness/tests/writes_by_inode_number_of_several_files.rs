//! 按 inode 号写数据（里程碑「删除与多次 COW」一·1，用户 2026-10-18 定「三条都新写，先给 core 加多 inode 写数据」）的验收：
//! 会话的 `UserChange::OverwriteOfAnInode`、`UserChange::SequentialWriteOfAnInode` 与它们落到的
//! `transaction::publish_overwrite_of_an_inode`、`transaction::publish_sequential_write_of_an_inode`。
//!
//! 四件事各一条用例：两个 inode 交错各写几次、中间关掉再可写挂载两次，读回各自逐字节等于最后写的；写一个 inode 那次发布不碰别的文件的
//! 数据单元与下段（盘上那几个槽的字节一个不变、它们不在这次换下的落点里）；inode 号落在上段第一片叶之外时上段长一层，重挂之后每个文件都读得回；
//! 对不在 inode 树里的号写在任何写之前拒成 `WriteToAnInodeThatIsNotInTheTree`。另一条钉住第一个文件之外有文件带数据单元时删那个文件、
//! 再删第一个文件都做成（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M3；此前拒成「条款没写」）。
//! 每一版都交池级 checker 判一遍：按 inode 号写出来的镜像一条违例都不许有。
//! 代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 2 条补的四条（正推腿 `m3-close-w5-code-r1-sonnet-output.md`
//! W1、W5 两节列的缺口）：别的号 nlink = 0 时写入口拒（缺口 A）；第一个文件删完、上段只剩零条目根之后写删之前就有的号，零条目根换下（缺口 G）；
//! 别的号的数据单元与下段节点按结构种类记的写与第一个文件同一套（新加的穷举分支，`write_accounting.rs`）；空间准入数别的号的单元与第一个文件同一套
//! （新加的穷举分支，`admission.rs`）。

//! 本档只核实现行为；同名场景交出的镜像由 checker 档给定状态引擎两段判。

mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/writes_by_inode_number_of_several_files.rs"]
mod scenarios;

#[test]
fn two_inodes_overwritten_in_turn_across_two_writable_remounts_each_read_back_what_was_last_written_to_it(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::two_inodes_overwritten_in_turn_across_two_writable_remounts_each_read_back_what_was_last_written_to_it);
}

#[test]
fn writing_one_inode_leaves_the_data_units_and_extent_lower_nodes_of_the_other_files_untouched_on_disk(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::writing_one_inode_leaves_the_data_units_and_extent_lower_nodes_of_the_other_files_untouched_on_disk);
}

#[test]
fn a_write_to_an_inode_past_the_first_upper_leaf_grows_the_upper_segment_and_every_file_reads_back_after_a_remount(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::a_write_to_an_inode_past_the_first_upper_leaf_grows_the_upper_segment_and_every_file_reads_back_after_a_remount);
}

#[test]
fn a_write_to_an_inode_number_that_is_not_in_the_inode_tree_is_refused_before_anything_reaches_the_disk(
) {
    scenarios::a_write_to_an_inode_number_that_is_not_in_the_inode_tree_is_refused_before_anything_reaches_the_disk();
}

#[test]
fn unlink_while_a_file_other_than_the_first_holds_data_units_deletes_it_and_then_the_first_file() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::unlink_while_a_file_other_than_the_first_holds_data_units_deletes_it_and_then_the_first_file);
}

#[test]
fn a_data_write_to_an_inode_created_after_the_first_file_was_deleted_is_refused_before_anything_reaches_the_disk(
) {
    scenarios::a_data_write_to_an_inode_created_after_the_first_file_was_deleted_is_refused_before_anything_reaches_the_disk();
}

#[test]
fn a_write_to_another_inode_being_deleted_is_refused_by_both_entries_before_anything_reaches_the_disk(
) {
    scenarios::a_write_to_another_inode_being_deleted_is_refused_by_both_entries_before_anything_reaches_the_disk();
}

#[test]
fn writing_a_file_after_the_first_file_emptied_the_upper_segment_replaces_the_zero_entry_root() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::writing_a_file_after_the_first_file_emptied_the_upper_segment_replaces_the_zero_entry_root);
}

#[test]
fn the_writes_by_structure_kind_of_another_inode_are_counted_as_the_first_files_are() {
    scenarios::the_writes_by_structure_kind_of_another_inode_are_counted_as_the_first_files_are();
}

#[test]
fn the_space_admission_counts_the_units_of_another_inode_as_it_counts_the_first_files() {
    scenarios::the_space_admission_counts_the_units_of_another_inode_as_it_counts_the_first_files();
}
