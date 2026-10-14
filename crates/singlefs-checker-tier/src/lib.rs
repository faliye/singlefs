//! checker 档（D13（验证路线） 已定项 15）：崩溃态枚举引擎与记录核对器（`crash`）、崩溃点重放的工具链身份与发现日志（`crash_point_replay_progress`）、
//! 崩溃注入（`crash_injection`，可写挂载的分段与崩溃镜像落池在 `writable_mount_phases`）与坏盘输入（`bad_disk_input`）两场战役、真设备一侧的设备日志比对（`device_log`）与运行模式（`on_device_modes`），
//! 崩溃放量（里程碑二「增补 4」）：崩溃点的身份与复用键（`crash_identity`）、先录后核的流水线（`crash_amplification`：崩溃点计划、路径编号、
//! 按块录入与核对、按权重分工、对账、门禁读一次）、判定存储（`verdict_store`，RocksDB，特性 `verdict-store`）、
//! 单元两道校验和上 GPU 与 CPU 对拍（`gpu_unit_checks`，特性 `gpu`）、崩溃放量取卡与 GPU 用例的卡池共用的显卡配置读法（`gpu_card_configuration`，`gpu` 与 `verdict-store` 开一个就编）、GPU 判器的判法（`crash_judge_gpu`）与它的派活（`crash_judge_dispatch`，不在判法闭包里）、
//! 崩溃判定库（`crash_library`：几类测试各一份长期存放、按键复用的库，录入段与判段只经它交接；verdict-store 因此是默认特性）、
//! 崩溃点重放枚举用例的两段入口（`crash_state_enumeration`：录入段写流进库，判段读回、按块上卡或 CPU 判、存结论、复用，崩溃点重放计数从库里重建）、
//! 两个判器共用、不挂特性的结论写法（`crash_judge_conclusions`：红项编码、恢复字与失败点码表、计数向量）。
//! 随机历史两项的给定状态表、它的判器 trait 与 CPU 判法（`crash_given_states`，不引随机历史：流水线引它，崩溃枚举用例的判法闭包里因此没有崩溃注入；崩溃注入照旧转出）。
//! 脚手架（内存池、录制器、理想模型、随机历史）在 `singlefs_harness`；池级 checker 判决器在 `singlefs_checker`。
#![forbid(unsafe_code)]

// 崩溃点重放枚举只吃从崩溃判定库里读回的流、结论存进库（`crash_library`、`crash_state_enumeration`），判定库（RocksDB）因此是 checker 档必带的：关掉 verdict-store 编不过。
#[cfg(not(feature = "verdict-store"))]
compile_error!("checker 档的崩溃点重放枚举要经崩溃判定库（crash_library）：verdict-store 是默认特性，不要用 --no-default-features 关掉它");

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
pub mod crash_library;
pub mod crash_point_replay_progress;
pub mod crash_state_enumeration;
#[cfg(feature = "gpu")]
pub mod crash_verify_gpu;
pub mod device_log;
#[cfg(any(feature = "gpu", feature = "verdict-store"))]
pub mod gpu_card_configuration;
#[cfg(feature = "gpu")]
pub mod gpu_unit_checks;
pub mod on_device_modes;
#[cfg(feature = "verdict-store")]
pub mod verdict_store;
pub mod writable_mount_phases;
