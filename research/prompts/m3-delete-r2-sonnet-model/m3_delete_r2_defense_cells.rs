//! m3-delete-r2 辩方腿（副本专用，被攻过零轮）：在第一轮攻方原型（`publish_unlink_batch_for_tests`，m3-delete-r1-opus-model）上量 V2 的逐格值。
//! 原型没有意图记录、`deleted_inodes`、墓碑；「意图那一次」用 keep = u64::MAX 的一次发布代替（extent 与数据单元照抄，只把 inode 1 记录写成 nlink = 0），
//! 墓碑单元（码 3，2 槽）与 `deleted_inodes` 根（码 2，1 槽）不在盘上，按槽数另加（输出里标 `s1_extra_slots`）。
//! 只打印 `D name=...` 行；断言钉住看到的事实。

use std::collections::BTreeMap;

use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::{admission_reading_before_a_publish, demand_of_the_roles_on_each_device};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_sequential_write, publish_unlink_batch_for_tests,
    warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const IMAGE_BYTES: u64 = 4 << 30;
const WRITE_TIME: u64 = 1_788_000_000;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"_SFS-M3DEL-DEF1\0",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn content_needing(units: usize, seed: usize) -> Vec<u8> {
    let payload = data_unit_payload_capacity();
    ((0..(units - 1) * payload + payload / 2)
        .map(|index| u8::try_from((index * 31 + seed * 7 + 1) % 251).expect("小于 256")))
    .collect()
}

struct Pool {
    devices: Devices,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    first: TransactionOutput,
    written: TransactionOutput,
}

