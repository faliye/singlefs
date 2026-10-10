//! 攻方原型（第 31 批代码轮第一轮，N3 第一问）：`on_device_modes::allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process`
//! 在内存里拼的根环表，与新池新建文件那个进程里真分配器（`allocator_after_make_filesystem` 起、记到 txg 3）的根环表、
//! 与「`root_ring_occupancy_after_make_filesystem` 装上再记 txg 1–3」那一张，三张的 `{:?}` 逐字比；再在两份同字节的盘上
//! 各用自己那个分配器接着覆盖写到 txg 80（转过三圈以上），每一步比根环表、落点与分配记录。几何扫四种 S（4、5、8、16）× 两种块大小。

use singlefs_checker_tier::on_device_modes::allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, root_ring_occupancy_after_make_filesystem,
    MakeFilesystemParameters,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::{
    e142_parameters, first_file_content, run_new_pool_file_creation, NewPoolFileCreationRun,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::SharedStream;

const DEVICE_BYTES: u64 = 4 << 30;

fn fresh_devices(physical_block_size: u32) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(physical_block_size)),
            )
        })
        .collect()
}

fn ring_text(allocator: &PoolAllocator) -> String {
    format!("{:?}", allocator.root_ring_occupancy())
}

fn content_of(length: usize, seed: u64) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index as u64 * 11 + seed * 5 + 1) % 251).expect("小于 256"))
        .collect()
}

fn main() {
    let mut findings = 0u64;
    let mut cells = 0u64;
    for slots_per_region in [4u64, 5, 8, 16] {
        for (physical_block_size, minimum_input_output_bytes) in [(512u32, 512u32), (4096, 4096)] {
            cells += 1;
            let mut parameters: MakeFilesystemParameters =
                e142_parameters(physical_block_size, minimum_input_output_bytes);
            parameters.geometry.root_ring_slots_per_region =
                RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
                    .expect("S 在区间里");
            let label = format!("S={slots_per_region} 块={physical_block_size}");
            // 甲：装置那条路（run_new_pool_file_creation 之后在记录上重建、装 on_device_modes 拼的那张）。
            let mut devices_a = fresh_devices(physical_block_size);
            let run: NewPoolFileCreationRun = run_new_pool_file_creation(
                &parameters,
                &mut devices_a,
                &SharedStream::new(),
                |_point, _devices| {},
            )
            .expect("新池新建文件");
            let mut allocator_a = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
                &parameters,
                &devices_a,
                &run,
            );
            // 乙：同一个进程里真分配器那条路（照 run_new_pool_file_creation_keeping_the_allocator 一步步走）。
            let mut devices_b = fresh_devices(physical_block_size);
            let genesis = make_filesystem(&parameters, &mut devices_b).expect("mkfs");
            let mut allocator_b = allocator_after_make_filesystem(&parameters, &devices_b, &genesis);
            let first_b = {
                let mut pool = PoolWriter::new(&parameters, &mut devices_b);
                let instance = acquire_instance(&mut pool).expect("取号");
                let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
                publish_first_file(
                    &mut pool,
                    &mut allocator_b,
                    warmed.roots.last().expect("暖机两代根"),
                    FirstFile {
                        content: &first_file_content(),
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                    },
                    instance,
                    &warmed.last_record_bytes,
                )
                .expect("第一个文件")
            };
            // 丙：core 的 mkfs 表装上、照次序记 txg 1–3。
            let mut allocator_c = PoolAllocator::new(vec![
                DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),
                DeviceFreeMap::new(DeviceIdentity(1), DEVICE_BYTES),
            ]);
            allocator_c.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
                &parameters,
                &genesis,
            ));
            for txg in 1..=first_b.root.checkpoint_txg.0 {
                allocator_c.record_root_written_by_this_process(CheckpointTxg(txg));
            }
            let (text_a, text_b, text_c) =
                (ring_text(&allocator_a), ring_text(&allocator_b), ring_text(&allocator_c));
            let same_output = run.output.root.checkpoint_txg == first_b.root.checkpoint_txg
                && run.output.allocation_records == first_b.allocation_records;
            println!(
                "name=table_after_txg_3 cell={label} genesis_txg={} first_file_txg={} same_first_file_output={same_output} on_device_equals_in_process={} on_device_equals_core_table_plus_records={}",
                genesis.root.checkpoint_txg.0,
                first_b.root.checkpoint_txg.0,
                text_a == text_b,
                text_a == text_c
            );
            if text_a != text_b || text_a != text_c || !same_output {
                findings += 1;
                println!("finding {label}: 甲 {text_a}\nfinding {label}: 乙 {text_b}\nfinding {label}: 丙 {text_c}");
            }
            let mut previous_a = run.output.clone();
            let mut previous_b = first_b.clone();
            let mut first_divergence: Option<String> = None;
            let mut steps_with_different_ring_tables = 0u64;
            for index in 0..77u64 {
                let content = content_of(2000 + usize::try_from(index * 37).expect("小"), index);
                let next_a = {
                    let mut pool = PoolWriter::new(&parameters, &mut devices_a);
                    publish_overwrite(&mut pool, &mut allocator_a, &previous_a, FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, InstanceGeneration(1)).expect("甲 覆盖写")
                };
                let next_b = {
                    let mut pool = PoolWriter::new(&parameters, &mut devices_b);
                    publish_overwrite(&mut pool, &mut allocator_b, &previous_b, FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, InstanceGeneration(1)).expect("乙 覆盖写")
                };
                let txg = next_a.root.checkpoint_txg.0;
                let tables_equal = ring_text(&allocator_a) == ring_text(&allocator_b);
                let records_equal = next_a.allocation_records == next_b.allocation_records;
                if !tables_equal {
                    steps_with_different_ring_tables += 1;
                }
                let deferred_equal = allocator_a.devices.iter().map(DeviceFreeMap::deferred_slots).collect::<Vec<_>>()
                    == allocator_b.devices.iter().map(DeviceFreeMap::deferred_slots).collect::<Vec<_>>();
                if first_divergence.is_none() && !(tables_equal && records_equal && deferred_equal) {
                    let first_record_difference = next_a.allocation_records.iter().zip(next_b.allocation_records.iter()).find(|(left, right)| left != right).map(|(left, right)| format!("{left:?} / {right:?}"));
                    let data_slots = (next_a.data_pointers.first().map(|pointer| pointer.locations[0].slot.0), next_b.data_pointers.first().map(|pointer| pointer.locations[0].slot.0));
                    first_divergence = Some(format!("txg {txg}: 表同 {tables_equal} 记录同 {records_equal} defer 同 {deferred_equal} 记录条数 {} / {} 第一处不同 {first_record_difference:?} 数据单元槽 {data_slots:?}", next_a.allocation_records.len(), next_b.allocation_records.len()));
                }
                previous_a = next_a;
                previous_b = next_b;
            }
            println!(
                "name=continued_overwrites cell={label} last_txg={} steps_with_different_ring_tables={steps_with_different_ring_tables} first_divergence={}",
                previous_a.root.checkpoint_txg.0,
                first_divergence.clone().unwrap_or_else(|| "none".to_string())
            );
            if first_divergence.is_some() {
                findings += 1;
            }
        }
    }
    println!("name=summary cells={cells} findings={findings}");
}
