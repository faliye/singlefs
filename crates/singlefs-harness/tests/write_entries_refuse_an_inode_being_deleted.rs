//! D8（核心索引结构） 已定项 16 第 4 条「写入口拒 nlink = 0 的 inode」：unlink 的意图那次发布把第一个文件那条记录写成 nlink = 0、
//! size 不变，从那一版起覆盖写与顺序写都要在任何写之前拒（`PublishError::WriteToAnInodeBeingDeleted`），不然现造的记录照 nlink = 1 写回去，
//! 一个正在回收的对象就复活了（`research/prompts/m3-delete-r2-main-verification.md` 第二节第 1 行）。
//!
//! 意图那次发布今天还写不出来（三·2 停在墓碑容器号那一格），「上一版」用新池新建文件那一版的内存态、把第一个文件那条记录的 nlink
//! 改成 0：写入口判的就是调用方交进来的那一版，与它从哪来无关。每条用例比拒绝前后的系统配置槽、根环、录制流步数（`DiskSnapshot`）
//! 与分配器的记录，逐项不变。

mod common;

use common::{
    build_pool, disk_snapshot, parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, PublishError, TransactionOutput,
    FIRST_INODE_NUMBER,
};

/// 新池新建文件那一版，第一个文件那条记录换成删除中的样子（nlink = 0，别的字段不动）。
fn version_whose_first_file_is_being_deleted(output: &TransactionOutput) -> TransactionOutput {
    let mut version = output.clone();
    version
        .inode_record
        .as_mut()
        .expect("新池新建文件那一版有第一个文件那条记录")
        .nlink = NLINK_OF_AN_INODE_BEING_DELETED;
    version
}

#[test]
fn overwrite_of_an_inode_being_deleted_is_refused_before_any_write() {
    let mut pool = build_pool("write-entry-refuses-overwrite-of-a-deleted-inode");
    let previous = version_whose_first_file_is_being_deleted(&pool.output);
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let allocation_records_before = pool.allocator.records().to_vec();
    let instance = previous.root.instance;
    let refusal = publish_overwrite_in_process(
        &mut pool,
        &previous,
        b"overwrite after the unlink intent",
        FIXED_WRITE_TIME_SECONDS + 1,
        instance,
    )
    .expect_err("覆盖写删除中的 inode 要被拒");
    assert!(
        matches!(
            refusal,
            PublishError::WriteToAnInodeBeingDeleted {
                inode: FIRST_INODE_NUMBER
            }
        ),
        "拒的成员说清是删除中的 inode 1：{refusal:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "系统配置槽、根环、录制流步数逐项不变：一个写、一道屏障都没发"
    );
    assert_eq!(
        pool.allocator.records(),
        allocation_records_before.as_slice(),
        "分配器的记录不变"
    );
}

#[test]
fn sequential_write_to_an_inode_being_deleted_is_refused_before_any_write() {
    let mut pool = build_pool("write-entry-refuses-sequential-write-of-a-deleted-inode");
    let previous = version_whose_first_file_is_being_deleted(&pool.output);
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let allocation_records_before = pool.allocator.records().to_vec();
    let instance = previous.root.instance;
    let content = vec![7u8; 70_000];
    let refusal = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            instance,
        )
        .expect_err("顺序写删除中的 inode 要被拒")
    };
    assert!(
        matches!(
            refusal,
            PublishError::WriteToAnInodeBeingDeleted {
                inode: FIRST_INODE_NUMBER
            }
        ),
        "拒的成员说清是删除中的 inode 1：{refusal:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "系统配置槽、根环、录制流步数逐项不变：一个写、一道屏障都没发"
    );
    assert_eq!(
        pool.allocator.records(),
        allocation_records_before.as_slice(),
        "分配器的记录不变"
    );
}
