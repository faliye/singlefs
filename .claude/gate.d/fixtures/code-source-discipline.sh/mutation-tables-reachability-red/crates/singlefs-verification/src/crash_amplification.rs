// 样本里的崩溃放量模块：入口文件只放 counted 与 never_called_by_the_test，crates-mutation-rows.py 的选路表、截断表、豁免表点名的函数拆在 crash_amplification/ 下的子模块文件里（空函数体），
// 函数落在表里点名的那份文件上，所以 static-check 判可达性，不是「一个都没有」的本次未判。counted 是点名的测试调得到的，never_called_by_the_test 没有测试调它。
pub mod cpu_checking;
pub mod crash_point_plan;
pub mod given_states_engine;
pub mod run_from_the_environment;
pub mod run_modes_and_cards;

pub fn counted(value: u64) -> u64 { value + 1 }

pub fn never_called_by_the_test(value: u64) -> u64 { value * 2 }