/// mkfs → 取号 → 暖机 → 第一个文件（1 单元）→ 顺序写成 `units` 个数据单元。
fn pool_with_a_file_of(units: usize, holes: Option<u64>) -> Pool {
    let parameters = parameters();
    let mut devices: Devices = (0..2u32)
        .map(|number| {
            (DeviceIdentity(number), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let first_content = content_needing(1, 1);
    let (instance, first, written) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机根"),
            FirstFile { content: &first_content, write_time_seconds: WRITE_TIME },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个文件");
        if let Some(holes) = holes {
            // 把分配器拷贝上每块盘的空闲槽标成已分配，只留 `holes` 个相隔 1624 槽（两片分配记录叶，叶罩 812 槽）的两槽空洞与尾部一大片全空段：
            // 数据单元按「最低的空槽」取落点，于是一个单元落一片叶；盘上不写（同第一轮攻方腿的「满池」造法）。
            for map in &mut allocator.devices {
                let start = map.unit_area_start().slot().0;
                let end = start + map.unit_area_slots();
                let tail_start = end - 8192;
                let hole_starts: std::collections::BTreeSet<u64> =
                    (0..holes).map(|k| start + 4096 + 2 * k * 812).collect();
                let mut slot = start;
                while slot < tail_start {
                    let in_hole = hole_starts.contains(&slot) || hole_starts.contains(&(slot - 1));
                    if map.is_free(SlotNumber(slot)) && !in_hole {
                        map.mark_allocated(SlotNumber(slot), 1);
                    }
                    slot += 1;
                }
            }
        }
        let content = content_needing(units, 2);
        let written = publish_sequential_write(
            &mut pool,
            &mut allocator,
            &first,
            FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 },
            instance,
        )
        .expect("顺序写");
        (instance, first, written)
    };
    Pool { devices, allocator, instance, first, written }
}

fn batch(pool: &mut Pool, previous: &TransactionOutput, keep_units: u64, size_after: u64) -> TransactionOutput {
    let parameters = parameters();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    publish_unlink_batch_for_tests(&mut writer, &mut pool.allocator, previous, keep_units, size_after, pool.instance)
        .unwrap_or_else(|error| panic!("留 {keep_units} 那一批：{error:?}"))
}

fn kinds(output: &TransactionOutput) -> String {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for role in &output.rewritten {
        let name = format!("{role:?}");
        let key = name.split(['(', ' ', '{']).next().unwrap_or("").to_owned();
        *counts.entry(key).or_default() += 1;
    }
    counts.iter().map(|(name, count)| format!("{name}:{count}")).collect::<Vec<_>>().join(",")
}

fn slots_of(output: &TransactionOutput) -> u64 {
    output.rewritten.iter().map(|role| role.span_slots()).sum()
}

fn counts(allocator: &PoolAllocator) -> String {
    allocator
        .devices
        .iter()
        .map(|map| {
            format!(
                "dev{}_allocated={} dev{}_free={} dev{}_deferred={}",
                map.device.0, map.allocated_slots(), map.device.0, map.free_slots(), map.device.0, map.deferred_slots()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 真读数（带实例切换预留与 checkpoint 保留池，待删占用与已承诺预留是今天的两个 0）下这一刻每块盘的可用（字节）。
fn baseline_available(allocator: &PoolAllocator, previous: &TransactionOutput) -> Vec<i128> {
    admission_reading_before_a_publish(allocator, Some(previous))
        .available_on_each_device()
        .into_iter()
        .map(|(_, available)| available.0)
        .collect()
}

fn alloc_leaves_rewritten(output: &TransactionOutput) -> u64 {
    output
        .rewritten
        .iter()
        .filter(|role| {
            matches!(
                role,
                singlefs_core::transaction::TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                    | singlefs_core::transaction::TransactionUnit::AllocationTree
            )
        })
        .count() as u64
}

fn run(units: usize, per_batch: u64, tag: &str, holes: Option<u64>) {
    let mut pool = pool_with_a_file_of(units, holes);
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let n = u64::try_from(units).expect("单元数");
    let demand_one = |output: &TransactionOutput| {
        demand_of_the_roles_on_each_device(&output.rewritten, &[DeviceIdentity(0), DeviceIdentity(1)])[0].bytes.0
            / SLOT_BYTES
    };
    // 写入那一刻（顺序写之后）与之前
    println!(
        "D name=publish tag={tag} step=first_file txg={} roles=[{}] slots_per_device={} {} baseline_available_per_device={:?}",
        pool.first.root.checkpoint_txg.0,
        kinds(&pool.first),
        slots_of(&pool.first),
        counts(&pool.allocator),
        "-"
    );
    let written = pool.written.clone();
    println!(
        "D name=publish tag={tag} step=sequential_write units={units} txg={} roles=[{}] slots_per_device={} demand_slots={} alloc_tree_nodes_rewritten={} {} baseline_available_per_device={:?}",
        written.root.checkpoint_txg.0,
        kinds(&written),
        slots_of(&written),
        demand_one(&written),
        alloc_leaves_rewritten(&written),
        counts(&pool.allocator),
        baseline_available(&pool.allocator, &written)
    );
    // 意图那一次：extent 与数据照抄，inode 1 记录 nlink = 0
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(Some(1)));
    let mut previous = batch(&mut pool, &written, u64::MAX, written.inode_record.size);
    let mut step = 0u64;
    println!(
        "D name=publish tag={tag} step=intent txg={} roles=[{}] slots_per_device={} demand_slots={} alloc_tree_nodes_rewritten={} {} baseline_available_per_device={:?} s1_extra_slots=3(deleted_inodes_root_1+tombstone_2)",
        previous.root.checkpoint_txg.0,
        kinds(&previous),
        slots_of(&previous),
        demand_one(&previous),
        alloc_leaves_rewritten(&previous),
        counts(&pool.allocator),
        baseline_available(&pool.allocator, &previous)
    );
    let mut remaining = n;
    while remaining > 0 {
        step += 1;
        let keep = remaining.saturating_sub(per_batch);
        let next = batch(&mut pool, &previous, keep, keep * payload);
        println!(
            "D name=publish tag={tag} step=batch_{step}_keep_{keep} txg={} roles=[{}] slots_per_device={} demand_slots={} alloc_tree_nodes_rewritten={} {} baseline_available_per_device={:?} s1_extra_slots={}",
            next.root.checkpoint_txg.0,
            kinds(&next),
            slots_of(&next),
            demand_one(&next),
            alloc_leaves_rewritten(&next),
            counts(&pool.allocator),
            baseline_available(&pool.allocator, &next),
            if keep == 0 { "last_batch:inode_leaf_2_dropped,deleted_inodes_root_1_added" } else { "0" }
        );
        previous = next;
        remaining = keep;
    }
}

#[test]
fn three_units_k_one() {
    run(3, 1, "n3k1", None);
}

#[test]
fn three_hundred_units_k_144_contiguous() {
    run(300, 144, "n300k144", None);
}

#[test]
fn one_hundred_forty_four_units_k_144_scattered_one_leaf_per_unit() {
    run(144, 144, "n144k144_scattered", Some(144));
}

fn memory_pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn devices_of(image: &MemoryPool) -> Devices {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

/// V1：同一份崩溃镜像（意图已提交）各自重新挂载、各自续做同一批（留 2），两份盘逐字节相同吗？
/// 第一轮攻方的 `redo_after_crash` 比的是「原进程里做完的一批」与「重挂载之后的一批」，两边实例与 txg 本来就不同；这里两边都走重挂载。
#[test]
fn the_same_crash_image_continued_twice_is_byte_equal() {
    let mut pool = pool_with_a_file_of(3, None);
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(Some(1)));
    let written = pool.written.clone();
    let _intent = batch(&mut pool, &written, u64::MAX, written.inode_record.size);
    let image_at_the_intent = memory_pool_of(&pool.devices);
    let mut results = Vec::new();
    for _ in 0..2 {
        let mut devices = devices_of(&image_at_the_intent);
        let mounted = mount_writable(&parameters(), &mut devices).expect("重挂载");
        let PoolVersion::WithFile(current) = mounted.current else { panic!("带文件的一版") };
        let mut mounted_pool = Pool {
            devices,
            allocator: mounted.allocator,
            instance: mounted.output.instance,
            first: current.clone(),
            written: current.clone(),
        };
        let next = batch(&mut mounted_pool, &current, 2, 2 * payload);
        results.push((memory_pool_of(&mounted_pool.devices), next, mounted_pool.instance));
    }
    let images_equal = results[0].0 == results[1].0;
    println!(
        "D name=redo_from_the_same_image images_byte_equal={images_equal} outputs_equal={} instance_first={} instance_second={} txg_first={} txg_second={}",
        results[0].1 == results[1].1,
        results[0].2 .0,
        results[1].2 .0,
        results[0].1.root.checkpoint_txg.0,
        results[1].1.root.checkpoint_txg.0
    );
    assert!(images_equal, "同一份镜像各自重挂载、各自续做同一批：盘上逐字节相同");
}
