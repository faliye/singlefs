//! 攻方腿（代码欠账批甲簇代码轮，Opus）原型：隔离单元的另一个来路——管理员回退那次发布自己释放 cur 的用户可见单元时读盘核出一份对不上、
//! 两份留在已分配（`roll_back_by_a_forward_publish` 的释放集那一段）。之后再回退到引用它的那一版，看新那一步认不认它、验不验它。
//! 历史：A（txg 3）→ 覆盖写 B（txg 4）→ 盘 1 上 B 的数据单元改坏 → 回退到 A（txg 5，释放 B 的数据单元时隔离）→ 回退到 B。

mod common;

use common::{build_pool, disk_snapshot, parameters, publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};

fn content_of_b() -> Vec<u8> {
    (0..2700usize).map(|index| u8::try_from((index * 11 + 9) % 247).expect("小于 256")).collect()
}

fn target(txg: u64) -> RollbackTarget {
    RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(txg) }
}

fn copy_at(devices: &[(DeviceIdentity, Recorded)], device: DeviceIdentity, slot: SlotNumber) -> Vec<u8> {
    let (_, block_device) = devices.iter().find(|(identity, _)| *identity == device).expect("盘");
    let mut bytes = vec![0u8; 32768];
    block_device.read_at(slot.to_device_offset(), &mut bytes).expect("读");
    bytes
}

fn write_at(devices: &mut [(DeviceIdentity, Recorded)], device: DeviceIdentity, slot: SlotNumber, bytes: &[u8]) {
    let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("盘");
    block_device.write_at(slot.to_device_offset(), bytes, WriteDurability::Plain).expect("写");
}

fn roll_back(pool: &mut BuiltPool, txg: u64) -> Result<RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(&publish_parameters, devices, &mut pool.allocator, &mut pool.output, target(txg))
}

/// 走到「回退到 A 那次发布隔离了 B 的数据单元」：交回池、B 的数据单元的槽、盘 `bad_device` 上那一份改坏之前的字节。
fn pool_after_the_rollback_isolated_the_data_unit_of_b(tag: &str, bad_device: DeviceIdentity) -> (BuiltPool, SlotNumber, Vec<u8>) {
    let mut pool = build_pool(tag);
    let version_a = pool.output.clone();
    let version_b = publish_overwrite_in_process(&mut pool, &version_a, &content_of_b(), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
        .expect("覆盖写 B");
    pool.output = version_b.clone();
    let slot_of_b = version_b.data_pointers[0].locations[0].slot;
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let intact = copy_at(devices, bad_device, slot_of_b);
    let mut corrupted = intact.clone();
    corrupted[4096] ^= 0xff;
    write_at(devices, bad_device, slot_of_b, &corrupted);
    let to_a = roll_back(&mut pool, 3).expect("回退到 A：释放 B 的数据单元时只隔离，不拒");
    eprintln!(
        "OPUS-K2-ROLLBACK-RELEASE bad_device={} quarantined_copies={} released={}",
        bad_device.0,
        to_a.quarantined_user_visible_copies.len(),
        to_a.released_user_visible_units.len()
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let record = pool.allocator.record_for(device, slot_of_b).expect("记录在");
        eprintln!("OPUS-K2-ROLLBACK-RELEASE record device={} is_released={} generation={}", device.0, record.is_released, record.generation.0);
    }
    (pool, slot_of_b, intact)
}

#[test]
fn opus_k2_rolling_back_to_b_after_a_rollback_isolated_its_data_unit() {
    for bad_device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let (mut pool, slot_of_b, intact) =
            pool_after_the_rollback_isolated_the_data_unit_of_b(&format!("opus-k2-refused-{}", bad_device.0), bad_device);
        let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
        let records_before = pool.allocator.records().to_vec();
        let outcome = roll_back(&mut pool, 4);
        eprintln!(
            "OPUS-K2-BACK-TO-B bad_device={} slot_of_b={} outcome={:?} disk_unchanged={} records_unchanged={}",
            bad_device.0,
            slot_of_b.0,
            outcome.as_ref().map(|rolled| rolled.target).map_err(|error| format!("{error:?}")),
            disk_snapshot(&pool.memory_pool(), &pool.stream) == before,
            pool.allocator.records() == records_before.as_slice(),
        );
        // 阳性对照：写回原样再回退到 B。
        write_at(pool.devices.as_mut().expect("镜像还开着"), bad_device, slot_of_b, &intact);
        let repaired = roll_back(&mut pool, 4);
        let cold = recover(&pool.memory_pool(), JournalPolicy::Consult);
        eprintln!(
            "OPUS-K2-BACK-TO-B-REPAIRED bad_device={} outcome={:?} cold_reads_b={}",
            bad_device.0,
            repaired.as_ref().map(|rolled| rolled.target).map_err(|error| format!("{error:?}")),
            matches!(&cold.outcome, RecoveryOutcome::FileRead { content, .. } if *content == content_of_b()),
        );
    }
}
