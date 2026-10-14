//! harness 测试文件 `crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, crash_state_devices, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, system_configuration_slots_of_every_device, unreadable_root_slot_of,
    with_unreadable_ranges, write_back_system_configuration_slots, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_system_configuration, readable_roots_with_ring_slots};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

/// 根槽没落的那次发布的 txg：它落到的区域 0 槽 0 里留着 mkfs 写的 txg 0 的根（24 槽的环）。
pub const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 24;

/// 这个进程写掉那一槽的那一版：24 + 3S。
pub const TXG_THAT_OVERWRITES_THE_SLOT: u64 = 48;

/// 一次覆盖写的内容长度：一个数据单元装得下。
pub const OVERWRITE_BYTES: usize = 3000;

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
pub const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

pub fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
}

/// 交 checker 档：重读不越过的那几条（[`INVARIANTS_THE_REREAD_KEEPS_GREEN`]）一条都不红。
pub fn hand_out_expecting_the_reread_keeps_its_invariants_green(
    images: &mut PoolCheckerImages,
    label: &str,
    image: MemoryPool,
) {
    images.hand_out(
        label,
        || image,
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_REREAD_KEEPS_GREEN),
    );
}

/// mkfs 写的 txg 0 那一版引用的单元（树表、实例表两条指针的两个落点；那一版的账就由这两条指针算，`mount::format_time_allocator`）里
/// 分配器上空着（可再分配）的槽：(盘, 槽)。
pub fn slots_of_the_mkfs_root_left_free(
    image: &MemoryPool,
    allocator: &PoolAllocator,
) -> Vec<(u32, u64)> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = &system_configuration.immutable;
    let mkfs_root = readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|(_, root)| root)
    .find(|root| root.checkpoint_txg == CheckpointTxg(0))
    .expect("txg 0 的根还在环里、读得出");
    [mkfs_root.tree_table, mkfs_root.instance_table]
        .iter()
        .flat_map(|pointer| pointer.locations)
        .filter(|location| {
            allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == location.device)
                .expect("落点的盘在分配器里")
                .is_free(location.slot)
        })
        .map(|location| (location.device.0, location.slot.0))
        .collect()
}

/// 上面那份盘面（崩溃状态）与 txg 24 落到的那一槽（区域 0 槽 0，里面留着 mkfs 写的 txg 0 的根）。`tag` 给池的镜像起名。
pub fn crash_state_with_the_mkfs_root_left_in_the_slot_of_txg_24(
    tag: &str,
) -> (MemoryPool, RootRingSlot) {
    let mut pool = build_pool(tag);
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    assert_eq!(
        3 * slots_per_region.count(),
        TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST,
        "根环 24 槽：txg 24 与 mkfs 的 txg 0 同槽"
    );
    // 迭代上界：txg 4 到 23，20 次覆盖写。
    for checkpoint_txg in 4..TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST {
        let output = overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &content_of_txg(checkpoint_txg),
        );
        assert_eq!(output.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
    }
    let hidden_slot = target_for_publish(
        CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
        slots_per_region,
    );
    let device_of_the_slot =
        parameters().region_devices[usize::try_from(hidden_slot.region).expect("区域号")];
    let offset_of_the_slot = slot_offset(
        hidden_slot,
        parameters().geometry.fixed_structure_slot_spacing,
    );
    let mut stale_root_slot =
        vec![0u8; usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽")];
    {
        let devices = pool.devices.as_ref().expect("镜像开着");
        let (_, device) = devices
            .iter()
            .find(|(identity, _)| *identity == device_of_the_slot)
            .expect("池里有这块盘");
        device
            .read_at(offset_of_the_slot, &mut stale_root_slot)
            .expect("读那个根槽");
    }
    let system_configuration_before = system_configuration_slots_of_every_device(&pool);
    overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_txg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
    );
    // 崩溃状态：txg 24 的单元与记录都落了，根槽与之后的系统配置轮换没落（D16（发布语义） 已定项 7 的持久顺序）。
    let mut crash_image = pool.memory_pool();
    crash_image
        .devices
        .get_mut(&device_of_the_slot)
        .expect("有这块盘")
        .write(offset_of_the_slot, &stale_root_slot);
    write_back_system_configuration_slots(&mut crash_image, &system_configuration_before);

    (crash_image, hidden_slot)
}

