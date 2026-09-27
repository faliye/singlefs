//! checker 档（D13（验证路线） 已定项 15）：崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、
//! 崩溃注入（`crash_injection`）与坏盘输入（`bad_disk_input`）两场战役、真设备一侧的设备日志比对（`device_log`）与运行模式（`on_device_modes`）。
//! 脚手架（内存池、录制器、理想模型、随机历史）在 `singlefs_harness`；池级 checker 判决器在 `singlefs_checker`。
#![forbid(unsafe_code)]

pub mod bad_disk_input;
pub mod crash;
pub mod crash_injection;
pub mod device_log;
pub mod layer0_progress;
pub mod on_device_modes;
