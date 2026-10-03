//! 回退候选集只收 txg 落在一圈窗口里的根（D16（发布语义） 已定项 1「回退候选集」：按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ txg 落在一圈窗口里；
//! D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合；用户 2026-10-03 定照 7-2 报告第四节做）。
//!
//! 盘面同 `one_revolution_window_after_a_hole_in_the_root_ring.rs`：实例 1 在 24 槽的根环上发到 txg 30，txg 31 那次发布崩在「记录已持久、根槽没持久」，
//! 那一槽里留着上一圈的 txg 7 的根；可写挂载（实例 2）恢复由记录施加出 txg 31，写行 32、暖机 33、34。txg 7 的根读得出、按实例表有效、
//! txg ≥ F_生效（F 没抬过，是 0）、带文件，只有一样不满足：现行那一版 txg 34 的窗口是 txg > 10，txg 7 在窗口外——窗口已许可它引用的单元被复用。
mod common;

use common::{
    build_pool, crash_state_devices, disk_snapshot, memory_pool_of_sparse_devices,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    system_configuration_slots_of_every_device, write_back_system_configuration_slots,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, RollbackCandidateExclusion, RollbackError,
    RollbackTarget,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::SharedStream;

/// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;
/// 洞那一槽里留着的上一圈的根：回退的目标。
const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;
/// 一次覆盖写的内容长度：一个数据单元装得下。
const OVERWRITE_BYTES: usize = 3000;

fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
}

/// 场景：上面那份盘面上挂着时回退到环里那条 txg 7 的根（实例 1）。
/// 预期：在任何写之前拒，报 `TargetNotACandidate { exclusion: OutsideTheOneRevolutionWindow }`；盘上逐字节不变（系统配置槽、根环里自证过的根、录制流步数）。
/// 改之前候选集不看窗口，txg 7 那条根被当成候选往下走。
#[test]
fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_is_refused_before_any_write(
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
        "txg 7 的根还在环里、读得出"
    );
    let refusal = roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut mounted.allocator,
        &mut current,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        },
    )
    .expect_err("txg 7 在现行那一版的一圈窗口外，不是候选");
    assert!(
        matches!(
            refusal,
            RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::OutsideTheOneRevolutionWindow,
                ..
            }
        ),
        "报落在一圈窗口外：{refusal:?}"
    );
    assert_eq!(
        disk_snapshot(&memory_pool_of_sparse_devices(&devices), &stream),
        before,
        "在任何写之前拒：盘上逐字节不变"
    );
}
