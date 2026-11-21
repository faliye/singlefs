//! 样本：再分的那个数不全来自登记位：参数 heavy_threads 有一处调用交写死的 8，worker_threads 由 let 绑定一个参数
pub fn shares_of_the_process(heavy_threads: usize) -> usize {
    (heavy_threads / 2).max(1)
}

pub fn shares_now() -> usize {
    shares_of_the_process(crate::heavy_threads::default_heavy_threads().get())
}

pub fn shares_of_eight() -> usize {
    shares_of_the_process(8)
}

pub fn half_of(given: usize) -> usize {
    let worker_threads = given;
    worker_threads / 2
}
