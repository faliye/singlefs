//! 样本：从已经按登记位取好的每台默认重活线程数再分：参数 heavy_threads 只由登记位读法交进来，registered_threads 由 let 绑定登记位读法
pub fn shares_of_the_process(heavy_threads: usize) -> usize {
    (heavy_threads / 2).max(1)
}

pub fn shares_now() -> usize {
    shares_of_the_process(crate::heavy_threads::default_heavy_threads().get())
}

pub fn half_now() -> usize {
    let registered_threads = crate::heavy_threads::default_heavy_threads().get();
    registered_threads / 2
}
