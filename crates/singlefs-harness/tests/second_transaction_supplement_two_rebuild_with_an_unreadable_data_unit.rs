//! 里程碑「第二个事务」增补 2 收口表第 13 行 C394 三问里的 N2（用户 2026-09-24 定，判决 `research/prompts/m2-newq-r1-main-verification.md`
//! 第三节 N2；D19（块指针的结构与宽度预算） 已定项 5）：从盘上重建上一版时读不出的数据单元照抄它的位置项、不读内容，挂载照常，
//! 读到那个文件时才报错。改之前：一个数据单元两份都读不出，可写挂载在 `recovery::rebuild_version` 上报 `UnitUnreadable`，整个池挂不上
//! （三方攻方腿的原型在 F1 / F2 / F9 三种历史上量到 62 / 120 挂不上）。
//!
//! 镜像：第一个事务那一版（实例 1、txg 3，一个 3000 字节的文件、一个数据单元）上把数据单元两块盘上的那一份都写零——
//! 位置提示与中央映射里那一条指的是同一个槽（今天没有搬迁），两条路读回来的字节整单元校验和都对不上。

mod common;

use common::{
    build_pool, crash_state_devices, memory_pool_of_sparse_devices, parameters, FILE_BYTES,
};
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{mount_read_only, FileReadFailure};
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::transaction::{TransactionUnit, FIRST_INODE_NUMBER};
use singlefs_format::DATA_UNIT_BYTES;
use singlefs_harness::SharedStream;

#[test]
fn data_unit_unreadable_on_both_devices_is_carried_by_its_locations_and_the_writable_mount_goes_on_until_the_file_is_read(
) {
    let pool = build_pool("n2-rebuild-with-an-unreadable-data-unit");
    let data_pointer = pool.output.data_pointers[0];
    let data_slot = data_pointer.locations[0].slot;
    assert_eq!(
        data_pointer.locations[1].slot, data_slot,
        "第一版两条位置条目同槽"
    );
    let mut image = pool.memory_pool();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.devices.get_mut(&device).expect("池里有这块盘").write(
            data_slot.to_device_offset(),
            &vec![0u8; usize::try_from(DATA_UNIT_BYTES).expect("32768")],
        );
    }

    // 重建那一步自己：做成，位置项照抄，内容不读（这一项的字节是空的）。
    let rebuilt = rebuild_version(&image, &pool.output.root, Some(pool.output.record.clone()))
        .unwrap_or_else(|failure| {
            panic!("数据单元两份都读不出：重建要照抄它的位置项、照常做成，实际 {failure:?}")
        });
    let RebuiltVersion::WithFile(previous) = rebuilt else {
        panic!("第一个事务那一版带文件");
    };
    assert_eq!(
        previous.data_pointers,
        vec![data_pointer],
        "读不出的那个数据单元的位置项照抄"
    );
    let carried = previous.unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    assert_eq!(carried.slot, data_slot, "那一项的落点照抄位置项");
    assert!(
        carried.bytes.is_empty(),
        "不读内容：那一项不带字节（照抄它的发布不写它）"
    );

    // 可写挂载照常：取号 2、写行、暖机，新实例这一版照抄那个数据单元的位置项。
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|failure| panic!("挂载照常，实际 {failure:?}"));
    let current = mounted
        .current
        .into_file_version()
        .expect("挂载之后现行那一版带文件");
    assert_eq!(current.root.instance, InstanceGeneration(2), "取到号 2");
    assert_eq!(
        current.data_pointers,
        vec![data_pointer],
        "新实例这一版照抄那个数据单元的位置项"
    );
    let after_the_mount = memory_pool_of_sparse_devices(&devices);

    // 读到那个文件时才报错：冷走读走到数据单元那一步，提示与映射都读不出。
    let report = recover(&after_the_mount, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((current.root.instance, current.root.checkpoint_txg)),
            failure: RecoveryFailure::MappingStillUnreadable { slot: data_slot },
        },
        "冷走读：挂载之后的最新根下面，读到那个数据单元才报错"
    );
    // 只读挂载打开得了、文件打开得了，读的那一刻报错。
    let mounted_read_only = mount_read_only(&after_the_mount)
        .unwrap_or_else(|failure| panic!("只读挂载打开得了，实际 {failure:?}"));
    let file = mounted_read_only
        .mounted
        .open_file(&after_the_mount, InodeNumber(FIRST_INODE_NUMBER))
        .expect("文件打开得了");
    assert_eq!(
        file.read_at(
            &after_the_mount,
            FileOffsetInBytes(0),
            u64::try_from(FILE_BYTES).expect("3000"),
        )
        .err(),
        Some(FileReadFailure::DataUnitChecksumMismatchEverywhere {
            unit_index_in_file: DataUnitIndexInFile::FIRST,
            hinted_slot: data_slot,
        }),
        "读那个文件的那一刻报错"
    );
}
