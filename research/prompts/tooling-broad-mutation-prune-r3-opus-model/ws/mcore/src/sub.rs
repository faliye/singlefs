//! X1：子进程里才执行的函数（形同 checker 档 crash_enumeration_sharded_across_processes.rs 的 golden 用例）。
pub fn score(i: u64) -> u64 {
    i * 7 + 3
}

/// X1b：子进程跑完打印之后 abort（模拟崩溃的子进程），它的覆盖写不出来。
pub fn score_b(i: u64) -> u64 {
    i * 5 + 2
}

/// X1c：父测试先后起两个子进程，这个函数只在第一个子进程里执行。
pub fn score_c(i: u64) -> u64 {
    i * 9 + 4
}
