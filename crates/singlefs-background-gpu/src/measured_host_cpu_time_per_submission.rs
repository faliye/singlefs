//! 挂载时在每张卡上量出的一次提交的主机 CPU 时间（D26（后台整理与放置回收） 已定项 13 第 2 条）。
//!
//! 分活是「按卡的顺序填满」（`crate::gpus_of_this_host::plans_of_a_batch_filling_the_gpus_in_order`）：
//! 量出的这个数照量、照报（`crate::background_gpu::ParticipatingGpuReport`），不用来分活。

/// 一次提交在主机上占的 CPU 时间（这条线程的 CPU 钟，不是挂钟），纳秒。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HostCpuNanosecondsPerSubmission(pub u64);
