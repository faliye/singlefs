//! checker 档（D13（验证路线） 已定项 15）：崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、
//! 崩溃注入（`crash_injection`，可写挂载的分段与崩溃镜像落池在 `writable_mount_phases`）与坏盘输入（`bad_disk_input`）两场战役、真设备一侧的设备日志比对（`device_log`）与运行模式（`on_device_modes`），
//! 崩溃放量（里程碑二「增补 4」）：崩溃点的身份与复用键（`crash_identity`）、先录后核的流水线（`crash_amplification`：崩溃点计划、路径编号、
//! 按块录入与核对、按权重分工、对账、门禁读一次）、判定存储（`verdict_store`，RocksDB，特性 `verdict-store`）、
//! 单元两道校验和上 GPU 与 CPU 对拍（`gpu_unit_checks`，特性 `gpu`）、崩溃放量取卡与 GPU 用例的卡池共用的显卡配置读法（`gpu_card_configuration`，`gpu` 与 `verdict-store` 开一个就编）、GPU 判器的判法（`crash_judge_gpu`）与它的派活（`crash_judge_dispatch`，不在判法闭包里）、
//! 两个判器共用、不挂特性的结论写法（`crash_judge_conclusions`：红项编码、恢复字与失败点码表、计数向量）。
//! 随机历史两项的给定状态表、它的判器 trait 与 CPU 就地判（`crash_given_states`，不引随机历史：流水线引它，层 0 用例的判法闭包里因此没有崩溃注入；崩溃注入照旧转出）。
//! 脚手架（内存池、录制器、理想模型、随机历史）在 `singlefs_harness`；池级 checker 判决器在 `singlefs_checker`。
#![forbid(unsafe_code)]

pub mod bad_disk_input;
pub mod crash;
#[cfg(feature = "verdict-store")]
pub mod crash_amplification;
pub mod crash_facts;
pub mod crash_given_states;
pub mod crash_identity;
pub mod crash_injection;
pub mod crash_judge_conclusions;
#[cfg(feature = "gpu")]
pub mod crash_judge_dispatch;
#[cfg(feature = "gpu")]
pub mod crash_judge_gpu;
pub mod crash_judge_tables;
#[cfg(feature = "gpu")]
pub mod crash_verify_gpu;
pub mod device_log;
#[cfg(any(feature = "gpu", feature = "verdict-store"))]
pub mod gpu_card_configuration;
#[cfg(feature = "gpu")]
pub mod gpu_unit_checks;
pub mod layer0_progress;
pub mod on_device_modes;
#[cfg(feature = "verdict-store")]
pub mod verdict_store;
pub mod writable_mount_phases;
