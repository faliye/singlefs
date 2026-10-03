//! 挂载那一刻读不出的根环槽，进程内成了一圈窗口里最小的那个时先重读（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；
//! `research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段，攻方 K2-e `unreadable_slot_i31.rs` 的
//! `u31-s4-9-mount.out` 那一形搬到 mkfs 默认的 24 槽环上）。
//!
//! 盘面：实例 1 发到 txg 23，txg 24 那次发布崩在「记录已持久、根槽与之后的系统配置轮换没持久」：txg 24 落到的区域 0 槽 0 里留着 mkfs 写的
//! txg 0 的根。可写挂载（实例 2）那一刻区域 0 槽 0 读不出（设备报读错），挂载之后读得出。挂载按记录施加出 txg 24 那一版，根环表把那一槽记成
//! 「挂载时读不出、能住的最大 txg 是 24」。
//!
//! - 照最大 txg 算（改之前）：写 txg 47 那一版时窗口是 txg > 23，txg 25–47 是这个进程写的有效根，那一槽按 24 算、成了最小的，门槛停在 24；
//!   池级 checker 读盘，那一槽里是窗口外的 txg 0，窗口里最旧的有效根是 25 ⇒ 记账比遍历多算释放代 25 的那几个槽，I-3.1 红。
//! - E2：记 txg 47 那条根之前那一槽成了窗口里最小的那个，先重读：读出 txg 0 的根（按实例表有效、在窗口外）⇒ 那一槽当没有，门槛取 25，与 checker 对得上。
//!   txg 46 时窗口里还有 txg 23 的根，比 24 旧，不重读，那一槽仍按 24 记着。
mod common;

use common::{
    build_pool, crash_state_devices, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, system_configuration_slots_of_every_device, unreadable_root_slot_of,
    with_unreadable_ranges, write_back_system_configuration_slots, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

/// 根槽没落的那次发布的 txg：它落到的区域 0 槽 0 里留着 mkfs 写的 txg 0 的根（24 槽的环）。
const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 24;
/// 那一槽在窗口里成了最小的那个的那一版：24 + 3S − 1。
const TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW: u64 = 47;
/// 一次覆盖写的内容长度：一个数据单元装得下。
const OVERWRITE_BYTES: usize = 3000;
/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
}

fn violations_of(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| INVARIANTS_THE_REREAD_KEEPS_GREEN.contains(invariant))
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

#[test]
fn a_root_ring_slot_unreadable_only_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window_and_the_allocated_statistic_holds(
) {
    let mut pool = build_pool("root-ring-slot-unreadable-at-mount-reread");
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
    let unreadable_slots_of_the_table =
        |mounted_allocator: &singlefs_core::allocator::PoolAllocator| {
            mounted_allocator
                .root_ring_occupancy()
                .expect("可写挂载装了根环表")
                .ring_slots_unreadable_at_mount()
        };
    assert_eq!(
        unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
        Some(&CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)),
        "根环表把那一槽记成挂载时读不出、能住的最大 txg 24"
    );
    let PoolVersion::WithFile(mut current) = mounted.current else {
        panic!("txg 24 那一版带文件");
    };
    let publish_parameters = parameters();
    // 迭代上界：覆盖写到 txg 48（那一槽被这个进程写掉的那一版）；跨轮携带的是现行那一版。
    while current.root.checkpoint_txg.0 < TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW + 1
    {
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
        if checkpoint_txg == TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW - 1 {
            assert_eq!(
                unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
                Some(&CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)),
                "txg 46：窗口里还有比 24 旧的 txg 23，那一槽没重读、仍按 24 记着"
            );
        }
        if checkpoint_txg == TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW {
            assert_eq!(
                unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
                None,
                "txg 47：那一槽成了窗口里最小的那个，重读出 txg 0 的根，不再按 24 记"
            );
        }
        // 盘上此刻的样子：崩溃状态加上挂载以来录下的全部写（读故障只在读那一侧，写原样落到里面那块盘上）。
        let mut image = crash_image.clone();
        image.apply(&stream.retained_operations());
        assert_eq!(
            violations_of(&image),
            Vec::<String>::new(),
            "txg {checkpoint_txg} 之后：那一槽重读之后按读出来的根算，回收门槛与 checker 的窗口对得上"
        );
    }
}
