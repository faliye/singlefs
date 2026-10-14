//! 管理员回退的目标引用一个隔离单元（C572（回退目标引用隔离单元不进逐盘验））：D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格——
//! R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：读盘核出一份对不上、
//! 两份都留在已分配）同样逐盘验，有一份读不出或对不上就拒（用户 2026-10-12 定「隔离单元也进逐盘验」）。
//!
//! 历史：新池新建文件 A（实例 1，txg 3）；盘 1 上 A 的数据单元那一份改坏；覆盖写 B（txg 4）换掉它，释放之前读盘核出盘 1 那一份对不上，
//! 两块盘那条记录都留在已分配、分配代照旧 3（隔离）；挂着时回退到 A。A 引用的数据单元在 B 的账里仍分配、同分配代，B 不引用它。
//! - 盘 1 那一份仍坏：在任何写之前拒成 `IsolatedUnitCopyUnreadableOrMismatched`（盘 1、那个槽），盘上逐字节不变、现行版本与分配器不动。
//!   不验它（改之前的样子）时回退做成，新根引用一个只有一份读得出的单元。
//! - 盘 1 那一份写回原样（两份都对得上）：回退照做，冷启动读回 A 的内容——逐盘验只拒对不上的，不拒隔离本身。

mod common;

use common::{
    build_pool, disk_snapshot, file_content, parameters, publish_overwrite_in_process, BuiltPool,
    Recorded, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    CopyQuarantinedAfterReleaseChecksumMismatch, QuarantinedCopyReading, TransactionUnit,
};

/// 覆盖写 B 的内容字节数：与 A 的长度不同，冷启动读回哪一版分得开。
const SECOND_CONTENT_BYTES: usize = 2700;
/// 覆盖写 B 比 A 晚这么多秒。
const SECONDS_BETWEEN_THE_TWO_WRITES: u64 = 60;

fn second_content() -> Vec<u8> {
    (0..SECOND_CONTENT_BYTES)
        .map(|index| u8::try_from((index * 11 + 9) % 247).expect("小于 256"))
        .collect()
}

/// A 那一版的根：回退目标。
fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
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
/// 顺带钉住前提：B 隔离了 A 的数据单元两块盘那两份（盘 1 对不上、盘 0 对得上一起留下），两条记录都仍分配、分配代 3，B 不引用那个槽。
fn pool_with_the_data_unit_of_the_first_version_isolated(
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
    let data_placement = singlefs_core::allocator::Placement {
        slot: data_slot,
        span: 2,
    };
    let data_unit = TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile::FIRST);
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        vec![
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: data_unit,
                device: DeviceIdentity(0),
                placement: data_placement,
                reading: QuarantinedCopyReading::IntactButAnotherCopyFailed,
            },
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: data_unit,
                device: DeviceIdentity(1),
                placement: data_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
        ],
        "前提：B 隔离了 A 的数据单元两份（盘 1 对不上，盘 0 对得上一起留下）"
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let record = pool
            .allocator
            .record_for(device, data_slot)
            .expect("隔离只是记录留在已分配，不删");
        assert!(
            !record.is_released && record.generation == CheckpointTxg(3),
            "前提：{device:?} 上那条记录仍分配、分配代照旧 3：{record:?}"
        );
    }
    assert!(
        second.units.iter().all(|unit| unit.slot != data_slot),
        "前提：B 不引用 A 的数据单元那个槽"
    );
    pool.output = second;
    (pool, data_slot, intact_copy_on_device_one)
}

fn roll_back_to_the_first_version(pool: &mut BuiltPool) -> Result<RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        first_root(),
    )
}

#[test]
fn rolling_back_to_the_version_referencing_an_isolated_unit_with_a_copy_that_does_not_verify_is_refused_before_any_write(
) {
    let (mut pool, data_slot, _) =
        pool_with_the_data_unit_of_the_first_version_isolated("rollback-isolated-unit-refused");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let current_before = pool.output.clone();
    let records_before = pool.allocator.records().to_vec();
    let refusal = roll_back_to_the_first_version(&mut pool)
        .expect_err("目标引用的隔离单元盘 1 那一份对不上：在任何写之前拒");
    assert!(
        matches!(
            refusal,
            RollbackError::IsolatedUnitCopyUnreadableOrMismatched {
                unit: TransactionUnit::Data(_),
                device: DeviceIdentity(1),
                slot,
            } if slot == data_slot
        ),
        "拒成隔离单元那一份对不上（盘 1、A 的数据单元那个槽）：{refusal:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒之前盘上一个字节都不动（系统配置槽、读得出的根、录制流步数）"
    );
    assert_eq!(pool.output, current_before, "现行版本不动");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器不动"
    );
}

#[test]
fn rolling_back_to_the_version_referencing_an_isolated_unit_whose_failing_copy_was_written_back_intact_goes_through(
) {
    let (mut pool, data_slot, intact_copy_on_device_one) =
        pool_with_the_data_unit_of_the_first_version_isolated("rollback-isolated-unit-repaired");
    write_copy_at(
        pool.devices.as_mut().expect("镜像还开着"),
        DeviceIdentity(1),
        data_slot,
        &intact_copy_on_device_one,
    );
    let rolled_back =
        roll_back_to_the_first_version(&mut pool).expect("隔离单元两份都对得上：回退照做");
    assert_eq!(rolled_back.target, first_root());
    assert!(
        rolled_back
            .resurrected_user_visible_copies
            .iter()
            .all(|copy| copy.placement.slot != data_slot),
        "隔离单元在账里本来就仍分配，不进复活集"
    );
    let cold = recover(&pool.memory_pool(), JournalPolicy::Consult);
    assert!(
        matches!(
            &cold.outcome,
            RecoveryOutcome::FileRead { content, .. } if *content == file_content()
        ),
        "冷启动读回 A 的内容：{:?}",
        cold.outcome
    );
}
