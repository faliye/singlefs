use singlefs_background_gpu::reclaim_dispatch;
pub mod gpu_unit_checks;
pub fn engine_sample() -> u32 { singlefs_harness::recorder_sample() }
pub fn judged_dispatch() -> u64 { reclaim_dispatch() }
