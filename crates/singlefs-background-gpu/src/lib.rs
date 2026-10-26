//! 产品的后台 GPU 路径（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条；用户 2026-10-26 定等 GPU 结果「设超时，超时算出错」）。核心层不依赖本包：挂载时由调用方把本包的实现递进去。
//!
//! 分三层：
//! - 纯逻辑，不要 GPU：哪些适配器算显卡与挂载那一刻的校验（[`adapter_inventory`]）、按每张卡量出的代价分活（[`work_split_by_measured_cost`]）、
//!   显存上限与切批（[`memory_limit_and_chunks`]）、两份核的比对与连续不确定计数（[`two_kernel_reconciliation`]，与哪一段无关）、
//!   校验和核对那一段的打包与回读解码（[`checksum_kernels`]）、区域扫描里单元头解析与映射反查那一段的打包、回读解码与 CPU 参照
//!   （[`region_record_kernels`]）、等 GPU 时的上限与可换的钟（[`waiting_for_the_gpu`]）、
//!   选段打分里每段活槽数那一段的打包、回读解码与 CPU 参照（[`live_slot_kernels`]）、挂载时开着的本机全部独显里此刻照哪组设置用哪几张、
//!   每张怎么切、等多久（[`gpus_of_this_host`]）。
//! - 起 GPU 的外壳（[`background_gpu`]）：枚举 wgpu 适配器、在参与的卡上起设备、量一次提交的主机 CPU 时间、交一批校验和核对、一批区域扫描的记录或一批聚簇段。
//! - 核心层要的那一份实现（[`background_gpu_compute`]）：`singlefs_core::background_compute::BackgroundGpuCompute` 三段各一个方法接到上面三段的两份核，
//!   连续不确定的计数随它活；挂载那一刻开卡、交回核心层收的把手与没开成而跳过的卡（[`background_gpu_compute::open_background_gpu_compute`]）。
//!
//! 与 checker 档的 GPU 判器不共用任何代码（`#[path]`、`include!` 也不用；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ⑧ 条）：
//! 两边共用一段写错的代码时，判器与被判的产品路径会一起错，GPU 对拍照样全绿。
//!
//! unsafe 默认拒：只在 [`background_gpu`] 的两处放开，读 Vulkan 物理设备的内存堆与读这条线程的 CPU 钟，各有 `// SAFETY:`。
#![deny(unsafe_code)]

pub mod adapter_inventory;
pub mod background_gpu;
pub mod background_gpu_compute;
pub mod checksum_kernels;
pub mod gpus_of_this_host;
pub mod live_slot_kernels;
pub mod memory_limit_and_chunks;
pub mod region_record_kernels;
pub mod two_kernel_reconciliation;
pub mod waiting_for_the_gpu;
pub mod work_split_by_measured_cost;
