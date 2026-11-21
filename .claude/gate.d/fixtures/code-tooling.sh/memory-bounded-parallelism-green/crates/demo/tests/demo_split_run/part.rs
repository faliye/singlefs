// 样本：目录式测试目标 demo_split_run 的子模块，按空闲内存定开几个工作线程
pub fn start_group(free_gibibytes: usize) -> usize {
    let workers = free_gibibytes / 4;
    workers
}