/// 场景：上面那份盘面上可写挂载（实例 2），那一槽只在挂载那一次读不出；之后覆盖写到 txg 48（这个进程写掉那一槽）。
/// 预期：挂载交回的根环表把那一槽记成挂载时读坏、钉在可能住过的最旧 txg 0（能住的最大 txg 24）；挂载之后第一次覆盖写记根之前重读，
/// 读出 txg 0 的根，那一槽不再按读坏记、算这个进程知道住着根的槽。挂载之后与每一次覆盖写之后：txg 0 那一版引用的单元在分配器上
/// 一个都不空闲，盘上那几条不变量不红。
pub fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants(
    images: &mut PoolCheckerImages,
) {
    let (crash_image, hidden_slot) = crash_state_with_the_mkfs_root_left_in_the_slot_of_txg_24(
        "root-ring-slot-unreadable-at-mount-reread",
    );
    let stream = SharedStream::retaining_contents();
    let unreadable_at_mount = SharedUnreadableRanges::new(
        vec![unreadable_root_slot_of(
            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
            FailingReadsOfARange::Every,
        )],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(
        crash_state_devices(&crash_image, &[], &[], &stream),
        &unreadable_at_mount,
    );
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    // 只在挂载那一次读错：挂载之后那一槽读得出。
    unreadable_at_mount.lift();
    assert!(
        unreadable_at_mount.failed_reads() > 0,
        "挂载那一次读那一槽读错了"
    );
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(24)),
        "恢复由记录施加出 txg 24"
    );
    let image_now = || {
        let mut image = crash_image.clone();
        image.apply(&stream.retained_operations());
        image
    };
    assert_eq!(
        mounted
            .allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .ring_slots_bad_at_mount()
            .get(&hidden_slot),
        Some(&TxgsARingSlotBadAtMountCanHold::new(
            CheckpointTxg(0),
            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)
        )),
        "根环表把那一槽记成挂载时读坏：钉在可能住过的最旧 txg 0，能住的最大 txg 24"
    );
    let after_the_mount = image_now();
    assert_eq!(
        slots_of_the_mkfs_root_left_free(&after_the_mount, &mounted.allocator),
        Vec::new(),
        "挂载之后：txg 0 引用的单元一个都不空闲"
    );
    hand_out_expecting_the_reread_keeps_its_invariants_green(images, "挂载之后", after_the_mount);
    let PoolVersion::WithFile(mut current) = mounted.current else {
        panic!("txg 24 那一版带文件");
    };
    let first_txg_overwritten = current.root.checkpoint_txg.0 + 1;
    let publish_parameters = parameters();
    // 迭代上界：覆盖写到 txg 48（那一槽被这个进程写掉的那一版）；跨轮携带的是现行那一版。
    while current.root.checkpoint_txg.0 < TXG_THAT_OVERWRITES_THE_SLOT {
        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
        current = {
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut mounted.allocator,
                &current,
                FirstFile {
                    content: &content_of_txg(checkpoint_txg),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
                },
                InstanceGeneration(2),
            )
            .expect("覆盖写")
        };
        assert_eq!(current.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
        if checkpoint_txg == first_txg_overwritten {
            let occupancy = mounted
                .allocator
                .root_ring_occupancy()
                .expect("可写挂载装了根环表");
            assert_eq!(
                (
                    occupancy.ring_slots_bad_at_mount().get(&hidden_slot).copied(),
                    occupancy
                        .ring_slots_known_to_hold_a_root()
                        .contains(&hidden_slot)
                ),
                (None, true),
                "txg {checkpoint_txg}：挂载之后第一次记根，那一槽钉在 0、是环里最小的那个，重读出 txg 0 的根，算这个进程知道住着根的槽"
            );
        }
        let image = image_now();
        let left_free = if checkpoint_txg < TXG_THAT_OVERWRITES_THE_SLOT {
            slots_of_the_mkfs_root_left_free(&image, &mounted.allocator)
        } else {
            Vec::new()
        };
        assert_eq!(
            left_free,
            Vec::new(),
            "txg {checkpoint_txg} 之后：txg 0 引用的单元一个都不空闲（txg 48 写掉那一槽之前）"
        );
        hand_out_expecting_the_reread_keeps_its_invariants_green(
            images,
            &format!("覆盖写到 txg {checkpoint_txg} 之后"),
            image,
        );
    }
}

