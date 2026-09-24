//! 核心层：地址 newtype、块设备抽象（步 0），字节编解码、校验和、指针、单元头、系统配置、根记录、根环几何与 mkfs（步 1），
//! 分配器与数据单元（步 2），树里的记录（步 3）、中央映射条目（步 4）、journal 记录与事务层（步 5），冷启动恢复（步 6）。
//!
//! 不同地址空间必须是不同的 Rust 类型，混用要编译不过（`.claude/rules/fs-design.md`）。
//! 块设备只暴露读、写、屏障、FUA 写、探测 `physical_block_size` 五个动作，不暴露介质是什么
//! （D17（实现分层与第三方管道） 已定项 5：录制钩子挂在这层接口上，不挂在操作 API 上）。
#![forbid(unsafe_code)]

pub mod address;
pub mod admission;
pub mod allocator;
pub mod block_device;
pub mod bytes;
pub mod checksum;
pub mod inode_tree;
pub mod instance_table;
pub mod journal;
pub mod make_filesystem;
pub mod mount;
pub mod mounted_read;
pub mod pointer;
pub mod records;
pub mod recovery;
pub mod root_record;
pub mod root_ring;
pub mod system_configuration;
pub mod transaction;
pub mod unit;
pub mod write_accounting;
pub mod write_request_split;
