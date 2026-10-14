//! harness 测试文件 `crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    parameters_of_the_ring_hole_pools, pool_of, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, raise_rollback_floor, Mounted, ShadowLedger};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_of_root, readable_roots,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_format::{
    DATA_UNIT_BYTES, ROOT_RING_REGIONS, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 丁-defer 那一形实例 1 发到的 txg（被抛弃的那一条；它前一条 7 就是 B）。
pub const ABANDONED_TXG_UNDER_THE_RAISED_FLOOR: u64 = 8;

/// 一圈窗口的长度（3S）。
pub fn ring_length_in_txgs() -> u64 {
    ROOT_RING_REGIONS * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS
}

/// 盘面上读得出的根、最新那条根指着的实例表、按它判被抛弃。
pub struct RootsOfTheRing {
    pub newest: RootRecord,
    pub valid: Vec<RootRecord>,
    pub abandoned: Vec<RootRecord>,
}

pub fn roots_of_the_ring(image: &MemoryPool) -> RootsOfTheRing {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let newest = choose_root(image, &system_configuration).expect("择得到根");
    let table = instance_table_of_root(image, &newest).expect("最新那条根的实例表读得出");
    let immutable = &system_configuration.immutable;
    let (abandoned, valid): (Vec<RootRecord>, Vec<RootRecord>) = readable_roots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .partition(|root| {
        table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    });
    RootsOfTheRing {
        newest,
        valid,
        abandoned,
    }
}

/// 一条根在不在锚在 `anchor` 上的一圈窗口里（txg > N − 3S）。
pub fn inside_the_one_revolution_window(root: &RootRecord, anchor: CheckpointTxg) -> bool {
    root.checkpoint_txg.0 + ring_length_in_txgs() > anchor.0
}

/// 场景：实例 1 发到 txg 8，8 被崩溃恢复抛弃（实例 2 写行 (1, 7)，B = 7）；再挂一次（实例 3）覆盖写到 txg 18；下一次可写挂载崩在写行 txg 19
/// 那次根槽写之前（19 与 B 同槽：B 留在洞里）；再一次可写挂载做完；同一个进程里把 F 抬到 8。
/// 预期（前提）：B 还在环里、有效、在 F 之下、在锚在最新根上的一圈窗口外，窗口里最旧的有效根高于 F；
/// 池级 checker 不判 I-3.1——丁-defer 的门槛取整个环：max(8, 环里最旧有效根 7) = 8，B 引用、释放代高于 8 的槽并进遍历。
/// 判别力：丁-defer 门槛改回一圈窗口（`crates/mutations.tsv`「第10g批 丁-defer 门槛改回一圈窗口…」）时门槛是窗口里最旧的有效根，
/// 那几槽不并、记账多于遍历，最后一项红。
pub fn raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring(
    images: &mut PoolCheckerImages,
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let root_before_the_abandoned_one = ABANDONED_TXG_UNDER_THE_RAISED_FLOOR - 1;
    let floor = CheckpointTxg(ABANDONED_TXG_UNDER_THE_RAISED_FLOOR);
    let image =
        pool_with_the_newest_root_of_instance_one_abandoned(ABANDONED_TXG_UNDER_THE_RAISED_FLOOR);
    let image_before_the_hole = {
        let mut devices = devices_of(&image);
        let Mounted {
            output,
            mut allocator,
            current,
        } = mount_writable(&parameters, &mut devices).expect("被抛弃的根造出来之后再挂一次");
        let PoolVersion::WithFile(mut current) = current else {
            panic!("B 那一版带文件");
        };
        // 迭代上界：覆盖写到 B 那一槽下一圈的 txg 之前那一个；跨轮携带的是现行那一版。
        while current.root.checkpoint_txg.0 + 1
            < root_before_the_abandoned_one + ring_length_in_txgs()
        {
            let salt = current.root.checkpoint_txg.0 + 1;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            current = publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_the_version(salt),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
                },
                output.instance,
            )
            .expect("覆盖写");
        }
        pool_of(&devices)
    };
    let (image_with_the_hole, row_txg_of_the_crashed_mount) =
        crash_a_mount_before_its_row_root(&image_before_the_hole);
    assert_eq!(
        row_txg_of_the_crashed_mount,
        root_before_the_abandoned_one + ring_length_in_txgs(),
        "前提：崩在写行之前的那次挂载要写的根槽就是 B 那一槽"
    );
    let mut devices = devices_of(&image_with_the_hole);
    let Mounted {
        mut allocator,
        current,
        ..
    } = mount_writable(&parameters, &mut devices).expect("洞之后的可写挂载做完");
    let PoolVersion::WithFile(mut current) = current else {
        panic!("现行那一版带文件");
    };
    raise_rollback_floor(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        floor,
        ShadowLedger::On,
    )
    .expect("F 抬到 8（不高于上限）");
    let image_after_the_raise = pool_of(&devices);
    let roots = roots_of_the_ring(&image_after_the_raise);
    let below_the_floor_outside_the_window: Vec<CheckpointTxg> = roots
        .valid
        .iter()
        .filter(|root| {
            root.checkpoint_txg < floor
                && !inside_the_one_revolution_window(root, roots.newest.checkpoint_txg)
        })
        .map(|root| root.checkpoint_txg)
        .collect();
    let oldest_valid_root_inside_the_window = roots
        .valid
        .iter()
        .filter(|root| inside_the_one_revolution_window(root, roots.newest.checkpoint_txg))
        .map(|root| root.checkpoint_txg)
        .min();
    assert!(
        below_the_floor_outside_the_window == vec![CheckpointTxg(root_before_the_abandoned_one)]
            && oldest_valid_root_inside_the_window.is_some_and(|oldest| oldest > floor),
        "前提：F 之下、一圈窗口外的有效根恰是 B {below_the_floor_outside_the_window:?}；窗口里最旧的有效根 {oldest_valid_root_inside_the_window:?} 高于 F（最新根 {:?}）",
        roots.newest.checkpoint_txg
    );
    // 丁-defer 门槛取整个环：B 引用、还在 defer 里的槽并进遍历，记账与遍历对得上
    images.hand_out(
        "把 F 抬到 8 之后",
        || image_after_the_raise,
        PoolCheckerExpectation::only_the_named_invariants().and_not_violated("I-3.1"),
    );
}

