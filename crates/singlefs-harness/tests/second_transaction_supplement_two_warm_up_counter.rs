//! 里程碑「第二个事务」增补 2 第 21 行（C366（暖机路径把 txg 写进计数器与 tail））：暖机的空记录按记录号接着数 jsn，系统配置的 tail 存的是
//! jsn 计数器（D23（journal 的角色与格式） 已定项 18），checkpoint_txg 照格式常量走 1、2，两个量各走各的。
//!
//! 今天只有 mkfs 之后第一次可写挂载走暖机，环是空的，txg 与 jsn 按构造相等（1、2），在这条路上分不出「计数器写成 txg」；
//! 欠账表那一格（暖机 txg 1 的记录落盘、根没落 → 崩 → 再挂载）在可写挂载里取号之前就被拒（树表 0 条的一版上要写实例 1 的行），
//! 走不到暖机。这里在函数这一层给不相等的起点：环里最后一条记录的 jsn 是 40 时，暖机写 jsn 41、42，txg 仍是 1、2。

mod common;

use common::{parameters, IMAGE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::recovery::{choose_superblock, PoolReader};
use singlefs_core::transaction::{acquire_instance, warm_up_after_journal_counter, PoolWriter};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::crash::SparseBlockDevice;

/// 环里最后一条记录的 jsn：与暖机的 txg 1、2 都不相等，计数器写成 txg 就分得出来。
const LAST_JOURNAL_COUNTER_BEFORE_WARM_UP: u64 = 40;

#[test]
fn warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two(
) {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let warmed = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        warm_up_after_journal_counter(
            &mut pool,
            &genesis.root,
            instance,
            LAST_JOURNAL_COUNTER_BEFORE_WARM_UP,
        )
        .expect("暖机")
    };

    let txg_and_counter: Vec<(CheckpointTxg, u64)> = warmed
        .records
        .iter()
        .map(|record| (record.checkpoint_txg, record.counter))
        .collect();
    assert_eq!(
        txg_and_counter,
        vec![(CheckpointTxg(1), 41), (CheckpointTxg(2), 42)],
        "jsn 接着前缀末 40 数（41、42），txg 照格式常量走 1、2"
    );
    let root_txgs: Vec<CheckpointTxg> = warmed
        .roots
        .iter()
        .map(|root| root.checkpoint_txg)
        .collect();
    assert_eq!(root_txgs, vec![CheckpointTxg(1), CheckpointTxg(2)]);

    let superblock = choose_superblock(&devices).expect("暖机之后超级块自证得过");
    assert_eq!(
        superblock.journal_tail, 42,
        "系统配置的 tail 存 jsn 计数器（最后一条空记录的 42），不是 txg 2"
    );

    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for (expected_txg, counter) in [(CheckpointTxg(1), 41), (CheckpointTxg(2), 42)] {
            let bytes = PoolReader::read(
                &devices,
                device,
                record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
                record_bytes,
            )
            .expect("环里那一格读得到");
            let record = JournalRecord::parse(&bytes, filesystem_identifier)
                .expect("jsn 对应的那一格里是一条自证过的记录");
            assert_eq!(
                (record.checkpoint_txg, record.counter),
                (expected_txg, counter),
                "盘 {} 上 jsn {counter} 那一格",
                device.0
            );
        }
        for counter_equal_to_txg in [1, 2] {
            let bytes = PoolReader::read(
                &devices,
                device,
                record_offset(counter_equal_to_txg, JOURNAL_RING_DEFAULT_BYTES),
                record_bytes,
            )
            .expect("环里那一格读得到");
            assert!(
                bytes.iter().all(|byte| *byte == 0),
                "盘 {} 上 jsn {counter_equal_to_txg} 那一格没写过：记录不按 txg 落格",
                device.0
            );
        }
    }
}
