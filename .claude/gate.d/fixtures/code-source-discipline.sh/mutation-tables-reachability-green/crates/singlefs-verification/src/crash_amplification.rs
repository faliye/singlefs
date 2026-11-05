// 样本里的崩溃放量文件：crates-mutation-rows.py 的选路表、截断表、豁免表点名的函数都在这里（空函数体），
// 所以 static-check 判可达性，不是「一个都没有」的本次未判。counted 是点名的测试调得到的，never_called_by_the_test 没有测试调它。
pub fn counted(value: u64) -> u64 { value + 1 }

pub fn never_called_by_the_test(value: u64) -> u64 { value * 2 }

pub fn judge_not_run_without_the_gpu_feature() {}

pub fn check_blocks_where_until_with_threads() {}

pub fn check_selected_blocks() {}

pub fn run_from_environment_with_observers() {}

fn judged_on_the_cpu_here() {}

fn assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root() {}