/// 实例 1 覆盖写到 `last` 那一刻的盘与它最后那次发布；另交最后那次发布之前每块盘两槽系统配置的字节（盘号、偏移、字节）。
pub struct PoolOfInstanceOne {
    pub devices: RingHoleDevices,
    pub last_publish: TransactionOutput,
    pub system_configuration_before_the_last_publish:
        Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)>,
}

/// 照 `crate::common_root_ring_holes::pool_overwritten_through` 同一个做法（mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`），
/// 多留最后那次发布与它之前的系统配置槽。
pub fn pool_of_instance_one_overwritten_through(last: u64) -> PoolOfInstanceOne {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    let mut system_configuration_before_the_last_publish = Vec::new();
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while current.root.checkpoint_txg.0 < last {
        let salt = current.root.checkpoint_txg.0 + 1;
        if salt == last {
            system_configuration_before_the_last_publish = system_configuration_slots_of(&devices);
        }
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            instance,
        )
        .expect("覆盖写");
    }
    PoolOfInstanceOne {
        devices,
        last_publish: current,
        system_configuration_before_the_last_publish,
    }
}

/// 每块盘两个系统配置槽的全部字节。
pub fn system_configuration_slots_of(
    devices: &RingHoleDevices,
) -> Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> {
    let spacing = u64::from(
        parameters_of_the_ring_hole_pools()
            .geometry
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut saved = Vec::new();
    // 迭代上界是盘数 × 每块盘两槽；没有提前出口。
    for (identity, device) in devices {
        for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = DeviceOffsetInBytes(slot * spacing);
            let mut bytes = vec![0u8; slot_bytes];
            device.read_at(offset, &mut bytes).expect("读系统配置槽");
            saved.push((*identity, offset, bytes));
        }
    }
    saved
}

pub fn write_bytes(
    devices: &mut RingHoleDevices,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    bytes: &[u8],
) {
    devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
        .write_at(offset, bytes, WriteDurability::Plain)
        .expect("写进内存盘");
}

/// 崩溃恢复造一条被抛弃的根（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 同一形，换到 12 槽小环上）：
/// 实例 1 最新那次发布（txg `last`）的系统配置轮换没落（每块盘两槽退回它之前的字节）；下一次可写挂载时它的根槽与数据单元一时读不出（清零），
/// 择根落到前一条根、它那条记录施加前验点名单元失败、不施加，实例 2 写行与暖机；挂载交回之后把它的根槽原样写回（数据单元不写回：那几槽实例 2 可能已经用了）。
/// 交回之后的盘面：txg `last` 的根读得出、按实例 2 的实例表被抛弃。
pub fn pool_with_the_newest_root_of_instance_one_abandoned(last: u64) -> MemoryPool {
    let parameters = parameters_of_the_ring_hole_pools();
    let PoolOfInstanceOne {
        mut devices,
        last_publish,
        system_configuration_before_the_last_publish,
    } = pool_of_instance_one_overwritten_through(last);
    for (device, offset, bytes) in &system_configuration_before_the_last_publish {
        write_bytes(&mut devices, *device, *offset, bytes);
    }
    let target = target_for_publish(
        last_publish.root.checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let mut root_slot = vec![0u8; root_slot_bytes];
    devices
        .iter()
        .find(|(identity, _)| *identity == root_slot_device)
        .expect("池里有这块盘")
        .1
        .read_at(root_slot_offset, &mut root_slot)
        .expect("暂存最新那条根的根槽");
    write_bytes(
        &mut devices,
        root_slot_device,
        root_slot_offset,
        &vec![0u8; root_slot_bytes],
    );
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    for location in last_publish
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
    {
        write_bytes(
            &mut devices,
            location.device,
            location.slot.to_device_offset(),
            &vec![0u8; data_unit_bytes],
        );
    }
    let mounted =
        mount_writable(&parameters, &mut devices).expect("择根落到前一条根，照常可写挂载");
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(last - 1)),
        "前提：恢复落到 txg {} 那一版，实例 2",
        last - 1
    );
    write_bytes(&mut devices, root_slot_device, root_slot_offset, &root_slot);
    pool_of(&devices)
}
