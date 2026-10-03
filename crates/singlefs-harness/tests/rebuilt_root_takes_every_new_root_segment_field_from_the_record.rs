//! 由 journal 记录重建的根（`recovery::replay_journal` 拼新根那一截）：所选根的七个字段换成被施加的末条记录新根段里的——
//! 树表指针、中央映射树根指针、树 ID 水位、F，加上实例表指针、分配记录树根指针、卸载记号（D23（journal 的角色与格式） 已定项 15，
//! 用户 2026-10-02「回到甲」；`research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行）。只有 fsid 照所选根。
//!
//! 「记录已持久、根槽没持久」的两种发布各一条（根槽改坏：只留记录与单元），施加出来的根与那次发布当初写的根逐字段相同：
//! - 首个文件：树表 0 条的一版写过行（分配记录树根住根记录），第一个文件版本把分配记录树挪进树表、根记录那一项写空；
//!   照抄所选根时施加出来的根还指着那棵旧分配记录树（第二轮攻方 G1：差在 `allocation_record_tree_root`）。
//! - 卸载：正常卸载那一串写的根带卸载记号；照抄时施加出来的根不带（第二轮攻方 G1：差在 `unmount_marker`）。
//!
//! 实例表指针那一项今天没有一种被施加的发布改它（改它的只有写行那次，那是新实例的第一次发布、从不施加；改它的专用清扫发布还没实现），
//! 施加处那一判由 `replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` 用改过新根段的记录钉。
mod common;

use common::{
    build_pool, format_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::pointer::NodePointer;
use singlefs_core::root_record::UnmountMarker;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};

/// 新池新建文件（txg 3）之后覆盖写三版（txg 4、5、6）的填充字节：现行 txg 6 除以 3 余 0，卸载那一串推两次空发布（txg 7、8）。
const FILL_BYTES_OF_THE_THREE_OVERWRITES: [u8; 3] = [0xB0, 0xC0, 0xD0];

/// 第一个文件版本的内容：一个数据单元装得下。
const FIRST_FILE_BYTES: usize = 3000;

/// 只做过 mkfs 的池：第一次可写挂载（取号 1、不写行，暖机 txg 1、2）→ 第二次可写挂载（取号 2，写行 txg 3 把分配记录树根写进根记录，
/// 暖机 txg 4：写行与它两条根已经覆盖两块盘）→ 同一次挂载里发第一个文件版本（txg 5）。它的根槽没落、记录落了：只读挂载择到 txg 4 那条
/// （分配记录树根不空），施加第一个文件版本那条记录；施加出来的根与那次写的根逐字段相同——分配记录树根是空的。
#[test]
fn a_root_rebuilt_from_a_first_file_record_over_a_version_that_wrote_a_row_takes_the_empty_allocation_record_tree_root_from_the_record(
) {
    let mut formatted = format_pool("rebuilt-root-first-file-after-a-row");
    let mut first_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first_mount_devices)
        .expect("第一次可写挂载：取号 1、不写行");
    formatted.devices = Some(first_mount_devices);
    let mut second_mount_devices = formatted.reopen_recorded();
    let mut mounted =
        mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载：写行");
    formatted.devices = Some(second_mount_devices);
    let (root_without_file, record_bytes_without_file) = match &mounted.current {
        PoolVersion::WithoutFile(version_without_file) => (
            version_without_file.root,
            version_without_file.record_bytes.clone(),
        ),
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {
            panic!("树表 0 条的一版上写行与暖机，现行那一版仍没有文件")
        }
    };
    assert_ne!(
        root_without_file.allocation_record_tree_root,
        NodePointer::empty_root(),
        "写过行的一版：分配记录树根住根记录、不空"
    );
    let publish_parameters = parameters();
    let first_file = {
        let devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root_without_file,
            FirstFile {
                content: &vec![0x5A; FIRST_FILE_BYTES],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            mounted.output.instance,
            &record_bytes_without_file,
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (
            first_file.root.instance,
            first_file.root.allocation_record_tree_root,
            first_file.record.new_allocation_record_tree_root
        ),
        (
            InstanceGeneration(2),
            NodePointer::empty_root(),
            NodePointer::empty_root()
        ),
        "第一个文件版本的根与它那条记录的新根段：分配记录树根都是空的（分配记录树挪进树表）"
    );

    let mut image = formatted.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, first_file.root.checkpoint_txg.0);
    let read_only = mount_read_only(&image).expect("只读挂载");
    assert_eq!(
        read_only.chosen_root, root_without_file,
        "第一个文件版本的根槽没落：所选根是它施加在其上的那一版"
    );
    assert_eq!(
        (
            read_only.journal.above_water,
            read_only.journal.prefix_applied
        ),
        (1, 1),
        "水位之上只有第一个文件版本那条记录，施加"
    );
    assert_eq!(
        read_only.effective_root, first_file.root,
        "施加出来的根与第一个文件版本那次写的根逐字段相同（分配记录树根取记录里的空指针，不照抄所选根的）"
    );
}

/// 卸载那一串第二次空发布（txg 8）的根槽没落、记录落了：只读挂载择到第一次（txg 7）那条，施加 txg 8 那条记录。
/// 施加出来的根与 txg 8 那次写的根逐字段相同：带卸载记号（取记录新根段里的，不是「由记录施加的一律不带」）。
#[test]
fn a_root_rebuilt_from_the_second_unmount_publish_record_takes_the_unmount_marker_from_the_record()
{
    let mut pool = build_pool("rebuilt-root-unmount-marker-from-the-record");
    for fill_byte in FILL_BYTES_OF_THE_THREE_OVERWRITES {
        overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B],
        );
    }
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(6),
        "新池新建文件 txg 3，覆盖写三版到 txg 6"
    );
    let mut current = PoolVersion::WithFile(pool.output.clone());
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let unmounted = unmount(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载");
    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」");
    };
    let [first_unmount_publish, second_unmount_publish] =
        <[_; 2]>::try_from(raised.publishes).expect("现行 txg 6 除以 3 余 0：卸载那一串推两次");
    assert_eq!(
        (
            first_unmount_publish.root.checkpoint_txg,
            second_unmount_publish.root.checkpoint_txg,
            second_unmount_publish.root.unmount_marker,
            second_unmount_publish.record.new_unmount_marker
        ),
        (
            CheckpointTxg(7),
            CheckpointTxg(8),
            UnmountMarker::WrittenByTheUnmountSequence,
            UnmountMarker::WrittenByTheUnmountSequence
        ),
        "卸载那一串两次空发布 txg 7、8：txg 8 的根带卸载记号，它那条记录的新根段也带"
    );

    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 8);
    let read_only = mount_read_only(&image).expect("只读挂载");
    assert_eq!(
        read_only.chosen_root, first_unmount_publish.root,
        "txg 8 的根槽没落：所选根是卸载那一串第一次空发布写的那条"
    );
    assert_eq!(
        (
            read_only.journal.above_water,
            read_only.journal.prefix_applied
        ),
        (1, 1),
        "水位之上只有 txg 8 那条记录，施加"
    );
    assert_eq!(
        read_only.effective_root, second_unmount_publish.root,
        "施加出来的根与 txg 8 那次写的根逐字段相同（卸载记号取记录里的）"
    );
}
