//! 根环上有洞之后的回收门槛取一圈窗口（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」，
//! 用户 2026-10-02 定；`research/prompts/m3-ring-r3-main-verification.md` 第二节第 3–5 行）。
//!
//! 盘面（与崩溃注入种子 0 第 135 段那一形同形，`/tmp/claude-1000/inject6-findings/report.md` 第三节）：实例 1 在 24 槽的根环上发到 txg 30，
//! txg 31 那次发布崩在「记录已持久、根槽没持久」（根槽与之后的系统配置轮换都没落：那一槽里留着上一圈的 txg 7 的根）。
//! 可写挂载（实例 2）恢复由记录施加出 txg 31 那一版，写行 txg 32、暖机 txg 33、34 盖掉 txg 8、9、10 的槽；之后覆盖写 txg 35、36、37。
//! txg 7 的根一直留在 txg 31 那一槽里，是环里最旧的有效根，而 txg 8–11 已经离开根环。
//!
//! - 分配器（进程内每记一条根，锚在这条根）：txg 35 起窗口是 txg > 11，txg 7 在窗口外，门槛跟着窗口里最旧的有效根往前走；
//!   照「环里最旧有效根」取时门槛卡在 7，只被 txg 8–11 那几版引用、已释放的槽回收不了、仍算已分配，而根环里已没有根引用它们 ⇒ I-3.1 红。
//! - 池级 checker 的候选过滤：txg 7 在镜像的窗口外（N = 最新根），不当候选走；照旧当候选走时，它引用的槽已被窗口门槛回收、可能已被复用 ⇒
//!   I-3.1 / I-2.1 / I-7.4 红。
//!
//! 每次覆盖写之后在盘上跑池级 checker，判这几条不红。
mod common;

use common::{
    build_pool, crash_state_devices, memory_pool_of_sparse_devices,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    system_configuration_slots_of_every_device, write_back_system_configuration_slots,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::SharedStream;

/// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;
/// 洞那一槽里留着的上一圈的根。
const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;
/// 一次覆盖写的内容长度：一个数据单元装得下。
const OVERWRITE_BYTES: usize = 3000;
/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
const INVARIANTS_THE_WINDOW_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
}

#[test]
fn reclaiming_after_a_hole_in_the_root_ring_follows_the_one_revolution_window_and_keeps_the_allocated_statistic(
) {
    let mut pool = build_pool("one-revolution-window-after-a-hole");
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    assert_eq!(
        3 * slots_per_region.count(),
        TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST - TXG_OF_THE_STALE_ROOT_IN_THE_HOLE,
        "根环 24 槽：txg 31 与 txg 7 同槽"
    );
    // 迭代上界：txg 4 到 30，27 次覆盖写。
    for checkpoint_txg in 4..TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST {
        let output = overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &content_of_txg(checkpoint_txg),
        );
        assert_eq!(output.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
    }
    // txg 31 那次发布之前：它要写的那个根槽里是 txg 7 的根，每块盘两槽系统配置是 txg 30 那次轮换之后的样子。
    let target = target_for_publish(
        CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
        slots_per_region,
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
    // 崩溃状态：txg 31 的单元与记录都落了，根槽与之后的系统配置轮换没落（D16（发布语义） 已定项 7 的持久顺序）。
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
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.checkpoint_txg,
            mounted.current.root().checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(31), CheckpointTxg(34)),
        "恢复由记录施加出 txg 31；写行 txg 32、暖机 txg 33、34"
    );
    let PoolVersion::WithFile(mut current) = mounted.current else {
        panic!("txg 31 那一版带文件");
    };
    let publish_parameters = parameters();
    // 迭代上界：覆盖写三次（txg 35、36、37）；跨轮携带的是现行那一版。
    for checkpoint_txg in 35..=37u64 {
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
        let image = memory_pool_of_sparse_devices(&devices);
        let violations: Vec<String> = check_pool_image(&image)
            .into_iter()
            .filter(|(invariant, _)| INVARIANTS_THE_WINDOW_KEEPS_GREEN.contains(invariant))
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
            })
            .collect();
        assert_eq!(
            violations,
            Vec::<String>::new(),
            "txg {checkpoint_txg} 之后：洞那一槽里的 txg 7 在窗口外，回收门槛与候选集都跟着窗口走"
        );
    }
}
