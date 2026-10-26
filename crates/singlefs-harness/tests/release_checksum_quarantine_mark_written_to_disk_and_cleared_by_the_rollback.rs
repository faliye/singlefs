//! 隔离的盘上记号（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，用户 2026-10-22 定落点甲）：写者隔离那次发布把两块盘上那条
//! 分配记录的跨度段置 0x4000（最高位 0x8000 仍是已释放标志），分配代不变，与这次发布的别的改动一起落盘；回退复活那一次，同一次发布清掉
//! 两块盘上的 0x4000（释放隔离记录时清掉记号那一条，核心层今天没有释放隔离记录的路径，钉在 `allocator.rs` 的单测里）。
//!
//! 历史：新池新建文件 A（实例 1，txg 3）；盘 1 上 A 的数据单元那一份改坏；覆盖写 B（txg 4）换掉它，释放之前读盘核出盘 1 那一份对不上，
//! 两块盘那条记录都留在已分配、分配代照旧 3、带上隔离记号。
//! - 盘上：从最新根读回的分配记录树（`recovery::allocation_records_under_root`，读者认记号、跨度取低位）里那一槽两块盘的记录仍分配、
//!   分配代 3、跨度 2、带记号；进程重开可写挂载之后分配器里的那两条照样带着。
//! - 盘 1 那一份写回原样之后挂着回退到 A：A 引用那个数据单元，回退那次发布清掉两块盘上的记号，读回的记录仍分配、分配代 3、不带记号。

mod common;

use common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, Recorded,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{
    AllocationRecord, AllocationRecordQuarantineMark, ALLOCATION_RECORD_QUARANTINE_MARK,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
};

/// 覆盖写 B 的内容字节数：与 A 的长度不同。
const SECOND_CONTENT_BYTES: usize = 2700;
/// 覆盖写 B 比 A 晚这么多秒。
const SECONDS_BETWEEN_THE_TWO_WRITES: u64 = 60;
/// A 那一版的 txg：隔离的记录的分配代。
const TXG_OF_THE_FIRST_VERSION: CheckpointTxg = CheckpointTxg(3);
/// 数据单元跨两槽。
const DATA_UNIT_SPAN_SLOTS: u16 = 2;

fn second_content() -> Vec<u8> {
    (0..SECOND_CONTENT_BYTES)
        .map(|index| u8::try_from((index * 13 + 5) % 241).expect("小于 256"))
        .collect()
}

fn read_copy_at(
    devices: &[(DeviceIdentity, Recorded)],
    device: DeviceIdentity,
    slot: SlotNumber,
) -> Vec<u8> {
    let (_, block_device) = devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let mut bytes = vec![0u8; usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768")];
    block_device
        .read_at(slot.to_device_offset(), &mut bytes)
        .expect("读一份数据单元");
    bytes
}

fn write_copy_at(
    devices: &mut [(DeviceIdentity, Recorded)],
    device: DeviceIdentity,
    slot: SlotNumber,
    bytes: &[u8],
) {
    let (_, block_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    block_device
        .write_at(slot.to_device_offset(), bytes, WriteDurability::Plain)
        .expect("写一份数据单元");
}

/// 走到「B 发布完、A 的数据单元隔离着」那一刻：交回池（`output` 是 B）、A 的数据单元的槽、盘 1 上那一份改坏之前的字节。
fn pool_with_the_data_unit_of_the_first_version_quarantined(
    tag: &str,
) -> (BuiltPool, SlotNumber, Vec<u8>) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let intact_copy_on_device_one = read_copy_at(devices, DeviceIdentity(1), data_slot);
    let mut corrupted_copy = intact_copy_on_device_one.clone();
    corrupted_copy[4096] ^= 0xff;
    write_copy_at(devices, DeviceIdentity(1), data_slot, &corrupted_copy);
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content(),
        FIXED_WRITE_TIME_SECONDS + SECONDS_BETWEEN_THE_TWO_WRITES,
        InstanceGeneration(1),
    )
    .expect("只一份对不上时覆盖写照成");
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch.len(),
        2,
        "前提：B 隔离了 A 的数据单元两块盘那两份"
    );
    pool.output = second;
    (pool, data_slot, intact_copy_on_device_one)
}

