//! 核心层：地址 newtype、块设备抽象（步 0），字节编解码、校验和、指针、单元头、系统配置、根记录、根环几何与 mkfs（步 1），
//! 分配器与数据单元（步 2），树里的记录（步 3）、中央映射条目（步 4）、journal 记录与事务层（步 5），冷启动恢复（步 6）。
//!
//! 不同地址空间必须是不同的 Rust 类型，混用要编译不过（`.claude/rules/fs-design.md`）。
//! 块设备（`block_device::BlockDevice`）改介质与守次序的动作只有五个：读、写（`write_at`，返回前要不要持久由它的
//! `WriteDurability` 参数定，FUA 写是 `ForceUnitAccess` 那一种、不是另一个方法）、写零（`write_zeroes_at`，没有持久性参数，
//! 持久靠后面的屏障）、discard（`discard_at`，交回 `DiscardOutcome`：落成了，或后端不认、一个字节没动）、屏障；另有只答问题、
//! 不碰介质的几个（物理块大小、设备字节数、journal 记录槽的提示）与给记着这次挂载读过什么的那一层改写的重读方法，全表在那个特质的文档里。
//! 不暴露介质是什么（D17（实现分层与第三方管道） 已定项 5：录制钩子挂在这层接口上，不挂在操作 API 上）。
//!
//! unsafe 默认拒：产品构建里全 crate 只在两个函数上放开。文件后端打洞那一个（`block_device::punch_hole_keeping_the_file_length`）
//! 调 `fallocate(2)`，std 没有打洞的安全封装（C603（测试与验证的资源浪费没有自检））；CRC-32C 走点名那条路的那一个
//! （`checksum::crc32_castagnoli_on_route`）在 x86_64 上调带 `#[target_feature(enable = "sse4.2")]` 的硬件 `crc32` 指令那一段，
//! std 没有「查过特性再调」的安全封装。
//! Verus 构建里另有一种放开：被验区域里每条 `singlefs-format` 常量的 `assume_specification` 各带一个 `allow(unsafe_code)`，
//! 因为 Verus 把它展开成一个 unsafe 的 fn 声明；产品构建把整条擦掉，不留 unsafe。这种规格写在哪、怎么登记、跨文件怎么核，
//! 见 `verified_model` 模块「singlefs-format 常量的规格」一段。
#![deny(unsafe_code)]

pub mod address;
pub mod admission;
pub mod allocation_record_tree;
pub mod allocator;
pub mod background_compute;
pub mod block_device;
pub mod bytes;
pub mod checksum;
pub mod code_two_tree;
pub mod compaction_plan;
pub mod container_tree;
pub mod deleted_inodes;
pub mod extent_tree;
pub mod inode_tree;
pub mod instance_table;
pub mod instance_table_sweep;
pub mod journal;
pub mod lineage_tree;
pub mod logged_operations;
pub mod make_filesystem;
pub mod mount;
pub mod mounted_read;
pub mod mounted_session;
pub mod pointer;
pub mod records;
pub mod recovery;
pub mod region_scan;
pub mod retirement_tree;
pub mod root_record;
pub mod root_ring;
pub mod system_configuration;
pub mod system_configuration_repair;
pub mod tombstone_tree;
pub mod transaction;
pub mod truncate;
pub mod unit;
pub mod unlink;
pub mod verified_model;
pub mod write_accounting;
pub mod write_request_split;
