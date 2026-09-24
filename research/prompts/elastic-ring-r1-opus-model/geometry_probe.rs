//! elastic-ring-r1 云端攻方腿探针（只在草稿副本里，不入库）：
//! 环长取非默认值时，mkfs 走得通而单元区起点仍是格式常量 50176。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::allocator::DeviceFreeMap;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{choose_system_configuration, PoolReader};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};

const FSID: [u8; 16] = [7u8; 16];

fn parameters(ring_bytes: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FSID,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: ring_bytes,
        },
    }
}

fn mebibytes(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// 一次 mkfs：给设备字节数与环长，打出几何与 checker 判决。
fn probe(tag: &str, device_bytes: u64, ring_bytes: u64) {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let outcome = make_filesystem(&parameters(ring_bytes), &mut devices);
    println!("== {tag}：盘 {} MiB，环 {} MiB", mebibytes(device_bytes), mebibytes(ring_bytes));
    match &outcome {
        Err(error) => {
            println!("   mkfs 被拒：{error:?}");
            return;
        }
        Ok(_) => println!("   mkfs 接受"),
    }
    let ring_end_slot = JOURNAL_RING_START_SLOT + ring_bytes / SLOT_BYTES;
    println!(
        "   环占槽 [{}, {})；代码里单元区起点常量 UNIT_AREA_START_SLOT = {}",
        JOURNAL_RING_START_SLOT, ring_end_slot, UNIT_AREA_START_SLOT
    );
    if ring_end_slot > UNIT_AREA_START_SLOT {
        println!(
            "   重叠 {} 槽 = {} MiB（环盖住单元区开头）",
            ring_end_slot - UNIT_AREA_START_SLOT,
            mebibytes((ring_end_slot - UNIT_AREA_START_SLOT) * SLOT_BYTES)
        );
    } else {
        println!(
            "   空洞 {} 槽 = {} MiB（既不是环也不是单元区）",
            UNIT_AREA_START_SLOT - ring_end_slot,
            mebibytes((UNIT_AREA_START_SLOT - ring_end_slot) * SLOT_BYTES)
        );
    }
    println!(
        "   mkfs 的两个单元落槽 {} 与 {}；在环里吗：{} / {}",
        INSTANCE_TABLE_SLOT.0,
        TREE_TABLE_GENESIS_SLOT.0,
        INSTANCE_TABLE_SLOT.0 < ring_end_slot,
        TREE_TABLE_GENESIS_SLOT.0 < ring_end_slot
    );
    // 环里哪一条记录会写到实例表那一槽上
    let ring_slots = ring_bytes / SLOT_BYTES; // 16 KiB 槽
    let _ = ring_slots;
    let instance_table_offset = INSTANCE_TABLE_SLOT.0 * SLOT_BYTES;
    let ring_start_offset = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    if instance_table_offset >= ring_start_offset && instance_table_offset < ring_start_offset + ring_bytes {
        let record_index = (instance_table_offset - ring_start_offset) / 4096;
        let counter = record_index + 1;
        println!(
            "   写到实例表那一槽的是 jsn {}（record_offset 复核 {:?}，实例表在 {}）",
            counter,
            record_offset(counter, ring_bytes).0,
            instance_table_offset
        );
    }
    // 分配器眼里的单元区
    let free_map = DeviceFreeMap::new(DeviceIdentity(0), device_bytes);
    let first_free_in_ring = (UNIT_AREA_START_SLOT..ring_end_slot.min(UNIT_AREA_START_SLOT + free_map.unit_area_slots()))
        .find(|slot| free_map.is_free(SlotNumber(*slot)));
    println!(
        "   分配器：单元区 {} 槽，起点 {}；环里还被当成空闲的第一个槽 {:?}",
        free_map.unit_area_slots(),
        UNIT_AREA_START_SLOT,
        first_free_in_ring
    );
    // 盘上字段
    let pool = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    let slot = pool
        .read(DeviceIdentity(0), singlefs_core::address::DeviceOffsetInBytes(0), 4096)
        .expect("系统配置槽 0");
    // checker 的读法：小端，环长 333、单元区起点 417（crates/singlefs-checker/src/lib.rs 的 read_u64）
    let on_disk_ring = u64::from_le_bytes(slot[333..341].try_into().unwrap());
    let on_disk_unit_area = u64::from_le_bytes(slot[417..425].try_into().unwrap());
    println!(
        "   盘上字段：环长 {} 字节，单元区起点 {}（偏移 333 / 417）",
        on_disk_ring, on_disk_unit_area
    );
    let configuration = choose_system_configuration(&pool);
    println!(
        "   choose_system_configuration 读回环长：{:?}",
        configuration.map(|c| c.immutable.sizes.journal_ring_bytes)
    );
    let verdicts = check_pool_image(&pool);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    let holds = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Holds))
        .count();
    println!(
        "   checker：{} 条判决，成立 {}，违例 {:?}",
        verdicts.len(),
        holds,
        violated
    );
}

#[test]
fn elastic_ring_probe() {
    probe("A 默认环", 4 << 30, JOURNAL_RING_DEFAULT_BYTES);
    probe("B 环取容量四分之一（1 GiB）", 4 << 30, 1 << 30);
    probe("C 小镜像按参数缩（1 GiB 盘、256 MiB 环）", 1 << 30, 256 << 20);
    probe("D 环 64 MiB（4 GiB 盘）", 4 << 30, 64 << 20);
    probe("E 环 12 KiB（I-8.1 按「任一事务」读的下界）", 4 << 30, 12 << 10);
}

/// 环取 1 GiB（4 GiB 盘上恰好是容量四分之一、mkfs 接受）时，跑完取号 + 暖机 + 第一个事务：
/// 用户数据单元落在哪些槽、那些槽在不在声明的环里。
#[test]
fn first_transaction_with_a_one_gibibyte_ring() {
    use singlefs_core::allocator::{PoolAllocator, Placement};
    use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter};
    const DEVICE_BYTES: u64 = 4 << 30;
    const RING: u64 = 1 << 30;
    let parameters = parameters(RING);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|n| (DeviceIdentity(n), SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), DEVICE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let content: Vec<u8> = (0..3000u32).map(|i| u8::try_from(i % 251).expect("小于 256")).collect();
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            &genesis.root,
            FirstFile { content: &content, write_time_seconds: 1_788_000_000 },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个事务")
    };
    let ring_end_slot = JOURNAL_RING_START_SLOT + RING / SLOT_BYTES;
    println!("== 1 GiB 环上的第一个事务：环占槽 [{}, {})", JOURNAL_RING_START_SLOT, ring_end_slot);
    println!("   第一个事务发布成功：txg {:?}", output.root.checkpoint_txg);
    let pool = MemoryPool {
        devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(),
        device_size_in_bytes: DEVICE_BYTES,
    };
    let verdicts = check_pool_image(&pool);
    let violated: Vec<(&str, String)> = verdicts
        .iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(why) => Some((*name, why.clone())),
            _ => None,
        })
        .collect();
    println!("   checker：违例 {:?}", violated);
    println!("   （单元落槽见下面这份 output 的 debug）");
    println!("   {:?}", output.root);
}
