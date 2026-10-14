//! harness 测试文件 `crates/singlefs-harness/tests/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, crash_state_devices, disk_snapshot, memory_pool_of_sparse_devices,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    system_configuration_slots_of_every_device, write_back_system_configuration_slots,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::SharedStream;

/// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
pub const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;

/// 洞那一槽里留着的上一圈的根：回退的目标。
pub const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;

/// 一次覆盖写的内容长度：一个数据单元装得下。
pub const OVERWRITE_BYTES: usize = 3000;

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
pub const INVARIANTS_THE_ROLLBACK_KEEPS_GREEN: [&str; 5] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

pub fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
}

/// 场景：上面那份盘面上挂着时回退到环里那条 txg 7 的根（实例 1）。
/// 预期：回退做成，向前发一条 txg 35 的根；冷启动择到它、读回 txg 7 那一版的内容；回退之后盘上那几条不变量不红。
/// 改之前候选集判一圈窗口，在任何写之前拒（`TargetNotACandidate { exclusion: OutsideTheOneRevolutionWindow }`）。
pub fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("rollback-target-outside-the-one-revolution-window");
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    // 迭代上界：txg 4 到 30，27 次覆盖写。
    for checkpoint_txg in 4..TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST {
        overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &content_of_txg(checkpoint_txg),
        );
    }
    let target = target_for_publish(
        CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
        slots_per_region,
    );
    assert_eq!(
        target,
        target_for_publish(
            CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
            slots_per_region
        ),
        "根环 24 槽：txg 31 与 txg 7 同槽"
    );
    let device_of_the_slot =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset_of_the_slot =
        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing);
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
    let mut crash_image = pool.memory_pool();
    crash_image
        .devices
        .get_mut(&device_of_the_slot)
        .expect("有这块盘")
        .write(offset_of_the_slot, &stale_root_slot);
    write_back_system_configuration_slots(&mut crash_image, &system_configuration_before);

    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&crash_image, &[], &[], &stream);
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let PoolVersion::WithFile(mut current) = mounted.current else {
        panic!("txg 31 那一版带文件");
    };
    assert_eq!(
        (mounted.output.instance, current.root.checkpoint_txg),
        (InstanceGeneration(2), CheckpointTxg(34)),
        "写行 txg 32、暖机 txg 33、34"
    );
    let before = disk_snapshot(&memory_pool_of_sparse_devices(&devices), &stream);
    assert!(
        before.readable_roots.iter().any(|root| {
            (root.instance, root.checkpoint_txg)
                == (
                    InstanceGeneration(1),
                    CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
                )
        }),
        "前提：txg 7 的根还在环里、读得出"
    );
    let rolled_back = roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut mounted.allocator,
        &mut current,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        },
    );
    assert!(
        rolled_back.is_ok(),
        "txg 7 在环里、有效、带文件：是候选，回退做成：{rolled_back:?}"
    );
    assert_eq!(
        current.root.checkpoint_txg,
        CheckpointTxg(35),
        "回退向前发一条 txg 35 的根"
    );
    let image = memory_pool_of_sparse_devices(&devices);
    images.hand_out(
        "回退到洞里 txg 7 那条根之后",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_ROLLBACK_KEEPS_GREEN),
    );
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(35)),
            content: content_of_txg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        },
        "冷启动择 txg 35 那条根、读回 txg 7 那一版的内容"
    );
}
