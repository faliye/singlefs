//! 由记录重建那次发布的根时，卸载记号从记录的新根段来、不照抄所选根（D23（journal 的角色与格式） 已定项 15：施加一条记录 = 把所选根的七个字段换成
//! 记录里的，卸载记号是其中一项；`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一 RR2，正推腿 H2 的补洞草稿）。
//!
//! 盘面：新池新建文件之后覆盖写三版（txg 4、5、6），正常卸载推两次带卸载记号的空发布（txg 7、8），两次的根槽都撕掉、两条记录留着。
//! 择根落到 txg 6（不带卸载记号），施加 txg 7、8 两条记录：重建出来的根与 txg 8 当初写的逐字段相同，带卸载记号。
//! 只撕 txg 8 一槽时所选根 txg 7 本身就带记号，照抄所选根也对，分不出两种取法；两槽都撕才分得出。
mod common;

use common::{
    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B,
};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::{unmount, ShadowLedger, Unmounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::root_record::UnmountMarker;
use singlefs_core::transaction::PoolVersion;

/// 场景：上面那份盘面只读挂载。
/// 预期：所选根是 txg 6、不带卸载记号；恢复施加了两条记录，生效的根与卸载那一串第二次空发布当初写的根逐字段相同（带卸载记号）。
#[test]
fn a_root_rebuilt_from_both_records_of_the_unmount_sequence_takes_the_unmount_marker_from_the_record(
) {
    let mut pool = build_pool("unmount-marker-of-a-root-rebuilt-from-the-records");
    // 迭代上界：三次覆盖写（txg 4、5、6）。
    for fill_byte in [0xB0u8, 0xC0, 0xD0] {
        overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B],
        );
    }
    let mut current = PoolVersion::WithFile(pool.output.clone());
    let unmounted = unmount(
        &parameters(),
        pool.devices.as_mut().expect("镜像开着"),
        &mut pool.allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载");
    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        panic!("现行那一版带文件，卸载推抬 F 的空发布");
    };
    let [first_unmount_publish, second_unmount_publish] =
        <[_; 2]>::try_from(raised.publishes).expect("txg 6 除以 3 余 0：卸载推两次空发布");
    assert_eq!(
        (
            first_unmount_publish.root.checkpoint_txg,
            second_unmount_publish.root.checkpoint_txg
        ),
        (CheckpointTxg(7), CheckpointTxg(8))
    );
    assert_eq!(
        second_unmount_publish.root.unmount_marker,
        UnmountMarker::WrittenByTheUnmountSequence
    );
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 7);
    tear_the_root_slot_written_for_txg(&mut image, 8);
    let mounted = mount_read_only(&image).expect("只读挂载");
    assert_eq!(
        (
            mounted.chosen_root.checkpoint_txg,
            mounted.chosen_root.unmount_marker
        ),
        (
            CheckpointTxg(6),
            UnmountMarker::NotWrittenByTheUnmountSequence
        ),
        "所选根是 txg 6，不带卸载记号"
    );
    assert_eq!(
        mounted.journal.prefix_applied, 2,
        "施加了卸载那一串的两条记录"
    );
    assert_eq!(
        mounted.effective_root, second_unmount_publish.root,
        "重建出来的根与 txg 8 当初写的逐字段相同：卸载记号从记录的新根段来"
    );
}
