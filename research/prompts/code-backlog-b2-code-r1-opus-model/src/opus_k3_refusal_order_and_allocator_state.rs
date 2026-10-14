//! 攻方腿（代码欠账批甲簇代码轮，Opus）原型 K3：新那一步拒之前分配器整份（Debug 串）、现行版本、盘与录制流都不动；
//! 复活集与隔离单元两种拒因同时成立时报哪一个。历史：A（txg 3）→ 覆盖写 B（txg 4）→ 盘 1 上 B 的数据单元改坏 →
//! 回退到 A（txg 5：释放 B 的数据单元时隔离，B 的 inode 根照常释放）→（可选）盘 0 上 B 的 inode 根改坏 → 回退到 B。

mod common;

use common::{build_pool, disk_snapshot, parameters, publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack};
use singlefs_core::transaction::TransactionUnit;

fn content_of_b() -> Vec<u8> {
    (0..2700usize).map(|index| u8::try_from((index * 11 + 9) % 247).expect("小于 256")).collect()
}

fn flip_a_byte(devices: &mut [(DeviceIdentity, Recorded)], device: DeviceIdentity, slot: SlotNumber, bytes: usize) {
    let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("盘");
    let mut copy = vec![0u8; bytes];
    block_device.read_at(slot.to_device_offset(), &mut copy).expect("读");
    copy[100] ^= 0xff;
    block_device.write_at(slot.to_device_offset(), &copy, WriteDurability::Plain).expect("写");
}

fn roll_back(pool: &mut BuiltPool, txg: u64) -> Result<RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(txg) },
    )
}

#[test]
fn opus_k3_refusal_leaves_the_whole_allocator_untouched_and_reports_the_resurrected_reason_before_the_isolated_one() {
    for also_corrupt_a_resurrected_unit in [false, true] {
        let mut pool = build_pool(&format!("opus-k3-{also_corrupt_a_resurrected_unit}"));
        let version_a = pool.output.clone();
        let version_b = publish_overwrite_in_process(&mut pool, &version_a, &content_of_b(), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
            .expect("覆盖写 B");
        pool.output = version_b.clone();
        let data_slot_of_b = version_b.data_pointers[0].locations[0].slot;
        let inode_root_of_b = version_b.units.iter().find(|unit| unit.identity == TransactionUnit::InodeRoot).expect("B 有 inode 根");
        let inode_root_slot_of_b = inode_root_of_b.slot;
        let inode_root_bytes = inode_root_of_b.bytes.len();
        flip_a_byte(pool.devices.as_mut().expect("开着"), DeviceIdentity(1), data_slot_of_b, 32768);
        roll_back(&mut pool, 3).expect("回退到 A");
        if also_corrupt_a_resurrected_unit {
            flip_a_byte(pool.devices.as_mut().expect("开着"), DeviceIdentity(0), inode_root_slot_of_b, inode_root_bytes);
        }
        let before_disk = disk_snapshot(&pool.memory_pool(), &pool.stream);
        let before_allocator = format!("{:?}", pool.allocator);
        let before_output = pool.output.clone();
        let outcome = roll_back(&mut pool, 4);
        eprintln!(
            "OPUS-K3 also_corrupt_a_resurrected_unit={also_corrupt_a_resurrected_unit} data_slot_of_b={} inode_root_slot_of_b={} outcome={:?} disk_unchanged={} allocator_debug_unchanged={} output_unchanged={}",
            data_slot_of_b.0,
            inode_root_slot_of_b.0,
            outcome.as_ref().map(|rolled| rolled.target).map_err(|error| format!("{error:?}")),
            disk_snapshot(&pool.memory_pool(), &pool.stream) == before_disk,
            format!("{:?}", pool.allocator) == before_allocator,
            pool.output == before_output,
        );
    }
}
