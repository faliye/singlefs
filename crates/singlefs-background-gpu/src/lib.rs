//! 产品的后台 GPU 路径（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条；用户 2026-10-26 定等 GPU 结果「设超时，超时算出错」）。核心层不依赖本包：挂载时由调用方把本包的实现递进去。
//!
//! 分三层：
//! - 纯逻辑，不要 GPU：哪些适配器算显卡与挂载那一刻的校验（[`adapter_inventory`]）、挂载时每张卡量出的一次提交的主机 CPU 时间（只量、只报，不用来分活，
//!   [`measured_host_cpu_time_per_submission`]）、
//!   显存上限与切批（[`memory_limit_and_chunks`]）、两份核的比对与连续不确定计数（[`two_kernel_reconciliation`]，与哪一段无关）、
//!   校验和核对那一段的打包与回读解码（[`checksum_kernels`]）、区域扫描里单元头解析与映射反查那一段的打包、回读解码与 CPU 参照
//!   （[`region_record_kernels`]）、等 GPU 时的上限与可换的钟（[`waiting_for_the_gpu`]）、
//!   选段打分里每段活槽数那一段的打包、回读解码与 CPU 参照（[`live_slot_kernels`]）、挂载时开着的本机全部独显里此刻照哪组设置用哪几张、
//!   每张怎么切、等多久、一批按卡的顺序填满（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡，用户 2026-11-02 定；[`gpus_of_this_host`]）。
//! - 起 GPU 的外壳（[`background_gpu`]）：枚举 wgpu 适配器、在参与的卡上起设备、量一次提交的主机 CPU 时间、交一批校验和核对、一批区域扫描的记录或一批聚簇段。
//! - 核心层要的那一份实现（[`background_gpu_compute`]）：`singlefs_core::background_compute::BackgroundGpuCompute` 三段各一个方法接到上面三段的两份核，
//!   连续不确定的计数随它活；挂载那一刻开卡、交回核心层收的把手与没开成而跳过的卡（[`background_gpu_compute::open_background_gpu_compute`]）。
//!
//! 与验证包的 GPU 判器不共用任何代码（`#[path]`、`include!` 也不用；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ⑧ 条）：
//! 两边共用一段写错的代码时，判器与被判的产品路径会一起错，GPU 对拍照样全绿。
//!
//! unsafe 默认拒：只在五处放开，[`background_gpu`] 里四处（读 Vulkan 物理设备的内存堆、读这条线程的 CPU 钟、
//! 读不受 NTP 调频的单调钟 `CLOCK_MONOTONIC_RAW`（[`background_gpu::monotonic_raw_time_now`]），与它的测试模块里在用例自己起的子进程中
//! 压本进程的地址空间上限，让 noop 后端真建不成缓冲），加真卡测试 `tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs` 读显卡空闲显存那一处；
//! 各有 `// SAFETY:`，逐处登记在 `.claude/gate.d/unsafe-allowances.tsv`。
#![deny(unsafe_code)]

pub mod adapter_inventory;
pub mod background_gpu;
pub mod background_gpu_compute;
pub mod checksum_kernels;
pub mod gpus_of_this_host;
pub mod live_slot_kernels;
pub mod measured_host_cpu_time_per_submission;
pub mod memory_limit_and_chunks;
pub mod region_record_kernels;
pub mod two_kernel_reconciliation;
pub mod waiting_for_the_gpu;