/// 场景：同一份盘面上可写挂载（实例 2），那一槽从挂载起每次读都报成功、字节全 0（盘上那一槽里是 txg 0 的根）；之后覆盖写到 txg 48
/// （这个进程写掉那一槽）。
/// 预期：挂载交回的根环表把那一槽记成挂载时读回全零、钉在它可能住过的最旧 txg 0，不进准入抬 F 的上限要并的那几个槽；挂载之后每次记根
/// 重读那一槽都读回全零，不算确认为空，照旧钉着、不算这个进程知道住着根的槽。挂载之后与每一次覆盖写之后：txg 48 写掉那一槽之前，txg 0
/// 那一版引用的单元在分配器上一个都不空闲，盘上那几条不变量不红（池级 checker 读盘上完好的 txg 0，环里最旧有效根是 0，与钉着的 0 对得上）。
pub fn a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it(
    images: &mut PoolCheckerImages,
) {
    let (crash_image, hidden_slot) = crash_state_with_the_mkfs_root_left_in_the_slot_of_txg_24(
        "root-ring-slot-reading-back-all-zero-since-the-mount",
    );
    let stream = SharedStream::retaining_contents();
    let reading_back_all_zero = SharedUnreadableRanges::new(
        vec![unreadable_root_slot_of(
            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
            FailingReadsOfARange::Every,
        )],
        UnreadableRangeReadBack::Zeros,
    );
    let mut devices = with_unreadable_ranges(
        crash_state_devices(&crash_image, &[], &[], &stream),
        &reading_back_all_zero,
    );
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert!(
        reading_back_all_zero.failed_reads() > 0,
        "挂载那一次读那一槽读回了全零"
    );
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(24)),
        "恢复由记录施加出 txg 24"
    );
    let image_now = || {
        let mut image = crash_image.clone();
        image.apply(&stream.retained_operations());
        image
    };
    // 那一槽在根环表里怎么记着：(读回全零钉着的 txg, 是否进准入抬 F 的上限要并的槽, 是否算这个进程知道住着根的槽)。
    let how_the_table_records_the_slot = |allocator: &PoolAllocator| {
        let occupancy = allocator.root_ring_occupancy().expect("可写挂载装了根环表");
        (
            occupancy
                .ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
                .get(&hidden_slot)
                .copied(),
            occupancy.ring_slots_bad_at_mount().contains_key(&hidden_slot),
            occupancy
                .ring_slots_known_to_hold_a_root()
                .contains(&hidden_slot),
        )
    };
    assert_eq!(
        how_the_table_records_the_slot(&mounted.allocator),
        (Some(CheckpointTxg(0)), false, false),
        "根环表把那一槽记成挂载时读回全零：钉在可能住过的最旧 txg 0，不进上限要并的槽"
    );
    let after_the_mount = image_now();
    assert_eq!(
        slots_of_the_mkfs_root_left_free(&after_the_mount, &mounted.allocator),
        Vec::new(),
        "挂载之后：txg 0 引用的单元一个都不空闲"
    );
    hand_out_expecting_the_reread_keeps_its_invariants_green(images, "挂载之后", after_the_mount);
    let PoolVersion::WithFile(mut current) = mounted.current else {
        panic!("txg 24 那一版带文件");
    };
    let publish_parameters = parameters();
    // 迭代上界：覆盖写到 txg 48（那一槽被这个进程写掉的那一版）；跨轮携带的是现行那一版。
    while current.root.checkpoint_txg.0 < TXG_THAT_OVERWRITES_THE_SLOT {
        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
        let reads_of_the_slot_before = reading_back_all_zero.reads_of_range(0);
        current = {
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut mounted.allocator,
                &current,
                FirstFile {
                    content: &content_of_txg(checkpoint_txg),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
                },
                InstanceGeneration(2),
            )
            .expect("覆盖写")
        };
        assert_eq!(current.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
        let image = image_now();
        if checkpoint_txg < TXG_THAT_OVERWRITES_THE_SLOT {
            assert_eq!(
                (
                    reading_back_all_zero.reads_of_range(0) > reads_of_the_slot_before,
                    how_the_table_records_the_slot(&mounted.allocator)
                ),
                (true, (Some(CheckpointTxg(0)), false, false)),
                "txg {checkpoint_txg}：记根之前重读那一槽、读回全零，照旧钉在 0"
            );
            assert_eq!(
                slots_of_the_mkfs_root_left_free(&image, &mounted.allocator),
                Vec::new(),
                "txg {checkpoint_txg} 之后：txg 0 引用的单元一个都不空闲"
            );
        } else {
            assert_eq!(
                how_the_table_records_the_slot(&mounted.allocator),
                (None, false, true),
                "txg {checkpoint_txg}：这个进程写掉那一槽，不再钉着、算知道住着根的槽"
            );
        }
        hand_out_expecting_the_reread_keeps_its_invariants_green(
            images,
            &format!("覆盖写到 txg {checkpoint_txg} 之后"),
            image,
        );
    }
}
