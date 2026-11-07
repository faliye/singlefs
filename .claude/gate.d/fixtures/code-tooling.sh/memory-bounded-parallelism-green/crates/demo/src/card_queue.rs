// 样本：按显存排卡上的活，不是按主机内存开进程或线程
pub fn lanes_on_the_card(gpu_memory_total: u64, per_lane: u64) -> u64 {
    let lanes = gpu_memory_total / per_lane;
    lanes
}
