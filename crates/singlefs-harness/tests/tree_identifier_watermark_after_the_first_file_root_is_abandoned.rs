//! 第一个文件那条根被崩溃恢复抛弃之后，新实例写行那次发布带的树 ID 水位（D8（核心索引结构） 已定项 8 ②「根环里全部根记录该字段的 max」；
//! 欠账 C511 第 3 步）：恢复落到暖机那一版（不带文件、水位停在 mkfs 那一档），而被抛弃的那条根已经把水位推到发第一个文件之后那一档。
//! 写行那次发布照条款取根环里的 max，不沿落到的那一版带——沿它带，新实例再发第一个文件时会把被抛弃根已经发出去的树 ID 再发一遍。

mod common;

use common::{build_pool, parameters, BuiltPool, Recorded};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_format::{
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
};

fn index_of_device(devices: &[(DeviceIdentity, Recorded)], identity: DeviceIdentity) -> usize {
    devices
        .iter()
        .position(|(candidate, _)| *candidate == identity)
        .expect("池里有这块盘")
}

/// 照 `tests/common` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 的造法另写一份：那一份要落到的那一版带文件
/// （挂载之后把池的现行版本换成它的文件版本），这里落到的是暖机那一版、不带文件。
/// 进程退出之后，第一个文件那一版（`pool.output`）的根槽与它的数据单元（两份）暂时读不出（暂存、清零），它那次发布之后轮换写的系统配置槽
/// （每块盘一槽）写回成轮换之前的那一代（系统配置没见证到它，C554 乙罩不到），重开可写挂载——择根落到暖机那条根，第一个文件那条记录
/// 施加前验点名单元失败、不施加；新实例写行与暖机。再把暂存的字节原样写回：第一个文件那条根又读得出，按新实例的实例表判是被抛弃的。
/// 交回那次挂载。
fn abandon_the_first_file_root_by_a_recovery_that_lands_on_the_warm_up_root(
    pool: &mut BuiltPool,
) -> Mounted {
    let first_file = pool.output.clone();
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        first_file.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let mut devices = pool.reopen_recorded();
    let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes))
        .chain(
            first_file
                .data_pointers
                .iter()
                .flat_map(|pointer| pointer.locations)
                .map(|location| {
                    (
                        location.device,
                        location.slot.to_device_offset(),
                        data_unit_bytes,
                    )
                }),
        );
    let mut saved: Vec<(
        DeviceIdentity,
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )> = Vec::new();
    for (identity, offset, length) in places_to_hide {
        let index = index_of_device(&devices, identity);
        let mut bytes = vec![0u8; length];
        devices[index].1.read_at(offset, &mut bytes).expect("暂存");
        devices[index]
            .1
            .write_at(offset, &vec![0u8; length], WriteDurability::Plain)
            .expect("清零");
        saved.push((identity, offset, bytes));
    }
    // 系统配置没见证到第一个文件那一版：轮换写那一槽（每块盘世代号最大的那一槽）换成另一槽的内容、世代号改成 g − 2，
    // 两槽都自证得过、见证值停在第一个文件之前（写法与理由同 `tests/common` 那一份）。
    let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let rotation_not_written_of_each_device: Vec<(
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )> = devices
        .iter()
        .map(|(identity, _)| {
            let mut verified_slots = singlefs_core::recovery::verified_system_configuration_slots(
                devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &publish_parameters.filesystem_identifier,
            );
            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest_slot] =
                <[_; 2]>::try_from(verified_slots).expect("造被抛弃的根之前每块盘两槽都自证得过");
            let newest_generation = newest_slot.quantities.slot_generation;
            let mut rotation_not_written = older;
            rotation_not_written.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("第一个文件之前至少还有两次轮换");
            (
                singlefs_core::address::DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                ),
                rotation_not_written.to_slot(),
            )
        })
        .collect();
    for ((_, device), (newest_slot, rotation_not_written)) in
        devices.iter_mut().zip(rotation_not_written_of_each_device)
    {
        device
            .write_at(newest_slot, &rotation_not_written, WriteDurability::Plain)
            .expect("见证第一个文件的系统配置槽写回成轮换之前的那一代");
    }
    let mounted = mount_writable(&publish_parameters, &mut devices)
        .expect("第一个文件那条根与它的数据单元读不出：择根落到暖机那条根，照常可写挂载");
    for (identity, offset, bytes) in &saved {
        let index = index_of_device(&devices, *identity);
        devices[index]
            .1
            .write_at(*offset, bytes, WriteDurability::Plain)
            .expect("原样写回");
    }
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    mounted
}

/// 盘面：mkfs、取号、暖机（txg 1、2）、发第一个文件（txg 3，水位推到发第一个文件之后那一档）；第一个文件那条根被一次落到暖机那条根的
/// 崩溃恢复抛弃，新实例可写挂载。写行那次发布带的水位等于根环里读得出的根（被抛弃的那条已写回、读得出）上水位的最大值，
/// 也就是被抛弃那条根的水位，高过落到的暖机那一版带的。
#[test]
fn the_row_publish_after_the_first_file_root_is_abandoned_carries_the_highest_watermark_in_the_ring(
) {
    let mut pool = build_pool("tree_identifier_watermark_after_the_first_file_root_is_abandoned");
    let first_file_root = pool.output.root;
    let mounted =
        abandon_the_first_file_root_by_a_recovery_that_lands_on_the_warm_up_root(&mut pool);
    let landed_root = &mounted.output.effective_root;
    assert_eq!(
        (
            first_file_root.checkpoint_txg,
            first_file_root.tree_identifier_watermark,
            landed_root.checkpoint_txg,
            landed_root.tree_identifier_watermark,
        ),
        (
            CheckpointTxg(3),
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
            CheckpointTxg(2),
            TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        ),
        "第一个文件那一版 txg 3、水位在发第一个文件之后那一档；恢复落到暖机那一版 txg 2、水位停在 mkfs 那一档"
    );
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let roots_in_the_ring = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    assert!(
        roots_in_the_ring
            .iter()
            .any(|root| root.checkpoint_txg == first_file_root.checkpoint_txg
                && root.instance == first_file_root.instance),
        "被抛弃的第一个文件那条根写回之后在根环里读得出"
    );
    let highest_watermark_in_the_ring = roots_in_the_ring
        .iter()
        .map(|root| root.tree_identifier_watermark)
        .max()
        .expect("根环里有根");
    let row_publish_root = mounted.output.row_publish.root();
    assert_eq!(
        (
            row_publish_root.tree_identifier_watermark,
            highest_watermark_in_the_ring,
        ),
        (
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
        ),
        "写行那次发布（txg {}）带的水位等于根环里读得出的根上水位的最大值（被抛弃的第一个文件那条根的），不是落到的暖机那一版的 {}",
        row_publish_root.checkpoint_txg.0,
        TREE_IDENTIFIER_WATERMARK_AT_MKFS
    );
}