/// 从镜像上最新根那棵分配记录树读回来的、起点是 `slot` 的那几条记录（两块盘各一条）。
fn records_read_back_from_the_newest_root(
    pool: &BuiltPool,
    slot: SlotNumber,
) -> Vec<AllocationRecord> {
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("择系统配置");
    let newest = choose_root(&image, &system_configuration).expect("最新根");
    allocation_records_under_root(&image, &newest)
        .expect("最新根那棵分配记录树读得回来（读者认隔离记号）")
        .into_iter()
        .filter(|record| record.slot == slot)
        .collect()
}

/// 隔离那次发布：两块盘上那条记录读回来仍分配、分配代不变、跨度取低位、带隔离记号，跨度段写回去是 0x4002；进程重开可写挂载之后的分配器同样带着。
/// 写者不置记号（记录一个字节不动）时读回来不带记号，这条红。
#[test]
fn quarantine_writes_the_mark_into_the_span_field_on_both_devices_and_keeps_the_generation() {
    let (mut pool, data_slot, _) =
        pool_with_the_data_unit_of_the_first_version_quarantined("quarantine-mark-written");
    let records = records_read_back_from_the_newest_root(&pool, data_slot);
    assert_eq!(
        records
            .iter()
            .map(|record| record.device)
            .collect::<Vec<_>>(),
        vec![DeviceIdentity(0), DeviceIdentity(1)],
        "两块盘各一条"
    );
    for record in &records {
        assert!(!record.is_released, "{:?}：隔离的记录仍分配", record.device);
        assert_eq!(
            record.generation, TXG_OF_THE_FIRST_VERSION,
            "{:?}：分配代不变",
            record.device
        );
        assert_eq!(
            record.span_slots, DATA_UNIT_SPAN_SLOTS,
            "{:?}：跨度取去掉记号的低位",
            record.device
        );
        assert_eq!(
            record.quarantine_mark,
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            "{:?}：盘上那条记录带隔离记号",
            record.device
        );
        let span_field = u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]);
        assert_eq!(
            span_field,
            ALLOCATION_RECORD_QUARANTINE_MARK | DATA_UNIT_SPAN_SLOTS,
            "{:?}：跨度段是 0x4002",
            record.device
        );
    }
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    pool.devices = Some(devices);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let record = mounted
            .allocator
            .record_for(device, data_slot)
            .expect("重挂之后分配器里有这条记录");
        assert_eq!(
            (record.is_released, record.quarantine_mark),
            (
                false,
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck
            ),
            "{device:?}：重挂之后的分配器里那条记录仍分配、带着记号"
        );
    }
}

/// 回退到 A（它引用那个数据单元）：盘 1 那一份先写回原样（逐盘验要读得出、对得上），回退那次发布清掉两块盘上的记号，
/// 读回来的记录仍分配、分配代 3、不带记号。回退不清记号时读回来仍带着，这条红。
#[test]
fn rolling_back_to_the_version_referencing_the_quarantined_unit_clears_the_mark_on_both_devices() {
    let (mut pool, data_slot, intact_copy_on_device_one) =
        pool_with_the_data_unit_of_the_first_version_quarantined(
            "quarantine-mark-cleared-by-rollback",
        );
    write_copy_at(
        pool.devices.as_mut().expect("镜像还开着"),
        DeviceIdentity(1),
        data_slot,
        &intact_copy_on_device_one,
    );
    let publish_parameters = parameters();
    roll_back_by_a_forward_publish(
        &publish_parameters,
        pool.devices.as_mut().expect("镜像还开着"),
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: TXG_OF_THE_FIRST_VERSION,
        },
    )
    .expect("隔离单元两份都对得上：回退照做");
    let records = records_read_back_from_the_newest_root(&pool, data_slot);
    assert_eq!(records.len(), 2, "两块盘各一条");
    for record in &records {
        assert!(
            !record.is_released && record.generation == TXG_OF_THE_FIRST_VERSION,
            "{:?}：回退之后那条记录仍分配、分配代 3：{record:?}",
            record.device
        );
        assert_eq!(
            record.quarantine_mark,
            AllocationRecordQuarantineMark::NotQuarantined,
            "{:?}：回退那次发布清掉了隔离记号",
            record.device
        );
    }
}
